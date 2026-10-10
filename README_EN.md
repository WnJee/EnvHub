<div align="center">

# ⚡ EnvHub

**A modern, blazing-fast, cross-platform development environment and runtime version manager.**

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

## 📖 Introduction

**EnvHub** is a cross-platform desktop application designed for developers to manage multi-language runtimes and development environments. Built on top of **Tauri v2 (Rust) + React 18 + Tailwind CSS**, EnvHub seamlessly integrates the powerful, battle-tested **Mise CLI** engine under the hood.

Unlike resource-heavy Electron apps, EnvHub achieves **minimal resource footprint (~30-50MB RAM, ~10MB package size)** while delivering instant global/local version switching, domestic mirror speedup & latency benchmarking, system package management, and automatic shell environment synchronization.

---

## ✨ Features

- 🚀 **Full-Stack Runtime Version Management**: Easily install, coexist, and switch versions for Node.js, Python, Go, Rust, Java, Bun, Deno, Ruby, PHP, and more.
- 📦 **Project-Level Environment Isolation**: Automatically detect and manage `.mise.toml`, `.tool-versions`, or `package.json` in project directories for zero-friction version switching on `cd`.
- ⚡ **Mirror Acceleration & Latency Benchmarks**: Built-in mirror sources for NPM, Python Pip, Go Modules (GOPROXY), Cargo (Rust Crates), and Homebrew with real-time ping testing and 1-click configuration persistence.
- 🛠️ **System Developer Toolbox**: Unified abstraction over platform package managers (`brew` on macOS, `winget` on Windows, `apt/pacman` on Linux) to install Git, Docker CLI, CMake, Neovim, etc.
- 🔍 **Environment Health Check & Shell Fixes**: Automatically detect Shell RC files (`~/.zshrc`, `~/.bashrc`) and fix PATH inheritance issues for GUI applications on macOS/Linux with 1-click auto-repair.
- 💻 **Streaming Mini-Terminal**: Tokio asynchronous subprocess output capture with real-time log streaming and animated progress indicators.

---

## ⚡ Quick Start: Cross-Platform Interactive CLI Wizard

> If you don't need or haven't installed the desktop client yet, you can directly use our **cross-platform interactive terminal setup wizard**.
> It automatically checks and installs **Mise**, prompts and sets up host package managers (macOS **Homebrew** / Windows **Scoop**), and provides a clean **numeric menu** to install runtimes and configure high-speed mirrors with 1-click.

### 1. Download & Run in Terminal

Run directly without manual git cloning (downloads script and starts interactive wizard):

- **🍎 macOS / 🐧 Linux**:
  ```bash
  curl -fsSL https://raw.githubusercontent.com/WnJee/EnvHub/main/scripts/setup_interactive.sh -o setup_interactive.sh && bash setup_interactive.sh
  ```

- **🪟 Windows (PowerShell)**:
  ```powershell
  irm https://raw.githubusercontent.com/WnJee/EnvHub/main/scripts/setup_interactive.ps1 -OutFile setup_interactive.ps1; powershell -ExecutionPolicy Bypass -File .\setup_interactive.ps1
  ```

---

### 2. Local Download / Cloned Execution

If you have cloned or downloaded the repository:

- **macOS / Linux**:
  ```bash
  bash scripts/setup_interactive.sh
  # or make executable and run:
  chmod +x scripts/setup_interactive.sh && ./scripts/setup_interactive.sh
  ```
- **Windows**:
  - **Option 1 (Recommended, double-click)**: Run [`scripts/setup_interactive.bat`](./scripts/setup_interactive.bat);
  - **Option 2 (PowerShell)**:
    ```powershell
    powershell -ExecutionPolicy Bypass -File scripts\setup_interactive.ps1
    ```

---

### 3. Interactive Terminal Preview

```text
  ______            _   _       _     
 |  ____|          | | | |     | |    
 | |__   _ ____   _| |_| |_   _| |__  
 |  __| | '_ \ \ / /  _  | | | | '_ \ 
 | |____| | | \ V /| | | | |_| | |_) |
 |______|_| |_|\_/ |_| |_|\__,_|_.__/ 
⚡ EnvHub Cross-Platform Interactive Wizard
----------------------------------------------------------------------
 OS: Darwin (arm64) | Package Manager: Homebrew (Ready) | Mise: v2024.x.x
======================================================================
 [Common Runtimes & Languages]
   1) Node.js   (JavaScript / TypeScript Runtime & NPM)
   2) Python    (Python3 Data Science, AI & Backend)
   3) Go        (Golang Cloud-Native High Performance Runtime)
   4) Java      (Java JDK Enterprise General Runtime)
   5) Rust      (Rust Toolchain & Cargo Package Manager)
   6) 🚀 1-Click Install All Common Runtimes (Node + Python + Go + Java)

 [System Tools, Mirrors & Diagnostics]
   7) ⚡ 1-Click Fast Mirror Acceleration (NPM, PyPI, Go Proxy, Cargo)
   8) 📋 View Installed Runtime Versions & Status
   9) 🔄 Reshim Global Binary Links (mise reshim)

   0) 🚪 Exit Wizard
======================================================================
Please select an option [0-9]: 
```

---

## 📸 Screenshots

### 1. Multi-Language Runtime Version Manager
> Manage, download, and switch multiple versions of Node.js, Python, Go, Rust, Java, Bun, Deno, Ruby, PHP, and more. Support 1-click cross-platform environment export script.

<div align="center">
  <img src="docs/images/runtimes.png" alt="Multi-Language Runtime Version Manager" width="90%" />
</div>

