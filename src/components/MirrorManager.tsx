import React, { useState } from 'react';
import { MirrorConfig } from '../types';
import { 
  Zap, 
  Check, 
  Server, 
  Activity, 
  FileText, 
  ArrowRight
} from 'lucide-react';

interface MirrorManagerProps {
  mirrors: MirrorConfig[];
  onSetMirror: (tool: string, mirrorUrl: string) => void;
  onPingMirrors: () => Promise<void>;
  isPinging: boolean;
}

export const MirrorManager: React.FC<MirrorManagerProps> = ({
  mirrors,
  onSetMirror,
  onPingMirrors,
  isPinging,
}) => {
  const [activeTab, setActiveTab] = useState<string>(mirrors[0]?.tool || 'npm');
  const [successToast, setSuccessToast] = useState<string | null>(null);

  const currentMirrorConfig = mirrors.find((m) => m.tool === activeTab) || mirrors[0];

  const handleApply = (tool: string, url: string, name: string) => {
    onSetMirror(tool, url);
    setSuccessToast(`已成功将 ${tool.toUpperCase()} 镜像源切换为「${name}」并写入本地配置！`);
    setTimeout(() => {
      setSuccessToast(null);
    }, 3500);
  };

  const getPingColor = (ping?: number) => {
    if (!ping) return 'text-slate-400 bg-slate-800';
    if (ping < 50) return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
    if (ping < 150) return 'text-amber-400 bg-amber-500/10 border-amber-500/20';
    return 'text-rose-400 bg-rose-500/10 border-rose-500/20';
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6 bg-[#090D16]">
      {/* Top Banner */}
      <div className="p-6 rounded-2xl bg-gradient-to-r from-amber-950/30 via-slate-900 to-slate-900 border border-slate-800/80 flex flex-col sm:flex-row sm:items-center justify-between gap-4 shadow-xl">
        <div>
          <div className="flex items-center gap-2 text-amber-400 font-mono text-xs uppercase font-semibold">
            <Zap className="w-4 h-4" />
            Domestic Network & Mirror Acceleration
          </div>
          <h2 className="text-xl font-bold text-white mt-1">国内镜像源一键加速与测速</h2>
          <p className="text-xs text-slate-400 mt-1 max-w-xl">
            解决 Node/Python/Go/Rust/Homebrew 在国内下载依赖经常超时、失败的问题。一键实时测试全国 CDN 延迟并自动写入系统配置。
          </p>
        </div>

        <button
          onClick={onPingMirrors}
          disabled={isPinging}
          className="px-4 py-2.5 rounded-xl bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white text-xs font-semibold shadow-lg shadow-amber-500/20 flex items-center gap-2 transition-all shrink-0"
        >
          <Activity className={`w-4 h-4 ${isPinging ? 'animate-spin' : ''}`} />
          {isPinging ? '正在测试全国节点延迟...' : '一键测速全部镜像'}
        </button>
      </div>

      {/* Success Notification */}
      {successToast && (
        <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-xs font-medium flex items-center gap-2 animate-in fade-in slide-in-from-top-2">
          <Check className="w-4 h-4 shrink-0" />
          <span>{successToast}</span>
        </div>
      )}

      {/* Mirror Tabs */}
      <div className="flex bg-slate-900/80 p-1.5 rounded-2xl border border-slate-800 gap-1.5 overflow-x-auto">
        {mirrors.map((m) => {
          const isActive = m.tool === activeTab;
          return (
            <button
              key={m.id}
              onClick={() => setActiveTab(m.tool)}
              className={`px-4 py-2 rounded-xl text-xs font-semibold transition-all flex items-center gap-2 shrink-0 ${
                isActive
                  ? 'bg-blue-600 text-white shadow-md shadow-blue-500/20'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'
              }`}
            >
              <Server className="w-3.5 h-3.5" />
              <span>{m.name}</span>
            </button>
          );
        })}
      </div>

      {/* Active Mirror Options Card */}
      {currentMirrorConfig && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-slate-200 flex items-center gap-2">
              <FileText className="w-4 h-4 text-blue-400" />
              {currentMirrorConfig.name} 可用高速镜像源列表
            </h3>
            <span className="text-xs text-slate-400 font-mono">
              当前生效: <span className="text-blue-400">{currentMirrorConfig.currentMirror}</span>
            </span>
          </div>

          <div className="grid grid-cols-1 gap-3">
            {currentMirrorConfig.options.map((opt) => {
              const isCurrent = currentMirrorConfig.currentMirror === opt.url;
              return (
                <div
                  key={opt.url}
                  className={`p-4 rounded-xl border transition-all flex flex-col sm:flex-row sm:items-center justify-between gap-4 ${
                    isCurrent
                      ? 'bg-blue-950/20 border-blue-500/40 shadow-sm'
                      : 'bg-slate-900/60 border-slate-800 hover:border-slate-700'
                  }`}
                >
                  <div className="space-y-1 min-w-0">
                    <div className="flex items-center gap-2.5">
                      <span className="text-sm font-bold text-white tracking-tight">
                        {opt.name}
                      </span>
                      {opt.isDefault && (
                        <span className="text-[10px] px-1.5 py-0.2 rounded bg-indigo-500/20 text-indigo-300 font-semibold border border-indigo-500/30">
                          推荐
                        </span>
                      )}
                      {isCurrent && (
                        <span className="text-[10px] px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400 font-semibold border border-blue-500/30 flex items-center gap-1">
                          <Check className="w-3 h-3" /> 正在使用
                        </span>
                      )}
                    </div>
                    <div className="text-xs text-slate-400 font-mono truncate">
                      {opt.url}
                    </div>
                  </div>

                  <div className="flex items-center gap-3 shrink-0">
                    {/* Ping Badge */}
                    <div
                      className={`px-2.5 py-1 rounded-lg border text-xs font-mono font-semibold flex items-center gap-1.5 ${getPingColor(
                        opt.ping
                      )}`}
                    >
                      <Activity className="w-3 h-3" />
                      <span>{opt.ping ? `${opt.ping} ms` : '未测速'}</span>
                    </div>

                    {/* Switch Button */}
                    {isCurrent ? (
                      <div className="px-3.5 py-1.5 rounded-lg bg-emerald-500/10 text-emerald-400 text-xs font-medium flex items-center gap-1 border border-emerald-500/20">
                        <Check className="w-3.5 h-3.5" /> 已生效
                      </div>
                    ) : (
                      <button
                        onClick={() =>
                          handleApply(currentMirrorConfig.tool, opt.url, opt.name)
                        }
                        className="px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-sm transition-colors flex items-center gap-1.5"
                      >
                        <span>应用此镜像</span>
                        <ArrowRight className="w-3 h-3" />
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};
