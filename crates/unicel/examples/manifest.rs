//! The wasm modules' manifest, for `mise run build:wasm`.
//!
//!     cargo run -q --release -p unicel --example manifest -- categories
//!     cargo run -q --release -p unicel --example manifest -- write web/pkg [--one all]
//!
//! `categories` prints the categories that have pieces: one wasm module is
//! built for each, into web/pkg/<category>. `write` then writes
//! web/pkg/modules.js and its types: every piece's meta, which module plays
//! each category, and an import for each module, so the shell loads only the
//! modules a page shows. `--one all` maps every category to the single module
//! web/pkg/all, the debug build with every piece and the debug pieces; that
//! module also plays a slug the manifest does not list.

use std::fmt::Write as _;
use unicel::registry;

fn fail(msg: &str) -> ! {
    eprintln!("{msg}\nusage: manifest categories | manifest write <dir> [--one <module>]");
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let categories: Vec<&str> = registry::categories().iter().filter(|(_, e)| !e.is_empty()).map(|(c, _)| c.slug()).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["categories"] => println!("{}", categories.join(" ")),
        ["write", dir] => write(dir, &categories, None),
        ["write", dir, "--one", one] => write(dir, &categories, Some(one)),
        _ => fail("unknown arguments"),
    }
}

fn write(dir: &str, categories: &[&str], one: Option<&str>) {
    let module = |c: &str| one.unwrap_or(c).to_string();
    let mut names: Vec<String> = categories.iter().map(|c| module(c)).collect();
    names.dedup();
    let pieces = registry::all().map(|e| format!("  {}", e.meta.to_json())).collect::<Vec<_>>().join(",\n");
    let mut js = String::from("// Written by `mise run build:wasm` (crates/unicel/examples/manifest.rs). Do not edit.\n\n");
    let _ = writeln!(js, "/** Every piece's meta, category by category. */\nexport const pieces = [\n{pieces}\n];\n");
    js += "/** The module that plays each category. */\nexport const modules = {\n";
    for c in categories {
        let _ = writeln!(js, "  {c:?}: {:?},", module(c));
    }
    js += "};\n\n/** Each module's bindings, imported on first use. */\nexport const imports = {\n";
    for n in &names {
        let _ = writeln!(js, "  {n:?}: () => import(\"./{n}/unicel.js\"),");
    }
    let fallback = one.map_or("null".to_string(), |o| format!("{o:?}"));
    let _ = writeln!(js, "}};\n\n/** The module asked for a slug that no category lists (the debug pieces), or null. */\nexport const fallback = {fallback};");
    let first = names.first().cloned().unwrap_or_else(|| fail("no category has pieces"));
    let dts = format!(
        "// Written by `mise run build:wasm` (crates/unicel/examples/manifest.rs). Do not edit.\n\
         // Every module has the same bindings; the types come from one of them.\n\n\
         /** A module's bindings: `Player`, `metas_json` and the rest, and `default`, which instantiates it. */\n\
         export type Bindings = typeof import(\"./{first}/unicel.js\");\n\
         /** An instantiated module's exports, `memory` among them. */\n\
         export type InitOutput = import(\"./{first}/unicel.js\").InitOutput;\n\
         /** A piece playing in a module. */\n\
         export type Player = import(\"./{first}/unicel.js\").Player;\n\n\
         /** Every piece's meta, category by category (the shell's `Meta`). */\n\
         export declare const pieces: readonly unknown[];\n\
         /** The module that plays each category. */\n\
         export declare const modules: Readonly<Record<string, string>>;\n\
         /** Each module's bindings, imported on first use. */\n\
         export declare const imports: Readonly<Record<string, () => Promise<Bindings>>>;\n\
         /** The module asked for a slug that no category lists (the debug pieces), or null. */\n\
         export declare const fallback: string | null;\n"
    );
    for (name, text) in [("modules.js", js), ("modules.d.ts", dts)] {
        let path = std::path::Path::new(dir).join(name);
        std::fs::write(&path, text).unwrap_or_else(|e| fail(&format!("could not write {}: {e}", path.display())));
    }
}
