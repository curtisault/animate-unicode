// The wasm boundary from JS, as web/src/mount.ts uses it: the manifest and
// the modules it names, Players, the typed array views over wasm memory,
// errors, and free(). Run by `mise run test`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fallback, imports, loadAll, modules, moduleOf, pieces as metas, root } from "./load.mjs";
import { WIDE_TAIL } from "../src/mount.ts";

const loaded = await loadAll();
const slugs = metas.map((m) => m.slug).sort();
/** The module that plays a slug, as the shell finds it. */
const of = (slug) => loaded.get(moduleOf(slug));

/** The frame as text rebuilt from the memory views, the way mount.ts reads them. */
function viewText(p, wasm) {
  const cols = p.cols();
  const cells = new Uint32Array(wasm.memory.buffer, p.cells_ptr(), cols * p.rows());
  const rows = [];
  for (let r = 0; r < p.rows(); r++) {
    const row = cells.subarray(r * cols, (r + 1) * cols).filter((cp) => cp !== WIDE_TAIL);
    rows.push(String.fromCodePoint(...row));
  }
  return rows.join("\n");
}

test("the manifest lists exactly the pieces its modules play", () => {
  assert.ok(slugs.includes("donut") && slugs.includes("braille-wave"));
  assert.deepEqual([...new Set(slugs)], slugs, "a slug listed twice");
  // Every category maps to a module the manifest imports, and every module is some category's.
  assert.deepEqual([...new Set(Object.values(modules))].sort(), Object.keys(imports).sort());
  for (const [name, { mod }] of loaded) {
    const listed = metas.filter((m) => modules[m.category] === name);
    // A module lists its own pieces in the manifest's order, with the same metas, and no others.
    assert.deepEqual(JSON.parse(mod.metas_json()), listed, `module ${name}`);
    assert.deepEqual(JSON.parse(mod.slugs_json()), listed.map((m) => m.slug).sort(), `module ${name}`);
    assert.equal(mod.wide_tail(), WIDE_TAIL, `module ${name}: the wide tail`);
  }
  assert.equal(fallback === null || fallback in imports, true);
  for (const m of metas) {
    assert.equal(typeof m.name, "string");
    assert.ok(Number.isInteger(m.cols) && Number.isInteger(m.rows) && Number.isInteger(m.fps));
    assert.ok(["basic", "extended", "wide"].includes(m.charset), `${m.slug}: charset ${m.charset}`);
  }
});

for (const meta of metas) {
  test(`${meta.slug}: memory views match text() frame after frame`, () => {
    const { mod, wasm } = of(meta.slug);
    const p = new mod.Player(meta.slug, "");
    try {
      assert.equal(p.cols(), meta.cols);
      assert.equal(p.rows(), meta.rows);
      assert.equal(p.fps(), meta.fps);
      assert.deepEqual(JSON.parse(p.meta_json()), meta);
      for (const [t, paper] of [[0, false], [1.3, false], [2.5, true], [7.7, false]]) {
        p.frame(t, paper);
        // Views are made fresh after every frame, as mount.ts does: memory may have grown.
        assert.equal(viewText(p, wasm), p.text(), `t=${t} paper=${paper}`);
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
  const fixture = readFileSync(resolve(root, "crates/unicel/tests/fixtures/donut.txt"), "utf8");
  const p = new (of("donut").mod.Player)("donut", "");
  for (const block of fixture.split("\n\n")) {
    const [head, ...body] = block.split("\n");
    const t = Number(head.slice(2));
    p.frame(t, false);
    assert.equal(p.text(), body.join("\n").replace(/\n+$/, ""), `donut at t=${t}`);
  }
  p.free();
});

test("paper flips a shaded piece", () => {
  const p = new (of("donut").mod.Player)("donut", "");
  p.frame(1, false);
  const ink = p.text();
  p.frame(1, true);
  assert.notEqual(p.text(), ink);
  p.free();
});

test("options reach the piece", () => {
  const { mod } = of("braille-wave");
  const slow = new mod.Player("braille-wave", "");
  const fast = new mod.Player("braille-wave", '{"speed": 3}');
  slow.frame(1, false);
  fast.frame(1, false);
  assert.notEqual(fast.text(), slow.text());
  slow.free();
  fast.free();
});

test("a bad slug or bad options throw a readable error", () => {
  const { mod } = of("donut");
  assert.throws(() => new mod.Player("nope", ""), { message: 'there is no piece named "nope"' });
  assert.throws(() => new mod.Player("donut", "{"), { message: /^options are not a JSON object/ });
  // A module plays only its own category: another category's slug is unknown to it.
  const other = metas.find((m) => moduleOf(m.slug) !== moduleOf("donut"));
  if (other) assert.throws(() => new mod.Player(other.slug, ""), { message: `there is no piece named "${other.slug}"` });
  // The module still works after an error: errors are values, not panics.
  const p = new mod.Player("donut", "");
  p.frame(0, false);
  p.free();
});

test("free() releases the player and its memory", () => {
  const p = new (of("donut").mod.Player)("donut", "");
  p.free();
  assert.throws(() => p.text(), "a freed player cannot be used");
  // Many players made and freed must not grow a module's memory: the grids are given back.
  for (const [name, { mod, wasm }] of loaded) {
    const own = JSON.parse(mod.slugs_json());
    const churn = () => {
      for (let i = 0; i < Math.max(60, 4 * own.length); i++) {
        const q = new mod.Player(own[i % own.length], "");
        q.frame(i / 30, false);
        q.free();
      }
    };
    churn();
    const before = wasm.memory.buffer.byteLength;
    churn();
    churn();
    assert.equal(wasm.memory.buffer.byteLength, before, `module ${name}`);
  }
});

test("a clock piece shows the wall clock set_clock gives it, in local time", () => {
  const p = new (of("calendar").mod.Player)("calendar", "");
  p.frame(0, false);
  assert.match(p.text(), /october 2026/, "untold, it shows the fixed moment");
  // 2027-01-31 23:30 UTC, an hour ahead: already 1 February locally.
  p.set_clock(Date.UTC(2027, 0, 31, 23, 30, 0), 60);
  p.frame(0.5, false);
  assert.match(p.text(), /february 2027/);
  assert.match(p.text(), /monday 1 +00:30:00/);
  p.free();
});

test("set_mono swaps a logo to its one-ink drawing", () => {
  const p = new (of("c").mod.Player)("c", "");
  p.frame(0, false);
  const colour = p.text();
  p.set_mono(true);
  p.frame(0, false);
  const mono = p.text();
  // In colour the C is a white part of the solid hexagon; in one ink it is cut out of it.
  assert.notEqual(mono, colour);
  assert.ok([...mono].filter((c) => c === " ").length > [...colour].filter((c) => c === " ").length);
  p.free();
});

test("a scene's back colours come through backs_ptr, one a cell, in its palette", () => {
  const { mod, wasm } = of("night-coast");
  const p = new mod.Player("night-coast", "");
  p.frame(0, false);
  const n = p.cols() * p.rows();
  const backs = new Uint8Array(wasm.memory.buffer, p.backs_ptr(), n);
  const palette = JSON.parse(p.meta_json()).palette;
  assert.ok(backs.every((b) => b < palette.length), "every cell has a back colour in the palette");
  p.set_mono(true);
  p.frame(0, false);
  assert.ok(new Uint8Array(wasm.memory.buffer, p.backs_ptr(), n).every((b) => b === 255), "none in one ink");
  p.free();
});
