// Shared number formatting for download/transfer readouts.

const UNITS = ['B', 'KB', 'MB', 'GB', 'TB'] as const;

/** `12.4 MB`; `null` for absent or non-positive values so the UI can skip the chip. */
export function formatBytes(bytes?: number | null): string | null {
  if (!bytes || bytes <= 0) return null;
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value >= 100 || unit === 0 ? Math.round(value) : value.toFixed(1)} ${UNITS[unit]}`;
}

/** `45s`, `2m05s`, `1h02m`; `null` when unknown or negative. */
export function formatEta(seconds?: number | null): string | null {
  if (seconds === null || seconds === undefined || seconds < 0) return null;
  const total = Math.round(seconds);
  if (total < 60) return `${Math.max(1, total)}s`;
  if (total < 3600) {
    return `${Math.floor(total / 60)}m${String(total % 60).padStart(2, '0')}s`;
  }
  return `${Math.floor(total / 3600)}h${String(Math.floor((total % 3600) / 60)).padStart(2, '0')}m`;
}
