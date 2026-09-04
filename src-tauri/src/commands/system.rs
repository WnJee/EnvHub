use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::env_helper;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemStatus {
    pub os: String,
    #[serde(rename = "osVersion")]
    pub os_version: String,
    pub arch: String,
    #[serde(rename = "defaultShell")]
    pub default_shell: String,
    #[serde(rename = "miseInstalled")]
    pub mise_installed: bool,
    #[serde(rename = "miseVersion")]
    pub mise_version: Option<String>,
    #[serde(rename = "misePath")]
    pub mise_path: Option<String>,
    #[serde(rename = "packageManager")]
    pub package_manager: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemTool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    #[serde(rename = "isInstalled")]
    pub is_installed: bool,
    #[serde(rename = "installedVersion")]
    pub installed_version: Option<String>,
    #[serde(rename = "installCommand")]
    pub install_command: String,
    pub icon: String,
    pub homepage: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvHealthCheck {
    pub id: String,
    pub title: String,
    pub status: String, // "ok" | "warning" | "error"
    pub message: String,
    pub shell: String,
    #[serde(rename = "configFile")]
    pub config_file: String,
    #[serde(rename = "canAutoFix")]
    pub can_auto_fix: bool,
}

#[tauri::command]
pub async fn get_system_status() -> Result<SystemStatus, String> {
    let os_name = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };

    let arch = env::consts::ARCH.to_string();
    let default_shell = env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(windows) { "powershell.exe".to_string() } else { "/bin/zsh".to_string() }
    });

    let mut os_version = format!("{} ({})", os_name, arch);
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = env_helper::create_silent_command("sw_vers").arg("-productVersion").output() {
            if out.status.success() {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                os_version = format!("macOS {}", v);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        os_version = "Windows (x64/ARM64)".to_string();
    }

    let mise_bin = env_helper::find_mise_binary();
    let mise_installed = mise_bin.is_some();
    let mise_path = mise_bin.as_ref().map(|p| p.to_string_lossy().to_string());

    let mut mise_version = None;
    if let Some(ref bin) = mise_bin {
        if let Ok(out) = env_helper::create_silent_command(&bin.to_string_lossy()).arg("--version").output() {
            if out.status.success() {
                mise_version = Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }
    }

    let pkg_manager = if cfg!(target_os = "macos") {
        if env_helper::create_silent_command("brew").arg("--version").output().is_ok() {
            "brew"
        } else {
            "none"
        }
    } else if cfg!(target_os = "windows") {
        if env_helper::create_silent_command("winget").arg("--version").output().is_ok() {
            "winget"
        } else if env_helper::create_silent_command("scoop").arg("--version").output().is_ok() {
            "scoop"
        } else {
            "winget"
        }
    } else {
        "apt"
    };

    Ok(SystemStatus {
        os: os_name.to_string(),
        os_version,
        arch,
        default_shell,
        mise_installed,
        mise_version,
        mise_path,
        package_manager: pkg_manager.to_string(),
    })
}

#[tauri::command]
pub async fn get_system_tools() -> Result<Vec<SystemTool>, String> {
    let mut tools = Vec::new();

    #[cfg(not(target_os = "windows"))]
    tools.push(SystemTool {
        id: "brew".to_string(),
        name: "Homebrew".to_string(),
        description: "macOS 与 Linux 必备的终端软件包与命令行工具包管理器".to_string(),
        category: "Package Manager".to_string(),
        is_installed: false,
        installed_version: None,
        install_command: "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"".to_string(),
        icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/homebrew/homebrew-original.svg".to_string(),
        homepage: "https://brew.sh".to_string(),
    });

    #[cfg(target_os = "windows")]
    tools.push(SystemTool {
        id: "scoop".to_string(),
        name: "Scoop".to_string(),
        description: "Windows 极简、不污染系统注册表的命令行软件与开发环境包管理器".to_string(),
        category: "Package Manager".to_string(),
        is_installed: false,
        installed_version: None,
        install_command: "powershell -ExecutionPolicy RemoteSigned -Command \"irm get.scoop.sh | iex\"".to_string(),
        icon: "https://raw.githubusercontent.com/ScoopInstaller/Scoop/master/packaging/scoop.ico".to_string(),
        homepage: "https://scoop.sh".to_string(),
    });

    tools.extend(vec![
        SystemTool {
            id: "git".to_string(),
            name: "Git".to_string(),
            description: "全球最主流的代码版本管理与协作工具".to_string(),
            category: "VCS".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install git".to_string() } else { "winget install Git.Git".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/git/git-original.svg".to_string(),
            homepage: "https://git-scm.com".to_string(),
        },
        SystemTool {
            id: "docker".to_string(),
            name: "Docker CLI".to_string(),
            description: "轻量级应用容器化引擎与开发环境虚拟化基建".to_string(),
            category: "Container".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install --cask docker".to_string() } else { "winget install Docker.DockerDesktop".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/docker/docker-original.svg".to_string(),
            homepage: "https://www.docker.com".to_string(),
        },
        SystemTool {
            id: "docker-compose".to_string(),
            name: "Docker Compose".to_string(),
            description: "定义和运行多容器 Docker 应用程序的统一编排工具".to_string(),
            category: "Container".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install docker-compose".to_string() } else { "winget install Docker.DockerCompose".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/docker/docker-original.svg".to_string(),
            homepage: "https://docs.docker.com/compose/".to_string(),
        },
        SystemTool {
            id: "nginx".to_string(),
            name: "Nginx".to_string(),
            description: "高性能 HTTP 和反向代理 Web 服务器，负载均衡利器".to_string(),
            category: "Server".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install nginx".to_string() } else { "winget install Nginx.Nginx".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/nginx/nginx-original.svg".to_string(),
            homepage: "https://nginx.org".to_string(),
        },
        SystemTool {
            id: "redis".to_string(),
            name: "Redis".to_string(),
            description: "超高速开源内存键值数据库与高性能消息缓存中间件".to_string(),
            category: "Database".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install redis".to_string() } else { "winget install Redis.Redis".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/redis/redis-original.svg".to_string(),
            homepage: "https://redis.io".to_string(),
        },
        SystemTool {
            id: "mysql".to_string(),
            name: "MySQL".to_string(),
            description: "全球最广泛使用的开源关系型数据库管理系统".to_string(),
            category: "Database".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install mysql".to_string() } else { "winget install Oracle.MySQL".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/mysql/mysql-original.svg".to_string(),
            homepage: "https://www.mysql.com".to_string(),
        },
        SystemTool {
            id: "postgresql".to_string(),
            name: "PostgreSQL".to_string(),
            description: "强大、高度可扩展且功能完备的企业级开源对象关系型数据库".to_string(),
            category: "Database".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install postgresql@16".to_string() } else { "winget install PostgreSQL.PostgreSQL".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/postgresql/postgresql-original.svg".to_string(),
            homepage: "https://www.postgresql.org".to_string(),
        },
        SystemTool {
            id: "mongodb".to_string(),
            name: "MongoDB".to_string(),
            description: "面向文档的现代化 NoSQL 高性能高可用数据库".to_string(),
            category: "Database".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install mongodb-community".to_string() } else { "winget install MongoDB.Server".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/mongodb/mongodb-original.svg".to_string(),
            homepage: "https://www.mongodb.com".to_string(),
        },
        SystemTool {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            description: "本地极速运行开源大语言模型 (DeepSeek, Llama 3, Qwen) 的轻量级引擎".to_string(),
            category: "AI & ML".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install --cask ollama".to_string() } else { "winget install Ollama.Ollama".to_string() },
            icon: "https://ollama.com/public/ollama.png".to_string(),
            homepage: "https://ollama.com".to_string(),
        },
        SystemTool {
            id: "gh".to_string(),
            name: "GitHub CLI (gh)".to_string(),
            description: "GitHub 官方命令行工具，直接在终端管理 PR、Issue 与 Actions".to_string(),
            category: "CLI Utility".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install gh".to_string() } else { "winget install GitHub.cli".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/github/github-original.svg".to_string(),
            homepage: "https://cli.github.com".to_string(),
        },
        SystemTool {
            id: "ripgrep".to_string(),
            name: "Ripgrep (rg)".to_string(),
            description: "基于 Rust 实现的超高速递归代码全文搜索工具".to_string(),
            category: "Search".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install ripgrep".to_string() } else { "winget install BurntSushi.ripgrep.MSVC".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-original.svg".to_string(),
            homepage: "https://github.com/BurntSushi/ripgrep".to_string(),
        },
        SystemTool {
            id: "fd".to_string(),
            name: "FD (fd-find)".to_string(),
            description: "现代化极速文件搜索工具，人性化语法替代传统 find".to_string(),
            category: "Search".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install fd".to_string() } else { "winget install sharkdp.fd".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-original.svg".to_string(),
            homepage: "https://github.com/sharkdp/fd".to_string(),
        },
        SystemTool {
            id: "lazygit".to_string(),
            name: "LazyGit".to_string(),
            description: "极速全键盘操作的终端可视化 Git 分支与提交管理面板".to_string(),
            category: "VCS".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install lazygit".to_string() } else { "winget install JesseDuffield.lazygit".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/git/git-original.svg".to_string(),
            homepage: "https://github.com/jesseduffield/lazygit".to_string(),
        },
        SystemTool {
            id: "bat".to_string(),
            name: "Bat".to_string(),
            description: "支持代码语法高亮与 Git 变动集成的现代化 cat 替代神器".to_string(),
            category: "CLI Utility".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install bat".to_string() } else { "winget install sharkdp.bat".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-original.svg".to_string(),
            homepage: "https://github.com/sharkdp/bat".to_string(),
        },
        SystemTool {
            id: "fzf".to_string(),
            name: "FZF".to_string(),
            description: "通用终端交互式模糊搜索神器，快速检索命令历史与文件".to_string(),
            category: "CLI Utility".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install fzf".to_string() } else { "winget install junegunn.fzf".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/go/go-original.svg".to_string(),
            homepage: "https://github.com/junegunn/fzf".to_string(),
        },
        SystemTool {
            id: "zoxide".to_string(),
            name: "Zoxide (z)".to_string(),
            description: "深度记忆目录权重的智能终端秒速跳转工具（替代 cd）".to_string(),
            category: "CLI Utility".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install zoxide".to_string() } else { "winget install ajeetdsouza.zoxide".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-original.svg".to_string(),
            homepage: "https://github.com/ajeetdsouza/zoxide".to_string(),
        },
        SystemTool {
            id: "ffmpeg".to_string(),
            name: "FFmpeg".to_string(),
            description: "领先的跨平台音视频编解码、转码与多媒体流处理框架".to_string(),
            category: "Media".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install ffmpeg".to_string() } else { "winget install Gyan.FFmpeg".to_string() },
            icon: "https://ffmpeg.org/favicon.ico".to_string(),
            homepage: "https://ffmpeg.org".to_string(),
        },
        SystemTool {
            id: "cmake".to_string(),
            name: "CMake".to_string(),
            description: "跨平台自动化建构系统，C/C++ 与 Rust 复杂工程依赖".to_string(),
            category: "Build".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install cmake".to_string() } else { "winget install Kitware.CMake".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/cmake/cmake-original.svg".to_string(),
            homepage: "https://cmake.org".to_string(),
        },
        SystemTool {
            id: "neovim".to_string(),
            name: "Neovim (nvim)".to_string(),
            description: "可高度定制化、支持 Lua 插件生态的下一代现代化终端编辑器".to_string(),
            category: "Editor".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install neovim".to_string() } else { "winget install Neovim.Neovim".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/neovim/neovim-original.svg".to_string(),
            homepage: "https://neovim.io".to_string(),
        },
    ]);

    // Check installed status and real versions
    for tool in &mut tools {
        if let Some(v) = probe_tool_version(&tool.id) {
            tool.is_installed = true;
            tool.installed_version = Some(v);
        }
    }

    Ok(tools)
}

fn probe_tool_version(tool_id: &str) -> Option<String> {
    if tool_id == "brew" || tool_id == "homebrew" {
        for b in &["brew", "/opt/homebrew/bin/brew", "/usr/local/bin/brew", "/home/linuxbrew/.linuxbrew/bin/brew"] {
            if let Ok(out) = env_helper::create_silent_command(b).arg("--version").output() {
                if out.status.success() {
                    let raw = String::from_utf8_lossy(&out.stdout);
                    for word in raw.split(|c: char| c.is_whitespace() || c == '/' || c == '(' || c == ')' || c == ',') {
                        let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                        let semver: String = candidate.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                        let clean = semver.trim_end_matches('.');
                        if clean.contains('.') && clean.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                            return Some(clean.to_string());
                        }
                    }
                    let first_line = raw.lines().next().unwrap_or("installed");
                    return Some(first_line.trim().to_string());
                }
            }
        }
        return None;
    }

    if tool_id == "scoop" {
        if let Ok(out) = env_helper::create_silent_command("scoop").arg("--version").output() {
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout);
                for word in raw.split_whitespace() {
                    let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                    let semver: String = candidate.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                    let clean = semver.trim_end_matches('.');
                    if clean.contains('.') && clean.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        return Some(clean.to_string());
                    }
                }
                return Some(raw.lines().next().unwrap_or("installed").trim().to_string());
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(out) = env_helper::create_silent_command("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "scoop --version"])
                .output()
            {
                if out.status.success() {
                    let raw = String::from_utf8_lossy(&out.stdout);
                    for word in raw.split_whitespace() {
                        let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                        let semver: String = candidate.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                        let clean = semver.trim_end_matches('.');
                        if clean.contains('.') && clean.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                            return Some(clean.to_string());
                        }
                    }
                    return Some(raw.lines().next().unwrap_or("installed").trim().to_string());
                }
            }
        }
        return None;
    }

    let (cmd, args): (&str, &[&str]) = match tool_id {
        "docker-compose" => ("docker-compose", &["--version"]),
        "nginx" => ("nginx", &["-v"]),
        "redis" => ("redis-server", &["--version"]),
        "mysql" => ("mysql", &["--version"]),
        "postgresql" => ("psql", &["--version"]),
        "mongodb" => ("mongod", &["--version"]),
        "neovim" => ("nvim", &["--version"]),
        "ripgrep" => ("rg", &["--version"]),
        "fd" => if cfg!(target_os = "linux") { ("fdfind", &["--version"]) } else { ("fd", &["--version"]) },
        "ffmpeg" => ("ffmpeg", &["-version"]),
        _ => (tool_id, &["--version"]),
    };

    if let Ok(out) = env_helper::create_silent_command(cmd).args(args).output() {
        if out.status.success() || (!out.stderr.is_empty() && tool_id == "nginx") {
            let stdout_str = String::from_utf8_lossy(&out.stdout);
            let stderr_str = String::from_utf8_lossy(&out.stderr);
            let raw = if !stdout_str.trim().is_empty() { stdout_str } else { stderr_str };

            for word in raw.split(|c: char| c.is_whitespace() || c == '/' || c == '(' || c == ')' || c == ',') {
                let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                let semver: String = candidate.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                let clean = semver.trim_end_matches('.');
                if clean.contains('.') && clean.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    return Some(clean.to_string());
                }
            }
            let first_line = raw.lines().next().unwrap_or("installed");
            return Some(first_line.trim().to_string());
        }
    }

    // Fallbacks
    if tool_id == "docker-compose" {
        if let Ok(out) = env_helper::create_silent_command("docker").args(["compose", "version"]).output() {
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout);
                for word in raw.split_whitespace() {
                    let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                    let clean: String = candidate.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                    if clean.contains('.') {
                        return Some(clean.trim_end_matches('.').to_string());
                    }
                }
            }
        }
    } else if tool_id == "mongodb" {
        if let Ok(out) = env_helper::create_silent_command("mongosh").args(["--version"]).output() {
            if out.status.success() {
                return Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }
    } else if tool_id == "redis" {
        if let Ok(out) = env_helper::create_silent_command("redis-cli").args(["--version"]).output() {
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout);
                for word in raw.split_whitespace() {
                    let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                    let clean: String = candidate.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                    if clean.contains('.') {
                        return Some(clean.trim_end_matches('.').to_string());
                    }
                }
            }
        }
    }

    None
}

