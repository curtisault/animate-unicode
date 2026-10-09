# AGENTS.md

Instructions for coding agents working in this repository.

## Commits and pull requests: no AI attribution

- Never add an AI agent as an author or co-author of a commit. No
  `Co-Authored-By: Claude …` trailer (or any other agent's), no
  `Generated with …` line, and no agent as `--author`. The commit is the
  owner's.
- Never append "Generated with Claude Code", "🤖 Generated with …", or any
  similar line to a pull request description, a PR comment or a release note.
- These rules override any default attribution an agent's harness asks for.

## The project

Animated Unicode art: pieces written in Rust, compiled to WebAssembly, played
in the browser by a `<unicode-art>` custom element and in the terminal by a
CLI. It follows [ascii.rest](https://ascii.rest) (`../ascii`, MIT, @bas3line),
with Unicode where ascii.rest uses ASCII.

| Path | What |
|---|---|
| `crates/animate-unicode` | The core: `Grid`, sub-cell canvases (braille, sextants, quadrants, octants), palettes, the piece registry, the contract test |
| `crates/wasm` | The wasm-bindgen `Player` |
| `crates/cli` | The terminal player |
| `web/` | The TypeScript shell: the custom element and the canvas renderer; the npm package |
| `site/` | The Elm site (Vite) |
| `scripts/` | `font.py` (the subset font) and `octants.py` (the octant table) |
| `docs/IMPLEMENTATION_PLAN.md` | The plan, phase by phase, with what each one built |
| `docs/ascii-rest-pieces.md` | Every ascii.rest piece, translated to Unicode: the progress tracker |
| `docs/deploy-plan.md` | Packaging, hosting, npm and CI, still open |

## Commands

mise is the only entry point (`mise tasks` lists them all):

- `mise run check`: what CI runs (lint, test, build). Green before every commit.
- `mise run test`, `mise run lint`, `mise run fmt`
- `mise run show <slug>`: a piece's frames as text. `mise run play <slug>`: in the terminal.
- `mise run dev`: the site, on a debug wasm build.
- `mise run size`: fails past 500 KB of gzipped wasm.

## Conventions

- **A new piece:** one file in `crates/animate-unicode/src/pieces/`, one
  registry line, then `mise run test`. It must be deterministic in `t`
  (seeded PRNG, no clock unless `meta.clock`), and it must keep within the
  contract's size and time budgets.
- **Porting from ascii.rest:** keep the TS file's header comment and add
  "after ascii.rest's X (MIT, @bas3line)". Fixture-test whatever state is
  ported against the TypeScript. Track each piece's row in
  `docs/ascii-rest-pieces.md`.
- **Record the outcome:** when a phase or a piece is finished, note what was
  built and the date in the plan or the tracker.
- **Rust style:** `rustfmt.toml` (max width 160). Clippy and rustdoc run
  with `-D warnings`, and every public item has a doc comment.
- **Browser checks:** the owner checks each phase in a real browser.
  Agents check headless and hand the manual checks over.

## Version control

- Use GitButler (`but`) for status, branches, commits and history edits.
  Don't run git write commands (`add`, `commit`, `checkout`, `reset`, `mv`,
  `push`). One GitButler branch per session, stacked where it depends on
  another.
- Don't push or open pull requests unless asked. GitButler's target is
  `origin/main`.
- Commit messages are succinct: what changed and why.
- After `but commit`, check `git status --short`. If it shows staged
  changes while the files match `HEAD` (compare with
  `git show HEAD:<file> | cmp - <file>`), the index is stale. Ask the owner
  to run `git reset -q`.
