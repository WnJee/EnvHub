import { api, isTauri } from './tauri';

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

export const CURRENT_APP_VERSION = '0.2.10';

/**
 * Compare two semver strings (e.g. "0.1.1" vs "0.1.0")
 * Returns 1 if v1 > v2, -1 if v1 < v2, 0 if equal
 */
export function compareVersions(v1: string, v2: string): number {
  const clean1 = v1.replace(/^v/, '').split('.').map((n) => parseInt(n, 10) || 0);
  const clean2 = v2.replace(/^v/, '').split('.').map((n) => parseInt(n, 10) || 0);

  const len = Math.max(clean1.length, clean2.length);
  for (let i = 0; i < len; i++) {
    const num1 = clean1[i] || 0;
    const num2 = clean2[i] || 0;
    if (num1 > num2) return 1;
    if (num1 < num2) return -1;
  }
  return 0;
}

function decodeHtmlEntities(str: string): string {
  return str
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
}

function cleanHtmlToMarkdown(html: string): string {
  return decodeHtmlEntities(html)
    .replace(/<h3>(.*?)<\/h3>/gi, '### $1\n')
    .replace(/<h4>(.*?)<\/h4>/gi, '#### $1\n')
    .replace(/<h5>(.*?)<\/h5>/gi, '##### $1\n')
    .replace(/<strong>(.*?)<\/strong>/gi, '**$1**')
    .replace(/<b>(.*?)<\/b>/gi, '**$1**')
    .replace(/<code>(.*?)<\/code>/gi, '`$1`')
    .replace(/<li>(.*?)<\/li>/gi, '- $1\n')
    .replace(/<\/?(ul|ol|p|div|span)>/gi, '\n')
    .replace(/\n\s*\n\s*\n/g, '\n\n')
    .trim();
}

function getPlatformDownloadUrl(latestVer: string): string {
  const userAgent = typeof navigator !== 'undefined' ? navigator.userAgent.toLowerCase() : '';
  const isWindows = userAgent.includes('win');
  const isLinux = userAgent.includes('linux');

  if (isWindows) {
    return `https://github.com/WnJee/EnvHub/releases/download/v${latestVer}/EnvHub_${latestVer}_x64-setup.exe`;
  }
  if (isLinux) {
    return `https://github.com/WnJee/EnvHub/releases/download/v${latestVer}/EnvHub_${latestVer}_amd64.AppImage`;
  }
  return `https://github.com/WnJee/EnvHub/releases/download/v${latestVer}/EnvHub_${latestVer}_universal.dmg`;
}

/**
 * Check for updates with multi-tier fallback (Backend curl -> GitHub API -> releases.atom -> redirect)
 */
