use crate::{Material, Ray, Vec3};
use std::ops::Range;

pub struct Hit<'a> {
    pub point: Vec3,
    /// A unit normal vector
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub material: &'a dyn Material,
}
impl<'a> Hit<'a> {
    pub fn new(
        point: Vec3,
        normal: Vec3,
        t: f64,
        front_face: bool,
        material: &'a dyn Material,
    ) -> Self {
        Self {
            point,
            normal,
            t,
            front_face,
            material,
        }
    }

    pub fn ray(
        point: Vec3,
        mut normal: Vec3,
        t: f64,
        ray: &Ray,
        material: &'a dyn Material,
    ) -> Self {
        // Dot product is negative when ray hits back face
        let front_face = ray.dir.dot(normal) < 0.;
        // Make suface normal always point against incident ray
        if !front_face {
            normal *= -1.;
        }
        Self::new(point, normal, t, front_face, material)
    }
}

pub trait Hittable {
    fn hit(&self, _ray: &Ray, _range: &Range<f64>) -> Option<Hit> {
        None
    }
}

#[derive(Default)]
pub struct HitList(pub Vec<Box<dyn Hittable + Send + Sync>>);
impl HitList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push<T: Hittable + Send + Sync + 'static>(&mut self, val: T) {
        self.0.push(Box::new(val))
    }
}
impl Hittable for HitList {
    fn hit(&self, ray: &Ray, range: &Range<f64>) -> Option<Hit> {
        let mut range = range.clone();
        let mut closest = None;
        for obj in &self.0 {
            if let Some(hit) = obj.hit(ray, &range) {
                range.end = hit.t;
                closest = Some(hit);
            }
        }
        closest
    }
}

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
}

/// Sphere whose center moves from `c0` at `t0` to `c1` at `t1`
pub struct MovingSphere<T> {
    pub c0: Vec3,
    pub c1: Vec3,
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
impl<T: Material> Hittable for MovingSphere<T> {
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
}
