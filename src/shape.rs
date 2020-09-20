use std::f64::consts::PI;
use std::fmt::Debug;
use std::ops::Range;

use crate::{Hit, Hittable, Material, Ray, Vec3, AABB};

fn sphere_uv(point: Vec3, center: Vec3, radius: f64) -> (f64, f64) {
    let p: Vec3 = (point - center) / radius;
    let phi = p.z.atan2(p.x);
    let theta = p.y.asin();
    let u = 1. - (phi + PI) / (2. * PI);
    let v = (theta + PI / 2.) / PI;
    (u, v)
}

#[derive(Debug)]
pub struct Sphere<T> {
    pub center: Vec3,
    pub radius: f64,
    pub material: T,
}
impl<T> Sphere<T> {
    pub fn new(center: Vec3, radius: f64, material: T) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
    pub fn from(c: [f64; 3], radius: f64, material: T) -> Self {
        Self::new(c.into(), radius, material)
    }
}
impl<T: Material> Hittable for Sphere<T> {
    fn hit(&self, ray: &Ray, hit_time: &Range<f64>) -> Option<Hit> {
        let oc = ray.origin - self.center;
        let a = ray.dir.norm_squared();
        let half_b = oc.dot(ray.dir);
        let c = oc.norm_squared() - self.radius.powi(2);
        let disciminant = half_b.powi(2) - a * c;

        if disciminant >= 0. {
            let root = disciminant.sqrt();
            let hit = |t| {
                let point = ray.at(t);
                let outward_normal = (point - self.center) / self.radius;
                let (u, v) = sphere_uv(point, self.center, self.radius);

                let ret = Hit::ray(point, outward_normal, t, ray, &self.material, u, v);
                Some(ret)
            };

            let t = (-half_b - root) / a;
            if hit_time.contains(&t) {
                return hit(t);
            }

            let t = (-half_b + root) / a;
            if hit_time.contains(&t) {
                return hit(t);
            }
        }

        None
    }

    fn bounding_box(&self, _shutter_time: &Range<f64>) -> Option<AABB> {
        let rad = Vec3::from([self.radius; 3]);
        Some(AABB::new(self.center - rad, self.center + rad))
    }
}

/// Sphere whose center moves from `center_0` (at `time = 0.0`) to `center_1` (at `time = 1.0`).
#[derive(Debug)]
pub struct MovingSphere<T> {
    center_0: Vec3,
    delta_c: Vec3,
    radius: f64,
    material: T,
}
impl<T> MovingSphere<T> {
    pub fn new(center_0: Vec3, center_1: Vec3, radius: f64, material: T) -> Self {
        Self {
            center_0,
            delta_c: center_1 - center_0,
            radius,
            material,
        }
    }

    pub fn from(c0: [f64; 3], c1: [f64; 3], radius: f64, material: T) -> Self {
        Self::new(c0.into(), c1.into(), radius, material)
    }

    // Returns the center at `time`
    pub fn center(&self, time: f64) -> Vec3 {
        self.center_0 + time * self.delta_c
    }
}
impl<T: Material> Hittable for MovingSphere<T> {
    fn hit(&self, ray: &Ray, hit_time: &Range<f64>) -> Option<Hit> {
        let center = self.center(ray.time);

        let oc = ray.origin - center;
        let a = ray.dir.norm_squared();
        let half_b = oc.dot(ray.dir);
        let c = oc.norm_squared() - self.radius.powi(2);
        let disciminant = half_b.powi(2) - a * c;

        if disciminant >= 0. {
            let root = disciminant.sqrt();
            let hit = |t| {
                let point = ray.at(t);
                let outward_normal = (point - center) / self.radius;
                let (u, v) = sphere_uv(point, center, self.radius);

                let ret = Hit::ray(point, outward_normal, t, ray, &self.material, u, v);
                Some(ret)
            };

            let t = (-half_b - root) / a;
            if hit_time.contains(&t) {
                return hit(t);
            }

            let t = (-half_b + root) / a;
            if hit_time.contains(&t) {
                return hit(t);
            }
        }

        None
    }

    fn bounding_box(&self, shutter_time: &Range<f64>) -> Option<AABB> {
        let rad = Vec3::from([self.radius; 3]);
        let aabb = AABB::new(
            self.center(shutter_time.start) - rad,
            self.center(shutter_time.start) + rad,
        );
        Some(aabb.surrounding(&AABB::new(
            self.center(shutter_time.end) - rad,
            self.center(shutter_time.end) + rad,
        )))
    }
}

/// Dummy Hittable for use in BVH node
#[derive(Debug)]
pub struct Dummy {}
impl Hittable for Dummy {
    /// Dummy will never return a hit.
    fn hit(&self, _ray: &Ray, _hit_time: &Range<f64>) -> Option<Hit> {
        None
    }

    /// Bounding box isn't applicable for Dummy
    fn bounding_box(&self, _shutter_time: &Range<f64>) -> Option<AABB> {
        unimplemented!("Hittable::bounding_box is not applicable for Dummy!")
    }
}
