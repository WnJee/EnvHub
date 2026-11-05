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
        // Every mutation funnels through this guard, so dropping the probe
        // caches here guarantees the next read reflects the new state instead
        // of a mid-install snapshot. Done on drop (not on acquire) so failed
        // operations also invalidate.
        crate::commands::mise::invalidate_probe_cache();
        crate::commands::system::invalidate_probe_cache();
        // A mutation may have created `~/.local/bin/mise`, the shims dir, a new
        // package-manager prefix, ... so the memoised PATH / binary are stale.
        invalidate_path_caches();
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

/// `augmented_path()` costs ~25 `Path::exists()` stats plus a PATH rebuild, and
/// every single process spawn calls it. Cache it; `invalidate_path_caches()`
/// (from `OperationGuard::drop`) refreshes it after anything installs a tool.
static AUGMENTED_PATH: std::sync::Mutex<Option<std::ffi::OsString>> = std::sync::Mutex::new(None);
static MISE_BINARY: std::sync::Mutex<Option<Option<PathBuf>>> = std::sync::Mutex::new(None);

/// Drop the memoised PATH / mise binary so the next call re-probes the disk.
pub fn invalidate_path_caches() {
    if let Ok(mut guard) = AUGMENTED_PATH.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = MISE_BINARY.lock() {
        *guard = None;
    }
}

fn augmented_path() -> std::ffi::OsString {
    if let Ok(guard) = AUGMENTED_PATH.lock() {
        if let Some(cached) = guard.as_ref() {
            return cached.clone();
        }
    }
    let computed = compute_augmented_path();
    if let Ok(mut guard) = AUGMENTED_PATH.lock() {
        *guard = Some(computed.clone());
    }
    computed
}

/// Highest installed version inside a version-managed directory.
///
/// `nvm` does not create a `current` symlink — binaries live in
/// `~/.nvm/versions/node/v22.12.0/bin`, so the newest version has to be picked
/// explicitly.
fn latest_versioned_dir(base: &std::path::Path) -> Option<PathBuf> {
    let mut best: Option<(Vec<u32>, PathBuf)> = None;
    for entry in std::fs::read_dir(base).ok()?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let key: Vec<u32> = name
            .trim_start_matches('v')
            .split('.')
            .map_while(|part| part.parse::<u32>().ok())
            .collect();
        if key.is_empty() {
            continue;
        }
        if best.as_ref().map(|(k, _)| key > *k).unwrap_or(true) {
            best = Some((key, path));
        }
    }
    best.map(|(_, path)| path)
}

/// Parse `reg query ... /v Path` output into the raw PATH string.
///
/// The output looks like:
/// ```text
/// HKEY_CURRENT_USER\Environment
///     Path    REG_EXPAND_SZ    C:\Users\me\AppData\Local\...;C:\tools
/// ```
/// The value itself may contain spaces, so only the first two tokens are split.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn parse_reg_query_output(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("Path").or_else(|| trimmed.strip_prefix("PATH")) else {
            continue;
        };
        let rest = rest.trim_start();
        let (kind, value) = rest.split_once(char::is_whitespace)?;
        if !kind.starts_with("REG_") {
            continue;
        }
        let value = value.trim_start();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

/// Expand `%USERPROFILE%` style references, which `REG_EXPAND_SZ` PATH values
/// are full of. Unknown variables are left untouched.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn expand_windows_env(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if let Some(offset) = raw[i + 1..].find('%') {
                let name = &raw[i + 1..i + 1 + offset];
                if !name.is_empty() {
                    if let Ok(value) = std::env::var(name) {
                        out.push_str(&value);
                        i = i + 1 + offset + 1;
                        continue;
                    }
                }
            }
        }
        let ch = raw[i..].chars().next().unwrap_or('%');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Read the persistent PATH from the Windows registry.
