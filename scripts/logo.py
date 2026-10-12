"""Draws the logos, companies and distros as octant art.

    mise run build:logos            # every logo
    mise run build:logos c rust     # just these

Each logo's SVG (devicon's or Simple Icons', pinned below and checked against
its SHA-256) is rendered with resvg, fitted into its frame (two columns and a
row of margin), and sampled at SS × SS points a dot. Each cell then takes
the octant (U+1CD00, with the blocks and quadrants that stand in for some) or
the wedge (U+1FB3C–1FB67) that covers its samples best, and the colour most of
its samples have: the nearest of the logo's colours, which are the first sixth
of the piece's `META.palette` (the colours as drawn on a light page). Where
the logo fills a cell and two colours meet in it, the cell is the lesser
colour's octant or wedge over the greater as its back colour. In one ink
white parts are holes, and a logo with a plain variant (c-plain.svg) is drawn
from that instead, as in ascii.rest.

The output, crates/unicel/src/pieces/<category>/art/<slug>.txt, is
checked in: the colour drawing, the colour of each cell and the colour behind
it (base 36, a space for none), and the one-ink drawing, `rows` lines each. src/logo.rs reads it. The SVGs are
downloaded once into target/logos/.
"""

import hashlib
import importlib.util
import io
import re
import sys
import unicodedata
import urllib.request
from pathlib import Path

import numpy as np
import resvg_py
from PIL import Image, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
PIECES = ROOT / "crates/unicel/src/pieces"
CACHE = ROOT / "target/logos"
DEVICON = "https://cdn.jsdelivr.net/gh/devicons/devicon@v{version}/icons/{dir}/{name}.svg"
SIMPLE = "https://cdn.jsdelivr.net/npm/simple-icons@16.34.0/icons/{name}.svg"
SS = 8  # samples a dot, each way
WEDGE = 0.7  # a wedge must miss less than this share of what the best octant misses
LIT = 0.4  # the share of a dot the logo must cover to light it: under a half, so strokes a dot thin hold together
CONTRAST = 0.3  # parts further apart than this in lightness (0 to 1) don't share a cell
WHITE = 0xF0  # a colour lighter than this in every channel is a hole in one ink

