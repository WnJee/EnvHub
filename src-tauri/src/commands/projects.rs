use serde::{Deserialize, Serialize};
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

#[tauri::command]
pub async fn get_projects() -> Result<Vec<ProjectEnv>, String> {
    // Return sample and current workspace
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let projects = vec![
        ProjectEnv {
            id: "p1".to_string(),
            name: "sharp-turing".to_string(),
            path: current_dir.to_string_lossy().to_string(),
            config_file: Some(".mise.toml".to_string()),
            tools: vec![
                ProjectTool { tool_id: "node".to_string(), version: "22.12.0".to_string() },
                ProjectTool { tool_id: "rust".to_string(), version: "1.83.0".to_string() },
            ],
            last_modified: Some("2026-09-02 14:00".to_string()),
        }
    ];

    Ok(projects)
}

#[tauri::command]
pub async fn scan_and_add_project(path: String) -> Result<ProjectEnv, String> {
    let p = Path::new(&path);
    let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "project".to_string());

    let mut config_file = None;
    let mut tools = Vec::new();

    if p.join(".mise.toml").exists() {
        config_file = Some(".mise.toml".to_string());
    } else if p.join(".tool-versions").exists() {
        config_file = Some(".tool-versions".to_string());
    } else if p.join("package.json").exists() {
        config_file = Some("package.json".to_string());
        tools.push(ProjectTool { tool_id: "node".to_string(), version: "22.12.0".to_string() });
    }

    if tools.is_empty() {
        tools.push(ProjectTool { tool_id: "node".to_string(), version: "22.12.0".to_string() });
    }

    Ok(ProjectEnv {
        id: format!("p_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        name,
        path,
        config_file,
        tools,
        last_modified: Some("刚刚".to_string()),
    })
}

#[tauri::command]
pub async fn set_project_tool_version(_project_id: String, _tool_id: String, _version: String) -> Result<bool, String> {
    // If mise binary exists, we can run `mise use <tool>@<version>` in the directory
    Ok(true)
}
