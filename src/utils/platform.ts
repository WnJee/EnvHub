/**
 * Single source of truth for the host platform in the frontend.
 *
 * Before this module three places each re-derived the platform from
 * `navigator.userAgent` with slightly different rules. Keep the detection here
 * and prefer the backend's authoritative `os` field (`get_system_status`) when
 * it is available — `setBackendPlatform()` lets `App` override the guess.
 */

export type Platform = 'macos' | 'windows' | 'linux' | 'unknown';

function detectFromUserAgent(): Platform {
  if (typeof navigator === 'undefined') return 'unknown';
  const ua = navigator.userAgent.toLowerCase();
  // Check Windows before Linux: some Windows UAs embed "linux" (WSL, Android).
  if (ua.includes('win')) return 'windows';
  if (ua.includes('mac') || ua.includes('darwin')) return 'macos';
  if (ua.includes('linux') || ua.includes('x11')) return 'linux';
  return 'unknown';
}

let platform: Platform | null = null;

/** Resolved once; safe to call during render. */
export function getPlatform(): Platform {
  if (platform === null) platform = detectFromUserAgent();
  return platform;
}

/**
 * Override the user-agent guess with the authoritative value reported by the
 * Rust side (`SystemStatus.os`). Accepts the raw backend string.
 */
export function setBackendPlatform(os: string | undefined | null): void {
  if (!os) return;
  const value = os.toLowerCase();
  if (value.includes('mac') || value.includes('darwin')) platform = 'macos';
  else if (value.includes('win')) platform = 'windows';
  else if (value.includes('linux')) platform = 'linux';
}

export const isMac = (): boolean => getPlatform() === 'macos';
export const isWindows = (): boolean => getPlatform() === 'windows';
export const isLinux = (): boolean => getPlatform() === 'linux';
