import React, { useEffect, useRef } from 'react';
import { 
  Terminal, 
  CheckCircle2, 
  AlertCircle, 
  Copy, 
  X, 
  Loader2,
  Zap,
  ArrowDownCircle
} from 'lucide-react';
import { useToast } from './Toast';

interface InstallModalProps {
  isOpen: boolean;
  toolId: string;
  version: string;
  logs: string[];
  progress: number;
  status: 'idle' | 'running' | 'completed' | 'failed';
  onClose: () => void;
  onSetGlobal: () => void;
}

export const InstallModal: React.FC<InstallModalProps> = ({
  isOpen,
  toolId,
  version,
  logs,
  progress,
  status,
  onClose,
  onSetGlobal,
}) => {
  const terminalEndRef = useRef<HTMLDivElement>(null);
  const toast = useToast();

  useEffect(() => {
    if (terminalEndRef.current) {
      terminalEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs]);

  if (!isOpen) return null;

  const copyLogs = () => {
    navigator.clipboard.writeText(logs.join('\n'));
    toast.success('已复制安装日志到剪贴板');
  };

  const getStageText = () => {
    if (status === 'completed') return '安装完成，环境已就绪';
    if (status === 'failed') return '安装未完成';
    if (progress < 25) return '正在连接镜像源并解析版本...';
    if (progress < 60) return '正在下载官方资源包...';
    if (progress < 85) return '正在校验完整性与解压...';
    return '正在配置环境变量与 Shims...';
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md animate-in fade-in duration-200">
      <div className="w-full max-w-2xl bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh] animate-in zoom-in-95 duration-150">
        {/* Header */}
        <div className="p-4 border-b border-slate-800 bg-[#0B1120] flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-xl shrink-0 ${
              status === 'completed'
                ? 'bg-emerald-500/20 text-emerald-400'
                : status === 'failed'
                ? 'bg-rose-500/20 text-rose-400'
                : 'bg-blue-500/20 text-blue-400'
            }`}>
              {status === 'completed' ? (
                <CheckCircle2 className="w-5 h-5" />
              ) : status === 'failed' ? (
                <AlertCircle className="w-5 h-5" />
              ) : (
                <Loader2 className="w-5 h-5 animate-spin" />
              )}
            </div>
            <div>
              <h3 className="text-sm font-bold text-white flex items-center gap-2">
                正在安装 {toolId.toUpperCase()}
                <span className="font-mono text-blue-400">v{version}</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                {status === 'running' && '正在下载并部署开发环境...'}
                {status === 'completed' && '安装成功！环境已配置就绪'}
                {status === 'failed' && '安装未完成，请查看下方详细日志'}
              </p>
            </div>
          </div>

          <button
            onClick={onClose}
            disabled={status === 'running'}
            className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 disabled:opacity-30 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Dynamic Progress Bar */}
        <div className="p-4 bg-slate-950/70 border-b border-slate-800 space-y-2">
          <div className="flex items-center justify-between text-xs">
            <div className="flex items-center gap-2">
              <ArrowDownCircle className={`w-3.5 h-3.5 ${status === 'running' ? 'text-blue-400 animate-bounce' : 'text-slate-400'}`} />
              <span className="text-slate-300 font-medium">{getStageText()}</span>
            </div>
            <span className="font-mono font-bold text-blue-400 text-xs px-2 py-0.5 rounded-md bg-blue-500/10 border border-blue-500/20">
              {progress}%
            </span>
          </div>

          {/* Progress Track */}
          <div className="w-full bg-slate-800/80 h-2.5 rounded-full overflow-hidden p-0.5 border border-slate-700/50">
            <div
              className={`h-full rounded-full transition-all duration-300 relative ${
                status === 'completed'
                  ? 'bg-emerald-500 shadow-sm shadow-emerald-500/50'
                  : status === 'failed'
                  ? 'bg-rose-500'
                  : 'bg-gradient-to-r from-blue-600 via-indigo-500 to-cyan-400 shadow-sm shadow-cyan-500/30'
              }`}
              style={{ width: `${Math.max(5, progress)}%` }}
            >
              {status === 'running' && (
                <div className="absolute inset-0 bg-white/20 animate-pulse" />
              )}
            </div>
          </div>
        </div>

        {/* Terminal Logs View */}
        <div className="flex-1 overflow-hidden flex flex-col bg-[#050811] p-4 font-mono text-xs text-slate-300">
          <div className="flex items-center justify-between pb-2 mb-2 border-b border-slate-800/80 text-[11px] text-slate-400">
            <div className="flex items-center gap-1.5">
              <Terminal className="w-3.5 h-3.5 text-blue-400" />
              <span>安装日志输出</span>
            </div>
            <button
              onClick={copyLogs}
              className="flex items-center gap-1 text-slate-400 hover:text-slate-200 transition-colors"
            >
              <Copy className="w-3 h-3" />
              <span>复制日志</span>
            </button>
          </div>

          <div className="flex-1 overflow-y-auto space-y-1 select-text">
            {logs.map((log, index) => {
              const isError = log.includes('error') || log.includes('Error') || log.includes('failed') || log.includes('✗');
              const isSuccess = log.includes('Successfully') || log.includes('completed') || log.includes('成功') || log.includes('✓');
              return (
                <div
                  key={index}
                  className={`leading-relaxed break-all ${
                    isError
                      ? 'text-rose-400 font-semibold'
                      : isSuccess
                      ? 'text-emerald-400 font-semibold'
                      : log.startsWith('[download]') || log.includes('GET ')
                      ? 'text-cyan-300'
                      : 'text-slate-300'
                  }`}
                >
                  {log}
                </div>
              );
            })}
            <div ref={terminalEndRef} />
          </div>
        </div>

        {/* Footer Actions */}
        <div className="p-4 bg-[#0B1120] border-t border-slate-800 flex items-center justify-between">
          <div className="text-xs text-slate-400">
            {status === 'running' ? (
              <span className="flex items-center gap-2 text-blue-400 font-medium">
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                正在安装中，请稍候...
              </span>
            ) : status === 'completed' ? (
              <span className="text-emerald-400 font-medium">✓ 已完成安装，可立即使用</span>
            ) : (
              <span className="text-slate-400">可关闭窗口或重试</span>
            )}
          </div>

          <div className="flex items-center gap-2">
            {status === 'completed' && (
              <button
                onClick={() => {
                  onSetGlobal();
                  onClose();
                }}
                className="px-3.5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-md shadow-blue-500/20 transition-colors flex items-center gap-1.5"
              >
                <Zap className="w-3.5 h-3.5" /> 设为全局主用
              </button>
            )}
            <button
              onClick={onClose}
              disabled={status === 'running'}
              className="px-4 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium transition-colors disabled:opacity-40"
            >
              {status === 'completed' ? '完成' : '关闭'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
