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
import { api } from '../services/tauri';
import { useToast } from './Toast';

interface RuntimeManagerProps {
  runtimes: RuntimeTool[];
  searchQuery: string;
  onSetGlobalVersion: (toolId: string, version: string) => void;
  onUninstallVersion: (toolId: string, version: string) => void;
  onOpenInstallModal: (toolId: string, version: string) => void;
}

export const formatVersion = (v: string | undefined | null): string => {
  if (!v) return '';
  const trimmed = v.trim();
  return trimmed.startsWith('v') ? trimmed : `v${trimmed}`;
};

export const filterLatestMinorVersions = (versions: string[]): string[] => {
  const groups = new Map<string, { patch: number; raw: string; isPrerelease: boolean }>();
  const nonSemver: string[] = [];

  for (const v of versions) {
    const trimmed = v.trim().replace(/^v/, '');
    const parts = trimmed.split('.');
    if (parts.length >= 2) {
      const maj = parseInt(parts[0], 10);
      const min = parseInt(parts[1], 10);
      if (!isNaN(maj) && !isNaN(min)) {
        const key = `${maj}.${min}`;
        const rawPatch = parts[2] || '0';
        const isPrerelease = rawPatch.includes('-') || rawPatch.includes('rc') || rawPatch.includes('beta') || rawPatch.includes('alpha');
        const match = rawPatch.match(/^\d+/);
        const patchNum = match ? parseInt(match[0], 10) : 0;

        const existing = groups.get(key);
        if (!existing) {
          groups.set(key, { patch: patchNum, raw: v, isPrerelease });
        } else {
          if (existing.isPrerelease && !isPrerelease) {
            groups.set(key, { patch: patchNum, raw: v, isPrerelease });
          } else if (!isPrerelease && !existing.isPrerelease) {
            if (patchNum > existing.patch) {
              groups.set(key, { patch: patchNum, raw: v, isPrerelease });
            }
          } else if (isPrerelease && existing.isPrerelease) {
            if (patchNum > existing.patch) {
              groups.set(key, { patch: patchNum, raw: v, isPrerelease });
            }
          }
        }
        continue;
      }
    }
    if (!nonSemver.includes(v)) {
      nonSemver.push(v);
    }
  }

  const sorted = Array.from(groups.entries())
    .map(([key, val]) => {
      const [maj, min] = key.split('.').map(Number);
      return { maj, min, patch: val.patch, raw: val.raw };
    })
    .sort((a, b) => {
      if (a.maj !== b.maj) return b.maj - a.maj;
      if (a.min !== b.min) return b.min - a.min;
      return b.patch - a.patch;
    })
    .map((item) => item.raw);

  return [...sorted, ...nonSemver];
};

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
  const [testingVersion, setTestingVersion] = useState<string | null>(null);
  const toast = useToast();

  const handleQuickTerminalCheck = async (toolId: string, version: string) => {
    setTestingVersion(version);
    try {
      const output = await api.openTerminalForRuntime(toolId, version);
      toast.success(output, `已唤起终端验证 ${toolId}`);
    } catch (err) {
      toast.error(`打开终端失败: ${err}`);
    } finally {
      setTestingVersion(null);
    }
  };

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
        <h3 className="text-base font-semibold text-slate-300">正在检查开发环境运行时...</h3>
        <p className="text-xs text-slate-400 mt-1">如无数据请检查是否安装了对应语言编译器或 Mise CLI</p>
      </div>
    );
  }

  // Dynamic version hint generation based on current tool
  const getToolExampleHint = (tool: RuntimeTool) => {
    if (tool.availableVersions && tool.availableVersions.length >= 2) {
      return `${tool.availableVersions[0]} / ${tool.availableVersions[1]} / latest`;
    }
    switch (tool.id) {
      case 'node':
        return '25.9.0 / 25.8.2 / lts';
      case 'python':
        return '3.13.2 / 3.12.9 / latest';
      case 'go':
        return '1.24.0 / 1.23.6';
      case 'rust':
        return '1.85.0 / 1.84.1';
      case 'java':
        return '21.0.6 / 17.0.14';
      case 'ruby':
        return '3.4.2 / 3.3.7';
      case 'bun':
        return '1.2.4 / 1.1.43';
      case 'deno':
        return '2.2.3 / 2.1.10';
      case 'php':
        return '8.4.4 / 8.3.17';
      default:
        return 'latest';
    }
  };

  const getToolPlaceholder = (tool: RuntimeTool) => {
    if (tool.availableVersions && tool.availableVersions.length > 0) {
      return `例如: ${tool.availableVersions[0]}`;
    }
    switch (tool.id) {
      case 'node':
        return '例如: 25.9.0';
      case 'python':
        return '例如: 3.13.2';
      case 'go':
        return '例如: 1.24.0';
      case 'rust':
        return '例如: 1.85.0';
      case 'java':
        return '例如: 21.0.6';
      case 'ruby':
        return '例如: 3.4.2';
      default:
        return '例如: latest';
    }
  };

  // Filter and deduplicate remote versions to only highest patch per minor release
  const curatedVersions = filterLatestMinorVersions(currentTool.availableVersions || []);
  const availableVersionsFiltered = curatedVersions.filter((v) => {
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
      <div className="w-56 sm:w-64 border-r border-slate-800/80 bg-[#0B1120]/60 flex flex-col shrink-0">
        <div className="p-3.5 border-b border-slate-800/80 flex items-center justify-between">
          <div className="text-xs font-bold uppercase tracking-wider text-slate-400 font-mono flex items-center gap-1.5">
            <Layers className="w-3.5 h-3.5 text-blue-400" />
            开发语言 ({filteredRuntimes.length})
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
                className={`w-full flex items-center justify-between p-2.5 rounded-xl transition-all duration-200 text-left group ${
                  isSelected
                    ? 'bg-blue-600/15 border border-blue-500/30 text-white shadow-sm'
                    : 'hover:bg-slate-800/40 border border-transparent text-slate-300'
                }`}
              >
                <div className="flex items-center gap-2.5 min-w-0">
                  <div className="w-8 h-8 rounded-lg bg-slate-900 border border-slate-800 flex items-center justify-center p-1 shrink-0 group-hover:scale-105 transition-transform">
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
                    <div className="text-xs font-semibold flex items-center gap-1.5">
                      {tool.name}
                      {tool.activeVersion && (
                        <span className="text-[9px] font-mono px-1 py-0.2 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">
                          {formatVersion(tool.activeVersion)}
                        </span>
                      )}
                    </div>
                    <div className="text-[10px] text-slate-400 truncate">{tool.description}</div>
                  </div>
                </div>

                <div className="flex items-center gap-1.5 pl-1.5 shrink-0">
                  <span className={`text-[9px] px-1.5 py-0.2 rounded-md font-mono ${
                    installedCount > 0 
                      ? 'bg-blue-500/20 text-blue-300 border border-blue-500/30' 
                      : 'bg-slate-900 text-slate-400'
                  }`}>
                    {installedCount > 0 ? `${installedCount} 个` : '未装'}
                  </span>
                  <ChevronRight className={`w-3 h-3 transition-transform ${isSelected ? 'text-blue-400 translate-x-0.5' : 'text-slate-400'}`} />
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Right Detail Pane */}
      <div className="flex-1 flex flex-col overflow-y-auto p-4 sm:p-6 space-y-5">
        {/* Tool Header Banner */}
        <div className="p-4 sm:p-5 rounded-2xl bg-gradient-to-r from-slate-900/90 via-slate-900/70 to-blue-950/20 border border-slate-800/80 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 shadow-xl">
          <div className="flex items-center gap-3.5 min-w-0">
            <div className="w-12 h-12 rounded-xl bg-slate-900/90 border border-slate-700/60 p-2.5 shadow-inner flex items-center justify-center shrink-0">
              <img 
                src={currentTool.icon} 
                alt={currentTool.name} 
                className="w-full h-full object-contain" 
              />
            </div>
            <div className="min-w-0">
              <div className="flex items-center gap-2.5">
                <h2 className="text-lg sm:text-xl font-bold text-white tracking-tight">{currentTool.name}</h2>
                <button
                  onClick={() => api.openUrl(currentTool.officialSite)}
                  className="text-xs text-slate-400 hover:text-blue-400 flex items-center gap-1 transition-colors"
                  title="打开官方网站"
                >
                  官网文档 <ExternalLink className="w-3 h-3" />
                </button>
              </div>
              <p className="text-xs text-slate-400 mt-0.5 truncate max-w-lg">{currentTool.description}</p>
            </div>
          </div>

          {/* Current Global & Active Status Card */}
          <div className="flex items-center gap-3 bg-slate-950/70 px-3.5 py-2 rounded-xl border border-slate-800 shrink-0">
            <div>
              <div className="text-[9px] uppercase font-mono text-slate-400">当前主用版本</div>
              <div className="text-xs font-bold font-mono text-emerald-400 flex items-center gap-1 mt-0.5">
                <Zap className="w-3 h-3 text-emerald-400" />
                {currentTool.activeVersion ? formatVersion(currentTool.activeVersion) : '未在 PATH 中'}
              </div>
            </div>
            <div className="h-6 w-px bg-slate-800 mx-1" />
            <div>
              <div className="text-[9px] uppercase font-mono text-slate-400">已安装版本</div>
              <div className="text-xs font-bold font-mono text-blue-400 mt-0.5">
                {currentTool.installedVersions.length} 个
              </div>
            </div>
          </div>
        </div>

        {/* Section 1: Installed Versions */}
        <div className="space-y-2.5">
          <div className="flex items-center justify-between">
            <h3 className="text-xs sm:text-sm font-bold text-slate-200 flex items-center gap-2">
              <Globe className="w-3.5 h-3.5 text-blue-400" />
              本地已安装版本
            </h3>
            <span className="text-[11px] text-slate-400">
              管理系统 PATH 与全局默认版本
            </span>
          </div>

          {currentTool.installedVersions.length === 0 ? (
            <div className="p-6 rounded-2xl bg-slate-900/40 border border-dashed border-slate-800 text-center">
              <ShieldAlert className="w-7 h-7 text-amber-400/80 mx-auto mb-2" />
              <p className="text-xs sm:text-sm font-medium text-slate-300">本机尚未安装 {currentTool.name}</p>
              <p className="text-[11px] text-slate-400 mt-0.5">可在下方版本仓库中选择或输入版本号进行安装</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
              {currentTool.installedVersions.map((ver) => {
                const isGlobal = currentTool.globalVersion === ver || currentTool.activeVersion === ver;

                return (
                  <div
                    key={ver}
                    className={`p-3.5 rounded-xl border transition-all duration-200 flex flex-col justify-between ${
                      isGlobal
                        ? 'bg-blue-950/20 border-blue-500/40 shadow-sm'
                        : 'bg-slate-900/60 border-slate-800 hover:border-slate-700'
                    }`}
                  >
                    <div className="flex items-start justify-between">
                      <div>
                        <div className="font-mono text-sm sm:text-base font-bold text-white flex items-center gap-2">
                          {formatVersion(ver)}
                          {isGlobal && (
                            <span className="text-[9px] font-sans px-2 py-0.2 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 flex items-center gap-1 font-semibold">
                              <Check className="w-2.5 h-2.5" /> 主用
                            </span>
                          )}
                        </div>
                        <div className="text-[10px] text-slate-400 font-mono mt-0.5">
                          环境已就绪
                        </div>
                      </div>

                      <button
                        onClick={() => onUninstallVersion(currentTool.id, ver)}
                        title="卸载此版本"
                        className="p-1 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>

                    <div className="mt-3 pt-2.5 border-t border-slate-800/80 flex items-center justify-between gap-2">
                      {!isGlobal ? (
                        <button
                          onClick={() => onSetGlobalVersion(currentTool.id, ver)}
                          className="flex-1 px-3 py-1 rounded-lg bg-blue-600/20 hover:bg-blue-600/30 text-blue-300 border border-blue-500/30 text-xs font-semibold transition-all flex items-center justify-center gap-1"
                        >
                          <Zap className="w-3 h-3" /> 设为主用版本
                        </button>
                      ) : (
                        <div className="text-xs text-emerald-400 font-medium flex items-center gap-1 py-0.5">
                          <Check className="w-3 h-3" /> 命令行终端默认调用
                        </div>
                      )}

                      <button
                        onClick={() => handleQuickTerminalCheck(currentTool.id, ver)}
                        disabled={testingVersion === ver}
                        title="在系统终端中快捷打开并验证此版本"
                        className="px-2.5 py-1 rounded-lg bg-slate-800/90 hover:bg-slate-700 text-slate-300 hover:text-white text-[11px] font-medium border border-slate-700/60 transition-all flex items-center gap-1 shrink-0"
                      >
                        <Terminal className="w-3 h-3 text-emerald-400" />
                        <span>终端查看</span>
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Section 2: Remote Versions & Install */}
        <div className="space-y-2.5 pt-3 border-t border-slate-800/80">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2.5">
            <div>
              <h3 className="text-xs sm:text-sm font-bold text-slate-200 flex items-center gap-2">
                <Download className="w-3.5 h-3.5 text-emerald-400" />
                在线版本仓库
              </h3>
              <p className="text-[11px] text-slate-400">选择官方发布版本一键下载并配置环境</p>
            </div>

            {/* Filter tags & Search */}
            <div className="flex items-center gap-2">
              <div className="flex bg-slate-900 p-0.5 rounded-lg border border-slate-800 text-xs">
                <button
                  onClick={() => setVersionFilter('all')}
                  className={`px-2 py-0.5 rounded-md transition-colors text-xs ${
                    versionFilter === 'all' ? 'bg-blue-600 text-white font-medium' : 'text-slate-400 hover:text-slate-200'
                  }`}
                >
                  全部
                </button>
                <button
                  onClick={() => setVersionFilter('lts')}
                  className={`px-2 py-0.5 rounded-md transition-colors text-xs ${
                    versionFilter === 'lts' ? 'bg-blue-600 text-white font-medium' : 'text-slate-400 hover:text-slate-200'
                  }`}
                >
                  LTS
                </button>
              </div>

              <div className="relative w-32 sm:w-36">
                <Search className="w-3 h-3 text-slate-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
                <input
                  type="text"
                  placeholder="过滤版本..."
                  value={remoteSearch}
                  onChange={(e) => setRemoteSearch(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-800 text-xs text-slate-200 rounded-lg pl-7 pr-2 py-1 focus:outline-none focus:border-blue-500"
                />
              </div>
            </div>
          </div>

          {/* Dynamic Custom Version Installer */}
          <div className="p-2.5 sm:p-3 rounded-xl bg-slate-900/70 border border-slate-800 flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div className="flex items-center gap-2 text-xs text-slate-300">
              <Terminal className="w-3.5 h-3.5 text-slate-400" />
              <span>安装指定版本 (例如: {getToolExampleHint(currentTool)}):</span>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="text"
                placeholder={getToolPlaceholder(currentTool)}
                value={customVersionInput}
                onChange={(e) => setCustomVersionInput(e.target.value)}
                className="bg-slate-950 border border-slate-800 text-xs text-white rounded-lg px-2.5 py-1 font-mono w-32 focus:outline-none focus:border-blue-500"
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
                <Plus className="w-3 h-3" /> 一键安装
              </button>
            </div>
          </div>

          {/* Remote Version Table / Cards */}
          {availableVersionsFiltered.length === 0 ? (
            <div className="p-5 rounded-xl bg-slate-900/30 border border-slate-800/80 text-center">
              <AlertCircle className="w-5 h-5 text-slate-400 mx-auto mb-1.5" />
              <p className="text-xs text-slate-400">
                暂未获取到在线版本列表。可通过上方输入框直接输入版本号进行安装。
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-2 max-h-64 overflow-y-auto p-1">
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
                        {formatVersion(ver)}
                      </span>
                      {isLts && (
                        <span className="text-[9px] px-1 py-0.2 rounded bg-indigo-500/20 text-indigo-300 font-semibold">
                          LTS
                        </span>
                      )}
                    </div>

                    <div className="mt-2 pt-1.5 border-t border-slate-800/60 flex justify-end">
                      {isInstalled ? (
                        <span className="text-[10px] text-emerald-400 font-medium flex items-center gap-1">
                          <Check className="w-2.5 h-2.5" /> 已就绪
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
