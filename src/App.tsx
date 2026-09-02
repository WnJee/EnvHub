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
import { ToastProvider, useToast } from './components/Toast';
import { api } from './services/tauri';
import { 
  RuntimeTool, 
  ProjectEnv, 
  SystemTool, 
  MirrorConfig, 
  EnvHealthCheck, 
  SystemStatus 
} from './types';

const MainDashboard: React.FC = () => {
  const [currentTab, setCurrentTab] = useState<TabType>('runtimes');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
  const [isPingingMirrors, setIsPingingMirrors] = useState<boolean>(false);
  const [isBootstrappingMise, setIsBootstrappingMise] = useState<boolean>(false);
  const toast = useToast();

  // Core state from real host environment
  const [systemStatus, setSystemStatus] = useState<SystemStatus>({
    os: 'macos',
    osVersion: '环境加载中...',
    arch: 'arm64',
    defaultShell: '/bin/zsh',
    miseInstalled: false,
    packageManager: 'none'
  });

  const [runtimes, setRuntimes] = useState<RuntimeTool[]>([]);
  const [projects, setProjects] = useState<ProjectEnv[]>([]);
  const [systemTools, setSystemTools] = useState<SystemTool[]>([]);
  const [mirrors, setMirrors] = useState<MirrorConfig[]>([]);
  const [healthChecks, setHealthChecks] = useState<EnvHealthCheck[]>([]);

  // Real Install Modal State
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

  // Load real data from host
  const loadAllData = async (isManual = false) => {
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

      if (isManual) {
        toast.success('已刷新并同步本机最新环境状态');
      }
    } catch (err) {
      console.error('Failed to load real environment data:', err);
      toast.error(`读取系统环境数据失败: ${err}`);
    } finally {
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    loadAllData();
  }, []);

  // Handlers for Runtimes
  const handleSetGlobalVersion = async (toolId: string, version: string) => {
    try {
      const ok = await api.setGlobalVersion(toolId, version);
      if (ok) {
        toast.success(`已将 ${toolId} 系统全局主用版本切换为 v${version}`);
        setRuntimes((prev) =>
          prev.map((t) =>
            t.id === toolId
              ? { ...t, globalVersion: version, activeVersion: version }
              : t
          )
        );
      }
    } catch (err) {
      toast.error(`切换版本失败: ${err}`);
    }
  };

  const handleUninstallVersion = async (toolId: string, version: string) => {
    if (!confirm(`确定要在本机真实卸载 ${toolId} v${version} 吗？`)) return;
    try {
      const ok = await api.uninstallVersion(toolId, version);
      if (ok) {
        toast.success(`已卸载 ${toolId} v${version}`);
        loadAllData();
      }
    } catch (err) {
      toast.error(`卸载失败: ${err}`);
    }
  };

  // Start Real Install Handler
  const handleOpenInstallModal = (toolId: string, version: string) => {
    setInstallModalState({
      isOpen: true,
      toolId,
      version,
      logs: [`[init] 正在连接底层 Mise CLI 管道，准备下载编译 ${toolId}@${version}...`],
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
        toast.success(`成功安装 ${toolId} v${version}`);
        loadAllData();
      } else {
        toast.error(`安装 ${toolId} v${version} 失败，请查看日志详情`);
      }
    });
  };

  // Handlers for Projects
  const handleAddProject = async (path: string) => {
    try {
      const newProj = await api.addProject(path);
      setProjects((prev) => [newProj, ...prev.filter((p) => p.path !== path)]);
      toast.success(`成功扫描并添加项目: ${newProj.name}`);
    } catch (err) {
      toast.error(`添加项目失败: ${err}`);
    }
  };

  const handleSetProjectToolVersion = async (projectId: string, toolId: string, version: string) => {
    try {
      const ok = await api.setProjectToolVersion(projectId, toolId, version);
      if (ok) {
        toast.success(`已将项目中的 ${toolId} 版本配置更新为 v${version}`);
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
    } catch (err) {
      toast.error(`更新项目版本失败: ${err}`);
    }
  };

  // Handlers for System Tools
  const handleInstallSystemTool = async (toolId: string) => {
    try {
      toast.info(`正在调用包管理器安装 ${toolId}...`);
      const ok = await api.installSystemTool(toolId);
      if (ok) {
        toast.success(`成功安装系统工具: ${toolId}`);
        loadAllData();
      }
    } catch (err) {
      toast.error(`安装失败: ${err}`);
    }
  };

  // Handlers for Mirrors
  const handleSetMirror = async (tool: string, mirrorUrl: string) => {
    try {
      await api.setMirror(tool, mirrorUrl);
      setMirrors((prev) =>
        prev.map((m) => (m.tool === tool ? { ...m, currentMirror: mirrorUrl } : m))
      );
    } catch (err) {
      toast.error(`配置镜像失败: ${err}`);
    }
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
      toast.success('已完成全部国内镜像源的真实延迟测速');
    } catch (err) {
      toast.error(`测速失败: ${err}`);
    } finally {
      setIsPingingMirrors(false);
    }
  };

  // Handlers for Health
  const handleAutoFixHealth = async (checkId: string) => {
    try {
      const ok = await api.autoFixHealthCheck(checkId);
      if (ok) {
        setHealthChecks((prev) =>
          prev.map((c) =>
            c.id === checkId ? { ...c, status: 'ok', message: '已自动写入 Shell RC 并生效' } : c
          )
        );
      }
    } catch (err) {
      toast.error(`自动修复失败: ${err}`);
    }
  };

  // Handlers for Bootstrap Mise
  const handleBootstrapMise = async () => {
    setIsBootstrappingMise(true);
    try {
      toast.info('正在拉取并自举安装 Mise CLI 引擎...');
      await api.installMiseCli();
      toast.success('Mise CLI 引擎已安装就绪！');
      loadAllData();
    } catch (err) {
      toast.error(`自举安装失败: ${err}`);
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
      subtitle: '真实检测 Node.js / Python / Go / Rust / Java / Bun 等多版本'
    },
    projects: {
      title: '项目工程环境隔离',
      subtitle: '针对不同工程目录绑定独立开发语言版本，CD 进入目录自动生效'
    },
    'system-tools': {
      title: '系统级开发工具箱',
      subtitle: '通过系统包管理器 (Homebrew / Winget / Apt) 真实管理 Git, Docker 等'
    },
    mirrors: {
      title: '国内源加速与真实测速',
      subtitle: '真实测试并写入 NPM / Pip / GoProxy / Cargo / Homebrew 国内高速镜像'
    },
    'env-health': {
      title: '环境健康自检与 Shell 修复',
      subtitle: '排查 ~/.zshrc、~/.bashrc 及 PATH 优先级，保障终端与 GUI 环境一致'
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
          onRefresh={() => loadAllData(true)}
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
              onRefresh={() => loadAllData(true)}
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

export const App: React.FC = () => {
  return (
    <ToastProvider>
      <MainDashboard />
    </ToastProvider>
  );
};

export default App;
