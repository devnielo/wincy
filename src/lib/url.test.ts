import { test } from "node:test";
import assert from "node:assert/strict";
import { browserUrl } from "./url.ts";

test("opens only http, https and mailto", () => {
  assert.equal(browserUrl(" https://github.com/devnielo/wincy "), "https://github.com/devnielo/wincy");
  assert.equal(browserUrl("www.example.org/docs"), "https://www.example.org/docs");
  assert.equal(browserUrl("mailto:hello@example.com"), "mailto:hello@example.com");
  assert.equal(browserUrl("javascript:alert(1)"), null);
  assert.equal(browserUrl("file:///C:/Windows/System32/calc.exe"), null);
  assert.equal(browserUrl("ftp://example.com"), null);
  assert.equal(browserUrl("not a url"), null);
});
