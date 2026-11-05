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
    /// False when this platform has no automated install path for the tool
    /// (vendor repository required, unsupported package manager, ...).
    #[serde(default = "default_true")]
    pub supported: bool,
    pub icon: String,
    pub homepage: String,
}

fn default_true() -> bool {
    true
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
    // Probing ~30 tools spawns ~30 processes; a burst of post-install refreshes
    // would otherwise re-probe everything on every click.
    if let Some(cached) = SYSTEM_TOOLS_CACHE.get("all") {
        return Ok(cached);
    }
    let mut tools = Vec::new();

    #[cfg(not(target_os = "windows"))]
    tools.push(SystemTool {
        id: "brew".to_string(),
        name: "Homebrew".to_string(),
        description: "macOS 与 Linux 必备的终端软件包与命令行工具包管理器".to_string(),
        category: "Package Manager".to_string(),
        is_installed: false,
        supported: true,
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
        supported: true,
        installed_version: None,
        install_command:
            "powershell -ExecutionPolicy RemoteSigned -Command \"irm get.scoop.sh | iex\""
                .to_string(),
        icon: "https://raw.githubusercontent.com/ScoopInstaller/Scoop/master/packaging/scoop.ico"
            .to_string(),
        homepage: "https://scoop.sh".to_string(),
    });

    tools.extend(vec![
        // 1. 代码协作与版本控制 (VCS)
        SystemTool {
            id: "git".to_string(),
            name: "Git".to_string(),
            description: "全球最主流的代码版本管理与协作工具".to_string(),
            category: "VCS".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install git".to_string() } else { "winget install Git.Git".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/git/git-original.svg".to_string(),
            homepage: "https://git-scm.com".to_string(),
        },
        SystemTool {
            id: "gh".to_string(),
            name: "GitHub CLI (gh)".to_string(),
            description: "GitHub 官方命令行工具，直接在终端管理 PR、Issue、Release 与 Actions".to_string(),
            category: "VCS".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install gh".to_string() } else { "winget install GitHub.cli".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/github/github-original.svg".to_string(),
            homepage: "https://cli.github.com".to_string(),
        },
        SystemTool {
            id: "lazygit".to_string(),
            name: "LazyGit".to_string(),
            description: "极速全键盘操作的终端可视化 Git 分支与提交管理面板".to_string(),
            category: "VCS".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install lazygit".to_string() } else { "winget install JesseDuffield.lazygit".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/git/git-original.svg".to_string(),
            homepage: "https://github.com/jesseduffield/lazygit".to_string(),
        },

        // 2. 容器与服务架构 (Container & Server)
        SystemTool {
            id: "docker".to_string(),
            name: "Docker CLI".to_string(),
            description: "轻量级应用容器化引擎与开发环境虚拟化基建".to_string(),
            category: "Container".to_string(),
            is_installed: false,
        supported: true,
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
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install docker-compose".to_string() } else { "winget install Docker.DockerCompose".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/docker/docker-original.svg".to_string(),
            homepage: "https://docs.docker.com/compose/".to_string(),
        },
        // 3. 数据库与数据分析 (Database)
        SystemTool {
            id: "redis".to_string(),
            name: "Redis".to_string(),
            description: "超高速开源内存键值数据库与高性能消息缓存中间件".to_string(),
            category: "Database".to_string(),
            is_installed: false,
        supported: true,
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
        supported: true,
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
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install postgresql@16".to_string() } else { "winget install PostgreSQL.PostgreSQL".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/postgresql/postgresql-original.svg".to_string(),
            homepage: "https://www.postgresql.org".to_string(),
        },
        SystemTool {
            id: "mongodb".to_string(),
            name: "MongoDB".to_string(),
            description: "面向文档的现代化 NoSQL 高性能高可用分布式数据库".to_string(),
            category: "Database".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install mongodb-community".to_string() } else { "winget install MongoDB.Server".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/mongodb/mongodb-original.svg".to_string(),
            homepage: "https://www.mongodb.com".to_string(),
        },
        SystemTool {
            id: "duckdb".to_string(),
            name: "DuckDB".to_string(),
            description: "高性能进程内嵌入式分析型 SQL 数据库，秒级极速分析 CSV、Parquet 与 JSON".to_string(),
            category: "Database".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install duckdb".to_string() } else { "winget install DuckDB.DuckDB".to_string() },
            icon: "https://duckdb.org/images/favicon/favicon.ico".to_string(),
            homepage: "https://duckdb.org".to_string(),
        },

        // 4. 搜索、代码分析与文本替换 (Search)
        SystemTool {
            id: "ripgrep".to_string(),
            name: "Ripgrep (rg)".to_string(),
            description: "基于 Rust 实现的超高速递归代码与文件全文搜索工具".to_string(),
            category: "Search".to_string(),
            is_installed: false,
        supported: true,
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
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install fd".to_string() } else { "winget install sharkdp.fd".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-original.svg".to_string(),
            homepage: "https://github.com/sharkdp/fd".to_string(),
        },
        SystemTool {
            id: "ast-grep".to_string(),
            name: "ast-grep (sg)".to_string(),
            description: "基于 AST 语法树结构的代码精确搜索、静态分析与重构模式批量替换利器".to_string(),
            category: "Search".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install ast-grep".to_string() } else { "winget install ast-grep.ast-grep".to_string() },
            icon: "https://ast-grep.github.io/logo.svg".to_string(),
            homepage: "https://ast-grep.github.io".to_string(),
        },

        // 5. 音视频与多媒体处理 (Media)
        SystemTool {
            id: "ffmpeg".to_string(),
            name: "FFmpeg".to_string(),
            description: "领先的跨平台音视频编解码、剪辑、格式转换与多媒体流处理框架".to_string(),
            category: "Media".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install ffmpeg".to_string() } else { "winget install Gyan.FFmpeg".to_string() },
            icon: "https://ffmpeg.org/favicon.ico".to_string(),
            homepage: "https://ffmpeg.org".to_string(),
        },
        SystemTool {
            id: "imagemagick".to_string(),
            name: "ImageMagick".to_string(),
            description: "工业级图像处理与格式转换命令行套件，支持批量缩放、裁剪与水印合成".to_string(),
            category: "Media".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install imagemagick".to_string() } else { "winget install ImageMagick.ImageMagick".to_string() },
            icon: "https://imagemagick.org/image/wand.png".to_string(),
            homepage: "https://imagemagick.org".to_string(),
        },
        SystemTool {
            id: "exiftool".to_string(),
            name: "ExifTool".to_string(),
            description: "功能完备的多媒体元数据（EXIF、IPTC、XMP）读取、批量修改与隐私清理神器".to_string(),
            category: "Media".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install exiftool".to_string() } else { "winget install PhilHarvey.ExifTool".to_string() },
            icon: "https://exiftool.org/favicon.ico".to_string(),
            homepage: "https://exiftool.org".to_string(),
        },
        SystemTool {
            id: "yt-dlp".to_string(),
            name: "yt-dlp".to_string(),
            description: "极其强大的多网站音视频与字幕提取、格式转换与批量下载命令行神器".to_string(),
            category: "Media".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install yt-dlp".to_string() } else { "winget install yt-dlp.yt-dlp".to_string() },
            icon: "https://raw.githubusercontent.com/yt-dlp/yt-dlp/master/.github/banner.svg".to_string(),
            homepage: "https://github.com/yt-dlp/yt-dlp".to_string(),
        },

        // 6. 网络传输与云存储 (Network)
        SystemTool {
            id: "aria2".to_string(),
            name: "aria2".to_string(),
            description: "轻量级多协议（HTTP/FTP/BT）、多线程多连接的极速命令行下载引擎".to_string(),
            category: "Network".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install aria2".to_string() } else { "winget install aria2.aria2".to_string() },
            icon: "https://aria2.github.io/favicon.ico".to_string(),
            homepage: "https://aria2.github.io".to_string(),
        },
        // 7. 命令行效率与结构化数据处理 (CLI Utility)
        SystemTool {
            id: "fzf".to_string(),
            name: "FZF".to_string(),
            description: "通用终端交互式模糊搜索神器，快速检索命令历史、文件与 Git 分支".to_string(),
            category: "CLI Utility".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install fzf".to_string() } else { "winget install junegunn.fzf".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/go/go-original.svg".to_string(),
            homepage: "https://github.com/junegunn/fzf".to_string(),
        },
        SystemTool {
            id: "jq".to_string(),
            name: "jq".to_string(),
            description: "轻量级且灵活强大的命令行 JSON 数据查询、切片、过滤与转换工具".to_string(),
            category: "CLI Utility".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install jq".to_string() } else { "winget install jqlang.jq".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/json/json-original.svg".to_string(),
            homepage: "https://jqlang.github.io/jq/".to_string(),
        },
        SystemTool {
            id: "yq".to_string(),
            name: "yq".to_string(),
            description: "便携高效的命令行 YAML、JSON、XML 与 CSV 数据处理与格式互转神器".to_string(),
            category: "CLI Utility".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install yq".to_string() } else { "winget install MikeFarah.yq".to_string() },
            icon: "https://raw.githubusercontent.com/mikefarah/yq/master/assets/yq-icon.png".to_string(),
            homepage: "https://github.com/mikefarah/yq".to_string(),
        },
        SystemTool {
            id: "pandoc".to_string(),
            name: "Pandoc".to_string(),
            description: "全能标记语言文档格式转换瑞士军刀，支持 Markdown、Docx、HTML、PDF 互转".to_string(),
            category: "CLI Utility".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install pandoc".to_string() } else { "winget install JohnMacFarlane.Pandoc".to_string() },
            icon: "https://pandoc.org/logo.png".to_string(),
            homepage: "https://pandoc.org".to_string(),
        },
        SystemTool {
            id: "yazi".to_string(),
            name: "Yazi".to_string(),
            description: "基于 Rust 构建的现代化超极速全键盘异步终端文件管理器与快速预览器".to_string(),
            category: "CLI Utility".to_string(),
            is_installed: false,
        supported: true,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install yazi".to_string() } else { "winget install sxyazi.yazi".to_string() },
            icon: "https://raw.githubusercontent.com/sxyazi/yazi/main/assets/logo.png".to_string(),
            homepage: "https://yazi-rs.github.io".to_string(),
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
        let manager = linux_package_manager();
        match linux_install_args(manager, &tool.id) {
            Ok(args) => tool.install_command = format!("{manager} {}", args.join(" ")),
            Err(reason) => {
                // No automated path on this distribution: surface the reason
                // instead of letting the user click install and hit an error.
                tool.supported = false;
                tool.install_command = reason;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows installs use Scoop first and fall back to WinGet. Report the
        // command that will actually run, not an unconditional `scoop install`.
        let has_scoop = env_helper::command_available("scoop");
        let has_winget = env_helper::command_available("winget");
        for tool in &mut tools {
            if tool.id == "scoop" {
                continue;
            }
            let (scoop_pkg, winget_pkg) = windows_packages_for(&tool.id);
            tool.install_command = if has_scoop {
                format!("scoop install {scoop_pkg}")
            } else if has_winget {
                format!("winget install {winget_pkg}")
            } else {
                tool.supported = false;
                "请先安装 Scoop 或 WinGet".to_string()
            };
        }
    }

    SYSTEM_TOOLS_CACHE.put("all", tools.clone());
    Ok(tools)
}

static SYSTEM_TOOLS_CACHE: crate::cache::TtlCache<Vec<SystemTool>> =
    crate::cache::TtlCache::new(std::time::Duration::from_secs(30));

/// Called by `env_helper::OperationGuard::drop` after any mutation.
pub(crate) fn invalidate_probe_cache() {
    SYSTEM_TOOLS_CACHE.clear();
}

/// Single source of truth for the Windows package names used by both the
/// displayed install command and the actual install call.
#[cfg(any(target_os = "windows", test))]
fn windows_packages_for(tool_id: &str) -> (&str, &str) {
    match tool_id {
        "git" => ("git", "Git.Git"),
        "gh" => ("gh", "GitHub.cli"),
        "lazygit" => ("lazygit", "JesseDuffield.lazygit"),
        "docker" => ("docker", "Docker.DockerDesktop"),
        "docker-compose" => ("docker-compose", "Docker.DockerCompose"),
        "nginx" => ("nginx", "Nginx.Nginx"),
        "redis" => ("redis", "Redis.Redis"),
        "mysql" => ("mysql", "Oracle.MySQL"),
        "postgresql" => ("postgresql", "PostgreSQL.PostgreSQL"),
        "mongodb" => ("mongodb", "MongoDB.Server"),
        "duckdb" => ("duckdb", "DuckDB.DuckDB"),
        "ollama" => ("ollama", "Ollama.Ollama"),
        "ripgrep" => ("ripgrep", "BurntSushi.ripgrep.MSVC"),
        "fd" => ("fd", "sharkdp.fd"),
        "sd" => ("sd", "chmln.sd"),
        "ast-grep" => ("ast-grep", "ast-grep.ast-grep"),
        "ffmpeg" => ("ffmpeg", "Gyan.FFmpeg"),
        "imagemagick" => ("imagemagick", "ImageMagick.ImageMagick"),
        "exiftool" => ("exiftool", "PhilHarvey.ExifTool"),
        "yt-dlp" => ("yt-dlp", "yt-dlp.yt-dlp"),
        "aria2" => ("aria2", "aria2.aria2"),
        "rclone" => ("rclone", "Rclone.Rclone"),
        "bat" => ("bat", "sharkdp.bat"),
        "fzf" => ("fzf", "junegunn.fzf"),
        "zoxide" => ("zoxide", "ajeetdsouza.zoxide"),
        "jq" => ("jq", "jqlang.jq"),
        "yq" => ("yq", "MikeFarah.yq"),
        "pandoc" => ("pandoc", "JohnMacFarlane.Pandoc"),
        "yazi" => ("yazi", "sxyazi.yazi"),
        other => (other, other),
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
        "ripgrep" => ("rg", &["--version"]),
        "fd" => (fd_binary(), &["--version"]),
        "ffmpeg" => ("ffmpeg", &["-version"]),
        "aria2" => ("aria2c", &["--version"]),
        "imagemagick" => ("magick", &["--version"]),
        "exiftool" => ("exiftool", &["-ver"]),
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
    } else if tool_id == "imagemagick" {
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("convert").arg("-version"),
            5,
        )
        .await
        {
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout);
                for word in raw.split(|c: char| c.is_whitespace() || c == '/' || c == '(' || c == ')' || c == ',') {
                    let candidate = word.trim_start_matches(|c: char| !c.is_ascii_digit());
                    let clean: String = candidate.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                    if clean.contains('.') {
                        return Some(clean.trim_end_matches('.').to_string());
                    }
                }
            }
        }
    } else if tool_id == "aria2" {
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("aria2").arg("--version"),
            5,
        )
        .await
        {
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
    } else if tool_id == "ast-grep" {
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("sg").arg("--version"),
            5,
        )
        .await
        {
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

    let binary_name = if tool_id == "ripgrep" {
        "rg"
    } else if tool_id == "fd" {
        fd_binary()
    } else if tool_id == "postgresql" {
        "psql"
    } else if tool_id == "mongodb" {
        "mongod"
    } else if tool_id == "redis" {
        "redis-server"
    } else if tool_id == "aria2" {
        if env_helper::command_available("aria2c") { "aria2c" } else { "aria2" }
    } else if tool_id == "imagemagick" {
        if env_helper::command_available("magick") { "magick" } else { "convert" }
    } else if tool_id == "ast-grep" {
        if env_helper::command_available("ast-grep") { "ast-grep" } else { "sg" }
    } else {
        &tool_id
    };
    let output = match env_helper::output_timeout(
        env_helper::create_silent_tokio_command(binary_name).arg(match tool_id.as_str() {
            "nginx" => "-v",
            "ffmpeg" => "-version",
            "exiftool" => "-ver",
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
    // The probe cache must not survive a mutation.
    invalidate_probe_cache();
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

        let (scoop_pkg, winget_pkg) = windows_packages_for(&tool_id);

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
        if tool_id == "docker" || tool_id == "ollama" {
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
        if manager == "none" {
            return Err(
                "未检测到受支持的包管理器（apt / dnf / pacman / zypper / apk / xbps），请参考发行版文档手动安装".into(),
            );
        }
        let args = linux_install_args(manager, &tool_id)?;
        let executable = match manager {
            "apt" => "/usr/bin/apt-get",
            "dnf" => "/usr/bin/dnf",
            "pacman" => "/usr/bin/pacman",
            "zypper" => "/usr/bin/zypper",
            "apk" => "/sbin/apk",
            "xbps" => "/usr/bin/xbps-install",
            _ => return Err("Unsupported Linux package manager".into()),
        };
        let manual_hint = format!("sudo {executable} {}", args.join(" "));
        let mut command = if env_helper::command_available("pkexec") {
            let mut command = env_helper::create_silent_tokio_command("pkexec");
            command.arg(executable);
            command
        } else if is_root() {
            // Containers and root shells have no polkit session at all.
            env_helper::create_silent_tokio_command(executable)
        } else if env_helper::command_available("sudo") {
            let _ = app.emit(
                "install-log",
                "未检测到 pkexec，改用 sudo -n（需已缓存 sudo 凭据）".to_string(),
            );
            let mut command = env_helper::create_silent_tokio_command("sudo");
            command.arg("-n").arg(executable);
            command
        } else {
            return Err(format!(
                "当前环境缺少 pkexec / sudo，无法图形化提权安装。请在终端手动执行：\n{manual_hint}"
            ));
        };
        command.args(&args);
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
    for (binary, name) in [
        ("apt-get", "apt"),
        ("dnf", "dnf"),
        ("yum", "dnf"),
        ("pacman", "pacman"),
        ("zypper", "zypper"),
        ("apk", "apk"),
        ("xbps-install", "xbps"),
    ] {
        if env_helper::command_available(binary) {
            return name;
        }
    }
    "none"
}

/// True when the process already runs as root, so elevation can be skipped.
#[cfg(target_os = "linux")]
fn is_root() -> bool {
    env_helper::create_silent_command("id")
        .arg("-u")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "0")
        .unwrap_or(false)
}

#[cfg(any(target_os = "linux", test))]
fn linux_install_args(manager: &str, tool: &str) -> Result<Vec<String>, String> {
    let package = match (manager, tool) {
        ("apt", "docker") => "docker.io",
        ("apt", "redis") => "redis-server",
        ("apt", "fd") => "fd-find",
        ("apt", "mysql") => "default-mysql-server",
        ("apt", "exiftool") => "libimage-exiftool-perl",
        ("dnf", "fd") => "fd-find",
        ("dnf", "mysql") | ("pacman", "mysql") => "mariadb",
        ("dnf", "imagemagick") => "ImageMagick",
        ("dnf", "exiftool") => "perl-Image-ExifTool",
        ("pacman", "exiftool") => "perl-image-exiftool",
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
        "zypper" => vec!["install", "--non-interactive"],
        "apk" => vec!["add"],
        "xbps" => vec!["-Sy"],
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

/// Profile files the current shell will actually source, most specific first.
/// Unsupported shells yield an empty list rather than an error so the health
/// panel can still render the rest of its checks.
fn shell_profiles(home: &std::path::Path, shell: &str) -> Vec<std::path::PathBuf> {
    if shell.ends_with("bash") {
        // macOS ships bash as a *login* shell: it sources .bash_profile /
        // .bash_login / .profile and never .bashrc, so writing only .bashrc
        // there silently does nothing.
        if cfg!(target_os = "macos") {
            let existing: Vec<_> = [".bash_profile", ".bash_login", ".profile"]
                .iter()
                .map(|name| home.join(name))
                .filter(|path| path.exists())
                .collect();
            return if existing.is_empty() {
                vec![home.join(".bash_profile")]
            } else {
                existing
            };
        }
        vec![home.join(".bashrc")]
    } else if shell.ends_with("zsh") {
        vec![home.join(".zshrc")]
    } else if shell.ends_with("fish") {
        vec![home.join(".config/fish/config.fish")]
    } else {
        Vec::new()
    }
}

/// PowerShell profiles to inspect on Windows.
///
/// `Documents` can be redirected (OneDrive / KFM) and users may override the
/// profile entirely, so ask PowerShell for its own `$PROFILE` values and fall
/// back to the conventional locations only when that probe fails.
async fn windows_shell_profiles(home: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found: Vec<std::path::PathBuf> = Vec::new();
    for host in ["pwsh", "powershell"] {
        let probe = env_helper::output_timeout(
            env_helper::create_silent_tokio_command(host).args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "$PROFILE.CurrentUserAllHosts; $PROFILE.CurrentUserCurrentHost",
            ]),
            5,
        )
        .await;
        if let Ok(output) = probe {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let line = line.trim();
                    if line.ends_with(".ps1") && !line.contains('$') {
                        let path = std::path::PathBuf::from(line);
                        if !found.contains(&path) {
                            found.push(path);
                        }
                    }
                }
            }
        }
    }
    if !found.is_empty() {
        return found;
    }
    let documents = dirs::document_dir().unwrap_or_else(|| home.join("Documents"));
    let mut candidates = Vec::new();
    for dir in ["PowerShell", "WindowsPowerShell"] {
        candidates.push(documents.join(dir).join("Microsoft.PowerShell_profile.ps1"));
        candidates.push(documents.join(dir).join("profile.ps1"));
    }
    let existing: Vec<_> = candidates.iter().filter(|p| p.exists()).cloned().collect();
    if existing.is_empty() {
        candidates
    } else {
        existing
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
        windows_shell_profiles(&home).await
    } else {
        shell_profiles(&home, &shell)
    };
    // A shell we cannot configure must degrade to an actionable warning instead
    // of failing the whole command (which blanked the entire health tab).
    let shell_supported = !profiles.is_empty();
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
    let profile = profiles
        .first()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let activation_status = if ready {
        "ok"
    } else if shell_supported {
        "warning"
    } else {
        "error"
    };
    let activation_message = if ready {
        "已检测到当前 Shell 的 Mise 激活配置；新终端生效".to_string()
    } else if shell_supported {
        "当前 Shell 尚未配置可用的 Mise 激活".to_string()
    } else {
        format!(
            "EnvHub 暂不支持自动配置 {shell}。请手动在该 Shell 的启动文件中添加 `mise activate`（参考 https://mise.jdx.dev）"
        )
    };
    Ok(vec![
        EnvHealthCheck {
            id: "mise-activated".into(),
            title: "Shell 激活配置".into(),
            status: activation_status.into(),
            message: activation_message.clone(),
            shell: shell.clone(),
            config_file: profile.clone(),
            can_auto_fix: !ready && shell_supported,
        },
        EnvHealthCheck {
            id: "path-priority".into(),
            title: "终端版本切换配置".into(),
            status: activation_status.into(),
            message: if shell_supported {
                "检查持久化的 Shell 激活配置，不把 GUI 进程 PATH 当作系统配置".to_string()
            } else {
                activation_message
            },
            shell: shell.clone(),
            config_file: profile,
            can_auto_fix: !ready && shell_supported,
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
        // Use the same resolver as the health check so detection and repair
        // never disagree about which profile is the real one.
        let detected = windows_shell_profiles(&home).await;
        let targets = if detected.is_empty() {
            vec![
                documents.join("WindowsPowerShell/Microsoft.PowerShell_profile.ps1"),
                documents.join("PowerShell/Microsoft.PowerShell_profile.ps1"),
            ]
        } else {
            detected
        };
        // Only the host-specific profiles are written: `profile.ps1` would run in
        // addition to them and double-activate mise.
        for path in targets {
            if path.file_name().map(|n| n != "Microsoft.PowerShell_profile.ps1") == Some(true) {
                continue;
            }
            let old = crate::config_file::read(&path)?;
            if !old.contains(&hook) {
                crate::config_file::write(&path, &(old + &hook))?;
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        let profiles = shell_profiles(&home, &shell);
        let Some(path) = profiles.first() else {
            return Err(format!(
                "暂不支持自动配置 {shell}，请手动在其启动文件中添加 `mise activate`"
            ));
        };
        let old = crate::config_file::read(path)?;
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
            crate::config_file::write(path, &(old + &hook))?;
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

    #[test]
    fn linux_install_args_supports_extra_distributions() {
        assert_eq!(
            linux_install_args("zypper", "git").unwrap(),
            ["install", "--non-interactive", "git"]
        );
        assert_eq!(linux_install_args("apk", "git").unwrap(), ["add", "git"]);
        assert_eq!(linux_install_args("xbps", "git").unwrap(), ["-Sy", "git"]);
        // Vendor-repository-only tools stay unsupported everywhere.
        assert!(linux_install_args("zypper", "mongodb").is_err());
    }

    #[test]
    fn windows_package_table_maps_known_tools_and_falls_back_to_id() {
        assert_eq!(windows_packages_for("ripgrep"), ("ripgrep", "BurntSushi.ripgrep.MSVC"));
        assert_eq!(windows_packages_for("git"), ("git", "Git.Git"));
        assert_eq!(windows_packages_for("unknown-tool"), ("unknown-tool", "unknown-tool"));
    }

    #[test]
    fn bash_profile_is_never_bashrc_on_macos() {
        let home = tempfile::tempdir().unwrap();
        let profiles = shell_profiles(home.path(), "/bin/bash");
        assert_eq!(profiles.len(), 1);
        if cfg!(target_os = "macos") {
            assert!(profiles[0].ends_with(".bash_profile"));
        } else {
            assert!(profiles[0].ends_with(".bashrc"));
        }
        assert!(shell_profiles(home.path(), "/usr/bin/zsh")[0].ends_with(".zshrc"));
        assert!(shell_profiles(home.path(), "/usr/bin/fish")[0]
            .ends_with("config.fish"));
        // Unsupported shells degrade to an empty list instead of an error.
        assert!(shell_profiles(home.path(), "/usr/bin/nu").is_empty());
    }
}