# slug: (category, colour drawing, one-ink drawing or None for the same). "d:"
# is devicon 2.16.0 ("d@2.17.0:" a later one, for logos it added; 2.17.0
# redrew C), "s:" Simple Icons (one colour, drawn in black).
LOGOS = {
    "c": ("logos", "d:c-original", "d:c-plain"),
    "clojure": ("logos", "d:clojure-original", None),
    "cpp": ("logos", "d:cplusplus-original", "d:cplusplus-plain"),
    "csharp": ("logos", "d:csharp-original", "d:csharp-plain"),
    "css": ("logos", "d:css3-original", "d:css3-plain"),
    "dart": ("logos", "d:dart-original", "d:dart-plain"),
    "elixir": ("logos", "d:elixir-original", None),
    "elm": ("logos", "d:elm-original", None),
    "erlang": ("logos", "d:erlang-original", "d:erlang-plain"),
    "go": ("logos", "d:go-original-wordmark", None),
    "haskell": ("logos", "d:haskell-original", "d:haskell-plain"),
    "html": ("logos", "d:html5-original", "d:html5-plain"),
    "java": ("logos", "d:java-original", "d:java-plain"),
    "javascript": ("logos", "d:javascript-original", "d:javascript-plain"),
    "julia": ("logos", "d:julia-original", "d:julia-plain"),
    "kotlin": ("logos", "d:kotlin-original", "d:kotlin-plain"),
    "lua": ("logos", "d:lua-original", "d:lua-plain"),
    "ocaml": ("logos", "d:ocaml-original", "d:ocaml-plain"),
    "perl": ("logos", "d:perl-original", "d:perl-plain"),
    "php": ("logos", "d:php-original", "d:php-plain"),
    "python": ("logos", "d:python-original", "d:python-plain"),
    "r": ("logos", "d:r-original", "d:r-plain"),
    "ruby": ("logos", "d:ruby-original", "d:ruby-plain"),
    "rust": ("logos", "d:rust-original", None),
    "scala": ("logos", "d:scala-original", "d:scala-plain"),
    "swift": ("logos", "d:swift-original", "d:swift-plain"),
    "typescript": ("logos", "d:typescript-original", "d:typescript-plain"),
    "zig": ("logos", "d:zig-original", None),
    "apple": ("companies", "s:apple", None),
    "cloudflare": ("companies", "s:cloudflare", None),
    "coderabbit": ("companies", "s:coderabbit", None),
    "helium": ("companies", "s:heliumbrowser", None),
    "mintlify": ("companies", "s:mintlify", None),
    "planetscale": ("companies", "s:planetscale", None),
    "playstation": ("companies", "s:playstation", None),
    "supabase": ("companies", "s:supabase", None),
    "vercel": ("companies", "s:vercel", None),
    "almalinux": ("distros", "d@2.17.0:almalinux-original", "d@2.17.0:almalinux-plain"),
    "alpine-linux": ("distros", "s:alpinelinux", None),
    "arch-linux": ("distros", "d:archlinux-original", None),
    "centos": ("distros", "d:centos-original", "d:centos-plain"),
    "debian": ("distros", "d:debian-original", "d:debian-plain"),
    "deepin": ("distros", "s:deepin", None),
    "elementary-os": ("distros", "s:elementary", None),
    "endeavouros": ("distros", "s:endeavouros", None),
    "fedora": ("distros", "d:fedora-original", "d:fedora-plain"),
    "gentoo": ("distros", "d:gentoo-plain", None),
    "kali-linux": ("distros", "d@2.17.0:kalilinux-original", None),
    "linux-mint": ("distros", "d@2.17.0:linuxmint-original", "s:linuxmint"),
    "manjaro": ("distros", "s:manjaro", None),
    "nixos": ("distros", "d:nixos-original", "d:nixos-plain"),
    "omarchy": ("distros", "s:omarchy", None),
    "opensuse": ("distros", "d:opensuse-original", None),
    "pop-os": ("distros", "s:popos", None),
    "red-hat": ("distros", "d:redhat-original", "d:redhat-plain"),
    "rocky-linux": ("distros", "d:rockylinux-original", None),
    "tux": ("distros", "d:linux-original", "d:linux-plain"),
    "ubuntu": ("distros", "s:ubuntu", None),
    "void-linux": ("distros", "s:voidlinux", None),
    "zorin-os": ("distros", "s:zorin", None),
}

# Logos whose parts differ only in colour, so each part is cut back from its
# neighbours by half a dot, leaving gaps that read in one ink.
GAPS = {"elm"}

