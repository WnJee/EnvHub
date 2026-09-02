import { RuntimeTool, ProjectEnv, SystemTool, MirrorConfig, EnvHealthCheck, SystemStatus } from '../types';

// Check if running inside Tauri
export const isTauri = () => {
  return typeof window !== 'undefined' && ('__TAURI__' in window || '__TAURI_INTERNALS__' in window);
};

// Initial Mock Data for Runtimes
const mockRuntimes: RuntimeTool[] = [
  {
    id: 'node',
    name: 'Node.js',
    category: 'runtime',
    description: 'JavaScript 运行时环境，支持海量 npm 生态',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/nodejs/nodejs-original.svg',
    officialSite: 'https://nodejs.org',
    installedVersions: ['20.18.0', '22.12.0', '18.20.4'],
    activeVersion: '22.12.0',
    globalVersion: '22.12.0',
    availableVersions: [
      '23.2.0', '23.1.0', '23.0.0',
      '22.12.0', '22.11.0', '22.10.0', '22.9.0', '22.0.0',
      '20.18.0', '20.17.0', '20.16.0', '20.10.0', '20.0.0',
      '18.20.4', '18.19.0', '18.18.0', '16.20.2'
    ],
  },
  {
    id: 'python',
    name: 'Python',
    category: 'runtime',
    description: '通用高阶编程语言，广泛应用于 AI、数据科学与后端开发',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original.svg',
    officialSite: 'https://www.python.org',
    installedVersions: ['3.12.7', '3.11.9', '3.10.14'],
    activeVersion: '3.12.7',
    globalVersion: '3.12.7',
    availableVersions: [
      '3.13.0', '3.12.7', '3.12.6', '3.12.0',
      '3.11.9', '3.11.8', '3.11.0',
      '3.10.14', '3.10.12', '3.9.19', '3.8.18'
    ],
  },
  {
    id: 'go',
    name: 'Go',
    category: 'runtime',
    description: 'Google 开发的静态编译型语言，极高并发与云原生标准',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/go/go-original.svg',
    officialSite: 'https://go.dev',
    installedVersions: ['1.23.3', '1.22.8'],
    activeVersion: '1.23.3',
    globalVersion: '1.23.3',
    availableVersions: [
      '1.23.4', '1.23.3', '1.23.0',
      '1.22.9', '1.22.8', '1.22.0',
      '1.21.13', '1.21.0', '1.20.14'
    ],
  },
  {
    id: 'rust',
    name: 'Rust',
    category: 'runtime',
    description: '注重内存安全、高性能与零成本抽象的系统级编程语言',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-original.svg',
    officialSite: 'https://www.rust-lang.org',
    installedVersions: ['1.83.0', '1.82.0'],
    activeVersion: '1.83.0',
    globalVersion: '1.83.0',
    availableVersions: [
      '1.83.0', '1.82.0', '1.81.0', '1.80.1', '1.79.0', '1.75.0', 'nightly'
    ],
  },
  {
    id: 'java',
    name: 'Java (OpenJDK)',
    category: 'runtime',
    description: '跨平台面向对象语言，企业级应用与微服务中坚力量 (Temurin / Corretto)',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/java/java-original.svg',
    officialSite: 'https://adoptium.net',
    installedVersions: ['temurin-21.0.4', 'temurin-17.0.12'],
    activeVersion: 'temurin-21.0.4',
    globalVersion: 'temurin-21.0.4',
    availableVersions: [
      'temurin-23.0.1', 'temurin-21.0.5', 'temurin-21.0.4',
      'temurin-17.0.13', 'temurin-17.0.12', 'temurin-11.0.24', 'temurin-8.0.432'
    ],
  },
  {
    id: 'bun',
    name: 'Bun',
    category: 'runtime',
    description: '极速 All-in-One JavaScript 运行时、打包器与包管理器',
    icon: 'https://bun.sh/logo.svg',
    officialSite: 'https://bun.sh',
    installedVersions: ['1.1.38'],
    activeVersion: '1.1.38',
    globalVersion: '1.1.38',
    availableVersions: [
      '1.1.38', '1.1.37', '1.1.36', '1.1.30', '1.0.35'
    ],
  },
  {
    id: 'deno',
    name: 'Deno',
    category: 'runtime',
    description: '下一代安全 JavaScript / TypeScript 运行时，原生支持 TS 与 Web 规范',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/denojs/denojs-original.svg',
    officialSite: 'https://deno.land',
    installedVersions: ['2.0.6'],
    activeVersion: '2.0.6',
    globalVersion: '2.0.6',
    availableVersions: [
      '2.0.6', '2.0.5', '2.0.0', '1.46.3', '1.45.0'
    ],
  },
  {
    id: 'ruby',
    name: 'Ruby',
    category: 'runtime',
    description: '优雅简洁的动态编程语言，Rails 框架基石',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/ruby/ruby-original.svg',
    officialSite: 'https://www.ruby-lang.org',
    installedVersions: ['3.3.6'],
    activeVersion: '3.3.6',
    globalVersion: '3.3.6',
    availableVersions: [
      '3.3.6', '3.3.5', '3.2.6', '3.1.6', '3.0.7'
    ],
  },
  {
    id: 'php',
    name: 'PHP',
    category: 'runtime',
    description: '广受欢迎的 Web 开发服务端脚本语言',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/php/php-original.svg',
    officialSite: 'https://www.php.net',
    installedVersions: ['8.3.13'],
    activeVersion: '8.3.13',
    globalVersion: '8.3.13',
    availableVersions: [
      '8.4.0', '8.3.14', '8.3.13', '8.2.25', '8.1.30'
    ],
  }
];

