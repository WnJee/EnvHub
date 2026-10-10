use crate::env_helper;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MirrorOption {
    pub name: String,
    pub url: String,
    pub ping: Option<u32>,
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MirrorConfig {
    pub id: String,
    pub name: String,
    pub tool: String,
    #[serde(rename = "currentMirror")]
    pub current_mirror: String,
    pub options: Vec<MirrorOption>,
}

#[tauri::command]
pub async fn get_mirrors() -> Result<Vec<MirrorConfig>, String> {
    // Refresh PATH so GUI launches can see Node/Go installed after login.
    let mut configs = Vec::new();

    // Platform package-manager mirrors lead the list because they affect
    // most subsequent installation operations.
    #[cfg(not(target_os = "windows"))]
    configs.push(MirrorConfig {
        id: "brew".to_string(),
        name: "Homebrew (macOS / Linux)".to_string(),
        tool: "brew".to_string(),
        current_mirror: "https://mirrors.ustc.edu.cn/homebrew-bottles".to_string(),
        options: vec![
            MirrorOption {
                name: "中国科学技术大学 USTC 镜像".to_string(),
                url: "https://mirrors.ustc.edu.cn/homebrew-bottles".to_string(),
                ping: None,
                is_default: Some(true),
            },
            MirrorOption {
                name: "清华大学 TUNA 镜像".to_string(),
                url: "https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "阿里云 Homebrew 镜像".to_string(),
                url: "https://mirrors.aliyun.com/homebrew/homebrew-bottles".to_string(),
                ping: None,
                is_default: None,
            },
        ],
    });

    #[cfg(target_os = "windows")]
    {
        let scoop_current = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("powershell").args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "scoop config SCOOP_REPO",
            ]),
            8,
        )
        .await
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://github.com/ScoopInstaller/Scoop".to_string());
        configs.push(MirrorConfig {
            id: "scoop".to_string(),
            name: "Scoop (Windows)".to_string(),
            tool: "scoop".to_string(),
            current_mirror: scoop_current,
            options: vec![
                MirrorOption {
                    name: "Scoop 官方 GitHub 仓库".to_string(),
                    url: "https://github.com/ScoopInstaller/Scoop".to_string(),
                    ping: None,
                    is_default: Some(true),
                },
                MirrorOption {
                    name: "Gitee Scoop 镜像".to_string(),
                    url: "https://gitee.com/scoop-installer/scoop".to_string(),
                    ping: None,
                    is_default: None,
                },
            ],
        });
    }

    // 1. NPM: Read ~/.npmrc or query `npm config get registry`
    let mut npm_current = "https://registry.npmjs.org".to_string();
    if let Some(home) = dirs::home_dir() {
        let npmrc = home.join(".npmrc");
        if npmrc.exists() {
            if let Ok(content) = fs::read_to_string(&npmrc) {
                for line in content.lines() {
                    let trim = line.trim();
                    if trim.starts_with("registry=") || trim.starts_with("registry =") {
                        if let Some(val) = trim.split('=').nth(1) {
                            npm_current = val.trim().to_string();
                        }
                    }
                }
            }
        }
    }
    if npm_current == "https://registry.npmjs.org" {
        if let Ok(out) = env_helper::output_timeout(
            env_helper::create_silent_tokio_command("npm").args(["config", "get", "registry"]),
            8,
        )
        .await
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() && s.starts_with("http") {
                    npm_current = s;
                }
            }
        }
    }

    configs.push(MirrorConfig {
        id: "npm".to_string(),
        name: "NPM (Node.js)".to_string(),
        tool: "npm".to_string(),
        current_mirror: npm_current,
        options: vec![
            MirrorOption {
                name: "淘宝 NPM 镜像 (npmmirror)".to_string(),
                url: "https://registry.npmmirror.com".to_string(),
                ping: None,
                is_default: Some(true),
            },
            MirrorOption {
                name: "腾讯云 NPM 镜像".to_string(),
                url: "https://mirrors.cloud.tencent.com/npm/".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "华为云 NPM 镜像".to_string(),
                url: "https://repo.huaweicloud.com/repository/npm/".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "官方源 (npmjs.org)".to_string(),
                url: "https://registry.npmjs.org".to_string(),
                ping: None,
                is_default: None,
            },
        ],
    });

    // 2. Pip: Read ~/.pip/pip.conf or ~/.config/pip/pip.conf
    let mut pip_current = "https://pypi.org/simple".to_string();
    if let Some(home) = dirs::home_dir() {
        let pip_paths = [
            home.join(".pip/pip.conf"),
            home.join(".config/pip/pip.conf"),
        ];
        for path in pip_paths {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trim = line.trim();
                        if trim.starts_with("index-url") {
                            if let Some(val) = trim.split('=').nth(1) {
                                pip_current = val.trim().to_string();
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    configs.push(MirrorConfig {
        id: "pip".to_string(),
        name: "Pip (Python)".to_string(),
        tool: "pip".to_string(),
        current_mirror: pip_current,
        options: vec![
            MirrorOption {
                name: "清华大学 TUNA 镜像".to_string(),
                url: "https://pypi.tuna.tsinghua.edu.cn/simple".to_string(),
                ping: None,
                is_default: Some(true),
            },
            MirrorOption {
                name: "阿里云开源镜像".to_string(),
                url: "https://mirrors.aliyun.com/pypi/simple/".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "豆瓣开源镜像".to_string(),
                url: "https://pypi.doubanio.com/simple/".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "官方 PyPI 源".to_string(),
                url: "https://pypi.org/simple".to_string(),
                ping: None,
                is_default: None,
            },
        ],
    });

    // 3. Go: Query `go env GOPROXY`
    let mut go_current = "https://proxy.golang.org,direct".to_string();
    if let Ok(out) = env_helper::output_timeout(
        env_helper::create_silent_tokio_command("go").args(["env", "GOPROXY"]),
        8,
    )
    .await
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                go_current = s;
            }
        }
    }

    configs.push(MirrorConfig {
        id: "go".to_string(),
        name: "Go Modules (GOPROXY)".to_string(),
        tool: "go".to_string(),
        current_mirror: go_current,
        options: vec![
            MirrorOption {
                name: "Goproxy 中国 (七牛云)".to_string(),
                url: "https://goproxy.cn,direct".to_string(),
                ping: None,
                is_default: Some(true),
            },
            MirrorOption {
                name: "阿里云 Go 模块代理".to_string(),
                url: "https://mirrors.aliyun.com/goproxy/,direct".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "官方 proxy.golang.org".to_string(),
                url: "https://proxy.golang.org,direct".to_string(),
                ping: None,
                is_default: None,
            },
        ],
    });

    // 4. Cargo: Read ~/.cargo/config.toml
    let mut cargo_current = "https://github.com/rust-lang/crates.io-index".to_string();
    if let Some(home) = dirs::home_dir() {
        let cargo_paths = [home.join(".cargo/config.toml"), home.join(".cargo/config")];
        for path in cargo_paths {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("rsproxy") {
                        cargo_current = "https://rsproxy.cn".to_string();
                    } else if content.contains("ustc") {
                        cargo_current = "https://mirrors.ustc.edu.cn/crates.io-index".to_string();
                    } else if content.contains("tuna") {
                        cargo_current =
                            "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"
                                .to_string();
                    }
                }
            }
        }
    }

    configs.push(MirrorConfig {
        id: "cargo".to_string(),
        name: "Cargo (Rust Crates)".to_string(),
        tool: "cargo".to_string(),
        current_mirror: cargo_current,
        options: vec![
            MirrorOption {
                name: "字节跳动 rsproxy (推荐)".to_string(),
                url: "https://rsproxy.cn".to_string(),
                ping: None,
                is_default: Some(true),
            },
            MirrorOption {
                name: "中国科学技术大学 USTC".to_string(),
                url: "https://mirrors.ustc.edu.cn/crates.io-index".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "清华大学 Crates 镜像".to_string(),
                url: "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "官方 crates.io".to_string(),
                url: "https://github.com/rust-lang/crates.io-index".to_string(),
                ping: None,
                is_default: None,
            },
        ],
    });

    // 5. Docker Hub 容器镜像源
    let mut docker_current = "https://registry-1.docker.io".to_string();
    if let Some(home) = dirs::home_dir() {
        let docker_cfg = home.join(".docker/daemon.json");
        if docker_cfg.exists() {
            if let Ok(content) = fs::read_to_string(&docker_cfg) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(mirrors) = val.get("registry-mirrors").and_then(|m| m.as_array()) {
                        if let Some(first) = mirrors.first().and_then(|f| f.as_str()) {
                            docker_current = first.to_string();
                        }
                    }
                }
            }
        }
    }

    configs.push(MirrorConfig {
        id: "docker".to_string(),
        name: "Docker Hub 容器镜像加速".to_string(),
        tool: "docker".to_string(),
        current_mirror: docker_current,
        options: vec![
            MirrorOption {
                name: "DaoCloud 镜像加速".to_string(),
                url: "https://docker.m.daocloud.io".to_string(),
                ping: None,
                is_default: Some(true),
            },
            MirrorOption {
                name: "中科大 Docker 镜像源".to_string(),
                url: "https://docker.mirrors.ustc.edu.cn".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "腾讯云容器镜像代理".to_string(),
                url: "https://mirror.ccs.tencentyun.com".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "Docker 官方 Docker Hub".to_string(),
                url: "https://registry-1.docker.io".to_string(),
                ping: None,
                is_default: None,
            },
        ],
    });

    #[cfg(target_os = "windows")]
    {
        configs.push(MirrorConfig {
            id: "nuget".to_string(),
            name: "NuGet (.NET Windows)".to_string(),
            tool: "nuget".to_string(),
            current_mirror: "https://nuget.cdn.azure.cn/v3/index.json".to_string(),
            options: vec![
                MirrorOption {
                    name: "Azure 中国 CDN 镜像 (推荐)".to_string(),
                    url: "https://nuget.cdn.azure.cn/v3/index.json".to_string(),
                    ping: None,
                    is_default: Some(true),
                },
                MirrorOption {
                    name: "华为云 NuGet 镜像".to_string(),
                    url: "https://repo.huaweicloud.com/repository/nuget/v3/index.json".to_string(),
                    ping: None,
                    is_default: None,
                },
                MirrorOption {
                    name: "NuGet.org 官方源".to_string(),
                    url: "https://api.nuget.org/v3/index.json".to_string(),
                    ping: None,
                    is_default: None,
                },
            ],
        });
    }

    // 7. Maven (Java)
    let mut maven_current = "https://repo.maven.apache.org/maven2".to_string();
    if let Some(home) = dirs::home_dir() {
        let m2_settings = home.join(".m2/settings.xml");
        if m2_settings.exists() {
            if let Ok(content) = fs::read_to_string(&m2_settings) {
                if content.contains("aliyun") {
                    maven_current = "https://maven.aliyun.com/repository/public".to_string();
                } else if content.contains("huaweicloud") {
                    maven_current = "https://repo.huaweicloud.com/repository/maven/".to_string();
                }
            }
        }
    }

    configs.push(MirrorConfig {
        id: "maven".to_string(),
        name: "Maven (Java)".to_string(),
        tool: "maven".to_string(),
        current_mirror: maven_current,
        options: vec![
            MirrorOption {
                name: "阿里云 Maven 仓库 (aliyun)".to_string(),
                url: "https://maven.aliyun.com/repository/public".to_string(),
                ping: None,
                is_default: Some(true),
            },
            MirrorOption {
                name: "华为云 Maven 镜像".to_string(),
                url: "https://repo.huaweicloud.com/repository/maven/".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "腾讯云 Maven 镜像".to_string(),
                url: "https://mirrors.cloud.tencent.com/nexus/repository/maven-public/".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "Apache 官方中央仓库".to_string(),
                url: "https://repo.maven.apache.org/maven2".to_string(),
                ping: None,
                is_default: None,
            },
        ],
    });

    // 8. Composer (PHP)
    configs.push(MirrorConfig {
        id: "composer".to_string(),
        name: "Composer (PHP Packagist)".to_string(),
        tool: "composer".to_string(),
        current_mirror: "https://mirrors.aliyun.com/composer/".to_string(),
        options: vec![
            MirrorOption {
                name: "阿里云 Composer 镜像".to_string(),
                url: "https://mirrors.aliyun.com/composer/".to_string(),
                ping: None,
                is_default: Some(true),
            },
            MirrorOption {
                name: "腾讯云 Composer 镜像".to_string(),
                url: "https://mirrors.cloud.tencent.com/composer/".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "华为云 Composer 镜像".to_string(),
                url: "https://repo.huaweicloud.com/repository/php/".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "官方 Packagist 源".to_string(),
                url: "https://repo.packagist.org".to_string(),
                ping: None,
                is_default: None,
            },
        ],
    });

    // 9. RubyGems (Ruby)
    configs.push(MirrorConfig {
        id: "rubygems".to_string(),
        name: "RubyGems (Ruby)".to_string(),
        tool: "rubygems".to_string(),
        current_mirror: "https://gems.ruby-china.com".to_string(),
        options: vec![
            MirrorOption {
                name: "Ruby China 镜像".to_string(),
                url: "https://gems.ruby-china.com".to_string(),
                ping: None,
                is_default: None,
            },
            MirrorOption {
                name: "清华大学 RubyGems 镜像".to_string(),
                url: "https://mirrors.tuna.tsinghua.edu.cn/rubygems/".to_string(),
                ping: None,
                is_default: Some(true),
            },
            MirrorOption {
                name: "官方 rubygems.org".to_string(),
                url: "https://rubygems.org".to_string(),
                ping: None,
                is_default: None,
            },
        ],
    });

    if let Some(home) = dirs::home_dir() {
        for config in &mut configs {
            if config.tool == "brew" {
                config.current_mirror = crate::config_file::read(&brew_path(&home))?
                    .lines()
                    .find_map(|l| {
                        l.split_once('=')
                            .filter(|(k, _)| k.trim() == "HOMEBREW_BOTTLE_DOMAIN")
                            .map(|(_, v)| v.trim().to_string())
                    })
                    .unwrap_or_else(|| "https://ghcr.io/v2/homebrew/core".into());
            } else if config.tool == "pip" {
                let text = crate::config_file::read(&pip_path(&home))?;
                if let Some(value) = text.lines().find_map(|l| {
                    l.split_once('=')
                        .filter(|(k, _)| k.trim() == "index-url")
                        .map(|(_, v)| v.trim().to_string())
                }) {
                    config.current_mirror = value;
                }
            } else if config.tool == "cargo" {
                let text = crate::config_file::read(&cargo_path(&home))?;
                let doc = text
                    .parse::<toml_edit::DocumentMut>()
                    .map_err(|e| e.to_string())?;
                let source = doc.get("source");
                let replacement = source
                    .and_then(|v| v.get("crates-io"))
                    .and_then(|v| v.get("replace-with"))
                    .and_then(|v| v.as_str());
                let registry = replacement
                    .and_then(|name| source.and_then(|v| v.get(name)))
                    .and_then(|v| v.get("registry"))
                    .and_then(|v| v.as_str());
                config.current_mirror = match registry {
                    Some("sparse+https://rsproxy.cn/index/") => "https://rsproxy.cn".into(),
                    Some(url) => url.into(),
                    None if replacement.is_none() => {
                        "https://github.com/rust-lang/crates.io-index".into()
                    }
                    None => "Unknown Cargo source replacement".into(),
                };
            } else if config.tool == "composer" {
                config.current_mirror = composer_path(&home)
                    .and_then(|p| crate::config_file::read(&p))
                    .and_then(|s| composer_mirror(&s))
                    .unwrap_or_else(|e| format!("配置读取失败: {e}"));
            } else if config.tool == "rubygems" {
                config.current_mirror = ruby_sources(&ruby_config_paths(&home))
                    .map(|sources| sources.join(", "))
                    .unwrap_or_else(|e| format!("配置读取失败: {e}"));
            } else if config.tool == "nuget" {
                config.current_mirror = "Unknown (verify with the package manager)".into();
            }
        }
    }
    Ok(configs)
}

