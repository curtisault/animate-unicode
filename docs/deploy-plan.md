# animate-unicode: deploy plan

Phase 8 of [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md), moved here on
2026-10-08 to be done later: packaging, the site's hosting, npm, CI and the
README. Phases 0–7 are built. This document starts with what they left in
place and what is still open, then the tasks as the plan set them, with notes
from the work since.

- [Where things stand](#where-things-stand)
- [Decisions to make first](#decisions-to-make-first)
- [Tasks](#tasks)
- [Acceptance](#acceptance)

## Where things stand

Already in the repository:

| What | Where | Notes |
|---|---|---|
| SPA fallback | `site/public/_redirects` | `/* /index.html 200`; files that exist (test.html, assets) are served as they are. Cloudflare and Netlify read it. |
| Long cache on hashed assets | `site/public/_headers` | `/assets/*` gets a year, immutable. Vite's preview server ignores the file, so it is untested until a host serves it. |
| npm package shape | `web/package.json` | `files`: `dist`, `pkg`, `fonts`. `prepublishOnly` runs `mise run build:web`, which builds the wasm first. |
| Tarball check | `web/test/package.test.mjs` | Dry-runs `npm pack` and fails if an export target, the wasm, the font or its licence is missing. |
| wasm-pack output fit to ship | `build:wasm` in `mise.toml` | `--no-pack`, and the `*` .gitignore wasm-pack writes is deleted (npm honoured it and left the wasm out). |
| Size gate | `mise run size` | Fails past 500 KB gzipped (`WASM_GZ_LIMIT` overrides) and writes the line to the GitHub step summary. 64.6 KB gz with 11 pieces. |
| CI workflow | `.github/workflows/ci.yml` | `npm ci`, then `mise run lint`, `test`, `build`, `size`. `test` already covers the Node wasm tests (release and debug builds) and elm-test, so the plan's CI additions are in. |
| Font | `web/fonts/` | Found next to the shell with `new URL("../fonts/…", import.meta.url)`, so bundlers copy it. Rebuilt only by `mise run build:font`, never in CI. |

Still open:

1. **A green CI run.** The repository is
   [curtisault/animate-unicode](https://github.com/curtisault/animate-unicode)
   (public), and `main` was pushed on 2026-10-08. The first run
   ([37878821812](https://github.com/curtisault/animate-unicode/actions/runs/37878821812))
   installed every tool, wasm32 target included (risk 7 answered), and
   built the wasm. It then failed at `cargo fmt`: mise's fresh rust install
   had no rustfmt or clippy. `mise.toml` now asks for both components. The
   test, build and size steps have not run on a runner yet.
2. **The name** (plan, risk 8). `animate-unicode` is now the repository's
   name. It is still a placeholder for the crate, the npm package and the tag.
   On 2026-10-08 `animate-unicode`, `unicode-art` and `unicode-art-element`
   were all free on npm.
3. **The rest of the first CI run**: the npm backend for elm-test (risk 6),
   uv for the font script (not run in CI), and Node's tests against both wasm
   builds.
4. **Timing budgets in CI** may flake on slow runners (risk 9): raise the
   worst-frame budget through an environment variable there rather than
   loosening it locally.

## Decisions to make first

- **The name**, before anything is published: npm package, crate names, the
  `<unicode-art>` tag, the site's domain.
- **The host**: the plan assumed Cloudflare Workers static assets, as
  ascii.rest uses (`../ascii/site/wrangler.jsonc`). `_redirects` and `_headers`
  also suit Netlify.
- ~~**Visibility** of the GitHub repository.~~ Public, decided 2026-10-08.

## Tasks

1. **Site hosting.** Cloudflare Workers static assets as ascii.rest does
   (`../ascii/site/wrangler.jsonc`, `_headers`, `_redirects`). Needs: SPA
   fallback to `index.html` (done, `_redirects`), `Content-Type:
   application/wasm` (Workers sets it), long cache on hashed assets (done,
   `_headers`), and CORS open on `/element.js`, the wasm **and the font** if
   the tag is to be used from other sites: a cross-origin `@font-face` load
   needs `Access-Control-Allow-Origin`, or the browser refuses the font and
   pieces fall back to the system face.
2. **Serving the library from the site root**
   (`<script type="module" src="https://<site>/element.js">`): a Vite `lib`
   build or a second esbuild step that emits `element.js` and the wasm at
   stable paths (not hashed). ascii.rest copies `dist/` into the site. Keep
   the font where the module looks for it: `element.ts` resolves
   `../fonts/animate-unicode-mono.woff2` against its own URL, so a stable
   `element.js` at the root needs the font at `/fonts/…` relative to it, or
   the bundler's rewrite of that `new URL(…)`. The stable files get a short
   cache, not the year `_headers` gives `/assets/*`.
3. **npm publish** of `web/` under the chosen name, with `pkg/` and `fonts/`
   included (the tarball test holds this), and a `release.yml` on tags.
   `prepublishOnly` needs mise on the publishing machine or runner.
4. **CI.** `ci.yml` exists and already runs the size gate, the Node tests and
   elm-test through `mise run test`. Left: a green run (see Still open, 1).
   Caching: `jdx/mise-action` caches tools, `Swatinem/rust-cache` the
   target dir; `actions/setup-node` is unnecessary since mise provides node.
5. **README.** Install, HTML usage, a pieces table generated from
   `metas_json()` by a script (as `../ascii/scripts/readme.ts` does), and
   contributing (a short CONTRIBUTING.md adapted from ascii.rest's).

## Acceptance

A tagged release publishes to npm and deploys the site; a plain HTML file on
another origin shows a piece with two lines, drawn in the bundled font.
