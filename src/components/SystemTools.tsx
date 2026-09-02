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
  Loader2,
  AlertCircle
} from 'lucide-react';
import { api } from '../services/tauri';
import { useToast } from './Toast';

interface SystemToolsProps {
  systemTools: SystemTool[];
  searchQuery: string;
  packageManager: string;
  onInstallSystemTool: (toolId: string) => void;
}

export const SystemTools: React.FC<SystemToolsProps> = ({
  systemTools,
  searchQuery,
  packageManager,
  onInstallSystemTool,
}) => {
  const [testingId, setTestingId] = useState<string | null>(null);
  const toast = useToast();

  const filteredTools = systemTools.filter(
    (t) =>
      t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.id.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.category.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const handleTestTool = async (toolId: string, toolName: string) => {
    setTestingId(toolId);
    try {
      const output = await api.testSystemTool(toolId);
      toast.success(output, `${toolName} 运行正常`);
    } catch (err) {
      toast.error(`测试执行失败: ${err}`, `${toolName} 未响应`);
    } finally {
      setTestingId(null);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-4 sm:p-6 space-y-5 bg-[#090D16]">
      {/* Top Banner */}
      <div className="p-4 sm:p-5 rounded-2xl bg-gradient-to-r from-slate-900 via-slate-900 to-blue-950/30 border border-slate-800/80 flex flex-col sm:flex-row sm:items-center justify-between gap-4 shadow-xl">
        <div>
          <div className="flex items-center gap-2 text-blue-400 font-mono text-xs uppercase font-semibold">
            <Wrench className="w-3.5 h-3.5" />
            System CLI Tools & Middlewares
          </div>
          <h2 className="text-lg sm:text-xl font-bold text-white mt-1">系统核心基建、数据库与工具箱</h2>
          <p className="text-xs text-slate-400 mt-0.5 max-w-xl">
            跨平台自动适配宿主包管理器 (macOS <code className="text-blue-400">Homebrew</code> / Windows <code className="text-blue-400">winget</code> / Linux <code className="text-blue-400">apt</code>)，管理 Docker、数据库与开发中间件。
          </p>
        </div>

        <div className="flex items-center gap-2.5 bg-slate-950/80 px-3.5 py-2 rounded-xl border border-slate-800 shrink-0">
          <PackageCheck className="w-4 h-4 text-emerald-400" />
          <div>
            <div className="text-[9px] uppercase font-mono text-slate-400">宿主包管理器</div>
            <div className="text-xs font-bold text-white font-mono uppercase">{packageManager} (Ready)</div>
          </div>
        </div>
      </div>

      {/* Empty State */}
      {filteredTools.length === 0 ? (
        <div className="p-10 rounded-2xl bg-slate-900/40 border border-dashed border-slate-800 text-center space-y-2">
          <AlertCircle className="w-8 h-8 text-slate-500 mx-auto" />
          <h3 className="text-sm font-semibold text-slate-300">未找到匹配的系统工具</h3>
          <p className="text-xs text-slate-500">可尝试搜索其他关键词如 docker, redis, mysql, nginx, ollama 等</p>
        </div>
      ) : (
        /* System Tools Grid */
        <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3.5">
          {filteredTools.map((tool) => (
            <div
              key={tool.id}
              className="p-4 rounded-2xl bg-slate-900/70 border border-slate-800 hover:border-slate-700 transition-all flex flex-col justify-between shadow-sm group"
            >
              <div>
                <div className="flex items-start justify-between gap-2">
                  <div className="flex items-center gap-2.5 min-w-0">
                    <div className="w-9 h-9 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center p-1.5 shrink-0">
                      <img
                        src={tool.icon}
                        alt={tool.name}
                        className="w-full h-full object-contain"
                        onError={(e) => {
                          (e.target as HTMLElement).style.display = 'none';
                        }}
                      />
                    </div>
                    <div className="min-w-0">
                      <h3 className="text-xs sm:text-sm font-bold text-white flex items-center gap-1.5 truncate">
                        <span className="truncate">{tool.name}</span>
                        <span className="text-[9px] font-mono px-1.5 py-0.2 rounded bg-slate-800 text-slate-400 font-normal shrink-0">
                          {tool.category}
                        </span>
                      </h3>
                      <button
                        onClick={() => api.openUrl(tool.homepage)}
                        className="text-[10px] text-slate-400 hover:text-blue-400 flex items-center gap-1 transition-colors mt-0.5"
                        title="打开官方网站"
                      >
                        官网 <ExternalLink className="w-2.5 h-2.5" />
                      </button>
                    </div>
                  </div>

                  {tool.isInstalled ? (
                    <span className="px-2 py-0.2 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] font-semibold flex items-center gap-1 shrink-0">
                      <Check className="w-2.5 h-2.5" /> 已安装
                    </span>
                  ) : (
                    <span className="px-2 py-0.2 rounded-full bg-slate-800 text-slate-400 border border-slate-700 text-[10px] font-medium shrink-0">
                      未安装
                    </span>
                  )}
                </div>

                <p className="text-xs text-slate-400 mt-2.5 line-clamp-2">
                  {tool.description}
                </p>

                {/* Install Command Preview */}
                <div className="mt-2.5 p-2 rounded-lg bg-slate-950 border border-slate-800/80 font-mono text-[10px] text-slate-300 flex items-center justify-between">
                  <div className="flex items-center gap-1.5 truncate text-slate-400">
                    <Terminal className="w-3 h-3 text-slate-400 shrink-0" />
                    <span className="truncate">{tool.installCommand}</span>
                  </div>
                </div>
              </div>

              {/* Bottom Status / Install Action */}
              <div className="mt-3.5 pt-2.5 border-t border-slate-800/80 flex items-center justify-between">
                {tool.isInstalled ? (
                  <div className="text-xs font-mono text-slate-300 flex items-center gap-1.5 truncate">
                    <ShieldCheck className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                    <span className="truncate">v{tool.installedVersion}</span>
                  </div>
                ) : (
                  <div className="text-[11px] text-slate-400">
                    未就绪
                  </div>
                )}

                {tool.isInstalled ? (
                  <button
                    onClick={() => handleTestTool(tool.id, tool.name)}
                    disabled={testingId === tool.id}
                    className="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition-colors flex items-center gap-1 shrink-0"
                  >
                    {testingId === tool.id ? (
                      <>
                        <Loader2 className="w-3 h-3 animate-spin text-blue-400" />
                        <span>测试中...</span>
                      </>
                    ) : (
                      '运行测试'
                    )}
                  </button>
                ) : (
                  <button
                    onClick={() => onInstallSystemTool(tool.id)}
                    className="px-3 py-1 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-md shadow-blue-500/20 transition-colors flex items-center gap-1 shrink-0"
                  >
                    <Download className="w-3 h-3" /> 一键安装
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