fn cargo_path(home: &std::path::Path) -> std::path::PathBuf {
    let dir = std::env::var_os("CARGO_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| home.join(".cargo"));
    if dir.join("config").exists() {
        dir.join("config")
    } else {
        dir.join("config.toml")
    }
}

fn pip_path(home: &std::path::Path) -> std::path::PathBuf {
    if let Some(path) = std::env::var_os("PIP_CONFIG_FILE") {
        return path.into();
    }
    if cfg!(windows) {
        dirs::config_dir()
            .unwrap_or_else(|| home.join("AppData/Roaming"))
            .join("pip/pip.ini")
    } else if cfg!(target_os = "macos") && home.join("Library/Application Support/pip").is_dir() {
        home.join("Library/Application Support/pip/pip.conf")
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| home.join(".config"))
            .join("pip/pip.conf")
    }
}

fn brew_path(home: &std::path::Path) -> std::path::PathBuf {
    home.join(".homebrew/brew.env")
}

fn composer_path(home: &Path) -> Result<PathBuf, String> {
    if let Some(dir) = std::env::var_os("COMPOSER_HOME").filter(|s| !s.is_empty()) {
        return Ok(PathBuf::from(dir).join("config.json"));
    }
    if cfg!(windows) {
        let dir = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(dirs::config_dir)
            .ok_or("无法定位 Composer 配置，请设置 COMPOSER_HOME")?;
        return Ok(dir.join("Composer/config.json"));
    }
    let xdg = std::env::vars_os().any(|(k, _)| k.to_string_lossy().starts_with("XDG_"))
        || Path::new("/etc/xdg").is_dir();
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    Ok(composer_unix_home(home, &config, xdg).join("config.json"))
}

