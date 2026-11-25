import { test } from "node:test";
import assert from "node:assert/strict";
import { matchesApp, parseQuery } from "./query.ts";

test("splits app: tokens from the fuzzy text", () => {
  assert.deepEqual(parseQuery("app:code debounce"), { apps: ["code"], text: "debounce" });
  assert.deepEqual(parseQuery("  APP:Chrome  app:edge "), { apps: ["chrome", "edge"], text: "" });
  assert.deepEqual(parseQuery("fix the app: bug"), { apps: [], text: "fix the bug" });
  assert.deepEqual(parseQuery("snapp:x myapp:y"), { apps: [], text: "snapp:x myapp:y" });
  assert.deepEqual(parseQuery("hello   world"), { apps: [], text: "hello world" });
  assert.deepEqual(parseQuery(""), { apps: [], text: "" });
  assert.deepEqual(parseQuery("app:Chrome.EXE"), { apps: ["chrome"], text: "" });
  assert.deepEqual(parseQuery("app:.exe x"), { apps: [], text: "x" });
});

test("matches the source app without .exe, case-insensitively, OR across terms", () => {
  assert.equal(matchesApp("Code.exe", ["code"]), true);
  assert.equal(matchesApp("msedge.exe", ["chrome", "edge"]), true);
  assert.equal(matchesApp("chrome.exe", ["exe"]), false);
  assert.equal(matchesApp(null, ["code"]), false);
  assert.equal(matchesApp(null, []), true);
});
