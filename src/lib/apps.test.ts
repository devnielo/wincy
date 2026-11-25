import { test } from "node:test";
import assert from "node:assert/strict";
import { normalize } from "./apps.ts";

test("normalizes ignored app names to exe file names", () => {
  assert.equal(normalize(" Bitwarden "), "bitwarden.exe");
  assert.equal(normalize("Proton Pass"), "proton pass.exe");
  assert.equal(normalize("KeePassXC.EXE"), "keepassxc.exe");
  assert.equal(normalize("C:\\Program Files\\Keeper\\keeper.exe"), "keeper.exe");
  assert.equal(normalize("   "), "");
});
