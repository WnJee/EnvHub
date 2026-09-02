import React from 'react';
import { Search, RefreshCw, CheckCircle2, AlertCircle, Apple, Monitor, Globe, Laptop } from 'lucide-react';
import { SystemStatus } from '../types';
import { isTauri } from '../services/tauri';

interface HeaderProps {
  title: string;
  subtitle: string;
  systemStatus: SystemStatus;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onRefresh: () => void;
  isRefreshing: boolean;
  onOpenBootstrap?: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  title,
  subtitle,
  systemStatus,
  searchQuery,
  onSearchChange,
  onRefresh,
  isRefreshing,
  onOpenBootstrap
}) => {
  const inDesktop = isTauri();

  return (
    <header className="h-[5.25rem] pt-7 border-b border-slate-800/80 bg-[#0B1120]/90 backdrop-blur-md px-6 flex items-center justify-between shrink-0 select-none" data-tauri-drag-region>
      {/* Title & Subtitle */}
      <div>
        <h1 className="text-base font-bold text-slate-100 flex items-center gap-2">
          {title}
        </h1>
        <p className="text-xs text-slate-400">{subtitle}</p>
      </div>

      {/* Center Search */}
      <div className="flex-1 max-w-md mx-6">
        <div className="relative">
          <Search className="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            placeholder="搜索运行时、版本号、项目或工具 (快捷键 ⌘K)..."
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            className="w-full bg-slate-900/90 border border-slate-800 text-xs text-slate-200 placeholder-slate-400 rounded-xl pl-9 pr-4 py-2 focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/30 transition-all"
          />
        </div>
      </div>

      {/* Right System Badges & Refresh */}
      <div className="flex items-center gap-3">
        {/* Runtime Environment Badge */}
        {inDesktop ? (
          <div className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-blue-500/10 border border-blue-500/20 text-blue-400 text-xs font-medium">
            <Laptop className="w-3.5 h-3.5" />
            <span className="text-[11px]">Tauri 桌面原生</span>
          </div>
        ) : (
          <div className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-400 text-xs font-medium">
            <Globe className="w-3.5 h-3.5" />
            <span className="text-[11px]">浏览器预览模式</span>
          </div>
        )}

        {/* OS & Shell Chip */}
        <div className="hidden lg:flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-xs text-slate-300">
          {systemStatus.os === 'macos' ? (
            <Apple className="w-3.5 h-3.5 text-slate-400" />
          ) : (
            <Monitor className="w-3.5 h-3.5 text-slate-400" />
          )}
          <span className="font-medium text-[11px]">{systemStatus.osVersion.split(' ')[0]}</span>
          <span className="text-slate-600">|</span>
          <span className="font-mono text-[11px] text-slate-400">{systemStatus.arch}</span>
          <span className="text-slate-600">|</span>
          <span className="font-mono text-[11px] text-blue-400">{systemStatus.defaultShell.split('/').pop()}</span>
        </div>

        {/* Mise Status Chip */}
        {systemStatus.miseInstalled ? (
          <div className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-medium">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span className="text-[11px]">mise 引擎正常</span>
          </div>
        ) : (
          <button
            onClick={onOpenBootstrap}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-300 hover:bg-amber-500/20 text-xs font-medium transition-colors"
          >
            <AlertCircle className="w-3.5 h-3.5 text-amber-400" />
            <span className="text-[11px]">一键安装 mise</span>
          </button>
        )}

        {/* Refresh Action */}
        <button
          onClick={onRefresh}
          disabled={isRefreshing}
          title="刷新全部环境数据"
          className="p-2 rounded-xl bg-slate-900 border border-slate-800 text-slate-400 hover:text-slate-200 hover:bg-slate-800/80 transition-all disabled:opacity-50"
        >
          <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin text-blue-400' : ''}`} />
        </button>
      </div>
    </header>
  );
};
