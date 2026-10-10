#!/usr/bin/env bash
# ==============================================================================
# EnvHub - 跨平台开发环境与多语言版本交互式向导 (macOS / Linux)
# 项目地址: https://github.com/WnJee/EnvHub
# ==============================================================================

set -u
set -o pipefail

# Keep user input separate from the script pipe (curl | bash).
exec 3</dev/tty || { echo "Interactive terminal required" >&2; exit 1; }

# --- 颜色与样式配置 ---
BOLD="\033[1m"
GREEN="\033[0;32m"
BLUE="\033[0;34m"
YELLOW="\033[0;33m"
CYAN="\033[0;36m"
RED="\033[0;31m"
PURPLE="\033[0;35m"
NC="\033[0m"

log_info() { echo -e "${BLUE}[EnvHub]${NC} $*"; }
log_succ() { echo -e "${GREEN}[✓ 成功]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[! 提示]${NC} $*"; }
log_err()  { echo -e "${RED}[✗ 错误]${NC} $*"; }
log_step() { echo -e "\n${BOLD}${CYAN}==> $*${NC}"; }

press_enter() {
  echo ""
  read -r -p "按回车键继续..." _ <&3 || exit 0
}

# --- 环境变量初始化与加载 ---
init_env_path() {
  export PATH="$HOME/.local/share/mise/shims:$HOME/.local/bin:/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:/usr/local/sbin:$PATH"
}
init_env_path

# ==============================================================================
# 阶段 1: 检查并安装 Mise 版本管理器
# ==============================================================================
ensure_mise() {
  log_step "步骤 1: 检查 Mise 版本管理器状态"
  
  if command -v mise &>/dev/null; then
    MISE_BIN="$(command -v mise)"
    log_succ "检测到 Mise 已就绪: $($MISE_BIN --version | head -n 1)"
    return 0
  fi

  for candidate in "$HOME/.local/bin/mise" "/opt/homebrew/bin/mise" "/usr/local/bin/mise"; do
    if [ -x "$candidate" ]; then
      MISE_BIN="$candidate"
      export PATH="$(dirname "$candidate"):$PATH"
      log_succ "检测到 Mise: $($MISE_BIN --version | head -n 1)"
      return 0
    fi
  done

  log_warn "未在当前系统中检测到 Mise，开始自动安装..."

  # 检测网络连通性
  USE_MIRROR=0
  if ! curl -Is --connect-timeout 3 https://raw.githubusercontent.com >/dev/null 2>&1; then
    log_warn "检测到 GitHub 连接较慢，正在切换至备用加速通道..."
    USE_MIRROR=1
  fi

  mkdir -p "$HOME/.local/bin"
  if [ "$USE_MIRROR" -eq 1 ]; then
    curl -fsSL https://mise.run | sh || curl -fsSL https://mise.jdx.dev/install.sh | sh
  else
    curl -fsSL https://mise.run | sh
  fi

  MISE_BIN="$HOME/.local/bin/mise"
  if [ ! -x "$MISE_BIN" ]; then
    log_err "Mise 安装未能成功生成可执行文件，请检查网络后重试！"
    exit 1
  fi

  export PATH="$HOME/.local/bin:$HOME/.local/share/mise/shims:$PATH"

  # 配置 Shell 激活钩子
  setup_shell_profile() {
    local rc_file="$1"
    local hook_cmd="$2"
    if [ -f "$rc_file" ]; then
      if ! grep -q "mise activate" "$rc_file" 2>/dev/null; then
        echo -e "\n# Mise Version Manager Hook\n$hook_cmd" >> "$rc_file"
        log_succ "已注入 Mise 激活钩子 -> $rc_file"
      fi
    else
      touch "$rc_file"
      echo -e "# Mise Version Manager Hook\n$hook_cmd" >> "$rc_file"
      log_succ "已创建并注入 Mise 激活钩子 -> $rc_file"
    fi
  }

  CURRENT_SHELL="$(basename "${SHELL:-bash}")"
  case "$CURRENT_SHELL" in
    zsh)
      setup_shell_profile "$HOME/.zshrc" 'eval "$(mise activate zsh)"'
      setup_shell_profile "$HOME/.zprofile" 'eval "$(mise activate zsh)"'
      ;;
    bash)
      setup_shell_profile "$HOME/.bashrc" 'eval "$(mise activate bash)"'
      setup_shell_profile "$HOME/.bash_profile" 'eval "$(mise activate bash)"'
      ;;
    *)
      setup_shell_profile "$HOME/.profile" 'eval "$(mise activate bash)"'
      ;;
  esac

  log_succ "Mise 安装并配置完成: $($MISE_BIN --version | head -n 1)"
}

