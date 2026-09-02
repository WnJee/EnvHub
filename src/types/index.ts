export type ToolCategory = 'runtime' | 'system' | 'package';

export interface VersionInfo {
  version: string;
  isInstalled: boolean;
  isActive: boolean;
  isLts?: boolean;
  isLatest?: boolean;
  installPath?: string;
  source?: string; // 'mise' | 'system'
}

export interface RuntimeTool {
  id: string;
  name: string;
  category: ToolCategory;
  description: string;
  icon: string;
  officialSite: string;
  installedVersions: string[];
  activeVersion?: string;
  globalVersion?: string;
  availableVersions: string[];
  isLoading?: boolean;
  isCustomPlugin?: boolean;
}

export interface ProjectEnv {
  id: string;
  name: string;
  path: string;
  configFile?: string; // '.mise.toml', '.tool-versions', 'package.json'
  tools: {
    toolId: string;
    version: string;
  }[];
  lastModified?: string;
}

export interface SystemTool {
  id: string;
  name: string;
  description: string;
  category: string;
  isInstalled: boolean;
  installedVersion?: string;
  installCommand: string;
  icon: string;
  homepage: string;
}

export interface MirrorConfig {
  id: string;
  name: string;
  tool: 'npm' | 'pip' | 'cargo' | 'go' | 'brew';
  currentMirror: string;
  options: {
    name: string;
    url: string;
    ping?: number; // ms
    isDefault?: boolean;
  }[];
}

export interface EnvHealthCheck {
  id: string;
  title: string;
  status: 'ok' | 'warning' | 'error';
  message: string;
  shell: string;
  configFile: string;
  canAutoFix: boolean;
  fixCommand?: string;
}

export interface InstallLog {
  id: string;
  timestamp: string;
  toolId: string;
  version: string;
  status: 'running' | 'completed' | 'failed';
  logs: string[];
  progress?: number; // 0 - 100
}

export interface SystemStatus {
  os: 'macos' | 'windows' | 'linux';
  osVersion: string;
  arch: string;
  defaultShell: string;
  miseInstalled: boolean;
  miseVersion?: string;
  misePath?: string;
  packageManager: 'brew' | 'winget' | 'apt' | 'pacman' | 'none';
}
