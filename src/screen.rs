use std::f64::consts;
use std::ops::Range;

use anyhow::{anyhow, ensure, Context, Result};
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;

use crate::{config, Axis, Color, CrateRng, Ray, ResultExt, Vec3};

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

    /// Used for depth of field. Set to `0` to disable depth of field.
    pub lens_radius: f64,
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
        let origin = if self.lens_radius == 0. {
            self.origin
        } else {
            let rand_disk = self.lens_radius * Vec3::rand_unit_disk(rng);
            let offset = rand_disk.x * self.u + rand_disk.y * self.v;
            self.origin + offset
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
    origin: Option<Vec3>,
    look_at: Option<Vec3>,
    view_up: Vec3,
    vfov_degrees: f64,
    aspect_ratio: f64,
    /// Used for depth of field. Set to `0` to disable depth of field.
    aperture: f64,
    /// If None, defaults to magnitude of vector between `origin` and `look_at`.
    focus_dist: Option<f64>,
    /// Used for motion blur. Set to `None` to disable.
    shutter_time: Option<Range<f64>>,
}
impl CameraBuilder {
    pub fn build(&self) -> Result<Camera> {
        self.verify().camera_context(self)?;

        let origin = self.origin.unwrap();
        let look_at = self.look_at.unwrap();

        let lens_radius = self.aperture / 2.;
        let focus_dist = self.focus_dist.unwrap_or_else(|| (origin - look_at).norm());
        let shutter_time = self.shutter_time.clone().map(Uniform::from);

        let theta = self.vfov_degrees.to_radians() / 2.;
        let half_height = focus_dist * theta.tan();
        let half_width = self.aspect_ratio * half_height;

        // Project view_up onto the plane of the camera and form the orthonormal basis.
        let view_up = Vec3::checked_normalized(self.view_up).unwrap();
        let w = Vec3::checked_normalized(origin - look_at).unwrap();
        let u = Vec3::checked_normalized(view_up.cross(w)).unwrap();
        let v = w.cross(u);

        let lower_left = origin - u * half_width - v * half_height - focus_dist * w;
        let horiz = 2. * u * half_width;
        let vert = 2. * v * half_height;

        Ok(Camera {
            origin,
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

    /// Deal with bad camera configurations.
    pub fn verify(&self) -> Result<()> {
        // Make sure that required parameters were set
        let origin = self
            .origin
            .ok_or_else(|| anyhow!("Camera's origin wasn't provided."))?;
        let look_at = self
            .look_at
            .ok_or_else(|| anyhow!("Camera's look_at wasn't provided."))?;

        // Error if the view_up vector has length 0.
        let view_up = Vec3::checked_normalized(self.view_up)
            .with_context(|| format!("Camera's view_up vector has length 0: {:?}", self.view_up))?;

        // Error if camera's origin and look_at are the same.
        let w = Vec3::checked_normalized(origin - look_at).with_context(|| {
            format!(
                "Camera's origin and look_at vectors are the same.\nOrigin: {:?}",
                origin,
            )
        })?;

        // Error if look_at and view_up are parallel.
        Vec3::checked_normalized(view_up.cross(w)).with_context(|| {
            format!(
                "Camera's look_at and view_up vectors are parellel.\nResp.: {:?} || {:?}",
                look_at, view_up,
            )
        })?;

        // Aperture can be 0 to disable depth of field
        ensure!(self.aperture >= 0., "Camera's aperture is less than 0.");

        ensure!(
            self.vfov_degrees > 0.,
            "Camera's fov is less than or equal to 0."
        );
        ensure!(
            self.aspect_ratio > 0.,
            "Camera's aspect ratio is less than or equal to 0."
        );
        if let Some(dist) = self.focus_dist {
            ensure!(
                dist > 0.,
                "Camera's focus distance is less than or equal to 0."
            );
        }

        Ok(())
    }

    // ===== Builder Methods =====
    pub fn origin<T: Into<Vec3>>(&mut self, origin: T) -> &mut Self {
        self.origin = Some(origin.into());
        self
    }
    pub fn look_at<T: Into<Vec3>>(&mut self, look_at: T) -> &mut Self {
        self.look_at = Some(look_at.into());
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
    /// Set the camera's view_up to be `deg` degrees counterclockwise from straight up,
    /// relative to the given Axis.
    /// # Example
    /// ```
    /// # use raytracing::{Axis, Camera};
    /// let c = Camera::builder()
    ///     .origin([0., 20., 0.])
    ///     .look_at([0., 10., 0.])
    ///     .view_up_degrees(15., Axis::Z)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn view_up_degrees(&mut self, deg: f64, axis: Axis) -> &mut Self {
        // Shift the angle by pi/2 so that an input of `deg: 0.0` will result
        // in view_up being straight up, as opposed to straight right.
        let rads = deg.to_radians() + consts::FRAC_PI_2;
        let (sin, cos) = rads.sin_cos();
        self.view_up = Vec3::from(match axis {
            Axis::X => [0., sin, -cos],
            Axis::Y => [cos, 0., -sin],
            Axis::Z => [cos, sin, 0.],
        });
        self
    }
    /// Used for depth of field. Set to `None` to disable depth of field.
    pub fn aperture(&mut self, aperture: f64) -> &mut Self {
        self.aperture = aperture;
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
        let width = config::GLOBAL().width.get() as f64;
        let height = config::GLOBAL().height.get() as f64;
        Self {
            origin: None,
            look_at: None,
            view_up: Vec3::UNIT_Y,
            vfov_degrees: 60.,
            aspect_ratio: width / height,
            aperture: 0.,
            focus_dist: None,
            shutter_time: None,
        }
    }
}
