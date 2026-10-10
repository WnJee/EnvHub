use crate::env_helper;
use serde::{Deserialize, Serialize};
use std::env;
#[cfg(unix)]
use std::fs;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};

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
    // A Tauri GUI launched from Explorer inherits a minimal/stale PATH.
    // Normalize it before probing mise and package managers.

    let os_name = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };

    let arch = env::consts::ARCH.to_string();
    let default_shell = env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(windows) {
            "powershell.exe".to_string()
        } else {
            "/bin/zsh".to_string()
        }
    });

    #[allow(unused_mut)]
    let mut os_version = format!("{} ({})", os_name, arch);
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("sw_vers").arg("-productVersion"),
            5,
        )
        .await
        {
            if out.status.success() {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                os_version = format!("macOS {}", v);
            }
        }
    }

    let mise_bin = env_helper::find_mise_binary();
    let mise_path = mise_bin.as_ref().map(|p| p.to_string_lossy().to_string());

    let mut mise_version = None;
    if let Some(ref bin) = mise_bin {
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command(&bin.to_string_lossy()).arg("--version"),
            5,
        )
        .await
        {
            if out.status.success() {
                mise_version = Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }
    }

    let pkg_manager = if cfg!(target_os = "macos") {
        if env_helper::output_timeout(
            env_helper::create_silent_tokio_command("brew").arg("--version"),
            5,
        )
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
        {
            "brew"
        } else {
            "none"
        }
    } else if cfg!(target_os = "windows") {
        if env_helper::output_timeout(
            env_helper::create_silent_tokio_command("winget").arg("--version"),
            5,
        )
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
            || env_helper::command_available("winget")
        {
            "winget"
        } else if env_helper::output_timeout(
            env_helper::create_silent_tokio_command("scoop").arg("--version"),
            5,
        )
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
            || env_helper::command_available("scoop")
        {
            "scoop"
        } else {
            "none"
        }
    } else {
        linux_package_manager()
    };

    Ok(SystemStatus {
        os: os_name.to_string(),
        os_version,
        arch,
        default_shell,
        mise_installed: mise_version.is_some(),
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
        install_command:
            "powershell -ExecutionPolicy RemoteSigned -Command \"irm get.scoop.sh | iex\""
                .to_string(),
        icon: "https://raw.githubusercontent.com/ScoopInstaller/Scoop/master/packaging/scoop.ico"
            .to_string(),
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
    let mut tasks = tokio::task::JoinSet::new();
    let limit = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
    for (index, tool) in tools.iter().enumerate() {
        let id = tool.id.clone();
        let limit = limit.clone();
        tasks.spawn(async move {
            let _permit = limit.acquire().await.unwrap();
            (index, probe_tool_version(&id).await)
        });
    }
    while let Some(result) = tasks.join_next().await {
        let (index, version) = result.map_err(|e| e.to_string())?;
        tools[index].is_installed = version.is_some();
        tools[index].installed_version = version;
    }
    #[cfg(target_os = "linux")]
    for tool in &mut tools {
        tool.install_command = linux_install_args(linux_package_manager(), &tool.id)
            .map(|args| format!("{} {}", linux_package_manager(), args.join(" ")))
            .unwrap_or_else(|reason| reason);
    }

    #[cfg(target_os = "windows")]
    for tool in &mut tools {
        if tool.id != "scoop" {
            // Windows installs use Scoop first. `install_system_tool` only
            // falls back to WinGet when Scoop is unavailable.
            tool.install_command = format!("scoop install {}", scoop_package_for(&tool.id));
        }
    }

    Ok(tools)
}

#[cfg(target_os = "windows")]
fn scoop_package_for(tool_id: &str) -> &str {
    match tool_id {
        "docker-compose" => "docker-compose",
        "postgresql" => "postgresql",
        "mongodb" => "mongodb",
        "ripgrep" => "ripgrep",
        "neovim" => "neovim",
        _ => tool_id,
    }
}

fn fd_binary() -> &'static str {
    // Debian renames the executable; Fedora/Arch and manual installs use `fd`.
    if cfg!(target_os = "linux") && !env_helper::command_available("fd") {
        "fdfind"
    } else {
        "fd"
    }
}

