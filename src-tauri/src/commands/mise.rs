use crate::env_helper;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};

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
    #[serde(rename = "managedVersions")]
    pub managed_versions: Vec<String>,
    #[serde(rename = "activeVersion")]
    pub active_version: Option<String>,
    #[serde(rename = "globalVersion")]
    pub global_version: Option<String>,
    #[serde(rename = "availableVersions")]
    pub available_versions: Vec<String>,
    #[serde(rename = "detectionError")]
    pub detection_error: Option<String>,
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
        name: "Java",
        category: "runtime",
        description: "跨平台企业级编程语言与通用运行环境",
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
        version_args: &["-v"],
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
    ToolMeta {
        id: "zig",
        name: "Zig",
        category: "runtime",
        description: "通用高性能系统编程语言与 C/C++ 极速交叉编译工具链",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/zig/zig-original.svg",
        official_site: "https://ziglang.org",
        exec_name: "zig",
        version_args: &["version"],
    },
    ToolMeta {
        id: "dotnet",
        name: ".NET SDK",
        category: "runtime",
        description: "微软开源跨平台开发平台，支持 C#、F# 与现代 Web/云原生应用",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/dot-net/dot-net-original.svg",
        official_site: "https://dotnet.microsoft.com",
        exec_name: "dotnet",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "dart",
        name: "Dart",
        category: "runtime",
        description: "客户端优化的快速响应语言，Flutter 跨平台开发核心驱动",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/dart/dart-original.svg",
        official_site: "https://dart.dev",
        exec_name: "dart",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "flutter",
        name: "Flutter",
        category: "runtime",
        description: "Google 跨平台 UI 软件开发工具包，支持移动、Web 与桌面端",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/flutter/flutter-original.svg",
        official_site: "https://flutter.dev",
        exec_name: "flutter",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "kotlin",
        name: "Kotlin",
        category: "runtime",
        description: "运行于 JVM 的现代静态类型语言，Android 官方首选开发语言",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/kotlin/kotlin-original.svg",
        official_site: "https://kotlinlang.org",
        exec_name: "kotlinc",
        version_args: &["-version"],
    },
    ToolMeta {
        id: "elixir",
        name: "Elixir",
        category: "runtime",
        description: "构建于 Erlang VM 之上的高可扩展、高并发函数式编程语言",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/elixir/elixir-original.svg",
        official_site: "https://elixir-lang.org",
        exec_name: "elixir",
        version_args: &["--version"],
    },
    ToolMeta {
        id: "erlang",
        name: "Erlang / OTP",
        category: "runtime",
        description: "极高容错与并发分布式系统的工业级编程语言与运行时",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/erlang/erlang-original.svg",
        official_site: "https://www.erlang.org",
        exec_name: "erl",
        version_args: &[
            "-eval",
            "erlang:display(erlang:system_info(otp_release)), halt().",
            "-noshell",
        ],
    },
    ToolMeta {
        id: "lua",
        name: "Lua",
        category: "runtime",
        description: "轻量、紧凑且快速的可嵌入式脚本语言，广泛用于游戏与 Nginx 生态",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/lua/lua-original.svg",
        official_site: "https://www.lua.org",
        exec_name: "lua",
        version_args: &["-v"],
    },
    ToolMeta {
        id: "terraform",
        name: "Terraform",
        category: "runtime",
        description: "HashiCorp 基础设施即代码 (IaC) 多云资源编排标准工具",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/terraform/terraform-original.svg",
        official_site: "https://www.terraform.io",
        exec_name: "terraform",
        version_args: &["version"],
    },
];

/// Robust extraction of clean semantic version number from CLI stdout
fn parse_version_output(_tool_id: &str, output: &str) -> Option<String> {
    let text = output.trim();
    if text.is_empty() {
        return None;
    }

    // Split words on whitespace, quotes, parentheses, brackets, colons
    for token in text.split(|c: char| {
        c.is_whitespace() || c == '"' || c == '(' || c == ')' || c == '[' || c == ']' || c == ','
    }) {
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
        let semver: String = candidate
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let clean_semver = semver.trim_end_matches('.');

        // Ensure it contains at least one dot (e.g. "3.4.4", "22.12.0", "1.94")
        if clean_semver.matches('.').count() >= 1
            && clean_semver
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return Some(clean_semver.to_string());
        }
    }

    None
}

