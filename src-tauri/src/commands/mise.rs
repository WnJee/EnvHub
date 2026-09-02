use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use crate::env_helper;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuntimeTool {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub icon: String,
    #[serde(rename = "officialSite")]
    pub official_site: String,
    #[serde(rename = "installedVersions")]
    pub installed_versions: Vec<String>,
    #[serde(rename = "activeVersion")]
    pub active_version: Option<String>,
    #[serde(rename = "globalVersion")]
    pub global_version: Option<String>,
    #[serde(rename = "availableVersions")]
    pub available_versions: Vec<String>,
}

#[derive(Clone)]
struct ToolMeta {
    id: &'static str,
    name: &'static str,
    category: &'static str,
    description: &'static str,
    icon: &'static str,
    official_site: &'static str,
    exec_name: &'static str,
    version_args: &'static [&'static str],
}

const SUPPORTED_TOOLS: &[ToolMeta] = &[
    ToolMeta {
        id: "node",
        name: "Node.js",
        category: "runtime",
        description: "JavaScript 运行时环境，支持海量 npm 生态",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/nodejs/nodejs-original.svg",
        official_site: "https://nodejs.org",
        exec_name: "node",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "python",
        name: "Python",
        category: "runtime",
        description: "通用高阶编程语言，广泛应用于 AI、数据科学与后端开发",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original.svg",
        official_site: "https://www.python.org",
        exec_name: "python3",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "go",
        name: "Go",
        category: "runtime",
        description: "Google 开发的静态编译型语言，极高并发与云原生标准",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/go/go-original.svg",
        official_site: "https://go.dev",
        exec_name: "go",
        version_args: &["version"],
    },
    ToolMeta {
        id: "rust",
        name: "Rust",
        category: "runtime",
        description: "注重内存安全、高性能与零成本抽象的系统级编程语言",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-original.svg",
        official_site: "https://www.rust-lang.org",
        exec_name: "rustc",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "java",
        name: "Java (OpenJDK)",
        category: "runtime",
        description: "跨平台企业级语言 (Temurin / Adoptium)",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/java/java-original.svg",
        official_site: "https://adoptium.net",
        exec_name: "java",
        version_args: &["-version"],
    },
    ToolMeta {
        id: "bun",
        name: "Bun",
        category: "runtime",
        description: "极速 All-in-One JavaScript 运行时、打包器与包管理器",
        icon: "https://bun.sh/logo.svg",
        official_site: "https://bun.sh",
        exec_name: "bun",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "deno",
        name: "Deno",
        category: "runtime",
        description: "下一代安全 JavaScript / TypeScript 运行时",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/denojs/denojs-original.svg",
        official_site: "https://deno.land",
        exec_name: "deno",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "ruby",
        name: "Ruby",
        category: "runtime",
        description: "优雅简洁的动态编程语言，Rails 框架基石",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/ruby/ruby-original.svg",
        official_site: "https://www.ruby-lang.org",
        exec_name: "ruby",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "php",
        name: "PHP",
        category: "runtime",
        description: "广受欢迎的 Web 服务端脚本语言",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/php/php-original.svg",
        official_site: "https://www.php.net",
        exec_name: "php",
        version_args: &["--version"],
    },
];