async fn probe_tool_version(tool_id: &str) -> Option<String> {
    if tool_id == "brew" || tool_id == "homebrew" {
        for b in &[
            "brew",
            "/opt/homebrew/bin/brew",
            "/usr/local/bin/brew",
            "/home/linuxbrew/.linuxbrew/bin/brew",
        ] {
            if let Ok(out) = env_helper::output_timeout(
                env_helper::create_silent_tokio_command(b).arg("--version"),
                5,
            )
            .await
            {
                if out.status.success() {
                    let raw = String::from_utf8_lossy(&out.stdout);
                    for word in raw.split(|c: char| {
                        c.is_whitespace() || c == '/' || c == '(' || c == ')' || c == ','
                    }) {
                        let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                        let semver: String = candidate
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                            .collect();
                        let clean = semver.trim_end_matches('.');
                        if clean.contains('.')
                            && clean
                                .chars()
                                .next()
                                .map(|c| c.is_ascii_digit())
                                .unwrap_or(false)
                        {
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
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("scoop").arg("--version"),
            5,
        )
        .await
        {
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout);
                for word in raw.split_whitespace() {
                    let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                    let semver: String = candidate
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    let clean = semver.trim_end_matches('.');
                    if clean.contains('.')
                        && clean
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                    {
                        return Some(clean.to_string());
                    }
                }
                return Some(raw.lines().next().unwrap_or("installed").trim().to_string());
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(out) = env_helper::output_timeout(
                env_helper::create_silent_tokio_command("powershell").args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    "scoop --version",
                ]),
                5,
            )
            .await
            {
                if out.status.success() {
                    let raw = String::from_utf8_lossy(&out.stdout);
                    for word in raw.split_whitespace() {
                        let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                        let semver: String = candidate
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                            .collect();
                        let clean = semver.trim_end_matches('.');
                        if clean.contains('.')
                            && clean
                                .chars()
                                .next()
                                .map(|c| c.is_ascii_digit())
                                .unwrap_or(false)
                        {
                            return Some(clean.to_string());
                        }
                    }
                    return Some(raw.lines().next().unwrap_or("installed").trim().to_string());
                }
            }
            if env_helper::command_available("scoop") {
                return Some("已安装（版本信息暂不可用）".to_string());
            }
        }
        return None;
    }

    let (cmd, args): (&str, &[&str]) = match tool_id {
        // Docker Desktop on Windows (and recent Linux/macOS installs) exposes
        // Compose as the `docker compose` subcommand rather than a standalone
        // `docker-compose` executable. Keep the standalone probe as a fallback.
        "docker-compose" => ("docker-compose", &["--version"]),
        "nginx" => ("nginx", &["-v"]),
        "redis" => ("redis-server", &["--version"]),
        "mysql" => ("mysql", &["--version"]),
        "postgresql" => ("psql", &["--version"]),
        "mongodb" => ("mongod", &["--version"]),
        "neovim" => ("nvim", &["--version"]),
        "ripgrep" => ("rg", &["--version"]),
        "fd" => (fd_binary(), &["--version"]),
        "ffmpeg" => ("ffmpeg", &["-version"]),
        _ => (tool_id, &["--version"]),
    };

    if let Ok(out) =
        env_helper::output_timeout(env_helper::create_silent_tokio_command(cmd).args(args), 5).await
    {
        if out.status.success() || (!out.stderr.is_empty() && tool_id == "nginx") {
            let stdout_str = String::from_utf8_lossy(&out.stdout);
            let stderr_str = String::from_utf8_lossy(&out.stderr);
            let raw = if !stdout_str.trim().is_empty() {
                stdout_str
            } else {
                stderr_str
            };

            for word in raw
                .split(|c: char| c.is_whitespace() || c == '/' || c == '(' || c == ')' || c == ',')
            {
                let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                let semver: String = candidate
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                let clean = semver.trim_end_matches('.');
                if clean.contains('.')
                    && clean
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                {
                    return Some(clean.to_string());
                }
            }
            let first_line = raw.lines().next().unwrap_or("installed");
            return Some(first_line.trim().to_string());
        }
    }

    // Fallbacks
    if tool_id == "docker-compose" {
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("docker").args(["compose", "version"]),
            5,
        )
        .await
        {
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout);
                for word in raw.split_whitespace() {
                    let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                    let clean: String = candidate
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if clean.contains('.') {
                        return Some(clean.trim_end_matches('.').to_string());
                    }
                }
            }
        }
    } else if tool_id == "mongodb" {
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("mongosh").args(["--version"]),
            5,
        )
        .await
        {
            if out.status.success() {
                return Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }
    } else if tool_id == "redis" {
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("redis-cli").args(["--version"]),
            5,
        )
        .await
        {
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout);
                for word in raw.split_whitespace() {
                    let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                    let clean: String = candidate
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
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
        for b in &[
            "brew",
            "/opt/homebrew/bin/brew",
            "/usr/local/bin/brew",
            "/home/linuxbrew/.linuxbrew/bin/brew",
        ] {
            if let Ok(output) = env_helper::output_timeout(
                env_helper::create_silent_tokio_command(b).arg("--version"),
                5,
            )
            .await
            {
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
        if let Ok(output) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("scoop").arg("--version"),
            5,
        )
        .await
        {
            if output.status.success() {
                let out_str = String::from_utf8_lossy(&output.stdout);
                let first_line = out_str.lines().next().unwrap_or("执行正常").trim();
                return Ok(format!("测试成功: {}", first_line));
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = env_helper::output_timeout(
                env_helper::create_silent_tokio_command("powershell").args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    "scoop --version",
                ]),
                5,
            )
            .await
            {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    let first_line = out_str.lines().next().unwrap_or("执行正常").trim();
                    return Ok(format!("测试成功: {}", first_line));
                }
            }
            if env_helper::command_available("scoop") {
                return Ok("测试成功: Scoop shim 已就绪（版本信息暂不可用）".to_string());
            }
        }
        return Err("未检测到 Scoop 命令".to_string());
    }

    if tool_id == "docker-compose" {
        if let Ok(output) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("docker").args(["compose", "version"]),
            5,
        )
        .await
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                return Ok(format!(
                    "测试成功: {}",
                    text.lines().next().unwrap_or("Docker Compose").trim()
                ));
            }
        }
    }

    let binary_name = if tool_id == "neovim" {
        "nvim"
    } else if tool_id == "ripgrep" {
        "rg"
    } else if tool_id == "fd" {
        fd_binary()
    } else if tool_id == "postgresql" {
        "psql"
    } else if tool_id == "mongodb" {
        "mongod"
    } else if tool_id == "redis" {
        "redis-server"
    } else {
        &tool_id
    };
    let output = match env_helper::output_timeout(
        env_helper::create_silent_tokio_command(binary_name).arg(match tool_id.as_str() {
            "nginx" => "-v",
            "ffmpeg" => "-version",
            _ => "--version",
        }),
        5,
    )
    .await
    {
        Ok(output) => {
            if output.status.success() || tool_id != "redis" {
                output
            } else {
                // Windows Redis distributions often ship only redis-cli.exe.
                env_helper::output_timeout(
                    env_helper::create_silent_tokio_command("redis-cli").arg("--version"),
                    5,
                )
                .await
                .map_err(|e| format!("无法执行 redis-cli: {}", e))?
            }
        }
        Err(_err) if tool_id == "redis" => {
            // Windows Redis distributions often ship only redis-cli.exe.
            env_helper::output_timeout(
                env_helper::create_silent_tokio_command("redis-cli").arg("--version"),
                5,
            )
            .await
            .map_err(|e| format!("无法执行 redis-cli: {}", e))?
        }
        Err(err) => return Err(format!("无法执行 {}: {}", binary_name, err)),
    };

    if output.status.success() {
        let out_str = String::from_utf8_lossy(&output.stdout);
        let first_line = out_str.lines().next().unwrap_or("执行正常").trim();
        Ok(format!("测试成功: {}", first_line))
    } else {
        Err("执行返回非零状态码".to_string())
    }
}

