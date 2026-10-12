//! sunrise: the sun coming up out of the sea at dawn. The glow along the
//! horizon gathers into bands, the sun climbs out of it and sinks back as dusk,
//! and its path glitters on the water. After ascii.rest's sunrise.ts (MIT,
//! @bas3line).
//!
//! The sun's loop, the glow, the stars and the sea's crests and glints are
//! ascii.rest's. The sky and the sea are a picture of square pixels, two to a
//! cell ([`crate::pixels`]): the glow is a smooth gradient rather than bands of
//! glyphs, and it warms with the dawn, violet before the sun, rose, orange and
//! gold as it breaks the sea, the sky above turning from night to a morning
//! blue as it climbs. The disc is solid, its rim cut in octants, and the
//! crests and glints are braille over the water, the glints scattering afresh
//! a few times a second so the path glitters. In one ink there are no
//! pixels: the glow is octant dots in ascii.rest's bands, the disc solid
//! inside a dark rim, the horizon a line.

use crate::noise::{clamp, smooth};
use crate::pixels::{Pixels, Rgb};
use crate::subcell::{Braille, Encoding, Octant};
use crate::util::dither;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// Sunrise's meta.
pub static META: Meta = Meta {
    name: "sunrise",
    slug: "sunrise",
    category: Category::Nature,
    note: "the sun lifting off a sea horizon, its path glittering",
    cols: 60,
    rows: 20,
    fps: 15,
    charset: Charset::Extended,
    // The glints' white, a star, the ground and the disc's oranges and
    // cream; then, fitted to the colours the sky and the sea take over the
    // loop, dark to light: night blues, the dawn's violets, roses and mauves,
    // the morning's blues, the oranges and golds of the glow.
    palette: Some(&[
        "#fffaea", "#dfe6ff", "#04060e", "#ff7a34", "#ff9640", "#fff2c8", "#0a0d1b", "#111324", "#181a2e", "#19233a", "#2b212e", "#291e3f", "#232e45",
        "#3c2c35", "#3e294a", "#2d3850", "#513637", "#34415c", "#533452", "#474145", "#434b5f", "#63453d", "#344d76", "#6b3b57", "#5a4d56", "#4e556c",
        "#745645", "#3f5c89", "#83495b", "#695861", "#5c6072", "#43689f", "#876750", "#786769", "#686a7b", "#995a5f", "#5a6d92", "#4f74a8", "#757583",
        "#90776d", "#ad705a", "#617eaa", "#85808a", "#7189ae", "#a78775", "#ca7f5a", "#948e96", "#8296b5", "#ac9898", "#de925e", "#98a1b5", "#c89e84",
        "#b1aab0", "#f5a262", "#c9b1a6", "#f5b272", "#dfbb9c", "#f9c282", "#fbcd9b", "#ffdeb7",
    ]),
    ground: Some("#04060e"),
    ..Meta::DEFAULT
};

const COLS: usize = 60;
const ROWS: usize = 20;
/// Seconds from one dawn to the next.
const LOOP: f64 = 48.0;
/// The part of the loop the sun is up.
const DAY: f64 = 0.85;
/// The horizon row.
const HZ: usize = 12;
const HZF: f64 = HZ as f64;
/// The sun's radius, in rows.
const R: f64 = 2.6;
/// The sun's highest, in rows above the horizon.
const TOP: f64 = 6.6;
/// The middle of its arc and how far it travels each way.
const CX: f64 = 30.0;
const WX: f64 = 8.0;
/// Seconds into the loop at t = 0, the sun a third up.
const START: f64 = 2.2;
/// The dark rim between the disc and its corona, in rows.
const GAP: f64 = 0.32;

/// ascii.rest's sunrise hash of three integers to 0–1 (each argument
/// truncated to 32 bits, as `Math.imul` does).
fn hash(a: f64, b: f64, c: f64) -> f64 {
    let i = |v: f64| v as i64 as i32;
    let mut h = i(a).wrapping_mul(0x27d4_eb2d) ^ i(b).wrapping_mul(0x1656_67b1) ^ i(c).wrapping_mul(0x2545_f491);
    h = (h ^ ((h as u32) >> 15) as i32).wrapping_mul(0x85eb_ca6b_u32 as i32);
    h = (h ^ ((h as u32) >> 13) as i32).wrapping_mul(0xc2b2_ae35_u32 as i32);
    ((h ^ ((h as u32) >> 16) as i32) as u32) as f64 / 4_294_967_296.0
}