# ==============================================================================
# 阶段 2: 检查并询问安装包管理器
# ==============================================================================
ensure_package_manager() {
  log_step "步骤 2: 检查系统包管理器状态"
  OS_TYPE="$(uname -s)"

  if [ "$OS_TYPE" = "Darwin" ]; then
    # macOS - 检查 Homebrew
    if command -v brew &>/dev/null || [ -x "/opt/homebrew/bin/brew" ] || [ -x "/usr/local/bin/brew" ]; then
      if [ -x "/opt/homebrew/bin/brew" ]; then eval "$(/opt/homebrew/bin/brew shellenv)"; fi
      if [ -x "/usr/local/bin/brew" ]; then eval "$(/usr/local/bin/brew shellenv)"; fi
      log_succ "检测到 macOS 包管理器 (Homebrew) 已就绪: $(brew --version | head -n 1)"
      PKG_MGR="Homebrew"
      return 0
    fi

    log_warn "未检测到 macOS 系统包管理器 (Homebrew)。"
    echo -e "${YELLOW}Homebrew 是 macOS 上安装 Git、数据库与开发中间件所必需的底层工具包管理器。${NC}"
    read -r -p "是否立即安装 Homebrew？[Y/n]: " choice <&3 || exit 0
    choice="${choice:-Y}"

    if [[ "$choice" =~ ^[Yy]$ ]]; then
      log_info "正在为您配置 Homebrew 安装环境..."
      if ! curl -Is --connect-timeout 3 https://raw.githubusercontent.com >/dev/null 2>&1; then
        log_info "启用国内高速镜像进行 Homebrew 安装..."
        /bin/zsh -c "$(curl -fsSL https://gitee.com/cunkai/HomebrewCN/raw/master/Homebrew.sh)"
      else
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
      fi

      if [ -x "/opt/homebrew/bin/brew" ]; then eval "$(/opt/homebrew/bin/brew shellenv)"; fi
      if [ -x "/usr/local/bin/brew" ]; then eval "$(/usr/local/bin/brew shellenv)"; fi
      log_succ "Homebrew 安装配置流程已完成！"
      PKG_MGR="Homebrew"
    else
      log_info "已跳过 Homebrew 安装。"
      PKG_MGR="未安装"
    fi

  elif [ "$OS_TYPE" = "Linux" ]; then
    # Linux - 检查常用包管理器
    if command -v apt-get &>/dev/null; then
      PKG_MGR="APT (Debian/Ubuntu)"
    elif command -v dnf &>/dev/null; then
      PKG_MGR="DNF (Fedora/RHEL)"
    elif command -v pacman &>/dev/null; then
      PKG_MGR="Pacman (Arch Linux)"
    elif command -v yum &>/dev/null; then
      PKG_MGR="YUM (CentOS/RHEL)"
    elif command -v brew &>/dev/null; then
      PKG_MGR="Linuxbrew"
    else
      PKG_MGR="标准 Linux"
    fi
    log_succ "检测到 Linux 宿主包管理器: ${PKG_MGR}"
  fi
}

# ==============================================================================
# 运行时安装与配置辅助函数
# ==============================================================================
install_runtime() {
  local tool_id="$1"
  local version="$2"
  local name="$3"

  log_step "正在安装全局运行环境: ${name} (${tool_id}@${version})"
  echo ">> 自动通过 Mise 远程解析并安装最新的匹配版本..."
  echo ">> 执行命令: mise use -g ${tool_id}@${version}"
  
  if "$MISE_BIN" use -g "${tool_id}@${version}"; then
    "$MISE_BIN" reshim
    log_succ "${name} (${tool_id}@${version}) 安装并设置为全局默认成功！"
    echo -n ">> 当前生效版本: "
    case "$tool_id" in
      node) node -v 2>/dev/null || true ;;
      python) python3 --version 2>/dev/null || python --version 2>/dev/null || true ;;
      go) go version 2>/dev/null || true ;;
      java) java -version 2>&1 | head -n 1 || true ;;
      rust) rustc --version 2>/dev/null || true ;;
      *) "$MISE_BIN" current "$tool_id" 2>/dev/null || true ;;
    esac
  else
    log_err "${name} 安装失败，请检查网络连接或日志。"
  fi
  press_enter
}