/// Helper to parse clean version string from command output
fn parse_version_output(tool_id: &str, output: &str) -> Option<String> {
    let text = output.trim();
    if text.is_empty() {
        return None;
    }

    match tool_id {
        "node" | "bun" | "deno" => {
            // "v22.12.0" -> "22.12.0" or "1.1.38"
            let v = text.lines().next()?.trim().trim_start_matches('v');
            let first_word = v.split_whitespace().next()?;
            Some(first_word.to_string())
        }
        "python" => {
            // "Python 3.12.7"
            let line = text.lines().next()?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(parts[1].to_string())
            } else {
                Some(line.to_string())
            }
        }
        "go" => {
            // "go version go1.23.3 darwin/arm64"
            let line = text.lines().next()?;
            for part in line.split_whitespace() {
                if let Some(stripped) = part.strip_prefix("go") {
                    if stripped.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        return Some(stripped.to_string());
                    }
                }
            }
            None
        }
        "rust" => {
            // "rustc 1.83.0 (90b35a623 2024-11-26)"
            let line = text.lines().next()?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(parts[1].to_string())
            } else {
                None
            }
        }
        "java" => {
            // "openjdk version "21.0.4" 2024-07-16 LTS"
            let first_line = text.lines().next()?;
            if let Some(start) = first_line.find('"') {
                if let Some(end) = first_line[start + 1..].find('"') {
                    return Some(first_line[start + 1..start + 1 + end].to_string());
                }
            }
            None
        }
        _ => {
            let line = text.lines().next()?;
            Some(line.trim().to_string())
        }
    }
}

