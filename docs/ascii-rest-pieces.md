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
| shapes | 12 | 11 | | | | 1 |
| space | 11 | 11 | | | | |
| physics | 14 | 14 | | | | |
| nature | 16 | 16 | | | | |
| creatures | 10 | 10 | | | | |
| objects | 14 | 14 | | | | |
| generative | 13 | 13 | | | | |
| effects | 8 | 6 | | | | 2 |
| ui | 12 | 11 | 1 | | | |
| data | 10 | 10 | | | | |
| type | 9 | 9 | | | | |
| logos | 28 | 28 | | | | |
| companies | 14 | 14 | | | | |
| distros | 23 | 23 | | | | |
| **all** | **209** | **205** | **1** | | | **3** |

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

`box-frames` counts as wip: it is ported and fixture-tested, but not yet
enhanced.

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

| # | what | needed by | status |
|---|---|---|---|
| E1 | **Two colours a cell.** A background colour per cell beside the foreground: `Grid`, the wasm `Player`, the canvas renderer (fill the cell, then draw the glyph), and the CLI (`48;2;r;g;b`). Half blocks (`▀` over a different `▄`) then give two square pixels a cell, each in its own colour. The `<pre>` path has one ink, so a piece using this needs an octant-dither fallback there. | the 15 scenes, sunrise, landscape; a filled plasma | todo |
| E2 | **Local time in `Env`.** `Env` has only `paper`. Clock pieces need the local wall time: from `Date` in the shell, the system clock in the CLI, and a fixed instant in the contract test so it stays deterministic. | analog-clock, calendar, digital-clock, landscape, sundial | todo |
| E3 | **Categories.** ascii.rest has `companies` and `distros`; `Category` stops at `Logos`. Add the two, or fold them into `logos`. The site's sidebar groups and the CLI's `list` follow. | the 37 company and distro logos | todo |
| E4 | **Logos from SVG.** A build script (`scripts/logo.py`, run with uvx like `font.py`) that renders each SVG and reduces it to an octant mask, choosing a wedge glyph (U+1FB3C–1FB67) where an edge cuts a cell at a slant, with a colour per cell. Its output is committed. One shared `logo` module draws any logo from that data and runs the glint or the scan line, so 65 logos cost data rather than 65 copies of the code. | logos, companies, distros | todo |
| E5 | **Size.** 64.6 KB gz with 11 pieces, about 2.8 KB more for each further piece. The 205 still to build would add some 575 KB at that rate, well past the 500 KB gate. E4 makes logos cheaper. Before wave 3, decide whether to raise the gate or to split pieces into several wasm modules ([risk 5](IMPLEMENTATION_PLAN.md#risks-and-open-questions)). | everything after wave 2 | todo |

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
   converted. E5 is decided before this wave.
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
| cube | 32×18, 24 | braille wireframe, hidden edges stippled | faces flat-shaded in octant dither between the quarter turns | todo |
| dna-helix | 64×15, 20 | braille strands, base pairs as ─ rungs | colour by base (A T C G), the far strand dimmed | todo |
| donut | 40×22, 30 | braille, z-buffer per dot, Bayer dither | — | covered: braille-donut |
| glxgears | 56×31, 24 | octant fill, braille rims | the red, green and blue gears of glxgears, lit faces | todo |
| gyroscope | 60×32, 30 | braille rings, depth-sorted per dot | a colour per gimbal; the rotor's spin shown by a moving stripe | todo |
| heart | 52×24, 30 | octant Bayer dither | red ramp with a specular highlight that rides the beat | todo |
| icosahedron | 46×22, 30 | braille wireframe, hidden edges dotted | faces flat-shaded in octant dither, edges kept as gaps | todo |
| mobius-strip | 58×20, 30 | braille, z-buffer per dot, dither | a colour gradient along the strip, so the one face reads as one | todo |
| spring | 40×28, 30 | braille coil, back coils dimmer | a shaded weight in octants and a shadow that grows as it nears | todo |
| tesseract | 54×28, 30 | braille lines, depth by dot density | edges coloured by their w coordinate | todo |
| torus-knot | 60×28, 30 | braille tube, z-buffer per dot, dither | a palette ramp for the lit tube | todo |
| twisted-ring | 46×21, 30 | braille, z-buffer per dot, dither | one colour running along the face, showing it is a single face | todo |

### space (11)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| black-hole | 64×20, 15 | octant disk, braille photon ring | temperature colours (white, orange, red), Doppler-bright side | todo |
| earth | 60×30, 20 | octant globe | ocean, land, ice and a dimmed night side in colour; a finer map if the size allows | todo |
| eclipse | 64×27, 20 | octant discs, braille corona rays | the corona and the diamond ring in colour | todo |
| galaxy | 64×26, 20 | braille stars, brightness by density | a yellow core shading to blue arms, a dark dust lane | todo |
| moon-phases | 46×23, 15 | octant dither on the disc | maria in grey; earthshine faintly on the dark side | todo |
| planet | 64×18, 20 | octant bands, braille ring | banded colours and the ring's shadow across the globe | todo |
| rocket | 48×28, 20 | octant body, quadrant exhaust (quadrant-fire's spread) | a coloured flame and a smoke cloud on the pad | todo |
| saptarishi | 64×18, 12 · options | stars as · • ● ✦ by magnitude | colour by spectral class; the asterism's lines fade in, in braille | todo |
| solar-system | 61×29, 20 · options | braille orbits as true ellipses, ● planets | a colour per planet; Kepler periods kept | todo |
| starfield | 64×24, 30 · options | braille streaks that lengthen with speed | warm-to-cold colour shift toward the edges | todo |
| three-body | 65×15, 30 | braille trails that thin with age | a colour per body | todo |

### physics (14)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| bouncing-balls | 64×22, 30 · options | octant balls, squash as ellipses | a colour per ball, shadows on the floor | todo |
| chladni | 45×23, 12 | braille grains, 8 a cell | the plate changing mode, tinted per mode | todo |
| double-pendulum | 48×25, 30 | braille arms and trail | the trail coloured by tip speed | todo |
| falling-sand | 55×22, 20 | the sand simulated per octant dot | grains in a few sand tones | todo |
| flag | 62×21, 30 · options | sextant cloth with a shading ramp | real stripes in colour, lit by the folds | todo |
| fountain | 57×22, 30 | braille drops | blue water, splash crowns in the basin | todo |
| harmonograph | 64×28, 30 | braille pen line (as braille-lissajous) | ink fading in colour along the line | todo |
| lorenz | 60×22, 30 | braille trajectory | coloured by age and by which wing it is on | todo |
| newtons-cradle | 74×17, 30 · options | octant balls, braille strings | steel shading with a moving highlight | todo |
| pendulum-wave | 67×19, 30 | octant bobs from above | colour by phase, so the waves show | todo |
| plucked-string | 64×14, 30 | braille string at 4× vertical resolution | the first harmonics drawn as faint coloured ghosts | todo |
| pond-ripples | 64×20, 20 | braille rings | water palette, raindrop splashes | todo |
| smoke | 44×24, 30 | braille density dither | grey thinning to nothing; a glowing incense tip | todo |
| wave-interference | 72×26, 24 · options | sextant field, braille nodal lines | crests and troughs in two colours | todo |

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
| epicycles | 60×24, 30 | braille circles and trace | a colour per circle | todo |
| flow-field | 64×22, 20 | braille trails, 8 particles a cell | colour by direction (a hue wheel) | todo |
| glider-gun | 56×20, 10 | one Life cell per octant dot | colour by cell age | todo |
| hilbert-curve | 65×17, 20 | a curve one order finer, in braille | the lit runners leave coloured trails | todo |
| julia-set | 66×24, 20 | octant dither on escape time | escape-time palette | todo |
| langtons-ant | 72×25, 30 | one grid square per octant dot | a coloured turmite variant as an option | todo |
| mandelbrot | 64×26, 15 | octant dither on escape time | smooth-coloured escape time | todo |
| maze | 61×21, 20 | a maze carved per braille dot | the solving path in colour | todo |
| plasma | 64×22, 24 | filled colour per cell | a full palette version; sextant-plasma is the contour one | todo |
| reaction-diffusion | 60×24, 20 | the field simulated per octant dot | a colour ramp for the two chemicals | todo |
| rule-30 | 64×20, 8 | one cell per braille dot | a colour per rule | todo |
| sierpinski | 59×26, 20 | braille points | colour by the vertex chosen | todo |
| voronoi | 64×22, 15 | octant region edges | a colour per region | todo |

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
| boot-log | 64×16, 15 | eighth blocks (already) | coloured status words, a braille spinner | todo |
| box-frames | 78×20, still | box drawing (already) | frames from Symbols for Legacy Computing (U+1FB00) | wip: ported and fixture-tested, not yet enhanced |
| calendar | 32×13, 8 · options, clock | box-drawn grid | today in colour | todo |
| digital-clock | 70×11, 10 · options, clock | octant segments with sloped ends | a lit and an unlit segment colour | todo |
| dividers | 63×25, still · options | box drawing, geometric shapes | ornaments from Symbols for Legacy Computing | todo |
| file-tree | 36×23, 15 · options | box drawing, ▸ ▾ markers | coloured folders and files | todo |
| form-controls | 54×16, 15 · options | ☐ ☑ ○ ◉ controls | the focused field in colour | todo |
| not-found | 60×21, 15 · options | octant block letters | an octant ghost, colour | todo |
| progress-bar | 48×11, 30 · options | eighth blocks | braille and gradient-coloured styles | todo |
| skeleton | 52×23, 24 · options | rounded box drawing, shade blocks | the shimmer as a colour gradient | todo |
| spinners | 66×13, 12 · options | braille spinners (⠋⠙⠹…) and the rest | twice the styles | todo |
| terminal | 62×18, 20 · options | box drawing | a coloured prompt and output | todo |

### data (10)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| bar-chart | 60×19, 24 · options | eighth blocks | a colour per series; horizontal bars as an option | todo |
| candlesticks | 70×22, 15 · options | half-block bodies, braille wicks | green up, red down | todo |
| cpu-meters | 66×16, 10 · options | eighth-block meters, a braille history graph | htop's colour segments | todo |
| equalizer | 62×16, 30 · options | eighth blocks | a green-to-red gradient | todo |
| gauge | 55×18, 30 · options | braille arc and needle | coloured zones | todo |
| heartbeat | 72×15, 25 · options | braille trace at 4× vertical resolution | a green phosphor glow | todo |
| heatmap | 66×12, 10 · options | ■ cells | GitHub's greens | todo |
| radar | 53×25, 30 · options | braille sweep | a green afterglow | todo |
| sparkline | 72×9, 12 · options | braille sparklines, two samples a cell | a colour per line | todo |
| uptime-bar | 64×15, 8 · options | eighth blocks | green, amber and red days | todo |

### type (9)

| piece | size, fps | Unicode | enhancement | status |
|---|---|---|---|---|
| big-text | 66×8, 24 · options | octant letterforms | the glint as a colour gradient | todo |
| dissolve | 64×13, 20 · options | braille dust | colour as the dust settles | todo |
| glitch | 64×13, 15 · options | octant pixel word | RGB channel split in the palette | todo |
| marquee | 61×10, 24 · options | a braille dot-matrix sign | coloured bulbs | todo |
| morse | 64×19, 15 · options | braille tape marks | the lamp's glow in colour | todo |
| scramble | 44×3, 20 · options | noise glyphs from the Extended ranges | letters settling into colour | todo |
| split-flap | 61×18, 15 · options | half-block flaps mid-turn | board colours | todo |
| typewriter | 44×3, 20 · options | text as it is, a choice of cursors | colour for the typed text | todo |
| wave-text | 36×5, 15 · options | text with eighth-block shadows | the swell in colour | todo |

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