# SHA-256 of each source, as first downloaded; `--pin` prints this table afresh.
SHA256 = {
    "d:archlinux-original": "f979ac525905449f58bdabcc37b5d6d48670ceb02c6ce4d778fd71502ea8982e",
    "d:c-original": "7f383aff199491bfc4d45ba3d4f0486cd1ece307216ace19bf205a8783ae68fd",
    "d:c-plain": "0d4699969cdd0e950d0dd928d9de71a68cf3ca213008ea5f91067b33c6040380",
    "d:centos-original": "b54c6ad8e100208fd0ade26633abd996c9d979b19c13c1b91caf48b49ed8c8d2",
    "d:centos-plain": "d5b5b19dc21f1ac695254841a2fc902462b105f411bfdd252ad0835d5caa5863",
    "d:clojure-original": "405b1b05bbbdd7ddfdbe331fec17360de2b6b482b7f67aa66540fa12e6e73e25",
    "d:cplusplus-original": "7ff8253551235e3a6b002a7c2bd6e3190d85113a64fcb40f2254473caeb025d4",
    "d:cplusplus-plain": "0277f8799d85db8f9b838cc6f86656f9fd92eb01191b46b9b777dd22827e8a4c",
    "d:csharp-original": "7093478d0888e22a224b6fa832da6c9ffd14c433de8bd14aab0c1cb50925f3ee",
    "d:csharp-plain": "ccb93dc80b4fbd7363158d520c948ba798f843eb77ece519d06ae1b92058a8b3",
    "d:css3-original": "36b7d94b657d571d3f94042acbf6a4c86a5301a222f83f4b4583ad2acf6e297d",
    "d:css3-plain": "8a9931190c761e07ff9ed032cfa8cf5483e47455ef930da6096f5765bb48e515",
    "d:dart-original": "8dec3d6f6d92d5c8db294baf7ea89cbad5c3dd0f7fde5ca972ffc6cebf9b2d40",
    "d:dart-plain": "bcbd8ce617b6edb9307aa549369ea9a181f558d984c8c6d2f2756664a0749655",
    "d:debian-original": "09b14c9813a322b58b11aa5390fb1a2b4e9bfd02868321482f1a19534a66ec54",
    "d:debian-plain": "832341037e209f9c33198d1175fb0d4e68d48c5bbc2ae6c8dd47d64a62a0e145",
    "d:elixir-original": "17c37b03035f1704f90631d7c945b45621813f395cef4487b7fe40ad5c47be34",
    "d:elm-original": "8ad7eb343a9764e62f073b8f8c8e3d7b003688897ccbd69429bfe67e9efef456",
    "d:erlang-original": "b605010afdc6ac75a2c53ebbf079bbcc47948133fcb1765f8b5eec8494a6d9c0",
    "d:erlang-plain": "1fc96ca258c10f6b2026fce1c36b4ec4a410958bde1aeda7e4867521164004e7",
    "d:fedora-original": "a82984dbe775e1c973ca5d0e78c908ed0730ee6c4deab903b630721644d109d6",
    "d:fedora-plain": "db78228109ec2cc6df4e78a4f457efb3c529d224c1ddca0e61e70d673706b7ed",
    "d:gentoo-plain": "2e71e2a4792a94e052b1d4551b0e1ec925e6edc694224c86ea4b00d30c84d49d",
    "d:go-original-wordmark": "d0400ee3d76ee43fa45e84a3b38b5710a9674969eb71af9d5bb59ed7916a3225",
    "d:haskell-original": "70aee9daacf630e53b54c3e3049d636e69c006d6eb5669c535a221d4fc3f9871",
    "d:haskell-plain": "cfcfa8aef61ace68353c6482b6cb8160048c438124e316766da9ab0a8b71c6a3",
    "d:html5-original": "34826e5b3315daadf4fa15f723a3c1d5ba4a89277bfd94e22ac4d7d3d54338c5",
    "d:html5-plain": "83f6ef40645815e902ad922303e47efa9dd1a2e3e30ee8832d21810307cbd525",
    "d:java-original": "7582e518a9c02425f97155e5a3bd39d1a3a7d421b78caf9c8df7443dad3edc5d",
    "d:java-plain": "7b05ebcc538e747d077ed3c3a2491122923d3ec755cccacb670be9372e4bbbc5",
    "d:javascript-original": "0656ff65fc8eeacda5c78d7f9ffe91ec1eb919db64f56e0b7dcd460af4bbd36c",
    "d:javascript-plain": "7aeba12131be6a9c4bf781171d850939c0b1356d63b42f3d4a3b43e187f94e46",
    "d:julia-original": "7738541f247986f495c154b385098b8a4db9b22963912050a81d4055e2437085",
    "d:julia-plain": "70494100e78e3129053fee43796e15d7043013bcea252058f710ada35a098914",
    "d:kotlin-original": "5053c2e1288582d1ac2cfc1e807eabb7f6059b1aa2f7d04e5cf00063682a2cbe",
    "d:kotlin-plain": "2ee1a71cacd67b1f6b6c5c319cac744c1597b7a5ea2afb7f82c350b9b282504a",
    "d:linux-original": "cf50ae4a263c6142963fad3561f7cb0311d0d41cc6189eadfb34f81a8a36acb0",
    "d:linux-plain": "f262932ac85eed81a79b47184b52210c83d6edb09408f50245ef285435f86f8b",
    "d:lua-original": "7bd82efe74ca5dc3741b8cdcf6c373c96123c638e79c42d0806b15c47e2fccb8",
    "d:lua-plain": "d04a08d8030efb6978df0e516a1247bb3f2092729a2131a90360e5646e99676f",
    "d:nixos-original": "bd6a5dc1211d3356964b79a0fa318d22753576a2f64a309930ee1fa259067cde",
    "d:nixos-plain": "f965fcf8a5961737f44e4aedab25ca6a7816baf847dc2a83aed173c141d86683",
    "d:ocaml-original": "5eff21a93a1cd31f981b6e4816bcb7fc4273e909c389585290d9a1fa097d7d24",
    "d:ocaml-plain": "ea713dcadb5fd591c304079382746d762427981ff80cd8e2f5a83b44a9cf5efa",
    "d:opensuse-original": "f23084ff594f24042a64d096f255e4ce865695c713c23ea64ff9d4454cc0dd9c",
    "d:perl-original": "f6cd4249349b4681ba273699aa31cf4df43f8c63dafdc640803b91cf239b8c8b",
    "d:perl-plain": "4e9ddc7ee24e7bc2bac8be9b3a13a43fad78f046df7b1adda19ca8444de9a843",
    "d:php-original": "f22619a19c99a123e4678d71826f65f223721a739631dff46de39827c9ee0633",
    "d:php-plain": "4af2f7e8e00d9285e6fe2427c0bf4ec6e0b652390110cab6e7e931c0ee20b62c",
    "d:python-original": "71493b4a732f0532b3a03a7a95da3ee11a65766367c8303b901ffbe7bf91a3a2",
    "d:python-plain": "5b2abd125bf9cb2ac6b0b9f0cdbcc9c8005886fa974d26f2b3438b831b6d8365",
    "d:r-original": "9d135e1a56f58f89203afc9253f1db91901e19522f92dfc10575e3312336e61b",
    "d:r-plain": "d949dce55eac56ac17f1403559323453244ef780f41fdf1fdad466f457287e07",
    "d:redhat-original": "c7ec409610e7c222191166a5fe7bd52b57e6d057cb8e51ee44d12a2743401e21",
    "d:redhat-plain": "27f2014b3ced7471cd97f424d244a15f9cabb76c4f749cfbdc47de06eda387d3",
    "d:rockylinux-original": "748f3c1ab46ef03a01b9f315fee680e660366e4d7eaec4dc54c215187b208e13",
    "d:ruby-original": "74ead50b8f0f31e1b34618088bc0b0e16c04f1d4539039ef9358717d8505a63c",
    "d:ruby-plain": "5d68fd019984d3525ed667944b14efa8a37359cd21dcb8bf5e6969abdd48dd5f",
    "d:rust-original": "16c6c6a98e990944fc5e1cc64cc8fc7b8be3a662ec4ee7951b7995d2e52f5e12",
    "d:scala-original": "2565b6286bf30ab28c7d6a988ab7a76c5ce28182b80634e2a2ae21e29b4ad7b9",
    "d:scala-plain": "0dc8eee576d3b25b9a3cc0f87b582c813acfda60b59bb35fd25801a3820fc52d",
    "d:swift-original": "46b6e4abd3ab12c589b8e276295f9439e2869bc4788d918efbc6e76147e8a06e",
    "d:swift-plain": "b653c8a3255d34cea4550e090cc928d3b095890093eea522f7f2917f21ea7f8b",
    "d:typescript-original": "c9191199f4049920c2fc19035b8a6664f37f4689fcd9e8434e786097e78863f0",
    "d:typescript-plain": "dbdda8a49cf4f4eac68ebea33bd532644d1dfb94142a6ed3f2c7a619605b8116",
    "d:zig-original": "f4af51779b6d6f6a0de1172f11cb19803b42b6c4a02eb6e43ed97d2dd640323f",
    "d@2.17.0:almalinux-original": "bc82554bde812764d0c9a919ed57d1848111cbba48a65d49fb5aa4b82d82f99b",
    "d@2.17.0:almalinux-plain": "80644a958cbe3f0b87e917b99c45ad36c87ed8913161b278cc9a1301c6612c88",
    "d@2.17.0:kalilinux-original": "ba8eb6ae69a478c8f9bb0d8e2f2a1e1edfe37d29c42f68b737e76d8417c27237",
    "d@2.17.0:linuxmint-original": "789960776b9a7b16e6af8cdbe3a2263a2c32d04a6227bd20c49bea9662a12459",
    "s:alpinelinux": "d37d0a7873c65174308d63929f09ab93c3aaa7f5c5df79cdec4be532e015fc98",
    "s:apple": "2a1509dccd25e6d2bc7a11a8e52941077e1a48555e192ce638699b9f083c2a7c",
    "s:cloudflare": "408096d99907fd99e0863cb5d73f77a72537c6a85fd8d5e2704178a83a891f08",
    "s:coderabbit": "8132ef42750b608836a8b1de6a29a218cfe25251fdb4c3a58fb37929499ec042",
    "s:deepin": "dad2d66d0b1c74e6d80c597545a6c18bd84dbf384a23101d7d109c27cf0f9517",
    "s:elementary": "da092a1d0868b7c36132fb4da9f9657c1963660df8ce931ff9dab363aebf14a2",
    "s:endeavouros": "f3963917a83a19829333cecbc9b237a1e7b3950b5dfb92ec8d27e2faf218a8d2",
    "s:heliumbrowser": "4775a09db70f1bd20caa897350195b33f669b8c8bbef432cc704c41b36f1c888",
    "s:linuxmint": "ab7e227a8e7b7d167d8bc4617c6323c205b4565ce1c206e6d1fd677669cd27d4",
    "s:manjaro": "b2d57d08c236453310773ca2f12163ce53e6757ba1698c10a8c0d39ac7cf0019",
    "s:mintlify": "a710b348a80536c41fccdf1078f1acc2f6bdb6b5054a91d8102dec2e18b34d22",
    "s:omarchy": "37cd8cc02703153b802ab3f43da1ef7ed63882a7bc02bee2dd0a9dbbfc6dc6f2",
    "s:planetscale": "ab0f55fdb46c4495ee5528752bf23333479bf200d4af2347215849155ee2c6b8",
    "s:playstation": "b68b4d7b63443759b9c4d77a5501c5758eeae06a6d594d278d5eb6cb4d3dbbe4",
    "s:popos": "b74950a4c3b1dd2300dc1fad68cfaf45843826408dd328d06497b42379bade1e",
    "s:supabase": "61bf16d93be418f723ae1e137e57afdad4f185352fbc6dea9482af1e0a89ccf7",
    "s:ubuntu": "b0aca674c881a07bb60b26afbcee07093ae419e29ee38e376f938817fa66ae03",
    "s:vercel": "57f1e4135486c566e9878fddd259f7f6bf8ec43f31e7e6e38a36d9e814a1c0e0",
    "s:voidlinux": "33edc76feaa84a0a8fbe5f5bf56b2bd8a5d6a7b01a0917bb5e17096b86e49e69",
    "s:zorin": "2ae2329d284e84afbd7111b7a4ffd52fdc38b0d139c045f7439f2b3f687e3e43",
}


