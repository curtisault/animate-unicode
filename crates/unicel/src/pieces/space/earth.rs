//! earth: the globe turning west to east under a sun low on its left, its
//! continents read off a hand-made map of the world at five degrees a cell.
//! After ascii.rest's earth.ts (MIT, @bas3line).
//!
//! The map, the turn and the light are ascii.rest's. The globe is drawn in
//! octants, one sample a dot, with the map blended between its cells so the
//! coasts run smooth at eight times the detail. Land is dense and the sea a
//! light texture, so they part in one ink; in colour the land greens in the
//! tropics and temperate belts, turns tan across the desert latitudes and
//! white at the poles, the sea is blue with the sun's glint on it, the night
//! side keeps its continents faintly, and a thin blue rim of air rings it.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// Earth's meta.
pub static META: Meta = Meta {
    name: "earth",
    slug: "earth",
    category: Category::Space,
    note: "an octant globe turning: green and desert land, ice, a blue sea, night",
    cols: 60,
    rows: 30,
    fps: 20,
    charset: Charset::Extended,
    // Sea: night, dim, lit, glint. Land: night, green, sunlit green, desert.
    // Ice, ice at night, the air's rim, a star.
    palette: Some(&["#0e2348", "#1d4f96", "#3a86d6", "#e4f2ff", "#1c3320", "#3d7a35", "#76aa4a", "#c8a65e", "#eef6fb", "#5c6c80", "#5aa8f4", "#8890a8"]),
    ground: Some("#030409"),
    ..Meta::DEFAULT
};

const SEA_NIGHT: u8 = 0;
const SEA: u8 = 1;
const SEA_LIT: u8 = 2;
const GLINT: u8 = 3;
const LAND_NIGHT: u8 = 4;
const GREEN: u8 = 5;
const GREEN_LIT: u8 = 6;
const DESERT: u8 = 7;
const ICE: u8 = 8;
const ICE_NIGHT: u8 = 9;
const AIR: u8 = 10;
const STAR: u8 = 11;

/// Equirectangular, 180 W at the left, the north pole at the top; # is land.
const MAP: [&str; 36] = [
    "                                                                        ",
    "                     ##   ######                                        ",
    "             ###################       #                #               ",
    "           ###########   #######                  ###########           ",
    "#  ##############   ###  #####         #################################",
    "   ##############    ##   ##    #    ### ############################## ",
    "    #    #########  ####           # ## ########################   ##   ",
    "          ###############         ## ###########################        ",
    "           #############           #############################        ",
    "           ###########            ### ####  ## ###############  #       ",
    "           ##########             ##     ################### #          ",
    "            ########              ##### #  #################  #         ",
    "             ####  #             ###########################            ",
    "               ##                ###############  #########             ",
    "                ###              ##############    ##  ###  #           ",
    "                  ##             #############     #    ##  #           ",
    "                    #####         ############          #   #           ",
    "                    ######            #######          ## ##            ",
    "                    ########          ######            # ### ##        ",
    "                    #########         ######             ##    ###      ",
    "                     ########          #####                  #         ",
    "                      ######          ###### #               ####       ",
    "                      ######           ####  #             #######      ",
    "                      ####             ####                ########     ",
    "                     ####              ###                 #######      ",
    "                     ###                                        ##      ",
    "                     ##                                               # ",
    "                     ##                                                 ",
    "                     #                                                  ",
    "                                                                        ",
    "                                                                        ",
    "                       #                       ##################       ",
    "                    ####        ######################################  ",
    "  ####################################################################  ",
    "########################################################################",
    "########################################################################",
];
const MW: usize = 72;
const MH: usize = 36;
/// Seconds per turn, the longitude facing us at t = 0 (degrees east), and how
/// far the north pole leans toward us (radians).
const DAY: f64 = 36.0;
const START: f64 = 20.0;
const TILT: f64 = 0.3;

/// A dot on the globe: its longitude and map row, its latitude in degrees,
/// the sun it gets, the glint the sea there would mirror, and whether it is on
/// the rim.
struct Spot {
    lon: f64,
    row: f64,
    lat: f64,
    sun: f64,
    glint: f64,
    rim: bool,
}

/// The globe: its map, every dot on it (by dot index), the stars, and the dots.
pub struct Earth {
    land: Vec<f64>,
    spots: Vec<(usize, Spot)>,
    stars: Vec<usize>,
    dots: ColorOctants,
}

impl Earth {
    /// How much land is at a point of the map, blended between cells.
    fn at(&self, u: f64, v: f64) -> f64 {
        let (x, y) = (u - 0.5, (v - 0.5).clamp(0.0, (MH - 1) as f64));
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (y0, y1) = (y0 as usize, (y0 as usize + 1).min(MH - 1));
        let a = (x0 as i64).rem_euclid(MW as i64) as usize;
        let b = (a + 1) % MW;
        let top = self.land[y0 * MW + a] * (1.0 - fx) + self.land[y0 * MW + b] * fx;
        let bot = self.land[y1 * MW + a] * (1.0 - fx) + self.land[y1 * MW + b] * fx;
        top * (1.0 - fy) + bot * fy
    }
}

