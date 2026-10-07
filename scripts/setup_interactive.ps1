# ==============================================================================
# EnvHub - 跨平台开发环境与多语言版本交互式向导 (Windows PowerShell)
# 项目地址: https://github.com/WnJee/EnvHub
# ==============================================================================

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

function Write-EnvInfo { param([string]$msg) Write-Host "[EnvHub] $msg" -ForegroundColor Cyan }
function Write-EnvSucc { param([string]$msg) Write-Host "[✓ 成功] $msg" -ForegroundColor Green }
function Write-EnvWarn { param([string]$msg) Write-Host "[! 提示] $msg" -ForegroundColor Yellow }
function Write-EnvErr  { param([string]$msg) Write-Host "[✗ 错误] $msg" -ForegroundColor Red }
function Write-EnvStep { param([string]$msg) Write-Host "`n==> $msg" -ForegroundColor Magenta }

function Pause-Console {
    Write-Host ""
    Write-Host "按回车键继续..." -ForegroundColor DarkGray
    $null = Read-Host
}

# --- 刷新与同步当前会话 PATH ---
function Update-SessionPath {
    $MiseCandidates = @(
        "$env:LOCALAPPDATA\mise\bin",
        "$env:LOCALAPPDATA\mise\shims",
        "$env:USERPROFILE\scoop\shims",
        "$env:USERPROFILE\.local\bin"
    )
    foreach ($p in $MiseCandidates) {
        if ((Test-Path $p) -and ($env:Path -notlike "*$p*")) {
            $env:Path = "$p;$env:Path"
        }
    }
}
Update-SessionPath

