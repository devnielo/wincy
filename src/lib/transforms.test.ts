import { test } from "node:test";
import assert from "node:assert/strict";
import { applicableTransforms, applyTransform, snippetTokens } from "./transforms.ts";

test("case transforms are unicode-aware", () => {
  assert.equal(applyTransform("upper", "straße ñandú"), "STRASSE ÑANDÚ");
  assert.equal(applyTransform("lower", "ÉCOLE Ñu"), "école ñu");
  assert.equal(applyTransform("title", "the QUICK brown-fox don't élan 42abc"), "The Quick Brown-Fox Don't Élan 42abc");
});

test("whitespace transforms", () => {
  assert.equal(applyTransform("trim", "  a b \n"), "a b");
  assert.equal(applyTransform("collapse", "  a \t  b\n\n\n\n  c   "), "a b\n\nc");
  assert.equal(applyTransform("joinLines", "one\r\n  two \n\nthree"), "one two three");
});

test("JSON transforms only apply to objects and arrays", () => {
  assert.equal(applyTransform("jsonFormat", '{"a":[1,{"b":"ñ"}]}'), '{\n  "a": [\n    1,\n    {\n      "b": "ñ"\n    }\n  ]\n}');
  assert.equal(applyTransform("jsonMinify", '[ 1,\n 2 ]'), "[1,2]");
  assert.equal(applyTransform("jsonFormat", "{a: 1}"), null);
  assert.equal(applyTransform("jsonFormat", "42"), null);
  assert.equal(applyTransform("jsonMinify", '"str"'), null);
  assert.equal(applyTransform("jsonMinify", "[1,2]"), null, "already minified: nothing would change");
});

test("base64 round-trips UTF-8 and rejects non-text", () => {
  assert.equal(applyTransform("base64Encode", "héllo 👋"), "aMOpbGxvIPCfkYs=");
  assert.equal(applyTransform("base64Decode", "aMOpbGxvIPCfkYs="), "héllo 👋");
  assert.equal(applyTransform("base64Decode", "aMOpbGxvIPCfkYs"), "héllo 👋", "padding is optional");
  assert.equal(applyTransform("base64Decode", "aGk_Pz8-"), "hi???>", "url-safe alphabet");
  assert.equal(applyTransform("base64Decode", "test"), null, "decodes to invalid UTF-8");
  assert.equal(applyTransform("base64Decode", "abcd"), null, "decodes to control characters");
  assert.equal(applyTransform("base64Decode", "hello world!"), null);
  assert.equal(applyTransform("base64Decode", "abcde"), null, "impossible length");
  const big = "x".repeat(200_000);
  assert.equal(applyTransform("base64Decode", applyTransform("base64Encode", big) ?? ""), big);
});

test("URL encode/decode", () => {
  assert.equal(applyTransform("urlEncode", "a b&c=ñ/?"), "a%20b%26c%3D%C3%B1%2F%3F");
  assert.equal(applyTransform("urlDecode", "a%20b%26c%3D%C3%B1"), "a b&c=ñ");
  assert.equal(applyTransform("urlDecode", "100%"), null);
  assert.equal(applyTransform("urlDecode", "%E0%A4%A"), null, "malformed sequence");
  assert.equal(applyTransform("urlDecode", "%C3"), null, "invalid UTF-8");
  assert.equal(applyTransform("urlEncode", "plain"), null, "nothing would change");
});

test("applicableTransforms lists only what changes the text", () => {
  assert.deepEqual(
    applicableTransforms("HELLO").map((t) => t.id),
    ["lower", "title", "base64Encode"],
  );
  assert.ok(applicableTransforms('{"a": 1}').some((t) => t.id === "jsonMinify"));
  assert.deepEqual(applicableTransforms("   "), []);
  // Same result as an earlier action: listed once.
  assert.ok(!applicableTransforms("#1E90FF").some((t) => t.id === "title"));
});

test("snippetTokens marks known variables, not escapes or unknown names", () => {
  assert.deepEqual(snippetTokens("Hi {name}, today is {date} at {time}. {{date} {clipboard}"), [
    { text: "Hi {name}, today is ", variable: false },
    { text: "{date}", variable: true },
    { text: " at ", variable: false },
    { text: "{time}", variable: true },
    { text: ". {{date} ", variable: false },
    { text: "{clipboard}", variable: true },
  ]);
  assert.deepEqual(snippetTokens(""), []);
});