fn get_mise_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".config/mise/config.toml"));
        paths.push(home.join(".mise.toml"));
        paths.push(home.join(".tool-versions"));
    }
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("mise/config.toml"));
        paths.push(config_dir.join("mise\\config.toml"));
    }
    if let Some(data_local) = dirs::data_local_dir() {
        paths.push(data_local.join("mise/config.toml"));
        paths.push(data_local.join("mise\\config.toml"));
    }
    if let Some(config) = std::env::var_os("MISE_GLOBAL_CONFIG_FILE") {
        paths.push(config.into());
    }
    if let Some(config) = std::env::var_os("MISE_CONFIG_DIR") {
        paths.push(PathBuf::from(config).join("config.toml"));
    }
    paths.sort();
    paths.dedup();
    paths
}

/// Runtime management is global. Always execute mise from the user's home
/// directory so a project's untrusted `.mise.toml` cannot block the desktop
/// application or accidentally influence a global install.
fn mise_working_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(std::env::temp_dir)
}

fn installed_from_json(value: &serde_json::Value) -> (Vec<String>, Option<String>) {
    let mut installed = Vec::new();
    let mut active = None;
    if let Some(items) = value.as_array() {
        for item in items {
            if item["installed"].as_bool() != Some(true) {
                continue;
            }
            if let Some(version) = item["version"].as_str() {
                if !installed.iter().any(|v| v == version) {
                    installed.push(version.to_string());
                }
                if item["active"].as_bool() == Some(true) {
                    active = Some(version.to_string());
                }
            }
        }
    }
    (installed, active)
}

async fn mise_versions(bin: &str, tool: &str) -> Result<serde_json::Value, String> {
    let output = env_helper::checked_output(
        env_helper::output_timeout(
            env_helper::create_silent_tokio_command(bin)
                .current_dir(mise_working_dir())
                .args(["ls", "--json", tool]),
            8,
        )
        .await?,
    )?;
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_runtimes() -> Result<Vec<RuntimeTool>, String> {
    let bin = env_helper::find_mise_binary();
    let mut tasks = tokio::task::JoinSet::new();
    let limit = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
    for (index, meta) in SUPPORTED_TOOLS.iter().cloned().enumerate() {
        let bin = bin.clone();
        let limit = limit.clone();
        tasks.spawn(async move {
            let _permit = limit.acquire().await.map_err(|e| e.to_string())?;
            let (mut installed, mut active) = (Vec::new(), None);
            let mut detection_error = None;
            if let Some(bin) = bin {
                match mise_versions(&bin.to_string_lossy(), meta.id).await {
                    Ok(json) => {
                        (installed, active) = installed_from_json(&json);
                    }
                    Err(error) => detection_error = Some(error),
                }
            }
            let managed = installed.clone();
            if active.is_none() {
                let exec = if cfg!(windows) && meta.id == "python" {
                    "python"
                } else {
                    meta.exec_name
                };
                if let Ok(output) = env_helper::output_timeout(
                    env_helper::create_silent_tokio_command(exec)
                        .current_dir(mise_working_dir())
                        .env("MISE_AUTO_INSTALL", "false")
                        .args(meta.version_args),
                    4,
                )
                .await
                {
                    if output.status.success() {
                        let text = format!(
                            "{} {}",
                            String::from_utf8_lossy(&output.stdout),
                            String::from_utf8_lossy(&output.stderr)
                        );
                        if let Some(version) = parse_version_output(meta.id, &text) {
                            if !installed.contains(&version) {
                                installed.push(version.clone());
                            }
                            active = Some(version);
                        }
                    }
                }
            }
            Ok::<_, String>((
                index,
                RuntimeTool {
                    id: meta.id.into(),
                    name: meta.name.into(),
                    category: meta.category.into(),
                    description: meta.description.into(),
                    icon: meta.icon.into(),
                    official_site: meta.official_site.into(),
                    installed_versions: installed,
                    managed_versions: managed,
                    active_version: active.clone(),
                    global_version: active,
                    available_versions: Vec::new(),
                    detection_error,
                },
            ))
        });
    }
    let mut result = Vec::new();
    while let Some(task) = tasks.join_next().await {
        result.push(task.map_err(|e| e.to_string())??);
    }
    result.sort_by_key(|(index, _)| *index);
    Ok(result.into_iter().map(|(_, tool)| tool).collect())
}

// Keep original identifiers: distro prefixes, prereleases and build suffixes are executable targets.
fn filter_latest_minor_versions(_tool_id: &str, versions: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    versions
        .into_iter()
        .rev()
        .filter_map(|v| {
            let v = v.trim().to_string();
            (!v.is_empty() && seen.insert(v.clone())).then_some(v)
        })
        .collect()
}

type RemoteCache = std::collections::HashMap<String, (std::time::Instant, Vec<String>)>;
static REMOTE_QUERIES: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(3);
static REMOTE_CACHE: std::sync::LazyLock<tokio::sync::Mutex<RemoteCache>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(std::collections::HashMap::new()));