# ==============================================================================
# 阶段 1: 检查并安装 Mise 版本管理器
# ==============================================================================
function Ensure-Mise {
    Write-EnvStep "步骤 1: 检查 Mise 版本管理器状态"
    
    $MiseCmd = Get-Command mise -ErrorAction SilentlyContinue
    if (-not $MiseCmd -and (Test-Path "$env:LOCALAPPDATA\mise\bin\mise.exe")) {
        $env:Path = "$env:LOCALAPPDATA\mise\bin;$env:LOCALAPPDATA\mise\shims;$env:Path"
        $MiseCmd = Get-Command "$env:LOCALAPPDATA\mise\bin\mise.exe" -ErrorAction SilentlyContinue
    }

    if ($MiseCmd) {
        $ver = (& mise --version) 2>$null
        Write-EnvSucc "检测到 Mise 已就绪: $ver"
        return
    }

    Write-EnvWarn "未在当前系统中检测到 Mise，开始自动安装..."

    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    try {
        Invoke-Expression (Invoke-RestMethod https://mise.jdx.dev/install.ps1)
    } catch {
        Write-EnvWarn "官方安装地址访问重试，尝试备用通道..."
        Invoke-Expression (Invoke-RestMethod https://mise.run)
    }

    Update-SessionPath

    # 注入 PowerShell Profile 环境变量与双内核兼容钩子
    $MiseHook = @'
# === EnvHub / Mise Activation Hook ===
$MiseBin = if (Get-Command mise -ErrorAction SilentlyContinue) { 'mise' } elseif (Test-Path "$env:LOCALAPPDATA\mise\bin\mise.exe") { "$env:LOCALAPPDATA\mise\bin\mise.exe" } else { $null }
if ($MiseBin) {
    $MiseShell = if ($PSVersionTable.PSEdition -eq 'Core') { 'pwsh' } else { 'powershell' }
    (& $MiseBin activate $MiseShell) | Out-String | Invoke-Expression
}
'@
    $ProfilePaths = @(
        $PROFILE,
        "$env:USERPROFILE\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1",
        "$env:USERPROFILE\Documents\PowerShell\Microsoft.PowerShell_profile.ps1"
    )
    foreach ($p in $ProfilePaths) {
        if ($p) {
            $pDir = Split-Path -Parent $p
            if (-not (Test-Path $pDir)) { New-Item -ItemType Directory -Path $pDir -Force | Out-Null }
            if (Test-Path $p) {
                $existing = Get-Content $p -Raw -ErrorAction SilentlyContinue
                if ($existing -notmatch "mise activate") {
                    Add-Content -Path $p -Value $MiseHook
                    Write-EnvSucc "已注入 Mise 激活钩子 -> $p"
                }
            } else {
                Set-Content -Path $p -Value $MiseHook
                Write-EnvSucc "已创建并配置 Profile -> $p"
            }
        }
    }

    # 注册 User Registry PATH
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    foreach ($p in @("$env:LOCALAPPDATA\mise\bin", "$env:LOCALAPPDATA\mise\shims")) {
        if ($UserPath -notlike "*$p*") {
            $UserPath = "$p;$UserPath"
        }
    }
    [Environment]::SetEnvironmentVariable("Path", $UserPath, "User")

    Write-EnvSucc "Mise 安装与环境变量配置完成！"
}

# ==============================================================================
# 阶段 2: 检查并询问安装包管理器 (Scoop / WinGet)
# ==============================================================================
function Ensure-PackageManager {
    Write-EnvStep "步骤 2: 检查 Windows 系统包管理器状态"

    $hasScoop = (Get-Command scoop -ErrorAction SilentlyContinue) -or (Test-Path "$env:USERPROFILE\scoop\shims\scoop.cmd")
    $hasWinget = Get-Command winget -ErrorAction SilentlyContinue

    if ($hasScoop) {
        $global:PKG_MGR = "Scoop"
        Write-EnvSucc "检测到 Windows 包管理器 (Scoop) 已就绪"
        return
    } elseif ($hasWinget) {
        $global:PKG_MGR = "WinGet"
        Write-EnvSucc "检测到 Windows 包管理器 (WinGet) 已就绪"
    } else {
        $global:PKG_MGR = "未安装"
    }

    Write-EnvWarn "未检测到 Scoop 极简包管理器（推荐用于免提权安装开发工具与数据库）。"
    Write-Host "是否现在自动安装 Scoop 包管理器？[Y/n]: " -NoNewline -ForegroundColor Yellow
    $ans = Read-Host
    if ([string]::IsNullOrWhiteSpace($ans) -or $ans -match '^[Yy]$') {
        Write-EnvInfo "正在安装 Scoop..."
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force
        Invoke-RestMethod get.scoop.sh | Invoke-Expression
        Update-SessionPath
        $global:PKG_MGR = "Scoop"
        Write-EnvSucc "Scoop 安装完成！"
    } else {
        Write-EnvInfo "已跳过 Scoop 安装。"
    }
}

# ==============================================================================
# 运行时安装辅助函数
# ==============================================================================
function Install-Runtime {
    param([string]$toolId, [string]$version, [string]$name)

    Write-EnvStep "正在安装全局运行环境: $name ($toolId@$version)"
    Write-Host ">> 执行: mise use -g $toolId@$version" -ForegroundColor Cyan

    & mise use -g "$toolId@$version"
    if ($LASTEXITCODE -eq 0) {
        & mise reshim
        Write-EnvSucc "$name ($toolId@$version) 安装并设置为全局默认成功！"
        Write-Host -NoNewline ">> 当前生效版本: " -ForegroundColor DarkGray
        switch ($toolId) {
            "node"   { node -v }
            "python" { python --version }
            "go"     { go version }
            "java"   { java -version 2>&1 | Select-Object -First 1 }
            "rust"   { rustc --version }
            Default  { & mise current $toolId }
        }
    } else {
        Write-EnvErr "$name 安装失败，请检查网络连接。"
    }
    Pause-Console
}

# --- 国内镜像加速配置 ---
function Setup-Mirrors {
    Write-EnvStep "配置国内高速开发镜像加速源"

    # NPM
    if (Get-Command npm -ErrorAction SilentlyContinue) {
        npm config set registry https://registry.npmmirror.com/ | Out-Null
        Write-EnvSucc "NPM 镜像源 -> https://registry.npmmirror.com/"
    }

    # Python Pip
    $PipDir = "$env:APPDATA\pip"
    if (-not (Test-Path $PipDir)) { New-Item -ItemType Directory -Path $PipDir -Force | Out-Null }
    @'
[global]
index-url = https://pypi.tuna.tsinghua.edu.cn/simple
trusted-host = pypi.tuna.tsinghua.edu.cn
'@ | Set-Content -Path "$PipDir\pip.ini" -Encoding UTF8
    Write-EnvSucc "Pip 镜像源 -> 清华大学开源镜像站"

    # Go Proxy
    if (Get-Command go -ErrorAction SilentlyContinue) {
        go env -w GOPROXY=https://goproxy.cn,direct | Out-Null
        Write-EnvSucc "Go Proxy -> https://goproxy.cn,direct"
    }

    # Rust Crates
    $CargoDir = "$env:USERPROFILE\.cargo"
    if (-not (Test-Path $CargoDir)) { New-Item -ItemType Directory -Path $CargoDir -Force | Out-Null }
    @'
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
'@ | Set-Content -Path "$CargoDir\config.toml" -Encoding UTF8
    Write-EnvSucc "Cargo 镜像源 -> 中科大 Crates.io 镜像"

    Write-EnvSucc "国内镜像源加速配置完成！"
    Pause-Console
}

# --- 查看运行环境列表 ---
function Show-RuntimeStatus {
    Write-EnvStep "当前已安装与激活的运行环境列表"
    & mise list
    Write-Host "`n全局生效版本 (mise current):" -ForegroundColor Cyan
    & mise current
    Pause-Console
}

# ==============================================================================
# 子菜单交互处理
# ==============================================================================
function Menu-Node {
    while ($true) {
        Clear-Host
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  ⚡ Node.js 运行环境安装与版本管理" -ForegroundColor White
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  1) Node.js 22 (Current LTS 最新长期支持版)"
        Write-Host "  2) Node.js 20 (Active LTS 稳定生产推荐版)"
        Write-Host "  3) Node.js 18 (Maintenance LTS 维护版)"
        Write-Host "  4) 自定义版本号 (如 20.18.0 或 22.11.0)"
        Write-Host "  0) 返回主菜单"
        Write-Host "------------------------------------------------------" -ForegroundColor Cyan
        Write-Host "请输入选择 [0-4]: " -NoNewline
        $c = Read-Host
        switch ($c) {
            "1" { Install-Runtime "node" "22" "Node.js"; return }
            "2" { Install-Runtime "node" "20" "Node.js"; return }
            "3" { Install-Runtime "node" "18" "Node.js"; return }
            "4" {
                Write-Host "请输入具体的 Node.js 版本号: " -NoNewline
                $v = Read-Host
                if ($v) { Install-Runtime "node" $v "Node.js" }
                return
            }
            "0" { return }
        }
    }
}

function Menu-Python {
    while ($true) {
        Clear-Host
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  ⚡ Python 运行环境安装与版本管理" -ForegroundColor White
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  1) Python 3.12 (推荐现代主流稳定版)"
        Write-Host "  2) Python 3.11 (高性能成熟版)"
        Write-Host "  3) Python 3.10 (广泛兼容版)"
        Write-Host "  4) 自定义版本号 (如 3.12.7 或 3.9.20)"
        Write-Host "  0) 返回主菜单"
        Write-Host "------------------------------------------------------" -ForegroundColor Cyan
        Write-Host "请输入选择 [0-4]: " -NoNewline
        $c = Read-Host
        switch ($c) {
            "1" { Install-Runtime "python" "3.12" "Python"; return }
            "2" { Install-Runtime "python" "3.11" "Python"; return }
            "3" { Install-Runtime "python" "3.10" "Python"; return }
            "4" {
                Write-Host "请输入具体的 Python 版本号: " -NoNewline
                $v = Read-Host
                if ($v) { Install-Runtime "python" $v "Python" }
                return
            }
            "0" { return }
        }
    }
}

