import { test } from "node:test";
import assert from "node:assert/strict";
import { ACCENTS, resolveAccent, type FixedAccent } from "./accent.ts";
import { contrastRatio, parseColor } from "./color.ts";

test("every accent keeps AA contrast for its text, in both modes", () => {
  for (const name of Object.keys(ACCENTS) as FixedAccent[]) {
    for (const dark of [false, true]) {
      const { color, foreground } = resolveAccent(name, null, dark);
      assert.ok(contrastRatio(parseColor(color)!, parseColor(foreground)!) >= 4.5, `${name} dark=${dark}`);
    }
  }
});

test("windows accent uses the system color, else blue", () => {
  assert.deepEqual(resolveAccent("windows", "#ffd700", true), { color: "#FFD700", foreground: "#000000" });
  assert.equal(resolveAccent("windows", "#0078d4", true).foreground, "#000000"); // lightened for the dark surface
  assert.equal(resolveAccent("windows", null, true).color, ACCENTS.blue[1]);
  assert.equal(resolveAccent("windows", "red;}", false).color, ACCENTS.blue[0]);
});

test("system accent text reaches AA on the surface, in both modes", () => {
  for (const sys of ["#0078D4", "#FFB900", "#0063B1", "#C30052", "#ffffff", "#000000"]) {
    for (const dark of [false, true]) {
      const { color, foreground } = resolveAccent("windows", sys, dark);
      const surface = parseColor(dark ? "#202020" : "#F3F3F3")!;
      assert.ok(contrastRatio(parseColor(color)!, surface) >= 4.5, `${sys} dark=${dark}`);
      assert.ok(contrastRatio(parseColor(color)!, parseColor(foreground)!) >= 4.5, `${sys} fg dark=${dark}`);
    }
  }
});
