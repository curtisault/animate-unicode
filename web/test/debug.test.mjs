// The debug pieces (crates/wasm/src/debug.rs). What a panicking piece does,
// as documented under "Panics" in crates/wasm/src/lib.rs: a debug build says
// what and where, the broken player stops, and the rest of the instance keeps
// working. How wide glyphs reach JS. And that a release build has none of it.
// Its own file, so a panic never shares an instance with the other tests.
// `mise run test` runs it against both builds (WASM_DEBUG=1 for the debug one).
import { test } from "node:test";
import assert from "node:assert/strict";
import { debugBuild, load } from "./load.mjs";
import { WIDE_TAIL } from "../src/mount.ts";

const { mod, wasm } = await load();

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
    assert.match(text, /panicked at crates[\\/]wasm[\\/]src[\\/]debug\.rs:\d+:\d+/);
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

  test("wide glyphs are one cell and a tail; text() drops the tails", () => {
    const p = new mod.Player("debug-wide", "");
    const meta = JSON.parse(p.meta_json());
    assert.equal(meta.charset, "wide");
    const at = (t) => {
      p.frame(t, false);
      return Array.from(new Uint32Array(wasm.memory.buffer, p.cells_ptr(), meta.cols * meta.rows));
    };
    const cells = at(0);
    const row = (r) => cells.slice(r * meta.cols, (r + 1) * meta.cols);
    assert.deepEqual(row(0).slice(0, 5), [0x61, 0x6f22, WIDE_TAIL, 0x5b57, WIDE_TAIL]); // a 漢 _ 字 _
    const lines = p.text().split("\n");
    assert.equal(lines[0], "a漢字b·c中d·e·");
    assert.equal(lines[2], "0123456789abcd");
    // Every row is cols cells: a wide glyph's two cells become one char of text.
    for (let r = 0; r < meta.rows; r++) {
      const tails = row(r).filter((c) => c === WIDE_TAIL).length;
      assert.equal([...lines[r]].length, meta.cols - tails, `row ${r}`);
    }
    // The fire steps along its row, and its tail goes with it.
    const fire = (cs) => cs.slice(meta.cols, 2 * meta.cols).indexOf(0x1f525);
    const x0 = fire(at(0)), x1 = fire(at(1.5));
    assert.notEqual(x0, x1);
    assert.equal(at(1.5)[meta.cols + x1 + 1], WIDE_TAIL);
    p.free();
  });

  test("the debug pieces are never listed", () => {
    const slugs = JSON.parse(mod.slugs_json());
    assert.ok(!slugs.includes("debug-panic") && !slugs.includes("debug-wide"));
  });
} else {
  test("a release build has no debug pieces or start hook", () => {
    for (const slug of ["debug-panic", "debug-wide"]) {
      assert.throws(() => new mod.Player(slug, ""), { message: `there is no piece named "${slug}"` });
    }
    assert.equal(mod.start, undefined);
  });
}
