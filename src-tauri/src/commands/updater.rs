use crate::env_helper;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteReleaseInfo {
    pub tag_name: String,
    pub body: String,
    pub html_url: String,
    pub published_at: String,
}

#[tauri::command]
pub async fn open_url_in_browser(url: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let status = env_helper::create_silent_command("open")
            .arg(&url)
            .status()
            .map_err(|e| format!("无法打开浏览器: {}", e))?;
        Ok(status.success())
    }

    #[cfg(target_os = "windows")]
    {
        let quoted_url = format!("\"{}\"", url.replace('"', ""));
        let status = env_helper::create_silent_command("cmd")
            .args(["/c", "start", "", &quoted_url])
            .status()
            .map_err(|e| format!("无法打开浏览器: {}", e))?;
        Ok(status.success())
    }

    #[cfg(target_os = "linux")]
    {
        let status = env_helper::create_silent_command("xdg-open")
            .arg(&url)
            .status()
            .map_err(|e| format!("无法打开浏览器: {}", e))?;
        Ok(status.success())
    }
}

#[tauri::command]
pub async fn open_path_in_file_manager(path: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let status = env_helper::create_silent_command("open")
            .args(["-R", &path])
            .status()
            .map_err(|e| format!("无法在访达中显示: {}", e))?;
        Ok(status.success())
    }

    #[cfg(target_os = "windows")]
    {
        // Explorer expects `/select,<path>` as a single argument. Passing it
        // as two arguments fails for many paths (especially those with spaces).
        let select_arg = format!("/select,{}", path);
        let status = env_helper::create_silent_command("explorer")
            .arg(select_arg)
            .status()
            .map_err(|e| format!("无法在资源管理器中显示: {}", e))?;
        Ok(status.success())
    }

    #[cfg(target_os = "linux")]
    {
        let status = env_helper::create_silent_command("xdg-open")
            .arg(&path)
            .status()
            .map_err(|e| format!("无法在文件管理器中显示: {}", e))?;
        Ok(status.success())
    }
}

#[tauri::command]
pub async fn open_installer_file(path: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let status = env_helper::create_silent_command("open")
            .arg(&path)
            .status()
            .map_err(|e| format!("无法打开安装包: {}", e))?;
        Ok(status.success())
    }

    #[cfg(target_os = "windows")]
    {
        let quoted_path = format!("\"{}\"", path.replace('"', ""));
        let status = env_helper::create_silent_command("cmd")
            .args(["/c", "start", "", &quoted_path])
            .status()
            .map_err(|e| format!("无法打开安装包: {}", e))?;
        Ok(status.success())
    }

    #[cfg(target_os = "linux")]
    {
        let status = env_helper::create_silent_command("xdg-open")
            .arg(&path)
            .status()
            .map_err(|e| format!("无法打开安装包: {}", e))?;
        Ok(status.success())
    }
}

static PENDING_UPDATE: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

fn download_percent(line: &str) -> Option<u32> {
    line.split_whitespace().find_map(|word| {
        word.strip_suffix('%')?
            .parse::<f32>()
            .ok()
            .map(|value| (value as u32).min(90))
    })
}

#[cfg(target_os = "macos")]
fn application_bundle() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    executable
        .ancestors()
        .find(|p| p.extension().and_then(|s| s.to_str()) == Some("app"))
        .map(|p| p.to_path_buf())
        .filter(|p| !p.to_string_lossy().contains("AppTranslocation"))
        .ok_or("Move EnvHub to Applications before updating".into())
}

