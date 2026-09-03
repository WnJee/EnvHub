import React from 'react';
import { 
  Layers, 
  FolderGit2, 
  Wrench, 
  Zap, 
  ShieldCheck, 
  Settings, 
  Cpu
} from 'lucide-react';

export type TabType = 'runtimes' | 'projects' | 'system-tools' | 'mirrors' | 'env-health' | 'settings';

interface SidebarProps {
  currentTab: TabType;
  onSelectTab: (tab: TabType) => void;
  installedCount: number;
  healthIssueCount: number;
  systemStatus: {
    miseInstalled: boolean;
    packageManager: string;
  };
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentTab,
  onSelectTab,
  installedCount,
  healthIssueCount,
  systemStatus
}) => {
  const navItems = [
    {
      id: 'runtimes' as TabType,
      label: '多语言运行时',
      sublabel: 'Runtimes',
      icon: Layers,
      badge: `${installedCount} 已装`,
      badgeColor: 'bg-blue-500/10 text-blue-400 border border-blue-500/20'
    },
    {
      id: 'system-tools' as TabType,
      label: '系统工具箱',
      sublabel: 'System CLI',
      icon: Wrench,
      badge: 'Git/Docker',
      badgeColor: 'bg-slate-500/10 text-slate-400 border border-slate-500/20'
    },
    {
      id: 'mirrors' as TabType,
      label: '国内源加速',
      sublabel: 'Mirrors',
      icon: Zap,
      badge: undefined,
      badgeColor: 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
    },
    {
      id: 'env-health' as TabType,
      label: '环境健康自检',
      sublabel: 'Health Check',
      icon: ShieldCheck,
      badge: healthIssueCount > 0 ? `${healthIssueCount} 需配置` : '就绪',
      badgeColor: healthIssueCount > 0 
        ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20' 
        : 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
    },
    {
      id: 'projects' as TabType,
      label: '项目环境隔离',
      sublabel: 'Projects',
      icon: FolderGit2,
      badge: undefined,
      badgeColor: 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/20'
    },
  ];

  return (
    <aside className="w-60 bg-[#0B1120] border-r border-slate-800/80 flex flex-col justify-between shrink-0 select-none">
      {/* Brand Header with macOS Traffic Lights Clearance & Drag Region */}
      <div>
        {/* Top Traffic Lights Drag Area */}
        <div 
          className="h-6 w-full shrink-0" 
          data-tauri-drag-region
        />
        
        {/* Brand Bar */}
        <div 
          className="h-13 flex items-center gap-3 px-4 pb-2 border-b border-slate-800/80" 
          data-tauri-drag-region
        >
          <div className="w-8 h-8 rounded-xl bg-gradient-to-tr from-blue-600 via-indigo-600 to-cyan-400 flex items-center justify-center shadow-lg shadow-blue-500/20 text-white font-bold shrink-0 pointer-events-none">
            <Cpu className="w-4 h-4" />
          </div>
          <div className="pointer-events-none">
            <div className="font-bold tracking-tight text-white flex items-center gap-1.5 text-sm">
              EnvHub
              <span className="text-[9px] uppercase font-mono px-1.5 py-0.2 rounded bg-blue-500/20 text-blue-400 border border-blue-500/30 font-semibold">
                v0.2.7
              </span>
            </div>
            <p className="text-[10px] text-slate-400 font-medium">开发环境与版本管理</p>
          </div>
        </div>

        {/* Navigation items */}
        <nav className="p-2.5 space-y-1">
          <div className="px-3 py-1.5 text-[10px] font-bold uppercase tracking-wider text-slate-400 font-mono">
            功能导航
          </div>
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = currentTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => onSelectTab(item.id)}
                className={`w-full flex items-center justify-between px-3 py-2 rounded-xl transition-all duration-200 group text-left ${
                  isActive
                    ? 'bg-blue-600/15 border border-blue-500/30 text-white shadow-sm'
                    : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40 border border-transparent'
                }`}
              >
                <div className="flex items-center gap-2.5 min-w-0">
                  <Icon
                    className={`w-4 h-4 transition-colors shrink-0 ${
                      isActive ? 'text-blue-400' : 'text-slate-400 group-hover:text-slate-300'
                    }`}
                  />
                  <div className="truncate">
                    <span className="text-xs font-semibold block leading-tight">{item.label}</span>
                    <span className="text-[9px] text-slate-400 font-mono block leading-tight">
                      {item.sublabel}
                    </span>
                  </div>
                </div>

                {item.badge && (
                  <span
                    className={`text-[9px] font-mono px-1.5 py-0.2 rounded-md font-medium border shrink-0 ${item.badgeColor}`}
                  >
                    {item.badge}
                  </span>
                )}
              </button>
            );
          })}
        </nav>
      </div>

      {/* Bottom Settings & Status */}
      <div className="p-2.5 border-t border-slate-800/80 space-y-1.5">
        <button
          onClick={() => onSelectTab('settings')}
          className={`w-full flex items-center justify-between px-3 py-2 rounded-xl transition-all duration-200 text-left ${
            currentTab === 'settings'
              ? 'bg-blue-600/15 border border-blue-500/30 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40 border border-transparent'
          }`}
        >
          <div className="flex items-center gap-2.5">
            <Settings className="w-4 h-4 text-slate-400" />
            <span className="text-xs font-semibold">设置与偏好</span>
          </div>
          <span className="text-[10px] text-slate-400 font-mono">
            {systemStatus.miseInstalled ? '就绪' : '未装'}
          </span>
        </button>

        {/* Engine Status Tag */}
        <div className="px-3 py-1.5 rounded-lg bg-slate-950/60 border border-slate-800/60 flex items-center justify-between text-[10px] font-mono text-slate-400">
          <span>引擎状态</span>
          <span className={systemStatus.miseInstalled ? 'text-emerald-400' : 'text-amber-400'}>
            {systemStatus.miseInstalled ? 'Mise Active' : 'CLI Missing'}
          </span>
        </div>
      </div>
    </aside>
  );
};