function Menu-Go {
    while ($true) {
        Clear-Host
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  ⚡ Go 语言运行环境安装与版本管理" -ForegroundColor White
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  1) Go 1.23 (最新官方稳定版)"
        Write-Host "  2) Go 1.22 (企业常用稳定版)"
        Write-Host "  3) Go 1.21 (历史长期支持版)"
        Write-Host "  4) 自定义版本号 (如 1.23.2 或 1.22.8)"
        Write-Host "  0) 返回主菜单"
        Write-Host "------------------------------------------------------" -ForegroundColor Cyan
        Write-Host "请输入选择 [0-4]: " -NoNewline
        $c = Read-Host
        switch ($c) {
            "1" { Install-Runtime "go" "1.23" "Go"; return }
            "2" { Install-Runtime "go" "1.22" "Go"; return }
            "3" { Install-Runtime "go" "1.21" "Go"; return }
            "4" {
                Write-Host "请输入具体的 Go 版本号: " -NoNewline
                $v = Read-Host
                if ($v) { Install-Runtime "go" $v "Go" }
                return
            }
            "0" { return }
        }
    }
}

function Menu-Java {
    while ($true) {
        Clear-Host
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  ⚡ Java (JDK) 运行环境安装与版本管理" -ForegroundColor White
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  1) Java 21 (LTS 现代主流长期支持版)"
        Write-Host "  2) Java 17 (LTS 经典稳定长期支持版)"
        Write-Host "  3) Java 11 (LTS 传统企业版)"
        Write-Host "  4) Java 8  (LTS 历史遗留兼容版)"
        Write-Host "  5) 自定义版本号 (如 corretto-21 或 openjdk-17)"
        Write-Host "  0) 返回主菜单"
        Write-Host "------------------------------------------------------" -ForegroundColor Cyan
        Write-Host "请输入选择 [0-5]: " -NoNewline
        $c = Read-Host
        switch ($c) {
            "1" { Install-Runtime "java" "21" "Java JDK"; return }
            "2" { Install-Runtime "java" "17" "Java JDK"; return }
            "3" { Install-Runtime "java" "11" "Java JDK"; return }
            "4" { Install-Runtime "java" "8"  "Java JDK"; return }
            "5" {
                Write-Host "请输入具体的 Java 版本号: " -NoNewline
                $v = Read-Host
                if ($v) { Install-Runtime "java" $v "Java JDK" }
                return
            }
            "0" { return }
        }
    }
}

