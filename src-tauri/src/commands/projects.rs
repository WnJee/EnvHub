use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectTool {
    #[serde(rename = "toolId")]
    pub tool_id: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectEnv {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(rename = "configFile")]
    pub config_file: Option<String>,
    pub tools: Vec<ProjectTool>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
}

fn get_projects_config_path() -> Option<PathBuf> {
    if let Some(cfg) = dirs::config_dir() {
        return Some(cfg.join("envhub/projects.json"));
    }
    dirs::home_dir().map(|h| h.join(".config/envhub/projects.json"))
}

fn load_saved_project_paths() -> Result<Vec<String>, String> {
    let p = get_projects_config_path().ok_or("Cannot locate project settings")?;
    let content = crate::config_file::read(&p)?;
    if content.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn save_project_paths(paths: &[String]) -> Result<(), String> {
    let p = get_projects_config_path().ok_or("Cannot locate project settings")?;
    crate::config_file::write(
        &p,
        &serde_json::to_string_pretty(paths).map_err(|e| e.to_string())?,
    )
}

/// Canonicalise a user-supplied project path so `/a/b`, `/a/b/`, `~/b` and
/// symlinked variants all resolve to the same entry.
fn normalize_project_path(raw: &str) -> PathBuf {
    let expanded = if raw == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from(raw))
    } else if let Some(rest) = raw.strip_prefix("~/") {
        dirs::home_dir()
            .map(|home| home.join(rest))
            .unwrap_or_else(|| PathBuf::from(raw))
    } else {
        PathBuf::from(raw)
    };
    expanded.canonicalize().unwrap_or(expanded)
}

/// Helper to extract real tools from a directory
fn inspect_directory_tools(dir: &Path) -> (Option<String>, Vec<ProjectTool>) {
    let mut config_file: Option<String> = None;
    let mut tools = Vec::new();
    let mut mise_fallback: Option<String> = None;

    for name in ["mise.toml", ".mise.toml"] {
        let path = dir.join(name);
        if !path.exists() {
            continue;
        }
        let mut declared_tools = false;
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(doc) = content.parse::<toml_edit::DocumentMut>() {
                if let Some(table) = doc.get("tools").and_then(|t| t.as_table_like()) {
                    for (id, item) in table.iter() {
                        let version = item
                            .as_str()
                            .or_else(|| item.get("version").and_then(|v| v.as_str()));
                        if let Some(version) = version {
                            tools.push(ProjectTool {
                                tool_id: id.into(),
                                version: version.into(),
                            });
                            declared_tools = true;
                        }
                    }
                    if declared_tools {
                        config_file = Some(name.to_string());
                        break;
                    }
                }
            }
        }
        // The file exists but declares no tools (e.g. only `[env]`). Do not let
        // it hide a sibling `.tool-versions`, but remember it as a fallback.
        if mise_fallback.is_none() {
            mise_fallback = Some(name.to_string());
        }
    }

    // 2. Check .tool-versions (asdf / mise standard)
    let tool_versions = dir.join(".tool-versions");
    if tool_versions.exists() && tools.is_empty() {
        if let Ok(content) = fs::read_to_string(&tool_versions) {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && !parts[0].starts_with('#') {
                    tools.push(ProjectTool {
                        tool_id: parts[0].to_string(),
                        version: parts[1].to_string(),
                    });
                }
            }
            if !tools.is_empty() {
                config_file = Some(".tool-versions".to_string());
            }
        }
    }

    if config_file.is_none() {
        config_file = mise_fallback;
    }

    // 3. Check package.json (Node.js)
    let pkg_json = dir.join("package.json");
    if pkg_json.exists() {
        if config_file.is_none() {
            config_file = Some("package.json".to_string());
        }
        if !tools.iter().any(|t| t.tool_id == "node") {
            let mut node_ver = "latest".to_string();
            if let Ok(content) = fs::read_to_string(&pkg_json) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(engine_node) = val
                        .get("engines")
                        .and_then(|e| e.get("node"))
                        .and_then(|n| n.as_str())
                    {
                        node_ver = engine_node.to_string();
                    }
                }
            }
            tools.push(ProjectTool {
                tool_id: "node".to_string(),
                version: node_ver,
            });
        }
    }

    // 4. Check Cargo.toml (Rust)
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() && !tools.iter().any(|t| t.tool_id == "rust") {
        tools.push(ProjectTool {
            tool_id: "rust".to_string(),
            version: "system".to_string(),
        });
    }

    // 5. Check go.mod (Go)
    let go_mod = dir.join("go.mod");
    if go_mod.exists() && !tools.iter().any(|t| t.tool_id == "go") {
        let mut go_ver = "latest".to_string();
        if let Ok(content) = fs::read_to_string(&go_mod) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("go ") {
                    if let Some(v) = trimmed.split_whitespace().nth(1) {
                        go_ver = v.to_string();
                    }
                }
            }
        }
        tools.push(ProjectTool {
            tool_id: "go".to_string(),
            version: go_ver,
        });
    }

    (config_file, tools)
}