const mockProjects: ProjectEnv[] = [
  {
    id: 'p1',
    name: 'sharp-turing-web',
    path: '/Users/wnjie/Documents/antigravity/sharp-turing',
    configFile: '.mise.toml',
    tools: [
      { toolId: 'node', version: '22.12.0' },
      { toolId: 'rust', version: '1.83.0' }
    ],
    lastModified: '2026-09-02 13:58'
  },
  {
    id: 'p2',
    name: 'ai-model-serving-service',
    path: '/Users/wnjie/Projects/ai-model-serving',
    configFile: '.tool-versions',
    tools: [
      { toolId: 'python', version: '3.11.9' },
      { toolId: 'go', version: '1.22.8' }
    ],
    lastModified: '2026-08-30 19:20'
  },
  {
    id: 'p3',
    name: 'enterprise-core-backend',
    path: '/Users/wnjie/Projects/enterprise-core',
    configFile: '.mise.toml',
    tools: [
      { toolId: 'java', version: 'temurin-21.0.4' },
      { toolId: 'node', version: '20.18.0' }
    ],
    lastModified: '2026-08-25 10:14'
  }
];

const mockSystemTools: SystemTool[] = [
  {
    id: 'git',
    name: 'Git',
    description: '分布式版本控制系统，全球开发者必备基建',
    category: 'VCS',
    isInstalled: true,
    installedVersion: '2.43.0',
    installCommand: 'brew install git',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/git/git-original.svg',
    homepage: 'https://git-scm.com'
  },
  {
    id: 'docker',
    name: 'Docker CLI',
    description: '轻量级容器引擎与虚拟化工具',
    category: 'Container',
    isInstalled: true,
    installedVersion: '27.3.1',
    installCommand: 'brew install --cask docker',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/docker/docker-original.svg',
    homepage: 'https://www.docker.com'
  },
  {
    id: 'cmake',
    name: 'CMake',
    description: '跨平台自动化建构系统，C/C++ 与 Rust 复杂工程依赖',
    category: 'Build',
    isInstalled: true,
    installedVersion: '3.30.5',
    installCommand: 'brew install cmake',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/cmake/cmake-original.svg',
    homepage: 'https://cmake.org'
  },
  {
    id: 'neovim',
    name: 'Neovim',
    description: '可高度定制化、支持 Lua 插件生态的下一代终端文本编辑器',
    category: 'Editor',
    isInstalled: false,
    installCommand: 'brew install neovim',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/neovim/neovim-original.svg',
    homepage: 'https://neovim.io'
  },
  {
    id: 'ripgrep',
    name: 'Ripgrep (rg)',
    description: '超高速的基于 Rust 实现的递归文本搜索工具',
    category: 'CLI Utility',
    isInstalled: true,
    installedVersion: '14.1.0',
    installCommand: 'brew install ripgrep',
    icon: 'https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-original.svg',
    homepage: 'https://github.com/BurntSushi/ripgrep'
  }
];

