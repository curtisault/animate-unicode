//! A little 3D: vectors as `[f64; 3]` and rotation matrices, for pieces that
//! turn solids. Projection stays in each piece, since each frames its own
//! picture.

/// A point or direction in 3D.
pub type V3 = [f64; 3];

/// The dot product.
pub fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The cross product.
pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// The vector scaled to length 1 (the zero vector stays zero).
pub fn unit(a: V3) -> V3 {
    let m = dot(a, a).sqrt();
    if m == 0.0 {
        a
    } else {
        [a[0] / m, a[1] / m, a[2] / m]
    }
}

/// A 3×3 matrix, row-major.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct M3(pub [f64; 9]);

impl M3 {
    /// The identity.
    pub const ID: M3 = M3([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);

    /// A turn of `a` radians about x (y towards z).
    pub fn rx(a: f64) -> M3 {
        let (s, c) = a.sin_cos();
        M3([1.0, 0.0, 0.0, 0.0, c, -s, 0.0, s, c])
    }

    /// A turn of `a` radians about y (z towards x).
    pub fn ry(a: f64) -> M3 {
        let (s, c) = a.sin_cos();
        M3([c, 0.0, s, 0.0, 1.0, 0.0, -s, 0.0, c])
    }

    /// A turn of `a` radians about z (x towards y).
    pub fn rz(a: f64) -> M3 {
        let (s, c) = a.sin_cos();
        M3([c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0])
    }

    /// `self` after `b`: `(self * b).apply(v) == self.apply(b.apply(v))`.
    pub fn mul(&self, b: &M3) -> M3 {
        let (a, b) = (&self.0, &b.0);
        let mut m = [0.0; 9];
        for i in 0..3 {
            for j in 0..3 {
                m[i * 3 + j] = a[i * 3] * b[j] + a[i * 3 + 1] * b[3 + j] + a[i * 3 + 2] * b[6 + j];
            }
        }
        M3(m)
    }

    /// The matrix times `v`.
    #[inline]
    pub fn apply(&self, v: V3) -> V3 {
        let m = &self.0;
        [m[0] * v[0] + m[1] * v[1] + m[2] * v[2], m[3] * v[0] + m[4] * v[1] + m[5] * v[2], m[6] * v[0] + m[7] * v[1] + m[8] * v[2]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: V3, b: V3) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-12)
    }

    #[test]
    fn turns_go_the_right_way_and_compose() {
        let q = std::f64::consts::FRAC_PI_2;
        assert!(near(M3::rx(q).apply([0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]));
        assert!(near(M3::ry(q).apply([0.0, 0.0, 1.0]), [1.0, 0.0, 0.0]));
        assert!(near(M3::rz(q).apply([1.0, 0.0, 0.0]), [0.0, 1.0, 0.0]));
        let v = [0.3, -0.7, 0.2];
        assert!(near(M3::rx(0.4).mul(&M3::ry(1.1)).apply(v), M3::rx(0.4).apply(M3::ry(1.1).apply(v))));
        assert!(near(M3::ID.apply(v), v));
    }

    #[test]
    fn vectors() {
        assert_eq!(cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
        assert!((dot(unit([3.0, 4.0, 0.0]), [1.0, 0.0, 0.0]) - 0.6).abs() < 1e-12);
        assert_eq!(unit([0.0; 3]), [0.0; 3]);
    }
}
