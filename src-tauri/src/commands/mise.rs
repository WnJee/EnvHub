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
    mise_aliases: &'static [&'static str],
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
        mise_aliases: &["node", "nodejs"],
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
        mise_aliases: &["python"],
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
        mise_aliases: &["go", "golang"],
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
        mise_aliases: &["rust"],
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
        mise_aliases: &["java"],
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
        mise_aliases: &["bun"],
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
        mise_aliases: &["deno"],
    },
    ToolMeta {
        id: "ruby",
        name: "Ruby",
        category: "runtime",
        description: "优雅简洁的动态编程语言，Rails 框架基石",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/ruby/ruby-original.svg",
        official_site: "https://www.ruby-lang.org",
        exec_name: "ruby",
        version_args: &["-v"],
        mise_aliases: &["ruby"],
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
        mise_aliases: &["php"],
    },
];

/// Robust extraction of clean semantic version number from CLI stdout
fn parse_version_output(_tool_id: &str, output: &str) -> Option<String> {
    let text = output.trim();
    if text.is_empty() {
        return None;
    }

    // Split words on whitespace, quotes, parentheses, brackets, colons
    for token in text.split(|c: char| c.is_whitespace() || c == '"' || c == '(' || c == ')' || c == '[' || c == ']' || c == ',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Strip leading non-digits (e.g. 'v', 'go', 'ruby-', 'Python')
        let candidate = trimmed.trim_start_matches(|c: char| !c.is_ascii_digit());
        if candidate.is_empty() {
            continue;
        }

        // Take only digits and dots (stopping before 'p', '+', '-', 'beta', date, etc.)
        let semver: String = candidate.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        let clean_semver = semver.trim_end_matches('.');
        
        // Ensure it contains at least one dot (e.g. "3.4.4", "22.12.0", "1.94")
        if clean_semver.matches('.').count() >= 1 && clean_semver.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return Some(clean_semver.to_string());
        }
    }

    None
}

#[tauri::command]
pub async fn get_runtimes() -> Result<Vec<RuntimeTool>, String> {
    // Ensure the latest prioritized PATH is active
    env_helper::fix_system_path();

    let mise_bin = env_helper::find_mise_binary();
    let mut tools = Vec::new();

    for meta in SUPPORTED_TOOLS {
        let mut installed_versions: Vec<String> = Vec::new();
        let mut active_version: Option<String> = None;
        let mut global_version: Option<String> = None;
        let mut available_versions: Vec<String> = Vec::new();

        // 1. Probe host system executable
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

        // Fallback for python: probe "python" if "python3" was not found
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
            for alias in meta.mise_aliases {
                if let Ok(output) = std::process::Command::new(bin).args(["ls", "--json", alias]).output() {
                    if output.status.success() {
                        if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                            if let Some(arr) = json_val.as_array() {
                                for item in arr {
                                    if let Some(ver_str) = item.get("version").and_then(|v| v.as_str()) {
                                        let clean_v = parse_version_output(meta.id, ver_str).unwrap_or_else(|| ver_str.to_string());
                                        if !installed_versions.contains(&clean_v) {
                                            installed_versions.push(clean_v.clone());
                                        }
                                        if item.get("active").and_then(|a| a.as_bool()).unwrap_or(false) {
                                            active_version = Some(clean_v.clone());
                                            global_version = Some(clean_v);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Remote versions from mise ls-remote
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
    Ok(vec![])
}

#[tauri::command]
pub async fn set_global_version(tool_id: String, version: String) -> Result<bool, String> {
    if let Some(bin) = env_helper::find_mise_binary() {
        let target = format!("{}@{}", tool_id, version);
        let status = Command::new(&bin)
            .args(["use", "-g", &target])
            .status()
            .await
            .map_err(|e| format!("无法执行 mise use: {}", e))?;

        if status.success() {
            return Ok(true);
        }
    }

    Ok(true)
}

#[tauri::command]
pub async fn uninstall_runtime_version(tool_id: String, version: String) -> Result<bool, String> {
    if let Some(bin) = env_helper::find_mise_binary() {
        let target = format!("{}@{}", tool_id, version);
        let status = Command::new(&bin)
            .args(["uninstall", &target])
            .status()
            .await
            .map_err(|e| format!("无法执行 mise uninstall: {}", e))?;

        return Ok(status.success());
    }
    Ok(true)
}

#[tauri::command]
pub async fn install_runtime_version(
    app: AppHandle,
    tool_id: String,
    version: String,
) -> Result<bool, String> {
    let mise_bin = env_helper::find_mise_binary()
        .ok_or_else(|| "未找到 Mise CLI 引擎，请先前往设置进行一键安装".to_string())?;

    let target = format!("{}@{}", tool_id, version);
    let _ = app.emit("install-log", format!("> mise install {}", target));
    let _ = app.emit("install-progress", 10);

    let mut child = Command::new(mise_bin)
        .args(["install", &target, "--verbose"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动安装进程失败: {}", e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app_clone = app.clone();
    if let Some(stdout) = stdout {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        tokio::spawn(async move {
            let mut progress = 15;
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_clone.emit("install-log", line.clone());
                
                let lower = line.to_lowercase();
                if lower.contains("download") || lower.contains("get ") {
                    progress = progress.max(35);
                } else if lower.contains("200 ok") || lower.contains("downloaded") {
                    progress = progress.max(65);
                } else if lower.contains("verify") || lower.contains("checksum") {
                    progress = progress.max(78);
                } else if lower.contains("extract") || lower.contains("install") {
                    progress = progress.max(88);
                } else if lower.contains("installed") || lower.contains("complete") {
                    progress = 98;
                } else if progress < 90 {
                    progress += 3;
                }
                let _ = app_clone.emit("install-progress", progress);
            }
        });
    }

    let app_clone2 = app.clone();
    if let Some(stderr) = stderr {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        tokio::spawn(async move {
            let mut progress = 20;
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_clone2.emit("install-log", line.clone());
                
                let lower = line.to_lowercase();
                if lower.contains("download") || lower.contains("get ") {
                    progress = progress.max(40);
                } else if lower.contains("200 ok") || lower.contains("downloaded") {
                    progress = progress.max(70);
                } else if lower.contains("extract") || lower.contains("install") {
                    progress = progress.max(88);
                } else if progress < 90 {
                    progress += 2;
                }
                let _ = app_clone2.emit("install-progress", progress);
            }
        });
    }

    let status = child.wait().await.map_err(|e| format!("等待安装完成失败: {}", e))?;
    if status.success() {
        let _ = app.emit("install-log", format!("✓ {} 安装成功！", target));
        let _ = app.emit("install-progress", 100);
        Ok(true)
    } else {
        let _ = app.emit("install-log", format!("✗ 安装失败，退出码: {:?}", status.code()));
        Ok(false)
    }
}

#[tauri::command]
pub async fn bootstrap_mise_cli() -> Result<bool, String> {
    let cmd = if cfg!(target_os = "windows") {
        "powershell -c \"irm https://mise.jdx.dev/install.ps1 | iex\""
    } else {
        "curl https://mise.run | sh"
    };

    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .await
        .map_err(|e| format!("执行自举脚本失败: {}", e))?;

    if status.success() {
        env_helper::fix_system_path();
        Ok(true)
    } else {
        Err(format!("自举脚本返回错误退出码: {:?}", status.code()))
    }
}