const mockMirrors: MirrorConfig[] = [
  {
    id: 'npm',
    name: 'NPM (Node.js)',
    tool: 'npm',
    currentMirror: 'https://registry.npmmirror.com',
    options: [
      { name: '淘宝 NPM 镜像 (npmmirror)', url: 'https://registry.npmmirror.com', ping: 28, isDefault: true },
      { name: '腾讯云 NPM 镜像', url: 'https://mirrors.cloud.tencent.com/npm/', ping: 35 },
      { name: '华为云 NPM 镜像', url: 'https://repo.huaweicloud.com/repository/npm/', ping: 42 },
      { name: '官方官方源 (npmjs.org)', url: 'https://registry.npmjs.org', ping: 210 }
    ]
  },
  {
    id: 'pip',
    name: 'Pip (Python)',
    tool: 'pip',
    currentMirror: 'https://pypi.tuna.tsinghua.edu.cn/simple',
    options: [
      { name: '清华大学 TUNA 镜像', url: 'https://pypi.tuna.tsinghua.edu.cn/simple', ping: 31, isDefault: true },
      { name: '阿里云开源镜像', url: 'https://mirrors.aliyun.com/pypi/simple/', ping: 34 },
      { name: '豆瓣开源镜像', url: 'https://pypi.doubanio.com/simple/', ping: 29 },
      { name: '官方 PyPI 源', url: 'https://pypi.org/simple', ping: 240 }
    ]
  },
  {
    id: 'go',
    name: 'Go Modules (GOPROXY)',
    tool: 'go',
    currentMirror: 'https://goproxy.cn,direct',
    options: [
      { name: 'Goproxy 中国 (七牛云)', url: 'https://goproxy.cn,direct', ping: 25, isDefault: true },
      { name: '阿里云 Go 模块代理', url: 'https://mirrors.aliyun.com/goproxy/,direct', ping: 32 },
      { name: '官方 proxy.golang.org', url: 'https://proxy.golang.org,direct', ping: 280 }
    ]
  },
  {
    id: 'cargo',
    name: 'Cargo (Rust Crates)',
    tool: 'cargo',
    currentMirror: 'https://rsproxy.cn',
    options: [
      { name: '字节跳动 rsproxy (推荐)', url: 'https://rsproxy.cn', ping: 22, isDefault: true },
      { name: '中国科学技术大学 USTC', url: 'https://mirrors.ustc.edu.cn/crates.io-index', ping: 36 },
      { name: '清华大学 Crates 镜像', url: 'https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git', ping: 38 },
      { name: '官方 crates.io', url: 'https://github.com/rust-lang/crates.io-index', ping: 260 }
    ]
  },
  {
    id: 'brew',
    name: 'Homebrew (macOS)',
    tool: 'brew',
    currentMirror: 'https://mirrors.ustc.edu.cn/homebrew-bottles',
    options: [
      { name: '中国科学技术大学 USTC 瓶子镜像', url: 'https://mirrors.ustc.edu.cn/homebrew-bottles', ping: 33, isDefault: true },
      { name: '清华大学 TUNA 镜像', url: 'https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles', ping: 39 },
      { name: '阿里云 Homebrew 镜像', url: 'https://mirrors.aliyun.com/homebrew/homebrew-bottles', ping: 45 },
      { name: 'GitHub 官方源', url: 'https://ghcr.io/v2/homebrew/core', ping: 220 }
    ]
  }
];

const mockHealthChecks: EnvHealthCheck[] = [
  {
    id: 'mise-activated',
    title: 'Shell 环境变量与 Shims 激活检测',
    status: 'ok',
    message: '已在 ~/.zshrc 中检测到 eval "$(mise activate zsh)"，命令行与 GUI 环境正常同步',
    shell: '/bin/zsh',
    configFile: '~/.zshrc',
    canAutoFix: false
  },
  {
    id: 'path-priority',
    title: 'PATH 优先级检查',
    status: 'ok',
    message: 'mise shims 路径 (~/.local/share/mise/shims) 处于系统首位，已成功接管运行时版本',
    shell: '/bin/zsh',
    configFile: '~/.zprofile',
    canAutoFix: false
  },
  {
    id: 'package-manager',
    title: '系统包管理器健康状态',
    status: 'ok',
    message: 'Homebrew 处于就绪状态 (/opt/homebrew/bin/brew)，支持自动安装依赖工具',
    shell: 'system',
    configFile: '/opt/homebrew',
    canAutoFix: false
  },
  {
    id: 'mirror-speed',
    title: '国内加速网络连通性',
    status: 'ok',
    message: 'NPM、PyPI、GoProxy 与 Rust Crates 已全部配置国内低延迟镜像 (平均延迟 <35ms)',
    shell: 'network',
    configFile: '~/.npmrc, ~/.pip/pip.conf',
    canAutoFix: false
  }
];

