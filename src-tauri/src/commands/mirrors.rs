use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
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
    let mut configs = Vec::new();

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
        if let Ok(out) = std::process::Command::new("npm").args(["config", "get", "registry"]).output() {
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
            MirrorOption { name: "淘宝 NPM 镜像 (npmmirror)".to_string(), url: "https://registry.npmmirror.com".to_string(), ping: None, is_default: Some(true) },
            MirrorOption { name: "腾讯云 NPM 镜像".to_string(), url: "https://mirrors.cloud.tencent.com/npm/".to_string(), ping: None, is_default: None },
            MirrorOption { name: "华为云 NPM 镜像".to_string(), url: "https://repo.huaweicloud.com/repository/npm/".to_string(), ping: None, is_default: None },
            MirrorOption { name: "官方源 (npmjs.org)".to_string(), url: "https://registry.npmjs.org".to_string(), ping: None, is_default: None },
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
            MirrorOption { name: "清华大学 TUNA 镜像".to_string(), url: "https://pypi.tuna.tsinghua.edu.cn/simple".to_string(), ping: None, is_default: Some(true) },
            MirrorOption { name: "阿里云开源镜像".to_string(), url: "https://mirrors.aliyun.com/pypi/simple/".to_string(), ping: None, is_default: None },
            MirrorOption { name: "豆瓣开源镜像".to_string(), url: "https://pypi.doubanio.com/simple/".to_string(), ping: None, is_default: None },
            MirrorOption { name: "官方 PyPI 源".to_string(), url: "https://pypi.org/simple".to_string(), ping: None, is_default: None },
        ],
    });

    // 3. Go: Query `go env GOPROXY`
    let mut go_current = "https://proxy.golang.org,direct".to_string();
    if let Ok(out) = std::process::Command::new("go").args(["env", "GOPROXY"]).output() {
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
            MirrorOption { name: "Goproxy 中国 (七牛云)".to_string(), url: "https://goproxy.cn,direct".to_string(), ping: None, is_default: Some(true) },
            MirrorOption { name: "阿里云 Go 模块代理".to_string(), url: "https://mirrors.aliyun.com/goproxy/,direct".to_string(), ping: None, is_default: None },
            MirrorOption { name: "官方 proxy.golang.org".to_string(), url: "https://proxy.golang.org,direct".to_string(), ping: None, is_default: None },
        ],
    });

    // 4. Cargo: Read ~/.cargo/config.toml
    let mut cargo_current = "https://github.com/rust-lang/crates.io-index".to_string();
    if let Some(home) = dirs::home_dir() {
        let cargo_paths = [
            home.join(".cargo/config.toml"),
            home.join(".cargo/config"),
        ];
        for path in cargo_paths {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("rsproxy") {
                        cargo_current = "https://rsproxy.cn".to_string();
                    } else if content.contains("ustc") {
                        cargo_current = "https://mirrors.ustc.edu.cn/crates.io-index".to_string();
                    } else if content.contains("tuna") {
                        cargo_current = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git".to_string();
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
            MirrorOption { name: "字节跳动 rsproxy (推荐)".to_string(), url: "https://rsproxy.cn".to_string(), ping: None, is_default: Some(true) },
            MirrorOption { name: "中国科学技术大学 USTC".to_string(), url: "https://mirrors.ustc.edu.cn/crates.io-index".to_string(), ping: None, is_default: None },
            MirrorOption { name: "清华大学 Crates 镜像".to_string(), url: "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git".to_string(), ping: None, is_default: None },
            MirrorOption { name: "官方 crates.io".to_string(), url: "https://github.com/rust-lang/crates.io-index".to_string(), ping: None, is_default: None },
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
            MirrorOption { name: "DaoCloud 镜像加速".to_string(), url: "https://docker.m.daocloud.io".to_string(), ping: None, is_default: Some(true) },
            MirrorOption { name: "中科大 Docker 镜像源".to_string(), url: "https://docker.mirrors.ustc.edu.cn".to_string(), ping: None, is_default: None },
            MirrorOption { name: "腾讯云容器镜像代理".to_string(), url: "https://mirror.ccs.tencentyun.com".to_string(), ping: None, is_default: None },
            MirrorOption { name: "Docker 官方 Docker Hub".to_string(), url: "https://registry-1.docker.io".to_string(), ping: None, is_default: None },
        ],
    });

    // 6. Homebrew (macOS)
    configs.push(MirrorConfig {
        id: "brew".to_string(),
        name: "Homebrew (macOS)".to_string(),
        tool: "brew".to_string(),
        current_mirror: "https://mirrors.ustc.edu.cn/homebrew-bottles".to_string(),
        options: vec![
            MirrorOption { name: "中国科学技术大学 USTC 镜像".to_string(), url: "https://mirrors.ustc.edu.cn/homebrew-bottles".to_string(), ping: None, is_default: Some(true) },
            MirrorOption { name: "清华大学 TUNA 镜像".to_string(), url: "https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles".to_string(), ping: None, is_default: None },
            MirrorOption { name: "阿里云 Homebrew 镜像".to_string(), url: "https://mirrors.aliyun.com/homebrew/homebrew-bottles".to_string(), ping: None, is_default: None },
        ],
    });

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
            MirrorOption { name: "阿里云 Maven 仓库 (aliyun)".to_string(), url: "https://maven.aliyun.com/repository/public".to_string(), ping: None, is_default: Some(true) },
            MirrorOption { name: "华为云 Maven 镜像".to_string(), url: "https://repo.huaweicloud.com/repository/maven/".to_string(), ping: None, is_default: None },
            MirrorOption { name: "腾讯云 Maven 镜像".to_string(), url: "https://mirrors.cloud.tencent.com/nexus/repository/maven-public/".to_string(), ping: None, is_default: None },
            MirrorOption { name: "Apache 官方中央仓库".to_string(), url: "https://repo.maven.apache.org/maven2".to_string(), ping: None, is_default: None },
        ],
    });

    // 8. Composer (PHP)
    configs.push(MirrorConfig {
        id: "composer".to_string(),
        name: "Composer (PHP Packagist)".to_string(),
        tool: "composer".to_string(),
        current_mirror: "https://mirrors.aliyun.com/composer/".to_string(),
        options: vec![
            MirrorOption { name: "阿里云 Composer 镜像".to_string(), url: "https://mirrors.aliyun.com/composer/".to_string(), ping: None, is_default: Some(true) },
            MirrorOption { name: "腾讯云 Composer 镜像".to_string(), url: "https://mirrors.cloud.tencent.com/composer/".to_string(), ping: None, is_default: None },
            MirrorOption { name: "华为云 Composer 镜像".to_string(), url: "https://repo.huaweicloud.com/repository/php/".to_string(), ping: None, is_default: None },
            MirrorOption { name: "官方 Packagist 源".to_string(), url: "https://repo.packagist.org".to_string(), ping: None, is_default: None },
        ],
    });

    // 9. RubyGems (Ruby)
    configs.push(MirrorConfig {
        id: "rubygems".to_string(),
        name: "RubyGems (Ruby)".to_string(),
        tool: "rubygems".to_string(),
        current_mirror: "https://gems.ruby-china.com".to_string(),
        options: vec![
            MirrorOption { name: "Ruby China 镜像 (推荐)".to_string(), url: "https://gems.ruby-china.com".to_string(), ping: None, is_default: Some(true) },
            MirrorOption { name: "清华大学 RubyGems 镜像".to_string(), url: "https://mirrors.tuna.tsinghua.edu.cn/rubygems/".to_string(), ping: None, is_default: None },
            MirrorOption { name: "官方 rubygems.org".to_string(), url: "https://rubygems.org".to_string(), ping: None, is_default: None },
        ],
    });

    Ok(configs)
}

