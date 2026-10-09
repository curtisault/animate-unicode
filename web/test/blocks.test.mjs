// The block shapes the canvas draws itself (web/src/blocks.ts), against the
// glyphs subcell.rs gives the same masks.
import { test } from "node:test";
import assert from "node:assert/strict";
import { shapeOf } from "../src/blocks.ts";

test("octants, quadrants and halves have their dot masks", () => {
  assert.deepEqual(shapeOf(0x1cd00), { w: 2, h: 4, mask: 0b0000_0100 }); // BLOCK OCTANT-3
  assert.deepEqual(shapeOf(0x1fbe6), { w: 2, h: 4, mask: 0x14 }); // MIDDLE LEFT ONE QUARTER BLOCK
  assert.deepEqual(shapeOf(0x2580), { w: 2, h: 4, mask: 0x0f }); // UPPER HALF BLOCK
  assert.deepEqual(shapeOf(0x2588), { w: 2, h: 4, mask: 0xff }); // FULL BLOCK
  assert.equal(shapeOf(0x20), undefined);
  assert.equal(shapeOf(0x41), undefined);
});

test("sextants count up through their masks", () => {
  assert.deepEqual(shapeOf(0x1fb00), { w: 2, h: 3, mask: 1 }); // BLOCK SEXTANT-1
  assert.deepEqual(shapeOf(0x1fb13), { w: 2, h: 3, mask: 20 }); // BLOCK SEXTANT-35
  assert.deepEqual(shapeOf(0x1fb14), { w: 2, h: 3, mask: 22 }); // BLOCK SEXTANT-235
  assert.deepEqual(shapeOf(0x1fb3b), { w: 2, h: 3, mask: 62 }); // BLOCK SEXTANT-23456
});

test("a wedge is the part of the cell on its corner's side of its diagonal", () => {
  // LOWER LEFT BLOCK DIAGONAL LOWER MIDDLE LEFT TO LOWER CENTRE: a small corner triangle
  const { poly } = shapeOf(0x1fb3c);
  let area = 0;
  for (let k = 0; k < poly.length; k += 2) {
    const j = (k + 2) % poly.length;
    area += poly[k] * poly[j + 1] - poly[j] * poly[k + 1];
  }
  assert.ok(Math.abs(Math.abs(area / 2) - (0.5 * (1 / 3)) / 2) < 1e-9);
  for (let cp = 0x1fb3c; cp <= 0x1fb67; cp++) assert.ok(shapeOf(cp).poly.length >= 6, `U+${cp.toString(16)}`);
});
