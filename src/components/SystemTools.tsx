import React, { useState } from 'react';
import { SystemTool } from '../types';
import { 
  Wrench, 
  Check, 
  Download, 
  ExternalLink, 
  Terminal, 
  PackageCheck, 
  ShieldCheck,
  Loader2
} from 'lucide-react';
import { api } from '../services/tauri';
import { useToast } from './Toast';

interface SystemToolsProps {
  systemTools: SystemTool[];
  packageManager: string;
  onInstallSystemTool: (toolId: string) => void;
}

export const SystemTools: React.FC<SystemToolsProps> = ({
  systemTools,
  packageManager,
  onInstallSystemTool,
}) => {
  const [testingId, setTestingId] = useState<string | null>(null);
  const toast = useToast();

  const handleTestTool = async (toolId: string, toolName: string) => {
    setTestingId(toolId);
    try {
      const output = await api.testSystemTool(toolId);
      toast.success(output, `${toolName} 测试运行正常`);
    } catch (err) {
      toast.error(`测试执行失败: ${err}`, `${toolName} 未响应`);
    } finally {
      setTestingId(null);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6 bg-[#090D16]">
      {/* Top Banner */}
      <div className="p-6 rounded-2xl bg-gradient-to-r from-slate-900 via-slate-900 to-blue-950/30 border border-slate-800/80 flex flex-col sm:flex-row sm:items-center justify-between gap-4 shadow-xl">
        <div>
          <div className="flex items-center gap-2 text-blue-400 font-mono text-xs uppercase font-semibold">
            <Wrench className="w-4 h-4" />
            System Level CLI Package Router
          </div>
          <h2 className="text-xl font-bold text-white mt-1">系统级开发基建与工具箱</h2>
          <p className="text-xs text-slate-400 mt-1 max-w-xl">
            跨平台自动适配宿主包管理器 (macOS <code className="text-blue-400">Homebrew</code> / Windows <code className="text-blue-400">winget</code> / Linux <code className="text-blue-400">apt</code>)，真实检测与安装底层依赖。
          </p>
        </div>

        <div className="flex items-center gap-3 bg-slate-950/80 px-4 py-2.5 rounded-xl border border-slate-800 shrink-0">
          <PackageCheck className="w-5 h-5 text-emerald-400" />
          <div>
            <div className="text-[10px] uppercase font-mono text-slate-400">当前宿主包管理器</div>
            <div className="text-xs font-bold text-white font-mono uppercase">{packageManager} (Ready)</div>
          </div>
        </div>
      </div>

      {/* System Tools Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {systemTools.map((tool) => (
          <div
            key={tool.id}
            className="p-5 rounded-2xl bg-slate-900/70 border border-slate-800 hover:border-slate-700 transition-all flex flex-col justify-between shadow-sm group"
          >
            <div>
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center p-2 shrink-0">
                    <img
                      src={tool.icon}
                      alt={tool.name}
                      className="w-full h-full object-contain"
                      onError={(e) => {
                        (e.target as HTMLElement).style.display = 'none';
                      }}
                    />
                  </div>
                  <div>
                    <h3 className="text-sm font-bold text-white flex items-center gap-2">
                      {tool.name}
                      <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-slate-800 text-slate-400 font-normal">
                        {tool.category}
                      </span>
                    </h3>
                    <a
                      href={tool.homepage}
                      target="_blank"
                      rel="noreferrer"
                      className="text-[11px] text-slate-400 hover:text-blue-400 flex items-center gap-1 transition-colors mt-0.5"
                    >
                      官方主页 <ExternalLink className="w-2.5 h-2.5" />
                    </a>
                  </div>
                </div>

                {tool.isInstalled ? (
                  <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-semibold flex items-center gap-1">
                    <Check className="w-3 h-3" /> 本机已装
                  </span>
                ) : (
                  <span className="px-2 py-0.5 rounded-full bg-slate-800 text-slate-400 border border-slate-700 text-[10px] font-medium">
                    未检测到
                  </span>
                )}
              </div>

              <p className="text-xs text-slate-400 mt-3 line-clamp-2">
                {tool.description}
              </p>

              {/* Install Command Preview */}
              <div className="mt-3 p-2 rounded-lg bg-slate-950 border border-slate-800/80 font-mono text-[11px] text-slate-300 flex items-center justify-between">
                <div className="flex items-center gap-1.5 truncate text-slate-400">
                  <Terminal className="w-3 h-3 text-slate-400 shrink-0" />
                  <span className="truncate">{tool.installCommand}</span>
                </div>
              </div>
            </div>

            {/* Bottom Status / Install Action */}
            <div className="mt-4 pt-3 border-t border-slate-800/80 flex items-center justify-between">
              {tool.isInstalled ? (
                <div className="text-xs font-mono text-slate-300 flex items-center gap-1.5">
                  <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
                  <span>v{tool.installedVersion}</span>
                </div>
              ) : (
                <div className="text-[11px] text-slate-400">
                  未就绪，可一键安装
                </div>
              )}

              {tool.isInstalled ? (
                <button
                  onClick={() => handleTestTool(tool.id, tool.name)}
                  disabled={testingId === tool.id}
                  className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition-colors flex items-center gap-1"
                >
                  {testingId === tool.id ? (
                    <>
                      <Loader2 className="w-3.5 h-3.5 animate-spin text-blue-400" />
                      <span>测试中...</span>
                    </>
                  ) : (
                    '真实运行测试'
                  )}
                </button>
              ) : (
                <button
                  onClick={() => onInstallSystemTool(tool.id)}
                  className="px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-md shadow-blue-500/20 transition-colors flex items-center gap-1.5"
                >
                  <Download className="w-3.5 h-3.5" /> 一键调用安装
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
