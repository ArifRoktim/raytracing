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
        let scatter_dir = hit.normal + Vec3::rand_unit_sphere();
        let scattered = Ray::new(hit.point, scatter_dir);
        // TODO: Make `Scatter` carry a reference to albedo instead of cloning it
        Some(Scatter::new(self.albedo.clone(), scattered))
    }
}

pub struct Metal {
    pub albedo: Albedo,
    /// The fuzziness of the Metal. Is between `0.0` and `1.0`
    pub fuzz: f64,
}
impl Metal {
    pub fn new(albedo: Albedo, fuzz: f64) -> Self {
        let fuzz = if fuzz > 1. { 1. } else { fuzz };
        Self { albedo, fuzz }
    }

    pub fn from(a: [f64; 3], fuzz: f64) -> Self {
        Self::new(Albedo::new(a[0], a[1], a[2]), fuzz)
    }
}
impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<Scatter> {
        let fuzz = self.fuzz * Vec3::rand_unit_sphere();
        let reflected = ray.dir.reflect(hit.normal) + fuzz;
        let mut scattered = Ray::new(hit.point, reflected);

        if scattered.dir.dot(hit.normal) > 0. {
            Some(Scatter::new(self.albedo.clone(), scattered))
        } else {
            // The fuzz scattered below the surface. Correct it.
            scattered.dir -= 2. * fuzz;
            assert!(scattered.dir.dot(hit.normal) > 0.);
            Some(Scatter::new(self.albedo.clone(), scattered))
        }
    }
}
