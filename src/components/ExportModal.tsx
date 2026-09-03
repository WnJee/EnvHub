import React, { useState, useMemo } from 'react';
import { 
  X, 
  Share2, 
  Download, 
  Copy, 
  Check, 
  Terminal, 
  Apple, 
  Monitor, 
  FileCode, 
  Layers, 
  Zap, 
  ShieldCheck, 
  FolderOpen
} from 'lucide-react';
import { RuntimeTool } from '../types';
import { 
  generateUnixShellScript, 
  generateWindowsPowerShellScript, 
  generateWindowsBatchScript, 
  generateMiseTomlConfig, 
  ExportToolItem 
} from '../utils/scriptGenerator';
import { api } from '../services/tauri';
import { useToast } from './Toast';

interface ExportModalProps {
  isOpen: boolean;
  onClose: () => void;
  runtimes: RuntimeTool[];
}

type PlatformTab = 'sh' | 'ps1' | 'bat' | 'toml';

export const ExportModal: React.FC<ExportModalProps> = ({
  isOpen,
  onClose,
  runtimes
}) => {
  const toast = useToast();
  const [activeTab, setActiveTab] = useState<PlatformTab>('sh');
  const [includeMirrors, setIncludeMirrors] = useState<boolean>(true);
  const [includeEnvHook, setIncludeEnvHook] = useState<boolean>(true);
  const [copied, setCopied] = useState<boolean>(false);
  const [savedPath, setSavedPath] = useState<string | null>(null);

  // Initialize tool items with active / installed versions
  const [toolItems, setToolItems] = useState<ExportToolItem[]>(() => {
    return runtimes
      .filter((r) => r.activeVersion || (r.installedVersions && r.installedVersions.length > 0))
      .map((r) => ({
        id: r.id,
        name: r.name,
        version: r.activeVersion || r.installedVersions[0] || 'latest',
        checked: true
      }));
  });

  // Keep toolItems synchronized when modal opens or runtimes change
  React.useEffect(() => {
    if (isOpen) {
      setToolItems(
        runtimes
          .filter((r) => r.activeVersion || (r.installedVersions && r.installedVersions.length > 0))
          .map((r) => ({
            id: r.id,
            name: r.name,
            version: r.activeVersion || r.installedVersions[0] || 'latest',
            checked: true
          }))
      );
      setSavedPath(null);
    }
  }, [isOpen, runtimes]);

  const toggleTool = (toolId: string) => {
    setToolItems((prev) =>
      prev.map((item) =>
        item.id === toolId ? { ...item, checked: !item.checked } : item
      )
    );
  };

  const handleSelectAll = (select: boolean) => {
    setToolItems((prev) => prev.map((item) => ({ ...item, checked: select })));
  };

  // Generate current script content based on activeTab
  const scriptContent = useMemo(() => {
    const opts = { includeMirrors, includeEnvHook };
    switch (activeTab) {
      case 'sh':
        return generateUnixShellScript(toolItems, opts);
      case 'ps1':
        return generateWindowsPowerShellScript(toolItems, opts);
      case 'bat':
        return generateWindowsBatchScript();
      case 'toml':
        return generateMiseTomlConfig(toolItems);
      default:
        return '';
    }
  }, [toolItems, activeTab, includeMirrors, includeEnvHook]);

  const currentFilename = useMemo(() => {
    switch (activeTab) {
      case 'sh':
        return 'setup_envhub.sh';
      case 'ps1':
        return 'setup_envhub.ps1';
      case 'bat':
        return 'setup_envhub.bat';
      case 'toml':
        return '.mise.toml';
    }
  }, [activeTab]);

  const runCommand = useMemo(() => {
    switch (activeTab) {
      case 'sh':
        return 'bash setup_envhub.sh';
      case 'ps1':
        return 'powershell -ExecutionPolicy Bypass -File setup_envhub.ps1';
      case 'bat':
        return '.\\setup_envhub.bat';
      case 'toml':
        return 'mise install';
    }
  }, [activeTab]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(scriptContent);
      setCopied(true);
      toast.success(`已将 ${currentFilename} 内容复制到剪贴板`, '复制成功');
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error('无法访问系统剪贴板', '复制失败');
    }
  };

  const handleCopyCommand = async () => {
    try {
      await navigator.clipboard.writeText(runCommand);
      toast.success(runCommand, '命令已复制');
    } catch {
      toast.error('无法访问系统剪贴板', '复制失败');
    }
  };

  const handleSaveFile = async () => {
    try {
      const path = await api.saveExportFile(currentFilename, scriptContent);
      setSavedPath(path);
      toast.success(`已保存到: ${path}`, '脚本保存成功');
    } catch (err: any) {
      toast.error(typeof err === 'string' ? err : err.message || '未知错误', '保存失败');
    }
  };

  const handleRevealInFinder = async () => {
    if (savedPath) {
      await api.openPathInFileManager(savedPath);
    }
  };

  if (!isOpen) return null;

  const selectedCount = toolItems.filter((t) => t.checked).length;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md animate-in fade-in duration-200">
      <div className="w-full max-w-4xl bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] animate-in zoom-in-95 duration-150">
        
        {/* Header */}
        <div className="p-4 sm:p-5 border-b border-slate-800 bg-gradient-to-r from-blue-950/40 via-slate-900 to-indigo-950/30 flex items-center justify-between shrink-0">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-blue-600 via-indigo-600 to-cyan-400 flex items-center justify-center shadow-lg shadow-blue-500/20 text-white shrink-0">
              <Share2 className="w-5 h-5" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-base font-bold text-white">一键导出本机开发环境部署脚本</h2>
                <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400 border border-blue-500/30 font-semibold">
                  跨平台环境同步
                </span>
              </div>
              <p className="text-xs text-slate-400 mt-0.5">
                将当前机器的语言运行时版本、国内镜像源与环境变量一键导出为自动化安装脚本
              </p>
            </div>
          </div>

          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content Body */}
        <div className="flex-1 flex flex-col md:flex-row overflow-hidden bg-[#090D16]">
          
          {/* Left Column: Tools & Options Selection */}
          <div className="w-full md:w-72 border-b md:border-b-0 md:border-r border-slate-800 p-4 space-y-4 overflow-y-auto bg-slate-950/60 shrink-0">
            {/* Tools Selector */}
            <div className="space-y-2.5">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-white flex items-center gap-1.5">
                  <Layers className="w-3.5 h-3.5 text-blue-400" />
                  已捕获语言环境 ({selectedCount}/{toolItems.length})
                </span>
                <div className="flex items-center gap-1.5 text-[10px]">
                  <button
                    onClick={() => handleSelectAll(true)}
                    className="text-blue-400 hover:text-blue-300 font-medium"
                  >
                    全选
                  </button>
                  <span className="text-slate-600">|</span>
                  <button
                    onClick={() => handleSelectAll(false)}
                    className="text-slate-400 hover:text-slate-300"
                  >
                    清空
                  </button>
                </div>
              </div>

              {/* Tool list checkboxes */}
              <div className="space-y-1.5 max-h-48 md:max-h-64 overflow-y-auto pr-1">
                {toolItems.length === 0 ? (
                  <div className="text-xs text-slate-400 text-center py-4 bg-slate-900/60 rounded-xl border border-slate-800">
                    暂未检测到已安装的语言环境
                  </div>
                ) : (
                  toolItems.map((tool) => (
                    <label
                      key={tool.id}
                      className={`flex items-center justify-between p-2 rounded-xl border cursor-pointer transition-all ${
                        tool.checked
                          ? 'bg-blue-950/30 border-blue-500/40 text-white shadow-sm'
                          : 'bg-slate-900/40 border-slate-800/80 text-slate-400 hover:bg-slate-900'
                      }`}
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <input
                          type="checkbox"
                          checked={tool.checked}
                          onChange={() => toggleTool(tool.id)}
                          className="w-3.5 h-3.5 rounded text-blue-600 bg-slate-900 border-slate-700"
                        />
                        <span className="text-xs font-semibold truncate">{tool.name}</span>
                      </div>
                      <span className="text-[10px] font-mono font-medium px-1.5 py-0.5 rounded bg-slate-900 border border-slate-800 text-cyan-400 shrink-0">
                        {tool.version}
                      </span>
                    </label>
                  ))
                )}
              </div>
            </div>

            {/* Config Options */}
            <div className="space-y-2 pt-2 border-t border-slate-800/80">
              <span className="text-xs font-bold text-white flex items-center gap-1.5">
                <Zap className="w-3.5 h-3.5 text-amber-400" />
                附加安装配置
              </span>

              <label className="flex items-start gap-2 p-2 rounded-xl bg-slate-900/60 border border-slate-800/80 cursor-pointer hover:bg-slate-900 transition-colors">
                <input
                  type="checkbox"
                  checked={includeMirrors}
                  onChange={(e) => setIncludeMirrors(e.target.checked)}
                  className="w-3.5 h-3.5 mt-0.5 rounded text-blue-600 bg-slate-900 border-slate-700 shrink-0"
                />
                <div>
                  <div className="text-xs font-medium text-slate-200">包含国内高速镜像源</div>
                  <div className="text-[10px] text-slate-400">
                    自动配置 NPM、Go、Pip 与 Cargo 的国内加速源
                  </div>
                </div>
              </label>

              <label className="flex items-start gap-2 p-2 rounded-xl bg-slate-900/60 border border-slate-800/80 cursor-pointer hover:bg-slate-900 transition-colors">
                <input
                  type="checkbox"
                  checked={includeEnvHook}
                  onChange={(e) => setIncludeEnvHook(e.target.checked)}
                  className="w-3.5 h-3.5 mt-0.5 rounded text-blue-600 bg-slate-900 border-slate-700 shrink-0"
                />
                <div>
                  <div className="text-xs font-medium text-slate-200">自动注入终端环境变量</div>
                  <div className="text-[10px] text-slate-400">
                    自动配置 zshrc / bashrc / PowerShell Profile
                  </div>
                </div>
              </label>
            </div>
          </div>

          {/* Right Column: Platform Tabs & Script Preview */}
          <div className="flex-1 flex flex-col overflow-hidden p-4 space-y-3">
            {/* Platform Selector Tabs */}
            <div className="flex items-center justify-between gap-2 flex-wrap shrink-0">
              <div className="flex items-center gap-1.5 bg-slate-950 p-1 rounded-xl border border-slate-800">
                <button
                  onClick={() => setActiveTab('sh')}
                  className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-all ${
                    activeTab === 'sh'
                      ? 'bg-blue-600 text-white shadow-md shadow-blue-500/20'
                      : 'text-slate-400 hover:text-white hover:bg-slate-800/50'
                  }`}
                >
                  <Apple className="w-3.5 h-3.5" />
                  <span>macOS / Linux (.sh)</span>
                </button>

                <button
                  onClick={() => setActiveTab('ps1')}
                  className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-all ${
                    activeTab === 'ps1'
                      ? 'bg-blue-600 text-white shadow-md shadow-blue-500/20'
                      : 'text-slate-400 hover:text-white hover:bg-slate-800/50'
                  }`}
                >
                  <Monitor className="w-3.5 h-3.5" />
                  <span>Windows (.ps1)</span>
                </button>

                <button
                  onClick={() => setActiveTab('bat')}
                  className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-all ${
                    activeTab === 'bat'
                      ? 'bg-blue-600 text-white shadow-md shadow-blue-500/20'
                      : 'text-slate-400 hover:text-white hover:bg-slate-800/50'
                  }`}
                >
                  <Terminal className="w-3.5 h-3.5" />
                  <span>Win 批处理 (.bat)</span>
                </button>

                <button
                  onClick={() => setActiveTab('toml')}
                  className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-all ${
                    activeTab === 'toml'
                      ? 'bg-blue-600 text-white shadow-md shadow-blue-500/20'
                      : 'text-slate-400 hover:text-white hover:bg-slate-800/50'
                  }`}
                >
                  <FileCode className="w-3.5 h-3.5" />
                  <span>.mise.toml</span>
                </button>
              </div>

              {/* Quick Action Buttons */}
              <div className="flex items-center gap-2">
                <button
                  onClick={handleCopy}
                  className="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium flex items-center gap-1.5 transition-all border border-slate-700"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5 text-blue-400" />}
                  <span>{copied ? '已复制' : '复制代码'}</span>
                </button>

                <button
                  onClick={handleSaveFile}
                  className="px-3.5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold flex items-center gap-1.5 transition-all shadow-md shadow-blue-500/20"
                >
                  <Download className="w-3.5 h-3.5" />
                  <span>保存为 {currentFilename}</span>
                </button>
              </div>
            </div>

            {/* Quick Run Command Bar */}
            <div className="p-2.5 rounded-xl bg-slate-950 border border-slate-800/80 flex items-center justify-between gap-2 shrink-0">
              <div className="flex items-center gap-2 min-w-0">
                <span className="text-[10px] uppercase font-mono font-bold text-slate-400 bg-slate-900 px-1.5 py-0.5 rounded border border-slate-800 shrink-0">
                  一键执行
                </span>
                <code className="text-xs font-mono text-cyan-400 truncate select-all">
                  {runCommand}
                </code>
              </div>
              <button
                onClick={handleCopyCommand}
                title="复制运行指令"
                className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors shrink-0"
              >
                <Copy className="w-3.5 h-3.5" />
              </button>
            </div>

            {/* Script Code Preview */}
            <div className="flex-1 min-h-[160px] rounded-xl bg-slate-950 border border-slate-800 overflow-hidden flex flex-col">
              <div className="p-2 bg-slate-900/80 border-b border-slate-800/80 flex items-center justify-between text-[11px] text-slate-400 font-mono px-3">
                <span>{currentFilename} ({scriptContent.split('\n').length} 行)</span>
                <span className="text-[10px] text-slate-400">UTF-8 • 准备就绪</span>
              </div>
              <pre className="flex-1 p-3.5 font-mono text-[11px] sm:text-xs text-slate-300 overflow-auto whitespace-pre leading-relaxed select-text bg-[#070B14]">
                <code>{scriptContent}</code>
              </pre>
            </div>

            {/* Saved Location Notification */}
            {savedPath && (
              <div className="p-2.5 rounded-xl bg-emerald-950/40 border border-emerald-500/30 flex items-center justify-between gap-2 animate-in fade-in shrink-0">
                <div className="flex items-center gap-2 min-w-0 text-emerald-400 text-xs font-medium">
                  <Check className="w-4 h-4 shrink-0" />
                  <span className="truncate">文件已成功保存到: {savedPath}</span>
                </div>
                <button
                  onClick={handleRevealInFinder}
                  className="px-2.5 py-1 rounded-lg bg-emerald-600/20 hover:bg-emerald-600/30 text-emerald-300 text-xs font-medium flex items-center gap-1 transition-colors border border-emerald-500/30 shrink-0"
                >
                  <FolderOpen className="w-3 h-3" />
                  <span>在访达/文件夹中显示</span>
                </button>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="p-3.5 bg-[#0B1120] border-t border-slate-800 flex items-center justify-between shrink-0">
          <div className="text-[11px] text-slate-400 flex items-center gap-1.5">
            <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
            <span>生成的脚本不含任何外部隐私数据，可安全分发至团队成员或服务器一键初始化。</span>
          </div>

          <button
            onClick={onClose}
            className="px-4 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold transition-colors"
          >
            关闭
          </button>
        </div>
      </div>
    </div>
  );
};
