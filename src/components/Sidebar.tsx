import React from 'react';
import { 
  Layers, 
  FolderGit2, 
  Wrench, 
  Zap, 
  ShieldCheck, 
  Settings, 
  Terminal, 
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
      badge: `${installedCount} 个已装`,
      badgeColor: 'bg-blue-500/10 text-blue-400 border border-blue-500/20'
    },
    {
      id: 'projects' as TabType,
      label: '项目环境隔离',
      sublabel: 'Projects',
      icon: FolderGit2,
      badge: '3',
      badgeColor: 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/20'
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
      sublabel: 'Mirrors Speedup',
      icon: Zap,
      badge: '极速',
      badgeColor: 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
    },
    {
      id: 'env-health' as TabType,
      label: '环境健康自检',
      sublabel: 'Health Check',
      icon: ShieldCheck,
      badge: healthIssueCount > 0 ? `${healthIssueCount} 需处理` : '健康',
      badgeColor: healthIssueCount > 0 
        ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20' 
        : 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
    },
  ];

  return (
    <aside className="w-64 bg-[#0B1120] border-r border-slate-800/80 flex flex-col justify-between shrink-0 select-none">
      {/* Brand Header with macOS Traffic Lights Clearance */}
      <div>
        <div className="h-7 w-full shrink-0" data-tauri-drag-region />
        <div className="h-14 flex items-center gap-3 px-5 border-b border-slate-800/80" data-tauri-drag-region>
          <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-blue-600 via-indigo-600 to-cyan-400 flex items-center justify-center shadow-lg shadow-blue-500/20 text-white font-bold shrink-0">
            <Cpu className="w-5 h-5" />
          </div>
          <div>
            <div className="font-bold tracking-tight text-white flex items-center gap-1.5 text-base">
              EnvHub
              <span className="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400 border border-blue-500/30 font-semibold">
                Tauri v2
              </span>
            </div>
            <p className="text-[11px] text-slate-400 font-medium">开发环境与版本管理</p>
          </div>
        </div>

        {/* Navigation items */}
        <nav className="p-3 space-y-1">
          <div className="px-3 py-2 text-[10px] font-bold uppercase tracking-wider text-slate-400 font-mono">
            Core Modules
          </div>
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = currentTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => onSelectTab(item.id)}
                className={`w-full flex items-center justify-between px-3.5 py-2.5 rounded-xl transition-all duration-200 group text-left ${
                  isActive
                    ? 'bg-blue-600/15 text-blue-400 border border-blue-500/30 shadow-sm'
                    : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40 border border-transparent'
                }`}
              >
                <div className="flex items-center gap-3">
                  <div className={`p-1.5 rounded-lg transition-colors ${
                    isActive ? 'bg-blue-500/20 text-blue-300' : 'bg-slate-800/60 text-slate-400 group-hover:text-slate-200'
                  }`}>
                    <Icon className="w-4 h-4" />
                  </div>
                  <div>
                    <div className="text-sm font-semibold tracking-tight">{item.label}</div>
                    <div className="text-[11px] text-slate-400 font-normal">{item.sublabel}</div>
                  </div>
                </div>
                {item.badge && (
                  <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${item.badgeColor}`}>
                    {item.badge}
                  </span>
                )}
              </button>
            );
          })}
        </nav>
      </div>

      {/* Footer / Status card */}
      <div className="p-3 space-y-2 border-t border-slate-800/80 bg-slate-950/40">
        <div className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className={`w-2.5 h-2.5 rounded-full ${systemStatus.miseInstalled ? 'bg-emerald-400 shadow-sm shadow-emerald-400/50 animate-pulse-subtle' : 'bg-amber-400'}`} />
            <div>
              <div className="text-xs font-semibold text-slate-200">Mise 引擎核心</div>
              <div className="text-[10px] text-slate-400 font-mono">
                {systemStatus.miseInstalled ? 'CLI 已就绪' : '未检测到 CLI'}
              </div>
            </div>
          </div>
          <div className="p-1 rounded bg-slate-800 text-slate-400">
            <Terminal className="w-3.5 h-3.5" />
          </div>
        </div>

        <button
          onClick={() => onSelectTab('settings')}
          className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-xs font-medium transition-colors ${
            currentTab === 'settings'
              ? 'bg-slate-800 text-slate-100'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
          }`}
        >
          <Settings className="w-4 h-4 text-slate-400" />
          <span>客户端与引擎设置</span>
        </button>
      </div>
    </aside>
  );
};
