use crate::{Ray, Vec3};

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn hit(&self, r: &Ray) -> Option<f64> {
        let oc = r.origin - self.center;
        let a = r.dir.dot(r.dir);
        let half_b = oc.dot(r.dir);
        let c = oc.norm_squared() - self.radius.powi(2);
        let disciminant = half_b.powi(2) - a * c;
        if disciminant < 0. {
            None
        } else {
            Some((-half_b - disciminant.sqrt()) / a)
        }
    }
}