/// Value noise along x for row r, sliding with z.
fn noise(x: f64, r: f64, z: f64) -> f64 {
    let (i, j) = (x.floor(), z.floor());
    let (u, v) = (smooth(0.0, 1.0, x - i), smooth(0.0, 1.0, z - j));
    let n = |a: f64, b: f64| hash(i + a, r, j + b);
    (n(0.0, 0.0) * (1.0 - u) + n(1.0, 0.0) * u) * (1.0 - v) + (n(0.0, 1.0) * (1.0 - u) + n(1.0, 1.0) * u) * v
}

/// Where the sun is a fraction s through the loop, as its column and height
/// above the horizon: up in an arc for the day, then a short dip under the
/// sea, its speed matched where the two meet.
fn sun(s: f64) -> (f64, f64) {
    if s < DAY {
        let p = PI * (s / DAY);
        return (CX - WX * p.cos(), -R + (TOP + R) * p.sin());
    }
    let q = PI * ((s - DAY) / (1.0 - DAY));
    (CX + WX * q.cos(), -R - (TOP + R) * ((1.0 - DAY) / DAY) * q.sin())
}

/// The light at a moment: the sun's place and how the glow lies.
struct Light {
    t: f64,
    sx: f64,
    sy: f64,
    h: f64,
    glow: f64,
    height: f64,
    night: f64,
    /// How much of the disc is above the sea.
    lit: f64,
    /// 0 before dawn to 1 with the sun at its highest.
    day: f64,
}

fn light(t: f64) -> Light {
    let u = (t + START).rem_euclid(LOOP);
    let (sx, h) = sun(u / LOOP);
    Light {
        t,
        sx,
        sy: HZF - h,
        h,
        // Broad and dim before dawn, strongest as the sun breaks the sea, and
        // drawn down into tighter bands as it climbs.
        glow: 0.72 + 0.28 * smooth(-4.0, 0.0, h) - 0.25 * smooth(1.0, TOP, h),
        height: 3.8 - 1.9 * smooth(0.0, TOP, h),
        night: 1.0 - smooth(-3.0, 3.0, h),
        lit: clamp((h + R) / (2.0 * R)),
        day: smooth(-1.5, TOP, h),
    }
}

impl Light {
    /// How far (x, y), in columns and rows, is outside the disc, in rows.
    fn edge(&self, x: f64, y: f64) -> f64 {
        ((x - self.sx) / 2.0).hypot(y - self.sy) - R
    }

    /// Whether (x, y) is on the disc, above the sea.
    fn disc(&self, x: f64, y: f64) -> bool {
        y < HZF && self.edge(x, y) < 0.0
    }

    /// The glow at (x, y) outside the disc: bands that bow up toward the sun,
    /// and a soft corona.
    fn glow_at(&self, x: f64, y: f64) -> f64 {
        let d = self.edge(x, y);
        let bow = 0.62 + 0.38 * (-((x - self.sx) / 17.0).powi(2)).exp();
        (self.glow * (-(HZF - y) / self.height).exp() * bow).max(0.42 * self.lit * (-(d - GAP) / 1.5).exp())
    }

    /// How bright the sun's path is on sea row r.
    fn bright(&self, k: f64) -> f64 {
        (0.25 + 0.75 * self.lit) * (1.05 - k * 0.05) * (0.4 + 0.6 * self.glow)
    }
}

/// What ascii.rest draws in a sky cell.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Sky {
    /// The disc, `@`.
    Disc,
    /// Near the disc, left dark so it keeps a clean edge.
    Rim,
    /// A glow band, 0 (nothing) to 5.
    Band(usize),
}

/// The glow bands, faint to strong.
#[cfg(test)]
const BANDS: [char; 6] = [' ', '.', '·', ':', '-', '='];

/// A sky cell as ascii.rest decides it: the disc sampled 3 by 4 in the cell,
/// so the horizon cuts it flat; a cell at least half covered is sun, and one
/// that comes near it is left dark.
fn sky_cell(l: &Light, r: usize, c: usize) -> Sky {
    let (mut cov, mut near) = (0, 0);
    for i in 0..4 {
        for j in 0..3 {
            let yy = r as f64 + (f64::from(i) + 0.5) / 4.0;
            let e = (c as f64 + (f64::from(j) + 0.5) / 3.0 - l.sx) / 2.0;
            let e = e.hypot(yy - l.sy) - R;
            if yy < HZF && e < 0.0 {
                cov += 1;
            }
            if e < GAP {
                near += 1;
            }
        }
    }
    if cov >= 6 {
        return Sky::Disc;
    }
    if near > 0 && l.lit > 0.0 {
        return Sky::Rim;
    }
    let v = l.glow_at(c as f64 + 0.5, r as f64 + 0.5);
    Sky::Band(((v * 6.0).floor().max(0.0) as usize).min(5))
}