def url(src: str) -> str:
    kind, name = src.split(":")
    if kind.startswith("d"):
        return DEVICON.format(version=kind.partition("@")[2] or "2.16.0", dir=name.split("-")[0], name=name)
    return SIMPLE.format(name=name)


def fetch(src: str, pin: bool) -> bytes:
    path = CACHE / (re.sub("[:@]", "-", src) + ".svg")
    if not path.exists():
        CACHE.mkdir(parents=True, exist_ok=True)
        with urllib.request.urlopen(url(src)) as r:
            path.write_bytes(r.read())
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if not pin and SHA256.get(src) != digest:
        sys.exit(f"{src}: SHA-256 {digest} is not the pinned {SHA256.get(src)}")
    return data


def octant_glyphs() -> list[str]:
    """The glyph for each of the 256 octant masks, as scripts/octants.py names them."""
    sys.dont_write_bytecode = True  # no __pycache__ in scripts/
    spec = importlib.util.spec_from_file_location("octants", ROOT / "scripts/octants.py")
    octants = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(octants)
    glyphs = []
    for mask in range(256):
        if mask in octants.ELSEWHERE:
            glyphs.append(unicodedata.lookup(octants.ELSEWHERE[mask]))
        else:
            dots = "".join(str(d + 1) for d in range(8) if mask >> d & 1)
            glyphs.append(unicodedata.lookup(f"BLOCK OCTANT-{dots}"))
    return glyphs


