import { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught error:', error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen bg-[#090D16] text-slate-200 flex flex-col items-center justify-center p-6 text-center select-none">
          <div className="p-4 rounded-2xl bg-rose-500/10 border border-rose-500/30 text-rose-400 mb-4 shadow-lg shadow-rose-500/10">
            <AlertTriangle className="w-10 h-10" />
          </div>
          <h2 className="text-lg font-bold text-white mb-2">应用界面遇到异常</h2>
          <p className="text-xs text-slate-400 max-w-md mb-6 leading-relaxed">
            {this.state.error?.message || '组件渲染过程中发生了意外异常，已为您隔离错误保护应用。'}
          </p>
          <button
            onClick={() => window.location.reload()}
            className="px-4 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-lg shadow-blue-500/20 transition-all flex items-center gap-2"
          >
            <RefreshCw className="w-4 h-4" /> 重新加载应用
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
