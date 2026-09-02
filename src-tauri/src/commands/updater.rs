use std::path::PathBuf;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[tauri::command]
pub async fn open_url_in_browser(url: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("open")
            .arg(&url)
            .status()
            .map_err(|e| format!("无法打开浏览器: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("cmd")
            .args(["/c", "start", "", &url])
            .status()
            .map_err(|e| format!("无法打开浏览器: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("xdg-open")
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
        let status = std::process::Command::new("open")
            .args(["-R", &path])
            .status()
            .map_err(|e| format!("无法在访达中显示: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("explorer")
            .args(["/select,", &path])
            .status()
            .map_err(|e| format!("无法在资源管理器中显示: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("xdg-open")
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
        let status = std::process::Command::new("open")
            .arg(&path)
            .status()
            .map_err(|e| format!("无法打开安装包: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("cmd")
            .args(["/c", "start", "", &path])
            .status()
            .map_err(|e| format!("无法打开安装包: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("xdg-open")
            .arg(&path)
            .status()
            .map_err(|e| format!("无法打开安装包: {}", e))?;
        return Ok(status.success());
    }
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

    let mut child = Command::new("curl")
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
                                progress = (p as u32).min(98);
                                let _ = app_clone.emit("update-download-progress", progress);
                            }
                        }
                    }
                } else if line.contains('#') {
                    let count = line.chars().filter(|&c| c == '#').count();
                    if count > 0 {
                        progress = (progress + (count as u32 * 2)).min(95);
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

    let _ = app.emit("update-download-progress", 100);

    // Automatically trigger installation / open installer file
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&target_str)
            .status();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", &target_str])
            .status();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&target_str)
            .status();
    }

    Ok(target_str)
}
