import { RuntimeTool, ProjectEnv, SystemTool, MirrorConfig, EnvHealthCheck, SystemStatus, InstallProgressDetail } from '../types';

// Check if running inside Tauri
export const isTauri = () => {
  return typeof window !== 'undefined' && ('__TAURI__' in window || '__TAURI_INTERNALS__' in window);
};

type InstallCommand = 'install_runtime_version' | 'install_system_tool' | 'bootstrap_mise_cli';

// Run an install command while forwarding its `install-log` / `install-progress`
// events. `install-progress-detail` carries the installer's real byte counts,
// speed and remaining time; the plain percentage still drives the bar.
async function streamInstall(
  command: InstallCommand,
  args: Record<string, string>,
  onLog: (log: string) => void,
  onProgress: (percent: number) => void,
  onDetail?: (detail: InstallProgressDetail) => void
): Promise<boolean> {
  if (!isTauri()) {
    onLog('[ERROR] 请在 EnvHub 桌面客户端中执行安装');
    return false;
  }
  const unlisteners: Array<() => void> = [];
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const { listen } = await import('@tauri-apps/api/event');
    unlisteners.push(
      await listen<string>('install-log', (event) => onLog(event.payload)),
      await listen<number>('install-progress', (event) => onProgress(event.payload))
    );
    if (onDetail) {
      unlisteners.push(
        await listen<InstallProgressDetail>('install-progress-detail', (event) => onDetail(event.payload))
      );
    }
    return await invoke<boolean>(command, args);
  } catch (err) {
    onLog(`[ERROR] 安装异常: ${err}`);
    return false;
  } finally {
    unlisteners.forEach((unlisten) => unlisten());
  }
}

