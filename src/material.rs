use std::fmt::Debug;

use rand::Rng;

use crate::{Color, CrateRng, Hit, Ray, Vec3};

/// A scattered ray and its color information
pub struct Scatter {
    pub albedo: Color,
    pub ray: Ray,
}
impl Scatter {
    pub fn new(albedo: Color, ray: Ray) -> Self {
        Self { albedo, ray }
    }
}

pub trait Material: Send + Sync + Debug {
    /// A material will either absorb a ray (`None`) or scatter it.
    fn scatter(&self, ray: &Ray, hit: &Hit, rng: &mut CrateRng) -> Option<Scatter>;
}

#[derive(Debug)]
/// Diffuse reflection
pub struct Lambertian {
    // TODO: Optimize by replacing with an Enum
    pub albedo: Box<dyn Texture>,
}
impl Lambertian {
    pub fn new<T: Texture + 'static>(albedo: T) -> Self {
        Self {
            albedo: Box::new(albedo),
        }
    }
}
impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit: &Hit, rng: &mut CrateRng) -> Option<Scatter> {
        let scatter_dir = hit.normal + Vec3::rand_unit_sphere(rng);
        let scattered = Ray::new(hit.point, scatter_dir, ray.time);
        let albedo = self.albedo.value(hit.u, hit.v, hit.point);
        Some(Scatter::new(albedo, scattered))
    }
}

#[derive(Debug)]
pub struct Metal {
    pub albedo: Color,
    /// The fuzziness of the Metal. Is between `0.0` and `1.0`
    pub fuzz: f64,
}
impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        let fuzz = fuzz.min(1.);
        Self { albedo, fuzz }
    }

    pub fn from(a: [f64; 3], fuzz: f64) -> Self {
        Self::new(a.into(), fuzz)
    }
}
impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &Hit, rng: &mut CrateRng) -> Option<Scatter> {
        let fuzz = self.fuzz * Vec3::rand_unit_sphere(rng);
        let reflected = ray.dir.reflect(hit.normal) + fuzz;
        let mut scattered = Ray::new(hit.point, reflected, ray.time);

        if scattered.dir.dot(hit.normal) <= 0. {
            // NOTE: Deviating from the book here.
            // The fuzz scattered below the surface. Correct it.
            scattered.dir -= 2. * fuzz;
        }
        Some(Scatter::new(self.albedo, scattered))
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
/// Used for debugging. Sets albedo to black and the "scattered" ray to the incident ray.
pub struct DbgBlack {}
impl Material for DbgBlack {
    fn scatter(&self, ray: &Ray, _hit: &Hit, _rng: &mut CrateRng) -> Option<Scatter> {
        // Just return the in-ray with albedo set to black
        Some(Scatter::new(Color::new(0., 0., 0.), ray.clone()))
    }
}

pub trait Texture: Send + Sync + Debug {
    fn value(&self, u: f64, v: f64, point: Vec3) -> Color;
}

#[derive(Debug)]
pub struct Checkered {
    pub odd: Box<dyn Texture>,
    pub even: Box<dyn Texture>,
}
impl Checkered {
    pub fn new<T: Texture + 'static, U: Texture + 'static>(odd: T, even: U) -> Self {
        Self {
            even: Box::new(even),
            odd: Box::new(odd),
        }
    }

    pub fn color<T: Into<Color>, U: Into<Color>>(odd: T, even: U) -> Self {
        Self {
            even: Box::new(even.into()),
            odd: Box::new(odd.into()),
        }
    }
}

impl Texture for Checkered {
    fn value(&self, u: f64, v: f64, point: Vec3) -> Color {
        let freq = 10.;
        // TODO: Optimize maybe, since we only need the signs, not the actual products
        let sines = (point.x * freq).sin() * (point.y * freq).sin() * (point.z * freq).sin();
        if sines < 0. {
            self.odd.value(u, v, point)
        } else {
            self.even.value(u, v, point)
        }
    }
}
