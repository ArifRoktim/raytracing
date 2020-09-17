pub mod color;
pub mod hit;
pub mod material;
pub mod screen;
pub mod shape;
pub mod vec3;

pub use color::Color;
pub use hit::{Hit, HitList, Hittable, AABB, BVH};
pub use material::{Material, Scatter};
pub use screen::{Camera, CameraBuilder, Screen};
pub use vec3::{Axis, Vec3};

pub type CrateRng = rand::rngs::SmallRng;

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
