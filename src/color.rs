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
impl From<[u8; 3]> for Rgb {
    fn from(c: [u8; 3]) -> Self {
        Self::new(c[0], c[1], c[2])
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
impl ops::MulAssign<f64> for Rgb {
    fn mul_assign(&mut self, rhs: f64) {
        self.r = (self.r as f64 * rhs) as u8;
        self.g = (self.g as f64 * rhs) as u8;
        self.b = (self.b as f64 * rhs) as u8;
    }
}

impl ops::Mul<Albedo> for Rgb {
    type Output = Self;

    fn mul(self, rhs: Albedo) -> Self::Output {
        Self::new(
            (self.r as f64 * rhs.r) as u8,
            (self.g as f64 * rhs.g) as u8,
            (self.b as f64 * rhs.b) as u8,
        )
    }
}
impl ops::MulAssign<Albedo> for Rgb {
    fn mul_assign(&mut self, rhs: Albedo) {
        self.r = (self.r as f64 * rhs.r) as u8;
        self.g = (self.g as f64 * rhs.g) as u8;
        self.b = (self.b as f64 * rhs.b) as u8;
    }
}

#[derive(Clone)]
pub struct Albedo {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}
impl Albedo {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }
}
impl From<[f64; 3]> for Albedo {
    fn from(a: [f64; 3]) -> Self {
        Self::new(a[0], a[1], a[2])
    }
}
impl Default for Albedo {
    fn default() -> Self {
        Self::new(1., 1., 1.)
    }
}

impl ops::Mul for Albedo {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }
}
impl ops::MulAssign for Albedo {
    fn mul_assign(&mut self, rhs: Self) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
    }
}
impl ops::Mul<f64> for Albedo {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}
impl ops::Mul<Albedo> for f64 {
    type Output = Albedo;

    fn mul(self, rhs: Albedo) -> Self::Output {
        rhs * self
    }
}
impl ops::MulAssign<f64> for Albedo {
    fn mul_assign(&mut self, rhs: f64) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
    }
}
impl ops::Mul<Rgb> for Albedo {
    type Output = Rgb;

    fn mul(self, rhs: Rgb) -> Self::Output {
        rhs * self
    }
}
