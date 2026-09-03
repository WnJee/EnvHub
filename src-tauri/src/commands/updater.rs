use std::path::PathBuf;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::env_helper;

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
        let _ = env_helper::create_silent_command("open")
            .args(["-n", "-a", "/Applications/EnvHub.app"])
            .spawn();
        std::process::exit(0);
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(exe) = std::env::current_exe() {
            let _ = env_helper::create_silent_command(&exe.to_string_lossy()).spawn();
        }
        std::process::exit(0);
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(exe) = std::env::current_exe() {
            let _ = env_helper::create_silent_command(&exe.to_string_lossy()).spawn();
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

    // Perform seamless in-place silent installation
    #[cfg(target_os = "macos")]
    {
        let mount_point = PathBuf::from("/tmp/envhub_silent_update_mount");
        let _ = env_helper::create_silent_command("hdiutil")
            .args(["detach", &mount_point.to_string_lossy(), "-force", "-quiet"])
            .status();
        let _ = std::fs::remove_dir_all(&mount_point);
        let _ = std::fs::create_dir_all(&mount_point);

        let attach_status = env_helper::create_silent_command("hdiutil")
            .args([
                "attach",
                &target_str,
                "-mountpoint",
                &mount_point.to_string_lossy(),
                "-nobrowse",
                "-readonly",
                "-quiet",
            ])
            .status();

        let mut in_place_success = false;
        if attach_status.map(|s| s.success()).unwrap_or(false) {
            let src_app = mount_point.join("EnvHub.app");
            if src_app.exists() {
                let dest_app = PathBuf::from("/Applications/EnvHub.app");
                let _ = std::fs::remove_dir_all(&dest_app);
                let cp_status = env_helper::create_silent_command("cp")
                    .args(["-R", &src_app.to_string_lossy(), "/Applications/"])
                    .status();

                if cp_status.map(|s| s.success()).unwrap_or(false) {
                    let _ = env_helper::create_silent_command("xattr")
                        .args(["-cr", "/Applications/EnvHub.app"])
                        .status();
                    in_place_success = true;
                }
            }
            let _ = env_helper::create_silent_command("hdiutil")
                .args(["detach", &mount_point.to_string_lossy(), "-force", "-quiet"])
                .status();
        }

        if !in_place_success {
            let _ = env_helper::create_silent_command("open")
                .arg(&target_str)
                .status();
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Execute NSIS silent update
        let silent_install = env_helper::create_silent_command(&target_str)
            .arg("/S")
            .status();
        if !silent_install.map(|s| s.success()).unwrap_or(false) {
            let _ = env_helper::create_silent_command("cmd")
                .args(["/c", "start", "", &target_str])
                .status();
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = env_helper::create_silent_command("xdg-open")
            .arg(&target_str)
            .status();
    }

    let _ = app.emit("update-download-progress", 100);

    Ok(target_str)
}
