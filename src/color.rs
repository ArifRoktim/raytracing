use std::ops;

#[derive(Copy, Clone, Debug, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Scales `r`, `g`, `b` from values of `0.0` to `1.0` onto `0u8` to `255`
    pub fn f64(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: (r * 255.999) as u8,
            g: (g * 255.999) as u8,
            b: (b * 255.999) as u8,
        }
    }
}

impl ops::Add for Rgb {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}
impl ops::Mul<f64> for Rgb {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(
            (self.r as f64 * rhs) as u8,
            (self.g as f64 * rhs) as u8,
            (self.b as f64 * rhs) as u8,
        )
    }
}
impl ops::Mul<Rgb> for f64 {
    type Output = Rgb;

    fn mul(self, rhs: Rgb) -> Self::Output {
        rhs * self
    }
}