/// What ascii.rest draws on a sea cell.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Sea {
    Calm,
    /// A crest, `-` out by the horizon and `~` nearer.
    Crest,
    /// A faint glint, `·`.
    Dot,
    /// A glint or a crest in the sun's path, `-`.
    Dash,
    /// A bright glint, `=`.
    Bright,
}

/// A sea row (below the horizon) as ascii.rest draws it, in perspective:
/// short close-set crests out by the horizon, longer and sparser swells
/// nearer, each run slowly lengthening and shrinking. Under the sun its path
/// breaks into glints that come and go.
fn sea_row(l: &Light, r: usize) -> [Sea; COLS] {
    let t = l.t;
    let k = (r - HZ) as f64;
    let scale = 0.6 * k.powf(1.2);
    let cut = 0.43 + 0.028 * k;
    let drift = t * (0.15 + 0.06 * k);
    let wide = 2.4 + k * 1.2;
    let bright = l.bright(k);
    let mut run = 0.0;
    let mut out = [Sea::Calm; COLS];
    for (c, cell) in out.iter_mut().enumerate() {
        let x = c as f64 + 0.5;
        // A crest, unless it has already run longer than a wave this far out.
        let mut wave = noise((x + drift) / scale, r as f64, t * 0.22 + k * 3.1) > cut;
        run = if wave { run + 1.0 } else { 0.0 };
        if run > (1.4 + 1.2 * k) * (0.7 + 0.6 * hash((x + drift).floor(), r as f64, 9.0)) {
            wave = false;
            run = 0.0;
        }
        let path = (-((x - l.sx) / wide).powi(2)).exp() * bright;
        let glint = noise(x * (1.3 / k.sqrt()), r as f64 + 40.0, t * 2.2 + k) * path;
        *cell = if glint > 0.45 {
            Sea::Bright
        } else if glint > 0.3 || (wave && path > 0.3) {
            Sea::Dash
        } else if glint > 0.2 {
            Sea::Dot
        } else if wave {
            Sea::Crest
        } else {
            Sea::Calm
        };
    }
    out
}

/// The stars, as column, row and a phase, ascii.rest's.
fn stars() -> Vec<(usize, usize, f64)> {
    (0..22).map(f64::from).map(|i| ((hash(i, 1.0, 7.0) * COLS as f64) as usize, (hash(i, 2.0, 7.0) * 7.0) as usize, hash(i, 3.0, 7.0))).collect()
}

/// A star's glyph if it shows: twinkling, and out only while it is dark.
fn star(l: &Light, (_, r, p): (usize, usize, f64)) -> Option<char> {
    let tw = 0.5 + 0.5 * (l.t * (1.1 + p) + p * 40.0).sin();
    ((0.25 + l.night) * tw > 0.6 + r as f64 * 0.04).then_some(if p < 0.15 {
        '+'
    } else if p < 0.55 {
        '·'
    } else {
        '.'
    })
}

/// A colour from stops along 0–1.
fn ramp(stops: &[(f64, Rgb)], v: f64) -> Rgb {
    let v = v.clamp(0.0, 1.0) as f32;
    for w in stops.windows(2) {
        let ((a, ca), (b, cb)) = (w[0], w[1]);
        let (a, b) = (a as f32, b as f32);
        if v <= b {
            let k = ((v - a) / (b - a)).clamp(0.0, 1.0);
            return [ca[0] + (cb[0] - ca[0]) * k, ca[1] + (cb[1] - ca[1]) * k, ca[2] + (cb[2] - ca[2]) * k];
        }
    }
    stops[stops.len() - 1].1
}