///
/// `winget`, `scoop` and most installers append to `HKCU\Environment\Path`, but
/// a process keeps the PATH it inherited at launch — so a tool installed from
/// EnvHub would stay invisible until the app restarted. Re-reading the registry
/// is what makes a freshly installed tool usable immediately.
#[cfg(target_os = "windows")]
fn windows_registry_paths() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for key in [
        "HKCU\\Environment",
        "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
    ] {
        let output = std::process::Command::new("reg")
            .args(["query", key, "/v", "Path"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let Ok(output) = output else { continue };
        if !output.status.success() {
            continue;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(raw) = parse_reg_query_output(&stdout) {
            for entry in expand_windows_env(&raw).split(';') {
                let entry = entry.trim();
                if !entry.is_empty() {
                    found.push(PathBuf::from(entry));
                }
            }
        }
    }
    found
}

/// Directories that package managers and version managers install into but
/// which are frequently missing from a GUI process' inherited PATH.
fn well_known_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    if let Some(home) = dirs::home_dir() {
        // Version-manager shims and per-user bin dirs. Highest priority: they
        // must win over `/usr/bin` so a pinned toolchain is what runs.
        for rel in [
            ".local/share/mise/shims",
            ".local/share/mise/bin",
            ".local/bin",
            ".cargo/bin",
            ".bun/bin",
            ".deno/bin",
            ".proto/shims",
            ".proto/bin",
            ".asdf/shims",
            ".pyenv/shims",
            ".volta/bin",
            "go/bin",
            ".go/bin",
            "Library/pnpm",
            ".local/share/pnpm",
            ".nix-profile/bin",
            "scoop/shims",
            ".dotnet/tools",
            ".pyenv/pyenv-win/bin",
            ".pyenv/pyenv-win/shims",
        ] {
            let path = home.join(rel);
            if path.exists() && !dirs.contains(&path) {
                dirs.push(path);
            }
        }

        // nvm keeps no `current` symlink; resolve the newest installed version.
        if let Some(newest) = latest_versioned_dir(&home.join(".nvm/versions/node")) {
            let bin = newest.join("bin");
            if bin.exists() && !dirs.contains(&bin) {
                dirs.push(bin);
            }
        }
    }

    // Windows per-user locations (%LOCALAPPDATA%).
    if let Some(local) = dirs::data_local_dir() {
        for rel in [
            "mise/shims",
            "mise/bin",
            "Microsoft/WinGet/Links",
            "Microsoft/WindowsApps",
            "pnpm",
            "Volta/bin",
            "Yarn/bin",
            "Programs/Python/Python313",
            "Programs/Python/Python313/Scripts",
            "Programs/Python/Python312",
            "Programs/Python/Python312/Scripts",
            "Programs/Python/Python311",
            "Programs/Python/Python311/Scripts",
        ] {
            let path = local.join(rel);
            if path.exists() && !dirs.contains(&path) {
                dirs.push(path);
            }
        }
    }

    // `npm i -g` installs to %APPDATA%\npm on Windows.
    if let Some(appdata) = dirs::config_dir() {
        let npm = appdata.join("npm");
        if npm.exists() && !dirs.contains(&npm) {
            dirs.push(npm);
        }
    }

    dirs
}

fn compute_augmented_path() -> std::ffi::OsString {
    let mut paths: Vec<PathBuf> = well_known_dirs();

    // 2. System-wide locations. Homebrew and the language runtimes installed by
    //    their official installers land here.
    let candidate_paths = [
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        "/usr/local/bin",
        "/usr/local/go/bin",
        "/opt/homebrew/opt/go/libexec/bin",
        "/opt/homebrew/opt/openjdk/bin",
        "/usr/local/opt/openjdk/bin",
        "/nix/var/nix/profiles/default/bin",
        "/snap/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
        "C:\\Program Files\\nodejs",
        "C:\\Program Files\\Go\\bin",
        "C:\\Program Files\\Git\\cmd",
        "C:\\Program Files\\Docker\\Docker\\resources\\bin",
        "C:\\Program Files\\Microsoft VS Code\\bin",
        "C:\\ProgramData\\chocolatey\\bin",
    ];

    for p in candidate_paths {
        let path_buf = PathBuf::from(p);
        if path_buf.exists() && !paths.contains(&path_buf) {
            paths.push(path_buf);
        }
    }

    // 3. Persistent PATH from the registry (Windows only): picks up whatever
    //    winget/scoop/an installer appended since this process started.
    #[cfg(target_os = "windows")]
    for p in windows_registry_paths() {
        if !paths.contains(&p) {
            paths.push(p);
        }
    }

    // 4. Whatever this process inherited.
    if let Ok(current_path) = env::var("PATH") {
        for split_path in env::split_paths(&current_path) {
            if !split_path.as_os_str().is_empty() && !paths.contains(&split_path) {
                paths.push(split_path);
            }
        }
    }

    env::join_paths(paths).unwrap_or_else(|_| env::var_os("PATH").unwrap_or_default())
}

/// Find the full path to the `mise` binary
pub fn find_mise_binary() -> Option<PathBuf> {
    if let Ok(guard) = MISE_BINARY.lock() {
        if let Some(cached) = guard.as_ref() {
            return cached.clone();
        }
    }
    let found = probe_mise_binary();
    if let Ok(mut guard) = MISE_BINARY.lock() {
        *guard = Some(found.clone());
    }
    found
}

fn probe_mise_binary() -> Option<PathBuf> {
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
    fn parses_reg_query_path_value_with_spaces() {
        let stdout = "\r\nHKEY_CURRENT_USER\\Environment\r\n    Path    REG_EXPAND_SZ    %USERPROFILE%\\AppData\\Local\\Microsoft\\WinGet\\Links;C:\\Program Files\\Tools\\bin\r\n\r\n";
        assert_eq!(
            parse_reg_query_output(stdout).as_deref(),
            Some("%USERPROFILE%\\AppData\\Local\\Microsoft\\WinGet\\Links;C:\\Program Files\\Tools\\bin")
        );
    }

    #[test]
    fn reg_query_parser_ignores_other_values() {
        // `reg query` prints every value when /v is omitted, and a missing key
        // yields an error message rather than a value.
        let stdout = "HKEY_CURRENT_USER\\Environment\r\n    TEMP    REG_EXPAND_SZ    %USERPROFILE%\\AppData\\Local\\Temp\r\n";
        assert_eq!(parse_reg_query_output(stdout), None);
        assert_eq!(parse_reg_query_output("ERROR: The system was unable to find the specified registry key or value.\r\n"), None);
    }

    #[test]
    fn expands_windows_env_references() {
        std::env::set_var("ENVHUB_TEST_ROOT", "C:\\root");
        assert_eq!(
            expand_windows_env("%ENVHUB_TEST_ROOT%\\bin;%NOPE_MISSING%\\x"),
            "C:\\root\\bin;%NOPE_MISSING%\\x"
        );
        // A lone % must not panic or swallow the rest of the string.
        assert_eq!(expand_windows_env("100%\\bin"), "100%\\bin");
        std::env::remove_var("ENVHUB_TEST_ROOT");
    }

    #[test]
    fn picks_highest_nvm_version() {
        let dir = tempfile::tempdir().unwrap();
        for version in ["v20.9.0", "v22.12.0", "v22.9.1", "not-a-version"] {
            std::fs::create_dir(dir.path().join(version)).unwrap();
        }
        std::fs::write(dir.path().join("v99.0.0"), "").unwrap(); // a file, not a dir
        let picked = latest_versioned_dir(dir.path()).unwrap();
        assert_eq!(picked.file_name().unwrap(), "v22.12.0");
    }

    #[test]
    fn missing_version_dir_is_not_an_error() {
        assert!(latest_versioned_dir(std::path::Path::new("/nonexistent-envhub")).is_none());
    }

    #[test]
    fn augmented_path_puts_shims_before_system_bins() {
        let path = augmented_path();
        let entries: Vec<PathBuf> = std::env::split_paths(&path).collect();
        // Deduplicated and non-empty, and /usr/bin never precedes a shims dir.
        assert!(entries.iter().all(|p| !p.as_os_str().is_empty()));
        let mut sorted = entries.clone();
        sorted.dedup();
        assert_eq!(sorted.len(), entries.len(), "duplicate entries in PATH");
        let first_system = entries.iter().position(|p| p == std::path::Path::new("/usr/bin"));
        let last_shim = entries
            .iter()
            .rposition(|p| p.to_string_lossy().contains("shims"));
        if let (Some(system), Some(shim)) = (first_system, last_shim) {
            assert!(shim < system, "shims must outrank /usr/bin");
        }
    }

    /// Whatever a package manager dropped on disk must actually reach the PATH
    /// we hand to child processes — this is the invariant that broke for
    /// `~/Library/pnpm` and `~/.nvm/versions/node/*/bin`.
    #[test]
    fn every_existing_well_known_dir_is_on_the_path() {
        let entries: Vec<PathBuf> = std::env::split_paths(&augmented_path()).collect();
        let mut checked = 0;
        for dir in well_known_dirs() {
            assert!(
                entries.contains(&dir),
                "well-known dir missing from PATH: {}",
                dir.display()
            );
            checked += 1;
        }
        // Guard against the list silently becoming empty on a misconfigured box.
        assert!(checked > 0, "no well-known dirs discovered at all");
    }

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