fn composer_unix_home(home: &Path, xdg_config: &Path, use_xdg: bool) -> PathBuf {
    let legacy = home.join(".composer");
    let xdg = xdg_config.join("composer");
    if !use_xdg || (!xdg.is_dir() && legacy.is_dir()) {
        legacy
    } else {
        xdg
    }
}

fn composer_config(content: &str) -> Result<serde_json::Value, String> {
    let value = if content.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(content).map_err(|e| format!("Composer config.json 无效: {e}"))?
    };
    if !value.is_object() {
        return Err("Composer config.json 必须是 JSON 对象".into());
    }
    Ok(value)
}

fn composer_mirror(content: &str) -> Result<String, String> {
    let config = composer_config(content)?;
    let repos = config.get("repositories");
    let entry = repos.and_then(|r| r.get("packagist").or_else(|| r.get("packagist.org")));
    match entry {
        Some(serde_json::Value::Bool(false)) => Ok("Packagist 已禁用".into()),
        Some(v) => v
            .get("url")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| "无法识别 Packagist 配置".into()),
        None if repos.is_some_and(|v| v.is_array()) => {
            Ok("自定义仓库列表（请用 Composer 检查）".into())
        }
        None => Ok("https://repo.packagist.org".into()),
    }
}

fn update_composer(content: &str, url: &str) -> Result<String, String> {
    let mut config = composer_config(content)?;
    if config.get("repositories").is_none() {
        config["repositories"] = serde_json::json!({});
    }
    let repos = config["repositories"].as_object_mut().ok_or(
        "Composer repositories 不是命名对象；请用 Composer 管理现有仓库列表，以免改变优先级",
    )?;
    repos.remove("packagist.org");
    repos.insert(
        "packagist".into(),
        serde_json::json!({"type": "composer", "url": url}),
    );
    serde_json::to_string_pretty(&config)
        .map(|s| s + "\n")
        .map_err(|e| e.to_string())
}