/// Sync (not `async`) so Tauri runs it on its blocking pool: it does blocking
/// file IO for every bound project.
#[tauri::command]
pub fn get_projects() -> Result<Vec<ProjectEnv>, String> {
    let saved_paths = load_saved_project_paths()?;
    let mut projects = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for path_str in saved_paths {
        let p = Path::new(&path_str);
        if !p.exists() {
            continue;
        }
        let normalized = normalize_project_path(&path_str);
        if !seen.insert(normalized.clone()) {
            continue;
        }
        let (config_file, tools) = inspect_directory_tools(p);
        let name = normalized
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "project".to_string());
        projects.push(ProjectEnv {
            id: normalized.to_string_lossy().to_string(),
            name,
            path: normalized.to_string_lossy().to_string(),
            config_file,
            tools,
            last_modified: Some("已绑定".to_string()),
        });
    }

    Ok(projects)
}

#[tauri::command]
pub fn scan_and_add_project(path: String) -> Result<ProjectEnv, String> {
    let normalized = normalize_project_path(&path);
    if !normalized.is_absolute() || !normalized.is_dir() {
        return Err(format!("指定路径不存在: {}", path));
    }
    let path = normalized.to_string_lossy().to_string();

    let name = normalized
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());
    let (config_file, tools) = inspect_directory_tools(&normalized);

    let _guard = crate::config_file::CONFIG_LOCK
        .lock()
        .map_err(|e| e.to_string())?;
    let mut saved = load_saved_project_paths()?;
    // Compare canonical forms so `/a/b/` does not create a duplicate entry.
    let already_saved = saved
        .iter()
        .any(|existing| normalize_project_path(existing) == normalized);
    if !already_saved {
        saved.push(path.clone());
        save_project_paths(&saved)?;
    }

    Ok(ProjectEnv {
        id: path.clone(),
        name,
        path,
        config_file,
        tools,
        last_modified: Some("刚刚添加".to_string()),
    })
}

#[tauri::command]
pub fn remove_project(path: String) -> Result<bool, String> {
    let _guard = crate::config_file::CONFIG_LOCK
        .lock()
        .map_err(|e| e.to_string())?;
    let target = normalize_project_path(&path);
    let mut saved = load_saved_project_paths()?;
    saved.retain(|existing| normalize_project_path(existing) != target);
    save_project_paths(&saved)?;
    Ok(true)
}

#[tauri::command]
pub fn set_project_tool_version(
    project_path: String,
    tool_id: String,
    version: String,
) -> Result<bool, String> {
    let target_dir = PathBuf::from(&project_path);
    if !target_dir.is_absolute() || !target_dir.is_dir() {
        return Err("Invalid project directory".into());
    }
    crate::env_helper::validate_target(&tool_id, &version)?;
    let config = if target_dir.join("mise.toml").exists() {
        target_dir.join("mise.toml")
    } else {
        target_dir.join(".mise.toml")
    };
    if !config.exists() && target_dir.join(".tool-versions").exists() {
        let _guard = crate::config_file::CONFIG_LOCK
            .lock()
            .map_err(|e| e.to_string())?;
        let path = target_dir.join(".tool-versions");
        let content = crate::config_file::read(&path)?;
        let mut found = false;
        let mut lines: Vec<String> = content
            .lines()
            .map(|line| {
                if line.split_whitespace().next() == Some(tool_id.as_str()) {
                    found = true;
                    let comment = line
                        .split_once('#')
                        .map(|(_, c)| format!(" #{}", c))
                        .unwrap_or_default();
                    format!("{} {}{}", tool_id, version, comment)
                } else {
                    line.to_string()
                }
            })
            .collect();
        if !found {
            lines.push(format!("{} {}", tool_id, version));
        }
        crate::config_file::write(&path, &(lines.join("\n") + "\n"))?;
    } else {
        crate::config_file::edit_toml(&config, |doc| {
            crate::config_file::set_tool(doc, &tool_id, &version)
        })?;
    }
    Ok(true)
}

