//! ascii.rest's scene noise (MIT, @bas3line): an integer hash, value noise
//! smoothed between lattice points, and fractal sums of it. Every one of its
//! scenes carries the same three functions, so the ports share these, which
//! give the same numbers as the TypeScript (fixture-tested below).

/// A hash of two integers to 0–1. Each argument is truncated to a 32-bit
/// integer first, as `x | 0` does in JS.
#[inline]
pub fn hash(x: f64, y: f64) -> f64 {
    let (x, y) = (x as i64 as i32, y as i64 as i32);
    let h = x.wrapping_mul(374_761_393).wrapping_add(y.wrapping_mul(668_265_263));
    let h = (h ^ ((h as u32) >> 13) as i32).wrapping_mul(1_274_126_177);
    ((h ^ ((h as u32) >> 16) as i32) as u32) as f64 / 4_294_967_296.0
}

/// Value noise at (x, y): the hashes of the four lattice points round it,
/// blended on a smoothstep.
#[inline]
pub fn noise(x: f64, y: f64) -> f64 {
    let (xi, yi) = (x.floor(), y.floor());
    lattice(xi, xi + 1.0, yi, x - xi, y - yi)
}

/// Value noise that repeats every `period` lattice cells in x (0: never), so
/// a cloud deck can drift forever.
#[inline]
pub fn noise_wrap(x: f64, y: f64, period: f64) -> f64 {
    if period == 0.0 {
        return noise(x, y);
    }
    let (xi, yi) = (x.floor(), y.floor());
    let x0 = ((xi % period) + period) % period;
    lattice(x0, (x0 + 1.0) % period, yi, x - xi, y - yi)
}

#[inline]
fn lattice(x0: f64, x1: f64, yi: f64, fx: f64, fy: f64) -> f64 {
    let (u, v) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (a, b, c, d) = (hash(x0, yi), hash(x1, yi), hash(x0, yi + 1.0), hash(x1, yi + 1.0));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Fractal noise: `octaves` layers of [`noise`], each at twice the frequency
/// and half the weight of the last, normalised to 0–1.
pub fn fbm(x: f64, y: f64, octaves: u32) -> f64 {
    fbm_wrap(x, y, octaves, 0.0)
}

/// [`fbm`] of [`noise_wrap`]: repeats every `period` lattice cells of its
/// first octave (each octave's period doubling with it).
pub fn fbm_wrap(x: f64, y: f64, octaves: u32, period: f64) -> f64 {
    let (mut s, mut n, mut amp, mut f) = (0.0, 0.0, 0.5, 1.0);
    for _ in 0..octaves {
        s += amp * noise_wrap(x * f, y * f, period * f);
        n += amp;
        amp *= 0.5;
        f *= 2.0;
    }
    s / n
}

/// 0–1 clamped.
#[inline]
pub fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

/// Smoothstep from `a` to `b` (either way round, as ascii.rest's `smooth`).
#[inline]
pub fn smooth(a: f64, b: f64, v: f64) -> f64 {
    let k = clamp((v - a) / (b - a));
    k * k * (3.0 - 2.0 * k)
}

/// `a` to `b` by `k`.
#[inline]
pub fn mix(a: f64, b: f64, k: f64) -> f64 {
    a + (b - a) * k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_ascii_rests_scene_noise() {
        // From night-coast.ts's hash, noise and fbm, run in node.
        let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(close(hash(0.0, 0.0), HASH[0]) && close(hash(37.0, -5.0), HASH[1]) && close(hash(123_456.7, 89.2), HASH[2]));
        assert!(close(noise(3.3, 7.8), NOISE[0]) && close(noise(-12.25, 0.5), NOISE[1]));
        assert!(close(noise_wrap(641.5, 3.25, 5.12), NOISE[2]));
        assert!(close(fbm(1.7, 2.9, 4), FBM[0]) && close(fbm_wrap(9.1, 0.4, 5, 11.52), FBM[1]));
    }

    // node's seventeen digits, kept as it printed them
    #[allow(clippy::excessive_precision)]
    const HASH: [f64; 3] = [0.0, 0.142_944_156_657_904_39, 0.954_321_089_666_336_77];
    #[allow(clippy::excessive_precision)]
    const NOISE: [f64; 3] = [0.476_107_904_937_729_2, 0.533_749_567_759_514_32, 0.427_075_861_149_205_59];
    #[allow(clippy::excessive_precision)]
    const FBM: [f64; 2] = [0.544_709_370_322_223_44, 0.363_345_471_868_784_91];
}