#[cfg(target_os = "macos")]
pub fn launch_homebrew_terminal_installer() -> Result<bool, String> {
    let script_content = r#"#!/bin/bash
clear
echo "========================================================"
echo "  ⚡ EnvHub - macOS Homebrew 自动化安装与加速向导"
echo "========================================================"
echo ""

# 1. 检查是否已经安装过 Homebrew
if [ -x "/opt/homebrew/bin/brew" ] || [ -x "/usr/local/bin/brew" ] || command -v brew &>/dev/null; then
    echo "✓ 检测到系统已存在 Homebrew，正在为您配置环境变量..."
    if [ -x "/opt/homebrew/bin/brew" ]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
        if [ -f "$HOME/.zprofile" ] && ! grep -q "/opt/homebrew/bin/brew shellenv" "$HOME/.zprofile" 2>/dev/null; then
            echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> "$HOME/.zprofile"
        fi
        if [ -f "$HOME/.zshrc" ] && ! grep -q "/opt/homebrew/bin/brew shellenv" "$HOME/.zshrc" 2>/dev/null; then
            echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> "$HOME/.zshrc"
        fi
    elif [ -x "/usr/local/bin/brew" ]; then
        eval "$(/usr/local/bin/brew shellenv)"
        if [ -f "$HOME/.zprofile" ] && ! grep -q "/usr/local/bin/brew shellenv" "$HOME/.zprofile" 2>/dev/null; then
            echo 'eval "$(/usr/local/bin/brew shellenv)"' >> "$HOME/.zprofile"
        fi
        if [ -f "$HOME/.zshrc" ] && ! grep -q "/usr/local/bin/brew shellenv" "$HOME/.zshrc" 2>/dev/null; then
            echo 'eval "$(/usr/local/bin/brew shellenv)"' >> "$HOME/.zshrc"
        fi
    fi
    echo "✓ Homebrew 当前版本: $(brew --version 2>/dev/null | head -n 1 || echo '就绪')"
    echo ""
    echo "========================================================"
    echo "🎉 Homebrew 环境变量已配置就绪，请返回 EnvHub 刷新即可！"
    echo "========================================================"
    echo ""
    read -p "按回车键退出当前窗口..."
    exit 0
fi

echo "正在检测网络连接状况与镜像源可用性..."
USE_MIRROR=0
if ! curl -Is --connect-timeout 4 https://raw.githubusercontent.com >/dev/null 2>&1; then
    echo "⚡ 官方 GitHub 访问超时，将自动启用国内极速镜像源（中科大/清华/Gitee加速）"
    USE_MIRROR=1
else
    echo "✓ GitHub 官方源网络连接正常"
fi

echo ""
echo "--------------------------------------------------------"
echo "提示: Homebrew 安装需要管理员权限，若出现密码提示请输入开机密码并按回车（输入时屏幕不显示密码属于正常现象）"
echo "--------------------------------------------------------"
echo ""

if [ "$USE_MIRROR" -eq 1 ]; then
    echo ">> 正在拉取国内一键安装脚本..."
    /bin/zsh -c "$(curl -fsSL https://gitee.com/cunkai/HomebrewCN/raw/master/Homebrew.sh)"
else
    echo ">> 正在启动 Homebrew 官方安装程序..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# 安装后处理：配置环境变量
if [ -x "/opt/homebrew/bin/brew" ]; then
    eval "$(/opt/homebrew/bin/brew shellenv)"
    if [ -f "$HOME/.zprofile" ] && ! grep -q "/opt/homebrew/bin/brew shellenv" "$HOME/.zprofile" 2>/dev/null; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> "$HOME/.zprofile"
    fi
    if [ -f "$HOME/.zshrc" ] && ! grep -q "/opt/homebrew/bin/brew shellenv" "$HOME/.zshrc" 2>/dev/null; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> "$HOME/.zshrc"
    fi
elif [ -x "/usr/local/bin/brew" ]; then
    eval "$(/usr/local/bin/brew shellenv)"
    if [ -f "$HOME/.zprofile" ] && ! grep -q "/usr/local/bin/brew shellenv" "$HOME/.zprofile" 2>/dev/null; then
        echo 'eval "$(/usr/local/bin/brew shellenv)"' >> "$HOME/.zprofile"
    fi
    if [ -f "$HOME/.zshrc" ] && ! grep -q "/usr/local/bin/brew shellenv" "$HOME/.zshrc" 2>/dev/null; then
        echo 'eval "$(/usr/local/bin/brew shellenv)"' >> "$HOME/.zshrc"
    fi
fi

echo ""
echo "========================================================"
if [ -x "/opt/homebrew/bin/brew" ] || [ -x "/usr/local/bin/brew" ] || command -v brew &>/dev/null; then
    echo "🎉 Homebrew 安装并配置成功！"
    echo "当前版本: $(brew --version 2>/dev/null | head -n 1 || echo '就绪')"
    echo "现在可以返回 EnvHub 体验一键安装工具与版本管理了。"
else
    echo "⚠️ 安装流程已结束。若尚未生效，请根据终端上方提示重试或手动运行安装。"
fi
echo "========================================================"
echo ""
read -p "按回车键退出当前窗口..."
"#;

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("envhub_install_homebrew.command");
    std::fs::write(&script_path, script_content).map_err(|e| format!("创建安装脚本失败: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755));
    }

    let status = std::process::Command::new("open")
        .args(["-a", "Terminal", script_path.to_str().unwrap_or("")])
        .status();

    if status.is_err() || !status.as_ref().unwrap().success() {
        let _ = std::process::Command::new("open")
            .arg(&script_path)
            .status()
            .map_err(|e| format!("拉起终端安装向导失败: {}", e))?;
    }

    Ok(true)
}

#[tauri::command]
pub async fn install_system_tool(app: AppHandle, tool_id: String) -> Result<bool, String> {
    let _operation = env_helper::begin_operation()?;
    env_helper::validate_target(&tool_id, "latest")?;
    // GUI applications inherit a stale PATH. Refresh it before looking for
    // Scoop/WinGet or any other package manager installed after app launch.

    let _ = app.emit("install-log", format!("> 开始安装系统工具: {}", tool_id));
    let _ = app.emit("install-progress", 10);

    #[cfg(target_os = "windows")]
    {
        if tool_id == "scoop" {
            let script = r#"
                # TLS 1.2 is available in Windows PowerShell 5.1 and newer.
                [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12;
                Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force;
                irm get.scoop.sh | iex
            "#;
            let _ = app.emit("install-log", "正在执行 Scoop 安装脚本...".to_string());
            let mut command = env_helper::create_silent_tokio_command("powershell");
            command.args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!("$ErrorActionPreference='Stop'; {script}"),
            ]);
            stream_install(&app, command).await?;
            if !env_helper::command_available("scoop") {
                return Err("Scoop installer completed but command is missing".into());
            }
            return Ok(true);
        }

        let has_scoop = env_helper::command_available("scoop");
        let has_winget = env_helper::command_available("winget");

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
            format!(
                "winget install --accept-source-agreements --accept-package-agreements {}",
                winget_pkg
            )
        } else {
            return Err(
                "未在 Windows 系统中检测到 Scoop 或 WinGet，请先在工具箱首位安装 Scoop".to_string(),
            );
        };

        let mut command = env_helper::create_silent_tokio_command("powershell");
        command.args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &format!("{}; if (-not $?) {{ exit 1 }}", cmd_string),
        ]);
        stream_install(&app, command).await
    }

    #[cfg(target_os = "macos")]
    {
        if tool_id == "brew" || tool_id == "homebrew" {
            let _ = app.emit(
                "install-log",
                "⚡ Homebrew 需要系统管理员 (sudo) 权限与交互式安装环境...".to_string(),
            );
            let _ = app.emit(
                "install-log",
                "🚀 已为您拉起 macOS 原生终端窗口执行 Homebrew 智能安装向导".to_string(),
            );
            let _ = app.emit(
                "install-log",
                "💡 请在弹出的终端窗口中输入开机密码以完成授权，安装完成后回到应用刷新即可。"
                    .to_string(),
            );
            let _ = app.emit("install-progress", 100);
            launch_homebrew_terminal_installer()?;
            return Err("已打开 Homebrew 安装终端；请完成安装后刷新，当前尚未确认安装成功".into());
        }

        // Check if brew is installed for other tools (git, docker, redis, etc.)
        let brew_bin = if std::path::Path::new("/opt/homebrew/bin/brew").exists() {
            "/opt/homebrew/bin/brew".to_string()
        } else if std::path::Path::new("/usr/local/bin/brew").exists() {
            "/usr/local/bin/brew".to_string()
        } else if env_helper::command_available("brew") {
            "brew".to_string()
        } else {
            let _ = app.emit(
                "install-log",
                "❌ 未检测到 Homebrew 包管理器，无法继续安装此工具。".to_string(),
            );
            let _ = app.emit(
                "install-log",
                "🚀 正在自动为您唤起 Homebrew 安装向导...".to_string(),
            );
            let _ = launch_homebrew_terminal_installer();
            return Err(
                "未检测到 Homebrew，已为您唤起 Homebrew 安装向导，请先完成 Homebrew 安装"
                    .to_string(),
            );
        };

        let mut command = env_helper::create_silent_tokio_command(&brew_bin);
        command.arg("install");
        if tool_id == "docker" {
            command.arg("--cask");
        }
        command.arg(&tool_id);
        stream_install(&app, command).await
    }

    #[cfg(target_os = "linux")]
    {
        if tool_id == "brew" || tool_id == "homebrew" {
            return Err(
                "Install Homebrew from an interactive terminal using https://brew.sh".into(),
            );
        }
        let manager = linux_package_manager();
        let args = linux_install_args(manager, &tool_id)?;
        if !env_helper::command_available("pkexec") {
            return Err("pkexec is required for graphical administrator authorization; install policykit or use your terminal".into());
        }
        let mut command = env_helper::create_silent_tokio_command("pkexec");
        let executable = match manager {
            "apt" => "/usr/bin/apt-get",
            "dnf" => "/usr/bin/dnf",
            "pacman" => "/usr/bin/pacman",
            _ => return Err("Unsupported Linux package manager".into()),
        };
        command.arg(executable).args(args);
        stream_install(&app, command).await
    }
}

