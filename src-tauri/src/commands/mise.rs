use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
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
    ToolMeta {
        id: "zig",
        name: "Zig",
        category: "runtime",
        description: "通用高性能系统编程语言与 C/C++ 极速交叉编译工具链",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/zig/zig-original.svg",
        official_site: "https://ziglang.org",
        exec_name: "zig",
        version_args: &["version"],
        mise_aliases: &["zig"],
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
        mise_aliases: &["dotnet", "dotnet-core"],
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
        mise_aliases: &["dart"],
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
        mise_aliases: &["flutter"],
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
        mise_aliases: &["kotlin"],
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
        mise_aliases: &["elixir"],
    },
    ToolMeta {
        id: "erlang",
        name: "Erlang / OTP",
        category: "runtime",
        description: "极高容错与并发分布式系统的工业级编程语言与运行时",
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/erlang/erlang-original.svg",
        official_site: "https://www.erlang.org",
        exec_name: "erl",
        version_args: &["-eval", "erlang:display(erlang:system_info(otp_release)), halt().", "-noshell"],
        mise_aliases: &["erlang"],
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
        mise_aliases: &["lua"],
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
        mise_aliases: &["terraform"],
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

/// Check if a version string is a valid semantic version (not 'latest', 'v1', 'lts')
fn is_valid_semver(v: &str) -> bool {
    let clean = v.trim().trim_start_matches('v');
    if clean.is_empty() || clean == "latest" || clean == "lts" || clean == "system" {
        return false;
    }
    clean.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && clean.contains('.')
}

/// Deduplicate installed versions: Remove redundant prefix versions (e.g. "21.0" or "21" when "21.0.2" is installed)
fn clean_and_deduplicate_installed_versions(raw_versions: Vec<String>) -> Vec<String> {
    let mut semvers: Vec<String> = Vec::new();

    for v in raw_versions {
        let trimmed = v.trim().trim_start_matches('v').to_string();
        if trimmed.is_empty() || trimmed == "latest" || trimmed == "lts" || trimmed == "system" {
            continue;
        }
        if !trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) || !trimmed.contains('.') {
            continue;
        }
        if !semvers.contains(&trimmed) {
            semvers.push(trimmed);
        }
    }

    // Filter out prefix versions if a more specific patch version exists
    let mut final_versions: Vec<String> = Vec::new();
    for v in &semvers {
        let is_prefix_of_more_specific = semvers.iter().any(|other| {
            other != v && other.starts_with(&format!("{}.", v))
        });
        if !is_prefix_of_more_specific {
            final_versions.push(v.clone());
        }
    }

    // Sort descending by semver components
    final_versions.sort_by(|a, b| {
        let a_parts: Vec<u64> = a.split('.').filter_map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok()).collect();
        let b_parts: Vec<u64> = b.split('.').filter_map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok()).collect();
        b_parts.cmp(&a_parts)
    });

    final_versions
}

