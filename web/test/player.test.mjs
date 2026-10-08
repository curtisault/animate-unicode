// The wasm boundary from JS, as web/src/mount.ts uses it: Players, the typed
// array views over wasm memory, errors, and free(). Run by `mise run test`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { load, root } from "./load.mjs";
import { WIDE_TAIL } from "../src/mount.ts";

const { mod, wasm } = await load();
const metas = JSON.parse(mod.metas_json());
const slugs = JSON.parse(mod.slugs_json());

/** The frame as text rebuilt from the memory views, the way mount.ts reads them. */
function viewText(p) {
  const cols = p.cols();
  const cells = new Uint32Array(wasm.memory.buffer, p.cells_ptr(), cols * p.rows());
  const rows = [];
  for (let r = 0; r < p.rows(); r++) {
    const row = cells.subarray(r * cols, (r + 1) * cols).filter((cp) => cp !== WIDE_TAIL);
    rows.push(String.fromCodePoint(...row));
  }
  return rows.join("\n");
}

test("slugs and metas list the same pieces", () => {
  assert.deepEqual(slugs, [...slugs].sort());
  assert.ok(slugs.includes("donut") && slugs.includes("braille-wave"));
  assert.deepEqual(metas.map((m) => m.slug).sort(), slugs);
  for (const m of metas) {
    assert.equal(typeof m.name, "string");
    assert.ok(Number.isInteger(m.cols) && Number.isInteger(m.rows) && Number.isInteger(m.fps));
    assert.ok(["basic", "extended", "wide"].includes(m.charset), `${m.slug}: charset ${m.charset}`);
  }
});

test("the wide tail is the same in Rust and in the shell", () => {
  assert.equal(mod.wide_tail(), WIDE_TAIL);
});

for (const meta of metas) {
  test(`${meta.slug}: memory views match text() frame after frame`, () => {
    const p = new mod.Player(meta.slug, "");
    try {
      assert.equal(p.cols(), meta.cols);
      assert.equal(p.rows(), meta.rows);
      assert.equal(p.fps(), meta.fps);
      assert.deepEqual(JSON.parse(p.meta_json()), meta);
      for (const [t, paper] of [[0, false], [1.3, false], [2.5, true], [7.7, false]]) {
        p.frame(t, paper);
        // Views are made fresh after every frame, as mount.ts does: memory may have grown.
        assert.equal(viewText(p), p.text(), `t=${t} paper=${paper}`);
        const colors = new Uint8Array(wasm.memory.buffer, p.colors_ptr(), meta.cols * meta.rows);
        const limit = meta.palette ? meta.palette.length : 1;
        assert.ok(colors.every((c) => c < limit), `t=${t}: a colour index past the palette`);
      }
    } finally {
      p.free();
    }
  });
}

test("the wasm donut draws the same frames as the native one", () => {
  // The fixture is ascii.rest's TS donut, which the native port matches in
  // tests/contract.rs; matching it here shows wasm floats agree too.
  const fixture = readFileSync(resolve(root, "crates/animate-unicode/tests/fixtures/donut.txt"), "utf8");
  const p = new mod.Player("donut", "");
  for (const block of fixture.split("\n\n")) {
    const [head, ...body] = block.split("\n");
    const t = Number(head.slice(2));
    p.frame(t, false);
    assert.equal(p.text(), body.join("\n").replace(/\n+$/, ""), `donut at t=${t}`);
  }
  p.free();
});

test("paper flips a shaded piece", () => {
  const p = new mod.Player("donut", "");
  p.frame(1, false);
  const ink = p.text();
  p.frame(1, true);
  assert.notEqual(p.text(), ink);
  p.free();
});

test("options reach the piece", () => {
  const slow = new mod.Player("braille-wave", "");
  const fast = new mod.Player("braille-wave", '{"speed": 3}');
  slow.frame(1, false);
  fast.frame(1, false);
  assert.notEqual(fast.text(), slow.text());
  slow.free();
  fast.free();
});

test("a bad slug or bad options throw a readable error", () => {
  assert.throws(() => new mod.Player("nope", ""), { message: 'there is no piece named "nope"' });
  assert.throws(() => new mod.Player("donut", "{"), { message: /^options are not a JSON object/ });
  // The module still works after an error: errors are values, not panics.
  const p = new mod.Player("donut", "");
  p.frame(0, false);
  p.free();
});

test("free() releases the player and its memory", () => {
  const p = new mod.Player("donut", "");
  p.free();
  assert.throws(() => p.text(), "a freed player cannot be used");
  // Many players made and freed must not grow memory: the grids are given back.
  const churn = () => {
    for (let i = 0; i < 500; i++) {
      const q = new mod.Player(slugs[i % slugs.length], "");
      q.frame(i / 30, false);
      q.free();
    }
  };
  churn();
  const before = wasm.memory.buffer.byteLength;
  churn();
  churn();
  assert.equal(wasm.memory.buffer.byteLength, before);
});
