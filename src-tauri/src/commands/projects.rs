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
    dirs::home_dir().map(|h| h.join(".config/envhub/projects.json"))
}

fn load_saved_project_paths() -> Vec<String> {
    if let Some(p) = get_projects_config_path() {
        if p.exists() {
            if let Ok(content) = fs::read_to_string(p) {
                if let Ok(paths) = serde_json::from_str::<Vec<String>>(&content) {
                    return paths;
                }
            }
        }
    }
    Vec::new()
}

fn save_project_paths(paths: &[String]) {
    if let Some(p) = get_projects_config_path() {
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(paths) {
            let _ = fs::write(p, content);
        }
    }
}

/// Helper to extract real tools from a directory
fn inspect_directory_tools(dir: &Path) -> (Option<String>, Vec<ProjectTool>) {
    let mut config_file = None;
    let mut tools = Vec::new();

    // 1. Check .mise.toml
    let mise_toml = dir.join(".mise.toml");
    if mise_toml.exists() {
        config_file = Some(".mise.toml".to_string());
        if let Ok(content) = fs::read_to_string(&mise_toml) {
            let mut in_tools_section = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "[tools]" {
                    in_tools_section = true;
                    continue;
                } else if trimmed.starts_with('[') {
                    in_tools_section = false;
                }
                if in_tools_section && trimmed.contains('=') {
                    let parts: Vec<&str> = trimmed.split('=').collect();
                    if parts.len() == 2 {
                        let tool_id = parts[0].trim().trim_matches('"').trim_matches('\'').to_string();
                        let version = parts[1].trim().trim_matches('"').trim_matches('\'').to_string();
                        tools.push(ProjectTool { tool_id, version });
                    }
                }
            }
        }
    }

    // 2. Check .tool-versions (asdf / mise standard)
    let tool_versions = dir.join(".tool-versions");
    if tool_versions.exists() && config_file.is_none() {
        config_file = Some(".tool-versions".to_string());
        if let Ok(content) = fs::read_to_string(&tool_versions) {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    tools.push(ProjectTool {
                        tool_id: parts[0].to_string(),
                        version: parts[1].to_string(),
                    });
                }
            }
        }
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
                    if let Some(engine_node) = val.get("engines").and_then(|e| e.get("node")).and_then(|n| n.as_str()) {
                        node_ver = engine_node.to_string();
                    }
                }
            }
            tools.push(ProjectTool { tool_id: "node".to_string(), version: node_ver });
        }
    }

    // 4. Check Cargo.toml (Rust)
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() && !tools.iter().any(|t| t.tool_id == "rust") {
        tools.push(ProjectTool { tool_id: "rust".to_string(), version: "system".to_string() });
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
        tools.push(ProjectTool { tool_id: "go".to_string(), version: go_ver });
    }

    (config_file, tools)
}

#[tauri::command]
pub async fn get_projects() -> Result<Vec<ProjectEnv>, String> {
    let saved_paths = load_saved_project_paths();
    let mut projects = Vec::new();

    for path_str in saved_paths {
        let p = Path::new(&path_str);
        if p.exists() {
            let (config_file, tools) = inspect_directory_tools(p);
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "project".to_string());
            projects.push(ProjectEnv {
                id: format!("p_{}", path_str.replace('/', "_").replace('\\', "_")),
                name,
                path: path_str,
                config_file,
                tools,
                last_modified: Some("已绑定".to_string()),
            });
        }
    }

    Ok(projects)
}

#[tauri::command]
pub async fn scan_and_add_project(path: String) -> Result<ProjectEnv, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("指定路径不存在: {}", path));
    }

    let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "project".to_string());
    let (config_file, mut tools) = inspect_directory_tools(p);

    if tools.is_empty() {
        tools.push(ProjectTool {
            tool_id: "node".to_string(),
            version: "latest".to_string(),
        });
    }

    let mut saved = load_saved_project_paths();
    if !saved.contains(&path) {
        saved.push(path.clone());
        save_project_paths(&saved);
    }

    Ok(ProjectEnv {
        id: format!("p_{}", path.replace('/', "_").replace('\\', "_")),
        name,
        path,
        config_file,
        tools,
        last_modified: Some("刚刚添加".to_string()),
    })
}

#[tauri::command]
pub async fn remove_project(path: String) -> Result<bool, String> {
    let mut saved = load_saved_project_paths();
    saved.retain(|p| p != &path);
    save_project_paths(&saved);
    Ok(true)
}

#[tauri::command]
pub async fn set_project_tool_version(project_path: String, tool_id: String, version: String) -> Result<bool, String> {
    let target_dir = PathBuf::from(&project_path);
    let mise_toml_path = target_dir.join(".mise.toml");

    let mut lines = Vec::new();
    let mut found_tool = false;
    let mut in_tools = false;

    if mise_toml_path.exists() {
        if let Ok(content) = fs::read_to_string(&mise_toml_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "[tools]" {
                    in_tools = true;
                    lines.push(line.to_string());
                    continue;
                } else if trimmed.starts_with('[') {
                    if in_tools && !found_tool {
                        lines.push(format!("{} = \"{}\"", tool_id, version));
                        found_tool = true;
                    }
                    in_tools = false;
                }

                if in_tools && trimmed.starts_with(&tool_id) && trimmed.contains('=') {
                    lines.push(format!("{} = \"{}\"", tool_id, version));
                    found_tool = true;
                } else {
                    lines.push(line.to_string());
                }
            }
        }
    }

    if !found_tool {
        if !lines.iter().any(|l| l.trim() == "[tools]") {
            lines.push("[tools]".to_string());
        }
        lines.push(format!("{} = \"{}\"", tool_id, version));
    }

    fs::write(mise_toml_path, lines.join("\n") + "\n").map_err(|e| format!("写入 .mise.toml 失败: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub async fn open_in_editor(path: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("open")
            .args(["-a", "Visual Studio Code", &path])
            .status()
            .map_err(|e| format!("打开 VS Code 失败: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("cmd")
            .args(["/c", "code", &path])
            .status()
            .map_err(|e| format!("打开 VS Code 失败: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("code")
            .arg(&path)
            .status()
            .map_err(|e| format!("打开 VS Code 失败: {}", e))?;
        return Ok(status.success());
    }
}

#[tauri::command]
pub async fn open_in_terminal(path: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("open")
            .args(["-a", "Terminal", &path])
            .status()
            .map_err(|e| format!("打开终端失败: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("cmd")
            .args(["/c", "start", "wt", "-d", &path])
            .status()
            .map_err(|e| format!("打开终端失败: {}", e))?;
        return Ok(status.success());
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("x-terminal-emulator")
            .args(["--working-directory", &path])
            .status()
            .map_err(|e| format!("打开终端失败: {}", e))?;
        return Ok(status.success());
    }
}
