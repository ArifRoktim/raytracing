use crate::{Hit, Hittable, Material, Ray, Vec3, AABB};
use std::fmt::Debug;
use std::ops::Range;

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
impl<T: Material + Send + Sync + Debug> Hittable for Sphere<T> {
    fn hit(&self, ray: &Ray, range: &Range<f64>) -> Option<Hit> {
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
                Some(Hit::ray(point, outward_normal, t, ray, &self.material))
            };

            let t = (-half_b - root) / a;
            if range.contains(&t) {
                return hit(t);
            }

            let t = (-half_b + root) / a;
            if range.contains(&t) {
                return hit(t);
            }
        }

        None
    }

    fn bounding_box(&self, _range: &Range<f64>) -> Option<AABB> {
        let rad = Vec3::from([self.radius; 3]);
        Some(AABB::new(self.center - rad, self.center + rad))
    }
}

/// Sphere whose center moves from `c0` at `t0` to `c1` at `t1`
#[derive(Debug)]
pub struct MovingSphere<T> {
    pub c0: Vec3,
    pub c1: Vec3,
    // TODO: Get rid of t0 and t1 and just make c0 move to c1 from t=0 to t=1
    pub t0: f64,
    pub t1: f64,
    pub radius: f64,
    pub material: T,
}
impl<T> MovingSphere<T> {
    pub fn new(c0: Vec3, c1: Vec3, t0: f64, t1: f64, radius: f64, material: T) -> Self {
        Self {
            c0,
            c1,
            t0,
            t1,
            radius,
            material,
        }
    }

    pub fn from(c0: [f64; 3], c1: [f64; 3], t0: f64, t1: f64, radius: f64, material: T) -> Self {
        Self::new(c0.into(), c1.into(), t0, t1, radius, material)
    }

    // Returns the center at time `t`
    pub fn center(&self, t: f64) -> Vec3 {
        self.c0 + (self.c1 - self.c0) * (t - self.t0) / (self.t1 - self.t0)
    }
}
impl<T: Material + Send + Sync + Debug> Hittable for MovingSphere<T> {
    fn hit(&self, ray: &Ray, range: &Range<f64>) -> Option<Hit> {
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
                Some(Hit::ray(point, outward_normal, t, ray, &self.material))
            };

            let t = (-half_b - root) / a;
            if range.contains(&t) {
                return hit(t);
            }

            let t = (-half_b + root) / a;
            if range.contains(&t) {
                return hit(t);
            }
        }

        None
    }

    fn bounding_box(&self, range: &Range<f64>) -> Option<AABB> {
        let rad = Vec3::from([self.radius; 3]);
        let aabb = AABB::new(
            self.center(range.start) - rad,
            self.center(range.start) + rad,
        );
        Some(aabb.surrounding(&AABB::new(
            self.center(range.end) - rad,
            self.center(range.end) + rad,
        )))
    }
}

/// Dummy Hittable for use in BVH node
#[derive(Debug)]
pub struct Dummy {}
impl Hittable for Dummy {
    /// Dummy will never return a hit.
    fn hit(&self, _ray: &Ray, _range: &Range<f64>) -> Option<Hit> {
        None
    }

    /// Bounding box isn't applicable for Dummy
    fn bounding_box(&self, _range: &Range<f64>) -> Option<AABB> {
        unimplemented!("Hittable::bounding_box is not applicable for Dummy!")
    }
}