#[tauri::command]
pub fn open_in_editor(path: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        // Prefer the `code` shim so the folder opens as a workspace.
        if let Ok(status) = crate::env_helper::create_silent_command("code")
            .arg(&path)
            .status()
        {
            if status.success() {
                return Ok(true);
            }
        }
        for app in ["Visual Studio Code", "Cursor", "Zed", "Sublime Text"] {
            if let Ok(status) = crate::env_helper::create_silent_command("open")
                .args(["-a", app, &path])
                .status()
            {
                if status.success() {
                    return Ok(true);
                }
            }
        }
        Err("未检测到可用的代码编辑器（VS Code / Cursor / Zed / Sublime Text）。请先安装其中之一".into())
    }

    #[cfg(target_os = "windows")]
    {
        // Invoke Code directly so paths containing spaces are passed as one
        // argument. Fall back to `start` for installations that only expose a
        // shell shim.
        if let Ok(status) = crate::env_helper::create_silent_command("code")
            .arg(&path)
            .status()
        {
            if status.success() {
                return Ok(true);
            }
        }
        let status = crate::env_helper::create_silent_command("cmd")
            .args(["/c", "start", "", "code", &path])
            .status()
            .map_err(|e| format!("打开 VS Code 失败: {}", e))?;
        Ok(status.success())
    }

    #[cfg(target_os = "linux")]
    {
        let status = crate::env_helper::create_silent_command("code")
            .arg(&path)
            .status()
            .map_err(|e| format!("打开 VS Code 失败: {}", e))?;
        Ok(status.success())
    }
}

#[tauri::command]
pub fn open_in_terminal(path: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let status = crate::env_helper::create_silent_command("open")
            .args(["-a", "Terminal", &path])
            .status()
            .map_err(|e| format!("打开终端失败: {}", e))?;
        Ok(status.success())
    }

    #[cfg(target_os = "windows")]
    {
        // Prefer Windows Terminal directly; this preserves spaces and Unicode
        // in project paths. Fall back to PowerShell when `wt.exe` is absent.
        if let Ok(status) = crate::env_helper::create_silent_command("wt")
            .args(["-d", &path])
            .status()
        {
            if status.success() {
                return Ok(true);
            }
        }
        let escaped = path.replace('\'', "''");
        let launched = std::process::Command::new("powershell")
            .args([
                "-NoExit",
                "-Command",
                &format!("Set-Location -LiteralPath '{}';", escaped),
            ])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
        drop(launched);
        Ok(true)
    }

    #[cfg(target_os = "linux")]
    {
        let status = crate::env_helper::create_silent_command("x-terminal-emulator")
            .args(["--working-directory", &path])
            .status()
            .map_err(|e| format!("打开终端失败: {}", e))?;
        Ok(status.success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_quoted_project_key_without_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".mise.toml");
        fs::write(
            &path,
            "[tools]\n\"node\" = \"20.0.0\"\n[env]\nKEEP = \"yes\"\n",
        )
        .unwrap();
        set_project_tool_version(
            dir.path().to_string_lossy().into_owned(),
            "node".into(),
            "22.0.0".into(),
        )
        .unwrap();
        let doc = fs::read_to_string(path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        assert_eq!(doc["tools"]["node"].as_str(), Some("22.0.0"));
        assert_eq!(doc["env"]["KEEP"].as_str(), Some("yes"));
    }

    #[test]
    fn preserves_asdf_format_and_other_tools() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".tool-versions");
        fs::write(&path, "node 20.0.0 # keep comment\npython 3.12.10\n").unwrap();
        set_project_tool_version(
            dir.path().to_string_lossy().into_owned(),
            "node".into(),
            "22.0.0".into(),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "node 22.0.0 # keep comment\npython 3.12.10\n"
        );
        assert!(!dir.path().join(".mise.toml").exists());
    }

    #[test]
    fn mise_toml_without_tools_table_does_not_hide_tool_versions() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("mise.toml"),
            "[env]\nNODE_ENV = \"development\"\n",
        )
        .unwrap();
        fs::write(dir.path().join(".tool-versions"), "node 20.1.0\npython 3.12.10\n")
            .unwrap();
        let (config_file, tools) = inspect_directory_tools(dir.path());
        assert_eq!(config_file.as_deref(), Some(".tool-versions"));
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].tool_id, "node");
    }

    #[test]
    fn mise_toml_with_tools_wins_over_tool_versions() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("mise.toml"), "[tools]\nnode = \"22.0.0\"\n").unwrap();
        fs::write(dir.path().join(".tool-versions"), "node 20.1.0\n").unwrap();
        let (config_file, tools) = inspect_directory_tools(dir.path());
        assert_eq!(config_file.as_deref(), Some("mise.toml"));
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].version, "22.0.0");
    }

    #[test]
    fn normalizes_trailing_separators_and_tilde() {
        let dir = tempfile::tempdir().unwrap();
        let plain = normalize_project_path(&dir.path().to_string_lossy());
        let trailing = normalize_project_path(&format!("{}/", dir.path().to_string_lossy()));
        assert_eq!(plain, trailing);
        if let Some(home) = dirs::home_dir() {
            assert_eq!(normalize_project_path("~"), home.canonicalize().unwrap_or(home));
        }
    }
}
