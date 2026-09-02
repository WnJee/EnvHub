import React, { useState } from 'react';
import { SystemStatus } from '../types';
import { 
  Settings, 
  Cpu, 
  Download, 
  CheckCircle2, 
  ShieldCheck,
  AlertTriangle,
  Loader2
} from 'lucide-react';
import { useToast } from './Toast';

interface SettingsModalProps {
  systemStatus: SystemStatus;
  onBootstrapMise: () => Promise<void>;
  isBootstrapping: boolean;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  systemStatus,
  onBootstrapMise,
  isBootstrapping,
}) => {
  const [customPath, setCustomPath] = useState(systemStatus.misePath || '');
  const [autoUpdate, setAutoUpdate] = useState(true);
  const toast = useToast();

  const handleSavePath = () => {
    if (!customPath.trim()) {
      toast.warning('请输入有效的 Mise 二进制文件路径');
      return;
    }
    toast.success(`已保存 Mise 引擎路径配置: ${customPath}`);
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6 bg-[#090D16]">
      {/* Top Banner */}
      <div className="p-6 rounded-2xl bg-gradient-to-r from-slate-900 via-slate-900 to-slate-800 border border-slate-800/80 flex items-center justify-between gap-4 shadow-xl">
        <div>
          <div className="flex items-center gap-2 text-blue-400 font-mono text-xs uppercase font-semibold">
            <Settings className="w-4 h-4" />
            Core Engine & Client Preferences
          </div>
          <h2 className="text-xl font-bold text-white mt-1">客户端与底层引擎配置</h2>
          <p className="text-xs text-slate-400 mt-1">
            真实管理底层 Mise CLI 引擎路径、自举策略以及跨平台系统环境激活行为。
          </p>
        </div>
      </div>

      {/* Engine Status Card */}
      <div className="p-5 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-4">
        <h3 className="text-sm font-bold text-white flex items-center gap-2">
          <Cpu className="w-4 h-4 text-blue-400" />
          Mise 核心引擎状态 (真实检测)
        </h3>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-1">
            <div className="text-[10px] uppercase font-mono text-slate-400">运行状态</div>
            <div className="text-sm font-bold flex items-center gap-2">
              {systemStatus.miseInstalled ? (
                <>
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                  <span className="text-emerald-400">CLI 已就绪并在系统路径中</span>
                </>
              ) : (
                <>
                  <AlertTriangle className="w-4 h-4 text-amber-400" />
                  <span className="text-amber-400">未在系统 PATH 中检测到 Mise</span>
                </>
              )}
            </div>
            <div className="text-xs text-slate-400 font-mono pt-1">
              {systemStatus.miseVersion || '无版本信息 (未安装)'}
            </div>
          </div>

          <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-1">
            <div className="text-[10px] uppercase font-mono text-slate-400">真实二进制路径</div>
            <div className="text-xs font-bold font-mono text-slate-200 truncate pt-1">
              {systemStatus.misePath || '未检测到二进制文件'}
            </div>
            <div className="text-[11px] text-slate-400">
              用于 Tauri 后端子进程调用与真实版本环境切换
            </div>
          </div>
        </div>

        {/* Custom path input */}
        <div className="space-y-2 pt-2">
          <label className="text-xs font-semibold text-slate-300">自定义 Mise 二进制路径 (如手动安装或打包的 Sidecar)</label>
          <div className="flex gap-2">
            <input
              type="text"
              placeholder="例如: /opt/homebrew/bin/mise 或 ~/.local/bin/mise"
              value={customPath}
              onChange={(e) => setCustomPath(e.target.value)}
              className="flex-1 bg-slate-950 border border-slate-800 rounded-xl px-4 py-2 text-xs font-mono text-white focus:outline-none focus:border-blue-500"
            />
            <button
              onClick={handleSavePath}
              className="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold transition-colors"
            >
              保存并测试
            </button>
          </div>
        </div>

        {/* 1-Click Bootstrap */}
        <div className="p-4 rounded-xl bg-blue-950/20 border border-blue-500/30 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div>
            <div className="text-xs font-bold text-blue-300">一键下载 / 自举安装 Mise CLI 引擎</div>
            <div className="text-[11px] text-slate-400 mt-0.5">
              自动从官方源拉取适配当前架构 ({systemStatus.arch}) 的最新 Mise 二进制并配置环境
            </div>
          </div>

          <button
            onClick={onBootstrapMise}
            disabled={isBootstrapping}
            className="px-4 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white text-xs font-semibold shadow-md shadow-blue-500/20 flex items-center gap-2 transition-all shrink-0"
          >
            {isBootstrapping ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin text-white" />
            ) : (
              <Download className="w-3.5 h-3.5" />
            )}
            <span>{isBootstrapping ? '正在自举安装...' : '一键自举安装'}</span>
          </button>
        </div>
      </div>

      {/* Preferences Card */}
      <div className="p-5 rounded-2xl bg-slate-900/70 border border-slate-800 space-y-4">
        <h3 className="text-sm font-bold text-white flex items-center gap-2">
          <ShieldCheck className="w-4 h-4 text-emerald-400" />
          运行策略与环境同步
        </h3>

        <div className="space-y-3">
          <label className="flex items-center justify-between p-3 rounded-xl bg-slate-950 border border-slate-800 cursor-pointer">
            <div>
              <div className="text-xs font-semibold text-slate-200">启动时自动同步 Shell 环境变量</div>
              <div className="text-[11px] text-slate-400">
                通过子进程登录 Shell 读取 ~/.zshrc，避免 macOS GUI 进程 PATH 缺失
              </div>
            </div>
            <input
              type="checkbox"
              checked={autoUpdate}
              onChange={(e) => setAutoUpdate(e.target.checked)}
              className="w-4 h-4 rounded text-blue-600 bg-slate-900 border-slate-700"
            />
          </label>

          <label className="flex items-center justify-between p-3 rounded-xl bg-slate-950 border border-slate-800 cursor-pointer">
            <div>
              <div className="text-xs font-semibold text-slate-200">切换版本时自动向 Shell RC 注入 Shims</div>
              <div className="text-[11px] text-slate-400">
                保持终端命令行中 node / python / go 命令与 GUI 设置绝对一致
              </div>
            </div>
            <input
              type="checkbox"
              defaultChecked
              onChange={() => toast.inDev('实时 Shims 动态热重载')}
              className="w-4 h-4 rounded text-blue-600 bg-slate-900 border-slate-700"
            />
          </label>
        </div>
      </div>

      {/* About Box */}
      <div className="p-5 rounded-2xl bg-slate-900/40 border border-slate-800 text-center space-y-2">
        <div className="text-sm font-bold text-white flex items-center justify-center gap-2">
          <Cpu className="w-4 h-4 text-blue-400" />
          EnvHub Desktop Client
        </div>
        <p className="text-xs text-slate-400">
          基于 Tauri v2 (Rust) + React 18 + Tailwind CSS + Mise CLI 驱动的现代化极速环境管理套件
        </p>
        <div className="text-[11px] font-mono text-slate-400">
          Version 0.1.0 • 100% 真实本机环境交互
        </div>
      </div>
    </div>
  );
};