### 2. System CLI & Developer Toolbox
> Unified integration with host package managers (macOS/Linux Homebrew, Windows Scoop / Winget) to install, run, and test Git, Docker CLI, Docker Compose, Nginx, Redis, Neovim, etc.

<div align="center">
  <img src="docs/images/system-tools.png" alt="System Developer Toolbox" width="90%" />
</div>

### 3. Mirror Acceleration & Speed Benchmarks
> Built-in mirrors for NPM, Pip, Go Modules, Cargo, Docker Hub, and Homebrew with real-time ping latency testing and 1-click config persistence.

<div align="center">
  <img src="docs/images/mirrors.png" alt="Mirror Acceleration & Speed Benchmarks" width="90%" />
</div>

### 4. Environment Health Diagnostics & Shell Auto-Repair
> Diagnose terminal PATH inheritance, Shell RC (`~/.zshrc` / `~/.bashrc`) activation, and Shims priority, with 1-click automatic repair.

<div align="center">
  <img src="docs/images/health-check.png" alt="Environment Health Diagnostics & Shell Auto-Repair" width="90%" />
</div>

### 5. Project-Level Environment Isolation
> Automatically scan and bind runtime versions to specific project workspaces (`.mise.toml`, `.tool-versions`), isolating project environments without polluting global setups.

<div align="center">
  <img src="docs/images/project-isolation.png" alt="Project Environment Isolation" width="90%" />
</div>

---

## 🏗️ Architecture

```
+-------------------------------------------------------------------------+
|                        EnvHub Desktop UI (React 18 + TS + Tailwind)     |
|  - Runtime Manager (Node, Python, Go, Rust, Java, Bun, Deno, etc.)       |
|  - Project Isolator (Auto-detect .mise.toml / .node-version & bind)     |
|  - System CLI Toolbox (Git, Docker, CMake via brew/winget/apt)          |
|  - Mirror Manager & Speedup (NPM / Pip / Go / Cargo / Brew)             |
|  - Environment Health Check (Shell RC injection & Shims diagnostic)     |
|  - Real-time Streaming Terminal Drawer (Live stdout/stderr stream)      |
+------------------------------------+------------------------------------+
                                     | Tauri IPC (invoke & event)
+------------------------------------v------------------------------------+
|                         Tauri 2.0 Rust Core Backend                     |
|  - env_helper: Cross-platform PATH inheritance & repair                 |
|  - mise_service: CLI detection, bootstrap, ls, install, use, uninstall  |
|  - project_service: Recursive directory scanner & config reader/writer  |
|  - system_service: OS detection & package manager routing               |
|  - mirror_service: Configuration file modifier & ping benchmarks        |
+------------------------------------+------------------------------------+
                                     | Subprocess Execution
+------------------------------------v------------------------------------+
|                             Underlying OS & CLI Tools                   |
|         [ mise CLI ]        |   [ brew / winget / apt ]                 |
+-------------------------------------------------------------------------+
```

---

## 🚀 Getting Started

### Prerequisites
- [Node.js](https://nodejs.org/) (>= 18.0.0)
- [Rust & Cargo](https://www.rust-lang.org/) (>= 1.75.0)

### 1. Clone the repository
```bash
git clone git@github.com:WnJee/EnvHub.git
cd EnvHub
```

### 2. Install dependencies
```bash
npm install
```

### 3. Web preview development mode (with realistic Mock service)
```bash
npm run dev
```
Open your browser at `http://localhost:1420` to explore the interface.

### 4. Run Tauri desktop application
```bash
npm run tauri dev
```

### 5. Build production installers
```bash
# Generates native installers (.dmg/.app for macOS, .msi/.exe for Windows, .deb/.AppImage for Linux)
npm run tauri build
```

---

## 📁 Directory Structure

```
├── src/                        # Frontend React 18 source
│   ├── components/             # Core UI components
│   │   ├── Sidebar.tsx         # Navigation sidebar
│   │   ├── Header.tsx          # Header with status & search
│   │   ├── RuntimeManager.tsx  # Multi-runtime management
│   │   ├── ProjectManager.tsx  # Project environment isolation
│   │   ├── SystemTools.tsx     # System CLI toolbox
│   │   ├── MirrorManager.tsx   # Mirror speedup & latency tester
│   │   ├── EnvHealth.tsx       # Shell health & PATH diagnostics
│   │   ├── InstallModal.tsx    # Live streaming install terminal
│   │   └── SettingsModal.tsx   # Engine settings & bootstrap
│   ├── services/
│   │   └── tauri.ts            # Tauri IPC bridge & Mock fallbacks
│   ├── types/                  # TypeScript interfaces
│   ├── App.tsx                 # Root layout & state coordinator
│   ├── index.css               # Tailwind CSS & developer dark theme
│   └── main.tsx                # App entrypoint
├── src-tauri/                  # Rust Tauri Backend (Tauri v2)
│   ├── src/
│   │   ├── commands/           # Modular Tauri IPC commands
│   │   │   ├── mise.rs         # Mise CLI interaction & streaming
│   │   │   ├── system.rs       # System environment & package managers
│   │   │   ├── mirrors.rs      # Mirror config parser & benchmarks
│   │   │   └── projects.rs     # Local project directory configs
│   │   ├── env_helper.rs       # Cross-platform environment resolver
│   │   ├── lib.rs              # App builder & command registration
│   │   └── main.rs             # Binary entrypoint
│   ├── Cargo.toml              # Rust crate dependencies
│   └── tauri.conf.json         # Tauri window & bundle configuration
├── package.json
├── README.md                   # Chinese Documentation
└── README_EN.md                # English Documentation
```

---

## 🤝 Contributing

Contributions are always welcome!
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to your branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
