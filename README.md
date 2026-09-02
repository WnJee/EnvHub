<div align="center">

# ⚡ EnvHub

**跨平台开发环境与多语言版本管理桌面客户端**

*A modern, blazing-fast, cross-platform development environment and runtime version manager.*

<p align="center">
  <a href="./README.md">简体中文</a> | <a href="./README_EN.md">English</a>
</p>

[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2.0-24C8D8?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.80+-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18-61DAFB?style=flat-square&logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.7-3178C6?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![TailwindCSS](https://img.shields.io/badge/TailwindCSS-3.4-38B2AC?style=flat-square&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)
[![Engine](https://img.shields.io/badge/Core%20Engine-Mise%20CLI-FF5F00?style=flat-square)](https://mise.jdx.dev/)

</div>

---

## 📖 项目介绍

**EnvHub** 是一款专为开发者打造的跨平台多语言环境版本管理桌面工具。它摒弃了 Electron 的笨重，基于 **Tauri v2 (Rust) + React 18 + Tailwind CSS** 构建，底层无缝集成成熟的高性能 **Mise CLI** 引擎。

EnvHub 实现了**极低资源占用（常驻内存 ~30-50MB，安装包 ~10MB）**，并在图形化界面下带来了即时切换全局/项目局部版本、国内镜像源一键测速与加速、系统级包管理以及终端环境变量自动同步等强大功能。

---

## ✨ 核心特性

- 🚀 **全语言生态运行时管理**：支持 Node.js、Python、Go、Rust、Java (OpenJDK/Temurin)、Bun、Deno、Ruby、PHP 等语言的多版本下载、共存与一键切换。
- 📦 **项目级环境精准隔离**：自动识别并维护项目根目录下的 `.mise.toml`、`.tool-versions` 或 `package.json`，进入目录即刻生效，无需污染全局系统。
- ⚡ **国内高速镜像源测速与切换**：内置 NPM (淘宝/腾讯)、Python Pip (清华/阿里/豆瓣)、Go Modules (Goproxy.cn)、Cargo (字节 rsproxy/中科大)、Homebrew 等国内 CDN 镜像源，支持实时节点测速并一键写入系统配置。
- 🛠️ **系统工具箱与跨平台包管理**：抽象统一接口，macOS 路由至 `brew`，Windows 路由至 `winget`，Linux 路由至 `apt/pacman`，一键安装 Git、Docker CLI、CMake、Neovim 等。
- 🔍 **环境健康体检与 Shell 修复**：解决 macOS/Linux 下 GUI 桌面应用无法继承终端 `~/.zshrc` PATH 的痛点，提供一键自动注入与 Shims 诊断。
- 💻 **流式异步微终端**：Rust 异步子进程实时捕获编译、下载与解压进度，向前端推流日志，UI 始终丝滑响应。

---

## 🏗️ 系统核心架构

```
+-------------------------------------------------------------------------+
|                        EnvHub Desktop UI (React 18 + TS + Tailwind)     |
|  - 语言运行时面板 (Node, Python, Go, Rust, Java, Bun, Deno, etc.)       |
|  - 项目目录管理 (自动探测 .mise.toml / .node-version 并一键绑定版本)     |
|  - 系统工具箱 (Git, Docker, CMake, Neovim 等 brew/winget/apt 管理)      |
|  - 镜像源一键测速与加速 (NPM / Pip / Go / Cargo / Brew)                 |
|  - 环境健康自检面板 (Shell RC 注入检测 / Shims 诊断与一键修复)           |
|  - 实时终端安装抽屉 (实时输出日志流与安装进度)                           |
+------------------------------------+------------------------------------+
                                     | Tauri IPC (invoke & event)
+------------------------------------v------------------------------------+
|                         Tauri 2.0 Rust Core Backend                     |
|  - env_helper: 跨平台环境变量继承与修复                                 |
|  - mise_service: mise CLI 探测、自举安装、ls、install、use、uninstall  |
|  - project_service: 递归扫描工程目录与解析/写入 mise 配置文件           |
|  - system_service: 检测操作系统与包管理器 (brew/winget/apt)             |
|  - mirror_service: 读取/修改各类语言工具的镜像源配置文件与测速          |
+------------------------------------+------------------------------------+
                                     | Subprocess Execution
+------------------------------------v------------------------------------+
|                             底层操作系统与 CLI                          |
|         [ mise CLI ]        |   [ brew / winget / apt ]                 |
+-------------------------------------------------------------------------+
```

---

## 🚀 快速开始

### 依赖环境
- [Node.js](https://nodejs.org/) (>= 18.0.0)
- [Rust & Cargo](https://www.rust-lang.org/) (>= 1.75.0)

### 1. 克隆代码仓库
```bash
git clone git@github.com:WnJee/EnvHub.git
cd EnvHub
```

### 2. 安装前端依赖
```bash
npm install
```

### 3. 网页端快速预览开发 (含智能 Mock 模式)
```bash
npm run dev
```
打开浏览器访问 `http://localhost:1420` 即可实时体验与交互。

### 4. 启动 Tauri 桌面端开发环境
```bash
npm run tauri dev
```

### 5. 编译与打包桌面客户端安装包
```bash
# 生成系统原生安装包 (macOS .dmg/.app, Windows .msi/.exe, Linux .deb/.AppImage)
npm run tauri build
```

---

## 📁 目录结构

```
├── src/                        # 前端 React 18 源码
│   ├── components/             # 核心 UI 组件
│   │   ├── Sidebar.tsx         # 侧边栏导航组件
│   │   ├── Header.tsx          # 顶部状态、搜索与系统标识
│   │   ├── RuntimeManager.tsx  # 语言运行时管理主面板
│   │   ├── ProjectManager.tsx  # 项目工程环境隔离面板
│   │   ├── SystemTools.tsx     # 系统级 CLI 工具箱
│   │   ├── MirrorManager.tsx   # 镜像源加速与多节点测速
│   │   ├── EnvHealth.tsx       # 环境健康自检与 Shell 修复
│   │   ├── InstallModal.tsx    # 异步安装日志终端抽屉
│   │   └── SettingsModal.tsx   # 引擎配置与自举安装
│   ├── services/
│   │   └── tauri.ts            # Tauri IPC 与浏览器端 Mock 服务
│   ├── types/                  # TypeScript 类型定义
│   ├── App.tsx                 # 主页面布局与状态流转
│   ├── index.css               # Tailwind CSS & 深色极客设计
│   └── main.tsx                # 前端挂载入口
├── src-tauri/                  # 后端 Rust (Tauri v2)
│   ├── src/
│   │   ├── commands/           # 模块化 Tauri IPC 指令
│   │   │   ├── mise.rs         # Mise CLI 交互、自举与安装管道
│   │   │   ├── system.rs       # 宿主系统状态与原生包管理器
│   │   │   ├── mirrors.rs      # 镜像源读写与测速指令
│   │   │   └── projects.rs     # 本地工程目录版本配置
│   │   ├── env_helper.rs       # 跨平台环境变量继承与修复
│   │   ├── lib.rs              # 核心应用装配与命令注册
│   │   └── main.rs             # 二进制启动入口
│   ├── Cargo.toml              # Rust 依赖声明
│   └── tauri.conf.json         # Tauri 桌面窗口与权限配置
├── package.json
└── README.md
```

---

## 🤝 参与贡献

欢迎提交 Issue 和 Pull Request！
1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送至分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📄 开源许可证

本项目基于 [MIT License](LICENSE) 开源。