#[tauri::command]
pub async fn set_mirror(tool: String, mirror_url: String) -> Result<bool, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;

    match tool.as_str() {
        "npm" => {
            let npmrc = home.join(".npmrc");
            let mut lines = Vec::new();
            if npmrc.exists() {
                if let Ok(content) = fs::read_to_string(&npmrc) {
                    for line in content.lines() {
                        if !line.trim().starts_with("registry=") && !line.trim().starts_with("registry =") {
                            lines.push(line.to_string());
                        }
                    }
                }
            }
            lines.push(format!("registry={}", mirror_url));
            fs::write(npmrc, lines.join("\n") + "\n").map_err(|e| format!("写入 ~/.npmrc 失败: {}", e))?;
        }
        "pip" => {
            let pip_dir = home.join(".pip");
            let _ = fs::create_dir_all(&pip_dir);
            let pip_conf = pip_dir.join("pip.conf");
            
            let host = mirror_url
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .split('/')
                .next()
                .unwrap_or("pypi.tuna.tsinghua.edu.cn");

            let content = format!("[global]\nindex-url = {}\ntrusted-host = {}\n", mirror_url, host);
            fs::write(pip_conf, content).map_err(|e| format!("写入 pip.conf 失败: {}", e))?;
        }
        "go" => {
            let status = std::process::Command::new("go")
                .args(["env", "-w", &format!("GOPROXY={}", mirror_url)])
                .status()
                .map_err(|e| format!("执行 go env -w 失败: {}", e))?;
            if !status.success() {
                return Err("go env -w 执行返回非零退出码".to_string());
            }
        }
        "cargo" => {
            let cargo_dir = home.join(".cargo");
            let _ = fs::create_dir_all(&cargo_dir);
            let cargo_conf = cargo_dir.join("config.toml");
            
            let config_content = if mirror_url.contains("rsproxy") {
                r#"[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[net]
git-fetch-with-cli = true
"#
            } else if mirror_url.contains("ustc") {
                r#"[source.crates-io]
replace-with = 'ustc'
[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
"#
            } else {
                r#"[source.crates-io]
replace-with = 'crates-io'
"#
            };
            fs::write(cargo_conf, config_content).map_err(|e| format!("写入 ~/.cargo/config.toml 失败: {}", e))?;
        }
        "docker" => {
            let docker_dir = home.join(".docker");
            let _ = fs::create_dir_all(&docker_dir);
            let docker_cfg = docker_dir.join("daemon.json");
            
            let val = serde_json::json!({
                "registry-mirrors": [mirror_url]
            });
            let content = serde_json::to_string_pretty(&val).unwrap_or_default();
            fs::write(docker_cfg, content).map_err(|e| format!("写入 ~/.docker/daemon.json 失败: {}", e))?;
        }
        "maven" => {
            let m2_dir = home.join(".m2");
            let _ = fs::create_dir_all(&m2_dir);
            let settings_file = m2_dir.join("settings.xml");

            let xml_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"
          xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
          xsi:schemaLocation="http://maven.apache.org/SETTINGS/1.0.0 http://maven.apache.org/xsd/settings-1.0.0.xsd">
  <mirrors>
    <mirror>
      <id>envhub-mirror</id>
      <mirrorOf>central</mirrorOf>
      <name>EnvHub Mirror</name>
      <url>{}</url>
    </mirror>
  </mirrors>
</settings>"#, mirror_url);
            fs::write(settings_file, xml_content).map_err(|e| format!("写入 ~/.m2/settings.xml 失败: {}", e))?;
        }
        "composer" => {
            let _ = std::process::Command::new("composer")
                .args(["config", "-g", "repos.packagist", "composer", &mirror_url])
                .status();
        }
        "rubygems" => {
            let _ = std::process::Command::new("gem")
                .args(["sources", "--add", &mirror_url])
                .status();
        }
        _ => {
            return Err("该镜像源已保存".to_string());
        }
    }

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
    ];

    let mut results = HashMap::new();

    for url in urls {
        let (host, port) = parse_host_port(url);
        let addr = format!("{}:{}", host, port);

        let start = Instant::now();
        let ping_res = timeout(Duration::from_millis(2500), TcpStream::connect(&addr)).await;

        match ping_res {
            Ok(Ok(_stream)) => {
                let ms = start.elapsed().as_millis().max(1) as u32;
                results.insert(url.to_string(), ms);
            }
            _ => {
                results.insert(url.to_string(), 999);
            }
        }
    }

    Ok(results)
}
