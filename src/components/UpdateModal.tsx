import React from 'react';
import { Sparkles, Download, X, Calendar, ExternalLink } from 'lucide-react';
import { UpdateInfo } from '../services/updater';

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
  if (!isOpen || !updateInfo || !updateInfo.hasUpdate) return null;

  const handleOpenRelease = () => {
    if (updateInfo.releaseUrl) {
      window.open(updateInfo.releaseUrl, '_blank');
    }
    onClose();
  };

  const handleDownload = () => {
    if (updateInfo.downloadUrl) {
      window.open(updateInfo.downloadUrl, '_blank');
    } else if (updateInfo.releaseUrl) {
      window.open(updateInfo.releaseUrl, '_blank');
    }
    onClose();
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
            onClick={onClose}
            className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Release Notes */}
        <div className="flex-1 overflow-y-auto p-5 space-y-3 bg-[#090D16]">
          <div className="text-xs font-bold uppercase tracking-wider text-slate-400 font-mono">
            更新日志
          </div>
          <div className="p-4 rounded-xl bg-slate-950/80 border border-slate-800 text-xs text-slate-300 leading-relaxed font-sans whitespace-pre-wrap select-text">
            {updateInfo.releaseNotes}
          </div>
        </div>

        {/* Footer Actions */}
        <div className="p-4 bg-[#0B1120] border-t border-slate-800 flex items-center justify-between">
          <button
            onClick={handleOpenRelease}
            className="text-xs text-slate-400 hover:text-blue-400 flex items-center gap-1 transition-colors"
          >
            <span>查看 GitHub 发行页</span>
            <ExternalLink className="w-3 h-3" />
          </button>

          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="px-3.5 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition-colors"
            >
              稍后提醒
            </button>
            <button
              onClick={handleDownload}
              className="px-4 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-md shadow-blue-500/20 transition-all flex items-center gap-1.5"
            >
              <Download className="w-3.5 h-3.5" />
              <span>立即下载更新</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
