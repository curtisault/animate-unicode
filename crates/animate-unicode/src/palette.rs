//! Shading with colour: pick a palette index by brightness.
//!
//! The convention for a coloured piece: order each shading run in
//! `Meta::palette` dimmest first, so `palette[0]` is the dimmest colour and a
//! [`Ramp`] over the run maps brightness 0 to its first entry. Colour 0 is
//! also what `Grid::clear` writes, so blank cells get the dimmest colour.

/// A run of palette entries, dimmest first, picked by a brightness in [0, 1].
///
/// It plays the part for colours that [`crate::util::ramp`] plays for glyphs.
/// A piece with several shaded things (a sky and a sea, say) gives each its
/// own run of the palette and its own `Ramp`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ramp {
    first: u8,
    len: u8,
}

impl Ramp {
    /// The `len` palette entries starting at index `first`. Panics if `len` is
    /// 0 or the run passes index 255.
    pub const fn new(first: u8, len: u8) -> Self {
        assert!(len > 0, "a ramp needs at least one colour");
        assert!(first as u16 + len as u16 <= 256, "a ramp must stay within 256 palette entries");
        Self { first, len }
    }

    /// The whole of a palette, dimmest first.
    pub const fn over(palette: &[&str]) -> Self {
        assert!(!palette.is_empty() && palette.len() <= 256, "a ramp needs 1-256 colours");
        Self::new(0, palette.len() as u8)
    }

    /// The number of colours in the run.
    pub const fn len(&self) -> u8 {
        self.len
    }

    /// Always false: a ramp has at least one colour.
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// The palette index for a brightness in [0, 1], rounded to the nearest
    /// step and clamped, so 0 is the first entry and 1 the last.
    pub fn at(&self, v: f64) -> u8 {
        let steps = f64::from(self.len - 1);
        self.first + (v.clamp(0.0, 1.0) * steps + 0.5) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_maps_brightness_to_its_run() {
        let r = Ramp::new(4, 3);
        assert_eq!(r.at(0.0), 4);
        assert_eq!(r.at(0.5), 5);
        assert_eq!(r.at(1.0), 6);
        assert_eq!(r.at(-3.0), 4, "clamped below");
        assert_eq!(r.at(9.0), 6, "clamped above");
        assert_eq!(r.at(f64::NAN), 4, "NaN is the dimmest, never a panic");
    }

    #[test]
    fn ramp_over_a_palette_and_a_single_colour() {
        const P: &[&str] = &["#000000", "#808080", "#ffffff"];
        assert_eq!(Ramp::over(P), Ramp::new(0, 3));
        let one = Ramp::new(9, 1);
        assert_eq!((one.at(0.0), one.at(1.0)), (9, 9));
    }
}
