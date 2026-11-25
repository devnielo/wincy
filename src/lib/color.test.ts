import { test } from "node:test";
import assert from "node:assert/strict";
import { parseColor, toHex, toHslString, toRgbString } from "./color.ts";

test("formats hsl()", () => {
  assert.equal(toHslString(parseColor("#1E90FF")!), "hsl(210, 100%, 56%)");
  assert.equal(toHslString(parseColor("rgba(255, 0, 0, 0.5)")!), "hsla(0, 100%, 50%, 0.5)");
  assert.equal(toHslString(parseColor("#808080")!), "hsl(0, 0%, 50%)");
  assert.equal(toHslString(parseColor("hsl(160, 84%, 39%)")!), "hsl(160, 84%, 39%)");
});

test("parses hex forms", () => {
  assert.deepEqual(parseColor("#fff"), { r: 255, g: 255, b: 255, a: 1 });
  assert.equal(toHex(parseColor(" #1E90ff ")!), "#1E90FF");
  assert.equal(toHex(parseColor("#0008")!), "#00000088");
  assert.equal(parseColor("#12345"), null);
});

test("parses rgb() in comma and space syntax", () => {
  assert.equal(toRgbString(parseColor("rgb(30, 144, 255)")!), "rgb(30, 144, 255)");
  assert.equal(toRgbString(parseColor("rgba(0,0,0,.5)")!), "rgba(0, 0, 0, 0.5)");
  assert.equal(toRgbString(parseColor("rgb(100% 0% 0% / 25%)")!), "rgba(255, 0, 0, 0.25)");
  assert.equal(parseColor("rgb(1, 2)"), null);
  assert.equal(parseColor("rgb(a, b, c)"), null);
  assert.equal(parseColor("rgb(1e3, 0, 0)"), null);
});

test("parses hsl()", () => {
  assert.equal(toHex(parseColor("hsl(210, 100%, 56%)")!), "#1F8FFF");
  assert.equal(toHex(parseColor("hsla(0deg 100% 50% / 0.5)")!), "#FF000080");
  assert.equal(parseColor("hsl(0, 100, 50)"), null);
});

test("rejects anything that could smuggle CSS", () => {
  assert.equal(parseColor("red; background: url(x)"), null);
  assert.equal(parseColor("rgb(1,2,3) url(x)"), null);
  assert.equal(parseColor("rgb(1,2,3);color:red"), null);
});