function Menu-Rust {
    while ($true) {
        Clear-Host
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  ⚡ Rust 语言与 Cargo 开发环境安装" -ForegroundColor White
        Write-Host "======================================================" -ForegroundColor Cyan
        Write-Host "  1) Rust latest (最新官方稳定版)"
        Write-Host "  2) Rust 1.81 (成熟稳定版)"
        Write-Host "  3) 自定义版本号 (如 1.80.0)"
        Write-Host "  0) 返回主菜单"
        Write-Host "------------------------------------------------------" -ForegroundColor Cyan
        Write-Host "请输入选择 [0-3]: " -NoNewline
        $c = Read-Host
        switch ($c) {
            "1" { Install-Runtime "rust" "latest" "Rust"; return }
            "2" { Install-Runtime "rust" "1.81" "Rust"; return }
            "3" {
                Write-Host "请输入具体的 Rust 版本号: " -NoNewline
                $v = Read-Host
                if ($v) { Install-Runtime "rust" $v "Rust" }
                return
            }
            "0" { return }
        }
    }
}

function Install-AllDefaults {
    Write-EnvStep "🚀 开始一键安装全套常用开发环境 (Node.js + Python + Go + Java)"
    Write-Host "将依次部署: Node.js 22, Python 3.12, Go 1.23, Java 21`n" -ForegroundColor Cyan

    & mise use -g node@22 python@3.12 go@1.23 java@21
    & mise reshim

    Write-EnvSucc "全套常用开发环境部署完成！"
    Write-Host "`n>> 验证环境状态:" -ForegroundColor Cyan
    Write-Host -NoNewline "  • Node.js: " ; try { node -v } catch { "未就绪" }
    Write-Host -NoNewline "  • Python:  " ; try { python --version } catch { "未就绪" }
    Write-Host -NoNewline "  • Go:      " ; try { go version } catch { "未就绪" }
    Write-Host -NoNewline "  • Java:    " ; try { java -version 2>&1 | Select-Object -First 1 } catch { "未就绪" }

    Pause-Console
}

