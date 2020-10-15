pub mod color;
pub mod config;
pub mod hit;
pub mod material;
pub mod screen;
pub mod shape;
pub mod vec3;

pub use color::Color;
pub use config::Config;
pub use hit::{Hit, HitList, Hittable, AABB, BVH};
pub use material::{Material, Scatter, Texture};
pub use screen::{Camera, CameraBuilder, Screen};
pub use vec3::{Axis, Vec3};

pub type CrateRng = rand::rngs::SmallRng;

use anyhow::{Context, Result};

#[derive(Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
    pub time: f64,
}
impl Ray {
    pub fn new(origin: Vec3, dir: Vec3, time: f64) -> Self {
        Self { origin, dir, time }
    }

    pub fn from(origin: [f64; 3], dir: [f64; 3], time: f64) -> Self {
        Self::new(origin.into(), dir.into(), time)
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + t * self.dir
    }
}

// ===== Extension Traits =====
pub trait F64Ext {
    fn lerp(self, low: f64, high: f64) -> f64;
    fn smooth(self) -> f64;
}
impl F64Ext for f64 {
    fn lerp(self, low: f64, high: f64) -> f64 {
        low * (1. - self) + high * self
    }

    fn smooth(self) -> f64 {
        // 6t^5 - 15t^4 + 10t^3
        self.powi(3) * (self * (6. * self - 15.) + 10.)
    }
}

pub trait ResultExt<T> {
    fn camera_context(self, builder: &CameraBuilder) -> Result<T>;
}
impl<T> ResultExt<T> for Result<T> {
    /// Attach the CameraBuilder to the Result as context.
    fn camera_context(self, builder: &CameraBuilder) -> Result<T> {
        self.with_context(|| format!("Invalid Camera configuration.\n{:#?}", builder))
    }
}