# --- 动态查看远程版本列表 ---
browse_remote_versions() {
  local tool_id="$1"
  local name="$2"
  log_step "正在查询 ${name} 官方远程最新可用版本列表..."
  echo "------------------------------------------------------"
  "$MISE_BIN" ls-remote "$tool_id" 2>/dev/null | grep -E '^[0-9]' | tail -n 25 | column -c 80 2>/dev/null || "$MISE_BIN" ls-remote "$tool_id" 2>/dev/null | tail -n 20
  echo "------------------------------------------------------"
  echo "提示: 可以输入上方列出的具体版本号 (如 24.9.0 或 3.14.2)，或输入 'latest' 获取最新版"
  read -r -p "请输入您要安装的 ${name} 版本号 (直接回车取消): " custom_v <&3 || exit 0
  if [ -n "$custom_v" ]; then
    install_runtime "$tool_id" "$custom_v" "$name"
  fi
}

# --- 国内镜像源配置 ---
setup_mirrors() {
  log_step "配置国内高速开发镜像加速源"

  # NPM
  if command -v npm &>/dev/null; then
    npm config set registry https://registry.npmmirror.com/ >/dev/null 2>&1 || true
    log_succ "NPM 镜像源 -> https://registry.npmmirror.com/"
  fi

  # Python Pip
  if command -v pip3 &>/dev/null || command -v pip &>/dev/null; then
    if command -v pip3 &>/dev/null; then
      pip3 config --user set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple || return 1
    else
      pip config --user set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple || return 1
    fi
    log_succ "Pip 镜像源 -> 清华大学开源镜像站"
  fi

  # Go Proxy
  if command -v go &>/dev/null; then
    go env -w GOPROXY=https://goproxy.cn,direct >/dev/null 2>&1 || true
    log_succ "Go Proxy -> https://goproxy.cn,direct"
  fi

  # Rust Crates
  mkdir -p "$HOME/.cargo"
  if [ ! -e "$HOME/.cargo/config.toml" ] && [ ! -e "$HOME/.cargo/config" ]; then
    cat > "$HOME/.cargo/config.toml" << 'EOF'
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
EOF
    log_succ "Cargo 镜像源 -> 中科大 Crates.io 镜像"
  fi

  # Homebrew Bottles
  if command -v brew &>/dev/null; then
    export HOMEBREW_BOTTLE_DOMAIN="https://mirrors.ustc.edu.cn/homebrew-bottles"
    log_succ "Homebrew Bottles -> 中科大镜像加速"
  fi

  log_succ "国内镜像源加速配置完成！"
  press_enter
}

# --- 查看已安装运行时 ---
show_runtime_status() {
  log_step "当前已安装与激活的运行环境列表"
  "$MISE_BIN" list
  echo ""
  log_info "全局生效版本 (mise current):"
  "$MISE_BIN" current
  press_enter
}

# ==============================================================================
# 子菜单交互处理 (动态解析最新版本，杜绝硬编码陈旧版本)
# ==============================================================================
menu_node() {
  while true; do
    clear
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo -e "${BOLD}  ⚡ Node.js 运行环境安装与版本管理${NC}"
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo "  1) Node.js lts       (🌟 官方长期支持版，自动解析最新 LTS 如 24/22)"
    echo "  2) Node.js latest    (🚀 官方最新发布版，含最新语言特性如 26/25)"
    echo "  3) Node.js 24        (稳定主流生产大版本)"
    echo "  4) Node.js 22        (经典活跃维护大版本)"
    echo "  5) 🔍 浏览远程可用版本列表 / 自定义输入版本号"
    echo "  0) 返回主菜单"
    echo -e "${CYAN}------------------------------------------------------${NC}"
    read -r -p "请输入选择 [0-5]: " sub_choice <&3 || exit 0
    case "$sub_choice" in
      1) install_runtime "node" "lts" "Node.js"; break ;;
      2) install_runtime "node" "latest" "Node.js"; break ;;
      3) install_runtime "node" "24" "Node.js"; break ;;
      4) install_runtime "node" "22" "Node.js"; break ;;
      5) browse_remote_versions "node" "Node.js"; break ;;
      0) break ;;
      *) echo "无效选择，请重新输入"; sleep 1 ;;
    esac
  done
}

