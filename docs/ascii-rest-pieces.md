# animate-unicode: the ascii.rest pieces

Every piece on [ascii.rest](https://ascii.rest), translated to Unicode and made
better. A row is not done when it matches the TypeScript. It is done when the
piece is drawn with the glyphs that suit it (braille, octants, sextants, eighth
blocks, box drawing) and has one improvement the ASCII original could not have.
Started 2026-10-08 against `../ascii` at `466ad06`: 209 pieces, the 210 files in
`src/pieces` less `index.ts`.

This widens [Phase 6](IMPLEMENTATION_PLAN.md#phase-6), whose own ten pieces are
done. Those six originals (braille-wave, braille-lissajous, sextant-plasma,
geometric-tiles, arrow-field, octant-sphere) are not ascii.rest pieces, so they
are not tracked here.

- [Progress](#progress)
- [What a row needs](#what-a-row-needs)
- [Engine work first](#engine-work-first)
- [Order](#order)
- [Pieces](#pieces)

## Progress

| category | pieces | todo | wip | built | done | covered |
|---|---|---|---|---|---|---|
| scenes | 15 | 15 | | | | |
| shapes | 12 | | | 11 | | 1 |
| space | 11 | | | 11 | | |
| physics | 14 | | | 14 | | |
| nature | 16 | 16 | | | | |
| creatures | 10 | 10 | | | | |
| objects | 14 | 14 | | | | |
| generative | 13 | | | 13 | | |
| effects | 8 | 6 | | | | 2 |
| ui | 12 | | | 12 | | |
| data | 10 | | | 10 | | |
| type | 9 | | | 9 | | |
| logos | 28 | 28 | | | | |
| companies | 14 | 14 | | | | |
| distros | 23 | 23 | | | | |
| **all** | **209** | **126** | | **80** | | **3** |

Recount after changing rows:

```sh
grep -E '^\| [a-z0-9-]+ \| [0-9]+×' docs/ascii-rest-pieces.md | awk -F'|' '{split($(NF-1),s,":"); gsub(/ /,"",s[1]); print s[1]}' | sort | uniq -c
```

Statuses, as the last cell of a row:

- **todo**: not started.
- **wip**: on a branch. Name it: `wip: pieces-shapes`.
- **built**: the contract is green and the piece has been checked headless. It
  is waiting for the owner's look in a real browser.
- **done**: the owner has looked at it in a browser, at 1× and 2× DPR, dark
  and light, and in the CLI.
- **covered: X**: an existing piece is already this one's Unicode version.
  ascii.rest's `donut` is also kept here as a byte-exact ASCII port; that one
  is the fixture for the core crate, not the translation.

## What a row needs

1. **The model, ported.** Port the geometry or the simulation from
   `../ascii/src/pieces/<slug>.ts`, keeping its seeded PRNG. Where there is
   state (a fire, a sand pile, a Life grid), fixture-test that state against
   the TypeScript at a few steps, as `quadrant-fire` does for doom-fire's heat.
   The text cannot be fixture-tested, because the drawing is meant to differ.
   Pieces that are a pure function of `t` need no fixture.
2. **Drawn in Unicode**, by the plan in the row's *Unicode* cell. Use the
   canvases that exist (`subcell::Canvas` with `Dots`, `Sextants`,
   `Quadrants`, `Octants`) and `palette::Ramp`, or add to them. Most pieces
   become `Charset::Extended`. A plan that stays in box drawing and blocks
   keeps `Basic`, which plays in any terminal font.
3. **One enhancement**, from the *enhancement* cell. The cell is a
   suggestion; change it if something better shows up while building, and
   change the row to match.
4. **Still readable in one ink.** In a `<pre>`, and with `--mono` in the CLI,
   colour is lost. A piece whose enhancement is colour must still read
   without it, and must shade the right way round with `paper`.
5. **Slug and header.** Keep ascii.rest's slug unless it is covered. Keep the
   TS file's header comment, and add "after ascii.rest's X (MIT,
   @bas3line)".
6. **Within the contract and budgets**: 4 ms average and 30 ms worst a frame
   (scenes 10 and 40), checked in release. `mise run check` green.

## Engine work first

Some rows cannot start until the engine has more. Each of these is a small
phase of its own, with its own branch.

Already added, with the shapes (branch `pieces-shapes`):
`subcell::ColorCanvas` (`ColorDots`, `ColorOctants`, …), a dot canvas with a
palette index and a depth per dot. A cell takes its commonest colour;
`draw_split` keeps only that colour's dots, and `dither_by` thins a lit surface
by its shade. It also brought `util::dither` (4×4 Bayer) and `geom` (vectors
and rotation matrices). The generative pieces added `util::Mulberry32`,
ascii.rest's own generator bit for bit, so ports place the same seeds, mazes
and particles (quadrant-fire now uses it too).

| # | what | needed by | status |
|---|---|---|---|
| E1 | **Two colours a cell.** A background colour per cell beside the foreground: `Grid`, the wasm `Player`, the canvas renderer (fill the cell, then draw the glyph), and the CLI (`48;2;r;g;b`). Half blocks (`▀` over a different `▄`) then give two square pixels a cell, each in its own colour. The `<pre>` path has one ink, so a piece using this needs an octant-dither fallback there. | the 15 scenes, sunrise, landscape; a filled plasma | todo |
| E2 | **Local time in `Env`.** `Env` has only `paper`. Clock pieces need the local wall time: from `Date` in the shell, the system clock in the CLI, and a fixed instant in the contract test so it stays deterministic. | analog-clock, calendar, digital-clock, landscape, sundial | done: `clock::Wall` and `Ticker`, `Env::now`, `Player::set_clock` (mount.ts calls it), chrono in the CLI |
| E3 | **Categories.** ascii.rest has `companies` and `distros`; `Category` stops at `Logos`. Add the two, or fold them into `logos`. The site's sidebar groups and the CLI's `list` follow, and each new category gets its Cargo feature in both crates, which makes it its own wasm module. | the 37 company and distro logos | todo |
| E4 | **Logos from SVG.** A build script (`scripts/logo.py`, run with uvx like `font.py`) that renders each SVG and reduces it to an octant mask, choosing a wedge glyph (U+1FB3C–1FB67) where an edge cuts a cell at a slant, with a colour per cell. Its output is committed. One shared `logo` module draws any logo from that data and runs the glint or the scan line, so 65 logos cost data rather than 65 copies of the code. | logos, companies, distros | todo |
| E5 | **Size.** 64.6 KB gz with 11 pieces, about 2.8 KB more for each further piece. After wave 1 (2026-10-09): 148.0 KB gz with 49 pieces, about 2.2 KB a piece since. After ui and data (2026-10-09): 213.7 KB gz with 70 pieces; those 21 cost about 3.1 KB each, the text pieces carrying their strings. After type (2026-10-09): 249.0 KB gz with 79 pieces; those 9 cost about 3.9 KB each, carrying fonts, stroke tables and Unicode's case tables (ASCII-only uppercasing would save 2 KB but mangle accented titles). The 137 still to build would add some 470 KB at wave 2's 3.4 KB a piece, to about 720 KB: well past the gate. Decide before wave 3. E4 makes logos cheaper. Before wave 3, decide whether to raise the gate or to split pieces into several wasm modules ([risk 5](IMPLEMENTATION_PLAN.md#risks-and-open-questions)). | everything after wave 2 | done (2026-10-09): a wasm module per category, each a Cargo feature; 41–91 KB gz each, gated at 150 KB a module ([plan](IMPLEMENTATION_PLAN.md#decisions-already-made)) |

Logo licences: devicon's SVGs are MIT, and Simple Icons' are CC0 (omarchy's
is MIT). Five companies' marks came from their own sites: command-code,
databuddy, greptile, orchid, polar. Check each before using it, or drop it.
Every logo is a trademark of its owner. Keep ascii.rest's line saying it is
shown to name the language, company or distribution.

## Order

Each wave is one branch per category, stacked.

1. **Shapes, generative, physics** (38). These are self-contained, and the
   canvases they need exist. They show what Unicode does with geometry.
2. **UI, data, type** (31). Box drawing, eighth blocks, and options-heavy
   pieces. E2 must come first for calendar and digital-clock.
3. **Space, nature, creatures, objects, effects** (57). Creatures and some
   objects are sprites that have to be redrawn at the higher resolution, not
   converted. E5 is done: each category is its own wasm module.
4. **Logos, companies, distros** (65), after E3 and E4.
5. **Scenes** (15), after E1. These are the largest: tokyo-rain alone is
   1 123 lines of TS.

## Pieces

*size, fps* is ascii.rest's, with its flags. The Unicode version may change
the size: a braille piece shows four times the rows of dots in the same
cells.

**Scenes.** Every scene is a halftone of `· • ●` in palette colours on square
cells (`cell: 1`), 200×100. Half blocks with two colours a cell (E1), on
ordinary 1:2 cells at 200×100, make that 200×200 solid pixels: twice the
vertical resolution, with no gaps between dots. The dithering then moves to
colour, and the shapes become solid.

**Logos, companies, distros.** These all share E4's plan, so their rows
carry only the source.

### scenes (15)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| alpine-dawn | 200×100, 15 · colour, square cells | half blocks, two colours a cell | alpenglow sliding down the faces; the lake mirror breaks up in ripples | todo |
| aurora-fjord | 200×100, 15 · colour, square cells | half blocks, two colours a cell | curtain folds in vertical eighth-block streaks; the cabin's window flickers | todo |
| deep-reef | 200×100, 15 · colour, square cells | half blocks, two colours a cell | caustics wandering over the sand; fish as octant sprites, not dots | todo |
| desert-night | 200×100, 15 · colour, square cells | half blocks, two colours a cell | stars by magnitude (· • ● ✦), meteors as braille streaks | todo |
| earthrise | 200×100, 15 · colour, square cells | half blocks, two colours a cell | the earth as an octant globe with cloud bands; craters lit by the low sun | todo |
| kyoto-dusk | 200×100, 15 · colour, square cells | half blocks, two colours a cell | petals drifting in braille; the lantern's light flickering on the wall | todo |
| lantern-lake | 200×100, 15 · colour, square cells | half blocks, two colours a cell | lanterns glow and dim as they climb; their reflections break on the water | todo |
| marine-drive | 200×100, 15 · colour, square cells | half blocks, two colours a cell | traffic lights streaming along the curve; the necklace reflected and rippling | todo |
| misty-forest | 200×100, 15 · colour, square cells | half blocks, two colours a cell | fog layers drifting at different speeds; sunbeams dithered in octants | todo |
| night-coast | 200×100, 15 · colour, square cells | half blocks, two colours a cell | the beam as a dithered cone through cloud; moonglade on the swell | todo |
| ocean-sunset | 200×100, 15 · colour, square cells | half blocks, two colours a cell | the sun flattening at the horizon; glitter as braille sparkles | todo |
| storm-plains | 200×100, 15 · colour, square cells | half blocks, two colours a cell | lightning lighting the cloud from inside; wheat waving in gust fronts | todo |
| taj-dawn | 200×100, 15 · colour, square cells | half blocks, two colours a cell | the haze lifting over the minutes; birds crossing the pool | todo |
| tokyo-rain | 200×100, 15 · colour, square cells | half blocks, two colours a cell | rain in braille at three depths; neon smeared in the road's reflection | todo |
| varanasi-ghats | 200×100, 15 · colour, square cells | half blocks, two colours a cell | diyas drifting on the current; smoke from the aarti in braille | todo |

### shapes (12)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| cube | 32×18, 24 | braille: faces dithered by their light, solid edges over them | a colour per pair of faces (red, green, blue), so each quarter turn hands one colour to the next | built |
| dna-helix | 64×15, 20 | braille: near backbone two dots thick and clearing round itself, far one a dot thick | base pairs coloured half by half (A T C G) from a seeded sequence; dims with depth | built |
| donut | 40×22, 30 | braille, z-buffer per dot, Bayer dither | — | covered: braille-donut |
| glxgears | 56×31, 24 | octants; a cell keeps only the dots of the part covering most of it | glxgears' red, green and blue; lit and shadowed walls; dark walls dithered for one ink | built |
| gyroscope | 60×32, 30 | braille, z-buffered per dot, dithered by light | gold, silver and copper gimbals round a steel rotor; two-dot gaps where parts cross | built |
| heart | 52×24, 30 | octants, ray-marched per dot, Bayer dither | a ramp of reds with a solid pink-white glint | built |
| icosahedron | 46×22, 30 | braille wireframe, hidden edges every third dot | edges coloured by depth, ice-white in front to deep blue behind | built |
| mobius-strip | 58×20, 30 | braille, z-buffered per dot, dithered | a rainbow running twice round the one face, so it never jumps at the twist | built |
| spring | 40×28, 30 | braille helix at four dots a row, stretching smoothly; front coils thick, back ones thin | a steel coil and a brass drum lit from the left | built |
| tesseract | 54×28, 30 | braille lines, far half dotted | edges coloured by w, amber toward the 4D eye and violet away | built |
| torus-knot | 60×28, 30 | braille, z-buffered per dot, dithered; crossings cut two dots back | a sea-glass ramp with a white glint | built |
| twisted-ring | 46×21, 30 | braille faces dithered by light, corners as solid lines | colour runs along the one face and returns after four laps | built |

### space (11)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| black-hole | 64×20, 15 | a ray traced through the bent space for every octant dot, dithered; a photon ring round the shadow | the gas by temperature, white-hot at the inner edge to deep red, hotter on the side coming toward us | built |
| earth | 60×30, 20 | an octant globe, the map blended so the coasts run smooth; land dense, sea a light texture | green and desert land by latitude, polar ice, a blue sea with the sun's glint, a dim night side, a blue rim of air | built |
| eclipse | 64×27, 20 | the sun, moon, beads and corona sampled per octant dot | a white-gold sun darkening to orange at the limb, a pearl corona cooling to blue, pink prominences, a white diamond ring with flares | built |
| galaxy | 64×26, 20 | braille, each dot a star shown where the light beats its own random threshold | a yellow core, blue-white arms with pink knots, brown dust lanes, white orbiting stars | built |
| moon-phases | 46×23, 15 | octant dither on the disc, a fine terminator | grey by the ground: dark maria, bright craters and rays; a faint blue earthshine; the dark side's rim | built |
| planet | 64×18, 20 | octants, the geometry worked out once and only the clouds turning | cream zones, brown belts, a white storm, a grey-blue pole, gold rings and a dusky inner ring, shadows both ways | built |
| rocket | 48×28, 20 | redrawn in octants: the sprite's outline at dot resolution, moving a dot at a time; a braced lattice tower | a cylinder-shaded white rocket, a flame white to red, grey smoke lit orange by it, a red beacon | built |
| saptarishi | 64×18, 12 · options | ● and • by magnitude; dotted braille lines; braille meteors | each star its spectral class (K orange, A white, B blue); the lines draw in round the dipper, hold and fade | built |
| solar-system | 61×29, 20 · options | braille orbits as true ellipses with the sun at a focus, each planet placed by Kepler's equation | a colour per planet, trails that stretch at perihelion, a glowing sun | built |
| starfield | 64×24, 30 · options | braille streaks as long as the shutter, thinning to the tail | amber ahead, cooling to blue at the edges | built |
| three-body | 65×15, 30 | a dotted braille eight; braille trails that thin with age | red, green and blue bodies | built |

### physics (14)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| bouncing-balls | 64×22, 30 · options | the TS physics; lit rubber spheres per octant dot, squashed to keep their area | a colour per ball, dithered shading, shadows that pale as a ball rises | built |
| chladni | 45×23, 12 | a grain per braille dot on dot-wide nodal lines; 2 600 grains, paired along a Hilbert curve | each figure's sand its own tint, grains in flight white | built |
| double-pendulum | 48×25, 30 | the TS RK4; braille rods and a trail twice as long | the trail coloured by tip speed, blue to white, thinning with age | built |
| falling-sand | 55×22, 20 | the TS automaton with a grain per octant dot: four times the grains | the heap pours in layers of three sand tones | built |
| flag | 62×21, 30 · options | the TS cloth strips laid per octant dot row, folds dithered | crimson and cream stripes, each in four shades of its fold's light | built |
| fountain | 57×22, 30 | the TS drops (same hash), each a braille dot; the surface ripples a dot at a time | water coloured by speed, deep blue to white | built |
| harmonograph | 64×28, 30 | a true braille curve a dot wide | ink turning rose to blue as the figure winds in | built |
| lorenz | 60×22, 30 | the TS RK4 and view; the path a joined braille curve | one wing ice blue, the other amber, by age and nearness | built |
| newtons-cradle | 74×17, 30 · options | octant chrome discs, dot-wide strings and legs | chrome shaded by what it reflects, with a sliding highlight | built |
| pendulum-wave | 67×19, 30 | octant discs moving a dot at a time | each bob coloured by its phase, so the waves show as bands of hue | built |
| plucked-string | 64×14, 30 | the TS harmonics, the string per braille dot | its first three harmonics drawn faintly behind it, each its colour | built |
| pond-ripples | 64×20, 20 | the TS rain (same hash); rings as true braille circles | water blue by strength; crossings of strong rings flash white | built |
| smoke | 44×24, 30 | the TS puffs and flow; the thread a braille line, the haze dithered | grey haze thinning out, a flickering ember | built |
| wave-interference | 72×26, 24 · options | the tank per octant dot, dithered by height | crests cyan to white, troughs deep blue, still lines dark lanes | built |

### nature (16)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| aurora | 64×20, 20 | sextant curtains, vertical eighth blocks | green to magenta palette, twinkling stars | todo |
| bonsai | 52×19, 10 | braille branches, octant leaves | leaves changing colour through the seasons before they fall | todo |
| campfire | 52×22, 20 | quadrant fire (quadrant-fire's spread), wedge logs | fire palette, sparks as braille | todo |
| cherry-blossom | 64×20, 15 | braille branch, petals as · • | pink petals tumbling on the wind | todo |
| contour-map | 72×20, 8 | contours at sub-cell resolution (as sextant-plasma) | hypsometric colour bands, sea to snow | todo |
| fern | 44×30, 20 | braille points, 8 a cell | greens by depth | todo |
| fireflies | 60×18, 15 · options | braille glow halos | yellow-green flashes spreading | todo |
| fractal-tree | 60×24, 15 · options | braille branches, octant trunk | leaf colour at the tips | todo |
| landscape | 64×18, 8 · options, clock | octant ridges, half-block sky | a sky gradient set by the hour | todo |
| lightning | 64×22, 20 | braille bolt and branches | the flash lighting the cloud in colour | todo |
| rain | 64×20, 30 | braille streaks at three depths | depth colours, braille splash crowns | todo |
| ruled-mountains | 64×20, 10 | braille lines at 4× vertical resolution | hidden lines removed per dot | todo |
| sea-swell | 64×14, 20 | octant water, wedge-cut sails | sky and sea in colour | todo |
| snowfall | 64×22, 20 | braille flakes, eighth-block drifts | flake size by depth | todo |
| sunrise | 60×20, 15 | half-block sky gradient, braille glitter | the sky warming in colour | todo |
| wind | 64×14, 15 | braille streamlines | leaves carried on the gusts | todo |

### creatures (10)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| aquarium | 60×20, 15 | octant fish sprites, braille bubbles | a colour per fish, swaying weed | todo |
| butterfly | 64×24, 30 | octant wings | patterned, coloured wings | todo |
| cat | 56×15, 15 | the sprite redrawn in octants | tabby stripes in colour | todo |
| fox | 47×16, 10 | the sprite redrawn in octants | orange coat with a white tail tip | todo |
| jellyfish | 44×26, 24 | octant bell, braille tentacles | a translucent glow palette | todo |
| owl | 41×24, 12 | the sprite redrawn in octants | eyes that catch the light as it blinks | todo |
| snake | 64×15, 20 | octant body | patterned scales in colour | todo |
| spider | 28×24, 15 | braille silk, octant body | a web drawn behind it | todo |
| starlings | 64×16, 30 | one bird a braille dot (boids) | eight times the birds in the same cells | todo |
| whale | 72×18, 15 | octant body, braille spout | sea colour and a spray that drifts | todo |

### objects (14)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| analog-clock | 47×23, 8 · options, clock | braille face and hands at any angle | a coloured second hand, finer ticks | todo |
| candle | 40×30, 15 | octant wax, quadrant flame | flame palette and a soft glow | todo |
| coffee | 52×24, 20 | braille steam, octant cup | colour for the cup and the coffee | todo |
| ferris-wheel | 64×26, 10 | braille rim and spokes | coloured cabins, lights along the rim at night | todo |
| hawa-mahal | 61×24, 10 | octant facade | lamps behind the jali in warm colours | todo |
| hourglass | 43×21, 30 | the sand per braille dot | sand colour, a glass highlight | todo |
| kite | 64×24, 15 | braille string, wedge-cut kite | coloured panels and a tail | todo |
| lava-lamp | 30×27, 15 | metaballs at octant resolution | wax and glass in colour, a glow | todo |
| lighthouse | 64×30, 20 | braille beam cone with dither | beam and lamp in colour | todo |
| skyline | 64×23, 10 | octant buildings, quadrant windows | warm and cool windows in colour | todo |
| sundial | 65×23, 10 · clock | octant instrument | the shadow in colour, a sunlit face | todo |
| train | 76×26, 15 | octant engine, braille smoke | colour for the engine and carriages | todo |
| vinyl | 64×25, 24 | braille grooves | a coloured label, a highlight sweeping the grooves | todo |
| windmill | 64×28, 20 | braille lattice sails | colour for the mill and the clouds | todo |

### generative (13)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| epicycles | 60×24, 30 | braille trace a dot wide; the first five circles' rims and arms dotted | a colour per circle; the trail cools from gold to embers | built |
| flow-field | 64×22, 20 | the TS field and particles (its own generator), trails laid per braille dot | each dot coloured by its heading, violet climbing to orange falling | built |
| glider-gun | 56×20, 10 | one Life cell per octant dot: a 112×80 field, the lane twice as long | colour by age: births white, cooling to blue; period 30 tested | built |
| hilbert-curve | 65×17, 20 | one order finer (32 by 16) in braille | two runners, amber and cyan, two dots wide, fading along their trails | built |
| julia-set | 66×24, 20 | per octant dot, the glow dithered | violet through rose and gold to white at the edge | built |
| langtons-ant | 72×25, 30 | one square per octant dot: four times the board; highway run to step 13 400 | squares coloured by when they last flipped, hot to cold; highway tested | built |
| mandelbrot | 64×26, 15 | per octant dot, the boundary glow dithered | smooth escape time through a cycling palette; 2.2 ms avg, 6.7 worst over the full zoom | built |
| maze | 61×21, 20 | a 30 by 20 maze in braille (four times the rooms); backtracker matches the TS | the stack a cyan thread, the way through in amber | built |
| plasma | 64×22, 24 | per octant dot in twelve bands, a cell keeping its main band's dots | filled colour bands, ink-blue to gold; density follows the band for one ink | built |
| reaction-diffusion | 60×24, 20 | ascii.rest's 60×48 reaction (matches the TS), read between grid points per octant dot | teal to cream by strength, with crisp edges | built |
| rule-30 | 64×20, 8 | one cell per octant dot: 128 across, four generations a row | a colour per rule; rule 90 grows six levels | built |
| sierpinski | 59×26, 20 | the chaos game per braille dot, truly equilateral | each dot coloured by the corner it jumped toward, so the three copies show | built |
| voronoi | 64×22, 15 | nearest seed per octant dot, borders left dark | a colour per cell, brighter toward its seed | built |

### effects (8)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| doom-fire | 60×18, 24 | quadrants, Doom palette | — | covered: quadrant-fire |
| fireworks | 64×24, 30 | braille sparks | a palette per shell, trails cooling | todo |
| matrix-rain | 63×24, 20 | full-width katakana and kanji | — | covered: kanji-rain |
| rotozoomer | 64×24, 30 | the texture sampled per octant dot | brick colours | todo |
| sparks | 64×20, 30 | braille sparks | colour cooling from white to red | todo |
| synthwave | 65×28, 20 | braille grid in perspective, octant sun stripes | a neon palette | todo |
| tunnel | 64×24, 30 | the texture sampled per octant dot | colour with depth fog | todo |
| tv-static | 58×26, 20 · options | octant noise, half the dots lit | a colour test card (SMPTE bars) | todo |

### ui (12)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| boot-log | 64×16, 15 | eighth-block memory bar; braille spinners for devices being probed | a console's colours: green ok, amber warn, dim timestamps | built |
| box-frames | 78×20, still | box drawing, ascii.rest's sheet to the cell (fixture-tested) | each frame its own hue round the wheel, labels dim | built |
| calendar | 32×13, 8 · options, clock | cal's layout; today in a half-block pill `▐ 8▌` | today in amber, weekends dim, the clock in its own colour | built |
| digital-clock | 70×11, 10 · options, clock | octant seven-segment digits with pointed segment ends | lit segments LED red, unlit a faint dithered trace | built |
| dividers | 63×25, still · options | ascii.rest's sheet to the cell (fixture-tested) | each rule shades teal to violet to rose along its length | built |
| file-tree | 36×23, 15 · options | tree lines, `▸` on closed folders | folders blue, files by kind, the cursor amber | built |
| form-controls | 54×16, 15 · options | `○ ◉` radios, `□ ▣` checkboxes, a `●` slider knob, `▾` select | focus amber, choices cyan, saved green | built |
| not-found | 60×21, 15 · options | ascii.rest's digits and shadow; a smooth octant ghost with a rippling hem | the code rose to amber, a pale ghost whose eyes look where it goes | built |
| progress-bar | 48×11, 30 · options | eighth blocks, and a braille bar filling half a cell at a time | each fill shades teal to amber along its length | built |
| skeleton | 52×23, 24 · options | rounded box drawing, shade blocks | the shimmer a band of colour lifting grey to blue-white | built |
| spinners | 66×25, 12 · options | ascii.rest's twelve plus twelve in braille and geometric shapes (rows 13 → 25) | each spinner its colour | built |
| terminal | 62×18, 20 · options | box drawing | a shell's colours: green path, blue sign, grey output, green ticks | built |

### data (10)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| bar-chart | 60×19, 24 · options | eighth blocks, dashed gridlines | each dataset its colour; the bars and tab take the live one's | built |
| candlesticks | 70×22, 15 · options | ascii.rest's chart to the cell (fixture-tested) | green hollow up candles, red solid down, the live one brighter, the last price amber | built |
| cpu-meters | 66×16, 10 · options | meters in block shades (`█ ▓ ▒ ░`) per part; the machine matches the TS | htop's colours: user green, kernel red, buffers blue, cache amber | built |
| equalizer | 62×16, 30 · options | eighth blocks (fixture-tested) | lit by height, green to yellow to red, white caps | built |
| gauge | 55×18, 30 · options | braille dial: rim, ticks and needle a dot wide | the band by zone, green, amber, red, solid to the reading | built |
| heartbeat | 72×15, 25 · options | a smooth PQRST waveform per braille dot | green phosphor, brightest at the pen; a red heart | built |
| heatmap | 66×12, 10 · options | days as squares that grow `· ▪ ◼ ■ █` | a code host's greens | built |
| radar | 53×25, 30 · options | braille scope, dithered afterglow, dotted rings | fading greens, the coast flaring as the beam passes | built |
| sparkline | 72×11, 12 · options | braille area charts two rows tall, two samples a cell (rows 9 → 11, three series) | a colour per series | built |
| uptime-bar | 64×15, 8 · options | ascii.rest's bars to the cell (fixture-tested) | green, amber and red days; the headline coloured by today | built |

### type (9)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| big-text | 66×8, 24 · options | octant letters, each outer corner taken off a dot; the double-line shadow as it is | rose to amber across the word, the glint passing in white | built |
| dissolve | 64×13, 20 · options | braille, a dot a grain: strokes wear away dot by dot and grains fall or fly in | each word its colour, its grains dimming as they fade | built |
| glitch | 64×13, 15 · options | ascii.rest's frames to the cell (fixture-tested) | the word white, the left-behind copy cyan, smears magenta, tears yellow, corrupt cells in any of the three | built |
| marquee | 61×10, 24 · options | a braille dot-matrix sign, an LED a dot with a gap round it; `●`/`·` bulbs | amber LEDs, warm bulbs, a gold frame | built |
| morse | 64×19, 15 · options | ascii.rest's frames to the cell (fixture-tested) | the lens glowing amber to white while keyed, yellow rays, amber marks, bright decoded letters | built |
| scramble | 44×3, 20 · options | noise from braille, sextants, quadrants, arrows and geometric shapes | noise flickering cyan, green and violet; settled letters bright | built |
| split-flap | 61×18, 15 · options | ascii.rest's frames to the cell (fixture-tested), half-block flaps mid-turn | cream cards on a dark board; remarks amber, red, green or cyan by what they ask | built |
| typewriter | 44×3, 20 · options | ascii.rest's frames to the cell (fixture-tested) | an editor's colours: prefix dim, phrase bright, cursor in the accent | built |
| wave-text | 36×5, 15 · options | ascii.rest's frames to the cell (fixture-tested) | each letter shaded by its height, deep blue in a trough to white on a crest | built |

### logos (28)

| piece | size, fps | drawn from | status |
|---|---|---|---|
| c | 49×28, 30 · colour, options | devicon's c-original.svg (MIT) | todo |
| clojure | 56×28, 30 · colour, options | devicon's clojure-original.svg (MIT) | todo |
| cpp | 50×28, 30 · colour, options | devicon's cplusplus-original.svg (MIT) | todo |
| csharp | 49×28, 30 · colour, options | devicon's csharp-original.svg (MIT) | todo |
| css | 50×28, 30 · colour, options | devicon's css3-original.svg (MIT) | todo |
| dart | 56×28, 30 · colour, options | devicon's dart-original.svg (MIT) | todo |
| elixir | 38×28, 30 · colour, options | devicon's elixir-original.svg (MIT) | todo |
| elm | 56×28, 30 · colour, options | devicon's elm-original.svg (MIT) | todo |
| erlang | 70×23, 30 · colour, options | devicon's erlang-original.svg (MIT) | todo |
| go | 79×16, 30 · colour, options | devicon's go-original-wordmark.svg (MIT) | todo |
| haskell | 69×25, 30 · colour, options | devicon's haskell-original.svg (MIT) | todo |
| html | 50×28, 30 · colour, options | devicon's html5-original.svg (MIT) | todo |
| java | 42×28, 30 · colour, options | devicon's java-original.svg (MIT) | todo |
| javascript | 56×28, 30 · colour, options | devicon's javascript-original.svg (MIT) | todo |
| julia | 60×28, 30 · colour, options | devicon's julia-original.svg (MIT) | todo |
| kotlin | 56×28, 30 · colour, options | devicon's kotlin-original.svg (MIT) | todo |
| lua | 56×28, 30 · colour, options | devicon's lua-original.svg (MIT) | todo |
| ocaml | 64×28, 30 · colour, options | devicon's ocaml-original.svg (MIT) | todo |
| perl | 56×28, 30 · colour, options | devicon's perl-original.svg (MIT) | todo |
| php | 69×19, 30 · colour, options | devicon's php-original.svg (MIT) | todo |
| python | 51×28, 30 · colour, options | devicon's python-original.svg (MIT) | todo |
| r | 69×27, 30 · colour, options | devicon's r-original.svg (MIT) | todo |
| ruby | 56×28, 30 · colour, options | devicon's ruby-original.svg (MIT) | todo |
| rust | 64×32, 30 · colour, options | devicon's rust-original.svg (MIT) | todo |
| scala | 37×28, 30 · colour, options | devicon's scala-original.svg (MIT) | todo |
| swift | 58×28, 30 · colour, options | devicon's swift-original.svg (MIT) | todo |
| typescript | 56×28, 30 · colour, options | devicon's typescript-original.svg (MIT) | todo |
| zig | 61×28, 30 · colour, options | devicon's zig-original.svg (MIT) | todo |

### companies (14)

| piece | size, fps | drawn from | status |
|---|---|---|---|
| apple | 46×28, 30 · colour, options | Simple Icons' apple.svg (CC0) | todo |
| cloudflare | 79×19, 30 · colour, options | Simple Icons' cloudflare.svg (CC0) | todo |
| coderabbit | 64×28, 30 · colour, options | Simple Icons' coderabbit.svg (CC0) | todo |
| command-code | 64×32, 30 · colour, options | the company's own mark (licence to check) | todo |
| databuddy | 56×28, 30 · colour, options | the company's own mark (licence to check) | todo |
| greptile | 49×28, 30 · colour, options | the company's own mark (licence to check) | todo |
| helium | 49×28, 30 · colour, options | Simple Icons' heliumbrowser.svg (CC0) | todo |
| mintlify | 56×28, 30 · colour, options | Simple Icons' mintlify.svg (CC0) | todo |
| orchid | 57×28, 30 · colour, options | the company's own mark (licence to check) | todo |
| planetscale | 56×28, 30 · colour, options | Simple Icons' planetscale.svg (CC0) | todo |
| playstation | 70×28, 30 · colour, options | Simple Icons' playstation.svg (CC0) | todo |
| polar | 51×28, 30 · colour, options | the company's own mark (licence to check) | todo |
| supabase | 55×28, 30 · colour, options | Simple Icons' supabase.svg (CC0) | todo |
| vercel | 64×28, 30 · colour, options | Simple Icons' vercel.svg (CC0) | todo |

### distros (23)

| piece | size, fps | drawn from | status |
|---|---|---|---|
| almalinux | 56×28, 30 · colour, options | devicon's almalinux-original.svg (MIT) | todo |
| alpine-linux | 64×28, 30 · colour, options | Simple Icons' alpinelinux.svg (CC0) | todo |
| arch-linux | 59×28, 30 · colour, options | devicon's archlinux-original.svg (MIT) | todo |
| centos | 56×28, 30 · colour, options | devicon's centos-original.svg (MIT) | todo |
| debian | 46×28, 30 · colour, options | devicon's debian-original.svg (MIT) | todo |
| deepin | 56×28, 30 · colour, options | Simple Icons' deepin.svg (CC0) | todo |
| elementary-os | 56×28, 30 · colour, options | Simple Icons' elementary.svg (CC0) | todo |
| endeavouros | 67×28, 30 · colour, options | Simple Icons' endeavouros.svg (CC0) | todo |
| fedora | 56×28, 30 · colour, options | devicon's fedora-original.svg (MIT) | todo |
| gentoo | 54×28, 30 · colour, options | devicon's gentoo-plain.svg (MIT) | todo |
| kali-linux | 71×27, 30 · colour, options | devicon's kalilinux-original.svg (MIT) | todo |
| linux-mint | 56×28, 30 · colour, options | devicon's linuxmint-original.svg (MIT) | todo |
| manjaro | 56×28, 30 · colour, options | Simple Icons' manjaro.svg (CC0) | todo |
| nixos | 64×28, 30 · colour, options | devicon's nixos-original.svg (MIT) | todo |
| omarchy | 56×28, 30 · colour, options | Simple Icons' omarchy.svg (MIT, from omarchy.org) | todo |
| opensuse | 69×18, 30 · colour, options | devicon's opensuse-original.svg (MIT) | todo |
| pop-os | 56×28, 30 · colour, options | Simple Icons' popos.svg (CC0) | todo |
| red-hat | 56×28, 30 · colour, options | devicon's redhat-original.svg (MIT) | todo |
| rocky-linux | 56×28, 30 · colour, options | devicon's rockylinux-original.svg (MIT) | todo |
| tux | 49×28, 30 · colour, options | devicon's linux-original.svg (MIT) | todo |
| ubuntu | 58×28, 30 · colour, options | Simple Icons' ubuntu.svg (CC0) | todo |
| void-linux | 56×28, 30 · colour, options | Simple Icons' voidlinux.svg (CC0) | todo |
| zorin-os | 64×28, 30 · colour, options | Simple Icons' zorin.svg (CC0) | todo |
