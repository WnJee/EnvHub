use std::env;
use std::path::PathBuf;

/// Fix and augment the PATH environment variable for GUI apps on macOS / Linux
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
        ];

        for up in user_paths {
            if up.exists() && !paths.contains(&up) {
                paths.push(up);
            }
        }
    }

    // 2. Standard UNIX / Homebrew / Language standard paths
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
            home.join(".local/share/mise/bin/mise"),
            home.join(".cargo/bin/mise"),
        ];
        for c in candidates {
            if c.exists() {
                return Some(c);
            }
        }
    }

    // 2. Check standard system locations
    let sys_candidates = [
        PathBuf::from("/opt/homebrew/bin/mise"),
        PathBuf::from("/usr/local/bin/mise"),
        PathBuf::from("/usr/bin/mise"),
    ];

    for candidate in sys_candidates {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    // 3. Check Windows standard locations
    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        let win_path = PathBuf::from(local_app_data).join("mise\\bin\\mise.exe");
        if win_path.exists() {
            return Some(win_path);
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
