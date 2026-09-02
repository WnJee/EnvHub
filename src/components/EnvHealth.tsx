import React, { useState } from 'react';
import { EnvHealthCheck } from '../types';
import { 
  ShieldCheck, 
  AlertTriangle, 
  CheckCircle2, 
  Wrench, 
  Terminal, 
  FileCode, 
  RefreshCw,
  Loader2
} from 'lucide-react';
import { useToast } from './Toast';

interface EnvHealthProps {
  healthChecks: EnvHealthCheck[];
  onAutoFix: (checkId: string) => Promise<void>;
  onRefresh: () => void;
}

export const EnvHealth: React.FC<EnvHealthProps> = ({
  healthChecks,
  onAutoFix,
  onRefresh,
}) => {
  const [fixingId, setFixingId] = useState<string | null>(null);
  const toast = useToast();

  const handleFix = async (id: string) => {
    setFixingId(id);
    try {
      await onAutoFix(id);
      toast.success('已自动向您的终端配置文件中写入激活脚本与 PATH，并同步当前环境！');
      onRefresh();
    } catch (err) {
      toast.error(`自动修复失败: ${err}`);
    } finally {
      setFixingId(null);
    }
  };

  const okCount = healthChecks.filter((c) => c.status === 'ok').length;
  const issueCount = healthChecks.filter((c) => c.status !== 'ok').length;

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6 bg-[#090D16]">
      {/* Top Banner */}
      <div className="p-6 rounded-2xl bg-gradient-to-r from-slate-900 via-slate-900 to-emerald-950/30 border border-slate-800/80 flex flex-col sm:flex-row sm:items-center justify-between gap-4 shadow-xl">
        <div>
          <div className="flex items-center gap-2 text-emerald-400 font-mono text-xs uppercase font-semibold">
            <ShieldCheck className="w-4 h-4" />
            Environment & Shell Integration Diagnostics
          </div>
          <h2 className="text-xl font-bold text-white mt-1">环境健康诊断与 Shell 激活</h2>
          <p className="text-xs text-slate-400 mt-1 max-w-xl">
            真实排查本机终端 Shell RC 配置与 GUI 环境变量继承状态，确保命令行与桌面客户端无缝同步。
          </p>
        </div>

        <div className="flex items-center gap-3">
          <div className="px-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-center shrink-0">
            <div className="text-[10px] uppercase font-mono text-slate-400">健康评级</div>
            <div className="text-sm font-bold text-emerald-400 font-mono mt-0.5">
              {issueCount === 0 ? '100% 优良' : `${okCount}/${healthChecks.length} 项就绪`}
            </div>
          </div>
          <button
            onClick={onRefresh}
            className="p-2.5 rounded-xl bg-slate-900 border border-slate-800 text-slate-300 hover:text-white hover:bg-slate-800 transition-colors"
            title="重新真实自检"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Health Checks List */}
      <div className="space-y-3">
        {healthChecks.map((item) => {
          const isOk = item.status === 'ok';
          return (
            <div
              key={item.id}
              className={`p-5 rounded-2xl border transition-all flex flex-col md:flex-row md:items-center justify-between gap-4 ${
                isOk
                  ? 'bg-slate-900/60 border-slate-800/80'
                  : 'bg-rose-950/20 border-rose-500/40 shadow-sm'
              }`}
            >
              <div className="flex items-start gap-4">
                <div className="p-2 rounded-xl mt-0.5 shrink-0">
                  {isOk ? (
                    <div className="w-8 h-8 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
                      <CheckCircle2 className="w-5 h-5" />
                    </div>
                  ) : (
                    <div className="w-8 h-8 rounded-xl bg-rose-500/10 border border-rose-500/20 flex items-center justify-center text-rose-400 animate-pulse">
                      <AlertTriangle className="w-5 h-5" />
                    </div>
                  )}
                </div>

                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <h3 className="text-sm font-bold text-white tracking-tight">
                      {item.title}
                    </h3>
                    <span
                      className={`text-[10px] font-mono px-2 py-0.2 rounded-full font-semibold ${
                        isOk
                          ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                          : 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                      }`}
                    >
                      {isOk ? '正常' : '需配置'}
                    </span>
                  </div>
                  <p className="text-xs text-slate-400">{item.message}</p>
                  <div className="flex items-center gap-3 text-[11px] font-mono text-slate-400 pt-1">
                    <span className="flex items-center gap-1">
                      <Terminal className="w-3 h-3 text-slate-400" />
                      {item.shell}
                    </span>
                    <span>•</span>
                    <span className="flex items-center gap-1">
                      <FileCode className="w-3 h-3 text-slate-400" />
                      {item.configFile}
                    </span>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center justify-end shrink-0">
                {isOk ? (
                  <div className="text-xs text-emerald-400 font-mono font-medium flex items-center gap-1 px-3 py-1.5 rounded-lg bg-emerald-500/5">
                    <ShieldCheck className="w-4 h-4" /> 状态良好
                  </div>
                ) : (
                  <button
                    onClick={() => handleFix(item.id)}
                    disabled={fixingId === item.id}
                    className="px-4 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white text-xs font-semibold shadow-md shadow-blue-500/20 transition-all flex items-center gap-1.5"
                  >
                    {fixingId === item.id ? (
                      <Loader2 className="w-3.5 h-3.5 animate-spin text-white" />
                    ) : (
                      <Wrench className="w-3.5 h-3.5" />
                    )}
                    <span>{fixingId === item.id ? '正在写入...' : '一键自动修复'}</span>
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