#[tauri::command]
pub async fn get_remote_versions(tool_id: String) -> Result<Vec<String>, String> {
    env_helper::validate_target(&tool_id, "latest")?;
    if let Some((when, values)) = REMOTE_CACHE.lock().await.get(&tool_id) {
        if when.elapsed() < std::time::Duration::from_secs(600) {
            return Ok(values.clone());
        }
    }
    let _permit = REMOTE_QUERIES.acquire().await.map_err(|e| e.to_string())?;
    let bin = env_helper::find_mise_binary().ok_or("Mise CLI is not installed")?;
    let output = env_helper::checked_output(
        env_helper::output_timeout(
            env_helper::create_silent_tokio_command(&bin.to_string_lossy())
                .current_dir(mise_working_dir())
                .args(["ls-remote", &tool_id]),
            20,
        )
        .await?,
    )?;
    let values = filter_latest_minor_versions(
        &tool_id,
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect(),
    );
    REMOTE_CACHE
        .lock()
        .await
        .insert(tool_id, (std::time::Instant::now(), values.clone()));
    Ok(values)
}

#[tauri::command]
pub async fn set_global_version(tool_id: String, version: String) -> Result<bool, String> {
    let _operation = env_helper::begin_operation()?;
    env_helper::validate_target(&tool_id, &version)?;
    let bin = env_helper::find_mise_binary().ok_or("Mise CLI is not installed")?;
    let (installed, _) =
        installed_from_json(&mise_versions(&bin.to_string_lossy(), &tool_id).await?);
    if !installed.contains(&version) {
        return Err("Install this version with Mise before activating it".into());
    }
    env_helper::checked_output(
        env_helper::output_timeout(
            env_helper::create_silent_tokio_command(&bin.to_string_lossy())
                .current_dir(mise_working_dir())
                .args(["use", "-g", &format!("{tool_id}@{version}")]),
            30,
        )
        .await?,
    )?;
    Ok(true)
}

fn remove_exact_tool(doc: &mut toml_edit::DocumentMut, tool: &str, version: &str) {
    if let Some(tools) = doc.get_mut("tools").and_then(|t| t.as_table_like_mut()) {
        let matches = tools.get(tool).and_then(|v| v.as_str()) == Some(version);
        if matches {
            tools.remove(tool);
        }
    }
}

