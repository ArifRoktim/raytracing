use std::ops;

use rand::distributions::{Distribution, Uniform};
use rand::Rng;

use crate::{CrateRng, Texture};

/// Each color value ranges from 0.0 to 1.0, where 1.0 is full brightness
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}
impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    pub fn rand(rng: &mut CrateRng) -> Self {
        let albedo = rng.gen::<[f64; 3]>();
        albedo.into()
    }

    pub fn rand_range(rng: &mut CrateRng, low: f64, high: f64) -> Self {
        let distr = Uniform::new(low, high);
        let albedo = [distr.sample(rng), distr.sample(rng), distr.sample(rng)];
        albedo.into()
    }
}
impl Texture for Color {
    fn value(&self, _u: f64, _v: f64, _point: crate::Vec3) -> Color {
        *self
    }
}

impl From<[f64; 3]> for Color {
    fn from(a: [f64; 3]) -> Self {
        Self::new(a[0], a[1], a[2])
    }
}
impl Default for Color {
    /// Returns white
    fn default() -> Self {
        Self::new(1., 1., 1.)
    }
}

impl ops::Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}
impl ops::AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}
impl ops::Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }
}
impl ops::MulAssign for Color {
    fn mul_assign(&mut self, rhs: Self) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
    }
}
impl ops::Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}
impl ops::Mul<Color> for f64 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}
impl ops::MulAssign<f64> for Color {
    fn mul_assign(&mut self, rhs: f64) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
    }
}
impl ops::DivAssign<f64> for Color {
    fn div_assign(&mut self, rhs: f64) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
    }
}
