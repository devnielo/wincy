/**
 * The backend compares exe file names case-insensitively, so store just the lowercased file
 * name. A bare name ("Bitwarden", as history rows show it) gets ".exe", or it would never match.
 */
export function normalize(value: string): string {
  const n = (value.trim().split(/[\\/]/).pop() ?? "").trim().toLowerCase();
  return n && !/\.[a-z0-9]+$/.test(n) ? `${n}.exe` : n;
}
