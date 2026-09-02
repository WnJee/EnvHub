import React, { useEffect, useRef } from 'react';
import { 
  Terminal, 
  CheckCircle2, 
  AlertCircle, 
  Copy, 
  X, 
  Loader2,
  Zap
} from 'lucide-react';

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

  useEffect(() => {
    if (terminalEndRef.current) {
      terminalEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs]);

  if (!isOpen) return null;

  const copyLogs = () => {
    navigator.clipboard.writeText(logs.join('\n'));
    alert('已复制全部安装日志到剪贴板！');
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
      <div className="w-full max-w-2xl bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh] animate-in zoom-in-95 duration-150">
        {/* Header */}
        <div className="p-4 border-b border-slate-800 bg-[#0B1120] flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-xl ${
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
              <p className="text-xs text-slate-400">
                {status === 'running' && '异步流式捕获下载与编译进度，UI 保持流畅响应'}
                {status === 'completed' && '安装成功！运行时已编译解压就绪'}
                {status === 'failed' && '安装过程遇到错误，请查看终端输出'}
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

        {/* Progress bar */}
        <div className="p-4 bg-slate-950/60 border-b border-slate-800 space-y-1.5">
          <div className="flex justify-between text-xs font-mono">
            <span className="text-slate-400">
              {status === 'running' && '下载 & 校验 & 释放 Shims...'}
              {status === 'completed' && '完成 100%'}
              {status === 'failed' && '已终止'}
            </span>
            <span className="font-bold text-blue-400">{progress}%</span>
          </div>
          <div className="w-full bg-slate-800 h-2 rounded-full overflow-hidden">
            <div
              className={`h-full transition-all duration-300 ${
                status === 'completed'
                  ? 'bg-emerald-500'
                  : status === 'failed'
                  ? 'bg-rose-500'
                  : 'bg-gradient-to-r from-blue-500 to-cyan-400'
              }`}
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>

        {/* Terminal Logs View */}
        <div className="flex-1 overflow-hidden flex flex-col bg-[#050811] p-4 font-mono text-xs text-slate-300">
          <div className="flex items-center justify-between pb-2 mb-2 border-b border-slate-800/80 text-[11px] text-slate-400">
            <div className="flex items-center gap-1.5">
              <Terminal className="w-3.5 h-3.5 text-blue-400" />
              <span>mise subprocess live stream</span>
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
              const isError = log.includes('error') || log.includes('Error') || log.includes('failed');
              const isSuccess = log.includes('Successfully') || log.includes('completed') || log.includes('matched');
              return (
                <div
                  key={index}
                  className={`leading-relaxed ${
                    isError
                      ? 'text-rose-400 font-semibold'
                      : isSuccess
                      ? 'text-emerald-400 font-semibold'
                      : log.startsWith('[download]')
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
              <span className="flex items-center gap-2 text-blue-400 font-mono">
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                后台子进程运行中...
              </span>
            ) : status === 'completed' ? (
              <span className="text-emerald-400 font-medium">✓ 已就绪，可立即开始开发</span>
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
                className="px-4 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-md shadow-blue-500/20 transition-colors flex items-center gap-1.5"
              >
                <Zap className="w-3.5 h-3.5" /> 设为系统全局主用
              </button>
            )}
            <button
              onClick={onClose}
              disabled={status === 'running'}
              className="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium transition-colors disabled:opacity-40"
            >
              {status === 'completed' ? '完成并关闭' : '关闭'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
