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

    // 5. Homebrew (macOS)
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
            
            // Extract host for trusted-host
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
        _ => {
            return Err("暂不支持该工具的镜像自动写入，功能正在开发中".to_string());
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
        "https://mirrors.ustc.edu.cn/homebrew-bottles",
        "https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles",
        "https://mirrors.aliyun.com/homebrew/homebrew-bottles",
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
                // If unreachable or timeout, record 999 ms
                results.insert(url.to_string(), 999);
            }
        }
    }

    Ok(results)
}
