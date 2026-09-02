开发一个跨平台（Win/Mac/Linux）的环境版本管理桌面客户端，最稳妥高效的方案是“桌面 UI 壳层 + 底层成熟 CLI 引擎”，避免从零实现每个语言的下载、解压、编译与 Path 劫持。

---

### 技术栈选型

* **桌面框架：Tauri (Rust + 前端)**
* 相比 Electron 内存占用极小（~30-50MB vs 150MB+），安装包体积小（~10MB）。
* 拥有原生的系统级 API 权限，方便调用子进程、处理文件系统与修改环境变量。


* **前端界面：Vue 3 / React + Tailwind CSS + Shadcn-ui**
* 现代化的状态管理与仪表盘 UI 库，便于展示版本列表、下载进度条与实时终端日志。


* **底层核心：集成 `mise` 或 `proto` CLI**
* 将 `mise` 或 `proto` 二进制文件作为 Sidecar 打包进应用，或检测系统已安装的二进制文件，通过 Rust 后端调用 CLI 命令获取输出并解析 JSON。



---

### 系统核心架构

```
+-------------------------------------------------------------+
|                     Tauri Frontend (UI)                     |
|  - 版本切换列表       - 在线版本仓库       - 环境变量管理面板   |
+------------------------------+------------------------------+
                               | IPC (Invoke / Event)
+------------------------------v------------------------------+
|                      Rust Backend (Core)                    |
|  - Process Runner: 执行 mise / winget / brew / apt 命令     |
|  - Path Manager: 跨平台修改系统 PATH / Shell RC 文件          |
|  - Downloader: 监听下载流并向前端推送进度事件 (SSE / Events) |
+------------------------------+------------------------------+
                               | Subprocess Execution
+------------------------------v------------------------------+
|                        底层环境与引擎                         |
|  [mise / proto] (语言管理)  |  [winget / brew / apt] (系统工具)
+-------------------------------------------------------------+

```

---

### 核心功能模块与技术实现

**1. 版本检测与列表拉取**

* **技术实现**：Tauri Rust 后端通过 `std::process::Command` 调用底层工具：
* 列出已安装：`mise ls --json`
* 列出远程可用：`mise ls-remote <tool> --json`


* **处理逻辑**：Rust 解析 JSON 输出后，直接返回结构化数据给前端表格渲染，支持按 LTS、最新版、已安装状态过滤。

**2. 在线安装与异步进度反馈**

* **难点**：下载大型运行时（如 Python 源码编译或 Node 预编译包）耗时较长，不能阻塞 UI。
* **技术实现**：
* Rust 后端开启异步子进程（`tokio::process::Command`），捕获 `stdout` 与 `stderr` 流。
* 利用正则解析日志中的下载百分比，通过 `app_handle.emit_all("download-progress", percent)` 向前端发送事件，实时驱动 UI 进度条。



**3. 版本切换与环境劫持**

* **项目级（局部生效）**：
* 用户在 UI 中选定某个目录，点击“应用版本”，程序在目标目录下写入 `.mise.toml` 或 `.tool-versions` 文件。


* **系统全局切换（全局生效）**：
* **Windows**：调用 Win32 API（修改注册表 `HKCU\Environment\Path`），广播 `WM_SETTINGCHANGE` 消息使新 PATH 立即生效。
* **macOS / Linux**：检查用户默认 Shell（Bash、Zsh、Fish），向 `~/.zshrc` 或 `~/.bashrc` 写入 shims 路径激活代码。



**4. 基础系统工具管理（Git 等）**

* **平台路由适配器**：
* 抽象统一的 `SystemPackageManager` 接口。
* Windows 端路由到 `winget install Git.Git`。
* macOS 端检测并路由到 `brew install git`。
* Linux 端检测发行版类型后路由到 `apt / pacman / dnf`（需要弹出系统鉴权提权窗口）。



---

### 关键挑战与避坑指南

* **Shell 环境变量继承问题**：在 macOS/Linux 上，双击桌面图标启动的 GUI 进程**不会加载** `.zshrc` 中的 PATH。需在 Rust 启动时通过伪终端模拟 `zsh -l -c "env"` 读取完整的系统环境。
* **权限控制**：Windows 注册表修改仅需当前用户权限（HKCU），尽量避免强制要求 Administrator 权限；Linux 下调用系统包管理器时，需调用 `pkexec` 弹出系统原生授权对话框。
* **网络加速与镜像源**：国内用户下载 Node/Python/Go 经常遇到超时，建议在设置中内置 NPM 淘宝源、Python 豆瓣/清华源的切换开关，自动写入底层配置。