#[tauri::command]
pub async fn get_runtimes() -> Result<Vec<RuntimeTool>, String> {
    let mise_bin = env_helper::find_mise_binary();
    let mut tools = Vec::new();

    for meta in SUPPORTED_TOOLS {
        let mut installed_versions: Vec<String> = Vec::new();
        let mut active_version: Option<String> = None;
        let mut global_version: Option<String> = None;
        let mut available_versions: Vec<String> = Vec::new();

        // 1. Probe host system directly for this executable
        let system_probe = std::process::Command::new(meta.exec_name)
            .args(meta.version_args)
            .output();

        if let Ok(out) = system_probe {
            if out.status.success() {
                let stdout_str = String::from_utf8_lossy(&out.stdout);
                let stderr_str = String::from_utf8_lossy(&out.stderr);
                let raw_ver = if !stdout_str.trim().is_empty() { stdout_str } else { stderr_str };

                if let Some(ver) = parse_version_output(meta.id, &raw_ver) {
                    if !installed_versions.contains(&ver) {
                        installed_versions.push(ver.clone());
                    }
                    active_version = Some(ver.clone());
                    global_version = Some(ver);
                }
            }
        }

        // Also check "python" if "python3" wasn't found
        if meta.id == "python" && installed_versions.is_empty() {
            if let Ok(out) = std::process::Command::new("python").arg("--version").output() {
                if out.status.success() {
                    let raw = String::from_utf8_lossy(&out.stdout);
                    if let Some(ver) = parse_version_output("python", &raw) {
                        installed_versions.push(ver.clone());
                        active_version = Some(ver.clone());
                        global_version = Some(ver);
                    }
                }
            }
        }

        // 2. Query Mise CLI if available
        if let Some(ref bin) = mise_bin {
            // Get all installed versions for this tool via mise
            if let Ok(output) = std::process::Command::new(bin).args(["ls", "--json", meta.id]).output() {
                if output.status.success() {
                    if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                        if let Some(arr) = json_val.as_array() {
                            for item in arr {
                                if let Some(ver_str) = item.get("version").and_then(|v| v.as_str()) {
                                    if !installed_versions.contains(&ver_str.to_string()) {
                                        installed_versions.push(ver_str.to_string());
                                    }
                                    if item.get("active").and_then(|a| a.as_bool()).unwrap_or(false) {
                                        active_version = Some(ver_str.to_string());
                                        global_version = Some(ver_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Also check remote versions for this tool (take top 25 latest versions)
            if let Ok(output) = std::process::Command::new(bin).args(["ls-remote", meta.id]).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut list: Vec<String> = stdout
                        .lines()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .rev()
                        .take(25)
                        .collect();
                    available_versions.append(&mut list);
                }
            }
        }

        tools.push(RuntimeTool {
            id: meta.id.to_string(),
            name: meta.name.to_string(),
            category: meta.category.to_string(),
            description: meta.description.to_string(),
            icon: meta.icon.to_string(),
            official_site: meta.official_site.to_string(),
            installed_versions,
            active_version,
            global_version,
            available_versions,
        });
    }

    Ok(tools)
}

#[tauri::command]
pub async fn get_remote_versions(tool_id: String) -> Result<Vec<String>, String> {
    if let Some(bin) = env_helper::find_mise_binary() {
        if let Ok(output) = std::process::Command::new(&bin).args(["ls-remote", &tool_id]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let list: Vec<String> = stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .rev()
                    .take(40)
                    .collect();
                return Ok(list);
            }
        }
    }

    // If mise is not installed, return empty list (no fake static data)
    Ok(Vec::new())
}

#[tauri::command]
pub async fn set_global_version(tool_id: String, version: String) -> Result<bool, String> {
    let mise_bin = env_helper::find_mise_binary().ok_or_else(|| {
        "未检测到 Mise CLI 引擎，无法修改全局生效版本，请先在设置中安装 Mise".to_string()
    })?;

    let arg = format!("{}@{}", tool_id, version);
    let output = std::process::Command::new(&mise_bin)
        .args(["use", "-g", &arg])
        .output()
        .map_err(|e| format!("执行 mise use 失败: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(true)
}

#[tauri::command]
pub async fn uninstall_runtime_version(tool_id: String, version: String) -> Result<bool, String> {
    let mise_bin = env_helper::find_mise_binary().ok_or_else(|| {
        "未检测到 Mise CLI 引擎，无法执行卸载操作".to_string()
    })?;

    let arg = format!("{}@{}", tool_id, version);
    let output = std::process::Command::new(&mise_bin)
        .args(["uninstall", &arg])
        .output()
        .map_err(|e| format!("执行 mise uninstall 失败: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(true)
}

#[tauri::command]
pub async fn install_runtime_version(
    app: AppHandle,
    tool_id: String,
    version: String,
) -> Result<bool, String> {
    let mise_bin = env_helper::find_mise_binary().ok_or_else(|| {
        let err_msg = "未检测到 Mise CLI 引擎，无法进行真实下载安装。请先在【客户端设置】中点击一键自举安装 Mise。";
        let _ = app.emit("install-log", format!("[ERROR] {}", err_msg));
        let _ = app.emit("install-progress", 0);
        err_msg.to_string()
    })?;

    let target = format!("{}@{}", tool_id, version);
    let _ = app.emit("install-log", format!("[mise] 正在向底层 CLI 提交真实安装任务: {}...", target));
    let _ = app.emit("install-progress", 10);

    let mut child = Command::new(&mise_bin)
        .args(["install", "-v", &target])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动安装子进程失败: {}", e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app_handle_1 = app.clone();
    if let Some(stdout) = stdout {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            let mut p = 15;
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app_handle_1.emit("install-log", line);
                p = (p + 3).min(90);
                let _ = app_handle_1.emit("install-progress", p);
            }
        });
    }

    let app_handle_2 = app.clone();
    if let Some(stderr) = stderr {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app_handle_2.emit("install-log", line);
            }
        });
    }

    let status = child.wait().await.map_err(|e| format!("等待安装完成失败: {}", e))?;
    if status.success() {
        let _ = app.emit("install-log", format!("[mise] 成功完成 {} 的真实安装与环境配置！", target));
        let _ = app.emit("install-progress", 100);
        Ok(true)
    } else {
        let _ = app.emit("install-log", format!("[mise] 安装退出，退出码: {:?}", status.code()));
        let _ = app.emit("install-progress", 0);
        Err(format!("安装失败，退出码: {:?}", status.code()))
    }
}

#[tauri::command]
pub async fn bootstrap_mise_cli() -> Result<bool, String> {
    #[cfg(unix)]
    {
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl https://mise.run | sh")
            .status()
            .await
            .map_err(|e| e.to_string())?;
        return Ok(status.success());
    }

    #[cfg(windows)]
    {
        let status = Command::new("powershell")
            .arg("-c")
            .arg("irm https://mise.run | iex")
            .status()
            .await
            .map_err(|e| e.to_string())?;
        return Ok(status.success());
    }
}
