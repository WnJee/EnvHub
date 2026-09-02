import React, { useState } from 'react';
import { 
  Sparkles, 
  Download, 
  X, 
  Calendar, 
  ExternalLink, 
  FolderOpen, 
  CheckCircle2, 
  AlertCircle,
  Loader2,
  RefreshCw
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
  const [downloadedPath, setDownloadedPath] = useState<string>('');
  const [errorMsg, setErrorMsg] = useState<string>('');

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
      const destPath = await api.downloadAndInstallUpdate(
        url,
        updateInfo.latestVersion,
        (p) => setProgress(p)
      );
      setDownloadedPath(destPath);
      setDownloadStatus('completed');
      setProgress(100);
    } catch (err: any) {
      console.error('Download update error:', err);
      setErrorMsg(typeof err === 'string' ? err : err.message || '下载安装包失败');
      setDownloadStatus('error');
    }
  };

  const handleOpenInstaller = async () => {
    if (downloadedPath) {
      await api.openInstallerFile(downloadedPath);
    }
  };

  const handleOpenInFolder = async () => {
    if (downloadedPath) {
      await api.openPathInFileManager(downloadedPath);
    }
  };

  const handleCloseModal = () => {
    if (downloadStatus !== 'downloading') {
      setDownloadStatus('idle');
      setProgress(0);
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md animate-in fade-in duration-200">
      <div className="w-full max-w-lg bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh] animate-in zoom-in-95 duration-150">
        {/* Header */}
        <div className="p-5 border-b border-slate-800 bg-gradient-to-r from-blue-950/40 via-slate-900 to-slate-900 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-blue-600 to-cyan-400 flex items-center justify-center shadow-lg shadow-blue-500/20 text-white shrink-0">
              <Sparkles className="w-5 h-5" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h3 className="text-base font-bold text-white">发现新版本 EnvHub</h3>
                <span className="text-xs font-mono font-bold px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400 border border-blue-500/30">
                  v{updateInfo.latestVersion}
                </span>
              </div>
              <div className="text-xs text-slate-400 flex items-center gap-2 mt-0.5 font-mono">
                <span>当前版本: v{updateInfo.currentVersion}</span>
                {updateInfo.publishedAt && (
                  <>
                    <span>•</span>
                    <span className="flex items-center gap-1">
                      <Calendar className="w-3 h-3" /> {updateInfo.publishedAt}
                    </span>
                  </>
                )}
              </div>
            </div>
          </div>

          <button
            onClick={handleCloseModal}
            className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body Content */}
        <div className="flex-1 overflow-y-auto p-5 space-y-4 bg-[#090D16]">
          {/* Download in Progress state */}
          {downloadStatus === 'downloading' && (
            <div className="p-5 rounded-2xl bg-blue-950/30 border border-blue-500/30 space-y-3 animate-in fade-in">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2.5">
                  <Loader2 className="w-4 h-4 text-blue-400 animate-spin" />
                  <span className="text-xs sm:text-sm font-semibold text-white">
                    正在应用内下载新版本安装包...
                  </span>
                </div>
                <span className="text-xs font-mono font-bold text-blue-400">{progress}%</span>
              </div>

              {/* Progress Bar */}
              <div className="w-full bg-slate-800 rounded-full h-2.5 overflow-hidden p-0.5">
                <div 
                  className="bg-gradient-to-r from-blue-500 to-cyan-400 h-full rounded-full transition-all duration-300 shadow-sm shadow-blue-500/50"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <p className="text-[11px] text-slate-400">下载完成后将自动为您打开安装包，请稍候...</p>
            </div>
          )}

          {/* Download Completed state */}
          {downloadStatus === 'completed' && (
            <div className="p-5 rounded-2xl bg-emerald-950/30 border border-emerald-500/30 space-y-3 animate-in fade-in">
              <div className="flex items-center gap-2.5 text-emerald-400">
                <CheckCircle2 className="w-5 h-5 shrink-0" />
                <span className="text-xs sm:text-sm font-semibold text-white">
                  安装包下载完成并已自动打开！
                </span>
              </div>
              <div className="text-[11px] text-slate-300 font-mono bg-slate-900/90 p-2.5 rounded-xl border border-slate-800 break-all select-all">
                {downloadedPath}
              </div>
              <div className="flex items-center gap-2 pt-1">
                <button
                  onClick={handleOpenInstaller}
                  className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold flex items-center gap-1.5 transition-all shadow-md shadow-emerald-600/20"
                >
                  <ExternalLink className="w-3.5 h-3.5" />
                  <span>重新打开安装包</span>
                </button>
                <button
                  onClick={handleOpenInFolder}
                  className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium flex items-center gap-1.5 transition-colors border border-slate-700"
                >
                  <FolderOpen className="w-3.5 h-3.5 text-blue-400" />
                  <span>在访达/文件夹中显示</span>
                </button>
              </div>
            </div>
          )}

          {/* Error state */}
          {downloadStatus === 'error' && (
            <div className="p-4 rounded-2xl bg-rose-950/30 border border-rose-500/30 space-y-2.5 animate-in fade-in">
              <div className="flex items-center gap-2 text-rose-400 text-xs font-semibold">
                <AlertCircle className="w-4 h-4 shrink-0" />
                <span>应用内下载异常</span>
              </div>
              <p className="text-[11px] text-rose-300/80">{errorMsg || '网络连接超时或目标资源受限'}</p>
              <div className="flex items-center gap-2 pt-1">
                <button
                  onClick={handleStartInAppDownload}
                  className="px-3 py-1.5 rounded-lg bg-rose-600/20 hover:bg-rose-600/30 text-rose-300 border border-rose-500/30 text-xs font-semibold flex items-center gap-1.5 transition-all"
                >
                  <RefreshCw className="w-3.5 h-3.5" /> 重试
                </button>
                <button
                  onClick={handleOpenInBrowser}
                  className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium flex items-center gap-1.5 transition-colors border border-slate-700"
                >
                  <ExternalLink className="w-3.5 h-3.5 text-blue-400" /> 在浏览器中下载
                </button>
              </div>
            </div>
          )}

          {/* Release Notes */}
          <div className="space-y-2">
            <div className="text-xs font-bold uppercase tracking-wider text-slate-400 font-mono">
              更新日志
            </div>
            <div className="p-4 rounded-xl bg-slate-950/80 border border-slate-800 text-xs text-slate-300 leading-relaxed font-sans whitespace-pre-wrap select-text max-h-56 overflow-y-auto">
              {updateInfo.releaseNotes}
            </div>
          </div>
        </div>

        {/* Footer Actions */}
        <div className="p-4 bg-[#0B1120] border-t border-slate-800 flex items-center justify-between">
          <button
            onClick={handleOpenRelease}
            className="text-xs text-slate-400 hover:text-blue-400 flex items-center gap-1.5 transition-colors py-1 px-1.5 rounded-lg hover:bg-slate-800/40"
            title="在系统浏览器中打开 GitHub Releases"
          >
            <span>查看 GitHub 发行页</span>
            <ExternalLink className="w-3 h-3" />
          </button>

          <div className="flex items-center gap-2">
            {downloadStatus === 'completed' ? (
              <button
                onClick={handleCloseModal}
                className="px-4 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold transition-colors"
              >
                完成
              </button>
            ) : (
              <>
                <button
                  onClick={handleCloseModal}
                  disabled={downloadStatus === 'downloading'}
                  className="px-3.5 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition-colors disabled:opacity-50"
                >
                  稍后提醒
                </button>
                <button
                  onClick={handleStartInAppDownload}
                  disabled={downloadStatus === 'downloading'}
                  className="px-4 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-500 disabled:bg-blue-800 text-white text-xs font-semibold shadow-md shadow-blue-500/20 transition-all flex items-center gap-1.5"
                >
                  {downloadStatus === 'downloading' ? (
                    <>
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                      <span>正在下载 {progress}%</span>
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
