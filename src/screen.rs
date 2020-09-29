use std::ops::Range;

use anyhow::{Context, Result};
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;

use crate::{config, Axis, Color, CrateRng, Ray, Vec3};

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
                // Check for invalid Colors, including NANs
                let bounds = 0.0..=1.0;
                if !bounds.contains(&p.r) || !bounds.contains(&p.g) || !bounds.contains(&p.b) {
                    panic!("Invalid color: {:?}", p);
                }

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

#[derive(Debug)]
pub struct Camera {
    pub origin: Vec3,
    pub horiz: Vec3,
    pub vert: Vec3,
    pub lower_left: Vec3,

    /// Used for depth of field. Set to `None` to disable depth of field.
    pub lens_radius: Option<f64>,
    /// Used for motion blur. Set to `None` to disable.
    pub shutter_time: Option<Uniform<f64>>,
    /// Width part of the orthonormal basis.
    pub u: Vec3,
    /// Height part of the orthonormal basis.
    pub v: Vec3,
    /// Depth part of the orthonormal basis.
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

#[derive(Debug)]
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
    pub fn build(&self) -> Result<Camera> {
        let lens_radius = self.aperture.map(|a| a / 2.);
        let focus_dist = self
            .focus_dist
            .unwrap_or_else(|| (self.origin - self.look_at).norm());
        let shutter_time = self.shutter_time.clone().map(Uniform::from);

        let theta = self.vfov_degrees.to_radians() / 2.;
        let half_height = focus_dist * theta.tan();
        let half_width = self.aspect_ratio * half_height;

        // Project view_up onto the plane of the camera and form the orthonormal basis.
        // Also deal with bad camera configurations.

        // Error if camera's origin and look_at are the same.
        let w = Vec3::checked_normalized(self.origin - self.look_at)
            .with_context(|| {
                format!(
                    "Camera's origin and look_at vectors are the same.\nOrigin: {:?}",
                    self.origin,
                )
            })
            .camera_context(self)?;

        // Error if the view_up vector has length 0.
        let view_up = Vec3::checked_normalized(self.view_up)
            .with_context(|| format!("Camera's view_up vector has length 0: {:?}", self.view_up))
            .camera_context(self)?;

        // Error if look_at and view_up are parallel.
        let u = Vec3::checked_normalized(view_up.cross(w))
            .with_context(|| {
                format!(
                    "Camera's look_at and view_up vectors are parellel.\nResp.: {:?} || {:?}",
                    self.look_at, view_up,
                )
            })
            .camera_context(self)?;

        let v = w.cross(u);
        let lower_left = self.origin - u * half_width - v * half_height - focus_dist * w;
        let horiz = 2. * u * half_width;
        let vert = 2. * v * half_height;

        Ok(Camera {
            origin: self.origin,
            horiz,
            vert,
            lower_left,
            lens_radius,
            shutter_time,
            u,
            v,
            w,
        })
    }
    // ===== Builder Methods =====
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
    /// Used for depth of field. Set to `None` to disable depth of field.
    pub fn aperture<T: Into<Option<f64>>>(&mut self, aperture: T) -> &mut Self {
        self.aperture = aperture.into();
        self
    }
    /// If None, defaults to magnitude of vector between `origin` and `look_at`.
    pub fn focus_dist<T: Into<Option<f64>>>(&mut self, dist: T) -> &mut Self {
        self.focus_dist = dist.into();
        self
    }
    /// Used for motion blur. Set to `None` to disable.
    pub fn shutter_time<T: Into<Option<Range<f64>>>>(&mut self, range: T) -> &mut Self {
        self.shutter_time = range.into();
        self
    }
}
impl Default for CameraBuilder {
    fn default() -> Self {
        let width = config::GLOBAL().width as f64;
        let height = config::GLOBAL().height as f64;
        Self {
            origin: Vec3::ORIGIN,
            look_at: Vec3::new(0., 0., -1.),
            view_up: Vec3::UNIT_Y,
            vfov_degrees: 60.,
            aspect_ratio: width / height,
            aperture: None,
            focus_dist: None,
            shutter_time: None,
        }
    }
}

trait ResultExt {
    fn camera_context(self, builder: &CameraBuilder) -> Result<Vec3>;
}
impl ResultExt for Result<Vec3> {
    /// Attach the CameraBuilder to the Result as context.
    fn camera_context(self, builder: &CameraBuilder) -> Result<Vec3> {
        self.with_context(|| format!("Invalid Camera configuration.\n{:#?}", builder))
    }
}
