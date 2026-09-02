use std::env;
use std::path::PathBuf;

/// Fix and augment the PATH environment variable for GUI apps on macOS / Linux
pub fn fix_system_path() {
    let mut paths: Vec<PathBuf> = Vec::new();

    // Standard UNIX / Homebrew / Local paths
    let candidate_paths = [
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        "/usr/local/bin",
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

    // User home directory paths
    if let Some(home) = dirs::home_dir() {
        let user_paths = [
            home.join(".local/bin"),
            home.join(".cargo/bin"),
            home.join(".local/share/mise/shims"),
            home.join(".local/share/mise/bin"),
            home.join(".proto/bin"),
            home.join(".proto/shims"),
            home.join(".nvm/versions/node/current/bin"),
        ];

        for up in user_paths {
            if up.exists() && !paths.contains(&up) {
                paths.push(up);
            }
        }
    }

    // Existing PATH
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
    // 1. Check if `mise` is in PATH
    if let Ok(path) = which::which("mise") {
        return Some(path);
    }

    // 2. Check standard installation locations
    let mut candidates = vec![
        PathBuf::from("/opt/homebrew/bin/mise"),
        PathBuf::from("/usr/local/bin/mise"),
        PathBuf::from("/usr/bin/mise"),
    ];

    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".local/bin/mise"));
        candidates.push(home.join(".cargo/bin/mise"));
        candidates.push(home.join(".local/share/mise/bin/mise"));
    }

    // Windows standard locations
    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local_app_data).join("mise\\bin\\mise.exe"));
    }

    for candidate in candidates {
        if candidate.exists() {
            return Some(candidate);
        }
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
