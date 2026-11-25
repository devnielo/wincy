// Fills the winget manifest templates for one release.
// Usage: node packaging/winget/render.mjs <version> <path to Wincy_<version>_x64-setup.exe>
// Hash the installer downloaded from the GitHub release, not a local build: winget checks the hosted file.
import { createHash } from "node:crypto";
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const [version, installer] = process.argv.slice(2);
if (!/^\d+\.\d+\.\d+$/.test(version ?? "") || !installer) {
  console.error("Usage: node packaging/winget/render.mjs <X.Y.Z> <path to Wincy_X.Y.Z_x64-setup.exe>");
  process.exit(1);
}
const name = `Wincy_${version}_x64-setup.exe`;
if (basename(installer) !== name) {
  console.error(`Expected the installer file ${name}, got ${basename(installer)}.`);
  process.exit(1);
}

const values = {
  VERSION: version,
  INSTALLER_URL: `https://github.com/devnielo/wincy/releases/download/v${version}/${name}`,
  SHA256: createHash("sha256").update(readFileSync(installer)).digest("hex").toUpperCase(),
};

const here = dirname(fileURLToPath(import.meta.url));
const templates = join(here, "templates");
const out = join(here, "manifests", "d", "devnielo", "Wincy", version);
mkdirSync(out, { recursive: true });
for (const file of readdirSync(templates)) {
  const text = readFileSync(join(templates, file), "utf8").replace(/\{\{(\w+)\}\}/g, (m, key) => values[key] ?? m);
  if (text.includes("{{")) throw new Error(`Unfilled placeholder left in ${file}`);
  writeFileSync(join(out, file), text);
}
console.log(`Wrote ${out}\nSHA256 ${values.SHA256}`);