fn get_mise_install_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/mise/installs"));
    }
    if let Some(data_local) = dirs::data_local_dir() {
        dirs.push(data_local.join("mise/installs"));
        dirs.push(data_local.join("mise\\installs"));
    }
    dirs
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
    paths
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

        // 1. Probe host system executable with fallback binary paths
        let mut probe_execs = vec![meta.exec_name.to_string()];
        if let Some(home) = dirs::home_dir() {
            match meta.id {
                "go" => {
                    probe_execs.push("/usr/local/go/bin/go".into());
                    probe_execs.push("/opt/homebrew/bin/go".into());
                    probe_execs.push(home.join("go/bin/go").to_string_lossy().to_string());
                    probe_execs.push(home.join(".local/share/mise/shims/go").to_string_lossy().to_string());
                    probe_execs.push("C:\\Program Files\\Go\\bin\\go.exe".into());
                }
                "python" => {
                    probe_execs.push("python".into());
                    probe_execs.push("/opt/homebrew/bin/python3".into());
                    probe_execs.push("/usr/local/bin/python3".into());
                }
                "rust" => {
                    probe_execs.push(home.join(".cargo/bin/rustc").to_string_lossy().to_string());
                    probe_execs.push(home.join(".cargo/bin/rustc.exe").to_string_lossy().to_string());
                    probe_execs.push("/opt/homebrew/bin/rustc".into());
                }
                "node" => {
                    probe_execs.push("/opt/homebrew/bin/node".into());
                    probe_execs.push("/usr/local/bin/node".into());
                    probe_execs.push("C:\\Program Files\\nodejs\\node.exe".into());
                    probe_execs.push(home.join(".local/share/mise/shims/node").to_string_lossy().to_string());
                }
                "bun" => {
                    probe_execs.push(home.join(".bun/bin/bun").to_string_lossy().to_string());
                    probe_execs.push(home.join(".bun/bin/bun.exe").to_string_lossy().to_string());
                }
                "deno" => {
                    probe_execs.push(home.join(".deno/bin/deno").to_string_lossy().to_string());
                    probe_execs.push(home.join(".deno/bin/deno.exe").to_string_lossy().to_string());
                }
                _ => {}
            }
        }

        for exec_path in probe_execs {
            if let Ok(out) = env_helper::create_silent_command(&exec_path).args(meta.version_args).output() {
                if out.status.success() || (meta.id == "java" && !out.stderr.is_empty()) {
                    let stdout_str = String::from_utf8_lossy(&out.stdout);
                    let stderr_str = String::from_utf8_lossy(&out.stderr);
                    let raw_ver = if !stdout_str.trim().is_empty() { stdout_str } else { stderr_str };

                    if let Some(ver) = parse_version_output(meta.id, &raw_ver) {
                        if is_valid_semver(&ver) {
                            if !installed_versions.contains(&ver) {
                                installed_versions.push(ver.clone());
                            }
                            if active_version.is_none() {
                                active_version = Some(ver.clone());
                                global_version = Some(ver);
                            }
                            break;
                        }
                    }
                }
            }
        }

        // 2. Scan Mise installs directories directly
        for base_dir in get_mise_install_dirs() {
            for alias in meta.mise_aliases {
                let install_dir = base_dir.join(alias);
                if install_dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(install_dir) {
                        for entry in entries.flatten() {
                            if entry.path().is_dir() {
                                if let Some(folder_name) = entry.file_name().to_str() {
                                    if let Some(clean_v) = parse_version_output(meta.id, folder_name) {
                                        if is_valid_semver(&clean_v) && !installed_versions.contains(&clean_v) {
                                            installed_versions.push(clean_v);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Query Mise CLI if available
        if let Some(ref bin) = mise_bin {
            let bin_str = bin.to_string_lossy();
            for alias in meta.mise_aliases {
                if let Ok(output) = env_helper::create_silent_command(&bin_str).args(["ls", "--json", alias]).output() {
                    if output.status.success() {
                        if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                            if let Some(arr) = json_val.as_array() {
                                for item in arr {
                                    if let Some(ver_str) = item.get("version").and_then(|v| v.as_str()) {
                                        if let Some(clean_v) = parse_version_output(meta.id, ver_str) {
                                            if is_valid_semver(&clean_v) {
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
                }
            }

            // Query Mise current active version
            if active_version.is_none() {
                if let Ok(cur_out) = env_helper::create_silent_command(&bin_str).args(["current", meta.id]).output() {
                    if cur_out.status.success() {
                        let cur_str = String::from_utf8_lossy(&cur_out.stdout).trim().to_string();
                        if !cur_str.is_empty() && !cur_str.starts_with("No version") {
                            if let Some(clean_v) = parse_version_output(meta.id, &cur_str) {
                                if is_valid_semver(&clean_v) {
                                    if !installed_versions.contains(&clean_v) {
                                        installed_versions.push(clean_v.clone());
                                    }
                                    active_version = Some(clean_v.clone());
                                    global_version = Some(clean_v);
                                }
                            }
                        }
                    }
                }
            }

            // Remote versions from mise ls-remote
            if let Ok(output) = env_helper::create_silent_command(&bin_str).args(["ls-remote", meta.id]).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let raw_list: Vec<String> = stdout
                        .lines()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .collect();
                    let curated: Vec<String> = filter_latest_minor_versions(raw_list)
                        .into_iter()
                        .take(25)
                        .collect();
                    available_versions = curated;
                }
            }
        }

        // 4. Read global Mise configuration files
        for cp in get_mise_config_paths() {
            if cp.exists() {
                if let Ok(content) = std::fs::read_to_string(&cp) {
                    for line in content.lines() {
                        let trim = line.trim();
                        for alias in meta.mise_aliases {
                            if trim.starts_with(alias) && (trim.contains('=') || trim.contains(' ')) {
                                let val = trim.split(|c: char| c == '=' || c.is_whitespace()).last().unwrap_or("").trim().trim_matches('"').trim_matches('\'');
                                if let Some(clean_v) = parse_version_output(meta.id, val) {
                                    if is_valid_semver(&clean_v) {
                                        if !installed_versions.contains(&clean_v) {
                                            installed_versions.push(clean_v.clone());
                                        }
                                        if global_version.is_none() {
                                            global_version = Some(clean_v.clone());
                                        }
                                        if active_version.is_none() {
                                            active_version = Some(clean_v);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 5. Clean, validate and deduplicate installed versions
        let installed_versions = clean_and_deduplicate_installed_versions(installed_versions);

        // 6. Ensure active_version and global_version match an actual installed version
        if let Some(ref act) = active_version {
            let act_clean = act.trim().trim_start_matches('v');
            if !installed_versions.contains(&act_clean.to_string()) {
                // Find matching installed version that starts with act_clean or fallback
                if let Some(matched) = installed_versions.iter().find(|iv| iv.starts_with(act_clean)) {
                    active_version = Some(matched.clone());
                    global_version = Some(matched.clone());
                } else if !installed_versions.is_empty() {
                    active_version = Some(installed_versions[0].clone());
                    global_version = Some(installed_versions[0].clone());
                } else {
                    active_version = None;
                    global_version = None;
                }
            }
        } else if !installed_versions.is_empty() {
            active_version = Some(installed_versions[0].clone());
            global_version = Some(installed_versions[0].clone());
        }

        // Fallback default curated versions if Mise ls-remote returned empty
        if available_versions.is_empty() {
            available_versions = get_fallback_curated_versions(meta.id);
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

fn get_fallback_curated_versions(tool_id: &str) -> Vec<String> {
    match tool_id {
        "node" => vec![
            "25.9.0".into(), "25.8.2".into(), "25.7.0".into(), "25.6.1".into(),
            "24.1.0".into(), "24.0.2".into(), "23.9.0".into(), "22.14.0".into(),
            "20.18.3".into(), "18.20.7".into(),
        ],
        "go" => vec![
            "1.24.0".into(), "1.23.6".into(), "1.22.12".into(), "1.21.13".into(),
            "1.20.14".into(), "1.19.13".into(),
        ],
        "python" => vec![
            "3.13.2".into(), "3.12.9".into(), "3.11.11".into(), "3.10.16".into(),
            "3.9.21".into(), "3.8.20".into(),
        ],
        "rust" => vec![
            "1.85.0".into(), "1.84.1".into(), "1.83.0".into(), "1.82.0".into(),
            "1.81.0".into(), "1.80.1".into(),
        ],
        "java" => vec![
            "21.0.6".into(), "17.0.14".into(), "11.0.26".into(), "8.0.442".into(),
        ],
        "ruby" => vec![
            "3.4.2".into(), "3.3.7".into(), "3.2.7".into(), "3.1.6".into(),
        ],
        "bun" => vec![
            "1.2.4".into(), "1.1.43".into(), "1.0.36".into(),
        ],
        "deno" => vec![
            "2.2.3".into(), "2.1.10".into(), "2.0.6".into(), "1.46.3".into(),
        ],
        "php" => vec![
            "8.4.4".into(), "8.3.17".into(), "8.2.27".into(), "8.1.31".into(),
        ],
        "zig" => vec![
            "0.14.0".into(), "0.13.0".into(), "0.12.1".into(), "0.11.0".into(),
        ],
        "dotnet" => vec![
            "9.0.2".into(), "8.0.13".into(), "7.0.20".into(), "6.0.36".into(),
        ],
        "dart" => vec![
            "3.7.0".into(), "3.6.2".into(), "3.5.4".into(), "3.4.4".into(),
        ],
        "flutter" => vec![
            "3.29.0".into(), "3.27.4".into(), "3.24.5".into(), "3.22.3".into(),
        ],
        "kotlin" => vec![
            "2.1.10".into(), "2.0.21".into(), "1.9.25".into(), "1.8.22".into(),
        ],
        "elixir" => vec![
            "1.18.2".into(), "1.17.3".into(), "1.16.3".into(), "1.15.7".into(),
        ],
        "erlang" => vec![
            "27.2.1".into(), "26.2.5".into(), "25.3.2".into(),
        ],
        "lua" => vec![
            "5.4.7".into(), "5.3.6".into(), "5.2.4".into(), "5.1.5".into(),
        ],
        "terraform" => vec![
            "1.10.5".into(), "1.9.8".into(), "1.8.5".into(), "1.7.5".into(),
        ],
        _ => vec!["latest".into()],
    }
}

/// Filters version list to only keep the latest patch version for each major.minor series
/// e.g. ["25.8.0", "25.8.1", "25.8.2", "25.9.0"] -> ["25.9.0", "25.8.2"]
fn filter_latest_minor_versions(versions: Vec<String>) -> Vec<String> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<(u64, u64), (u64, bool, String)> = BTreeMap::new();
    let mut non_semver: Vec<String> = Vec::new();

    for v in versions {
        let trimmed = v.trim().trim_start_matches('v');
        let parts: Vec<&str> = trimmed.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse::<u64>();
            let minor = parts[1].parse::<u64>();
            let (patch, is_prerelease) = if parts.len() >= 3 {
                let p_raw = parts[2];
                let is_pre = p_raw.contains('-') || p_raw.contains("rc") || p_raw.contains("beta") || p_raw.contains("alpha");
                let patch_num: u64 = p_raw.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
                (patch_num, is_pre)
            } else {
                (0, false)
            };

            if let (Ok(maj), Ok(min)) = (major, minor) {
                if let Some((existing_patch, existing_pre, _)) = groups.get(&(maj, min)) {
                    if *existing_pre && !is_prerelease {
                        groups.insert((maj, min), (patch, is_prerelease, v.clone()));
                    } else if !is_prerelease && !existing_pre {
                        if patch >= *existing_patch {
                            groups.insert((maj, min), (patch, is_prerelease, v.clone()));
                        }
                    } else if is_prerelease && *existing_pre {
                        if patch >= *existing_patch {
                            groups.insert((maj, min), (patch, is_prerelease, v.clone()));
                        }
                    }
                } else {
                    groups.insert((maj, min), (patch, is_prerelease, v.clone()));
                }
                continue;
            }
        }
        if !non_semver.contains(&v) {
            non_semver.push(v);
        }
    }

    let mut result: Vec<String> = groups.into_iter().rev().map(|(_, (_, _, ver))| ver).collect();
    for item in non_semver {
        if !result.contains(&item) {
            result.push(item);
        }
    }
    result
}

#[tauri::command]
pub async fn get_remote_versions(tool_id: String) -> Result<Vec<String>, String> {
    if let Some(bin) = env_helper::find_mise_binary() {
        if let Ok(output) = std::process::Command::new(&bin).args(["ls-remote", &tool_id]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let raw_list: Vec<String> = stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                let curated: Vec<String> = filter_latest_minor_versions(raw_list)
                    .into_iter()
                    .take(40)
                    .collect();
                return Ok(curated);
            }
        }
    }
    Ok(get_fallback_curated_versions(&tool_id))
}

#[tauri::command]
pub async fn set_global_version(tool_id: String, version: String) -> Result<bool, String> {
    if let Some(bin) = env_helper::find_mise_binary() {
        let target = format!("{}@{}", tool_id, version);
        let status = env_helper::create_silent_tokio_command(&bin.to_string_lossy())
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
    let clean_ver = version.trim().trim_start_matches('v').to_string();

    // 1. Run mise uninstall <tool>@<version>
    if let Some(bin) = env_helper::find_mise_binary() {
        let targets = [
            format!("{}@{}", tool_id, clean_ver),
            format!("{}@{}", tool_id, version),
        ];
        for target in targets {
            let _ = env_helper::create_silent_tokio_command(&bin.to_string_lossy())
                .args(["uninstall", &target])
                .status()
                .await;
        }
    }

    // 2. Direct clean up directory in all candidate installs locations
    for base_dir in get_mise_install_dirs() {
        let install_candidates = [
            base_dir.join(&tool_id).join(&clean_ver),
            base_dir.join(&tool_id).join(&version),
            base_dir.join(&tool_id).join(format!("v{}", clean_ver)),
        ];
        for d in install_candidates {
            if d.exists() {
                let _ = std::fs::remove_dir_all(&d);
            }
        }
    }

    // 3. Clean up from all configuration files
    for config_file in get_mise_config_paths() {
        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                let lines: Vec<String> = content.lines()
                    .filter(|l| {
                        let trim = l.trim();
                        !(trim.starts_with(&tool_id) && (trim.contains(&clean_ver) || trim.contains(&version)))
                    })
                    .map(|l| l.to_string())
                    .collect();
                let _ = std::fs::write(&config_file, lines.join("\n") + "\n");
            }
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
    env_helper::fix_system_path();

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
    let _ = app.emit("install-log", format!("> {} install {}", mise_bin.display(), target));
    let _ = app.emit("install-progress", 10);

    let mut cmd = env_helper::create_silent_tokio_command(&mise_bin.to_string_lossy());
    cmd.args(["install", &target, "--verbose"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Ok(curr_path) = env::var("PATH") {
        cmd.env("PATH", curr_path);
    }

    let mut child = cmd.spawn()
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
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app_clone2.emit("install-log", line.clone());
            }
        });
    }

    let status = child.wait().await.map_err(|e| format!("等待安装完成失败: {}", e))?;
    if status.success() {
        let _ = app.emit("install-log", format!("✓ {} 安装成功！已完成环境注入与软链接配置", target));
        let _ = app.emit("install-progress", 100);
        Ok(true)
    } else {
        let _ = app.emit("install-log", format!("❌ 安装失败，退出码: {:?}", status.code()));
        if cfg!(target_os = "windows") {
            let _ = app.emit("install-log", "💡 提示: Windows 环境下部分语言（如 Python/Node）需预先安装 7-Zip、Git 或 Visual C++ 运行库以支持解压。可在【系统工具箱】一键安装 Git 与 Scoop。".to_string());
        }
        Ok(false)
    }
}

#[tauri::command]
pub async fn bootstrap_mise_cli(app: AppHandle) -> Result<bool, String> {
    let _ = app.emit("install-log", "> 正在初始化 Mise CLI 跨平台运行时引擎安装程序...".to_string());
    let _ = app.emit("install-progress", 10);

    #[cfg(target_os = "windows")]
    {
        // Method 1: Direct standalone binary download from Cloudflare CDN into %LOCALAPPDATA%\mise\bin\mise.exe
        let _ = app.emit("install-log", "方案 1: 正在从官方 CDN 极速拉取 Windows x64 独立可执行引擎...".to_string());
        let _ = app.emit("install-progress", 25);

        let direct_download_ps = r#"
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13;
            $binDir = "$env:LOCALAPPDATA\mise\bin";
            if (-not (Test-Path $binDir)) {
                New-Item -ItemType Directory -Force -Path $binDir | Out-Null;
            }
            $exePath = "$binDir\mise.exe";
            try {
                Invoke-WebRequest -Uri "https://mise.jdx.dev/mise-latest-windows-x64.exe" -OutFile $exePath -TimeoutSec 30;
                if (Test-Path $exePath) {
                    & $exePath --version
                    exit 0
                }
            } catch {
                exit 1
            }
            exit 1
        "#;

        let status1 = env_helper::create_silent_tokio_command("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", direct_download_ps])
            .status()
            .await;

        if let Ok(s) = status1 {
            if s.success() {
                let _ = app.emit("install-log", "✓ 官方独立二进制引擎下载成功！正在同步环境变量与 Shims...".to_string());
                let _ = app.emit("install-progress", 85);
                env_helper::fix_system_path();
                let _ = app.emit("install-log", "✓ Mise CLI 引擎已成功部署就绪！".to_string());
                let _ = app.emit("install-progress", 100);
                return Ok(true);
            }
        }

        // Method 2: Official install.ps1 script
        let _ = app.emit("install-log", "方案 2: 尝试通过官方 PowerShell 安装脚本执行构建...".to_string());
        let _ = app.emit("install-progress", 45);

        let script_ps = r#"
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13;
            Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force;
            irm https://mise.jdx.dev/install.ps1 | iex
        "#;

        let status2 = env_helper::create_silent_tokio_command("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script_ps])
            .status()
            .await;

        if let Ok(s) = status2 {
            if s.success() {
                let _ = app.emit("install-log", "✓ PowerShell 官方自举安装完成！".to_string());
                env_helper::fix_system_path();
                let _ = app.emit("install-progress", 100);
                return Ok(true);
            }
        }

        // Method 3: Scoop install mise
        let has_scoop = env_helper::create_silent_command("scoop").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
        if has_scoop {
            let _ = app.emit("install-log", "方案 3: 尝试通过 Scoop 包管理器安装 mise...".to_string());
            let status3 = env_helper::create_silent_tokio_command("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "scoop install mise"])
                .status()
                .await;

            if let Ok(s) = status3 {
                if s.success() {
                    let _ = app.emit("install-log", "✓ 通过 Scoop 成功安装 mise！".to_string());
                    env_helper::fix_system_path();
                    let _ = app.emit("install-progress", 100);
                    return Ok(true);
                }
            }
        }

        // Method 4: WinGet install
        let has_winget = env_helper::create_silent_command("winget").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
        if has_winget {
            let _ = app.emit("install-log", "方案 4: 尝试通过 WinGet 安装 jdx.mise...".to_string());
            let status4 = env_helper::create_silent_tokio_command("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "winget install jdx.mise --accept-source-agreements --accept-package-agreements"])
                .status()
                .await;

            if let Ok(s) = status4 {
                if s.success() {
                    let _ = app.emit("install-log", "✓ 通过 WinGet 成功安装 mise！".to_string());
                    env_helper::fix_system_path();
                    let _ = app.emit("install-progress", 100);
                    return Ok(true);
                }
            }
        }

        let _ = app.emit("install-log", "❌ 所有自动安装途径均失败，请检查网络连接或是否开启了代理拦截。".to_string());
        return Err("Windows 下自动安装 Mise 失败，请检查网络连接或手动在终端执行: irm https://mise.jdx.dev/install.ps1 | iex".to_string());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app.emit("install-log", "> 执行官方自举脚本: curl https://mise.run | sh".to_string());
        let mut child = env_helper::create_silent_tokio_command("sh")
            .args(["-c", "curl https://mise.run | sh"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("执行自举脚本失败: {}", e))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let app_c1 = app.clone();
        if let Some(stdout) = stdout {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = app_c1.emit("install-log", line);
                }
            });
        }

        let app_c2 = app.clone();
        if let Some(stderr) = stderr {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = app_c2.emit("install-log", line);
                }
            });
        }

        let status = child.wait().await.map_err(|e| format!("等待自举脚本完成失败: {}", e))?;
        if status.success() {
            env_helper::fix_system_path();
            let _ = app.emit("install-log", "✓ Mise CLI 引擎已安装就绪！".to_string());
            let _ = app.emit("install-progress", 100);
            return Ok(true);
        } else {
            // Try brew install mise as fallback on macOS
            #[cfg(target_os = "macos")]
            {
                let _ = app.emit("install-log", "尝试通过 Homebrew 安装 mise...".to_string());
                let brew_status = env_helper::create_silent_tokio_command("brew")
                    .args(["install", "mise"])
                    .status()
                    .await;
                if let Ok(bs) = brew_status {
                    if bs.success() {
                        env_helper::fix_system_path();
                        let _ = app.emit("install-log", "✓ 通过 Homebrew 成功安装 mise！".to_string());
                        let _ = app.emit("install-progress", 100);
                        return Ok(true);
                    }
                }
            }

            let _ = app.emit("install-log", format!("❌ 自举脚本返回错误退出码: {:?}", status.code()));
            return Err(format!("自举脚本返回错误退出码: {:?}", status.code()));
        }
    }
}

#[tauri::command]
pub async fn open_terminal_for_runtime(tool_id: String, version: String) -> Result<String, String> {
    env_helper::fix_system_path();

    let (exec, args): (&str, &[&str]) = match tool_id.as_str() {
        "node" => ("node", &["--version"]),
        "python" => ("python3", &["--version"]),
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
        "erlang" => ("erl", &["-eval", "erlang:display(erlang:system_info(otp_release)), halt().", "-noshell"]),
        "lua" => ("lua", &["-v"]),
        "terraform" => ("terraform", &["version"]),
        _ => (&tool_id, &["--version"]),
    };

    let probe_out = env_helper::create_silent_command(exec).args(args).output();
    let version_output = if let Ok(out) = probe_out {
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if !stdout.is_empty() { stdout } else { stderr }
    } else {
        format!("{} v{}", tool_id, version)
    };

    #[cfg(target_os = "macos")]
    {
        let script = format!("echo '=== EnvHub 环境检测: {} ==='; {} {}; echo ''; exec $SHELL", tool_id, exec, args.join(" "));
        let _ = std::process::Command::new("osascript")
            .args(["-e", &format!("tell application \"Terminal\" to do script \"{}\"", script)])
            .status();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "powershell", "-NoExit", "-Command", &format!("Write-Host '=== EnvHub 环境检测: {} ===' -ForegroundColor Cyan; {} {}; Write-Host ''", tool_id, exec, args.join(" "))])
            .status();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("x-terminal-emulator").status();
    }

    Ok(version_output)
}