def wedges() -> list[tuple[str, list[bool]]]:
    """Each wedge glyph and its samples, from its name: the half of the cell on
    its corner's side of the line between two points on the sextant grid."""
    xs = {"LEFT": 0.0, "CENTRE": 0.5, "RIGHT": 1.0}

    def point(words: str) -> tuple[float, float]:
        *ys, x = words.split()
        y = {"UPPER": 0.0, "UPPER MIDDLE": 1 / 3, "LOWER MIDDLE": 2 / 3, "LOWER": 1.0}[" ".join(ys)]
        return xs[x], y

    out = []
    w, h = 2 * SS, 4 * SS
    for cp in range(0x1FB3C, 0x1FB68):
        m = re.fullmatch(r"(\w+ \w+) BLOCK DIAGONAL (.+) TO (.+)", unicodedata.name(chr(cp)))
        corner, (ax, ay), (bx, by) = point(m.group(1)), point(m.group(2)), point(m.group(3))
        side = lambda px, py: (bx - ax) * (py - ay) - (by - ay) * (px - ax)  # noqa: E731
        want = side(*corner) > 0
        out.append((chr(cp), [(side((i + 0.5) / w, (j + 0.5) / h) > 0) == want for j in range(h) for i in range(w)]))
    return out


def palette(category: str, slug: str) -> list[tuple[int, int, int]]:
    """The logo's colours as drawn: the first sixth of the piece's palette."""
    rs = (PIECES / category / (slug.replace("-", "_") + ".rs")).read_text()
    hexes = re.findall(r'"#([0-9a-f]{6})"', rs.split("palette:")[1].split("]")[0])
    return [tuple(int(h[k : k + 2], 16) for k in (0, 2, 4)) for h in hexes[: len(hexes) // 6]]


def size(category: str, slug: str) -> tuple[int, int]:
    rs = (PIECES / category / (slug.replace("-", "_") + ".rs")).read_text()
    return int(re.search(r"cols: (\d+)", rs).group(1)), int(re.search(r"rows: (\d+)", rs).group(1))


def render(svg: bytes, cols: int, rows: int) -> Image.Image:
    """The SVG fitted into the frame less its margin, at SS samples a dot."""
    big = Image.open(io.BytesIO(bytes(resvg_py.svg_to_bytes(svg_string=svg.decode(), width=2048)))).convert("RGBA")
    x0, y0, x1, y1 = big.getchannel("A").point(lambda a: 255 if a > 8 else 0).getbbox()
    bw, bh = (cols - 4) * 2 * SS, (rows - 2) * 4 * SS
    scale = min(bw / (x1 - x0), bh / (y1 - y0))
    full = Image.open(io.BytesIO(bytes(resvg_py.svg_to_bytes(svg_string=svg.decode(), width=round(2048 * scale))))).convert("RGBA")
    crop = full.crop((round(x0 * scale), round(y0 * scale), round(x1 * scale), round(y1 * scale)))
    out = Image.new("RGBA", (cols * 2 * SS, rows * 4 * SS))
    out.paste(crop, (2 * 2 * SS + (bw - crop.width) // 2, 4 * SS + (bh - crop.height) // 2))
    return out


def labels(img: Image.Image, colours: list, recolour: bool, holes: bool) -> np.ndarray:
    """Each sample's colour index, or -1 where it is bare (or white, for one ink)."""
    a = np.asarray(img).astype(np.int32)
    rgb, alpha = a[..., :3], a[..., 3]
    if recolour:
        lab = np.zeros(alpha.shape, np.int32)
    else:
        pal = np.array(colours, np.int32)
        lab = ((rgb[..., None, :] - pal) ** 2).sum(-1).argmin(-1).astype(np.int32)
    bare = alpha < 128
    if holes:
        bare |= rgb.min(-1) >= WHITE
    lab[bare] = -1
    return lab


def gaps(lab: np.ndarray) -> np.ndarray:
    """Each part cut back by half a dot wherever it meets a part of another colour."""
    out = lab.copy()
    for k in set(np.unique(lab)) - {-1}:
        img = Image.fromarray(((lab >= 0) & (lab != k)).astype(np.uint8) * 255)
        near = np.asarray(img.filter(ImageFilter.MaxFilter(SS + 1))) > 0
        out[(lab == k) & near] = -1
    return out


def bare_contrast(lab: np.ndarray, colours: list, cols: int, rows: int) -> np.ndarray:
    """A cell has one colour, so where a part meets one much lighter or darker,
    the cell takes the colour most of it has and leaves the other bare: a part
    a dot thin (C++'s plusses) shows as a cut, not lost under its neighbour.
    Shades of a gradient, closer than CONTRAST, fill their cells as one."""
    lum = np.array([(0.2126 * r + 0.7152 * g + 0.0722 * b) / 255 for r, g, b in colours] + [0.0])
    h, w = 4 * SS, 2 * SS
    cell = lab.reshape(rows, h, cols, w).transpose(0, 2, 1, 3).reshape(rows, cols, h * w)
    out = cell.copy()
    for y in range(rows):
        for x in range(cols):
            c = cell[y, x]
            if (c >= 0).any():
                major = np.bincount(c[c >= 0]).argmax()
                out[y, x][(c >= 0) & (np.abs(lum[c] - lum[major]) > CONTRAST)] = -1
    return out.reshape(rows, cols, h, w).transpose(0, 2, 1, 3).reshape(lab.shape)


def cells(lab: np.ndarray, cols: int, rows: int, octs: list[str], wed: list) -> tuple[list[str], list[str]]:
    """The glyph and the colour digit of each cell."""
    h, w = 4 * SS, 2 * SS
    cell = lab.reshape(rows, h, cols, w).transpose(0, 2, 1, 3)  # rows, cols, h, w
    on = cell >= 0
    # The best octant: each dot lit where most of its samples are.
    dots = on.reshape(rows, cols, 4, SS, 2, SS).sum((3, 5))  # rows, cols, 4, 2
    lit = dots > LIT * SS * SS
    mask = (lit.reshape(rows, cols, 8).astype(np.int32) << np.arange(8)).sum(-1)
    miss = np.where(lit, SS * SS - dots, dots).sum((2, 3))
    shapes = np.array([g for _, g in wed]).reshape(len(wed), h, w)
    wmiss = (on[:, :, None] != shapes).sum((3, 4))  # rows, cols, wedges
    best = wmiss.argmin(-1)
    art, ink = [], []
    for y in range(rows):
        line, inkline = "", ""
        for x in range(cols):
            m = int(mask[y, x])
            if not on[y, x].any():
                line, inkline = line + " ", inkline + " "
                continue
            glyph = octs[m]
            if 0 < m < 255 and wmiss[y, x, best[y, x]] < WEDGE * miss[y, x]:
                glyph = wed[best[y, x]][0]
            if glyph == " ":
                inkline += " "
            else:
                counts = np.bincount(cell[y, x][on[y, x]])
                inkline += DIGITS[int(counts.argmax())]
            line += glyph
        art.append(line.rstrip())
        ink.append(inkline.rstrip())
    return art, ink


DIGITS = "0123456789abcdefghijklmnopqrstuvwxyz"


def two_colours(lab: np.ndarray, colours: list, cols: int, rows: int, octs: list[str], wed: list, art: list[str], ink: list[str]) -> tuple[list[str], list[str], list[str]]:
    """Where the logo covers a cell and two colours meet in it, the cell is
    redrawn as the lesser colour's octant or wedge over the greater as its back
    colour (a third colour joins whichever of the two it is nearer), so a part a
    dot thin keeps its colour and a boundary between colours its slant. Returns
    the art, the ink and the back colour of each cell (space for none)."""
    h, w = 4 * SS, 2 * SS
    cell = lab.reshape(rows, h, cols, w).transpose(0, 2, 1, 3)
    shapes = np.array([g for _, g in wed]).reshape(len(wed), h, w)
    near = lambda a, b: sum((p - q) ** 2 for p, q in zip(colours[a], colours[b]))  # noqa: E731
    art = [list(line.ljust(cols)) for line in art]
    ink = [list(line.ljust(cols)) for line in ink]
    back = [[" "] * cols for _ in range(rows)]
    for y in range(rows):
        for x in range(cols):
            c = cell[y, x]
            on = c >= 0
            if on.mean() <= 0.9:
                continue
            counts = np.bincount(c[on], minlength=len(colours))
            order = [int(k) for k in np.argsort(-counts, kind="stable") if counts[k] > 0]
            if len(order) < 2:
                continue
            under, over = order[0], order[1]
            fg = np.zeros((h, w), bool)
            for k in order[1:]:
                if k == over or near(k, over) < near(k, under):
                    fg |= c == k
            dots = fg.reshape(4, SS, 2, SS).sum((1, 3))
            lit = dots > LIT * SS * SS
            m = int((lit.reshape(8).astype(np.int32) << np.arange(8)).sum())
            if m == 0:
                art[y][x], ink[y][x] = "█", DIGITS[under]
                continue
            if m == 255:
                art[y][x], ink[y][x] = "█", DIGITS[over]
                continue
            glyph = octs[m]
            miss = np.where(lit, SS * SS - dots, dots).sum()
            wmiss = (fg[None] != shapes).sum((1, 2))
            if wmiss.min() < WEDGE * miss:
                glyph = wed[int(wmiss.argmin())][0]
            art[y][x], ink[y][x], back[y][x] = glyph, DIGITS[over], DIGITS[under]
    return ["".join(line).rstrip() for line in art], ["".join(line).rstrip() for line in ink], ["".join(line).rstrip() for line in back]


def main() -> None:
    args = sys.argv[1:]
    pin = "--pin" in args
    slugs = [a for a in args if a != "--pin"] or list(LOGOS)
    octs, wed = octant_glyphs(), wedges()
    digests = {}
    for slug in slugs:
        category, colour_src, mono_src = LOGOS[slug]
        cols, rows = size(category, slug)
        colours = palette(category, slug)
        recolour = colour_src.startswith("s")
        svg = fetch(colour_src, pin)
        digests[colour_src] = hashlib.sha256(svg).hexdigest()
        img = render(svg, cols, rows)
        lab = labels(img, colours, recolour, False)
        if slug in GAPS:
            lab = gaps(lab)
        art, ink = cells(bare_contrast(lab, colours, cols, rows), cols, rows, octs, wed)
        art, ink, back = two_colours(lab, colours, cols, rows, octs, wed, art, ink)
        if mono_src:
            msvg = fetch(mono_src, pin)
            digests[mono_src] = hashlib.sha256(msvg).hexdigest()
            mono, _ = cells(labels(render(msvg, cols, rows), colours, True, True), cols, rows, octs, wed)
        else:
            lab = labels(img, colours, recolour, True)
            if slug in GAPS:
                lab = gaps(lab)
            mono, _ = cells(lab, cols, rows, octs, wed)
        out = PIECES / category / "art" / f"{slug}.txt"
        out.parent.mkdir(exist_ok=True)
        head = f"# {slug}: {colour_src}" + (f", {mono_src} in one ink" if mono_src else "") + ". Made by scripts/logo.py: don't edit.\n"
        out.write_text(head + "# colour\n" + "\n".join(art) + "\n# ink\n" + "\n".join(ink) + "\n# back\n" + "\n".join(back) + "\n# one ink\n" + "\n".join(mono) + "\n")
        print(f"{slug}: {sum(c != ' ' for line in art for c in line)} cells", file=sys.stderr)
    if pin:
        print("SHA256 = {")
        for src in sorted(digests):
            print(f'    "{src}": "{digests[src]}",')
        print("}")


if __name__ == "__main__":
    main()
