use crate::{Color, CrateRng, Hit, Ray, Vec3};
use rand::Rng;

pub struct Scatter {
    pub albedo: Color,
    pub ray: Ray,
}
impl Scatter {
    pub fn new(albedo: Color, ray: Ray) -> Self {
        Self { albedo, ray }
    }
}

pub trait Material {
    /// A material will either absorb a ray (`None`) or scatter it.
    fn scatter(&self, _ray: &Ray, _hit: &Hit, _rng: &mut CrateRng) -> Option<Scatter> {
        None
    }
}

pub struct Lambertian {
    pub albedo: Color,
}
impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }

    pub fn from(a: [f64; 3]) -> Self {
        Self::new(Color::new(a[0], a[1], a[2]))
    }
}
impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit: &Hit, rng: &mut CrateRng) -> Option<Scatter> {
        let scatter_dir = hit.normal + Vec3::rand_unit_sphere(rng);
        let scattered = Ray::new(hit.point, scatter_dir, ray.time);
        // TODO: Make `Scatter` carry a reference to albedo instead of cloning it
        Some(Scatter::new(self.albedo.clone(), scattered))
    }
}

pub struct Metal {
    pub albedo: Color,
    /// The fuzziness of the Metal. Is between `0.0` and `1.0`
    pub fuzz: f64,
}
impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        let fuzz = if fuzz > 1. { 1. } else { fuzz };
        Self { albedo, fuzz }
    }

    pub fn from(a: [f64; 3], fuzz: f64) -> Self {
        Self::new(Color::new(a[0], a[1], a[2]), fuzz)
    }
}
impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &Hit, rng: &mut CrateRng) -> Option<Scatter> {
        let fuzz = self.fuzz * Vec3::rand_unit_sphere(rng);
        let reflected = ray.dir.reflect(hit.normal) + fuzz;
        let mut scattered = Ray::new(hit.point, reflected, ray.time);

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

pub struct Dielectric {
    pub ref_index: f64,
}
impl Dielectric {
    pub fn new(ref_index: f64) -> Self {
        Self { ref_index }
    }

    pub fn schlick(cos: f64, eta_i_over_eta_t: f64) -> f64 {
        let r0 = (1. - eta_i_over_eta_t) / (1. + eta_i_over_eta_t);
        let r0 = r0 * r0;
        r0 + (1. - r0) * (1. - cos).powi(5)
    }
}
impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &Hit, rng: &mut CrateRng) -> Option<Scatter> {
        let eta_i_over_eta_t = if hit.front_face {
            1. / self.ref_index
        } else {
            self.ref_index
        };
        let unit_dir = Vec3::normalized(ray.dir);
        let cos_theta = (-unit_dir).dot(hit.normal).min(1.0);
        let sin_theta = (1. - cos_theta.powi(2)).sqrt();

        let dir = if eta_i_over_eta_t * sin_theta > 1.0
            || rng.gen::<f64>() < Self::schlick(cos_theta, eta_i_over_eta_t)
        {
            unit_dir.reflect(hit.normal)
        } else {
            unit_dir.refract(hit.normal, eta_i_over_eta_t)
        };

        let scattered = Ray::new(hit.point, dir, ray.time);
        Some(Scatter::new(Color::default(), scattered))
    }
}