fn lerp(a: Rgb, b: Rgb, k: f64) -> Rgb {
    let k = k.clamp(0.0, 1.0) as f32;
    [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
}

fn rgb(n: u32) -> Rgb {
    [((n >> 16) & 255) as f32 / 255.0, ((n >> 8) & 255) as f32 / 255.0, (n & 255) as f32 / 255.0]
}

/// The glow's colour for a strength v: violet where it is faint, through
/// rose and orange to gold, paler as the sun climbs.
fn warm(l: &Light, v: f64) -> Rgb {
    let c = ramp(&[(0.0, rgb(0x2a1e4a)), (0.22, rgb(0x5a2e66)), (0.42, rgb(0xb4526a)), (0.6, rgb(0xec8c50)), (0.8, rgb(0xfabc60)), (1.0, rgb(0xffe4a0))], v);
    lerp(c, rgb(0xfde8b8), 0.7 * l.day)
}

/// The sky at (x, y), without the disc: night to morning blue overhead,
/// warmed by the glow.
fn sky_rgb(l: &Light, x: f64, y: f64) -> Rgb {
    let zenith = lerp(rgb(0x060a1a), rgb(0x3f6aa8), l.day);
    let low = lerp(rgb(0x161a3a), rgb(0x92b2d6), l.day);
    let base = lerp(zenith, low, (y / HZF).powf(1.6));
    let v = l.glow_at(x, y);
    lerp(base, warm(l, v), clamp(v * 1.5))
}

/// The disc: deep orange as it breaks the sea, near white at its highest,
/// a little darker at the limb.
fn sun_rgb(l: &Light, x: f64, y: f64) -> Rgb {
    let c = lerp(rgb(0xff7a34), rgb(0xfff2c8), smooth(-R, TOP * 0.85, l.h));
    let limb = clamp(-l.edge(x, y) / R);
    lerp(lerp(c, rgb(0xff9640), 0.35 * (1.0 - limb) * (1.0 - l.day)), rgb(0xfffaea), 0.25 * limb)
}

/// The sea at (x, y): the far water catching the sky, darker toward us,
/// with the glow on it under the horizon and the sun's path laid soft.
fn sea_rgb(l: &Light, x: f64, y: f64) -> Rgb {
    let k = (y - HZF).max(0.0);
    let f = k / (ROWS - HZ) as f64;
    let far = lerp(rgb(0x121a34), rgb(0x3e5a7c), l.day);
    let near = lerp(rgb(0x030610), rgb(0x0e1c34), l.day);
    let mut c = lerp(far, near, f.powf(0.6));
    let hz = l.glow * (-k / 1.4).exp();
    c = lerp(c, warm(l, l.glow * 0.8), 0.45 * hz);
    let wide = 2.4 + k * 1.2;
    let path = (-((x - l.sx) / wide).powi(2)).exp() * l.bright(k.max(1.0)) * smooth(-R, 0.0, l.h);
    lerp(c, sun_rgb(l, l.sx, l.sy), 0.3 * path)
}

/// The braille glyph for a sea cell: crests as lines (a zigzag nearer), the
/// glints as dots scattered afresh a few times a second.
fn sea_glyph(m: Sea, c: usize, r: usize, t: f64) -> char {
    let k = r - HZ;
    let seed = (t * 7.5).floor();
    let dots = |p: f64, rows: u8| {
        let mut mask = 0u8;
        for d in 0..8u8 {
            if rows & (1 << (d / 2)) != 0 && hash(c as f64 * 8.0 + f64::from(d), r as f64, seed) < p {
                mask |= 1 << d;
            }
        }
        mask
    };
    let mask = match m {
        Sea::Calm => 0,
        Sea::Crest if k > 3 => {
            if c.is_multiple_of(2) {
                (1 << 4) | (1 << 3)
            } else {
                (1 << 2) | (1 << 5)
            }
        }
        Sea::Crest => (1 << 2) | (1 << 3),
        Sea::Dot => 1 << (hash(c as f64, r as f64 + 3.0, seed) * 8.0) as u8,
        Sea::Dash => dots(0.75, 0b0110) | 1 << (2 + (hash(c as f64, r as f64 + 5.0, seed) * 4.0) as u8),
        Sea::Bright => dots(0.8, 0b1111) | (1 << 2) | (1 << 5),
    };
    Braille::glyph(mask)
}

/// A star's glyph here: `+` as a bullet, `·` as is, `.` as one braille dot.
fn star_glyph(ch: char) -> char {
    match ch {
        '+' => '•',
        '·' => '·',
        _ => '⠂',
    }
}

/// The sunrise: its stars, and the picture.
pub struct Sunrise {
    stars: Vec<(usize, usize, f64)>,
    pixels: Pixels,
}

/// A sunrise; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Sunrise { stars: stars(), pixels: Pixels::new(&META) })
}

