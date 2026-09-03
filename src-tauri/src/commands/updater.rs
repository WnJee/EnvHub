use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::env_helper;

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
        return Ok(status.success());
    }

    #[cfg(target_os = "windows")]
    {
        let status = env_helper::create_silent_command("cmd")
            .args(["/c", "start", "", &url])
            .status()
            .map_err(|e| format!("无法打开浏览器: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "linux")]
    {
        let status = env_helper::create_silent_command("xdg-open")
            .arg(&url)
            .status()
            .map_err(|e| format!("无法打开浏览器: {}", e))?;
        return Ok(status.success());
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
        return Ok(status.success());
    }

    #[cfg(target_os = "windows")]
    {
        let status = env_helper::create_silent_command("explorer")
            .args(["/select,", &path])
            .status()
            .map_err(|e| format!("无法在资源管理器中显示: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "linux")]
    {
        let status = env_helper::create_silent_command("xdg-open")
            .arg(&path)
            .status()
            .map_err(|e| format!("无法在文件管理器中显示: {}", e))?;
        return Ok(status.success());
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
        return Ok(status.success());
    }

    #[cfg(target_os = "windows")]
    {
        let status = env_helper::create_silent_command("cmd")
            .args(["/c", "start", "", &path])
            .status()
            .map_err(|e| format!("无法打开安装包: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "linux")]
    {
        let status = env_helper::create_silent_command("xdg-open")
            .arg(&path)
            .status()
            .map_err(|e| format!("无法打开安装包: {}", e))?;
        return Ok(status.success());
    }
}

#[tauri::command]
pub async fn relaunch_application(_app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let mut dest_app = PathBuf::from("/Applications/EnvHub.app");
        if let Ok(mut exe) = std::env::current_exe() {
            while let Some(parent) = exe.parent() {
                if parent.extension().and_then(|s| s.to_str()) == Some("app") {
                    dest_app = parent.to_path_buf();
                    break;
                }
                exe = parent.to_path_buf();
            }
        }

        let script = format!(
            "sleep 0.5 && open -n -a \"{}\"",
            dest_app.to_string_lossy()
        );
        let _ = env_helper::create_silent_command("sh")
            .args(["-c", &script])
            .spawn();
        std::process::exit(0);
    }

    #[cfg(target_os = "windows")]
    {
        let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("C:\\"));
        let mut installer_path: Option<PathBuf> = None;
        if let Ok(entries) = std::fs::read_dir(&download_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with("EnvHub_") && fname.ends_with("_x64-setup.exe") {
                    installer_path = Some(entry.path());
                }
            }
        }

        if let Some(inst) = installer_path {
            let script = format!(
                "Start-Sleep -Seconds 1; Start-Process -FilePath '{}' -ArgumentList '/S' -Wait",
                inst.to_string_lossy()
            );
            let _ = env_helper::create_silent_command("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
                .spawn();
        } else if let Ok(exe) = std::env::current_exe() {
            let _ = env_helper::create_silent_command(exe.to_str().unwrap_or_default()).spawn();
        }
        std::process::exit(0);
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(exe) = std::env::current_exe() {
            let _ = env_helper::create_silent_command(exe.to_str().unwrap_or_default()).spawn();
        }
        std::process::exit(0);
    }

    #[allow(unreachable_code)]
    Ok(())
}

#[tauri::command]
pub async fn download_and_install_update(
    app: AppHandle,
    download_url: String,
    version: String,
) -> Result<String, String> {
    let download_dir = dirs::download_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("Downloads"))
            .unwrap_or_else(|| PathBuf::from("/tmp"))
    });

    let filename = if cfg!(target_os = "macos") {
        format!("EnvHub_{}_universal.dmg", version)
    } else if cfg!(target_os = "windows") {
        format!("EnvHub_{}_x64-setup.exe", version)
    } else {
        format!("EnvHub_{}_amd64.AppImage", version)
    };

    let target_file = download_dir.join(&filename);
    let target_str = target_file.to_string_lossy().to_string();

    let _ = app.emit("update-download-progress", 5);

    let mut child = env_helper::create_silent_tokio_command("curl")
        .args([
            "-L",
            "-f",
            "--progress-bar",
            "-o",
            &target_str,
            &download_url,
        ])
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动下载进程失败: {}", e))?;

    let stderr = child.stderr.take();
    let app_clone = app.clone();

    if let Some(stderr) = stderr {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        tokio::spawn(async move {
            let mut progress: u32 = 10;
            while let Ok(Some(line)) = lines.next_line().await {
                if line.contains('%') {
                    for part in line.split_whitespace() {
                        if part.ends_with('%') {
                            if let Ok(p) = part.trim_end_matches('%').parse::<f32>() {
                                progress = (p as u32).min(90);
                                let _ = app_clone.emit("update-download-progress", progress);
                            }
                        }
                    }
                } else if line.contains('#') {
                    let count = line.chars().filter(|&c| c == '#').count();
                    if count > 0 {
                        progress = (progress + (count as u32 * 2)).min(90);
                        let _ = app_clone.emit("update-download-progress", progress);
                    }
                }
            }
        });
    }

    let status = child.wait().await.map_err(|e| format!("等待下载完成失败: {}", e))?;
    if !status.success() {
        return Err(format!("下载失败，请检查网络或在浏览器中下载，退出码: {:?}", status.code()));
    }

    let _ = app.emit("update-download-progress", 92);

    // Perform seamless in-place silent replacement on macOS
    #[cfg(target_os = "macos")]
    {
        // 1. Determine destination app path (current running bundle or /Applications/EnvHub.app)
        let mut dest_app = PathBuf::from("/Applications/EnvHub.app");
        if let Ok(mut exe) = std::env::current_exe() {
            while let Some(parent) = exe.parent() {
                if parent.extension().and_then(|s| s.to_str()) == Some("app") {
                    dest_app = parent.to_path_buf();
                    break;
                }
                exe = parent.to_path_buf();
            }
        }

        // 2. Mount DMG silently into a unique random mountpoint in /tmp
        let mount_output = env_helper::create_silent_command("hdiutil")
            .args(["attach", &target_str, "-nobrowse", "-readonly", "-mountrandom", "/tmp"])
            .output()
            .map_err(|e| format!("挂载 DMG 更新包失败: {}", e))?;

        if !mount_output.status.success() {
            return Err(format!("挂载 DMG 失败: {}", String::from_utf8_lossy(&mount_output.stderr)));
        }

        let mut mounted_path: Option<PathBuf> = None;
        let stdout_str = String::from_utf8_lossy(&mount_output.stdout);
        for line in stdout_str.lines() {
            for token in line.split('\t') {
                let trimmed = token.trim();
                if (trimmed.contains("dmg.") || trimmed.starts_with("/Volumes/") || trimmed.starts_with("/private/tmp/") || trimmed.starts_with("/tmp/"))
                    && std::path::Path::new(trimmed).is_dir()
                {
                    mounted_path = Some(PathBuf::from(trimmed));
                    break;
                }
            }
            if mounted_path.is_some() {
                break;
            }
        }

        let mount_dir = match mounted_path {
            Some(m) => m,
            None => {
                return Err("未能解析出 DMG 挂载点目录，请重试".to_string());
            }
        };

        // Find .app bundle inside mount directory
        let mut src_app: Option<PathBuf> = None;
        if mount_dir.join("EnvHub.app").exists() {
            src_app = Some(mount_dir.join("EnvHub.app"));
        } else if let Ok(entries) = std::fs::read_dir(&mount_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("app") {
                    src_app = Some(entry.path());
                    break;
                }
            }
        }

        let src = match src_app {
            Some(s) => s,
            None => {
                let _ = env_helper::create_silent_command("hdiutil")
                    .args(["detach".as_ref(), mount_dir.as_os_str(), "-force".as_ref(), "-quiet".as_ref()])
                    .status();
                return Err("未在安装包镜像中找到 EnvHub.app".to_string());
            }
        };

        // Use ditto to replace dest_app (preserves signatures, permissions and symlinks)
        let ditto_status = env_helper::create_silent_command("ditto")
            .args([src.as_os_str(), dest_app.as_os_str()])
            .status();

        let ditto_ok = match ditto_status {
            Ok(s) => s.success(),
            Err(e) => {
                let _ = env_helper::create_silent_command("hdiutil")
                    .args(["detach".as_ref(), mount_dir.as_os_str(), "-force".as_ref(), "-quiet".as_ref()])
                    .status();
                return Err(format!("执行 ditto 失败: {}", e));
            }
        };

        if !ditto_ok {
            let _ = env_helper::create_silent_command("hdiutil")
                .args(["detach".as_ref(), mount_dir.as_os_str(), "-force".as_ref(), "-quiet".as_ref()])
                .status();
            return Err("更新程序复制失败，请确认对应用程序目录具有写入权限".to_string());
        }

        // Clear quarantine attributes
        let _ = env_helper::create_silent_command("xattr")
            .args(["-cr".as_ref(), dest_app.as_os_str()])
            .status();

        // Unmount DMG silently
        let _ = env_helper::create_silent_command("hdiutil")
            .args(["detach".as_ref(), mount_dir.as_os_str(), "-force".as_ref(), "-quiet".as_ref()])
            .status();

        // Clean up temporary downloaded DMG to save disk space
        let _ = std::fs::remove_file(&target_file);
    }

    #[cfg(target_os = "linux")]
    {
        // Linux AppImage replacement
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::fs::copy(&target_file, &exe);
        }
    }

    let _ = app.emit("update-download-progress", 100);

    Ok(target_str)
}

#[tauri::command]
pub async fn check_github_latest_release() -> Result<RemoteReleaseInfo, String> {
    // 1. Try github API with User-Agent
    if let Ok(out) = env_helper::create_silent_command("curl")
        .args([
            "-s",
            "-H", "User-Agent: EnvHub-Desktop",
            "-H", "Accept: application/vnd.github.v3+json",
            "--max-time", "6",
            "https://api.github.com/repos/WnJee/EnvHub/releases/latest"
        ])
        .output()
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
    if let Ok(out) = env_helper::create_silent_command("curl")
        .args([
            "-s",
            "-L",
            "--max-time", "8",
            "https://github.com/WnJee/EnvHub/releases.atom"
        ])
        .output()
    {
        if out.status.success() {
            let xml = String::from_utf8_lossy(&out.stdout);
            if let Some(tag_idx) = xml.find("/releases/tag/") {
                let rest = &xml[tag_idx + "/releases/tag/".len()..];
                let tag: String = rest.chars().take_while(|c| *c != '"' && *c != '\'' && *c != '<' && !c.is_whitespace()).collect();
                
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
                        body: if title.is_empty() { format!("EnvHub {}", tag) } else { title },
                        html_url: format!("https://github.com/WnJee/EnvHub/releases/tag/{}", tag),
                        published_at: "".to_string(),
                    });
                }
            }
        }
    }

    Err("无法获取 GitHub 最新版本信息".to_string())
}