#[tauri::command]
pub async fn uninstall_runtime_version(tool_id: String, version: String) -> Result<bool, String> {
    let _operation = env_helper::begin_operation()?;
    env_helper::validate_target(&tool_id, &version)?;
    let bin = env_helper::find_mise_binary().ok_or("Mise CLI is not installed")?;
    let (installed, _) =
        installed_from_json(&mise_versions(&bin.to_string_lossy(), &tool_id).await?);
    if !installed.contains(&version) {
        return Err(
            "This is not an installed Mise version; use its original package manager".into(),
        );
    }
    env_helper::checked_output(
        env_helper::output_timeout(
            env_helper::create_silent_tokio_command(&bin.to_string_lossy())
                .current_dir(mise_working_dir())
                .args(["uninstall", &format!("{tool_id}@{version}")]),
            60,
        )
        .await?,
    )?;
    for path in get_mise_config_paths() {
        if !path.is_file() {
            continue;
        }
        if path.file_name().and_then(|s| s.to_str()) == Some(".tool-versions") {
            let _guard = crate::config_file::CONFIG_LOCK
                .lock()
                .map_err(|e| e.to_string())?;
            let content = crate::config_file::read(&path)?;
            let new = content
                .lines()
                .filter(|line| {
                    let fields: Vec<_> = line
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .split_whitespace()
                        .collect();
                    fields != [tool_id.as_str(), version.as_str()]
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            if new != content {
                crate::config_file::write(&path, &new)?;
            }
        } else {
            crate::config_file::edit_toml(&path, |doc| {
                remove_exact_tool(doc, &tool_id, &version);
                Ok(())
            })?;
        }
    }
    Ok(true)
}

#[tauri::command]
pub async fn install_runtime_version(
    app: AppHandle,
    tool_id: String,
    version: String,
) -> Result<bool, String> {
    let _operation = env_helper::begin_operation()?;
    env_helper::validate_target(&tool_id, &version)?;

    let mise_bin = match env_helper::find_mise_binary() {
        Some(b) => b,
        None => {
            let err = "未找到 Mise CLI 引擎，请先点击右上角【一键部署 Mise 引擎】".to_string();
            let _ = app.emit("install-log", format!("❌ {}", err));
            let _ = app.emit("install-progress", 0);
            return Err(err);
        }
    };

    let target = format!("{}@{}", tool_id, version);
    let _ = app.emit(
        "install-log",
        format!("> {} install {}", mise_bin.display(), target),
    );
    let _ = app.emit("install-progress", 10);

    let mut cmd = env_helper::create_silent_tokio_command(&mise_bin.to_string_lossy());
    cmd.current_dir(mise_working_dir())
        .args(["install", &target, "--verbose"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动安装进程失败: {}", e))?;
    let child_pid = child.id();
    env_helper::set_active_install_pid(child_pid);

    let mut readers = Vec::new();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app_clone = app.clone();
    if let Some(stdout) = stdout {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        readers.push(tokio::spawn(async move {
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
        }));
    }

    let app_clone2 = app.clone();
    if let Some(stderr) = stderr {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        readers.push(tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_clone2.emit("install-log", line.clone());
            }
        }));
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("等待安装完成失败: {}", e))?;
    for reader in readers {
        let _ = reader.await;
    }
    env_helper::clear_active_install_pid(child_pid);
    if status.success() {
        let _ = app.emit(
            "install-log",
            format!("✓ {} 安装成功！已完成环境注入与软链接配置", target),
        );
        let _ = app.emit("install-progress", 100);
        Ok(true)
    } else {
        let _ = app.emit(
            "install-log",
            format!("❌ 安装失败，退出码: {:?}", status.code()),
        );
        if cfg!(target_os = "windows") {
            let _ = app.emit("install-log", "💡 提示: Windows 环境下部分语言（如 Python/Node）需预先安装 7-Zip、Git 或 Visual C++ 运行库以支持解压。可在【系统工具箱】一键安装 Git 与 Scoop。".to_string());
        }
        Ok(false)
    }
}

#[tauri::command]
pub async fn cancel_current_install() -> Result<bool, String> {
    env_helper::cancel_active_install()
}

#[tauri::command]
pub async fn bootstrap_mise_cli(app: AppHandle) -> Result<bool, String> {
    let _operation = env_helper::begin_operation()?;
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = env_helper::create_silent_tokio_command("powershell");
        command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "$ErrorActionPreference='Stop'; [Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; irm https://mise.jdx.dev/install.ps1 | iex"]);
        command
    };
    #[cfg(not(target_os = "windows"))]
    let mut command = {
        let mut command = env_helper::create_silent_tokio_command("bash");
        command.args([
            "-c",
            "set -o pipefail; curl -fsSL --connect-timeout 10 --max-time 120 https://mise.run | sh",
        ]);
        command
    };
    command.current_dir(mise_working_dir());
    super::system::stream_install(&app, command).await?;
    let bin = env_helper::find_mise_binary()
        .ok_or("Installer exited but Mise executable was not found")?;
    env_helper::checked_output(
        env_helper::output_timeout(
            env_helper::create_silent_tokio_command(&bin.to_string_lossy()).arg("--version"),
            5,
        )
        .await?,
    )?;
    Ok(true)
}