/// Earth; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let land = MAP.iter().flat_map(|r| r.bytes().map(|b| f64::from(u8::from(b == b'#')))).collect();
    let dots = ColorOctants::new(META.cols, META.rows);
    let (w, h) = (dots.width(), dots.height());
    let (cx, cy) = (META.cols as f64 / 2.0, META.rows as f64 / 2.0);
    let r0 = META.rows as f64 / 2.0 - 1.0; // radius in rows; it spans twice as many columns
                                           // The sun is behind us and a little left and above, so the whole disc is
                                           // lit but for a thin crescent of night on the right.
    let m = (0.42f64 * 0.42 + 0.28 * 0.28 + 0.86 * 0.86).sqrt();
    let (lx, ly, lz) = (-0.42 / m, 0.28 / m, 0.86 / m);
    let hm = (lx * lx + ly * ly + (lz + 1.0) * (lz + 1.0)).sqrt();
    let (hx, hy, hz) = (lx / hm, ly / hm, (lz + 1.0) / hm);
    let (st, ct) = TILT.sin_cos();
    let mut spots = vec![];
    let mut stars = vec![];
    let mut rand = Mulberry32::new(5);
    for dy in 0..h {
        let y = (cy - (dy as f64 + 0.5) / 4.0) / r0;
        for dx in 0..w {
            let x = ((dx as f64 + 0.5) / 2.0 - cx) / (2.0 * r0);
            let d = x * x + y * y;
            if d >= 1.0 {
                if d.sqrt() > 1.1 && rand.next_f64() < 0.004 {
                    stars.push(dy * w + dx);
                }
                continue;
            }
            let z = (1.0 - d).sqrt();
            let (py, pz) = (y * ct + z * st, z * ct - y * st);
            let lat = py.asin();
            spots.push((
                dy * w + dx,
                Spot {
                    lon: f64::atan2(x, pz),
                    row: (FRAC_PI_2 - lat) / PI * MH as f64,
                    lat: lat.to_degrees(),
                    sun: (x * lx + y * ly + z * lz).max(0.0),
                    glint: (x * hx + y * hy + z * hz).max(0.0).powi(300),
                    rim: d > 0.955,
                },
            ));
        }
    }
    Box::new(Earth { land, spots, stars, dots })
}

/// A fixed scatter of wave crests on the sea, so the texture turns with the globe.
fn crest(u: f64, v: f64) -> bool {
    let h = ((u * 4.0).floor() * 12.9898 + (v * 4.0).floor() * 78.233).sin() * 43_758.545_3;
    h - h.floor() < 0.07
}

impl Piece for Earth {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let turn = (START - 360.0 * t.rem_euclid(DAY) / DAY).to_radians();
        let w = self.dots.width();
        self.dots.clear();
        for &k in &self.stars {
            self.dots.set((k % w) as i64, (k / w) as i64, STAR);
        }
        for (k, s) in &self.spots {
            let (x, y) = (k % w, k / w);
            let u = (((s.lon + turn) / TAU + 0.5) * MW as f64).rem_euclid(MW as f64);
            let v = s.sun;
            // On paper a dot's ink is its shadow: land darkest at the dusk.
            let light = |v: f64| if env.paper { 1.0 - v } else { v };
            let pick = if self.at(u, s.row) >= 0.5 {
                let polar = s.lat.abs() > 62.0;
                if v < 0.03 {
                    // Land stays faintly through the night, so the globe reads whole.
                    dither(x, y, 0.22).then_some(if polar { ICE_NIGHT } else { LAND_NIGHT })
                } else if dither(x, y, 0.55 + 0.45 * light(v)) {
                    Some(if polar {
                        ICE
                    } else if (15.0..35.0).contains(&s.lat.abs()) {
                        DESERT
                    } else if v > 0.6 {
                        GREEN_LIT
                    } else {
                        GREEN
                    })
                } else {
                    None
                }
            } else if v < 0.1 {
                // The sea goes dark before the land does.
                None
            } else if s.glint > 0.4 {
                Some(GLINT)
            } else if v > 0.5 && (s.glint > 0.08 || crest(u, s.row)) {
                Some(SEA_LIT)
            } else {
                dither(x, y, 0.12 + 0.3 * light(v)).then_some(if v > 0.5 { SEA } else { SEA_NIGHT })
            };
            if let Some(color) = pick {
                self.dots.set(x as i64, y as i64, color);
            } else if s.rim {
                // The air, a thin ring round the whole globe, unbroken through the night.
                self.dots.set(x as i64, y as i64, AIR);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