/// The 8 dots of cell (c, r), as (x, y) in columns and rows.
fn subdots(c: usize, r: usize) -> impl Iterator<Item = (u8, f64, f64)> {
    (0..8u8).map(move |d| (d, c as f64 + (f64::from(d % 2) + 0.5) / 2.0, r as f64 + (f64::from(d / 2) + 0.5) / 4.0))
}

impl Piece for Sunrise {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let l = light(t);
        if env.mono {
            // One ink: the glow in ascii.rest's bands as dots, the disc solid
            // inside its dark rim, the horizon a line.
            grid.clear();
            for r in 0..HZ {
                for c in 0..COLS {
                    let mut mask = 0u8;
                    for (d, x, y) in subdots(c, r) {
                        let on = if l.disc(x, y) {
                            true
                        } else if l.edge(x, y) < GAP && l.lit > 0.0 {
                            false
                        } else {
                            let band = (l.glow_at(x, y) * 6.0).floor().clamp(0.0, 5.0);
                            dither(2 * c + usize::from(d % 2), 4 * r + usize::from(d / 2), band * 0.085)
                        };
                        if on {
                            mask |= 1 << d;
                        }
                    }
                    if mask != 0 {
                        grid.set(c as i64, r as i64, Octant::glyph(mask), 0);
                    }
                }
            }
            for c in 0..COLS {
                grid.set(c as i64, HZ as i64, '▔', 0);
            }
        } else {
            for py in 0..2 * ROWS {
                let y = (py as f64 + 0.5) / 2.0;
                for px in 0..COLS {
                    let x = px as f64 + 0.5;
                    let c = if py < 2 * HZ {
                        if l.disc(x, y) {
                            sun_rgb(&l, x, y)
                        } else {
                            sky_rgb(&l, x, y)
                        }
                    } else if py == 2 * HZ {
                        // the horizon's haze
                        lerp(sea_rgb(&l, x, y), sky_rgb(&l, x, HZF - 0.25), 0.5)
                    } else {
                        sea_rgb(&l, x, y)
                    };
                    self.pixels.set(px, py, c);
                }
            }
            self.pixels.draw(grid, env);
            // The disc's rim in octants, over the sky behind it.
            for r in 0..HZ {
                for c in 0..COLS {
                    let mask = subdots(c, r).filter(|&(_, x, y)| l.disc(x, y)).fold(0u8, |m, (d, _, _)| m | 1 << d);
                    if mask != 0 && mask != 255 {
                        let (x, y) = (c as f64 + 0.5, r as f64 + 0.5);
                        let ink = self.pixels.nearest(sun_rgb(&l, x, y));
                        let back = self.pixels.nearest(sky_rgb(&l, x, y));
                        grid.set(c as i64, r as i64, Octant::glyph(mask), ink);
                        grid.set_back(c as i64, r as i64, back);
                    }
                }
            }
        }
        // Stars where ascii.rest leaves them: not on the disc, nor under a glow band.
        for &s in &self.stars {
            let (c, r, _) = s;
            let Some(ch) = star(&l, s) else { continue };
            if matches!(sky_cell(&l, r, c), Sky::Disc | Sky::Band(1..)) {
                continue;
            }
            let ink = self.pixels.nearest(rgb(0xdfe6ff));
            grid.set(c as i64, r as i64, star_glyph(ch), ink);
            if !env.mono {
                let back = self.pixels.nearest(sky_rgb(&l, c as f64 + 0.5, r as f64 + 0.5));
                grid.set_back(c as i64, r as i64, back);
            }
        }
        // The sea's crests and glints in braille.
        let glint = sun_rgb(&l, l.sx, l.sy);
        for r in HZ + 1..ROWS {
            for (c, &m) in sea_row(&l, r).iter().enumerate() {
                if m == Sea::Calm {
                    continue;
                }
                let (x, y) = (c as f64 + 0.5, r as f64 + 0.5);
                let water = sea_rgb(&l, x, y);
                let colour = match m {
                    Sea::Bright => lerp(glint, rgb(0xfffaea), 0.5),
                    Sea::Dash => glint,
                    Sea::Dot => lerp(glint, water, 0.3),
                    _ => lerp(water, sky_rgb(&l, x, HZF - 1.0), 0.4 + 0.1 * l.day),
                };
                let ink = self.pixels.nearest(colour);
                grid.set(c as i64, r as i64, sea_glyph(m, c, r, t), ink);
                if !env.mono {
                    grid.set_back(c as i64, r as i64, self.pixels.nearest(water));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The frame at t as ascii.rest draws it, from the ported state.
    #[allow(clippy::needless_range_loop)] // the index reads several arrays at once
    fn ascii(t: f64) -> String {
        let l = light(t);
        let mut g = vec![vec![' '; COLS]; ROWS];
        for s in stars() {
            if let Some(ch) = star(&l, s) {
                g[s.1][s.0] = ch;
            }
        }
        for r in 0..HZ {
            for c in 0..COLS {
                match sky_cell(&l, r, c) {
                    Sky::Disc => g[r][c] = '@',
                    Sky::Rim | Sky::Band(0) => {}
                    Sky::Band(k) => g[r][c] = BANDS[k],
                }
            }
        }
        g[HZ].fill('▔');
        for r in HZ + 1..ROWS {
            for (c, m) in sea_row(&l, r).into_iter().enumerate() {
                g[r][c] = match m {
                    Sea::Calm => ' ',
                    Sea::Crest => {
                        if r - HZ > 3 {
                            '~'
                        } else {
                            '-'
                        }
                    }
                    Sea::Dot => '·',
                    Sea::Dash => '-',
                    Sea::Bright => '=',
                };
            }
        }
        g.iter().map(|row| row.iter().collect::<String>()).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn matches_ascii_rests_frames() {
        // sunrise.ts's frames, from node, at t = 0, 18 and 42 (fps 15).
        for (t, want) in [(0.0, T0), (18.0, T18), (42.0, T42)] {
            assert_eq!(ascii(t), want.trim_matches('\n'), "t = {t}");
        }
    }

    const T0: &str = r#"
                                                            
  .                                  ·                      
                  .      .                    .    .        
                                        .                   
                                                          · 
                 ..........                                 
    ....................................                    
............................................................
............·····················...........................
··············································..............
······::::::::::::   @@@  ::::::::::::······················
:::::::---------  @@@@@@@@  ---------:::::::::::::::::::::::
▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔
-  -- -- - ---    -·   ·· -  -   -    -- -   --     -  --  -
  ---  ---   -   ---·· · ----  -      -   -  ---   --       
     ---          ·----   · --   ---   --- --- -            
    ~~~~~~    ~~~~- -···    ~~      ~~~~~~   ~~~~~ ~~~~~    
                  ·· ·---·          ~~       ~           ~~~
~~                     ·--~~~~~        ~~~~                 
    ~~~~~       ·  ·· ---~~~~~ ~~~                          
"#;
    const T18: &str = r#"
                                                            
                          ........                          
                        ...      ...                        
                      ... @@@@@@@  ...                      
                     ... @@@@@@@@@@ ..                      
                     ..· @@@@@@@@@@ ..                      
                      .. @@@@@@@@@@ ..                      
                      ...  @@@@@@  ..                       
                        ...     ....                        
                 ..........................                 
..........................········..........................
···················::::::::::::::::::::::···················
▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔
- -- -    --  ---  ---  -  -==-  -       -   -- - - --     -
-  -  -    ---- ----  ---·--·-·=· ·--  -      -- --    ---  
---      -----          ------=--·    ---    --  --  ---  --
  ~~~~ ~~~~~ ~~~~       ---------=· ·            ~~~~~~ ~~  
~~~~~ ~               ·  ------·---·   ~~~~~        ~~~~~~ ~
                    ~~----=--==----····                     
~~~~~~~~~            ··-----------··-·                      
"#;
    const T42: &str = r#"
                                                            
  .                                  ·     ·                
                                .             .    .        
                                        .                   
                                             +     .        
        ·                                                   
                      .    .......                          
             ...................................            
............................................................
.....................···················....................
........·············································.......
················::::::::::::::::::::::::::::::··············
▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔
  -  --      -- -  -  -  --  ---    - -- ---  -  -- -- -  --
-             --        -  ---   --- ---- ---  -- ---       
 ------ ---- -   ------ ---   ---- --- --  ----     --      
     ~~~~~~  ~~~~      ~~~~~     ~~             ~~~~        
~~~~~ ~~~~~ ~  ~~~~~~                     ~~~~~~~~ ~~~~~~   
                                          ~~~~~~          ~~
~~~~~~~ ~~~~~~~~~~ ~~~~~   ~~~~~~~~~    ~~~~~~~             
"#;
}