fn ruby_config_paths(home: &Path) -> Vec<PathBuf> {
    let legacy = home.join(".gemrc");
    let xdg = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"))
        .join("gem/gemrc");
    let mut paths = vec![if !legacy.exists() && xdg.is_file() {
        xdg
    } else {
        legacy
    }];
    if let Some(extra) = std::env::var_os("GEMRC") {
        paths.extend(std::env::split_paths(&extra).filter(|p| !p.as_os_str().is_empty()));
    }
    paths
}

fn ruby_sources_in(content: &str) -> Result<Option<Vec<String>>, String> {
    if content.trim().is_empty() {
        return Ok(None);
    }
    let doc: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(content).map_err(|e| format!("RubyGems 配置 YAML 无效: {e}"))?;
    let map = doc.as_mapping().ok_or("RubyGems 配置必须是 YAML 映射")?;
    let source = map
        .get(serde_yaml_ng::Value::String(":sources".into()))
        .or_else(|| map.get(serde_yaml_ng::Value::String("sources".into())));
    source
        .map(|v| {
            v.as_sequence()
                .ok_or("RubyGems sources 必须是列表")?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| "RubyGems 源必须是字符串".to_string())
                })
                .collect()
        })
        .transpose()
}

fn ruby_sources(paths: &[PathBuf]) -> Result<Vec<String>, String> {
    let mut sources = vec!["https://rubygems.org".into()];
    for path in paths {
        if let Some(values) = ruby_sources_in(&crate::config_file::read(path)?)? {
            sources = values;
        }
    }
    Ok(sources)
}

