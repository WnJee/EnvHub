export const MAX_LOG_LINES = 2000;
export const MAX_LOG_LINE_LENGTH = 8192;

export function appendLogs(previous: string[], incoming: string[]): string[] {
  const bounded = incoming.slice(-MAX_LOG_LINES).map(line => line.slice(0, MAX_LOG_LINE_LENGTH));
  const keep = MAX_LOG_LINES - bounded.length;
  return [...(keep ? previous.slice(-keep) : []), ...bounded];
}