export async function checkForUpdates(currentVersion: string = CURRENT_APP_VERSION): Promise<UpdateInfo> {
  let latestVer = '';
  let releaseTitle = '';
  let releaseNotes = '';
  let releaseUrl = '';
  let publishedAt = '';
  let downloadUrl = '';

  // Tier 1: Try Tauri backend if in desktop app (bypasses browser CORS & direct curl)
  if (isTauri()) {
    try {
      const info = await api.checkGithubLatestRelease();
      if (info && info.tag_name) {
        latestVer = info.tag_name.replace(/^v/, '');
        releaseNotes = info.body || '';
        releaseUrl = info.html_url || `https://github.com/WnJee/EnvHub/releases/tag/v${latestVer}`;
        publishedAt = info.published_at ? new Date(info.published_at).toLocaleDateString() : '';
      }
    } catch (err) {
      console.warn('Backend update check failed, trying fallback:', err);
    }
  }

  // Tier 2: Try GitHub REST API
  if (!latestVer) {
    try {
      const response = await fetch('https://api.github.com/repos/WnJee/EnvHub/releases/latest', {
        headers: { Accept: 'application/vnd.github.v3+json' },
        cache: 'no-cache',
      });
      if (response.ok) {
        const data = await response.json();
        latestVer = (data.tag_name || '').replace(/^v/, '');
        releaseTitle = data.name || '';
        releaseNotes = data.body || '';
        releaseUrl = data.html_url || `https://github.com/WnJee/EnvHub/releases/tag/v${latestVer}`;
        publishedAt = data.published_at ? new Date(data.published_at).toLocaleDateString() : '';

        // Check assets
        if (data.assets && Array.isArray(data.assets) && data.assets.length > 0) {
          const userAgent = typeof navigator !== 'undefined' ? navigator.userAgent.toLowerCase() : '';
          const isWindows = userAgent.includes('win');
          const isLinux = userAgent.includes('linux');

          if (isWindows) {
            const winAsset = data.assets.find((a: any) => a.name.endsWith('.exe') || a.name.endsWith('.msi'));
            if (winAsset) downloadUrl = winAsset.browser_download_url;
          } else if (isLinux) {
            const linuxAsset = data.assets.find((a: any) => a.name.endsWith('.AppImage') || a.name.endsWith('.deb'));
            if (linuxAsset) downloadUrl = linuxAsset.browser_download_url;
          } else {
            const macAsset = data.assets.find((a: any) => a.name.endsWith('.dmg'));
            if (macAsset) downloadUrl = macAsset.browser_download_url;
          }
        }
      }
    } catch (e) {
      console.warn('GitHub API check failed, falling back to atom feed:', e);
    }
  }

  // Tier 3: Fallback to GitHub releases.atom (Zero rate limits, always works)
  if (!latestVer) {
    try {
      const response = await fetch('https://github.com/WnJee/EnvHub/releases.atom', { cache: 'no-cache' });
      if (response.ok) {
        const xml = await response.text();
        const entryMatch = xml.match(/<entry>([\s\S]*?)<\/entry>/);
        if (entryMatch) {
          const entry = entryMatch[1];
          const tagMatch = entry.match(/\/releases\/tag\/([^\"'\/<>\s]+)/);
          const titleMatch = entry.match(/<title>([\s\S]*?)<\/title>/);
          const updatedMatch = entry.match(/<updated>([\s\S]*?)<\/updated>/);
          const contentMatch = entry.match(/<content type=\"html\">([\s\S]*?)<\/content>/);

          if (tagMatch) {
            latestVer = tagMatch[1].replace(/^v/, '');
          }
          if (titleMatch) {
            releaseTitle = decodeHtmlEntities(titleMatch[1]);
          }
          if (contentMatch) {
            releaseNotes = cleanHtmlToMarkdown(contentMatch[1]);
          }
          if (updatedMatch) {
            publishedAt = new Date(updatedMatch[1]).toLocaleDateString();
          }
          releaseUrl = `https://github.com/WnJee/EnvHub/releases/tag/v${latestVer}`;
        }
      }
    } catch (e) {
      console.warn('releases.atom check failed, trying redirect:', e);
    }
  }

  // Tier 4: Fallback to GitHub latest release redirect URL
  if (!latestVer) {
    try {
      const res = await fetch('https://github.com/WnJee/EnvHub/releases/latest', { method: 'HEAD' });
      if (res.url && res.url.includes('/releases/tag/')) {
        const tag = res.url.split('/releases/tag/')[1].split('/')[0];
        latestVer = tag.replace(/^v/, '');
        releaseUrl = res.url;
      }
    } catch (e) {
      console.warn('redirect check failed:', e);
    }
  }

  if (!latestVer) {
    return {
      hasUpdate: false,
      latestVersion: currentVersion,
      currentVersion,
    };
  }

  if (!downloadUrl) {
    downloadUrl = getPlatformDownloadUrl(latestVer);
  }

  const hasUpdate = compareVersions(latestVer, currentVersion) > 0;

  return {
    hasUpdate,
    latestVersion: latestVer,
    currentVersion,
    releaseTitle: releaseTitle || `EnvHub v${latestVer}`,
    releaseNotes: releaseNotes || '修复已知问题与体验优化',
    releaseUrl: releaseUrl || `https://github.com/WnJee/EnvHub/releases/tag/v${latestVer}`,
    publishedAt: publishedAt || new Date().toLocaleDateString(),
    downloadUrl,
  };
}