/// The old installation is untouched until staging has completed successfully.
#[cfg(any(unix, test))]
fn commit_staged(
    staged: &std::path::Path,
    destination: &std::path::Path,
    backup: &std::path::Path,
) -> Result<(), String> {
    if backup.exists() {
        return Err("Update backup already exists".into());
    }
    let had_original = destination.exists();
    if had_original {
        std::fs::rename(destination, backup).map_err(|e| e.to_string())?;
    }
    if let Err(error) = std::fs::rename(staged, destination) {
        if had_original {
            std::fs::rename(backup, destination).map_err(|restore| {
                format!(
                    "Update failed: {error}; restore failed: {restore}; backup: {}",
                    backup.display()
                )
            })?;
        }
        return Err(format!(
            "Update failed, previous installation restored: {error}"
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn relaunch_application(_app: AppHandle) -> Result<(), String> {
    let _operation = env_helper::begin_operation()?;
    let pending = PENDING_UPDATE
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("No completed update is ready")?;
    if !pending.exists() {
        return Err("The prepared update no longer exists".into());
    }
    #[cfg(target_os = "macos")]
    {
        let quoted = env_helper::shell_quote(&pending.to_string_lossy());
        env_helper::create_silent_command("sh")
            .args(["-c", &format!("sleep 1; open -n -F {quoted}")])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        let quoted = pending.to_string_lossy().replace('\'', "''");
        // Bind to this session's exact installer, never scan Downloads for arbitrary old files.
        let exe = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\'', "''");
        let script = format!("Start-Sleep -Seconds 1; $p = Start-Process -FilePath '{quoted}' -ArgumentList '/S' -Wait -PassThru; if ($p.ExitCode -eq 0) {{ Start-Process -FilePath '{exe}' }}");
        env_helper::create_silent_command("powershell")
            .args(["-NoProfile", "-Command", &script])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        env_helper::create_silent_command(&pending.to_string_lossy())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    std::process::exit(0);
}

#[tauri::command]
pub async fn download_and_install_update(
    app: AppHandle,
    download_url: String,
    version: String,
) -> Result<String, String> {
    let _operation = env_helper::begin_operation()?;
    env_helper::validate_target("envhub", &version)?;
    let prefix = format!("https://github.com/WnJee/EnvHub/releases/download/v{version}/");
    if !download_url.starts_with(&prefix) {
        return Err("Update URL must point to the selected official release".into());
    }
    let filename = download_url
        .strip_prefix(&prefix)
        .ok_or("Invalid release URL")?;
    if filename.contains('/')
        || filename.contains('?')
        || filename.contains('#')
        || filename.contains('%')
        || !filename.starts_with("EnvHub_")
    {
        return Err("Invalid update filename".into());
    }
    let expected = if cfg!(target_os = "macos") {
        ".dmg"
    } else if cfg!(windows) {
        "-setup.exe"
    } else {
        ".AppImage"
    };
    if !filename.ends_with(expected) {
        return Err("The selected asset is not a supported installer for this platform".into());
    }
    #[cfg(target_os = "linux")]
    let destination = std::env::var_os("APPIMAGE").map(PathBuf::from).filter(|p| p.is_file())
        .ok_or("This installation is managed by the system package manager. Download its deb/rpm package and update with that manager.")?;
    #[cfg(target_os = "macos")]
    let destination = application_bundle()?;
    *PENDING_UPDATE.lock().map_err(|e| e.to_string())? = None;
    let download_dir = dirs::cache_dir()
        .ok_or("Cannot locate cache directory")?
        .join("envhub/updates");
    std::fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;
    let session = tempfile::Builder::new()
        .prefix("release-")
        .tempdir_in(&download_dir)
        .map_err(|e| e.to_string())?;
    let target = session.path().join(filename);
    let _ = app.emit("update-download-progress", 5);
    let mut child = env_helper::create_silent_tokio_command("curl")
        .args([
            "-L",
            "-f",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--connect-timeout",
            "15",
            "--max-time",
            "1800",
            "--progress-bar",
            "--show-error",
            "-o",
        ])
        .arg(&target)
        .arg(&download_url)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let pid = child.id();
    env_helper::set_active_install_pid(pid);
    let stderr = child.stderr.take().ok_or("Missing download stderr")?;
    let progress_app = app.clone();
    let reader = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut bytes = Vec::new();
        let mut message = String::new();
        while let Ok(count) = reader.read_until(b'\r', &mut bytes).await {
            if count == 0 {
                break;
            }
            let line = String::from_utf8_lossy(&bytes);
            if let Some(progress) = download_percent(&line) {
                let _ = progress_app.emit("update-download-progress", progress.max(5));
            } else if message.len() < 8192 {
                message.extend(line.chars().take(2048));
            }
            bytes.clear();
        }
        message
    });
    let status = child.wait().await.map_err(|e| e.to_string())?;
    env_helper::clear_active_install_pid(pid);
    let details = reader.await.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("Download failed or cancelled: {status}: {details}"));
    }
    if std::fs::metadata(&target).map_err(|e| e.to_string())?.len() == 0 {
        return Err("Empty update package".into());
    }
    let _ = app.emit("update-download-progress", 92);

    #[cfg(target_os = "macos")]
    {
        let mount = tempfile::Builder::new()
            .prefix("envhub-mount-")
            .tempdir()
            .map_err(|e| e.to_string())?;
        env_helper::checked_output(
            env_helper::output_timeout(
                env_helper::create_silent_tokio_command("hdiutil")
                    .args(["attach", "-nobrowse", "-readonly", "-mountpoint"])
                    .arg(mount.path())
                    .arg(&target),
                60,
            )
            .await?,
        )?;
        let result = async {
            let source = mount.path().join("EnvHub.app");
            if !source.join("Contents/MacOS/envhub").is_file()
                || !source.join("Contents/Info.plist").is_file()
            {
                return Err("Invalid EnvHub application bundle".into());
            }
            let parent = destination
                .parent()
                .ok_or("Invalid application directory")?;
            let staging = tempfile::Builder::new()
                .prefix(".envhub-stage-")
                .tempdir_in(parent)
                .map_err(|e| e.to_string())?;
            let staged = staging.path().join("EnvHub.app");
            env_helper::checked_output(
                env_helper::output_timeout(
                    env_helper::create_silent_tokio_command("ditto")
                        .arg(&source)
                        .arg(&staged),
                    180,
                )
                .await?,
            )?;
            if !staged.join("Contents/MacOS/envhub").is_file() {
                return Err("Incomplete staged application".into());
            }
            let recovery = tempfile::Builder::new()
                .prefix(".envhub-recovery-")
                .tempdir_in(parent)
                .map_err(|e| e.to_string())?
                .keep();
            commit_staged(&staged, &destination, &recovery.join("EnvHub.app"))?;
            Ok::<(), String>(())
        }
        .await;
        let detach = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("hdiutil")
                .arg("detach")
                .arg(mount.path())
                .arg("-force"),
            30,
        )
        .await;
        result?;
        if let Err(error) = detach.and_then(env_helper::checked_output) {
            let _ = app.emit(
                "update-warning",
                format!("Update installed, but the disk image could not be detached: {error}"),
            );
        }
        *PENDING_UPDATE.lock().map_err(|e| e.to_string())? = Some(destination.clone());
        let _ = app.emit("update-download-progress", 100);
        Ok(destination.to_string_lossy().into_owned())
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut header = [0u8; 16];
        std::io::Read::read_exact(
            &mut std::fs::File::open(&target).map_err(|e| e.to_string())?,
            &mut header,
        )
        .map_err(|e| e.to_string())?;
        if !header.starts_with(b"\x7fELF") || header.get(8..11) != Some(b"AI\x02") {
            return Err("Invalid AppImage package".into());
        }
        let parent = destination.parent().ok_or("Invalid AppImage location")?;
        let staged = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        std::fs::copy(&target, staged.path()).map_err(|e| e.to_string())?;
        std::fs::set_permissions(staged.path(), std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
        let recovery = tempfile::Builder::new()
            .prefix(".envhub-recovery-")
            .tempdir_in(parent)
            .map_err(|e| e.to_string())?
            .keep();
        commit_staged(
            staged.path(),
            &destination,
            &recovery.join("previous.AppImage"),
        )?;
        *PENDING_UPDATE.lock().map_err(|e| e.to_string())? = Some(destination.clone());
        let _ = app.emit("update-download-progress", 100);
        Ok(destination.to_string_lossy().into_owned())
    }
    #[cfg(target_os = "windows")]
    {
        let mut header = [0u8; 16];
        std::io::Read::read_exact(
            &mut std::fs::File::open(&target).map_err(|e| e.to_string())?,
            &mut header,
        )
        .map_err(|e| e.to_string())?;
        if !header.starts_with(b"MZ") {
            return Err("Invalid Windows installer".into());
        }
        session.keep();
        *PENDING_UPDATE.lock().map_err(|e| e.to_string())? = Some(target.clone());
        let _ = app.emit("update-download-progress", 100);
        Ok(target.to_string_lossy().into_owned())
    }
}

