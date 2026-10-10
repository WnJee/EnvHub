use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub const CREATE_NO_WINDOW: u32 = 0x08000000;
static ACTIVE_INSTALL_PID: AtomicU32 = AtomicU32::new(0);
static OPERATION: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

pub struct OperationGuard {
    _permit: tokio::sync::SemaphorePermit<'static>,
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        ACTIVE_INSTALL_PID.store(0, Ordering::SeqCst);
    }
}

pub fn begin_operation() -> Result<OperationGuard, String> {
    OPERATION
        .try_acquire()
        .map(|permit| OperationGuard { _permit: permit })
        .map_err(|_| "Another installation or update is running".into())
}

pub fn validate_target(tool: &str, version: &str) -> Result<(), String> {
    let valid = |s: &str| {
        !s.is_empty()
            && !s.starts_with('-')
            && !s.contains("..")
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || ".-_+".contains(c))
    };
    if !valid(tool) || !valid(version) {
        return Err("Invalid tool or version".into());
    }
    Ok(())
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub async fn output_timeout(
    cmd: &mut tokio::process::Command,
    seconds: u64,
) -> Result<std::process::Output, String> {
    cmd.kill_on_drop(true)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = cmd.spawn().map_err(|e| e.to_string())?;
    let pid = child.id();
    let wait = child.wait_with_output();
    tokio::pin!(wait);
    match tokio::time::timeout(std::time::Duration::from_secs(seconds), &mut wait).await {
        Ok(result) => result.map_err(|e| e.to_string()),
        Err(_) => {
            if let Some(pid) = pid {
                let _ = kill_process_tree(pid);
            }
            let _ = tokio::time::timeout(std::time::Duration::from_secs(2), &mut wait).await;
            Err(format!("Command timed out after {seconds}s"))
        }
    }
}

pub fn checked_output(output: std::process::Output) -> Result<std::process::Output, String> {
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "Command failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

pub fn set_active_install_pid(pid: Option<u32>) {
    ACTIVE_INSTALL_PID.store(pid.unwrap_or(0), Ordering::SeqCst);
}

pub fn clear_active_install_pid(pid: Option<u32>) {
    if let Some(pid) = pid {
        let _ = ACTIVE_INSTALL_PID.compare_exchange(pid, 0, Ordering::SeqCst, Ordering::SeqCst);
    }
}

pub fn cancel_active_install() -> Result<bool, String> {
    let pid = ACTIVE_INSTALL_PID.load(Ordering::SeqCst);
    if pid == 0 {
        return Ok(false);
    }

    kill_process_tree(pid)
}

fn kill_process_tree(pid: u32) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    let status = create_silent_command("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status();
    #[cfg(not(target_os = "windows"))]
    let status = {
        std::process::Command::new("/bin/kill")
            .args(["-KILL", "--", &format!("-{pid}")])
            .status()
    };

    status
        .map(|status| status.success())
        .map_err(|e| format!("终止安装进程失败: {}", e))
}

/// Create a synchronous Command with hidden console window on Windows
pub fn create_silent_command(program: &str) -> std::process::Command {
    let resolved_program = resolve_windows_program(program);
    #[allow(unused_mut)]
    let mut cmd = std::process::Command::new(resolved_program);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.env("PATH", augmented_path());
    cmd
}

/// Create an asynchronous Tokio Command with hidden console window on Windows
pub fn create_silent_tokio_command(program: &str) -> tokio::process::Command {
    let resolved_program = resolve_windows_program(program);
    #[allow(unused_mut)]
    let mut cmd = tokio::process::Command::new(resolved_program);
    #[cfg(unix)]
    cmd.process_group(0);
    cmd.kill_on_drop(true);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.env("PATH", augmented_path());
    cmd
}

/// Resolve commands that are normally available from System32 but may be
/// missing from a GUI process' inherited PATH (notably PowerShell on Windows).
fn resolve_windows_program(program: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        if program.eq_ignore_ascii_case("powershell")
            || program.eq_ignore_ascii_case("powershell.exe")
        {
            if let Ok(root) = env::var("SystemRoot") {
                let candidate = PathBuf::from(root)
                    .join("System32")
                    .join("WindowsPowerShell")
                    .join("v1.0")
                    .join("powershell.exe");
                if candidate.is_file() {
                    return candidate.to_string_lossy().into_owned();
                }
            }
        }
    }
    program.to_string()
}