#[tauri::command]
pub async fn test_system_tool(tool_id: String) -> Result<String, String> {
    if tool_id == "brew" || tool_id == "homebrew" {
        for b in &["brew", "/opt/homebrew/bin/brew", "/usr/local/bin/brew", "/home/linuxbrew/.linuxbrew/bin/brew"] {
            if let Ok(output) = env_helper::create_silent_command(b).arg("--version").output() {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    let first_line = out_str.lines().next().unwrap_or("执行正常").trim();
                    return Ok(format!("测试成功: {}", first_line));
                }
            }
        }
        return Err("未检测到 Homebrew 可执行文件".to_string());
    }

    if tool_id == "scoop" {
        if let Ok(output) = env_helper::create_silent_command("scoop").arg("--version").output() {
            if output.status.success() {
                let out_str = String::from_utf8_lossy(&output.stdout);
                let first_line = out_str.lines().next().unwrap_or("执行正常").trim();
                return Ok(format!("测试成功: {}", first_line));
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = env_helper::create_silent_command("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "scoop --version"])
                .output()
            {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    let first_line = out_str.lines().next().unwrap_or("执行正常").trim();
                    return Ok(format!("测试成功: {}", first_line));
                }
            }
        }
        return Err("未检测到 Scoop 命令".to_string());
    }

    let binary_name = if tool_id == "neovim" { "nvim" } else if tool_id == "ripgrep" { "rg" } else { &tool_id };
    let output = env_helper::create_silent_command(binary_name)
        .arg("--version")
        .output()
        .map_err(|e| format!("无法执行 {}: {}", binary_name, e))?;

    if output.status.success() {
        let out_str = String::from_utf8_lossy(&output.stdout);
        let first_line = out_str.lines().next().unwrap_or("执行正常").trim();
        Ok(format!("测试成功: {}", first_line))
    } else {
        Err("执行返回非零状态码".to_string())
    }
}

