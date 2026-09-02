use std::env;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Create a synchronous Command with hidden console window on Windows
pub fn create_silent_command(program: &str) -> std::process::Command {
    #[allow(unused_mut)]
    let mut cmd = std::process::Command::new(program);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Create an asynchronous Tokio Command with hidden console window on Windows
pub fn create_silent_tokio_command(program: &str) -> tokio::process::Command {
    #[allow(unused_mut)]
    let mut cmd = tokio::process::Command::new(program);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Fix and augment the PATH environment variable for GUI apps on macOS / Windows / Linux
pub fn fix_system_path() {
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

    if let Ok(new_path) = env::join_paths(paths) {
        env::set_var("PATH", new_path);
    }
}

/// Find the full path to the `mise` binary
pub fn find_mise_binary() -> Option<PathBuf> {
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
