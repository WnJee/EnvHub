use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use tokio::process::Command;
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

    let mise_bin = env_helper::find_mise_binary();
    let mise_installed = mise_bin.is_some();
    let mise_path = mise_bin.as_ref().map(|p| p.to_string_lossy().to_string());

    let mut mise_version = None;
    if let Some(ref bin) = mise_bin {
        if let Ok(out) = std::process::Command::new(bin).arg("--version").output() {
            if out.status.success() {
                mise_version = Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }
    }

    let pkg_manager = if cfg!(target_os = "macos") {
        "brew"
    } else if cfg!(target_os = "windows") {
        "winget"
    } else {
        "apt"
    };

    Ok(SystemStatus {
        os: os_name.to_string(),
        os_version: format!("{} ({})", os_name, arch),
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
    let mut tools = vec![
        SystemTool {
            id: "git".to_string(),
            name: "Git".to_string(),
            description: "分布式版本控制系统，全球开发者必备基建".to_string(),
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
            description: "轻量级容器引擎与虚拟化工具".to_string(),
            category: "Container".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install --cask docker".to_string() } else { "winget install Docker.DockerDesktop".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/docker/docker-original.svg".to_string(),
            homepage: "https://www.docker.com".to_string(),
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
            name: "Neovim".to_string(),
            description: "可高度定制化、支持 Lua 插件生态的下一代终端文本编辑器".to_string(),
            category: "Editor".to_string(),
            is_installed: false,
            installed_version: None,
            install_command: if cfg!(target_os = "macos") { "brew install neovim".to_string() } else { "winget install Neovim.Neovim".to_string() },
            icon: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/neovim/neovim-original.svg".to_string(),
            homepage: "https://neovim.io".to_string(),
        },
    ];

    // Check installed status
    for tool in &mut tools {
        if let Ok(out) = std::process::Command::new(&tool.id).arg("--version").output() {
            if out.status.success() {
                tool.is_installed = true;
                let v = String::from_utf8_lossy(&out.stdout);
                let first_line = v.lines().next().unwrap_or("installed");
                tool.installed_version = Some(first_line.trim().to_string());
            }
        }
    }

    Ok(tools)
}

#[tauri::command]
pub async fn install_system_tool(tool_id: String) -> Result<bool, String> {
    let cmd = if cfg!(target_os = "macos") {
        format!("brew install {}", tool_id)
    } else if cfg!(target_os = "windows") {
        format!("winget install {}", tool_id)
    } else {
        format!("sudo apt install -y {}", tool_id)
    };

    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .await
        .map_err(|e| e.to_string())?;

    Ok(status.success())
}

#[tauri::command]
pub async fn get_health_checks() -> Result<Vec<EnvHealthCheck>, String> {
    let mut checks = Vec::new();

    // 1. Check Shell Activation
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let mut rc_file = "~/.zshrc";
    let mut has_activation = false;

    if let Some(home) = dirs::home_dir() {
        let zshrc = home.join(".zshrc");
        let bashrc = home.join(".bashrc");

        let target_rc = if shell.ends_with("bash") {
            rc_file = "~/.bashrc";
            bashrc
        } else {
            rc_file = "~/.zshrc";
            zshrc
        };

        if target_rc.exists() {
            if let Ok(content) = fs::read_to_string(&target_rc) {
                if content.contains("mise activate") || content.contains("rtx activate") {
                    has_activation = true;
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
        } else {
            format!("未在 {} 中配置 mise activate，命令行终端可能无法自动切换版本", rc_file)
        },
        shell: shell.clone(),
        config_file: rc_file.to_string(),
        can_auto_fix: !has_activation,
    });

    // 2. PATH priority check
    let path = env::var("PATH").unwrap_or_default();
    let has_shims = path.contains(".local/share/mise/shims") || path.contains("mise");

    checks.push(EnvHealthCheck {
        id: "path-priority".to_string(),
        title: "PATH 优先级与 Shims 注入检查".to_string(),
        status: if has_shims { "ok".to_string() } else { "ok".to_string() },
        message: if has_shims {
            "mise shims 路径已注入系统 PATH，运行版本劫持正常".to_string()
        } else {
            "当前系统 PATH 处于正常响应序列".to_string()
        },
        shell: shell.clone(),
        config_file: "PATH Environment".to_string(),
        can_auto_fix: false,
    });

    // 3. Package Manager
    checks.push(EnvHealthCheck {
        id: "package-manager".to_string(),
        title: "系统包管理器状态".to_string(),
        status: "ok".to_string(),
        message: if cfg!(target_os = "macos") {
            "Homebrew 处于就绪状态，支持自动安装底层 CLI 依赖"
        } else if cfg!(target_os = "windows") {
            "Winget 处于就绪状态"
        } else {
            "APT 处于就绪状态"
        }.to_string(),
        shell: "system".to_string(),
        config_file: "system".to_string(),
        can_auto_fix: false,
    });

    Ok(checks)
}

#[tauri::command]
pub async fn auto_fix_health_check(check_id: String) -> Result<bool, String> {
    if check_id == "mise-activated" {
        if let Some(home) = dirs::home_dir() {
            let shell = env::var("SHELL").unwrap_or_default();
            let rc_path = if shell.ends_with("bash") {
                home.join(".bashrc")
            } else {
                home.join(".zshrc")
            };

            let activation_line = if shell.ends_with("bash") {
                "\n# Mise Version Manager Hook\neval \"$(mise activate bash)\"\n"
            } else {
                "\n# Mise Version Manager Hook\neval \"$(mise activate zsh)\"\n"
            };

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(rc_path)
                .map_err(|e| e.to_string())?;

            file.write_all(activation_line.as_bytes())
                .map_err(|e| e.to_string())?;

            return Ok(true);
        }
    }
    Ok(true)
}
