
#[derive(Copy, Clone, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {r, g, b}
    }

    /// ```
    /// use raytracing::color::Rgb;
    /// let a = Rgb::f64(1., 0., 0.5);
    /// assert_eq!(a.r, 255u8);
    /// ```
    pub fn f64(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: (r * 255.999) as u8,
            g: (g * 255.999) as u8,
            b: (b * 255.999) as u8,
        }
    }
}