menu_python() {
  while true; do
    clear
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo -e "${BOLD}  ⚡ Python 运行环境安装与版本管理${NC}"
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo "  1) Python latest     (🌟 官方最新稳定版，自动匹配最新 Python 3.x)"
    echo "  2) Python 3.14       (最新特性发布版)"
    echo "  3) Python 3.13       (现代主流稳定版，性能大幅提升)"
    echo "  4) Python 3.12       (企业生产成熟版)"
    echo "  5) 🔍 浏览远程可用版本列表 / 自定义输入版本号"
    echo "  0) 返回主菜单"
    echo -e "${CYAN}------------------------------------------------------${NC}"
    read -r -p "请输入选择 [0-5]: " sub_choice <&3 || exit 0
    case "$sub_choice" in
      1) install_runtime "python" "latest" "Python"; break ;;
      2) install_runtime "python" "3.14" "Python"; break ;;
      3) install_runtime "python" "3.13" "Python"; break ;;
      4) install_runtime "python" "3.12" "Python"; break ;;
      5) browse_remote_versions "python" "Python"; break ;;
      0) break ;;
      *) echo "无效选择，请重新输入"; sleep 1 ;;
    esac
  done
}

menu_go() {
  while true; do
    clear
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo -e "${BOLD}  ⚡ Go 语言运行环境安装与版本管理${NC}"
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo "  1) Go latest         (🌟 官方最新稳定版，自动匹配最新如 1.27)"
    echo "  2) Go 1.27           (最新发布大版本)"
    echo "  3) Go 1.26           (成熟稳定推荐版本)"
    echo "  4) Go 1.25           (广泛兼容版本)"
    echo "  5) 🔍 浏览远程可用版本列表 / 自定义输入版本号"
    echo "  0) 返回主菜单"
    echo -e "${CYAN}------------------------------------------------------${NC}"
    read -r -p "请输入选择 [0-5]: " sub_choice <&3 || exit 0
    case "$sub_choice" in
      1) install_runtime "go" "latest" "Go"; break ;;
      2) install_runtime "go" "1.27" "Go"; break ;;
      3) install_runtime "go" "1.26" "Go"; break ;;
      4) install_runtime "go" "1.25" "Go"; break ;;
      5) browse_remote_versions "go" "Go"; break ;;
      0) break ;;
      *) echo "无效选择，请重新输入"; sleep 1 ;;
    esac
  done
}

menu_java() {
  while true; do
    clear
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo -e "${BOLD}  ⚡ Java (JDK) 运行环境安装与版本管理${NC}"
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo "  1) Java lts          (🌟 官方最新 LTS 长期支持版)"
    echo "  2) Java 25           (现代主流长期支持 LTS 版)"
    echo "  3) Java 21           (企业生产通用 LTS 版)"
    echo "  4) Java 17           (经典稳定 LTS 版)"
    echo "  5) Java 8            (历史遗留兼容版)"
    echo "  6) 🔍 浏览远程可用版本列表 / 自定义发行版 (如 corretto / temurin)"
    echo "  0) 返回主菜单"
    echo -e "${CYAN}------------------------------------------------------${NC}"
    read -r -p "请输入选择 [0-6]: " sub_choice <&3 || exit 0
    case "$sub_choice" in
      1) install_runtime "java" "lts" "Java JDK"; break ;;
      2) install_runtime "java" "25" "Java JDK"; break ;;
      3) install_runtime "java" "21" "Java JDK"; break ;;
      4) install_runtime "java" "17" "Java JDK"; break ;;
      5) install_runtime "java" "8"  "Java JDK"; break ;;
      6) browse_remote_versions "java" "Java JDK"; break ;;
      0) break ;;
      *) echo "无效选择，请重新输入"; sleep 1 ;;
    esac
  done
}

menu_rust() {
  while true; do
    clear
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo -e "${BOLD}  ⚡ Rust 语言与 Cargo 开发环境安装${NC}"
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo "  1) Rust latest / stable (🌟 官方最新稳定版工具链)"
    echo "  2) Rust nightly         (每日构建尝鲜版)"
    echo "  3) 🔍 浏览远程可用版本列表 / 自定义输入版本号"
    echo "  0) 返回主菜单"
    echo -e "${CYAN}------------------------------------------------------${NC}"
    read -r -p "请输入选择 [0-3]: " sub_choice <&3 || exit 0
    case "$sub_choice" in
      1) install_runtime "rust" "latest" "Rust"; break ;;
      2) install_runtime "rust" "nightly" "Rust"; break ;;
      3) browse_remote_versions "rust" "Rust"; break ;;
      0) break ;;
      *) echo "无效选择，请重新输入"; sleep 1 ;;
    esac
  done
}

