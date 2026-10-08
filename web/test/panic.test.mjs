// What a panicking piece does, as documented under "Panics" in
// crates/wasm/src/lib.rs. A debug build says what and where, the broken player
// stops, and the rest of the instance keeps working; a release build has none
// of the debug hooks. Its own file, so a panic never shares an instance with
// the other tests. `mise run test` runs it against both builds (WASM_DEBUG=1
// for the debug one).
import { test } from "node:test";
import assert from "node:assert/strict";
import { debugBuild, load } from "./load.mjs";

const { mod } = await load();

if (debugBuild) {
  test("a panicking piece reports itself, stops, and leaves the rest running", () => {
    const logged = [];
    const error = console.error;
    console.error = (...args) => logged.push(args.join(" "));
    const healthy = new mod.Player("donut", "");
    const broken = new mod.Player("debug-panic", "");
    try {
      healthy.frame(0, false);
      broken.frame(0.5, false); // fine for its first second
      assert.equal(broken.text(), "fine\n....");
      assert.throws(() => broken.frame(1, false), WebAssembly.RuntimeError);
    } finally {
      console.error = error;
    }
    const text = logged.join("\n");
    assert.match(text, /panicked at crates[\\/]wasm[\\/]src[\\/]lib\.rs:\d+:\d+/);
    assert.match(text, /index out of bounds: the len is 8 but the index is 8/);

    // The broken player stays borrowed by the trapped call, and cannot even be freed.
    assert.throws(() => broken.frame(1.1, false), { message: /recursive use of an object/ });
    assert.throws(() => broken.free(), { message: /while it was borrowed/ });
    // Everything else carries on.
    const before = healthy.text();
    healthy.frame(1, false);
    assert.notEqual(healthy.text(), before);
    const fresh = new mod.Player("braille-wave", "");
    fresh.frame(1, false);
    assert.ok(fresh.text().trim().length > 0);
    fresh.free();
    healthy.free();
  });

  test("the debug piece is never listed", () => {
    assert.ok(!JSON.parse(mod.slugs_json()).includes("debug-panic"));
  });
} else {
  test("a release build has no debug piece or start hook", () => {
    assert.throws(() => new mod.Player("debug-panic", ""), { message: 'there is no piece named "debug-panic"' });
    assert.equal(mod.start, undefined);
  });
}