#[tauri::command]
pub async fn check_github_latest_release() -> Result<RemoteReleaseInfo, String> {
    // 1. Try github API with User-Agent
    if let Ok(out) = env_helper::output_timeout(
        env_helper::create_silent_tokio_command("curl").args([
            "-s",
            "-H",
            "User-Agent: EnvHub-Desktop",
            "-H",
            "Accept: application/vnd.github.v3+json",
            "--max-time",
            "6",
            "https://api.github.com/repos/WnJee/EnvHub/releases/latest",
        ]),
        8,
    )
    .await
    {
        if out.status.success() {
            let body = String::from_utf8_lossy(&out.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(tag) = val["tag_name"].as_str() {
                    return Ok(RemoteReleaseInfo {
                        tag_name: tag.to_string(),
                        body: val["body"].as_str().unwrap_or("").to_string(),
                        html_url: val["html_url"].as_str().unwrap_or("").to_string(),
                        published_at: val["published_at"].as_str().unwrap_or("").to_string(),
                    });
                }
            }
        }
    }

    // 2. Fallback to releases.atom (Zero rate limits, always works)
    if let Ok(out) = env_helper::output_timeout(
        env_helper::create_silent_tokio_command("curl").args([
            "-s",
            "-L",
            "--max-time",
            "8",
            "https://github.com/WnJee/EnvHub/releases.atom",
        ]),
        8,
    )
    .await
    {
        if out.status.success() {
            let xml = String::from_utf8_lossy(&out.stdout);
            if let Some(tag_idx) = xml.find("/releases/tag/") {
                let rest = &xml[tag_idx + "/releases/tag/".len()..];
                let tag: String = rest
                    .chars()
                    .take_while(|c| *c != '"' && *c != '\'' && *c != '<' && !c.is_whitespace())
                    .collect();

                let mut title = String::new();
                if let Some(entry_start) = xml.find("<entry>") {
                    let entry_xml = &xml[entry_start..];
                    if let Some(et_start) = entry_xml.find("<title>") {
                        let et_rest = &entry_xml[et_start + 7..];
                        if let Some(et_end) = et_rest.find("</title>") {
                            title = et_rest[..et_end].to_string();
                        }
                    }
                }

                if !tag.is_empty() {
                    return Ok(RemoteReleaseInfo {
                        tag_name: tag.clone(),
                        body: if title.is_empty() {
                            format!("EnvHub {}", tag)
                        } else {
                            title
                        },
                        html_url: format!("https://github.com/WnJee/EnvHub/releases/tag/{}", tag),
                        published_at: "".to_string(),
                    });
                }
            }
        }
    }

    Err("无法获取 GitHub 最新版本信息".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_curl_carriage_return_progress() {
        assert_eq!(download_percent("##### 25.7%\r"), Some(25));
        assert_eq!(download_percent("##### 100.0%"), Some(90));
        assert_eq!(download_percent("curl: connection failed"), None);
    }
    #[test]
    fn failed_commit_restores_original() {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("app");
        std::fs::write(&destination, "old").unwrap();
        assert!(commit_staged(
            &dir.path().join("missing"),
            &destination,
            &dir.path().join("backup")
        )
        .is_err());
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "old");
    }
    #[test]
    fn successful_commit_keeps_recovery_copy() {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("app");
        let staged = dir.path().join("staged");
        let backup = dir.path().join("backup");
        std::fs::write(&destination, "old").unwrap();
        std::fs::write(&staged, "new").unwrap();
        commit_staged(&staged, &destination, &backup).unwrap();
        assert_eq!(std::fs::read_to_string(destination).unwrap(), "new");
        assert_eq!(std::fs::read_to_string(backup).unwrap(), "old");
    }
}