#[tauri::command]
pub async fn install_system_tool(app: AppHandle, tool_id: String) -> Result<bool, String> {
    let _ = app.emit("install-log", format!("> 开始安装系统工具: {}", tool_id));
    let _ = app.emit("install-progress", 10);

    #[cfg(target_os = "windows")]
    {
        if tool_id == "scoop" {
            let script = r#"
                [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13;
                Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force;
                irm get.scoop.sh | iex
            "#;
            let _ = app.emit("install-log", "正在执行 Scoop 安装脚本...".to_string());
            let status = env_helper::create_silent_tokio_command("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
                .status()
                .await
                .map_err(|e| format!("安装 Scoop 失败: {}", e))?;

            if !status.success() {
                return Err(format!("安装 Scoop 返回退出码: {:?}", status.code()));
            }
            env_helper::fix_system_path();
            let _ = app.emit("install-progress", 100);
            return Ok(true);
        }

        let has_scoop = env_helper::create_silent_command("scoop").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
        let has_winget = env_helper::create_silent_command("winget").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);

        let (scoop_pkg, winget_pkg): (&str, &str) = match tool_id.as_str() {
            "git" => ("git", "Git.Git"),
            "docker" => ("docker", "Docker.DockerDesktop"),
            "docker-compose" => ("docker-compose", "Docker.DockerCompose"),
            "nginx" => ("nginx", "Nginx.Nginx"),
            "redis" => ("redis", "Redis.Redis"),
            "mysql" => ("mysql", "Oracle.MySQL"),
            "postgresql" => ("postgresql", "PostgreSQL.PostgreSQL"),
            "mongodb" => ("mongodb", "MongoDB.Server"),
            "ollama" => ("ollama", "Ollama.Ollama"),
            "gh" => ("gh", "GitHub.cli"),
            "ripgrep" => ("ripgrep", "BurntSushi.ripgrep.MSVC"),
            "fd" => ("fd", "sharkdp.fd"),
            "lazygit" => ("lazygit", "JesseDuffield.lazygit"),
            "bat" => ("bat", "sharkdp.bat"),
            "fzf" => ("fzf", "junegunn.fzf"),
            "zoxide" => ("zoxide", "ajeetdsouza.zoxide"),
            "ffmpeg" => ("ffmpeg", "Gyan.FFmpeg"),
            "cmake" => ("cmake", "Kitware.CMake"),
            "neovim" => ("neovim", "Neovim.Neovim"),
            _ => (tool_id.as_str(), tool_id.as_str()),
        };

        let cmd_string = if has_scoop {
            let _ = app.emit("install-log", format!("使用 Scoop 极速安装 {}", scoop_pkg));
            format!("scoop install {}", scoop_pkg)
        } else if has_winget {
            let _ = app.emit("install-log", format!("使用 WinGet 安装 {}", winget_pkg));
            format!("winget install --accept-source-agreements --accept-package-agreements {}", winget_pkg)
        } else {
            return Err("未在 Windows 系统中检测到 Scoop 或 WinGet，请先在工具箱首位安装 Scoop".to_string());
        };

        let mut child = env_helper::create_silent_tokio_command("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &cmd_string])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动安装进程失败: {}", e))?;

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

        let status = child.wait().await.map_err(|e| format!("等待安装完成失败: {}", e))?;
        if !status.success() {
            let _ = app.emit("install-log", format!("❌ 安装失败，退出码: {:?}", status.code()));
            return Err(format!("安装执行失败，退出码: {:?}", status.code()));
        }
        let _ = app.emit("install-log", format!("✓ {} 安装成功！", tool_id));
        let _ = app.emit("install-progress", 100);
        return Ok(true);
    }

    #[cfg(target_os = "macos")]
    {
        let cmd = if tool_id == "brew" || tool_id == "homebrew" {
            "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"".to_string()
        } else if tool_id == "docker" {
            "brew install --cask docker".to_string()
        } else {
            format!("brew install {}", tool_id)
        };
        let _ = app.emit("install-log", format!("> 执行: {}", cmd));

        let mut child = env_helper::create_silent_tokio_command("sh")
            .args(["-c", &cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("调用 Homebrew 失败: {}", e))?;

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

        let status = child.wait().await.map_err(|e| format!("等待 Homebrew 完成失败: {}", e))?;
        if !status.success() {
            let _ = app.emit("install-log", format!("❌ Homebrew 安装失败，退出码: {:?}", status.code()));
            return Err(format!("Homebrew 执行失败，退出码: {:?}", status.code()));
        }
        let _ = app.emit("install-log", format!("✓ {} 安装成功！", tool_id));
        let _ = app.emit("install-progress", 100);
        return Ok(true);
    }

    #[cfg(target_os = "linux")]
    {
        let cmd = if tool_id == "brew" || tool_id == "homebrew" {
            "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"".to_string()
        } else {
            format!("sudo apt install -y {}", tool_id)
        };
        let _ = app.emit("install-log", format!("> 执行: {}", cmd));

        let mut child = env_helper::create_silent_tokio_command("sh")
            .args(["-c", &cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("调用安装工具失败: {}", e))?;

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

        let status = child.wait().await.map_err(|e| format!("等待完成失败: {}", e))?;
        if !status.success() {
            let _ = app.emit("install-log", format!("❌ 安装失败，退出码: {:?}", status.code()));
            return Err(format!("安装执行失败，退出码: {:?}", status.code()));
        }
        let _ = app.emit("install-log", format!("✓ {} 安装成功！", tool_id));
        let _ = app.emit("install-progress", 100);
        return Ok(true);
    }
}

#[tauri::command]
pub async fn get_health_checks() -> Result<Vec<EnvHealthCheck>, String> {
    // 1. Refresh latest PATH in the current process
    env_helper::fix_system_path();

    let mut checks = Vec::new();
    let is_windows = cfg!(target_os = "windows");

    let shell = if is_windows {
        "PowerShell".to_string()
    } else {
        env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
    };

    let mut rc_file = if is_windows {
        "Microsoft.PowerShell_profile.ps1".to_string()
    } else if shell.ends_with("bash") {
        "~/.bashrc".to_string()
    } else {
        "~/.zshrc".to_string()
    };

    let mut has_activation = false;

    if is_windows {
        // Windows profile locations (PowerShell 5, PowerShell 7, OneDrive variants)
        if let Some(home) = dirs::home_dir() {
            let win_profiles = [
                home.join("Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1"),
                home.join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1"),
                home.join("OneDrive/Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1"),
                home.join("OneDrive/Documents/PowerShell/Microsoft.PowerShell_profile.ps1"),
            ];

            for p in win_profiles {
                if p.exists() {
                    if let Ok(content) = fs::read_to_string(&p) {
                        if content.contains("mise activate") || content.contains("rtx activate") {
                            has_activation = true;
                            rc_file = p.file_name().and_then(|n| n.to_str()).unwrap_or("Microsoft.PowerShell_profile.ps1").to_string();
                            break;
                        }
                    }
                }
            }
        }
    } else {
        // macOS / Linux
        if let Some(home) = dirs::home_dir() {
            let zshrc = home.join(".zshrc");
            let zprofile = home.join(".zprofile");
            let bashrc = home.join(".bashrc");
            let bash_profile = home.join(".bash_profile");

            for f in [&zshrc, &zprofile, &bashrc, &bash_profile] {
                if f.exists() {
                    if let Ok(content) = fs::read_to_string(f) {
                        if content.contains("mise activate") || content.contains("rtx activate") {
                            has_activation = true;
                            rc_file = f.file_name().and_then(|n| n.to_str()).unwrap_or("profile").to_string();
                            break;
                        }
                    }
                }
            }
        }
    }

    checks.push(EnvHealthCheck {
        id: "mise-activated".to_string(),
        title: "Shell 环境变量与 Shims 激活检测".to_string(),
        status: if has_activation { "ok".to_string() } else { "warning".to_string() },
        message: if has_activation {
            format!("已在 {} 中检测到 mise activate，命令行环境同步正常", rc_file)
        } else if is_windows {
            "未在 PowerShell Profile 中检测到 mise activate，命令行可能无法自动同步运行时版本".to_string()
        } else {
            format!("未在 {} 中配置 mise activate，命令行可能无法自动切换版本", rc_file)
        },
        shell: shell.clone(),
        config_file: rc_file.clone(),
        can_auto_fix: !has_activation,
    });

    // 2. PATH priority check
    let path = env::var("PATH").unwrap_or_default();
    let has_shims_in_path = path.contains(".local/share/mise/shims")
        || path.contains("mise/shims")
        || path.contains("mise\\shims")
        || path.contains("AppData\\Local\\mise\\shims")
        || path.contains("scoop\\shims");

    let has_shims_configured = has_shims_in_path || has_activation;

    checks.push(EnvHealthCheck {
        id: "path-priority".to_string(),
        title: "PATH 优先级与 Shims 注入检查".to_string(),
        status: if has_shims_configured { "ok".to_string() } else { "warning".to_string() },
        message: if has_shims_configured {
            "mise shims 路径已在系统 PATH 与终端配置中生效，运行版本劫持正常".to_string()
        } else {
            "未在系统 PATH 中检测到 mise shims，点击一键自动修复注入环境变量".to_string()
        },
        shell: shell.clone(),
        config_file: if is_windows { "System PATH Registry".to_string() } else { "PATH Environment".to_string() },
        can_auto_fix: !has_shims_configured,
    });

    // 3. Package Manager check
    let (pkg_mgr_ok, pkg_mgr_name, pkg_mgr_msg) = if is_windows {
        let has_scoop = env_helper::create_silent_command("scoop").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
        let has_winget = env_helper::create_silent_command("winget").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
        if has_scoop {
            (true, "Scoop".to_string(), "Scoop 包管理器处于就绪状态，支持自动安装底层 CLI 依赖".to_string())
        } else if has_winget {
            (true, "WinGet".to_string(), "WinGet 包管理器处于就绪状态，支持安装底层 CLI 依赖".to_string())
        } else {
            (false, "Scoop / WinGet".to_string(), "未检测到 Scoop 或 WinGet 包管理器，点击一键自动安装 Scoop".to_string())
        }
    } else {
        let has_brew = env_helper::create_silent_command("brew").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
            || std::path::Path::new("/opt/homebrew/bin/brew").exists()
            || std::path::Path::new("/usr/local/bin/brew").exists();
        if has_brew {
            (true, "Homebrew".to_string(), "Homebrew 处于就绪状态，支持自动安装底层 CLI 依赖".to_string())
        } else {
            (false, "Homebrew".to_string(), "未检测到 Homebrew，点击一键自动安装 Homebrew".to_string())
        }
    };

    checks.push(EnvHealthCheck {
        id: "package-manager".to_string(),
        title: "系统包管理器状态".to_string(),
        status: if pkg_mgr_ok { "ok".to_string() } else { "warning".to_string() },
        message: pkg_mgr_msg,
        shell: if is_windows { "PowerShell".to_string() } else { "bash".to_string() },
        config_file: pkg_mgr_name,
        can_auto_fix: !pkg_mgr_ok,
    });

    Ok(checks)
}

#[tauri::command]
pub async fn auto_fix_health_check(check_id: String) -> Result<bool, String> {
    if check_id == "package-manager" {
        #[cfg(target_os = "windows")]
        {
            // Install Scoop on Windows with TLS 1.2
            let script = r#"
                [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13;
                Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force;
                irm get.scoop.sh | iex
            "#;
            let status = env_helper::create_silent_tokio_command("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
                .status()
                .await
                .map_err(|e| format!("安装 Scoop 失败: {}", e))?;

            if !status.success() {
                return Err(format!("安装 Scoop 返回退出码: {:?}", status.code()));
            }
            env_helper::fix_system_path();
            return Ok(true);
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Install Homebrew on macOS / Linux
            let script = "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"";
            let status = env_helper::create_silent_tokio_command("sh")
                .args(["-c", script])
                .status()
                .await
                .map_err(|e| format!("安装 Homebrew 失败: {}", e))?;

            if !status.success() {
                return Err(format!("Homebrew 安装返回退出码: {:?}", status.code()));
            }
            env_helper::fix_system_path();
            return Ok(true);
        }
    }

    if let Some(home) = dirs::home_dir() {
        #[cfg(target_os = "windows")]
        {
            // 1. Write to PowerShell profile
            let hook = "\n# Mise Version Manager Hook\nif (Get-Command mise -ErrorAction SilentlyContinue) {\n    (& mise activate ps1) | Out-String | Invoke-Expression\n}\n";
            
            let profile_dirs = [
                home.join("Documents/WindowsPowerShell"),
                home.join("Documents/PowerShell"),
            ];

            for p_dir in profile_dirs {
                let _ = fs::create_dir_all(&p_dir);
                let p_file = p_dir.join("Microsoft.PowerShell_profile.ps1");
                let existing = fs::read_to_string(&p_file).unwrap_or_default();
                if !existing.contains("mise activate") {
                    let _ = fs::write(&p_file, format!("{}{}", existing, hook));
                }
            }

            // 2. Inject shims and mise bin into Windows User Registry PATH
            let add_path_script = r#"
                $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
                $shimsPath = "$env:LOCALAPPDATA\mise\shims"
                $binPath = "$env:LOCALAPPDATA\mise\bin"
                $changed = $false
                if ($userPath -notlike "*$shimsPath*") {
                    $userPath = "$shimsPath;$userPath"
                    $changed = $true
                }
                if ($userPath -notlike "*$binPath*") {
                    $userPath = "$binPath;$userPath"
                    $changed = $true
                }
                if ($changed) {
                    [Environment]::SetEnvironmentVariable("Path", $userPath, "User")
                }
            "#;
            let _ = env_helper::create_silent_tokio_command("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", add_path_script])
                .status()
                .await;

            env_helper::fix_system_path();
            return Ok(true);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let shell = env::var("SHELL").unwrap_or_default();
            let rc_path = if shell.ends_with("bash") {
                home.join(".bashrc")
            } else {
                home.join(".zshrc")
            };
            let zprofile_path = home.join(".zprofile");

            // Ensure shims directory exists
            let shims_dir = home.join(".local/share/mise/shims");
            let _ = fs::create_dir_all(&shims_dir);

            let mut fix_lines = Vec::new();
            let existing_rc = fs::read_to_string(&rc_path).unwrap_or_default();

            if !existing_rc.contains("mise/shims") {
                fix_lines.push("\n# Mise Shims PATH\nexport PATH=\"$HOME/.local/share/mise/shims:$PATH\"\n");
            }

            if !existing_rc.contains("mise activate") && !existing_rc.contains("rtx activate") {
                if shell.ends_with("bash") {
                    fix_lines.push("# Mise Version Manager Hook\neval \"$(mise activate bash)\"\n");
                } else {
                    fix_lines.push("# Mise Version Manager Hook\neval \"$(mise activate zsh)\"\n");
                }
            }

            if !fix_lines.is_empty() {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&rc_path)
                    .map_err(|e| format!("打开 {} 失败: {}", rc_path.display(), e))?;

                for l in &fix_lines {
                    file.write_all(l.as_bytes())
                        .map_err(|e| format!("写入 {} 失败: {}", rc_path.display(), e))?;
                }
            }

            // On macOS, also write to .zprofile if not present
            #[cfg(target_os = "macos")]
            {
                let existing_profile = fs::read_to_string(&zprofile_path).unwrap_or_default();
                if !existing_profile.contains("mise/shims") && !existing_profile.contains("mise activate") {
                    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&zprofile_path) {
                        let _ = f.write_all(b"\n# Mise PATH Hook\nexport PATH=\"$HOME/.local/share/mise/shims:$PATH\"\neval \"$(mise activate zsh)\"\n");
                    }
                }
            }

            // Update current running process PATH
            env_helper::fix_system_path();
            return Ok(true);
        }
    }
    Ok(true)
}

#[tauri::command]
pub async fn save_export_file(filename: String, content: String) -> Result<String, String> {
    let download_dir = dirs::download_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("Downloads"))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    });

    let target_file = download_dir.join(&filename);
    fs::write(&target_file, content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if filename.ends_with(".sh") {
            let _ = fs::set_permissions(&target_file, fs::Permissions::from_mode(0o755));
        }
    }

    Ok(target_file.to_string_lossy().to_string())
}
