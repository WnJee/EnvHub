import React, { useState } from 'react';
import { RuntimeTool } from '../types';
import { 
  Check, 
  Download, 
  ExternalLink, 
  Globe, 
  Trash2, 
  ShieldAlert, 
  Search, 
  Layers, 
  ChevronRight,
  Terminal,
  Zap,
  Plus,
  AlertCircle
} from 'lucide-react';

interface RuntimeManagerProps {
  runtimes: RuntimeTool[];
  searchQuery: string;
  onSetGlobalVersion: (toolId: string, version: string) => void;
  onUninstallVersion: (toolId: string, version: string) => void;
  onOpenInstallModal: (toolId: string, version: string) => void;
}

export const RuntimeManager: React.FC<RuntimeManagerProps> = ({
  runtimes,
  searchQuery,
  onSetGlobalVersion,
  onUninstallVersion,
  onOpenInstallModal,
}) => {
  const [selectedToolId, setSelectedToolId] = useState<string>(runtimes[0]?.id || 'node');
  const [versionFilter, setVersionFilter] = useState<'all' | 'installed' | 'lts'>('all');
  const [remoteSearch, setRemoteSearch] = useState<string>('');
  const [customVersionInput, setCustomVersionInput] = useState<string>('');

  const filteredRuntimes = runtimes.filter(
    (t) =>
      t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.id.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const currentTool = runtimes.find((t) => t.id === selectedToolId) || runtimes[0];

  if (!currentTool) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-8 text-center text-slate-400 bg-[#090D16]">
        <ShieldAlert className="w-12 h-12 text-slate-600 mb-3" />
        <h3 className="text-base font-semibold text-slate-300">正在探测本机开发环境运行时...</h3>
        <p className="text-xs text-slate-400 mt-1">如无数据请检查是否安装了对应语言编译器或 Mise CLI</p>
      </div>
    );
  }

  // Filter remote versions
  const availableVersionsFiltered = currentTool.availableVersions.filter((v) => {
    if (remoteSearch && !v.toLowerCase().includes(remoteSearch.toLowerCase())) {
      return false;
    }
    if (versionFilter === 'installed') {
      return currentTool.installedVersions.includes(v);
    }
    if (versionFilter === 'lts') {
      return v.includes('20.') || v.includes('18.') || v.includes('21.') || v.includes('3.12') || v.includes('temurin-21') || v.includes('lts');
    }
    return true;
  });

  return (
    <div className="flex-1 flex overflow-hidden bg-[#090D16]">
      {/* Left List of Runtimes */}
      <div className="w-80 border-r border-slate-800/80 bg-[#0B1120]/60 flex flex-col shrink-0">
        <div className="p-4 border-b border-slate-800/80 flex items-center justify-between">
          <div className="text-xs font-bold uppercase tracking-wider text-slate-400 font-mono flex items-center gap-1.5">
            <Layers className="w-3.5 h-3.5 text-blue-400" />
            开发环境运行时 ({filteredRuntimes.length})
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {filteredRuntimes.map((tool) => {
            const isSelected = tool.id === currentTool.id;
            const installedCount = tool.installedVersions.length;

            return (
              <button
                key={tool.id}
                onClick={() => {
                  setSelectedToolId(tool.id);
                  setRemoteSearch('');
                }}
                className={`w-full flex items-center justify-between p-3 rounded-xl transition-all duration-200 text-left group ${
                  isSelected
                    ? 'bg-blue-600/15 border border-blue-500/30 text-white shadow-sm'
                    : 'hover:bg-slate-800/40 border border-transparent text-slate-300'
                }`}
              >
                <div className="flex items-center gap-3 min-w-0">
                  <div className="w-9 h-9 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-center p-1.5 shrink-0 group-hover:scale-105 transition-transform">
                    <img 
                      src={tool.icon} 
                      alt={tool.name} 
                      className="w-full h-full object-contain"
                      onError={(e) => {
                        (e.target as HTMLElement).style.display = 'none';
                      }}
                    />
                  </div>
                  <div className="truncate">
                    <div className="text-sm font-semibold flex items-center gap-1.5">
                      {tool.name}
                      {tool.activeVersion && (
                        <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">
                          v{tool.activeVersion}
                        </span>
                      )}
                    </div>
                    <div className="text-[11px] text-slate-400 truncate">{tool.description}</div>
                  </div>
                </div>

                <div className="flex items-center gap-2 pl-2 shrink-0">
                  <span className={`text-[10px] px-1.5 py-0.5 rounded-md font-mono ${
                    installedCount > 0 
                      ? 'bg-blue-500/20 text-blue-300 border border-blue-500/30' 
                      : 'bg-slate-900 text-slate-400'
                  }`}>
                    {installedCount > 0 ? `${installedCount} 已装` : '未安装'}
                  </span>
                  <ChevronRight className={`w-3.5 h-3.5 transition-transform ${isSelected ? 'text-blue-400 translate-x-0.5' : 'text-slate-400'}`} />
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Right Detail Pane */}
      <div className="flex-1 flex flex-col overflow-y-auto p-6 space-y-6">
        {/* Tool Header Banner */}
        <div className="p-6 rounded-2xl bg-gradient-to-r from-slate-900/90 via-slate-900/70 to-blue-950/20 border border-slate-800/80 flex flex-col md:flex-row items-start md:items-center justify-between gap-6 shadow-xl">
          <div className="flex items-center gap-4">
            <div className="w-16 h-16 rounded-2xl bg-slate-900/90 border border-slate-700/60 p-3 shadow-inner flex items-center justify-center shrink-0">
              <img 
                src={currentTool.icon} 
                alt={currentTool.name} 
                className="w-full h-full object-contain" 
              />
            </div>
            <div>
              <div className="flex items-center gap-3">
                <h2 className="text-2xl font-bold text-white tracking-tight">{currentTool.name}</h2>
                <a
                  href={currentTool.officialSite}
                  target="_blank"
                  rel="noreferrer"
                  className="text-xs text-slate-400 hover:text-blue-400 flex items-center gap-1 transition-colors"
                >
                  官网文档 <ExternalLink className="w-3 h-3" />
                </a>
              </div>
              <p className="text-xs text-slate-400 mt-1 max-w-xl">{currentTool.description}</p>
            </div>
          </div>

          {/* Current Global & Active Status Card */}
          <div className="flex items-center gap-3 bg-slate-950/70 p-3 rounded-xl border border-slate-800">
            <div>
              <div className="text-[10px] uppercase font-mono text-slate-400">本机系统环境生效版本</div>
              <div className="text-sm font-bold font-mono text-emerald-400 flex items-center gap-1.5 mt-0.5">
                <Zap className="w-3.5 h-3.5 text-emerald-400" />
                {currentTool.activeVersion ? `v${currentTool.activeVersion}` : '未在 PATH 中检测到'}
              </div>
            </div>
            <div className="h-8 w-px bg-slate-800 mx-1" />
            <div>
              <div className="text-[10px] uppercase font-mono text-slate-400">检测到的已装版本</div>
              <div className="text-sm font-bold font-mono text-blue-400 mt-0.5">
                {currentTool.installedVersions.length} 个
              </div>
            </div>
          </div>
        </div>

        {/* Section 1: Installed Versions Management */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-slate-200 flex items-center gap-2">
              <Globe className="w-4 h-4 text-blue-400" />
              本机已安装版本 (真实探测)
            </h3>
            <span className="text-xs text-slate-400">
              真实读取自系统 PATH 及 Mise 本地库
            </span>
          </div>

          {currentTool.installedVersions.length === 0 ? (
            <div className="p-8 rounded-2xl bg-slate-900/40 border border-dashed border-slate-800 text-center">
              <ShieldAlert className="w-8 h-8 text-amber-400/80 mx-auto mb-2" />
              <p className="text-sm font-medium text-slate-300">本机尚未安装 {currentTool.name} 或未加入环境变量</p>
              <p className="text-xs text-slate-400 mt-1">可在下方在线版本仓库中输入版本号进行真实下载安装</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              {currentTool.installedVersions.map((ver) => {
                const isGlobal = currentTool.globalVersion === ver || currentTool.activeVersion === ver;

                return (
                  <div
                    key={ver}
                    className={`p-4 rounded-xl border transition-all duration-200 flex flex-col justify-between ${
                      isGlobal
                        ? 'bg-blue-950/20 border-blue-500/40 shadow-sm'
                        : 'bg-slate-900/60 border-slate-800 hover:border-slate-700'
                    }`}
                  >
                    <div className="flex items-start justify-between">
                      <div>
                        <div className="font-mono text-base font-bold text-white flex items-center gap-2">
                          v{ver}
                          {isGlobal && (
                            <span className="text-[10px] font-sans px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 flex items-center gap-1 font-semibold">
                              <Check className="w-3 h-3" /> 当前主用
                            </span>
                          )}
                        </div>
                        <div className="text-[11px] text-slate-400 font-mono mt-1">
                          系统真实环境可用
                        </div>
                      </div>

                      <button
                        onClick={() => onUninstallVersion(currentTool.id, ver)}
                        title="卸载此版本"
                        className="p-1.5 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>

                    <div className="mt-4 pt-3 border-t border-slate-800/80 flex items-center justify-between gap-2">
                      {!isGlobal ? (
                        <button
                          onClick={() => onSetGlobalVersion(currentTool.id, ver)}
                          className="flex-1 px-3 py-1.5 rounded-lg bg-blue-600/20 hover:bg-blue-600/30 text-blue-300 border border-blue-500/30 text-xs font-semibold transition-all flex items-center justify-center gap-1"
                        >
                          <Zap className="w-3.5 h-3.5" /> 切换为主用版本
                        </button>
                      ) : (
                        <div className="text-xs text-emerald-400 font-medium flex items-center gap-1 py-1">
                          <Check className="w-3.5 h-3.5" /> 当前终端 PATH 正在调用
                        </div>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Section 2: Remote Versions & 1-Click Install */}
        <div className="space-y-3 pt-4 border-t border-slate-800/80">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div>
              <h3 className="text-sm font-bold text-slate-200 flex items-center gap-2">
                <Download className="w-4 h-4 text-emerald-400" />
                在线版本仓库 (mise registry)
              </h3>
              <p className="text-xs text-slate-400">选择官方发布版本一键在线下载并配置环境</p>
            </div>

            {/* Filter tags & Search */}
            <div className="flex items-center gap-2">
              <div className="flex bg-slate-900 p-0.5 rounded-lg border border-slate-800 text-xs">
                <button
                  onClick={() => setVersionFilter('all')}
                  className={`px-2.5 py-1 rounded-md transition-colors ${
                    versionFilter === 'all' ? 'bg-blue-600 text-white font-medium' : 'text-slate-400 hover:text-slate-200'
                  }`}
                >
                  全部
                </button>
                <button
                  onClick={() => setVersionFilter('lts')}
                  className={`px-2.5 py-1 rounded-md transition-colors ${
                    versionFilter === 'lts' ? 'bg-blue-600 text-white font-medium' : 'text-slate-400 hover:text-slate-200'
                  }`}
                >
                  LTS 长期维护
                </button>
              </div>

              <div className="relative w-36">
                <Search className="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
                <input
                  type="text"
                  placeholder="过滤版本号..."
                  value={remoteSearch}
                  onChange={(e) => setRemoteSearch(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-800 text-xs text-slate-200 rounded-lg pl-7 pr-2.5 py-1.5 focus:outline-none focus:border-blue-500"
                />
              </div>
            </div>
          </div>

          {/* Quick Custom Version Installer */}
          <div className="p-3 rounded-xl bg-slate-900/70 border border-slate-800 flex items-center justify-between gap-3">
            <div className="flex items-center gap-2 text-xs text-slate-300">
              <Terminal className="w-4 h-4 text-slate-400" />
              <span>自定义安装版本/分支 (例如: 22.12.0 / 3.12.7 / latest):</span>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="text"
                placeholder="例如: 22.12.0"
                value={customVersionInput}
                onChange={(e) => setCustomVersionInput(e.target.value)}
                className="bg-slate-950 border border-slate-800 text-xs text-white rounded-lg px-3 py-1 font-mono w-32 focus:outline-none focus:border-blue-500"
              />
              <button
                disabled={!customVersionInput.trim()}
                onClick={() => {
                  if (customVersionInput.trim()) {
                    onOpenInstallModal(currentTool.id, customVersionInput.trim());
                    setCustomVersionInput('');
                  }
                }}
                className="px-3 py-1 rounded-lg bg-blue-600 hover:bg-blue-500 disabled:opacity-40 text-white text-xs font-semibold transition-colors flex items-center gap-1"
              >
                <Plus className="w-3.5 h-3.5" /> 真实安装
              </button>
            </div>
          </div>

          {/* Remote Version Table / Cards */}
          {availableVersionsFiltered.length === 0 ? (
            <div className="p-6 rounded-xl bg-slate-900/30 border border-slate-800/80 text-center">
              <AlertCircle className="w-6 h-6 text-slate-400 mx-auto mb-2" />
              <p className="text-xs text-slate-400">
                暂未从底层 CLI 拉取到远程版本列表。您可以通过上方输入框输入版本号直接执行真实安装。
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-2 max-h-72 overflow-y-auto p-1">
              {availableVersionsFiltered.map((ver) => {
                const isInstalled = currentTool.installedVersions.includes(ver);
                const isLts = ver.includes('20.') || ver.includes('18.') || ver.includes('21.') || ver.includes('3.12');

                return (
                  <div
                    key={ver}
                    className={`p-2.5 rounded-xl border flex flex-col justify-between transition-all ${
                      isInstalled
                        ? 'bg-slate-900/30 border-slate-800/80 opacity-70'
                        : 'bg-slate-900/80 border-slate-800 hover:border-blue-500/40 hover:bg-slate-850'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="font-mono text-xs font-bold text-slate-200">
                        v{ver}
                      </span>
                      {isLts && (
                        <span className="text-[9px] px-1 py-0.2 rounded bg-indigo-500/20 text-indigo-300 font-semibold">
                          LTS
                        </span>
                      )}
                    </div>

                    <div className="mt-2 pt-2 border-t border-slate-800/60 flex justify-end">
                      {isInstalled ? (
                        <span className="text-[10px] text-emerald-400 font-medium flex items-center gap-1">
                          <Check className="w-3 h-3" /> 已就绪
                        </span>
                      ) : (
                        <button
                          onClick={() => onOpenInstallModal(currentTool.id, ver)}
                          className="w-full py-1 rounded-lg bg-slate-800 hover:bg-blue-600 text-slate-300 hover:text-white text-[11px] font-medium transition-colors flex items-center justify-center gap-1"
                        >
                          <Download className="w-3 h-3" /> 安装
                        </button>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
