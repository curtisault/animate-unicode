//! kanji rain: the falling code of the films, in full-width katakana and
//! kanji, each glyph two cells wide. 24 streams fall at their own speeds, a
//! bright head and a trail that fades through the greens, their glyphs
//! changing now and then as they fall. The first `Charset::Wide` piece: every
//! glyph is drawn with `set_wide`, its right half a `WIDE_TAIL`.
//!
//! Wide glyphs come from the system's CJK fonts, so this is a canvas piece
//! first; in a `<pre>` the columns line up only where the font's CJK is
//! exactly two cells wide.

use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// The kanji rain's meta.
pub static META: Meta = Meta {
    name: "kanji rain",
    slug: "kanji-rain",
    category: Category::Effects,
    note: "falling streams of full-width katakana and kanji, two cells a glyph",
    cols: 48,
    rows: 20,
    fps: 20,
    charset: Charset::Wide,
    // Dimmest first; the last is the stream's head.
    palette: Some(&["#0a2614", "#0e3a1d", "#134f27", "#196833", "#20833f", "#29a24d", "#3cc161", "#79de8f", "#dcffe6"]),
    ground: Some("#040a06"),
    options: Some(r#"{"speed": 1.0}"#),
    ..Meta::DEFAULT
};

/// Full-width katakana and kanji: every one East Asian Width W.
const GLYPHS: &[char] = &[
    'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ', 'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ハ', 'ヒ',
    'フ', 'ヘ', 'ホ', 'マ', 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ', 'ワ', 'ン', '一', '二', '三', '四', '五', '六', '七', '八',
    '九', '十', '日', '月', '火', '水', '木', '金', '土', '雨', '光', '夜',
];
const STREAMS: usize = 24;

/// The rain; only its speed is state, the streams come from a hash.
pub struct KanjiRain {
    speed: f64,
}

/// A kanji rain; reads `speed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(KanjiRain { speed: options.f64("speed", 1.0) })
}

/// A fixed fraction in [0, 1) for a stream and a salt.
fn hash(i: usize, salt: u32) -> f64 {
    let mut h = (i as u32).wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

impl Piece for KanjiRain {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let t = t * self.speed;
        let rows = META.rows as f64;
        let top = (META.palette.unwrap().len() - 1) as f64;
        grid.clear();
        for i in 0..STREAMS {
            let speed = 5.0 + 9.0 * hash(i, 1); // cells a second
            let len = 5.0 + (hash(i, 2) * 11.0).floor();
            let period = rows + len + (hash(i, 3) * rows).floor();
            let head = (t * speed + hash(i, 4) * period).rem_euclid(period) - len.min(4.0);
            // How often this stream's glyphs change, per second.
            let churn = 1.5 + 4.0 * hash(i, 5);
            for k in 0..len as usize {
                let y = (head - k as f64).floor();
                if y < 0.0 || y >= rows {
                    continue;
                }
                let age = k as f64 / len;
                let colour = if k == 0 { top } else { ((1.0 - age) * (top - 1.0)).round().max(0.0) };
                let tick = (t * churn + y * 0.37).floor() as u32;
                let g = GLYPHS[(hash(i * 64 + y as usize, tick) * GLYPHS.len() as f64) as usize];
                grid.set_wide((2 * i) as i64, y as i64, g, colour as u8);
            }
        }
    }
}