#[tauri::command]
pub async fn open_terminal_for_runtime(tool_id: String, version: String) -> Result<String, String> {
    env_helper::validate_target(&tool_id, &version)?;

    let (exec, args): (&str, &[&str]) = match tool_id.as_str() {
        "node" => ("node", &["--version"]),
        "python" => {
            #[cfg(target_os = "windows")]
            {
                ("python", &["--version"])
            }
            #[cfg(not(target_os = "windows"))]
            {
                ("python3", &["--version"])
            }
        }
        "go" => ("go", &["version"]),
        "rust" => ("rustc", &["--version"]),
        "java" => ("java", &["-version"]),
        "ruby" => ("ruby", &["-v"]),
        "bun" => ("bun", &["--version"]),
        "deno" => ("deno", &["--version"]),
        "php" => ("php", &["--version"]),
        "zig" => ("zig", &["version"]),
        "dotnet" => ("dotnet", &["--version"]),
        "dart" => ("dart", &["--version"]),
        "flutter" => ("flutter", &["--version"]),
        "kotlin" => ("kotlinc", &["-version"]),
        "elixir" => ("elixir", &["--version"]),
        "erlang" => (
            "erl",
            &[
                "-eval",
                "erlang:display(erlang:system_info(otp_release)), halt().",
                "-noshell",
            ],
        ),
        "lua" => ("lua", &["-v"]),
        "terraform" => ("terraform", &["version"]),
        _ => (&tool_id, &["--version"]),
    };

    let mise = env_helper::find_mise_binary().ok_or_else(|| "未找到 Mise CLI".to_string())?;
    let target = format!("{}@{}", tool_id, version);
    let command_args = args
        .iter()
        .map(|a| env_helper::shell_quote(a))
        .collect::<Vec<_>>()
        .join(" ");

    #[cfg(target_os = "macos")]
    {
        let script_content = format!(
            "#!/bin/bash\nclear\necho '========================================'\necho '  ⚡ EnvHub 终端环境检测: {} {}'\necho '========================================'\necho ''\n{} exec '{}' -- {} {}\necho ''\necho '----------------------------------------'\necho '已在当前版本环境中执行完毕，当前终端会话保持就绪：'\nexec $SHELL -l\n",
            tool_id, version, env_helper::shell_quote(&mise.to_string_lossy()), target, exec, command_args
        );
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join(format!("envhub_check_{}.command", tool_id));
        std::fs::write(&script_path, script_content).map_err(|e| e.to_string())?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let status = std::process::Command::new("open")
            .args(["-a", "Terminal"])
            .arg(&script_path)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("Failed to open Terminal".into());
        }
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "Write-Host '=== EnvHub 环境检测: {} {} ===' -ForegroundColor Cyan; & '{}' exec '{}' -- {} {}; Write-Host ''; Write-Host '可关闭此终端窗口' -ForegroundColor DarkGray",
            tool_id, version, mise.to_string_lossy().replace('\'', "''"), target, exec, command_args
        );
        std::process::Command::new("cmd.exe")
            .args([
                "/c",
                "start",
                "",
                "powershell.exe",
                "-NoExit",
                "-NoProfile",
                "-Command",
                &script,
            ])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let command = format!(
            "{} exec '{}' -- {} {}; exec $SHELL",
            env_helper::shell_quote(&mise.to_string_lossy()),
            target,
            exec,
            command_args
        );
        std::process::Command::new("x-terminal-emulator")
            .args(["-e", "sh", "-lc", &command])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
    }

    Ok(format!("已在终端执行 {} {}", exec, command_args))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_and_vendor_versions() {
        let value = serde_json::json!([
            {"version":"99.0.0", "installed":false, "active":false},
            {"version":"temurin-21.0.2+13", "installed":true, "active":true}
        ]);
        let (versions, active) = installed_from_json(&value);
        assert_eq!(versions, ["temurin-21.0.2+13"]);
        assert_eq!(active.as_deref(), Some("temurin-21.0.2+13"));
    }
    #[test]
    fn remote_targets_are_lossless() {
        assert_eq!(
            filter_latest_minor_versions("node", vec!["23.0.0-rc.1".into()]),
            ["23.0.0-rc.1"]
        );
    }
    #[test]
    fn uninstall_only_removes_exact_tool() {
        let mut doc = "[tools]\npython = \"3.12.10\"\n[env]\npython_service = \"3.12.1\"\n"
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        remove_exact_tool(&mut doc, "python", "3.12.1");
        assert_eq!(doc["tools"]["python"].as_str(), Some("3.12.10"));
        remove_exact_tool(&mut doc, "python", "3.12.10");
        assert!(doc["tools"].get("python").is_none());
        assert_eq!(doc["env"]["python_service"].as_str(), Some("3.12.1"));
    }
}
