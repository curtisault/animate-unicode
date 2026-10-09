"""Builds web/fonts/animate-unicode-mono.woff2, the font the pieces draw in.

    mise run build:font

It is a subset of Iosevka Fixed Extended (SIL OFL 1.1, Renzhi Li; licence in
web/fonts/OFL.txt): printable ASCII, the few Latin-1 and punctuation glyphs
`Charset::Basic` allows, and every range `Charset::Extended` allows, plus the
octants for later. Every glyph is 0.6 em wide, so at the shell's 1.2 line
height a cell is exactly twice as tall as it is wide, as the pieces assume.

The output is checked in; this only needs running to change the ranges or the
Iosevka version. It downloads the release once (about 100 MB) into
target/font/, checks its SHA-256, subsets, renames the family to
"animate-unicode mono" (a modified version should not carry the original's
name), and checks that the font covers its ranges and that
web/src/element.ts declares the same unicode-range.
"""

import hashlib
import io
import sys
import urllib.request
import zipfile
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

VERSION = "34.9.0"
ASSET = f"PkgTTF-Unhinted-IosevkaFixed-{VERSION}.zip"
URL = f"https://github.com/be5invis/Iosevka/releases/download/v{VERSION}/{ASSET}"
SHA256 = "d89f5705b5be44572b83e3f482ac9575383effedd605e838f919ce91f031f56a"
MEMBER = "IosevkaFixed-Extended.ttf"
FAMILY = "animate-unicode mono"

# (first, last) code points. Keep in step with allowed() in
# crates/animate-unicode/tests/contract.rs; octants are ahead of it.
RANGES = [
    (0x0020, 0x007E),  # printable ASCII
    (0x00B0, 0x00B0),  # °
    (0x00B7, 0x00B7),  # ·
    (0x2022, 0x2022),  # •
    (0x2190, 0x21FF),  # arrows
    (0x2500, 0x25FF),  # box drawing, blocks, geometric shapes (● U+25CF among them)
    (0x2800, 0x28FF),  # braille
    (0x1CD00, 0x1CDE5),  # octants (Unicode 16)
    (0x1FB00, 0x1FBFF),  # symbols for legacy computing: sextants, wedges
]
# Ranges where some code points are unassigned or the font lacks a glyph; the
# browser takes those from the next face in the stack.
PARTIAL = {(0x1FB00, 0x1FBFF)}

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "web" / "fonts" / "animate-unicode-mono.woff2"
ELEMENT = ROOT / "web" / "src" / "element.ts"


def unicode_range() -> str:
    return ",".join(f"U+{a:04X}" if a == b else f"U+{a:04X}-{b:04X}" for a, b in RANGES)


def source() -> bytes:
    cache = ROOT / "target" / "font" / ASSET
    if not cache.exists():
        cache.parent.mkdir(parents=True, exist_ok=True)
        print(f"downloading {URL}")
        with urllib.request.urlopen(URL) as r, open(cache.with_suffix(".part"), "wb") as f:
            while chunk := r.read(1 << 20):
                f.write(chunk)
        cache.with_suffix(".part").rename(cache)
    digest = hashlib.sha256(cache.read_bytes()).hexdigest()
    if digest != SHA256:
        sys.exit(f"{cache}: sha256 {digest}, expected {SHA256}; delete it and run again")
    with zipfile.ZipFile(cache) as z:
        name = next(n for n in z.namelist() if n.rsplit("/", 1)[-1] == MEMBER)
        return z.read(name)


def rename(font: TTFont) -> None:
    names = font["name"]
    keep = {0, 13, 14}  # copyright, licence, licence URL
    names.names = [n for n in names.names if n.nameID in keep]
    for name_id, value in {
        1: FAMILY,
        2: "Regular",
        3: f"{FAMILY} {VERSION}",
        4: FAMILY,
        5: f"Version {VERSION}; subset of Iosevka Fixed Extended",
        6: "animate-unicode-mono",
    }.items():
        names.setName(value, name_id, 3, 1, 0x409)


def main() -> None:
    font = TTFont(io.BytesIO(source()))
    options = subset.Options()
    options.flavor = "woff2"
    options.hinting = False
    options.desubroutinize = True
    options.layout_features = []
    options.name_IDs = ["*"]
    options.notdef_outline = True
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=[c for a, b in RANGES for c in range(a, b + 1)])
    subsetter.subset(font)
    rename(font)

    cmap = font.getBestCmap()
    widths = {font["hmtx"][g][0] for g in cmap.values()}
    if widths != {600}:
        sys.exit(f"glyph widths {sorted(widths)}, expected only 600 (0.6 em)")
    for a, b in RANGES:
        missing = [c for c in range(a, b + 1) if c not in cmap]
        if missing and (a, b) not in PARTIAL:
            sys.exit(f"the font lacks {len(missing)} of U+{a:04X}-{b:04X}, first U+{missing[0]:04X}")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    font.flavor = "woff2"  # options.flavor only applies through subset.save_font
    font.save(OUT)
    css = unicode_range()
    print(f"{OUT.relative_to(ROOT)}: {OUT.stat().st_size} bytes, {len(cmap)} glyphs")
    print(f"unicode-range: {css}")
    if f'UNICODE_RANGE = "{css}"' not in ELEMENT.read_text():
        sys.exit(f'{ELEMENT.relative_to(ROOT)} does not declare UNICODE_RANGE = "{css}"')


if __name__ == "__main__":
    main()
