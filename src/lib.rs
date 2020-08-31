pub mod color;
pub mod material;
pub mod shape;
pub mod vec3;

pub use color::{Albedo, Rgb};
pub use material::{Material, Scatter};
pub use shape::{Hit, HitList, Hittable};
pub use vec3::Vec3;

pub struct Screen {
    pub width: usize,
    pub height: usize,
    /// Flat buffer of 24-bit pixels with length of `width * height`
    pub buffer: Box<[Rgb]>,
}
impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![Rgb::default(); width * height].into(),
        }
    }

    /// Encodes each Pixel into `0RGB` and optionally applies gamma correction
    pub fn encode(&self, gamma2: bool) -> Box<[u32]> {
        self.buffer
            .iter()
            .map(|p| {
                let (r, g, b) = if gamma2 {
                    let (r, g, b) = (p.r as f64 / 255., p.g as f64 / 255., p.b as f64 / 255.);
                    let (r, g, b) = (r.sqrt() * 255., g.sqrt() * 255., b.sqrt() * 255.);
                    (r as u32, g as u32, b as u32)
                } else {
                    (p.r as u32, p.g as u32, p.b as u32)
                };

                (r << 16) | (g << 8) | b
            })
            .collect()
    }

    pub fn rows_mut(&mut self) -> std::slice::ChunksExactMut<Rgb> {
        self.buffer.chunks_exact_mut(self.width)
    }
}

/// # Note
/// Camera assumes an aspect ratio of 16:9
pub struct Camera {
    pub origin: Vec3,
    pub horiz: Vec3,
    pub vert: Vec3,
    pub lower_left_corner: Vec3,
}
impl Camera {
    fn new(origin: Vec3, horiz: Vec3, vert: Vec3, bot_left_corner: Vec3) -> Self {
        Self {
            origin,
            horiz,
            vert,
            lower_left_corner: bot_left_corner,
        }
    }

    pub fn get_ray(&self, i: f64, j: f64) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + i * self.horiz + j * self.vert,
        )
    }
}
impl Default for Camera {
    fn default() -> Self {
        let horiz = Vec3::new(4., 0., 0.);
        let vert = Vec3::new(0., horiz.x / 16. * 9., 0.);
        let lower_left_corner = Vec3::ORIGIN - horiz / 2. - vert / 2. - Vec3::new(0., 0., 1.);
        Self::new(Vec3::ORIGIN, horiz, vert, lower_left_corner)
    }
}

#[derive(Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}
impl Ray {
    pub fn new(orig: Vec3, dir: Vec3) -> Self {
        Self { origin: orig, dir }
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + t * self.dir
    }
}
