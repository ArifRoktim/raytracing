use crate::{Color, CrateRng, Ray, Vec3};
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;
use std::ops::Range;

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

    /// Used for depth of field. Set to `None` to disable depth of field.
    pub lens_radius: Option<f64>,
    /// Used for motion blur. Set to `None` to disable.
    pub shutter_time: Option<Uniform<f64>>,
    /// Orthonormal basis
    pub u: Vec3,
    /// Orthonormal basis
    pub v: Vec3,
    /// Orthonormal basis
    pub w: Vec3,
}
impl Camera {
    pub fn builder() -> CameraBuilder {
        CameraBuilder::default()
    }

    pub fn get_ray(&self, i: f64, j: f64, rng: &mut CrateRng) -> Ray {
        let origin = if let Some(radius) = self.lens_radius {
            let rand_disk = radius * Vec3::rand_unit_disk(rng);
            let offset = rand_disk.x * self.u + rand_disk.y * self.v;
            self.origin + offset
        } else {
            self.origin
        };
        let time = self.shutter_time.map_or(0., |s| s.sample(rng));

        Ray::new(
            origin,
            self.lower_left + i * self.horiz + j * self.vert - origin,
            time,
        )
    }
}

pub struct CameraBuilder {
    origin: Vec3,
    look_at: Vec3,
    view_up: Vec3,
    vfov_degrees: f64,
    aspect_ratio: f64,
    /// Used for depth of field. Set to `None` to disable depth of field.
    aperture: Option<f64>,
    /// If None, defaults to magnitude of vector between `origin` and `look_at`.
    focus_dist: Option<f64>,
    /// Used for motion blur. Set to `None` to disable.
    shutter_time: Option<Range<f64>>,
}
impl CameraBuilder {
    pub fn build(&self) -> Camera {
        let lens_radius = self.aperture.map(|a| a / 2.);
        let focus_dist = self
            .focus_dist
            .unwrap_or_else(|| (self.origin - self.look_at).norm());
        let shutter_time = self.shutter_time.clone().map(Uniform::from);

        let theta = self.vfov_degrees.to_radians() / 2.;
        let half_height = focus_dist * theta.tan();
        let half_width = self.aspect_ratio * half_height;

        // Project view_up onto the plane of the camera
        let w = Vec3::normalized(self.origin - self.look_at);
        let u = Vec3::normalized(self.view_up.cross(w));
        let v = w.cross(u);

        let lower_left = self.origin - u * half_width - v * half_height - focus_dist * w;
        let horiz = 2. * u * half_width;
        let vert = 2. * v * half_height;

        Camera {
            origin: self.origin,
            horiz,
            vert,
            lower_left,
            lens_radius,
            shutter_time,
            u,
            v,
            w,
        }
    }
    // ===Builder Methods===
    pub fn origin<T: Into<Vec3>>(&mut self, origin: T) -> &mut Self {
        self.origin = origin.into();
        self
    }
    pub fn look_at<T: Into<Vec3>>(&mut self, look_at: T) -> &mut Self {
        self.look_at = look_at.into();
        self
    }
    pub fn vfov_degrees(&mut self, vfov: f64) -> &mut Self {
        self.vfov_degrees = vfov;
        self
    }
    pub fn aspect_ratio(&mut self, aspect_ratio: f64) -> &mut Self {
        self.aspect_ratio = aspect_ratio;
        self
    }
    pub fn view_up<T: Into<Vec3>>(&mut self, view_up: T) -> &mut Self {
        self.view_up = view_up.into();
        self
    }
    pub fn aperture(&mut self, aperture: f64) -> &mut Self {
        self.aperture = Some(aperture);
        self
    }
    pub fn focus_dist(&mut self, dist: f64) -> &mut Self {
        self.focus_dist = Some(dist);
        self
    }
    pub fn shutter_time(&mut self, start: f64, end: f64) -> &mut Self {
        self.shutter_time = Some(Range { start, end });
        self
    }
}
impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            origin: Vec3::ORIGIN,
            look_at: Vec3::new(0., 0., -1.),
            view_up: Vec3::UNIT_Y,
            vfov_degrees: 60.,
            aspect_ratio: 16. / 9.,
            aperture: None,
            focus_dist: None,
            shutter_time: None,
        }
    }
}