pub(crate) async fn stream_install(
    app: &AppHandle,
    mut command: tokio::process::Command,
) -> Result<bool, String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let pid = child.id();
    env_helper::set_active_install_pid(pid);
    let stdout = child.stdout.take().ok_or("Missing stdout")?;
    let stderr = child.stderr.take().ok_or("Missing stderr")?;
    let app1 = app.clone();
    let app2 = app.clone();
    let out = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app1.emit("install-log", line);
        }
    });
    let err = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app2.emit("install-log", line);
        }
    });
    let status = child.wait().await.map_err(|e| e.to_string())?;
    let _ = tokio::join!(out, err);
    env_helper::clear_active_install_pid(pid);
    if !status.success() {
        return Err(format!("Installation failed or cancelled: {status}"));
    }
    let _ = app.emit("install-progress", 100);
    Ok(true)
}

fn linux_package_manager() -> &'static str {
    for (binary, name) in [("apt-get", "apt"), ("dnf", "dnf"), ("pacman", "pacman")] {
        if env_helper::command_available(binary) {
            return name;
        }
    }
    "none"
}

#[cfg(any(target_os = "linux", test))]
fn linux_install_args(manager: &str, tool: &str) -> Result<Vec<String>, String> {
    let package = match (manager, tool) {
        ("apt", "docker") => "docker.io",
        ("apt", "redis") => "redis-server",
        ("apt", "fd") => "fd-find",
        ("apt", "mysql") => "default-mysql-server",
        ("dnf", "fd") => "fd-find",
        ("dnf", "mysql") | ("pacman", "mysql") => "mariadb",
        (_, "mongodb" | "ollama" | "lazygit") => {
            return Err(format!(
            "{tool} requires a vendor repository; install it following the official instructions"
        ))
        }
        _ => tool,
    };
    let prefix = match manager {
        "apt" | "dnf" => vec!["install", "-y"],
        "pacman" => vec!["-S", "--noconfirm", "--needed"],
        _ => return Err("Unsupported Linux package manager".into()),
    };
    Ok(prefix
        .into_iter()
        .chain([package])
        .map(str::to_string)
        .collect())
}