# ==============================================================================
# 主菜单循环
# ==============================================================================
function Main-Menu {
    while ($true) {
        Clear-Host
        $miseVer = try { (& mise --version) 2>$null } catch { "未就绪" }

        Write-Host "  ______            _   _       _     " -ForegroundColor Blue
        Write-Host " |  ____|          | | | |     | |    " -ForegroundColor Blue
        Write-Host " | |__   _ ____   _| |_| |_   _| |__  " -ForegroundColor Blue
        Write-Host " |  __| | '_ \ \ / /  _  | | | | '_ \ " -ForegroundColor Blue
        Write-Host " | |____| | | \ V /| | | | |_| | |_) |" -ForegroundColor Blue
        Write-Host " |______|_| |_|\_/ |_| |_|\__,_|_.__/ " -ForegroundColor Blue
        Write-Host "⚡ EnvHub 跨平台开发环境交互式向导 (Windows PowerShell)" -ForegroundColor White
        Write-Host "----------------------------------------------------------------------" -ForegroundColor Cyan
        Write-Host " 宿主系统: Windows | 包管理器: $global:PKG_MGR | Mise: $miseVer" -ForegroundColor White
        Write-Host "======================================================================" -ForegroundColor Cyan
        Write-Host " [常用编程语言与运行环境]" -ForegroundColor Yellow
        Write-Host "   1) Node.js   (JavaScript / TypeScript 运行环境与 NPM)"
        Write-Host "   2) Python    (Python3 科学计算、AI 与后端开发环境)"
        Write-Host "   3) Go        (Golang 高性能云原生开发环境)"
        Write-Host "   4) Java      (Java JDK 企业级通用运行环境)"
        Write-Host "   5) Rust      (Rust 语言编译器与 Cargo 包管理)"
        Write-Host "   6) 🚀 一键安装全部常用开发环境 (Node + Python + Go + Java)"
        Write-Host ""
        Write-Host " [系统工具、镜像与诊断]" -ForegroundColor Yellow
        Write-Host "   7) ⚡ 一键配置国内高速镜像加速 (NPM, PyPI, Go Proxy, Cargo)"
        Write-Host "   8) 📋 查看当前已安装的运行时版本列表与状态"
        Write-Host "   9) 🔄 刷新与同步全局 Shims (mise reshim)"
        Write-Host ""
        Write-Host "   0) 🚪 退出脚本 (Exit)"
        Write-Host "======================================================================" -ForegroundColor Cyan
        Write-Host "请输入对应的功能数字 [0-9]: " -NoNewline
        $choice = Read-Host

        switch ($choice) {
            "1" { Menu-Node }
            "2" { Menu-Python }
            "3" { Menu-Go }
            "4" { Menu-Java }
            "5" { Menu-Rust }
            "6" { Install-AllDefaults }
            "7" { Setup-Mirrors }
            "8" { Show-RuntimeStatus }
            "9" {
                & mise reshim
                Write-EnvSucc "已成功刷新全局 Shims 软链接！"
                Pause-Console
            }
            "0" {
                Write-Host ""
                Write-EnvSucc "感谢使用 EnvHub 交互式向导，再见！"
                Write-Host ""
                return
            }
        }
    }
}

# --- 执行入口 ---
Ensure-Mise
Ensure-PackageManager
Main-Menu
