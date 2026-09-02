import React, { createContext, useContext, useState, ReactNode } from 'react';
import { CheckCircle2, AlertCircle, Info, X, Clock } from 'lucide-react';

export type ToastType = 'info' | 'success' | 'warning' | 'error' | 'dev';

export interface ToastMessage {
  id: string;
  type: ToastType;
  title?: string;
  message: string;
  duration?: number;
}

interface ToastContextType {
  toast: (options: { type?: ToastType; title?: string; message: string; duration?: number }) => void;
  info: (message: string, title?: string) => void;
  success: (message: string, title?: string) => void;
  warning: (message: string, title?: string) => void;
  error: (message: string, title?: string) => void;
  inDev: (featureName?: string) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export const ToastProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);

  const removeToast = (id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  const addToast = (options: { type?: ToastType; title?: string; message: string; duration?: number }) => {
    const id = 'toast_' + Date.now() + '_' + Math.random().toString(36).substring(2, 7);
    const newToast: ToastMessage = {
      id,
      type: options.type || 'info',
      title: options.title,
      message: options.message,
      duration: options.duration || 3000,
    };

    setToasts((prev) => [...prev, newToast]);

    setTimeout(() => {
      removeToast(id);
    }, newToast.duration);
  };

  const info = (message: string, title?: string) => addToast({ type: 'info', title, message });
  const success = (message: string, title?: string) => addToast({ type: 'success', title, message });
  const warning = (message: string, title?: string) => addToast({ type: 'warning', title, message });
  const error = (message: string, title?: string) => addToast({ type: 'error', title, message });
  const inDev = (featureName?: string) => {
    addToast({
      type: 'dev',
      title: '功能正在开发中',
      message: featureName ? `「${featureName}」功能正在开发与适配中，敬请期待！` : '该功能正在积极开发中，敬请期待！',
    });
  };

  return (
    <ToastContext.Provider value={{ toast: addToast, info, success, warning, error, inDev }}>
      {children}
      {/* Toast Container */}
      <div className="fixed bottom-6 right-6 z-50 flex flex-col gap-2 max-w-md w-full pointer-events-none select-none">
        {toasts.map((t) => {
          return (
            <div
              key={t.id}
              className={`pointer-events-auto p-4 rounded-xl border shadow-xl flex items-start justify-between gap-3 animate-in slide-in-from-bottom-3 fade-in duration-200 ${
                t.type === 'success'
                  ? 'bg-emerald-950/90 border-emerald-500/40 text-emerald-200'
                  : t.type === 'error'
                  ? 'bg-rose-950/90 border-rose-500/40 text-rose-200'
                  : t.type === 'warning'
                  ? 'bg-amber-950/90 border-amber-500/40 text-amber-200'
                  : t.type === 'dev'
                  ? 'bg-indigo-950/90 border-indigo-500/50 text-indigo-200'
                  : 'bg-slate-900/90 border-slate-700 text-slate-200'
              }`}
            >
              <div className="flex items-start gap-3 min-w-0">
                <div className="mt-0.5 shrink-0">
                  {t.type === 'success' && <CheckCircle2 className="w-5 h-5 text-emerald-400" />}
                  {t.type === 'error' && <AlertCircle className="w-5 h-5 text-rose-400" />}
                  {t.type === 'warning' && <AlertCircle className="w-5 h-5 text-amber-400" />}
                  {t.type === 'dev' && <Clock className="w-5 h-5 text-indigo-400 animate-pulse" />}
                  {t.type === 'info' && <Info className="w-5 h-5 text-blue-400" />}
                </div>
                <div>
                  {t.title && <div className="text-xs font-bold tracking-tight">{t.title}</div>}
                  <div className="text-xs text-slate-300 mt-0.5 leading-relaxed">{t.message}</div>
                </div>
              </div>
              <button
                onClick={() => removeToast(t.id)}
                className="p-1 rounded-lg text-slate-400 hover:text-white transition-colors"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </div>
          );
        })}
      </div>
    </ToastContext.Provider>
  );
};

export const useToast = () => {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return ctx;
};