/// Fix and augment the PATH environment variable for GUI apps on macOS / Windows / Linux
pub fn fix_system_path() {
    env::set_var("PATH", augmented_path());
}

fn augmented_path() -> std::ffi::OsString {
    let mut paths: Vec<PathBuf> = Vec::new();

    // 1. Highest priority: User version manager shims and local binaries
    if let Some(home) = dirs::home_dir() {
        let user_paths = [
            home.join(".local/share/mise/shims"),
            home.join(".local/share/mise/bin"),
            home.join(".cargo/bin"),
            home.join(".local/bin"),
            home.join("go/bin"),
            home.join(".go/bin"),
            home.join(".bun/bin"),
            home.join(".deno/bin"),
            home.join(".nvm/current/bin"),
            home.join(".proto/shims"),
            home.join(".proto/bin"),
            home.join("scoop/shims"),
            home.join("scoop/apps"),
        ];

        for up in user_paths {
            if up.exists() && !paths.contains(&up) {
                paths.push(up);
            }
        }
    }

    // Windows specific AppData paths
    if let Some(data_local) = dirs::data_local_dir() {
        let win_user_paths = [
            data_local.join("mise/shims"),
            data_local.join("mise/bin"),
            data_local.join("Programs/Python/Python312"),
            data_local.join("Programs/Python/Python311"),
            data_local.join("Microsoft/WinGet/Links"),
            data_local.join("Microsoft/WindowsApps"),
        ];
        for wp in win_user_paths {
            if wp.exists() && !paths.contains(&wp) {
                paths.push(wp);
            }
        }
    }

    // 2. Standard UNIX / Homebrew / Language / Windows Program Files standard paths
    let candidate_paths = [
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        "/usr/local/bin",
        "/usr/local/go/bin",
        "/opt/homebrew/opt/go/libexec/bin",
        "/opt/homebrew/opt/openjdk/bin",
        "/usr/local/opt/openjdk/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
        "C:\\Program Files\\nodejs",
        "C:\\Program Files\\Go\\bin",
        "C:\\Program Files\\Git\\cmd",
        "C:\\Program Files\\Docker\\Docker\\resources\\bin",
        "C:\\ProgramData\\chocolatey\\bin",
    ];

    for p in candidate_paths {
        let path_buf = PathBuf::from(p);
        if path_buf.exists() && !paths.contains(&path_buf) {
            paths.push(path_buf);
        }
    }

    // 3. Existing PATH elements
    if let Ok(current_path) = env::var("PATH") {
        for split_path in env::split_paths(&current_path) {
            if !paths.contains(&split_path) {
                paths.push(split_path);
            }
        }
    }

    env::join_paths(paths).unwrap_or_else(|_| env::var_os("PATH").unwrap_or_default())
}

/// Find the full path to the `mise` binary
pub fn find_mise_binary() -> Option<PathBuf> {
    if let Some(path) = custom_mise_path() {
        return Some(path);
    }
    // 1. Check user custom install locations first
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".local/bin/mise"),
            home.join(".local/bin/mise.exe"),
            home.join(".local/share/mise/bin/mise"),
            home.join(".local/share/mise/bin/mise.exe"),
            home.join(".cargo/bin/mise"),
            home.join(".cargo/bin/mise.exe"),
            home.join("scoop/shims/mise.exe"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }

    // 2. Check Windows LocalAppData locations
    if let Some(data_local) = dirs::data_local_dir() {
        let win_candidates = [
            data_local.join("mise/bin/mise.exe"),
            data_local.join("Microsoft/WinGet/Links/mise.exe"),
        ];
        for wc in win_candidates {
            if wc.exists() {
                return Some(wc);
            }
        }
    }

    // 3. Check standard system locations
    let sys_candidates = [
        PathBuf::from("/opt/homebrew/bin/mise"),
        PathBuf::from("/usr/local/bin/mise"),
        PathBuf::from("/usr/bin/mise"),
        PathBuf::from("C:\\Program Files\\mise\\bin\\mise.exe"),
        PathBuf::from("C:\\ProgramData\\chocolatey\\bin\\mise.exe"),
    ];

    for candidate in sys_candidates {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    // 4. Check if `mise` is in PATH
    if let Ok(path) = which::which("mise") {
        return Some(path);
    }

    None
}

fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("envhub/settings.json"))
}

fn custom_mise_path() -> Option<PathBuf> {
    let content = std::fs::read_to_string(settings_path()?).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let path = PathBuf::from(value["misePath"].as_str()?);
    path.is_file().then_some(path)
}

#[tauri::command]
pub async fn save_mise_path(path: String) -> Result<String, String> {
    let _operation = begin_operation()?;
    let path = if let Some(suffix) = path.strip_prefix("~/") {
        dirs::home_dir()
            .ok_or("Cannot locate home directory")?
            .join(suffix)
    } else {
        PathBuf::from(path)
    };
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    if !path.is_file() {
        return Err("Mise path must be an executable file".into());
    }
    let output = checked_output(
        output_timeout(
            create_silent_tokio_command(&path.to_string_lossy()).arg("--version"),
            5,
        )
        .await?,
    )?;
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return Err("Executable did not report a Mise version".into());
    }
    let _guard = crate::config_file::CONFIG_LOCK
        .lock()
        .map_err(|e| e.to_string())?;
    let settings = settings_path().ok_or("Cannot locate settings")?;
    let old = crate::config_file::read(&settings)?;
    let mut value: serde_json::Value = if old.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(&old).map_err(|e| e.to_string())?
    };
    value["misePath"] = serde_json::json!(path);
    crate::config_file::write(
        &settings,
        &serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?,
    )?;
    Ok(path.to_string_lossy().into_owned())
}

/// Check for command shims such as Scoop's `.cmd`/`.ps1` files.
pub fn command_available(name: &str) -> bool {
    let path = augmented_path();
    for dir in env::split_paths(&path) {
        if dir.join(name).is_file() {
            return true;
        }
        for suffix in [".exe", ".cmd", ".bat", ".ps1"] {
            if dir.join(format!("{}{}", name, suffix)).is_file() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_targets_rejected() {
        for version in ["../outside", "-force", "1;exit", "$(id)"] {
            assert!(validate_target("node", version).is_err());
        }
        assert!(validate_target("java", "temurin-21.0.2+13").is_ok());
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_kills_descendants_and_releases_ownership() {
        let operation = begin_operation().unwrap();
        assert!(begin_operation().is_err());
        let mut child = create_silent_tokio_command("sh")
            .args(["-c", "sh -c 'sleep 30 & wait' & wait"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        set_active_install_pid(child.id());
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert!(cancel_active_install().unwrap());
        assert!(!child.wait().await.unwrap().success());
        // A living grandchild would retain the pipe and prevent EOF.
        use tokio::io::AsyncReadExt;
        let mut pipe = child.stdout.take().unwrap();
        let mut bytes = Vec::new();
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            pipe.read_to_end(&mut bytes),
        )
        .await
        .unwrap()
        .unwrap();
        drop(operation);
        assert!(begin_operation().is_ok());
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn command_timeout_is_bounded() {
        let before = std::time::Instant::now();
        assert!(output_timeout(
            create_silent_tokio_command("sh").args(["-c", "sleep 30"]),
            1
        )
        .await
        .is_err());
        assert!(before.elapsed() < std::time::Duration::from_secs(3));
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn nonzero_exit_propagates_stderr() {
        let output = output_timeout(
            create_silent_tokio_command("sh").args(["-c", "echo failed >&2; exit 7"]),
            2,
        )
        .await
        .unwrap();
        let error = checked_output(output).unwrap_err();
        assert!(error.contains("failed"));
        assert!(error.contains('7'));
    }
}

mod which {
    use std::env;
    use std::path::PathBuf;

    pub fn which(name: &str) -> Result<PathBuf, ()> {
        let path_var = env::var("PATH").map_err(|_| ())?;
        for dir in env::split_paths(&path_var) {
            let full_path = dir.join(name);
            if full_path.is_file() {
                return Ok(full_path);
            }
            #[cfg(windows)]
            {
                let exe_path = dir.join(format!("{}.exe", name));
                if exe_path.is_file() {
                    return Ok(exe_path);
                }
            }
        }
        Err(())
    }
}
