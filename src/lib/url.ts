/** The URL to open for a url clip: http/https/mailto only, so javascript:, file: or ftp: never reach the opener. */
export function browserUrl(text: string): string | null {
  const raw = text.trim();
  try {
    const url = new URL(/^www\./i.test(raw) ? `https://${raw}` : raw);
    return ["http:", "https:", "mailto:"].includes(url.protocol) ? url.href : null;
  } catch {
    return null;
  }
}
