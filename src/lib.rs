pub mod color;
pub mod ray;
pub mod shape;
pub mod vec3;

pub use color::Rgb;
pub use ray::Ray;
pub use shape::{Hit, HitList, Hittable};
pub use vec3::Vec3;
