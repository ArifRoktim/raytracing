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
pub struct HitList(pub Vec<Box<dyn Hittable>>);
impl HitList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push<T: Hittable + 'static>(&mut self, val: T) {
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
impl<T: Material> Sphere<T> {
    pub fn new(center: Vec3, radius: f64, material: T) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
    pub fn from(c: [f64; 3], radius: f64, material: T) -> Self {
        Self::new(Vec3::new(c[0], c[1], c[2]), radius, material)
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
