//! A minimal 2D vector, so the simulation has no engine dependency.

use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::math;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub const ZERO: V2 = V2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> V2 {
        V2 { x, y }
    }

    /// Unit vector at `angle` radians.
    pub fn from_angle(angle: f32) -> V2 {
        V2::new(math::cos(angle), math::sin(angle))
    }

    pub fn length_sq(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> f32 {
        math::sqrt(self.length_sq())
    }

    pub fn distance(self, other: V2) -> f32 {
        (self - other).length()
    }

    pub fn distance_sq(self, other: V2) -> f32 {
        (self - other).length_sq()
    }

    /// Unit vector in the same direction; zero stays zero.
    pub fn normalized(self) -> V2 {
        let len = self.length();
        if len > 1e-6 {
            self * (1.0 / len)
        } else {
            V2::ZERO
        }
    }

    /// Shortened to at most `max` length.
    pub fn clamp_length(self, max: f32) -> V2 {
        let len = self.length();
        if len > max {
            self * (max / len)
        } else {
            self
        }
    }

    pub fn dot(self, other: V2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Rotated a quarter turn counter-clockwise.
    pub fn perp(self) -> V2 {
        V2::new(-self.y, self.x)
    }

    pub fn angle(self) -> f32 {
        math::atan2(self.y, self.x)
    }

    /// Rotated by `angle` radians.
    pub fn rotated(self, angle: f32) -> V2 {
        let (s, c) = (math::sin(angle), math::cos(angle));
        V2::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }
}

impl Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        V2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 {
        V2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f32> for V2 {
    type Output = V2;
    fn mul(self, k: f32) -> V2 {
        V2::new(self.x * k, self.y * k)
    }
}

impl Neg for V2 {
    type Output = V2;
    fn neg(self) -> V2 {
        V2::new(-self.x, -self.y)
    }
}

impl AddAssign for V2 {
    fn add_assign(&mut self, o: V2) {
        self.x += o.x;
        self.y += o.y;
    }
}

impl SubAssign for V2 {
    fn sub_assign(&mut self, o: V2) {
        self.x -= o.x;
        self.y -= o.y;
    }
}

impl MulAssign<f32> for V2 {
    fn mul_assign(&mut self, k: f32) {
        self.x *= k;
        self.y *= k;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizing_zero_stays_zero() {
        assert_eq!(V2::ZERO.normalized(), V2::ZERO);
        let n = V2::new(3.0, 4.0).normalized();
        assert!((n.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn rotation_by_a_quarter_turn_is_perp() {
        let v = V2::new(2.0, 1.0);
        let r = v.rotated(std::f32::consts::FRAC_PI_2);
        assert!((r.x - v.perp().x).abs() < 1e-5 && (r.y - v.perp().y).abs() < 1e-5);
    }
}
