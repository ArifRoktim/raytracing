pub mod color;
pub mod material;
pub mod shape;
pub mod vec3;

pub use color::Color;
pub use material::{Material, Scatter};
pub use shape::{Hit, HitList, Hittable};
pub use vec3::Vec3;

use rayon::prelude::*;

pub struct Screen {
    pub width: usize,
    pub height: usize,
    /// Flat buffer of 24-bit pixels with length of `width * height`
    pub buffer: Box<[Color]>,
}
impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![Color::default(); width * height].into(),
        }
    }

    /// Encodes each Pixel into `0RGB` and applies gamma correction
    pub fn encode(&self) -> Box<[u32]> {
        self.buffer
            .iter()
            .map(|p| {
                assert!(0.0 <= p.r && p.r <= 1.0);
                assert!(0.0 <= p.g && p.g <= 1.0);
                assert!(0.0 <= p.b && p.b <= 1.0);

                let (r, g, b) = (
                    255.99 * p.r.sqrt(),
                    255.99 * p.g.sqrt(),
                    255.99 * p.b.sqrt(),
                );
                let (r, g, b) = (r as u32, g as u32, b as u32);
                (r << 16) | (g << 8) | b
            })
            .collect()
    }

    pub fn rows_mut(&mut self) -> std::slice::ChunksExactMut<Color> {
        self.buffer.chunks_exact_mut(self.width)
    }

    pub fn par_rows_mut(&mut self) -> rayon::slice::ChunksExactMut<Color> {
        self.buffer.par_chunks_exact_mut(self.width)
    }
}

pub struct Camera {
    pub origin: Vec3,
    pub horiz: Vec3,
    pub vert: Vec3,
    pub lower_left: Vec3,
    pub lens_radius: f64,
    // Orthonormal basis to describe camera's orientation
    // x
    pub u: Vec3,
    // y
    pub v: Vec3,
    // z
    pub w: Vec3,
}
impl Camera {
    pub fn new(
        look_from: Vec3,
        look_at: Vec3,
        view_up: Option<Vec3>,
        vfov_degrees: f64,
        aspect_ratio: f64,
        aperature: f64,
        focus_dist: Option<f64>,
    ) -> Self {
        // Fill in default values if not provided
        let view_up = match view_up {
            Some(vup) => vup,
            None => Vec3::new(0., 1., 0.),
        };
        let focus_dist = match focus_dist {
            Some(f) => f,
            None => (look_from - look_at).norm(),
        };

        let theta = vfov_degrees.to_radians() / 2.;
        let half_height = focus_dist * theta.tan();
        let half_width = aspect_ratio * half_height;

        // Project view_up onto the plane of the camera
        let w = Vec3::normalized(look_from - look_at);
        let u = Vec3::normalized(view_up.cross(w));
        let v = w.cross(u);

        let lower_left = look_from - u * half_width - v * half_height - focus_dist * w;
        let horiz = 2. * u * half_width;
        let vert = 2. * v * half_height;

        let lens_radius = aperature / 2.;
        Self {
            origin: look_from,
            horiz,
            vert,
            lower_left,
            lens_radius,
            u,
            v,
            w,
        }
    }

    pub fn from<T: Into<Vec3>>(
        look_from: T,
        look_at: T,
        view_up: Option<T>,
        vfov_degrees: f64,
        aspect_ratio: f64,
        aperature: f64,
        focus_dist: Option<f64>,
    ) -> Self {
        let look_from = look_from.into();
        let look_at = look_at.into();
        let view_up = view_up.map(|v| v.into());

        Self::new(
            look_from,
            look_at,
            view_up,
            vfov_degrees,
            aspect_ratio,
            aperature,
            focus_dist,
        )
    }

    pub fn get_ray(&self, i: f64, j: f64) -> Ray {
        let rand_disk = self.lens_radius * Vec3::rand_unit_disk();
        let offset = rand_disk.x * self.u + rand_disk.y * self.v;
        let origin = self.origin + offset;

        Ray::new(
            origin,
            self.lower_left + i * self.horiz + j * self.vert - origin,
        )
    }
}

#[derive(Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}
impl Ray {
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Self { origin, dir }
    }

    pub fn from<T: Into<Vec3>, U: Into<Vec3>>(origin: T, dir: U) -> Self {
        Self::new(origin.into(), dir.into())
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + t * self.dir
    }
}
