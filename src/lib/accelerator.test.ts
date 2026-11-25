import { test } from "node:test";
import assert from "node:assert/strict";
import { acceleratorFromEvent, type KeyLike } from "./accelerator.ts";

const ev = (code: string, mods: Partial<KeyLike> = {}): KeyLike => ({
  code,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  metaKey: false,
  ...mods,
});

test("builds accelerators in canonical modifier order", () => {
  assert.equal(acceleratorFromEvent(ev("KeyV", { altKey: true })), "Alt+V");
  assert.equal(acceleratorFromEvent(ev("Space", { shiftKey: true, ctrlKey: true })), "Ctrl+Shift+Space");
  assert.equal(acceleratorFromEvent(ev("Digit1", { metaKey: true, altKey: true })), "Alt+Super+1");
  assert.equal(acceleratorFromEvent(ev("ArrowUp", { ctrlKey: true })), "Ctrl+Up");
});

test("allows bare function keys", () => {
  assert.equal(acceleratorFromEvent(ev("F9")), "F9");
  assert.equal(acceleratorFromEvent(ev("F24", { shiftKey: true })), "Shift+F24");
});

test("rejects combos that would break editing or the palette's Alt+N keys", () => {
  for (const code of ["KeyC", "KeyV", "KeyX", "KeyZ", "KeyY", "KeyA"]) {
    assert.equal(acceleratorFromEvent(ev(code, { ctrlKey: true })), null, code);
    assert.equal(acceleratorFromEvent(ev(code, { ctrlKey: true, shiftKey: true })), null, code);
  }
  assert.equal(acceleratorFromEvent(ev("Digit3", { altKey: true })), null);
  assert.equal(acceleratorFromEvent(ev("KeyV", { ctrlKey: true, altKey: true })), "Ctrl+Alt+V");
  assert.equal(acceleratorFromEvent(ev("Digit3", { altKey: true, shiftKey: true })), "Alt+Shift+3");
  assert.equal(acceleratorFromEvent(ev("Digit0", { altKey: true })), "Alt+0");
});

test("rejects modifier-only, unmodified and shift-only combos", () => {
  assert.equal(acceleratorFromEvent(ev("AltLeft", { altKey: true })), null);
  assert.equal(acceleratorFromEvent(ev("KeyA")), null);
  assert.equal(acceleratorFromEvent(ev("KeyA", { shiftKey: true })), null);
  assert.equal(acceleratorFromEvent(ev("IntlBackslash", { ctrlKey: true })), null);
});
