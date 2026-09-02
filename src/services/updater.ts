export interface UpdateInfo {
  hasUpdate: boolean;
  latestVersion: string;
  currentVersion: string;
  releaseNotes?: string;
  releaseUrl?: string;
  publishedAt?: string;
  downloadUrl?: string;
}

export const CURRENT_APP_VERSION = '0.2.4';

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

/**
 * Check for updates from GitHub Releases API
 */
export async function checkForUpdates(currentVersion: string = CURRENT_APP_VERSION): Promise<UpdateInfo> {
  try {
    const response = await fetch('https://api.github.com/repos/WnJee/EnvHub/releases/latest', {
      headers: {
        Accept: 'application/vnd.github.v3+json',
      },
      cache: 'no-cache',
    });

    if (!response.ok) {
      throw new Error(`GitHub API HTTP ${response.status}`);
    }

    const data = await response.json();
    const tagName = data.tag_name || '';
    const latestVer = tagName.replace(/^v/, '');

    const hasUpdate = compareVersions(latestVer, currentVersion) > 0;

    // Pick OS specific download asset
    let downloadUrl = `https://github.com/WnJee/EnvHub/releases/download/v${latestVer}/EnvHub_${latestVer}_universal.dmg`;

    const userAgent = typeof navigator !== 'undefined' ? navigator.userAgent.toLowerCase() : '';
    const isWindows = userAgent.includes('win');
    const isLinux = userAgent.includes('linux');

    if (isWindows) {
      downloadUrl = `https://github.com/WnJee/EnvHub/releases/download/v${latestVer}/EnvHub_${latestVer}_x64-setup.exe`;
    } else if (isLinux) {
      downloadUrl = `https://github.com/WnJee/EnvHub/releases/download/v${latestVer}/EnvHub_${latestVer}_amd64.AppImage`;
    }

    if (data.assets && Array.isArray(data.assets) && data.assets.length > 0) {
      if (isWindows) {
        const winAsset = data.assets.find((a: { name: string }) => a.name.endsWith('.exe') || a.name.endsWith('.msi'));
        if (winAsset) downloadUrl = winAsset.browser_download_url;
      } else if (isLinux) {
        const linuxAsset = data.assets.find((a: { name: string }) => a.name.endsWith('.AppImage') || a.name.endsWith('.deb'));
        if (linuxAsset) downloadUrl = linuxAsset.browser_download_url;
      } else {
        // macOS default
        const macAsset = data.assets.find((a: { name: string }) => a.name.endsWith('.dmg'));
        if (macAsset) downloadUrl = macAsset.browser_download_url;
      }
    }

    return {
      hasUpdate,
      latestVersion: latestVer,
      currentVersion,
      releaseNotes: data.body || '无更新说明',
      releaseUrl: data.html_url,
      publishedAt: data.published_at ? new Date(data.published_at).toLocaleDateString() : '',
      downloadUrl,
    };
  } catch (err) {
    console.warn('Check update failed:', err);
    return {
      hasUpdate: false,
      latestVersion: currentVersion,
      currentVersion,
    };
  }
}