fn has_activation(content: &str) -> bool {
    content.lines().any(|line| {
        let line = line.trim();
        !line.starts_with('#')
            && line.contains("activate")
            && (line.contains("mise") || line.contains("MiseBin") || line.contains("rtx"))
            && !line.contains("activate ps1")
    })
}

fn shell_profile(home: &std::path::Path, shell: &str) -> Result<std::path::PathBuf, String> {
    if shell.ends_with("bash") {
        Ok(home.join(".bashrc"))
    } else if shell.ends_with("zsh") {
        Ok(home.join(".zshrc"))
    } else if shell.ends_with("fish") {
        Ok(home.join(".config/fish/config.fish"))
    } else {
        Err(format!(
            "Unsupported shell: {shell}; configure Mise manually"
        ))
    }
}

#[tauri::command]
pub async fn get_health_checks() -> Result<Vec<EnvHealthCheck>, String> {
    let home = dirs::home_dir().ok_or("Cannot locate home directory")?;
    let shell = env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(windows) {
            "PowerShell".into()
        } else {
            "/bin/bash".into()
        }
    });
    let profiles = if cfg!(windows) {
        vec![
            dirs::document_dir()
                .unwrap_or_else(|| home.join("Documents"))
                .join("WindowsPowerShell/Microsoft.PowerShell_profile.ps1"),
            dirs::document_dir()
                .unwrap_or_else(|| home.join("Documents"))
                .join("PowerShell/Microsoft.PowerShell_profile.ps1"),
        ]
    } else {
        vec![shell_profile(&home, &shell)?]
    };
    let activated = profiles.iter().any(|p| {
        crate::config_file::read(p)
            .map(|c| has_activation(&c))
            .unwrap_or(false)
    });
    let ready = activated && env_helper::find_mise_binary().is_some();
    let manager = if cfg!(windows) {
        if env_helper::command_available("scoop") {
            "scoop"
        } else if env_helper::command_available("winget") {
            "winget"
        } else {
            "none"
        }
    } else if cfg!(target_os = "linux") {
        linux_package_manager()
    } else if env_helper::command_available("brew") {
        "brew"
    } else {
        "none"
    };
    let profile = profiles[0].display().to_string();
    Ok(vec![
        EnvHealthCheck {
            id: "mise-activated".into(),
            title: "Shell 激活配置".into(),
            status: if ready { "ok" } else { "warning" }.into(),
            message: if ready {
                "已检测到当前 Shell 的 Mise 激活配置；新终端生效"
            } else {
                "当前 Shell 尚未配置可用的 Mise 激活"
            }
            .into(),
            shell: shell.clone(),
            config_file: profile.clone(),
            can_auto_fix: !ready,
        },
        EnvHealthCheck {
            id: "path-priority".into(),
            title: "终端版本切换配置".into(),
            status: if ready { "ok" } else { "warning" }.into(),
            message: "检查持久化的 Shell 激活配置，不把 GUI 进程 PATH 当作系统配置".into(),
            shell: shell.clone(),
            config_file: profile,
            can_auto_fix: !ready,
        },
        EnvHealthCheck {
            id: "package-manager".into(),
            title: "系统包管理器".into(),
            status: if manager == "none" { "warning" } else { "ok" }.into(),
            message: format!("检测到包管理器: {manager}"),
            shell,
            config_file: manager.into(),
            can_auto_fix: manager == "none" && !cfg!(target_os = "linux"),
        },
    ])
}

