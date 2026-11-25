export interface ParsedQuery {
  /** Lowercased `app:` terms; a clip matches if its app contains any of them. */
  apps: string[];
  /** The rest of the query, for fuzzy search. */
  text: string;
}

/** Splits `app:<term>` tokens (case-insensitive, repeatable) from the fuzzy-search text. */
export function parseQuery(query: string): ParsedQuery {
  const apps: string[] = [];
  const rest: string[] = [];
  for (const token of query.trim().split(/\s+/)) {
    const m = /^app:(.*)$/i.exec(token);
    const term = m?.[1].toLowerCase().replace(/\.exe$/, ""); // as matchesApp does, so "app:chrome.exe" works
    if (!m) rest.push(token);
    else if (term) apps.push(term); // a bare "app:" (still typing) filters nothing
  }
  return { apps, text: rest.join(" ") };
}

export function matchesApp(sourceApp: string | null, apps: string[]): boolean {
  if (apps.length === 0) return true;
  const name = sourceApp?.toLowerCase().replace(/\.exe$/, "");
  return !!name && apps.some((term) => name.includes(term));
}
