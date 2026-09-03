export interface ExportToolItem {
  id: string;
  name: string;
  version: string;
  checked: boolean;
}

export interface ExportScriptOptions {
  includeMirrors: boolean;
  includeEnvHook: boolean;
}

/**
 * Generate macOS / Linux Shell Setup Script (setup_envhub.sh)
 */
export function generateUnixShellScript(
  tools: ExportToolItem[],
  options: ExportScriptOptions
): string {
  const selected = tools.filter((t) => t.checked && t.version && t.version !== 'system');
  
  const toolInstalls = selected
    .map((t) => `  install_tool "${t.id}" "${t.version}" "${t.name}"`)
    .join('\n');

  const toolVerifications = selected
    .map((t) => `  verify_tool "${t.id}"`)
    .join('\n');

  return `#!/usr/bin/env bash
# ==============================================================================
# EnvHub - 跨平台开发环境与多语言版本一键部署脚本 (macOS / Linux)
# 由 EnvHub 自动导出生成 | https://github.com/WnJee/EnvHub
# ==============================================================================

set -e

# --- 颜色与样式输出 ---
BOLD="\\033[1m"
GREEN="\\033[0;32m"
BLUE="\\033[0;34m"
YELLOW="\\033[0;33m"
CYAN="\\033[0;36m"
RED="\\033[0;31m"
NC="\\033[0m"

log_info() { echo -e "\${BLUE}[EnvHub]\${NC} \$*"; }
log_succ() { echo -e "\${GREEN}[✓ 成功]\${NC} \$*"; }
log_warn() { echo -e "\${YELLOW}[! 提示]\${NC} \$*"; }
log_step() { echo -e "\\n\${BOLD}\${CYAN}==> \$*\${NC}"; }

echo -e "\${BOLD}\${BLUE}"
echo "  ______            _   _       _     "
echo " |  ____|          | | | |     | |    "
echo " | |__   _ ____   _| |_| |_   _| |__  "
echo " |  __| | '_ \\ \\ / /  _  | | | | '_ \\ "
echo " | |____| | | \\ V /| | | | |_| | |_) |"
echo " |______|_| |_|\\_/ |_| |_|\\__,_|_.__/ "
echo -e "\${NC}"
echo -e "\${BOLD}开始在本机自动部署固定语言运行时与开发环境...\${NC}\\n"

# 1. 检查并安装 Mise CLI 运行时管理引擎
log_step "步骤 1/4: 检查与就绪 Mise 核心引擎"
if ! command -v mise &> /dev/null; then
  log_info "未检测到 mise，正在通过官方脚本极速安装..."
  curl -fsSL https://mise.run | sh
  export PATH="\$HOME/.local/share/mise/shims:\$HOME/.local/bin:\$PATH"
else
  log_succ "已检测到 mise: \$(mise --version)"
fi

# 2. 配置 Shell 环境变量与激活 Hook
${
  options.includeEnvHook
    ? `# 2. 配置 Shell 环境变量与激活 Hook
log_step "步骤 2/4: 配置系统终端环境变量与 Shims 劫持"
export PATH="\$HOME/.local/share/mise/shims:\$HOME/.local/bin:\$PATH"

setup_shell_hook() {
  local rc_file="\$1"
  local hook_cmd="\$2"
  if [ -f "\$rc_file" ]; then
    if ! grep -q "mise activate" "\$rc_file" 2>/dev/null; then
      echo -e "\\n# === EnvHub / Mise Activation Hook ===" >> "\$rc_file"
      echo 'export PATH="\$HOME/.local/share/mise/shims:\$PATH"' >> "\$rc_file"
      echo "\$hook_cmd" >> "\$rc_file"
      log_succ "已注入环境配置到: \$rc_file"
    else
      log_info "\$rc_file 已存在 mise 激活配置"
    fi
  fi
}

[ -f "\$HOME/.zshrc" ] && setup_shell_hook "\$HOME/.zshrc" 'eval "\$(~/.local/bin/mise activate zsh)"'
[ -f "\$HOME/.bashrc" ] && setup_shell_hook "\$HOME/.bashrc" 'eval "\$(~/.local/bin/mise activate bash)"'
[ -f "\$HOME/.zprofile" ] && setup_shell_hook "\$HOME/.zprofile" 'eval "\$(~/.local/bin/mise activate zsh)"'
`
    : `# 跳过环境变量注入配置`
}

