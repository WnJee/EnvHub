import React, { useState } from 'react';
import { ProjectEnv, RuntimeTool } from '../types';
import { 
  FolderGit2, 
  Plus, 
  FolderSearch, 
  Code, 
  FileCode, 
  Check, 
  Terminal, 
  Clock,
  ShieldAlert,
  Trash2
} from 'lucide-react';
import { api } from '../services/tauri';
import { useToast } from './Toast';

interface ProjectManagerProps {
  projects: ProjectEnv[];
  runtimes: RuntimeTool[];
  onAddProject: (path: string) => void;
  onRemoveProject?: (path: string) => void;
  onSetProjectToolVersion: (projectId: string, toolId: string, version: string) => void;
}

export const ProjectManager: React.FC<ProjectManagerProps> = ({
  projects,
  runtimes,
  onAddProject,
  onRemoveProject,
  onSetProjectToolVersion,
}) => {
  const [newProjectPath, setNewProjectPath] = useState<string>('');
  const [isAdding, setIsAdding] = useState<boolean>(false);
  const toast = useToast();

  const handleCreateProject = () => {
    if (newProjectPath.trim()) {
      onAddProject(newProjectPath.trim());
      setNewProjectPath('');
      setIsAdding(false);
    }
  };

  const handleOpenVSCode = async (path: string) => {
    try {
      await api.openInEditor(path);
      toast.success(`已在 VS Code 中打开: ${path}`);
    } catch (err) {
      toast.error(`打开编辑器失败: ${err}`);
    }
  };

  const handleOpenTerminal = async (path: string) => {
    try {
      await api.openInTerminal(path);
      toast.success(`已在终端打开目录: ${path}`);
    } catch (err) {
      toast.error(`打开终端失败: ${err}`);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-4 sm:p-6 space-y-5 bg-[#090D16]">
      {/* Top Banner */}
      <div className="p-4 sm:p-5 rounded-2xl bg-gradient-to-r from-indigo-950/40 via-slate-900 to-slate-900 border border-slate-800/80 flex flex-col sm:flex-row sm:items-center justify-between gap-4 shadow-xl">
        <div>
          <div className="flex items-center gap-2 text-indigo-400 font-mono text-xs uppercase font-semibold">
            <FolderGit2 className="w-3.5 h-3.5" />
            Project Isolation
          </div>
          <h2 className="text-lg sm:text-xl font-bold text-white mt-1">项目级环境与版本隔离</h2>
          <p className="text-xs text-slate-400 mt-0.5 max-w-xl">
            自动识别与写入 <code className="text-blue-400">.mise.toml</code> 或 <code className="text-blue-400">.tool-versions</code>，实现工程间版本独立管理。
          </p>
        </div>

        <button
          onClick={() => setIsAdding(true)}
          className="px-3.5 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-lg shadow-blue-500/20 flex items-center gap-2 transition-all shrink-0"
        >
          <Plus className="w-3.5 h-3.5" /> 添加工程目录
        </button>
      </div>

      {/* Add Project Modal / Input */}
      {isAdding && (
        <div className="p-4 rounded-2xl bg-slate-900 border border-blue-500/40 shadow-xl space-y-3 animate-in fade-in zoom-in-95 duration-150">
          <div className="flex items-center justify-between">
            <h3 className="text-xs sm:text-sm font-bold text-white flex items-center gap-2">
              <FolderSearch className="w-4 h-4 text-blue-400" />
              添加本地代码工程绝对路径
            </h3>
            <button
              onClick={() => setIsAdding(false)}
              className="text-xs text-slate-400 hover:text-slate-200"
            >
              取消
            </button>
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              placeholder="例如: /Users/username/workspace/my-app"
              value={newProjectPath}
              onChange={(e) => setNewProjectPath(e.target.value)}
              className="flex-1 bg-slate-950 border border-slate-800 rounded-xl px-3.5 py-1.5 text-xs text-white placeholder-slate-400 focus:outline-none focus:border-blue-500 font-mono"
            />
            <button
              onClick={handleCreateProject}
              className="px-4 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold transition-colors"
            >
              绑定工程
            </button>
          </div>
        </div>
      )}

      {/* Project Cards Grid */}
      {projects.length === 0 ? (
        <div className="p-10 rounded-2xl bg-slate-900/40 border border-dashed border-slate-800 text-center">
          <ShieldAlert className="w-8 h-8 text-slate-600 mx-auto mb-2.5" />
          <h3 className="text-xs sm:text-sm font-semibold text-slate-300">暂无绑定的工程项目</h3>
          <p className="text-xs text-slate-400 mt-1">点击右上角按钮添加本地项目目录进行版本配置</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
          {projects.map((project) => (
            <div
              key={project.id}
              className="p-4 sm:p-5 rounded-2xl bg-slate-900/70 border border-slate-800 hover:border-slate-700 transition-all flex flex-col justify-between group shadow-sm"
            >
              <div>
                {/* Project Header */}
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-sm sm:text-base text-white tracking-tight truncate">
                        {project.name}
                      </span>
                      <span className="text-[9px] font-mono px-1.5 py-0.2 rounded-full bg-slate-800 text-slate-300 border border-slate-700">
                        {project.configFile || '自动配置'}
                      </span>
                    </div>
                    <div className="text-[11px] text-slate-400 font-mono truncate mt-0.5 flex items-center gap-1.5">
                      <FileCode className="w-3 h-3 text-slate-400 shrink-0" />
                      <span className="truncate">{project.path}</span>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 shrink-0">
                    <span className="text-[10px] text-slate-400 flex items-center gap-1 font-mono">
                      <Clock className="w-3 h-3" />
                      {project.lastModified || '已同步'}
                    </span>
                    {onRemoveProject && (
                      <button
                        onClick={() => onRemoveProject(project.path)}
                        title="移除此工程绑定"
                        className="p-1 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    )}
                  </div>
                </div>

                {/* Bound Tools & Versions */}
                <div className="mt-3.5 pt-3 border-t border-slate-800/80 space-y-2">
                  <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400 font-mono">
                    工程语言与版本
                  </div>

                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                    {project.tools.map((item) => {
                      const toolInfo = runtimes.find((r) => r.id === item.toolId);
                      return (
                        <div
                          key={item.toolId}
                          className="p-2 rounded-xl bg-slate-950/80 border border-slate-800 flex items-center justify-between gap-2"
                        >
                          <div className="flex items-center gap-2 min-w-0">
                            {toolInfo?.icon && (
                              <img
                                src={toolInfo.icon}
                                alt={item.toolId}
                                className="w-3.5 h-3.5 object-contain shrink-0"
                              />
                            )}
                            <span className="text-xs font-semibold text-slate-200 capitalize truncate">
                              {toolInfo?.name || item.toolId}
                            </span>
                          </div>

                          {/* Version Switcher Dropdown */}
                          <select
                            value={item.version}
                            onChange={(e) =>
                              onSetProjectToolVersion(project.id, item.toolId, e.target.value)
                            }
                            className="bg-slate-900 border border-slate-700 text-blue-400 text-xs font-mono rounded-lg px-2 py-0.5 focus:outline-none focus:border-blue-500 cursor-pointer"
                          >
                            <option value={item.version}>v{item.version}</option>
                            {toolInfo?.installedVersions
                              .filter((v) => v !== item.version)
                              .map((v) => (
                                <option key={v} value={v}>
                                  v{v}
                                </option>
                              ))}
                          </select>
                        </div>
                      );
                    })}
                  </div>
                </div>
              </div>

              {/* Bottom Actions */}
              <div className="mt-4 pt-2.5 border-t border-slate-800/60 flex items-center justify-between text-xs">
                <div className="text-[10px] text-slate-400 flex items-center gap-1 font-mono">
                  <Check className="w-3 h-3 text-emerald-400" />
                  进入目录自动生效
                </div>

                <div className="flex items-center gap-1.5">
                  <button
                    onClick={() => handleOpenVSCode(project.path)}
                    className="px-2 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-[11px] font-medium transition-colors flex items-center gap-1"
                  >
                    <Code className="w-3 h-3 text-blue-400" /> VS Code
                  </button>
                  <button
                    onClick={() => handleOpenTerminal(project.path)}
                    className="px-2 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-[11px] font-medium transition-colors flex items-center gap-1"
                  >
                    <Terminal className="w-3 h-3 text-emerald-400" /> 终端
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