#[tauri::command]
pub async fn auto_fix_health_check(check_id: String) -> Result<bool, String> {
    let _operation = env_helper::begin_operation()?;
    if check_id == "package-manager" {
        #[cfg(target_os = "macos")]
        {
            launch_homebrew_terminal_installer()?;
            return Err("已打开 Homebrew 安装终端；请完成安装后刷新，当前尚未确认安装成功".into());
        }
        #[cfg(target_os = "windows")]
        {
            env_helper::checked_output(
                env_helper::output_timeout(
                    env_helper::create_silent_tokio_command("powershell").args([
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-Command",
                        "$ErrorActionPreference='Stop'; irm https://get.scoop.sh | iex",
                    ]),
                    180,
                )
                .await?,
            )?;
            return Ok(true);
        }
        #[cfg(target_os = "linux")]
        {
            return Err(
                "Install a supported package manager using your distribution's instructions".into(),
            );
        }
    }
    if !["mise-activated", "path-priority"].contains(&check_id.as_str()) {
        return Err("Unknown health check".into());
    }
    let home = dirs::home_dir().ok_or("Cannot locate home directory")?;
    let bin = env_helper::find_mise_binary().ok_or("Install Mise first")?;
    let _guard = crate::config_file::CONFIG_LOCK
        .lock()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    {
        let hook = format!("\n# EnvHub Mise activation\n$MiseBin = '{}'\n$MiseShell = if ($PSVersionTable.PSEdition -eq 'Core') {{ 'pwsh' }} else {{ 'powershell' }}\n(& $MiseBin activate $MiseShell) | Out-String | Invoke-Expression\n", bin.to_string_lossy().replace('\'', "''"));
        let documents = dirs::document_dir().unwrap_or_else(|| home.join("Documents"));
        for relative in [
            "WindowsPowerShell/Microsoft.PowerShell_profile.ps1",
            "PowerShell/Microsoft.PowerShell_profile.ps1",
        ] {
            let path = documents.join(relative);
            let old = crate::config_file::read(&path)?;
            if !old.contains(&hook) {
                crate::config_file::write(&path, &(old + &hook))?;
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        let path = shell_profile(&home, &shell)?;
        let old = crate::config_file::read(&path)?;
        let quoted = env_helper::shell_quote(&bin.to_string_lossy());
        let hook = if shell.ends_with("fish") {
            format!("\n# EnvHub Mise activation\n{quoted} activate fish | source\n")
        } else {
            format!(
                "\n# EnvHub Mise activation\neval \"$({quoted} activate {})\"\n",
                if shell.ends_with("bash") {
                    "bash"
                } else {
                    "zsh"
                }
            )
        };
        // An absolute invocation also repairs old hooks whose `mise` is absent from PATH.
        if !old.contains(&hook) {
            crate::config_file::write(&path, &(old + &hook))?;
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

    if std::path::Path::new(&filename)
        .file_name()
        .and_then(|v| v.to_str())
        != Some(filename.as_str())
    {
        return Err("Invalid filename".into());
    }
    let target_file = download_dir.join(&filename);
    crate::config_file::write(&target_file, &content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if filename.ends_with(".sh") {
            let _ = fs::set_permissions(&target_file, fs::Permissions::from_mode(0o755));
        }
    }

    Ok(target_file.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn activation_is_read_only_and_detects_variable_hook() {
        assert!(has_activation(
            "(& $MiseBin activate $MiseShell) | Out-String | Invoke-Expression"
        ));
        assert!(!has_activation("# mise activate zsh"));
        assert!(!has_activation("mise activate ps1"));
    }
    #[test]
    fn package_names_are_platform_specific() {
        assert_eq!(
            linux_install_args("apt", "fd").unwrap(),
            ["install", "-y", "fd-find"]
        );
        assert_eq!(
            linux_install_args("pacman", "mysql").unwrap(),
            ["-S", "--noconfirm", "--needed", "mariadb"]
        );
        assert!(linux_install_args("none", "git").is_err());
    }
}
