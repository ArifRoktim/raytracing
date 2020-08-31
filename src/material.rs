use crate::{Albedo, Hit, Ray, Vec3};

pub struct Scatter {
    pub albedo: Albedo,
    pub ray: Ray,
}
impl Scatter {
    pub fn new(albedo: Albedo, ray: Ray) -> Self {
        Self { albedo, ray }
    }
}

pub trait Material {
    /// A material will either absorb a ray (`None`) or scatter it.
    fn scatter(&self, _ray: &Ray, _hit: &Hit) -> Option<Scatter> {
        None
    }
}

pub struct Lambertian {
    pub albedo: Albedo,
}
impl Lambertian {
    pub fn new(albedo: Albedo) -> Self {
        Self { albedo }
    }

    pub fn from(a: [f64; 3]) -> Self {
        Self::new(Albedo::new(a[0], a[1], a[2]))
    }
}
impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &Hit) -> Option<Scatter> {
        let scatter_dir = hit.normal + Vec3::rand_unit_ball();
        let scattered = Ray::new(hit.point, scatter_dir);
        // TODO: Make `Scatter` carry a reference to albedo instead of cloning it
        Some(Scatter::new(self.albedo.clone(), scattered))
    }
}
