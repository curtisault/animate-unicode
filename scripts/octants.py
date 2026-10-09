"""Prints the octant lookup table for crates/animate-unicode/src/subcell.rs.

    python3 scripts/octants.py path/to/UnicodeData.txt

UnicodeData.txt is Unicode 16's (https://www.unicode.org/Public/16.0.0/ucd/).
An octant cell is 2 × 4 dots, numbered row by row: 1 2 / 3 4 / 5 6 / 7 8, and
a dot mask has bit n-1 for dot n (the `Canvas` convention: bit row * 2 + col).
Unicode 16 gives 230 of the 256 masks a BLOCK OCTANT-<dots> character; the
other 26 already had a glyph (a space, a half or full block, a quadrant, a
quarter-height row or corner, a middle quarter) and get no octant. Those are listed here by name, and
every code point is looked up by its name in the data, never typed in.
"""

import sys

# Masks whose glyph is not a BLOCK OCTANT, by character name.
ELSEWHERE = {
    0x00: "SPACE",
    0xFF: "FULL BLOCK",
    0x0F: "UPPER HALF BLOCK",
    0xF0: "LOWER HALF BLOCK",
    0x55: "LEFT HALF BLOCK",
    0xAA: "RIGHT HALF BLOCK",
    0x05: "QUADRANT UPPER LEFT",
    0x0A: "QUADRANT UPPER RIGHT",
    0x50: "QUADRANT LOWER LEFT",
    0xA0: "QUADRANT LOWER RIGHT",
    0xA5: "QUADRANT UPPER LEFT AND LOWER RIGHT",
    0x5A: "QUADRANT UPPER RIGHT AND LOWER LEFT",
    0x5F: "QUADRANT UPPER LEFT AND UPPER RIGHT AND LOWER LEFT",
    0xAF: "QUADRANT UPPER LEFT AND UPPER RIGHT AND LOWER RIGHT",
    0xF5: "QUADRANT UPPER LEFT AND LOWER LEFT AND LOWER RIGHT",
    0xFA: "QUADRANT UPPER RIGHT AND LOWER LEFT AND LOWER RIGHT",
    0x03: "UPPER ONE QUARTER BLOCK",
    0x3F: "UPPER THREE QUARTERS BLOCK",
    0xC0: "LOWER ONE QUARTER BLOCK",
    0xFC: "LOWER THREE QUARTERS BLOCK",
    0x01: "LEFT HALF UPPER ONE QUARTER BLOCK",
    0x02: "RIGHT HALF UPPER ONE QUARTER BLOCK",
    0x40: "LEFT HALF LOWER ONE QUARTER BLOCK",
    0x80: "RIGHT HALF LOWER ONE QUARTER BLOCK",
    0x14: "MIDDLE LEFT ONE QUARTER BLOCK",
    0x28: "MIDDLE RIGHT ONE QUARTER BLOCK",
}


def main() -> None:
    by_name, table = {}, {}
    with open(sys.argv[1], encoding="utf-8") as f:
        for line in f:
            code, name = line.split(";")[:2]
            cp = int(code, 16)
            by_name[name] = cp
            if name.startswith("BLOCK OCTANT-"):
                mask = sum(1 << (int(d) - 1) for d in name.removeprefix("BLOCK OCTANT-"))
                if mask in table:
                    sys.exit(f"two octants for mask {mask:#04x}")
                table[mask] = cp
    octants = len(table)
    for mask, name in ELSEWHERE.items():
        if mask in table:
            sys.exit(f"{name} and U+{table[mask]:04X} both claim mask {mask:#04x}")
        if name not in by_name:
            sys.exit(f"no character named {name}")
        table[mask] = by_name[name]
    missing = [m for m in range(256) if m not in table]
    if missing:
        sys.exit(f"{octants} octants + {len(ELSEWHERE)} others; no glyph for masks {[hex(m) for m in missing]}")
    print(f"// {octants} BLOCK OCTANT characters and {len(ELSEWHERE)} older glyphs, by dot mask.")
    print("const TABLE: [char; 256] = [")
    for row in range(0, 256, 8):
        print("    " + " ".join(f"'\\u{{{table[m]:04X}}}'," for m in range(row, row + 8)))
    print("];")


if __name__ == "__main__":
    main()
