import React, { useState } from 'react';
import { 
  Sparkles, 
  Download, 
  X, 
  Calendar, 
  ExternalLink, 
  CheckCircle2, 
  AlertCircle,
  Loader2,
  RefreshCw,
  RotateCcw,
  ShieldCheck,
  Zap,
  ArrowRight,
  Apple,
  Monitor,
  Terminal
} from 'lucide-react';
import { UpdateInfo } from '../services/updater';
import { api } from '../services/tauri';

interface UpdateModalProps {
  isOpen: boolean;
  updateInfo: UpdateInfo | null;
  onClose: () => void;
}

export const UpdateModal: React.FC<UpdateModalProps> = ({
  isOpen,
  updateInfo,
  onClose,
}) => {
  const [downloadStatus, setDownloadStatus] = useState<'idle' | 'downloading' | 'completed' | 'error'>('idle');
  const [progress, setProgress] = useState<number>(0);
  const [errorMsg, setErrorMsg] = useState<string>('');
  const [isRelaunching, setIsRelaunching] = useState<boolean>(false);

  if (!isOpen || !updateInfo || !updateInfo.hasUpdate) return null;

  const handleOpenRelease = async () => {
    if (updateInfo.releaseUrl) {
      await api.openUrl(updateInfo.releaseUrl);
    }
  };

  const handleOpenInBrowser = async () => {
    if (updateInfo.downloadUrl) {
      await api.openUrl(updateInfo.downloadUrl);
    } else if (updateInfo.releaseUrl) {
      await api.openUrl(updateInfo.releaseUrl);
    }
  };

  const handleStartInAppDownload = async () => {
    const url = updateInfo.downloadUrl || updateInfo.releaseUrl;
    if (!url) return;

    setDownloadStatus('downloading');
    setProgress(5);
    setErrorMsg('');

    try {
      await api.downloadAndInstallUpdate(
        url,
        updateInfo.latestVersion,
        (p) => setProgress(p)
      );
      setDownloadStatus('completed');
      setProgress(100);
    } catch (err: any) {
      console.error('Download update error:', err);
      setErrorMsg(typeof err === 'string' ? err : err.message || '下载安装包失败');
      setDownloadStatus('error');
    }
  };

  const handleRelaunch = async () => {
    setIsRelaunching(true);
    try {
      await api.relaunchApp();
    } catch (err) {
      console.error('Relaunch error:', err);
      setIsRelaunching(false);
    }
  };

  const handleCloseModal = () => {
    if (downloadStatus !== 'downloading') {
      setDownloadStatus('idle');
      setProgress(0);
      onClose();
    }
  };

  // Detect platform chip
  const userAgent = typeof navigator !== 'undefined' ? navigator.userAgent.toLowerCase() : '';
  const isWindows = userAgent.includes('win');
  const isLinux = userAgent.includes('linux');

  const getStepText = (p: number) => {
    if (p < 30) return '正在连接高速通道建立安全下载...';
    if (p < 90) return '正在应用内接收最新安装程序包...';
    if (p < 100) return '正在进行就地覆盖与系统签名安全授权...';
    return '新版本已就绪，准备重启生效！';
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-in fade-in duration-200">
      <div className="w-full max-w-xl bg-[#0C1222] border border-slate-700/80 rounded-3xl shadow-2xl overflow-hidden flex flex-col max-h-[88vh] animate-in zoom-in-95 duration-150">
        
        {/* Header with gradient badge */}
        <div className="p-5 sm:p-6 border-b border-slate-800/90 bg-gradient-to-r from-blue-950/50 via-indigo-950/30 to-slate-900 flex items-center justify-between shrink-0 relative overflow-hidden">
          <div className="absolute -right-8 -top-8 w-32 h-32 bg-blue-500/10 rounded-full blur-2xl pointer-events-none" />
          
          <div className="flex items-center gap-3.5 relative z-10">
            <div className="w-11 h-11 rounded-2xl bg-gradient-to-tr from-blue-600 via-indigo-600 to-cyan-400 flex items-center justify-center shadow-lg shadow-blue-500/25 text-white shrink-0">
              <Sparkles className="w-6 h-6 animate-pulse" />
            </div>
            <div>
              <div className="flex items-center gap-2 flex-wrap">
                <h3 className="text-base sm:text-lg font-bold text-white tracking-tight">
                  发现新版本可用
                </h3>
                
                {/* Version Jump Pill */}
                <div className="flex items-center gap-1 px-2 py-0.5 rounded-full bg-blue-500/15 border border-blue-500/30 text-blue-400 text-xs font-mono font-bold">
                  <span>v{updateInfo.currentVersion}</span>
                  <ArrowRight className="w-3 h-3 text-slate-500" />
                  <span className="text-cyan-300 font-extrabold">v{updateInfo.latestVersion}</span>
                </div>
              </div>

              <div className="text-xs text-slate-400 flex items-center gap-2 mt-1 font-sans">
                {isWindows ? (
                  <span className="flex items-center gap-1 text-slate-300">
                    <Monitor className="w-3 h-3 text-cyan-400" /> Windows x64
                  </span>
                ) : isLinux ? (
                  <span className="flex items-center gap-1 text-slate-300">
                    <Terminal className="w-3 h-3 text-amber-400" /> Linux AppImage
                  </span>
                ) : (
                  <span className="flex items-center gap-1 text-slate-300">
                    <Apple className="w-3 h-3 text-blue-400" /> macOS Universal
                  </span>
                )}
                {updateInfo.publishedAt && (
                  <>
                    <span className="text-slate-600">•</span>
                    <span className="flex items-center gap-1 text-slate-400">
                      <Calendar className="w-3 h-3" /> {updateInfo.publishedAt}
                    </span>
                  </>
                )}
              </div>
            </div>
          </div>

          <button
            onClick={handleCloseModal}
            disabled={downloadStatus === 'downloading'}
            className="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800/80 transition-colors disabled:opacity-30 relative z-10"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Feature Highlight Pill */}
        <div className="px-6 py-2.5 bg-blue-950/40 border-b border-blue-500/20 flex items-center gap-2 text-xs text-blue-300 shrink-0">
          <Zap className="w-3.5 h-3.5 text-amber-400 shrink-0" />
          <span className="truncate">
            支持应用内全自动就地升级与一键重启，全程零手动拖拽安装包。
          </span>
        </div>

        {/* Body Content */}
        <div className="flex-1 overflow-y-auto p-5 sm:p-6 space-y-4.5 bg-[#090D16]">
          
          {/* Download in Progress State */}
          {downloadStatus === 'downloading' && (
            <div className="p-5 rounded-2xl bg-gradient-to-b from-blue-950/40 to-slate-900 border border-blue-500/30 space-y-3.5 animate-in fade-in shadow-lg">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2.5">
                  <Loader2 className="w-4 h-4 text-cyan-400 animate-spin" />
                  <span className="text-xs sm:text-sm font-bold text-white">
                    {getStepText(progress)}
                  </span>
                </div>
                <span className="text-sm font-mono font-extrabold text-cyan-400">{progress}%</span>
              </div>

              {/* Glowing Progress Bar */}
              <div className="w-full bg-slate-950 rounded-full h-3 overflow-hidden p-0.5 border border-slate-800">
                <div 
                  className="bg-gradient-to-r from-blue-600 via-cyan-400 to-indigo-400 h-full rounded-full transition-all duration-300 shadow-md shadow-cyan-500/30"
                  style={{ width: `${progress}%` }}
                />
              </div>

              <div className="flex items-center justify-between text-[11px] text-slate-400 font-mono">
                <span>准备就地升级覆盖</span>
                <span>请勿关闭应用程序</span>
              </div>
            </div>
          )}

          {/* Download & Installation Completed State */}
          {downloadStatus === 'completed' && (
            <div className="p-5 rounded-2xl bg-gradient-to-b from-emerald-950/40 via-slate-900 to-slate-900 border border-emerald-500/40 space-y-4 animate-in fade-in shadow-xl">
              <div className="flex items-start gap-3.5">
                <div className="w-10 h-10 rounded-xl bg-emerald-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400 shrink-0 mt-0.5">
                  <CheckCircle2 className="w-6 h-6" />
                </div>
                <div>
                  <h4 className="text-sm sm:text-base font-bold text-white">
                    🎉 新版本已在应用内就绪！
                  </h4>
                  <p className="text-xs text-slate-300 mt-1 leading-relaxed">
                    新版本已在后台就地完成安全替换与授权，点击下方按钮立即重启即可生效并体验全部新特性！
                  </p>
                </div>
              </div>

              <div className="pt-1">
                <button
                  onClick={handleRelaunch}
                  disabled={isRelaunching}
                  className="w-full py-3 px-5 rounded-xl bg-gradient-to-r from-emerald-600 via-teal-600 to-cyan-600 hover:from-emerald-500 hover:to-cyan-500 text-white text-sm font-bold flex items-center justify-center gap-2.5 transition-all shadow-lg shadow-emerald-500/25 active:scale-[0.99]"
                >
                  <RotateCcw className={`w-4 h-4 ${isRelaunching ? 'animate-spin' : ''}`} />
                  <span>{isRelaunching ? '正在重启应用...' : '立即重启应用生效'}</span>
                </button>
              </div>
            </div>
          )}

          {/* Error State */}
          {downloadStatus === 'error' && (
            <div className="p-4 sm:p-5 rounded-2xl bg-rose-950/30 border border-rose-500/30 space-y-3 animate-in fade-in">
              <div className="flex items-center gap-2.5 text-rose-400 text-xs font-bold">
                <AlertCircle className="w-4 h-4 shrink-0" />
                <span>应用内下载更新异常</span>
              </div>
              <p className="text-xs text-rose-300/80 leading-relaxed">
                {errorMsg || '网络连接超时或目标资源受限，请重试或在浏览器中直接下载。'}
              </p>
              <div className="flex items-center gap-2 pt-1">
                <button
                  onClick={handleStartInAppDownload}
                  className="px-3.5 py-2 rounded-xl bg-rose-600/20 hover:bg-rose-600/30 text-rose-300 border border-rose-500/30 text-xs font-bold flex items-center gap-1.5 transition-all"
                >
                  <RefreshCw className="w-3.5 h-3.5" /> 重试应用内更新
                </button>
                <button
                  onClick={handleOpenInBrowser}
                  className="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium flex items-center gap-1.5 transition-colors border border-slate-700"
                >
                  <ExternalLink className="w-3.5 h-3.5 text-blue-400" /> 浏览器直连下载
                </button>
              </div>
            </div>
          )}

          {/* Release Notes Section */}
          <div className="space-y-2.5">
            <div className="flex items-center justify-between">
              <span className="text-xs font-bold uppercase tracking-wider text-slate-400 font-mono flex items-center gap-1.5">
                <ShieldCheck className="w-3.5 h-3.5 text-blue-400" />
                版本发布更新日志
              </span>
              <span className="text-[10px] font-mono text-slate-400">
                {updateInfo.releaseTitle || `v${updateInfo.latestVersion}`}
              </span>
            </div>

            <div className="p-4 sm:p-5 rounded-2xl bg-slate-950/80 border border-slate-800/90 text-xs text-slate-300 leading-relaxed font-sans whitespace-pre-wrap select-text max-h-60 overflow-y-auto space-y-2">
              {updateInfo.releaseNotes}
            </div>
          </div>
        </div>

        {/* Footer Actions */}
        <div className="p-4 sm:p-5 bg-[#080C16] border-t border-slate-800 flex items-center justify-between shrink-0">
          <button
            onClick={handleOpenRelease}
            className="text-xs text-slate-400 hover:text-blue-400 flex items-center gap-1.5 transition-colors py-1.5 px-2 rounded-lg hover:bg-slate-800/40"
            title="在系统默认浏览器中打开 GitHub Release 发行页面"
          >
            <span>GitHub 发行页</span>
            <ExternalLink className="w-3 h-3" />
          </button>

          <div className="flex items-center gap-2.5">
            {downloadStatus === 'completed' ? (
              <button
                onClick={handleCloseModal}
                className="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition-colors"
              >
                稍后手动重启
              </button>
            ) : (
              <>
                <button
                  onClick={handleCloseModal}
                  disabled={downloadStatus === 'downloading'}
                  className="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition-colors disabled:opacity-50"
                >
                  稍后提醒
                </button>
                <button
                  onClick={handleStartInAppDownload}
                  disabled={downloadStatus === 'downloading'}
                  className="px-5 py-2 rounded-xl bg-gradient-to-r from-blue-600 via-indigo-600 to-cyan-500 hover:from-blue-500 hover:to-cyan-400 disabled:opacity-50 text-white text-xs font-bold shadow-lg shadow-blue-500/25 transition-all flex items-center gap-2 active:scale-95"
                >
                  {downloadStatus === 'downloading' ? (
                    <>
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                      <span>正在更新 {progress}%</span>
                    </>
                  ) : (
                    <>
                      <Download className="w-3.5 h-3.5" />
                      <span>立即应用内更新</span>
                    </>
                  )}
                </button>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
