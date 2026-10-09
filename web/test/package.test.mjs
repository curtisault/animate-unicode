// The npm package as it would be published: `npm pack --dry-run` lists the
// tarball, and every file the exports point at, plus the wasm and the font
// they load (and the font's licence), must be in it. (wasm-pack's pkg/.gitignore once kept pkg/ out entirely.) Needs
// web/dist and web/pkg built: `mise run test` builds both first.
import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { debugBuild, root } from "./load.mjs";

const web = new URL("../", import.meta.url);

test("the npm tarball holds every export and the wasm", { skip: debugBuild && "checks the release build in web/pkg" }, () => {
  const [pack] = JSON.parse(execFileSync("npm", ["pack", "--dry-run", "--json", "--ignore-scripts"], { cwd: web, encoding: "utf8" }));
  const files = new Set(pack.files.map((f) => f.path));
  const pkg = JSON.parse(readFileSync(new URL("package.json", web), "utf8"));
  const wanted = Object.values(pkg.exports).flatMap((e) => [e.types, e.default]).map((p) => p.replace(/^\.\//, ""));
  wanted.push("pkg/animate_unicode_bg.wasm", "fonts/animate-unicode-mono.woff2", "fonts/OFL.txt");
  for (const f of wanted) assert.ok(files.has(f), `${f} is missing from the tarball (${pack.files.length} files, root ${root})`);
  assert.ok(![...files].some((f) => f.endsWith(".gitignore")), "a .gitignore made it into the tarball");
});