# 3. 配置国内常用镜像加速源 (可选)
${
  options.includeMirrors
    ? `log_step "步骤 3/4: 配置国内高速镜像源"
# NPM 镜像源加速
if command -v npm &> /dev/null; then
  npm config set registry https://registry.npmmirror.com/ || true
  log_succ "已配置 NPM 镜像源 -> https://registry.npmmirror.com/"
fi

# Python Pip 镜像源加速
mkdir -p "\$HOME/.pip"
cat << 'PIPCFG' > "\$HOME/.pip/pip.conf"
[global]
index-url = https://pypi.tuna.tsinghua.edu.cn/simple
trusted-host = pypi.tuna.tsinghua.edu.cn
PIPCFG
log_succ "已配置 Pip 镜像源 -> 清华大学 TUNA 镜像"

# Go GOPROXY 加速
if command -v go &> /dev/null; then
  go env -w GOPROXY=https://goproxy.cn,direct || true
  log_succ "已配置 Go GOPROXY -> https://goproxy.cn"
fi

# Rust Crates.io 镜像加速 (rsproxy)
mkdir -p "\$HOME/.cargo"
cat << 'CARGOCFG' > "\$HOME/.cargo/config.toml"
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[net]
git-fetch-with-cli = true
CARGOCFG
log_succ "已配置 Cargo 镜像源 -> rsproxy.cn"
`
    : `log_step "步骤 3/4: 跳过国内镜像源配置"`
}

# 4. 批量下载并锁定语言运行时版本
log_step "步骤 4/4: 下载并全局锁定指定多语言环境版本"

install_tool() {
  local tool_id="\$1"
  local ver="\$2"
  local name="\$3"
  log_info "正在安装 \${name} (\${tool_id}@\${ver})..."
  mise use -g "\${tool_id}@\${ver}"
  log_succ "\${name} \${ver} 已安装并设为主用版本"
}

${toolInstalls}

# 刷新全局 Shims
mise reshim

# 5. 验证安装结果
log_step "环境部署完成！验证当前主用版本："

verify_tool() {
  local tool_id="\$1"
  local current=\$(mise current "\$tool_id" 2>/dev/null || echo "未就绪")
  echo -e "  \${GREEN}✔\${NC} \${BOLD}\${tool_id}\${NC}: \${CYAN}\${current}\${NC}"
}

${toolVerifications}

echo ""
log_succ "🎉 全部语言环境与版本配置部署完成！"
log_warn "请重启终端，或执行以下命令使新环境变量立即生效："
if [ -n "\$ZSH_VERSION" ] || [ "\$SHELL" = "*/zsh" ]; then
  echo -e "  \${BOLD}\${CYAN}source ~/.zshrc\${NC}\\n"
else
  echo -e "  \${BOLD}\${CYAN}source ~/.bashrc\${NC}\\n"
fi
`;
}

/**
 * Generate Windows PowerShell Setup Script (setup_envhub.ps1)
 */
export function generateWindowsPowerShellScript(
  tools: ExportToolItem[],
  options: ExportScriptOptions
): string {
  const selected = tools.filter((t) => t.checked && t.version && t.version !== 'system');

  const toolInstalls = selected
    .map((t) => `  Install-EnvTool -ToolId "${t.id}" -Version "${t.version}" -Name "${t.name}"`)
    .join('\n');

  const toolVerifications = selected
    .map((t) => `  Verify-EnvTool -ToolId "${t.id}"`)
    .join('\n');

  return `<#
==============================================================================
 EnvHub - Windows 跨平台多语言环境一键安装脚本 (PowerShell)
 由 EnvHub 自动导出生成 | https://github.com/WnJee/EnvHub
==============================================================================
#>

[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()
$OutputEncoding = [System.Text.UTF8Encoding]::new()
$ErrorActionPreference = "Stop"

function Write-EnvStep {
    param([string]$Text)
    Write-Host "\`n==> $Text" -ForegroundColor Cyan
}

function Write-EnvSucc {
    param([string]$Text)
    Write-Host "[✓ 成功] $Text" -ForegroundColor Green
}

function Write-EnvInfo {
    param([string]$Text)
    Write-Host "[EnvHub] $Text" -ForegroundColor Blue
}

function Write-EnvWarn {
    param([string]$Text)
    Write-Host "[! 提示] $Text" -ForegroundColor Yellow
}

Write-Host @"
  ______            _   _       _     
 |  ____|          | | | |     | |    
 | |__   _ ____   _| |_| |_   _| |__  
 |  __| | '_ \\ \\ / /  _  | | | | '_ \\ 
 | |____| | | \\ V /| | | | |_| | |_) |
 |______|_| |_|\\_/ |_| |_|\\__,_|_.__/ 

开始在 Windows 本机自动安装部署语言环境...
"@ -ForegroundColor Cyan

# 1. 检查并安装 Mise CLI
Write-EnvStep "步骤 1/4: 检查与安装 Mise 引擎"
$MiseCmd = Get-Command "mise" -ErrorAction SilentlyContinue

if (-not $MiseCmd) {
    Write-EnvInfo "未检测到 mise CLI，正在通过官方 PowerShell 脚本安装..."
    irm https://mise.jdx.dev/install.ps1 | iex
    $env:Path = "$env:LOCALAPPDATA\\mise\\bin;$env:LOCALAPPDATA\\mise\\shims;" + $env:Path
    Write-EnvSucc "Mise 引擎安装完成"
} else {
    Write-EnvSucc "已检测到 Mise: $((mise --version))"
}