fn is_public_ruby_source(url: &str) -> bool {
    matches!(
        url.trim_end_matches('/'),
        "https://rubygems.org"
            | "http://rubygems.org"
            | "https://gems.ruby-china.com"
            | "https://gems.ruby-china.org"
            | "https://mirrors.tuna.tsinghua.edu.cn/rubygems"
    )
}

fn update_ruby(content: &str, existing: &[String], url: &str) -> Result<String, String> {
    ruby_sources_in(content)?;
    let sources: Vec<_> = std::iter::once(url.to_string())
        .chain(
            existing
                .iter()
                .filter(|s| {
                    !is_public_ruby_source(s)
                        && s.trim_end_matches('/') != url.trim_end_matches('/')
                })
                .cloned(),
        )
        .collect();
    // Edit only the top-level source block: a YAML reserialization can turn Ruby
    // symbol keys into quoted strings, changing unrelated gemrc semantics.
    let key = regex::Regex::new(r#"^(?::sources|sources|\"sources\"|'sources')\s*:"#).unwrap();
    let mut output = String::new();
    let mut skipping = false;
    let mut found = false;
    for line in content.lines() {
        if key.is_match(line) {
            skipping = true;
            found = true;
            continue;
        }
        let top_level = !line.starts_with(char::is_whitespace)
            && !line.trim().is_empty()
            && !line.starts_with('#')
            && !line.starts_with("- ")
            && line != "-";
        if skipping && top_level {
            skipping = false;
        }
        if (!skipping || line.trim().starts_with('#') || line.trim().is_empty())
            && line.trim() != "..."
        {
            output.push_str(line);
            output.push('\n');
        }
    }
    if ruby_sources_in(content)?.is_some() && !found {
        return Err("无法安全编辑此 RubyGems YAML 布局，请将 sources 改为顶层块格式".into());
    }
    output.push_str(":sources:\n");
    for source in &sources {
        output.push_str(&format!(
            "- {}\n",
            serde_json::to_string(&source).map_err(|e| e.to_string())?
        ));
    }
    if ruby_sources_in(&output)? != Some(sources) {
        return Err("RubyGems sources 写入校验失败，未修改原配置".into());
    }
    let other_settings = |text: &str| -> Result<serde_yaml_ng::Value, String> {
        let mut value = if text.trim().is_empty() {
            serde_yaml_ng::Value::Mapping(Default::default())
        } else {
            serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?
        };
        if let Some(map) = value.as_mapping_mut() {
            map.remove(serde_yaml_ng::Value::String(":sources".into()));
            map.remove(serde_yaml_ng::Value::String("sources".into()));
        }
        Ok(value)
    };
    if other_settings(content)? != other_settings(&output)? {
        return Err("无法保留此 RubyGems YAML 布局的其他设置，未修改原配置".into());
    }
    Ok(output)
}

fn replace_setting(content: &str, section: Option<&str>, key: &str, value: &str) -> String {
    let mut lines = Vec::new();
    let mut inside = section.is_none();
    let mut found_section = section.is_none();
    let mut written = false;
    for line in content.lines() {
        let trim = line.trim();
        if trim.starts_with('[') && trim.ends_with(']') {
            if inside && !written {
                lines.push(format!("{key} = {value}"));
                written = true;
            }
            inside = section == Some(trim.trim_matches(['[', ']']));
            found_section |= inside;
        }
        if inside
            && trim
                .split_once('=')
                .map(|(k, _)| k.trim() == key)
                .unwrap_or(false)
        {
            if !written {
                lines.push(format!("{key} = {value}"));
                written = true;
            }
        } else {
            lines.push(line.to_string());
        }
    }
    if !written {
        if !found_section {
            lines.push(format!("[{}]", section.unwrap_or_default()));
        }
        lines.push(format!("{key} = {value}"));
    }
    lines.join("\n") + "\n"
}

fn update_cargo(doc: &mut toml_edit::DocumentMut, url: &str) -> Result<(), String> {
    if !doc.contains_key("source") {
        doc["source"] = toml_edit::Item::Table(toml_edit::Table::new());
    }
    if doc["source"].as_table_like().is_none() {
        return Err("Invalid Cargo source table".into());
    }
    if url == "https://github.com/rust-lang/crates.io-index" {
        if let Some(source) = doc["source"]
            .get_mut("crates-io")
            .and_then(|v| v.as_table_like_mut())
        {
            source.remove("replace-with");
        }
    } else {
        let registry = match url {
            "https://rsproxy.cn" => "sparse+https://rsproxy.cn/index/".to_string(),
            "https://mirrors.ustc.edu.cn/crates.io-index" => {
                "https://mirrors.ustc.edu.cn/crates.io-index".to_string()
            }
            "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git" => url.to_string(),
            _ => return Err("Unsupported Cargo mirror".into()),
        };
        doc["source"]["crates-io"]["replace-with"] = toml_edit::value("envhub-mirror");
        // A fresh owned table prevents a stale replace-with chain from forming a cycle.
        let mut table = toml_edit::Table::new();
        table["registry"] = toml_edit::value(registry);
        doc["source"]["envhub-mirror"] = toml_edit::Item::Table(table);
    }
    Ok(())
}

fn update_maven(content: &str, url: &str) -> Result<String, String> {
    use xmltree::{Element, XMLNode};
    let mut root = if content.trim().is_empty() {
        Element::new("settings")
    } else {
        Element::parse(content.as_bytes()).map_err(|e| e.to_string())?
    };
    if root.name != "settings" {
        return Err("Invalid Maven settings root".into());
    }
    if root.get_child("mirrors").is_none() {
        root.children
            .push(XMLNode::Element(Element::new("mirrors")));
    }
    let mirrors = root.get_mut_child("mirrors").ok_or("Missing mirrors")?;
    mirrors.children.retain(|n| {
        !n.as_element()
            .map(|m| {
                m.get_child("id").and_then(Element::get_text).as_deref() == Some("envhub-mirror")
            })
            .unwrap_or(false)
    });
    let mut mirror = Element::new("mirror");
    for (name, text) in [
        ("id", "envhub-mirror"),
        ("mirrorOf", "central"),
        ("url", url),
    ] {
        let mut child = Element::new(name);
        child.children.push(XMLNode::Text(text.into()));
        mirror.children.push(XMLNode::Element(child));
    }
    mirrors.children.insert(0, XMLNode::Element(mirror));
    let mut bytes = Vec::new();
    root.write_with_config(
        &mut bytes,
        xmltree::EmitterConfig::new().perform_indent(true),
    )
    .map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_mirror(tool: String, mirror_url: String) -> Result<bool, String> {
    if !mirror_url.starts_with("https://")
        || mirror_url
            .chars()
            .any(|c| c.is_control() || c == '\'' || c == '"')
    {
        return Err("Mirror must be a valid HTTPS URL".into());
    }
    let home = dirs::home_dir().ok_or("Cannot locate home directory")?;
    let command: Option<(&str, Vec<String>)> = match tool.as_str() {
        "go" => Some((
            "go",
            vec!["env".into(), "-w".into(), format!("GOPROXY={mirror_url}")],
        )),
        "scoop" => Some((
            "powershell",
            vec![
                "-NoProfile".into(),
                "-Command".into(),
                format!("scoop config SCOOP_REPO '{mirror_url}'; if (-not $?) {{ exit 1 }}"),
            ],
        )),
        "nuget" => Some((
            "dotnet",
            vec![
                "nuget".into(),
                "update".into(),
                "source".into(),
                "EnvHub-Mirror".into(),
                "--source".into(),
                mirror_url.clone(),
            ],
        )),
        _ => None,
    };
    if let Some((program, args)) = command {
        let mut output = env_helper::output_timeout(
            env_helper::create_silent_tokio_command(program)
                .current_dir(&home)
                .args(&args),
            20,
        )
        .await?;
        if tool == "nuget" && !output.status.success() {
            output = env_helper::output_timeout(
                env_helper::create_silent_tokio_command("dotnet").args([
                    "nuget",
                    "add",
                    "source",
                    &mirror_url,
                    "--name",
                    "EnvHub-Mirror",
                ]),
                20,
            )
            .await?;
        }
        env_helper::checked_output(output)?;
        return Ok(true);
    }
    if tool == "cargo" {
        crate::config_file::edit_toml(&cargo_path(&home), |doc| update_cargo(doc, &mirror_url))?;
        return Ok(true);
    }
    let _guard = crate::config_file::CONFIG_LOCK
        .lock()
        .map_err(|e| e.to_string())?;
    let (path, content) = match tool.as_str() {
        "composer" => {
            let path = composer_path(&home)?;
            let content = update_composer(&crate::config_file::read(&path)?, &mirror_url)?;
            (path, content)
        }
        "rubygems" => {
            let paths = ruby_config_paths(&home);
            let sources = ruby_sources(&paths)?;
            let path = paths.last().ok_or("无法定位 RubyGems 配置")?.clone();
            let content = update_ruby(&crate::config_file::read(&path)?, &sources, &mirror_url)?;
            (path, content)
        }
        "npm" => {
            let path = home.join(".npmrc");
            let content = replace_setting(
                &crate::config_file::read(&path)?,
                None,
                "registry",
                &mirror_url,
            );
            (path, content)
        }
        "pip" => {
            let path = pip_path(&home);
            let content = replace_setting(
                &crate::config_file::read(&path)?,
                Some("global"),
                "index-url",
                &mirror_url,
            );
            (path, content)
        }
        "brew" => {
            let path = brew_path(&home);
            let content = replace_setting(
                &crate::config_file::read(&path)?,
                None,
                "HOMEBREW_BOTTLE_DOMAIN",
                &mirror_url,
            )
            .replace("HOMEBREW_BOTTLE_DOMAIN = ", "HOMEBREW_BOTTLE_DOMAIN=");
            (path, content)
        }
        "docker" => {
            if cfg!(target_os = "linux") {
                return Err("Linux Docker daemon configuration requires an administrator: edit /etc/docker/daemon.json and restart Docker".into());
            }
            let path = home.join(".docker/daemon.json");
            if !path.is_file() {
                return Err("Open Docker Desktop > Settings > Docker Engine to configure registry-mirrors; no existing daemon.json was found".into());
            }
            let old = crate::config_file::read(&path)?;
            let mut value: serde_json::Value = if old.trim().is_empty() {
                serde_json::json!({})
            } else {
                serde_json::from_str(&old).map_err(|e| e.to_string())?
            };
            if !value.is_object() {
                return Err("Docker configuration is not an object".into());
            }
            value["registry-mirrors"] = if mirror_url == "https://registry-1.docker.io" {
                serde_json::json!([])
            } else {
                serde_json::json!([mirror_url])
            };
            (
                path,
                serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?,
            )
        }
        "maven" => {
            let path = home.join(".m2/settings.xml");
            let content = update_maven(&crate::config_file::read(&path)?, &mirror_url)?;
            (path, content)
        }
        _ => return Err(format!("Unsupported mirror: {tool}")),
    };
    crate::config_file::write(&path, &content)?;
    Ok(true)
}

/// Helper to parse host and port from URL for real TCP connect measurement
fn parse_host_port(url_str: &str) -> (String, u16) {
    let clean = url_str
        .split(',')
        .next()
        .unwrap_or(url_str)
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("sparse+")
        .trim_start_matches("git://");

    let host_part = clean.split('/').next().unwrap_or(clean);
    let port = if url_str.starts_with("http://") || url_str.starts_with("git://") {
        80
    } else {
        443
    };

    (host_part.to_string(), port)
}

#[tauri::command]
pub async fn ping_mirrors() -> Result<HashMap<String, u32>, String> {
    let urls = [
        "https://registry.npmmirror.com",
        "https://mirrors.cloud.tencent.com/npm/",
        "https://repo.huaweicloud.com/repository/npm/",
        "https://registry.npmjs.org",
        "https://pypi.tuna.tsinghua.edu.cn/simple",
        "https://mirrors.aliyun.com/pypi/simple/",
        "https://pypi.doubanio.com/simple/",
        "https://pypi.org/simple",
        "https://goproxy.cn,direct",
        "https://mirrors.aliyun.com/goproxy/,direct",
        "https://proxy.golang.org,direct",
        "https://rsproxy.cn",
        "https://mirrors.ustc.edu.cn/crates.io-index",
        "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git",
        "https://github.com/rust-lang/crates.io-index",
        "https://docker.m.daocloud.io",
        "https://docker.mirrors.ustc.edu.cn",
        "https://mirror.ccs.tencentyun.com",
        "https://registry-1.docker.io",
        "https://mirrors.ustc.edu.cn/homebrew-bottles",
        "https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles",
        "https://mirrors.aliyun.com/homebrew/homebrew-bottles",
        "https://maven.aliyun.com/repository/public",
        "https://repo.huaweicloud.com/repository/maven/",
        "https://mirrors.cloud.tencent.com/nexus/repository/maven-public/",
        "https://repo.maven.apache.org/maven2",
        "https://mirrors.aliyun.com/composer/",
        "https://mirrors.cloud.tencent.com/composer/",
        "https://repo.huaweicloud.com/repository/php/",
        "https://repo.packagist.org",
        "https://gems.ruby-china.com",
        "https://mirrors.tuna.tsinghua.edu.cn/rubygems/",
        "https://rubygems.org",
        "https://github.com/ScoopInstaller/Scoop",
        "https://gitee.com/scoop-installer/scoop",
        "https://nuget.cdn.azure.cn/v3/index.json",
        "https://repo.huaweicloud.com/repository/nuget/v3/index.json",
        "https://api.nuget.org/v3/index.json",
    ];

    let mut results = HashMap::new();

    let mut tasks = tokio::task::JoinSet::new();
    let limit = std::sync::Arc::new(tokio::sync::Semaphore::new(8));
    for url in urls {
        let limit = limit.clone();
        tasks.spawn(async move {
            let _permit = limit.acquire().await.unwrap();
            let (host, port) = parse_host_port(url);
            let addr = format!("{}:{}", host, port);

            let start = Instant::now();
            let ping_res = timeout(Duration::from_millis(2500), TcpStream::connect(&addr)).await;

            match ping_res {
                Ok(Ok(_stream)) => {
                    let ms = start.elapsed().as_millis().max(1) as u32;
                    (url.to_string(), ms)
                }
                _ => (url.to_string(), 999),
            }
        });
    }

    while let Some(result) = tasks.join_next().await {
        let (url, ping) = result.map_err(|e| e.to_string())?;
        results.insert(url, ping);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn composer_mirror_works_without_executable_and_preserves_settings() {
        let old = r#"{"config":{"secure-http":true,"github-oauth":{"github.com":"secret"}},"repositories":{"private":{"type":"vcs","url":"https://private.example/repo"},"packagist.org":false}}"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("composer/config.json");
        let url = "https://mirrors.aliyun.com/composer/";
        crate::config_file::write(&path, &update_composer(old, url).unwrap()).unwrap();
        let saved = crate::config_file::read(&path).unwrap();
        let doc = composer_config(&saved).unwrap();
        assert_eq!(doc["config"]["secure-http"], true);
        assert_eq!(doc["config"]["github-oauth"]["github.com"], "secret");
        assert!(doc["repositories"].get("private").is_some());
        assert!(doc["repositories"].get("packagist.org").is_none());
        assert_eq!(composer_mirror(&saved).unwrap(), url);
        assert_eq!(composer_mirror("").unwrap(), "https://repo.packagist.org");
        assert!(update_composer("[]", url).is_err());
        assert!(update_composer("{invalid", url).is_err());
    }
    #[test]
    fn composer_home_follows_xdg_and_legacy_precedence() {
        let home = tempfile::tempdir().unwrap();
        let xdg = home.path().join("xdg");
        assert_eq!(
            composer_unix_home(home.path(), &xdg, false),
            home.path().join(".composer")
        );
        assert_eq!(
            composer_unix_home(home.path(), &xdg, true),
            xdg.join("composer")
        );
        fs::create_dir(home.path().join(".composer")).unwrap();
        assert_eq!(
            composer_unix_home(home.path(), &xdg, true),
            home.path().join(".composer")
        );
        fs::create_dir_all(xdg.join("composer")).unwrap();
        assert_eq!(
            composer_unix_home(home.path(), &xdg, true),
            xdg.join("composer")
        );
    }
    #[test]
    fn composer_preserves_private_repository_priority() {
        let result = update_composer(r#"{"repositories":{"z-first":{"type":"vcs","url":"https://first.example"},"a-second":{"type":"vcs","url":"https://second.example"}}}"#, "https://mirrors.aliyun.com/composer/").unwrap();
        let config = composer_config(&result).unwrap();
        let keys: Vec<_> = config["repositories"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["z-first", "a-second", "packagist"]);
    }
    #[test]
    fn ruby_source_switch_preserves_private_sources_and_tls_settings() {
        let old = "---\n# user settings\n:sources:\n- https://rubygems.org/\n- https://mirrors.tuna.tsinghua.edu.cn/rubygems/\n- https://private.example/gems\n:ssl_verify_mode: 1\n:ssl_ca_cert: /custom/ca.pem\ngem: --no-document\n";
        let url = "https://gems.ruby-china.com";
        let sources = ruby_sources_in(old).unwrap().unwrap();
        let updated = update_ruby(old, &sources, url).unwrap();
        assert_eq!(
            ruby_sources_in(&updated).unwrap().unwrap(),
            [url, "https://private.example/gems"]
        );
        assert!(updated
            .contains(":ssl_verify_mode: 1\n:ssl_ca_cert: /custom/ca.pem\ngem: --no-document"));
        assert!(updated.contains("# user settings"));
        assert!(!updated.contains("tuna.tsinghua"));
        assert_eq!(
            update_ruby(&updated, &ruby_sources_in(&updated).unwrap().unwrap(), url).unwrap(),
            updated
        );
    }
    #[test]
    fn ruby_reads_last_config_override_and_rejects_invalid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join(".gemrc");
        let extra = dir.path().join("extra.gemrc");
        fs::write(&base, ":sources:\n- https://rubygems.org\n").unwrap();
        fs::write(
            &extra,
            "sources: [https://private.example]\n:verbose: true\n",
        )
        .unwrap();
        let paths = vec![base, extra.clone()];
        let sources = ruby_sources(&paths).unwrap();
        assert_eq!(sources, ["https://private.example"]);
        let updated = update_ruby(
            &fs::read_to_string(&extra).unwrap(),
            &sources,
            "https://gems.ruby-china.com",
        )
        .unwrap();
        crate::config_file::write(&extra, &updated).unwrap();
        assert_eq!(
            ruby_sources(&paths).unwrap(),
            ["https://gems.ruby-china.com", "https://private.example"]
        );
        for invalid in [
            "[",
            ":sources: nope",
            "- not-a-map",
            "{sources: [https://old.example]}",
        ] {
            assert!(update_ruby(invalid, &[], "https://gems.ruby-china.com").is_err());
        }
    }
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn ruby_gems_accepts_edited_symbol_keys_without_network() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.gemrc");
        let old = ":ssl_verify_mode: 1\ngem: --no-document\n";
        fs::write(
            &path,
            update_ruby(old, &[], "https://gems.ruby-china.com").unwrap(),
        )
        .unwrap();
        let output = env_helper::checked_output(env_helper::output_timeout(
            env_helper::create_silent_tokio_command("/usr/bin/ruby").args(["-rrubygems", "-rjson", "-e", "c = Gem::ConfigFile.new(['--config-file', ARGV[0]]); puts JSON.generate([c.sources, c.ssl_verify_mode, c['gem']])"])
                .arg(&path).env_remove("GEMRC").current_dir(dir.path()), 5).await.unwrap()).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            parsed,
            serde_json::json!([["https://gems.ruby-china.com"], 1, "--no-document"])
        );
    }
    #[test]
    fn cargo_switch_preserves_settings_and_official_has_no_cycle() {
        let mut doc = "[build]\njobs = 3\n[alias]\nb = 'build'\n".parse().unwrap();
        update_cargo(&mut doc, "https://rsproxy.cn").unwrap();
        update_cargo(&mut doc, "https://github.com/rust-lang/crates.io-index").unwrap();
        assert_eq!(doc["build"]["jobs"].as_integer(), Some(3));
        assert_eq!(doc["alias"]["b"].as_str(), Some("build"));
        assert!(doc["source"]["crates-io"].get("replace-with").is_none());
    }
    #[test]
    fn ini_preserves_other_settings_and_sections() {
        let text = "[global]\ntimeout = 60\nindex-url = old\n[install]\nuser = true\n";
        let result = replace_setting(text, Some("global"), "index-url", "new");
        assert!(result.contains("timeout = 60"));
        assert!(result.contains("[install]\nuser = true"));
        assert!(!result.contains("old"));
        assert_eq!(result.matches("index-url").count(), 1);
    }
    #[test]
    fn maven_preserves_credentials() {
        let text = "<settings><servers><server><id>private</id><password>secret</password></server></servers></settings>";
        let text = update_maven(text, "https://one.example").unwrap();
        let text = update_maven(&text, "https://two.example").unwrap();
        assert!(text.contains("secret"));
        assert!(!text.contains("one.example"));
        assert_eq!(text.matches("<mirror>").count(), 1);
    }
}
