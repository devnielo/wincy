// Text transforms for the quick actions (Ctrl+K). Pure: the result is pasted, the clip is never modified.

export const TRANSFORMS = [
  "upper",
  "lower",
  "title",
  "trim",
  "collapse",
  "joinLines",
  "jsonFormat",
  "jsonMinify",
  "base64Encode",
  "base64Decode",
  "urlEncode",
  "urlDecode",
] as const;

export type TransformId = (typeof TRANSFORMS)[number];

function parseJsonContainer(text: string): unknown {
  const t = text.trim();
  // A bare number or string is valid JSON but nobody wants to "format" it.
  if (!/^[[{]/.test(t)) throw new Error("not a JSON object or array");
  return JSON.parse(t);
}

function base64Encode(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let binary = "";
  // Chunked: String.fromCharCode(...bytes) overflows the stack on large clips.
  for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(binary);
}

function base64Decode(text: string): string {
  const compact = text.replace(/\s+/g, "");
  if (!/^[A-Za-z0-9+/_-]+={0,2}$/.test(compact) || compact.length % 4 === 1) throw new Error("not base64");
  const standard = compact.replace(/-/g, "+").replace(/_/g, "/").replace(/=+$/, "");
  const binary = atob(standard.padEnd(Math.ceil(standard.length / 4) * 4, "="));
  const decoded = new TextDecoder("utf-8", { fatal: true }).decode(Uint8Array.from(binary, (c) => c.charCodeAt(0)));
  // Random words ("test", "abcd") are valid base64 too; only offer decodes that read as text.
  if (!decoded || /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F�]/.test(decoded)) throw new Error("not text");
  return decoded;
}

const RUN: Record<TransformId, (text: string) => string> = {
  upper: (t) => t.toUpperCase(),
  lower: (t) => t.toLowerCase(),
  title: (t) => t.toLowerCase().replace(/(^|[^\p{L}\p{N}'’])(\p{L})/gu, (_, before: string, c: string) => before + c.toUpperCase()),
  trim: (t) => t.trim(),
  collapse: (t) =>
    t
      .split(/\r?\n/)
      .map((line) => line.replace(/[^\S\r\n]+/g, " ").trim())
      .join("\n")
      .replace(/\n{3,}/g, "\n\n")
      .trim(),
  joinLines: (t) => t.replace(/[^\S\r\n]*(?:\r?\n[^\S\r\n]*)+/g, " ").trim(),
  jsonFormat: (t) => JSON.stringify(parseJsonContainer(t), null, 2),
  jsonMinify: (t) => JSON.stringify(parseJsonContainer(t)),
  base64Encode,
  base64Decode,
  urlEncode: (t) => encodeURIComponent(t),
  urlDecode: (t) => {
    if (!/%[0-9A-Fa-f]{2}/.test(t)) throw new Error("nothing to decode");
    return decodeURIComponent(t);
  },
};

/** The transformed text, or null when the transform doesn't apply (invalid input, or nothing would change). */
export function applyTransform(id: TransformId, text: string): string | null {
  if (!text.trim()) return null;
  try {
    const result = RUN[id](text);
    return result && result !== text ? result : null;
  } catch {
    return null;
  }
}

/** Every transform that applies to `text`, in menu order, with its result. */
export function applicableTransforms(text: string): { id: TransformId; result: string }[] {
  const seen = new Set<string>(); // e.g. Title Case of "#1E90FF" equals its lowercase
  return TRANSFORMS.flatMap((id) => {
    const result = applyTransform(id, text);
    if (result === null || seen.has(result)) return [];
    seen.add(result);
    return [{ id, result }];
  });
}

export type SnippetToken = { text: string; variable: boolean };

const VARIABLE = /\{\{|\{(?:date|time|datetime|clipboard)\}/g;

/** Splits a snippet body into literal text and the variables the backend expands (`{{` is an escaped `{`, not a variable). */
export function snippetTokens(body: string): SnippetToken[] {
  const tokens: SnippetToken[] = [];
  let last = 0;
  for (const m of body.matchAll(VARIABLE)) {
    if (m[0] === "{{") continue;
    if (m.index > last) tokens.push({ text: body.slice(last, m.index), variable: false });
    tokens.push({ text: m[0], variable: true });
    last = m.index + m[0].length;
  }
  if (last < body.length) tokens.push({ text: body.slice(last), variable: false });
  return tokens;
}