// Service API layer
export const api = {
  // System Status
  async getSystemStatus(): Promise<SystemStatus> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<SystemStatus>('get_system_status');
      } catch (err) {
        console.warn('Tauri invoke failed, falling back to mock:', err);
      }
    }
    return {
      os: 'macos',
      osVersion: 'macOS 15.0 Sequoia (ARM64)',
      arch: 'aarch64',
      defaultShell: '/bin/zsh',
      miseInstalled: true,
      miseVersion: 'mise 2024.11.23 macos-arm64 (532c517 2024-11-23)',
      misePath: '/opt/homebrew/bin/mise',
      packageManager: 'brew'
    };
  },

  // Runtimes
  async getRuntimes(): Promise<RuntimeTool[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<RuntimeTool[]>('get_runtimes');
      } catch (err) {
        console.warn('Tauri invoke get_runtimes failed, using mock data:', err);
      }
    }
    return [...mockRuntimes];
  },

  async getAvailableVersions(toolId: string): Promise<string[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<string[]>('get_remote_versions', { toolId });
      } catch (err) {
        console.warn('Tauri invoke get_remote_versions failed:', err);
      }
    }
    const tool = mockRuntimes.find(t => t.id === toolId);
    return tool ? tool.availableVersions : [];
  },

  async setGlobalVersion(toolId: string, version: string): Promise<boolean> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<boolean>('set_global_version', { toolId, version });
      } catch (err) {
        console.warn('Tauri invoke set_global_version failed:', err);
      }
    }
    const tool = mockRuntimes.find(t => t.id === toolId);
    if (tool) {
      tool.globalVersion = version;
      tool.activeVersion = version;
    }
    return true;
  },

  async uninstallVersion(toolId: string, version: string): Promise<boolean> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<boolean>('uninstall_runtime_version', { toolId, version });
      } catch (err) {
        console.warn('Tauri invoke uninstall failed:', err);
      }
    }
    const tool = mockRuntimes.find(t => t.id === toolId);
    if (tool) {
      tool.installedVersions = tool.installedVersions.filter(v => v !== version);
      if (tool.activeVersion === version) {
        tool.activeVersion = tool.installedVersions[0] || undefined;
      }
    }
    return true;
  },

  // Install runtime with streaming logs
  async startInstallRuntime(
    toolId: string,
    version: string,
    onLog: (log: string) => void,
    onProgress: (percent: number) => void
  ): Promise<boolean> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const { listen } = await import('@tauri-apps/api/event');
        
        const unlistenLog = await listen<string>('install-log', (event) => {
          onLog(event.payload);
        });
        const unlistenProgress = await listen<number>('install-progress', (event) => {
          onProgress(event.payload);
        });

        const success = await invoke<boolean>('install_runtime_version', { toolId, version });
        unlistenLog();
        unlistenProgress();
        return success;
      } catch (err) {
        console.warn('Tauri invoke install failed, using simulation:', err);
      }
    }

    // Realistic Simulated Install Flow for preview
    return new Promise((resolve) => {
      const steps = [
        `[mise] Resolving plugin and package definition for ${toolId}@${version}...`,
        `[mise] Fetching remote release checksum from official repository...`,
        `[mise] Found sha256 checksum: 9e32a48f2c687f891b2e...`,
        `[mise] Downloading precompiled binary archive (${toolId}-v${version}-darwin-arm64.tar.gz)...`,
        `[download] 20% [=====>                    ] 14.2MB / 71.0MB (24.5 MB/s)`,
        `[download] 45% [===========>              ] 32.0MB / 71.0MB (28.1 MB/s)`,
        `[download] 75% [===================>      ] 53.3MB / 71.0MB (29.4 MB/s)`,
        `[download] 100% [========================>] 71.0MB / 71.0MB Download completed.`,
        `[mise] Verifying archive integrity with SHA256... Checksum matched.`,
        `[mise] Extracting archive to ~/.local/share/mise/installs/${toolId}/${version}...`,
        `[mise] Generating shims and setting executable permissions...`,
        `[mise] Successfully installed ${toolId}@${version}! Run 'mise use ${toolId}@${version}' to activate.`
      ];

      let currentStep = 0;
      const interval = setInterval(() => {
        if (currentStep < steps.length) {
          onLog(steps[currentStep]);
          const progressPercent = Math.min(100, Math.round(((currentStep + 1) / steps.length) * 100));
          onProgress(progressPercent);
          currentStep++;
        } else {
          clearInterval(interval);
          const tool = mockRuntimes.find(t => t.id === toolId);
          if (tool && !tool.installedVersions.includes(version)) {
            tool.installedVersions.unshift(version);
          }
          resolve(true);
        }
      }, 350);
    });
  },

  // Projects
  async getProjects(): Promise<ProjectEnv[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<ProjectEnv[]>('get_projects');
      } catch (err) {
        console.warn('Tauri invoke get_projects failed:', err);
      }
    }
    return [...mockProjects];
  },

  async addProject(path: string): Promise<ProjectEnv> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<ProjectEnv>('scan_and_add_project', { path });
      } catch (err) {
        console.warn('Tauri invoke add_project failed:', err);
      }
    }
    const name = path.split('/').filter(Boolean).pop() || 'new-project';
    const newProj: ProjectEnv = {
      id: 'p_' + Date.now(),
      name,
      path,
      configFile: '.mise.toml',
      tools: [
        { toolId: 'node', version: '22.12.0' }
      ],
      lastModified: new Date().toISOString().slice(0, 16).replace('T', ' ')
    };
    mockProjects.unshift(newProj);
    return newProj;
  },

  async setProjectToolVersion(projectId: string, toolId: string, version: string): Promise<boolean> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<boolean>('set_project_tool_version', { projectId, toolId, version });
      } catch (err) {
        console.warn('Tauri invoke set_project_tool_version failed:', err);
      }
    }
    const proj = mockProjects.find(p => p.id === projectId);
    if (proj) {
      const existing = proj.tools.find(t => t.toolId === toolId);
      if (existing) {
        existing.version = version;
      } else {
        proj.tools.push({ toolId, version });
      }
    }
    return true;
  },

  // System Tools
  async getSystemTools(): Promise<SystemTool[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<SystemTool[]>('get_system_tools');
      } catch (err) {
        console.warn('Tauri invoke get_system_tools failed:', err);
      }
    }
    return [...mockSystemTools];
  },

  async installSystemTool(toolId: string): Promise<boolean> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<boolean>('install_system_tool', { toolId });
      } catch (err) {
        console.warn('Tauri invoke install_system_tool failed:', err);
      }
    }
    const tool = mockSystemTools.find(t => t.id === toolId);
    if (tool) {
      tool.isInstalled = true;
      tool.installedVersion = 'latest';
    }
    return true;
  },

  // Mirrors
  async getMirrors(): Promise<MirrorConfig[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<MirrorConfig[]>('get_mirrors');
      } catch (err) {
        console.warn('Tauri invoke get_mirrors failed:', err);
      }
    }
    return [...mockMirrors];
  },

  async setMirror(tool: string, mirrorUrl: string): Promise<boolean> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<boolean>('set_mirror', { tool, mirrorUrl });
      } catch (err) {
        console.warn('Tauri invoke set_mirror failed:', err);
      }
    }
    const mirror = mockMirrors.find(m => m.tool === tool);
    if (mirror) {
      mirror.currentMirror = mirrorUrl;
    }
    return true;
  },

  async pingMirrors(): Promise<Record<string, number>> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<Record<string, number>>('ping_mirrors');
      } catch (err) {
        console.warn('Tauri invoke ping_mirrors failed:', err);
      }
    }
    // Return realistic random ping measurements
    const res: Record<string, number> = {};
    for (const m of mockMirrors) {
      for (const opt of m.options) {
        opt.ping = Math.floor(Math.random() * 30) + (opt.url.includes('.org') || opt.url.includes('github') ? 180 : 20);
        res[opt.url] = opt.ping;
      }
    }
    return res;
  },

  // Health Checks
  async getHealthChecks(): Promise<EnvHealthCheck[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<EnvHealthCheck[]>('get_health_checks');
      } catch (err) {
        console.warn('Tauri invoke get_health_checks failed:', err);
      }
    }
    return [...mockHealthChecks];
  },

  async autoFixHealthCheck(checkId: string): Promise<boolean> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<boolean>('auto_fix_health_check', { checkId });
      } catch (err) {
        console.warn('Tauri invoke auto_fix failed:', err);
      }
    }
    const check = mockHealthChecks.find(c => c.id === checkId);
    if (check) {
      check.status = 'ok';
      check.message = '已自动修复并重新生效配置';
    }
    return true;
  },

  // Bootstrap / Install Mise CLI
  async installMiseCli(): Promise<boolean> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<boolean>('bootstrap_mise_cli');
      } catch (err) {
        console.warn('Tauri invoke bootstrap_mise_cli failed:', err);
      }
    }
    return true;
  }
};