install_all_defaults() {
  log_step "🚀 开始一键安装全套最新常用开发环境 (Node.js + Python + Go + Java)"
  echo "采用智能语义标签，自动拉取官方最新 LTS 与稳定版："
  echo "  • Node.js -> @lts"
  echo "  • Python  -> @latest"
  echo "  • Go      -> @latest"
  echo "  • Java    -> @lts"
  echo ""
  
  "$MISE_BIN" use -g node@lts python@latest go@latest java@lts
  "$MISE_BIN" reshim

  log_succ "全套常用开发环境部署完成！"
  echo ""
  echo ">> 验证环境状态:"
  echo -n "  • Node.js: " && (node -v 2>/dev/null || echo "未就绪")
  echo -n "  • Python:  " && (python3 --version 2>/dev/null || python --version 2>/dev/null || echo "未就绪")
  echo -n "  • Go:      " && (go version 2>/dev/null || echo "未就绪")
  echo -n "  • Java:    " && (java -version 2>&1 | head -n 1 || echo "未就绪")

  press_enter
}

# ==============================================================================
# 主菜单循环
# ==============================================================================
main_menu() {
  while true; do
    clear
    local mise_ver
    mise_ver="$("$MISE_BIN" --version 2>/dev/null | head -n 1 || echo '未就绪')"
    local os_info
    os_info="$(uname -s) ($(uname -m))"

    echo -e "${BOLD}${BLUE}"
    echo "  ______            _   _       _     "
    echo " |  ____|          | | | |     | |    "
    echo " | |__   _ ____   _| |_| |_   _| |__  "
    echo " |  __| | '_ \\ \\ / /  _  | | | | '_ \\ "
    echo " | |____| | | \\ V /| | | | |_| | |_) |"
    echo " |______|_| |_|\\_/ |_| |_|\\__,_|_.__/ "
    echo -e "${NC}"
    echo -e "${BOLD}⚡ EnvHub 跨平台开发环境交互式向导 (macOS / Linux)${NC}"
    echo -e "${CYAN}----------------------------------------------------------------------${NC}"
    echo -e " 系统架构: ${BOLD}${os_info}${NC} | 包管理器: ${BOLD}${PKG_MGR:-检测中}${NC} | Mise: ${GREEN}${mise_ver}${NC}"
    echo -e "${CYAN}======================================================================${NC}"
    echo -e " ${BOLD}[常用编程语言与运行环境 - 动态解析最新版]${NC}"
    echo "   1) Node.js   (JavaScript / TypeScript 运行环境与 NPM)"
    echo "   2) Python    (Python3 科学计算、AI 与后端开发环境)"
    echo "   3) Go        (Golang 高性能云原生开发环境)"
    echo "   4) Java      (Java JDK 企业级通用运行环境)"
    echo "   5) Rust      (Rust 语言编译器与 Cargo 包管理)"
    echo "   6) 🚀 一键安装全部常用开发环境 (Node.js LTS + Python + Go + Java)"
    echo ""
    echo -e " ${BOLD}[系统工具、镜像与诊断]${NC}"
    echo "   7) ⚡ 一键配置国内高速镜像加速 (NPM, PyPI, Go Proxy, Cargo)"
    echo "   8) 📋 查看当前已安装的运行时版本列表与状态"
    echo "   9) 🔄 刷新与同步全局 Shims (mise reshim)"
    echo ""
    echo "   0) 🚪 退出脚本 (Exit)"
    echo -e "${CYAN}======================================================================${NC}"
    read -r -p "请输入对应的功能数字 [0-9]: " main_choice <&3 || exit 0

    case "$main_choice" in
      1) menu_node ;;
      2) menu_python ;;
      3) menu_go ;;
      4) menu_java ;;
      5) menu_rust ;;
      6) install_all_defaults ;;
      7) setup_mirrors ;;
      8) show_runtime_status ;;
      9)
        "$MISE_BIN" reshim
        log_succ "已成功刷新全局 Shims 软链接！"
        press_enter
        ;;
      0)
        echo ""
        log_succ "感谢使用 EnvHub 交互式向导，再见！"
        echo ""
        exit 0
        ;;
      *)
        echo -e "${RED}无效选项，请输入 0 到 9 之间的数字${NC}"
        sleep 1
        ;;
    esac
  done
}

# --- 执行入口 ---
ensure_mise
ensure_package_manager
main_menu
