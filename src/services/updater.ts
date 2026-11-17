import { api, isTauri } from './tauri';
import { getPlatform } from '../utils/platform';

export interface UpdateInfo {
  hasUpdate: boolean;
  latestVersion: string;
  currentVersion: string;
  releaseTitle?: string;
  releaseNotes?: string;
  releaseUrl?: string;
  publishedAt?: string;
  downloadUrl?: string;
}

/**
 * Fallback shown before `getVersion()` resolves (and in browser preview).
 * The authoritative value comes from the bundle via `resolveAppVersion()`.
 * Injected from `package.json` at build time (`vite.config.ts` → `define:
 * __APP_VERSION__`), because a hand-edited literal here used to stay at 0.2.20
 * while the app already shipped 0.2.22 — which made the update dialog report a
 * bogus "0.2.20 → 0.2.22" upgrade on every single launch.
 */
export const CURRENT_APP_VERSION = __APP_VERSION__;

let resolvedAppVersion: string | null = null;

/**
 * Read the real app version from the Tauri bundle instead of trusting the
 * hard-coded constant (which drifts from `tauri.conf.json` / `Cargo.toml`).
 */
export async function resolveAppVersion(): Promise<string> {
  if (resolvedAppVersion) return resolvedAppVersion;
  if (isTauri()) {
    try {
      const { getVersion } = await import('@tauri-apps/api/app');
      resolvedAppVersion = await getVersion();
      return resolvedAppVersion;
    } catch {
      // Fall through to the compiled-in fallback.
    }
  }
  resolvedAppVersion = CURRENT_APP_VERSION;
  return resolvedAppVersion;
}

/** `[major, minor, patch, isRelease]`: a pre-release sorts below its release. */
function versionKey(version: string): number[] {
  const [core, preRelease] = version.replace(/^v/, '').split('+')[0].split('-', 2);
  const parts = core.split('.').map((n) => parseInt(n, 10) || 0);
  return [parts[0] || 0, parts[1] || 0, parts[2] || 0, preRelease === undefined ? 1 : 0];
}

/**
 * Compare two semver strings (e.g. "0.1.1" vs "0.1.0")
 * Returns 1 if v1 > v2, -1 if v1 < v2, 0 if equal
 */
export function compareVersions(v1: string, v2: string): number {
  const key1 = versionKey(v1);
  const key2 = versionKey(v2);
  for (let i = 0; i < key1.length; i++) {
    if (key1[i] !== key2[i]) return key1[i] > key2[i] ? 1 : -1;
  }
  return 0;
}

/**
 * `AbortSignal.timeout()` only exists in WebKitGTK >= 2.40 / Safari 16+, so on
 * older Linux distros the update check threw a TypeError and silently never
 * worked. Fall back to a plain AbortController.
 */
function timeoutSignal(ms: number): AbortSignal {
  const factory = (AbortSignal as unknown as { timeout?: (ms: number) => AbortSignal }).timeout;
  if (typeof factory === 'function') return factory.call(AbortSignal, ms);
  const controller = new AbortController();
  setTimeout(() => controller.abort(), ms);
  return controller.signal;
}

/**
 * Installer published for this machine, or `null` when the release has none
 * (Linux builds are x86_64 only; Windows on ARM runs the x64 installer).
 */
function installerSuffix(arch: string): string | null {
  switch (getPlatform()) {
    case 'windows':
      return '_x64-setup.exe';
    case 'linux':
      return arch === 'x86_64' ? '_amd64.AppImage' : null;
    default:
      return '_universal.dmg';
  }
}

interface LatestRelease {
  tag: string;
  title?: string;
  notes?: string;
  url?: string;
  publishedAt?: string;
}

/**
 * Backend first (curl, no CORS limits), GitHub REST API second. The Atom feed
 * and the `/releases/latest` redirect send no CORS headers, so only the backend
 * can use them.
 */
async function fetchLatestRelease(): Promise<LatestRelease> {
  if (isTauri()) {
    try {
      const info = await api.checkGithubLatestRelease();
      if (info?.tag_name) {
        return { tag: info.tag_name, notes: info.body, url: info.html_url, publishedAt: info.published_at };
      }
    } catch (err) {
      console.warn('Backend update check failed, trying GitHub API:', err);
    }
  }

  const response = await fetch('https://api.github.com/repos/WnJee/EnvHub/releases/latest', {
    headers: { Accept: 'application/vnd.github.v3+json' },
    cache: 'no-cache',
    signal: timeoutSignal(10000),
  });
  const data = response.ok ? await response.json() : null;
  if (!data?.tag_name) throw new Error('无法连接更新服务');
  return { tag: data.tag_name, title: data.name, notes: data.body, url: data.html_url, publishedAt: data.published_at };
}

/** `arch` is the backend's `SystemStatus.arch`. */
export async function checkForUpdates(
  currentVersion: string = CURRENT_APP_VERSION,
  arch = 'x86_64'
): Promise<UpdateInfo> {
  const release = await fetchLatestRelease();
  const latestVersion = release.tag.replace(/^v/, '');
  const suffix = installerSuffix(arch);

  return {
    hasUpdate: compareVersions(latestVersion, currentVersion) > 0,
    latestVersion,
    currentVersion,
    releaseTitle: release.title || `EnvHub v${latestVersion}`,
    releaseNotes: release.notes || '修复已知问题与体验优化',
    releaseUrl: release.url || `https://github.com/WnJee/EnvHub/releases/tag/v${latestVersion}`,
    publishedAt: new Date(release.publishedAt || Date.now()).toLocaleDateString(),
    downloadUrl: suffix
      ? `https://github.com/WnJee/EnvHub/releases/download/v${latestVersion}/EnvHub_${latestVersion}${suffix}`
      : undefined,
  };
}
