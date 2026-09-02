import React from 'react';
import { Search, RefreshCw, CheckCircle2, AlertCircle, Apple, Monitor, Globe, Laptop } from 'lucide-react';
import { SystemStatus } from '../types';
import { isTauri } from '../services/tauri';
import { getCurrentWindow } from '@tauri-apps/api/window';

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

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button === 0) {
      const target = e.target as HTMLElement;
      if (!target.closest('button') && !target.closest('input') && !target.closest('select') && !target.closest('a')) {
        if (isTauri()) {
          try {
            getCurrentWindow().startDragging();
          } catch (_) {}
        }
      }
    }
  };

  return (
    <header 
      className="h-[4.75rem] pt-6 border-b border-slate-800/80 bg-[#0B1120]/95 backdrop-blur-md px-5 flex items-center justify-between shrink-0 select-none drag-region" 
      data-tauri-drag-region
      onMouseDown={handleMouseDown}
    >
      {/* Title & Subtitle */}
      <div className="shrink-0 min-w-0 pr-2 pointer-events-none" data-tauri-drag-region>
        <h1 className="text-sm sm:text-base font-bold text-slate-100 flex items-center gap-2 truncate">
          {title}
        </h1>
        <p className="text-[11px] text-slate-400 truncate hidden sm:block">{subtitle}</p>
      </div>

      {/* Center Search */}
      <div className="flex-1 max-w-xs min-w-[120px] mx-3 no-drag">
        <div className="relative">
          <Search className="w-3.5 h-3.5 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
          <input
            type="text"
            placeholder="搜索运行时或工具..."
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            className="w-full bg-slate-900/90 border border-slate-800 text-xs text-slate-200 placeholder-slate-400 rounded-xl pl-8 pr-3 py-1.5 focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/30 transition-all no-drag"
          />
        </div>
      </div>

      {/* Right System Badges & Refresh */}
      <div className="flex items-center gap-2 shrink-0 no-drag">
        {/* Environment Badge */}
        {inDesktop ? (
          <div className="hidden md:flex items-center gap-1.5 px-2 py-1 rounded-lg bg-blue-500/10 border border-blue-500/20 text-blue-400 text-xs font-medium shrink-0">
            <Laptop className="w-3.5 h-3.5" />
            <span className="text-[11px]">桌面端</span>
          </div>
        ) : (
          <div className="hidden md:flex items-center gap-1.5 px-2 py-1 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-400 text-xs font-medium shrink-0">
            <Globe className="w-3.5 h-3.5" />
            <span className="text-[11px]">网页预览</span>
          </div>
        )}

        {/* OS & Shell Chip */}
        <div className="hidden xl:flex items-center gap-2 px-2.5 py-1 rounded-lg bg-slate-900 border border-slate-800 text-xs text-slate-300 shrink-0">
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
          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-medium shrink-0">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span className="text-[11px] hidden sm:inline">Mise 已就绪</span>
          </div>
        ) : (
          <button
            onClick={onOpenBootstrap}
            className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-300 hover:bg-amber-500/20 text-xs font-medium transition-colors shrink-0 no-drag"
          >
            <AlertCircle className="w-3.5 h-3.5 text-amber-400" />
            <span className="text-[11px]">安装 Mise</span>
          </button>
        )}

        {/* Refresh Action */}
        <button
          onClick={onRefresh}
          disabled={isRefreshing}
          title="刷新环境数据"
          className="p-1.5 sm:p-2 rounded-xl bg-slate-900 border border-slate-800 text-slate-400 hover:text-slate-200 hover:bg-slate-800/80 transition-all disabled:opacity-50 shrink-0 no-drag"
        >
          <RefreshCw className={`w-3.5 h-3.5 sm:w-4 sm:h-4 ${isRefreshing ? 'animate-spin text-blue-400' : ''}`} />
        </button>
      </div>
    </header>
  );
};
