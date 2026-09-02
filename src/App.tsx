import React, { useState, useEffect } from 'react';
import { Sidebar, TabType } from './components/Sidebar';
import { Header } from './components/Header';
import { RuntimeManager } from './components/RuntimeManager';
import { ProjectManager } from './components/ProjectManager';
import { SystemTools } from './components/SystemTools';
import { MirrorManager } from './components/MirrorManager';
import { EnvHealth } from './components/EnvHealth';
import { SettingsModal } from './components/SettingsModal';
import { InstallModal } from './components/InstallModal';
import { api } from './services/tauri';
import { 
  RuntimeTool, 
  ProjectEnv, 
  SystemTool, 
  MirrorConfig, 
  EnvHealthCheck, 
  SystemStatus 
} from './types';

export const App: React.FC = () => {
  const [currentTab, setCurrentTab] = useState<TabType>('runtimes');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
  const [isPingingMirrors, setIsPingingMirrors] = useState<boolean>(false);
  const [isBootstrappingMise, setIsBootstrappingMise] = useState<boolean>(false);

  // Core state
  const [systemStatus, setSystemStatus] = useState<SystemStatus>({
    os: 'macos',
    osVersion: 'macOS 15.0',
    arch: 'aarch64',
    defaultShell: '/bin/zsh',
    miseInstalled: true,
    packageManager: 'brew'
  });

  const [runtimes, setRuntimes] = useState<RuntimeTool[]>([]);
  const [projects, setProjects] = useState<ProjectEnv[]>([]);
  const [systemTools, setSystemTools] = useState<SystemTool[]>([]);
  const [mirrors, setMirrors] = useState<MirrorConfig[]>([]);
  const [healthChecks, setHealthChecks] = useState<EnvHealthCheck[]>([]);

  // Install Modal State
  const [installModalState, setInstallModalState] = useState<{
    isOpen: boolean;
    toolId: string;
    version: string;
    logs: string[];
    progress: number;
    status: 'idle' | 'running' | 'completed' | 'failed';
  }>({
    isOpen: false,
    toolId: '',
    version: '',
    logs: [],
    progress: 0,
    status: 'idle'
  });

  // Load initial data
  const loadAllData = async () => {
    setIsRefreshing(true);
    try {
      const [status, rList, pList, sTools, mList, hList] = await Promise.all([
        api.getSystemStatus(),
        api.getRuntimes(),
        api.getProjects(),
        api.getSystemTools(),
        api.getMirrors(),
        api.getHealthChecks()
      ]);

      setSystemStatus(status);
      setRuntimes(rList);
      setProjects(pList);
      setSystemTools(sTools);
      setMirrors(mList);
      setHealthChecks(hList);
    } catch (err) {
      console.error('Failed to load application data:', err);
    } finally {
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    loadAllData();
  }, []);

  // Handlers for Runtimes
  const handleSetGlobalVersion = async (toolId: string, version: string) => {
    const ok = await api.setGlobalVersion(toolId, version);
    if (ok) {
      setRuntimes((prev) =>
        prev.map((t) =>
          t.id === toolId
            ? { ...t, globalVersion: version, activeVersion: version }
            : t
        )
      );
    }
  };

  const handleUninstallVersion = async (toolId: string, version: string) => {
    if (!confirm(`确定要卸载 ${toolId} v${version} 吗？`)) return;
    const ok = await api.uninstallVersion(toolId, version);
    if (ok) {
      setRuntimes((prev) =>
        prev.map((t) => {
          if (t.id === toolId) {
            const updated = t.installedVersions.filter((v) => v !== version);
            return {
              ...t,
              installedVersions: updated,
              activeVersion: t.activeVersion === version ? updated[0] : t.activeVersion,
              globalVersion: t.globalVersion === version ? updated[0] : t.globalVersion,
            };
          }
          return t;
        })
      );
    }
  };

  // Start Install Handler
  const handleOpenInstallModal = (toolId: string, version: string) => {
    setInstallModalState({
      isOpen: true,
      toolId,
      version,
      logs: [`[init] Preparing install pipeline for ${toolId}@${version}...`],
      progress: 5,
      status: 'running'
    });

    api.startInstallRuntime(
      toolId,
      version,
      (log) => {
        setInstallModalState((prev) => ({
          ...prev,
          logs: [...prev.logs, log]
        }));
      },
      (progress) => {
        setInstallModalState((prev) => ({
          ...prev,
          progress
        }));
      }
    ).then((success) => {
      setInstallModalState((prev) => ({
        ...prev,
        status: success ? 'completed' : 'failed',
        progress: success ? 100 : prev.progress
      }));
      if (success) {
        // Update local runtimes
        setRuntimes((prev) =>
          prev.map((t) => {
            if (t.id === toolId) {
              const list = t.installedVersions.includes(version)
                ? t.installedVersions
                : [version, ...t.installedVersions];
              return { ...t, installedVersions: list };
            }
            return t;
          })
        );
      }
    });
  };

  // Handlers for Projects
  const handleAddProject = async (path: string) => {
    const newProj = await api.addProject(path);
    setProjects((prev) => [newProj, ...prev]);
  };

  const handleSetProjectToolVersion = async (projectId: string, toolId: string, version: string) => {
    const ok = await api.setProjectToolVersion(projectId, toolId, version);
    if (ok) {
      setProjects((prev) =>
        prev.map((p) => {
          if (p.id === projectId) {
            const updatedTools = p.tools.map((t) =>
              t.toolId === toolId ? { ...t, version } : t
            );
            return { ...p, tools: updatedTools };
          }
          return p;
        })
      );
    }
  };

  // Handlers for System Tools
  const handleInstallSystemTool = async (toolId: string) => {
    const ok = await api.installSystemTool(toolId);
    if (ok) {
      setSystemTools((prev) =>
        prev.map((t) =>
          t.id === toolId
            ? { ...t, isInstalled: true, installedVersion: 'latest' }
            : t
        )
      );
    }
  };

  // Handlers for Mirrors
  const handleSetMirror = async (tool: string, mirrorUrl: string) => {
    await api.setMirror(tool, mirrorUrl);
    setMirrors((prev) =>
      prev.map((m) => (m.tool === tool ? { ...m, currentMirror: mirrorUrl } : m))
    );
  };

  const handlePingMirrors = async () => {
    setIsPingingMirrors(true);
    try {
      const pings = await api.pingMirrors();
      setMirrors((prev) =>
        prev.map((m) => ({
          ...m,
          options: m.options.map((opt) => ({
            ...opt,
            ping: pings[opt.url] !== undefined ? pings[opt.url] : opt.ping
          }))
        }))
      );
    } finally {
      setIsPingingMirrors(false);
    }
  };

  // Handlers for Health
  const handleAutoFixHealth = async (checkId: string) => {
    const ok = await api.autoFixHealthCheck(checkId);
    if (ok) {
      setHealthChecks((prev) =>
        prev.map((c) =>
          c.id === checkId ? { ...c, status: 'ok', message: '已修复并生效' } : c
        )
      );
    }
  };

  // Handlers for Bootstrap Mise
  const handleBootstrapMise = async () => {
    setIsBootstrappingMise(true);
    try {
      await api.installMiseCli();
      alert('Mise CLI 引擎已自举安装完成！');
      loadAllData();
    } finally {
      setIsBootstrappingMise(false);
    }
  };

  // Calculate totals
  const totalInstalledVersions = runtimes.reduce(
    (acc, cur) => acc + cur.installedVersions.length,
    0
  );
  const healthIssues = healthChecks.filter((c) => c.status !== 'ok').length;

  const tabTitles: Record<TabType, { title: string; subtitle: string }> = {
    runtimes: {
      title: '多语言运行时版本管理',
      subtitle: '支持 Node.js / Python / Go / Rust / Java / Bun 等多版本并存与极速切换'
    },
    projects: {
      title: '项目工程环境隔离',
      subtitle: '针对不同工程目录绑定独立开发语言版本，CD 进入目录自动生效'
    },
    'system-tools': {
      title: '系统级开发工具箱',
      subtitle: '通过系统包管理器 (Homebrew / Winget / Apt) 自动管理 Git, Docker 等'
    },
    mirrors: {
      title: '国内源加速与节点测速',
      subtitle: '一键测速并切换 NPM / Pip / GoProxy / Cargo / Homebrew 国内高速镜像'
    },
    'env-health': {
      title: '环境健康自检与 Shell 修复',
      subtitle: '全面排查 ~/.zshrc、~/.bashrc 及 PATH 优先级，保障终端与 GUI 环境一致'
    },
    settings: {
      title: '客户端设置与引擎配置',
      subtitle: '配置 Mise CLI 路径、自举策略与偏好设置'
    }
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[#090D16] text-slate-100 font-sans select-none">
      {/* Sidebar */}
      <Sidebar
        currentTab={currentTab}
        onSelectTab={setCurrentTab}
        installedCount={totalInstalledVersions}
        healthIssueCount={healthIssues}
        systemStatus={systemStatus}
      />

      {/* Main Container */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {/* Header */}
        <Header
          title={tabTitles[currentTab]?.title || 'EnvHub'}
          subtitle={tabTitles[currentTab]?.subtitle || ''}
          systemStatus={systemStatus}
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          onRefresh={loadAllData}
          isRefreshing={isRefreshing}
          onOpenBootstrap={() => setCurrentTab('settings')}
        />

        {/* Content View */}
        <main className="flex-1 flex min-h-0 overflow-hidden">
          {currentTab === 'runtimes' && (
            <RuntimeManager
              runtimes={runtimes}
              searchQuery={searchQuery}
              onSetGlobalVersion={handleSetGlobalVersion}
              onUninstallVersion={handleUninstallVersion}
              onOpenInstallModal={handleOpenInstallModal}
            />
          )}

          {currentTab === 'projects' && (
            <ProjectManager
              projects={projects}
              runtimes={runtimes}
              onAddProject={handleAddProject}
              onSetProjectToolVersion={handleSetProjectToolVersion}
            />
          )}

          {currentTab === 'system-tools' && (
            <SystemTools
              systemTools={systemTools}
              packageManager={systemStatus.packageManager}
              onInstallSystemTool={handleInstallSystemTool}
            />
          )}

          {currentTab === 'mirrors' && (
            <MirrorManager
              mirrors={mirrors}
              onSetMirror={handleSetMirror}
              onPingMirrors={handlePingMirrors}
              isPinging={isPingingMirrors}
            />
          )}

          {currentTab === 'env-health' && (
            <EnvHealth
              healthChecks={healthChecks}
              onAutoFix={handleAutoFixHealth}
              onRefresh={loadAllData}
            />
          )}

          {currentTab === 'settings' && (
            <SettingsModal
              systemStatus={systemStatus}
              onBootstrapMise={handleBootstrapMise}
              isBootstrapping={isBootstrappingMise}
            />
          )}
        </main>
      </div>

      {/* Real-time Streaming Install Modal */}
      <InstallModal
        isOpen={installModalState.isOpen}
        toolId={installModalState.toolId}
        version={installModalState.version}
        logs={installModalState.logs}
        progress={installModalState.progress}
        status={installModalState.status}
        onClose={() => setInstallModalState((prev) => ({ ...prev, isOpen: false }))}
        onSetGlobal={() => {
          handleSetGlobalVersion(installModalState.toolId, installModalState.version);
        }}
      />
    </div>
  );
};

export default App;