// Service API layer directly connected to Tauri Backend
export const api = {
  async saveMisePath(path: string): Promise<string> {
    if (!isTauri()) throw new Error('请在桌面客户端中设置');
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<string>('save_mise_path', { path });
  },
  // System Status
  async getSystemStatus(): Promise<SystemStatus> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<SystemStatus>('get_system_status');
      } catch (err) {
        console.warn('get_system_status invoke error:', err);
        throw err;
      }
    }
    return {
      os: 'macos',
      osVersion: '桌面环境探测中',
      arch: 'arm64',
      defaultShell: '/bin/zsh',
      miseInstalled: false,
      packageManager: 'none'
    };
  },

  // Runtimes
  async getRuntimes(): Promise<RuntimeTool[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<RuntimeTool[]>('get_runtimes');
      } catch (err) {
        console.warn('get_runtimes invoke error:', err);
        throw err;
      }
    }
    return [];
  },

  async getAvailableVersions(toolId: string): Promise<string[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<string[]>('get_remote_versions', { toolId });
      } catch (err) {
        console.warn('get_remote_versions invoke error:', err);
        throw err;
      }
    }
    return [];
  },

  async setGlobalVersion(toolId: string, version: string): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('set_global_version', { toolId, version });
    }
    throw new Error('未运行在 Tauri 桌面原生环境中，无法修改全局系统版本');
  },

  async uninstallVersion(toolId: string, version: string): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('uninstall_runtime_version', { toolId, version });
    }
    throw new Error('未运行在 Tauri 桌面原生环境中，无法执行卸载');
  },

  // Real install runtime with streaming logs from Rust backend
  async startInstallRuntime(
    toolId: string,
    version: string,
    onLog: (log: string) => void,
    onProgress: (percent: number) => void,
    onDetail?: (detail: InstallProgressDetail) => void
  ): Promise<boolean> {
    return streamInstall('install_runtime_version', { toolId, version }, onLog, onProgress, onDetail);
  },

  async cancelCurrentInstall(): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('cancel_current_install');
    }
    return false;
  },

  // Projects
  async getProjects(): Promise<ProjectEnv[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<ProjectEnv[]>('get_projects');
      } catch (err) {
        console.warn('get_projects invoke error:', err);
        throw err;
      }
    }
    return [];
  },

  async addProject(path: string): Promise<ProjectEnv> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<ProjectEnv>('scan_and_add_project', { path });
    }
    throw new Error('未运行在 Tauri 桌面环境中');
  },

  async setProjectToolVersion(projectPath: string, toolId: string, version: string): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('set_project_tool_version', { projectPath, toolId, version });
    }
    throw new Error('未运行在 Tauri 桌面环境中');
  },

  async removeProject(path: string): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('remove_project', { path });
    }
    return true;
  },

  async openTerminalForRuntime(toolId: string, version: string): Promise<string> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<string>('open_terminal_for_runtime', { toolId, version });
    }
    return `模拟终端检测: ${toolId} v${version}`;
  },

  async openInEditor(path: string): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('open_in_editor', { path });
    }
    throw new Error('桌面原生环境不可用');
  },

  async openInTerminal(path: string): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('open_in_terminal', { path });
    }
    throw new Error('桌面原生环境不可用');
  },

  // System Tools
  async getSystemTools(): Promise<SystemTool[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<SystemTool[]>('get_system_tools');
      } catch (err) {
        console.warn('get_system_tools invoke error:', err);
        throw err;
      }
    }
    return [];
  },

  async testSystemTool(toolId: string): Promise<string> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<string>('test_system_tool', { toolId });
    }
    throw new Error('桌面原生环境不可用');
  },

  async startInstallSystemTool(
    toolId: string,
    onLog: (log: string) => void,
    onProgress: (percent: number) => void,
    onDetail?: (detail: InstallProgressDetail) => void
  ): Promise<boolean> {
    return streamInstall('install_system_tool', { toolId }, onLog, onProgress, onDetail);
  },

  // Mirrors
  async getMirrors(): Promise<MirrorConfig[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<MirrorConfig[]>('get_mirrors');
      } catch (err) {
        console.warn('get_mirrors invoke error:', err);
        throw err;
      }
    }
    return [];
  },

  async setMirror(tool: string, mirrorUrl: string): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('set_mirror', { tool, mirrorUrl });
    }
    throw new Error('桌面原生环境不可用');
  },

  async pingMirrors(): Promise<Record<string, number>> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<Record<string, number>>('ping_mirrors');
    }
    return {};
  },

  // Health Checks
  async getHealthChecks(): Promise<EnvHealthCheck[]> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<EnvHealthCheck[]>('get_health_checks');
      } catch (err) {
        console.warn('get_health_checks invoke error:', err);
        throw err;
      }
    }
    return [];
  },

  async autoFixHealthCheck(checkId: string): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('auto_fix_health_check', { checkId });
    }
    throw new Error('桌面原生环境不可用');
  },

  // Bootstrap / Install Mise CLI with real-time log streaming
  async startBootstrapMise(
    onLog: (log: string) => void,
    onProgress: (percent: number) => void,
    onDetail?: (detail: InstallProgressDetail) => void
  ): Promise<boolean> {
    return streamInstall('bootstrap_mise_cli', {}, onLog, onProgress, onDetail);
  },

  // Open URL in system default browser
  async openUrl(url: string): Promise<boolean> {
    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<boolean>('open_url_in_browser', { url });
      } catch (err) {
        console.warn('open_url_in_browser invoke error:', err);
      }
    }
    window.open(url, '_blank');
    return true;
  },

  // Open Path in Finder / Explorer
  async openPathInFileManager(path: string): Promise<boolean> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<boolean>('open_path_in_file_manager', { path });
    }
    return true;
  },

  // Relaunch the application after in-place update
  async relaunchApp(): Promise<void> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('relaunch_application');
    } else {
      window.location.reload();
    }
  },

  // Check GitHub Latest Release via Backend
  async checkGithubLatestRelease(): Promise<{ tag_name: string; body: string; html_url: string; published_at: string }> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke('check_github_latest_release');
    }
    throw new Error('Not in desktop environment');
  },

  // In-App Download and Launch Update Package
  async downloadAndInstallUpdate(
    downloadUrl: string,
    version: string,
    onProgress: (percent: number) => void
  ): Promise<string> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');

      const unlisten = await listen<number>('update-download-progress', (event) => {
        onProgress(event.payload);
      });

      try {
        const destPath = await invoke<string>('download_and_install_update', {
          downloadUrl,
          version
        });
        unlisten();
        return destPath;
      } catch (err) {
        unlisten();
        throw err;
      }
    }
    throw new Error('请在桌面端运行以执行下载更新');
  },

  // Save Exported Script / Config File
  async saveExportFile(filename: string, content: string): Promise<string> {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<string>('save_export_file', { filename, content });
    }

    // Web Fallback: trigger browser download
    const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    return `已触发浏览器下载: ${filename}`;
  }
};