# 2. 注入 PowerShell Profile 环境变量与 Shims
${
  options.includeEnvHook
    ? `Write-EnvStep "步骤 2/4: 配置 PowerShell Profile 与环境变量"
$ProfileDir = Split-Path -Parent $PROFILE
if (-not (Test-Path $ProfileDir)) {
    New-Item -ItemType Directory -Path $ProfileDir -Force | Out-Null
}

$MiseHook = "\`n# === EnvHub / Mise Activation Hook ===\`n(& mise activate ps1) | Out-String | Invoke-Expression\`n"
if (Test-Path $PROFILE) {
    $ExistingProfile = Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue
    if ($ExistingProfile -notmatch "mise activate") {
        Add-Content -Path $PROFILE -Value $MiseHook
        Write-EnvSucc "已注入环境变量激活代码到 $PROFILE"
    }
} else {
    Set-Content -Path $PROFILE -Value $MiseHook
    Write-EnvSucc "已创建并配置 Profile: $PROFILE"
}
`
    : `# 跳过 PowerShell Profile 注入`
}

# 3. 国内镜像源加速配置
${
  options.includeMirrors
    ? `Write-EnvStep "步骤 3/4: 配置国内高速镜像源"

# NPM 镜像源
if (Get-Command "npm" -ErrorAction SilentlyContinue) {
    npm config set registry https://registry.npmmirror.com/ | Out-Null
    Write-EnvSucc "已配置 NPM 镜像 -> https://registry.npmmirror.com/"
}

# Python Pip 镜像源
$PipDir = "$env:APPDATA\\pip"
if (-not (Test-Path $PipDir)) { New-Item -ItemType Directory -Path $PipDir -Force | Out-Null }
@'
[global]
index-url = https://pypi.tuna.tsinghua.edu.cn/simple
trusted-host = pypi.tuna.tsinghua.edu.cn
'@ | Set-Content -Path "$PipDir\\pip.ini" -Encoding UTF8
Write-EnvSucc "已配置 Pip 镜像 -> 清华大学 TUNA 镜像"

# Go GOPROXY
if (Get-Command "go" -ErrorAction SilentlyContinue) {
    go env -w GOPROXY="https://goproxy.cn,direct" | Out-Null
    Write-EnvSucc "已配置 Go GOPROXY -> https://goproxy.cn"
}

# Rust Cargo (rsproxy)
$CargoDir = "$env:USERPROFILE\\.cargo"
if (-not (Test-Path $CargoDir)) { New-Item -ItemType Directory -Path $CargoDir -Force | Out-Null }
@'
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[net]
git-fetch-with-cli = true
'@ | Set-Content -Path "$CargoDir\\config.toml" -Encoding UTF8
Write-EnvSucc "已配置 Cargo 镜像 -> rsproxy.cn"
`
    : `Write-EnvStep "步骤 3/4: 跳过国内镜像源配置"`
}

# 4. 批量下载并锁定语言运行时版本
Write-EnvStep "步骤 4/4: 下载并全局锁定指定多语言环境版本"

function Install-EnvTool {
    param(
        [string]$ToolId,
        [string]$Version,
        [string]$Name
    )
    Write-EnvInfo "正在安装 $Name ($ToolId@$Version)..."
    mise use -g "$ToolId@$Version"
    Write-EnvSucc "$Name $Version 已成功安装并激活"
}

${toolInstalls}

mise reshim

# 5. 验证安装结果
Write-EnvStep "验证当前主用版本状态："

function Verify-EnvTool {
    param([string]$ToolId)
    $Current = mise current $ToolId 2>$null
    Write-Host "  [✔] $ToolId : " -NoNewline -ForegroundColor Green
    Write-Host "$Current" -ForegroundColor Cyan
}

${toolVerifications}

Write-Host "\`n🎉 全部语言环境与版本配置在 Windows 部署完成！" -ForegroundColor Green
Write-EnvWarn "请重启当前 PowerShell 窗口以使全局命令与 Shims 路径生效。"
`;
}

/**
 * Generate Windows Batch Wrapper Script (setup_envhub.bat)
 */
export function generateWindowsBatchScript(): string {
  return `@echo off
chcp 65001 > nul
title EnvHub - Windows 自动化环境部署
echo ==============================================================================
echo  EnvHub 环境一键部署启动器
echo ==============================================================================
echo 正在以 Bypass 权限启动 PowerShell 安装脚本...
echo.

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup_envhub.ps1"

echo.
echo ==============================================================================
echo  脚本运行结束，请按任意键退出...
echo ==============================================================================
pause > nul
`;
}

/**
 * Generate .mise.toml Configuration File
 */
export function generateMiseTomlConfig(tools: ExportToolItem[]): string {
  const selected = tools.filter((t) => t.checked && t.version && t.version !== 'system');

  const toolsToml = selected
    .map((t) => `  ${t.id} = "${t.version}"`)
    .join('\n');

  return `# ==============================================================================
# EnvHub - 跨平台开发环境与多语言版本配置文件
# 放置于项目根目录或 ~/.config/mise/config.toml 中生效
# ==============================================================================

[tools]
${toolsToml}

[settings]
legacy_version_file = true
experimental = true
`;
}
