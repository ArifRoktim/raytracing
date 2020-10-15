use std::fmt::Debug;
use std::sync::Arc;

use rand::distributions::{Distribution, Uniform};
use rand::{Rng, SeedableRng};

use crate::{Color, CrateRng, F64Ext, Hit, Ray, Vec3};

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

pub trait Material: Sync + Debug {
    /// A material will either absorb a ray (`None`) or scatter it.
    fn scatter(&self, ray: &Ray, hit: &Hit, rng: &mut CrateRng) -> Option<Scatter>;
}

#[derive(Debug)]
/// Diffuse reflection
pub struct Lambertian<T> {
    pub albedo: T,
}
impl<T> Lambertian<T> {
    pub fn new(albedo: T) -> Self {
        Self { albedo }
    }
}
impl<T: Texture> Material for Lambertian<T> {
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

// ===== Textures =====
pub trait Texture: Sync + Debug {
    fn value(&self, u: f64, v: f64, point: Vec3) -> Color;
}

#[derive(Clone, Debug)]
pub struct Checkered<O, E> {
    pub freq: f64,
    pub odd: O,
    pub even: E,
}
impl<O, E> Checkered<O, E> {
    pub fn new(freq: f64, odd: O, even: E) -> Self {
        Self { freq, odd, even }
    }
}
impl Checkered<Color, Color> {
    pub fn color<T: Into<Color>, U: Into<Color>>(freq: f64, odd: T, even: U) -> Self {
        Self {
            freq,
            even: even.into(),
            odd: odd.into(),
        }
    }
}
impl<O: Texture, E: Texture> Texture for Checkered<O, E> {
    fn value(&self, u: f64, v: f64, point: Vec3) -> Color {
        let mut parity = (point.x * self.freq).sin() < 0.;
        parity ^= (point.y * self.freq).sin() < 0.;
        parity ^= (point.z * self.freq).sin() < 0.;
        if parity {
            self.odd.value(u, v, point)
        } else {
            self.even.value(u, v, point)
        }
    }
}

type Callback = dyn Fn(&ValueNoise, Vec3) -> f64 + Send + Sync;
/// 3D Value Noise
pub struct ValueNoise {
    randoms: [f64; Self::SIZE],
    /// The permutations table.
    perms: [u16; Self::SIZE * 2],
    freq: f64,
    callback: Option<Box<Callback>>,
}
impl ValueNoise {
    const SIZE: usize = 256;
    /// Used for calculating the modulo/euclidean remainder by 256.
    const MASK: isize = (Self::SIZE - 1) as isize;

    pub fn new<T: Into<Option<u64>>>(seed: T, freq: f64) -> Self {
        let mut rng = match seed.into() {
            Some(seed) => CrateRng::seed_from_u64(seed),
            None => CrateRng::from_entropy(),
        };

        let mut randoms = [0.0; Self::SIZE];
        let mut perms = [0; Self::SIZE * 2];

        for i in 0..Self::SIZE {
            randoms[i] = rng.gen();
            perms[i] = i as u16;
        }

        let index = Uniform::new(0, Self::SIZE);
        for i in 0..Self::SIZE {
            let j = index.sample(&mut rng);
            perms.swap(i, j);
            perms[i + Self::SIZE] = perms[i];
        }

        Self {
            randoms,
            perms,
            freq,
            callback: None,
        }
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn hash(&self, x: isize, y: isize, z: isize) -> usize {
        let perm_xy = self.perms[x as usize] + y as u16;
        let plus_z = self.perms[perm_xy as usize] + z as u16;
        self.perms[plus_z as usize] as usize
    }

    pub fn eval(&self, p: Vec3) -> f64 {
        self.callback
            .as_ref()
            .map(|callback| (callback)(self, p))
            .unwrap_or_else(|| self.noise(p))
    }

    fn noise(&self, mut p: Vec3) -> f64 {
        p *= self.freq;

        let floor_p = p.map(|f| f.floor());
        let t = p - floor_p;
        let smooth = t.map(|f| f.smooth());

        // The 6 values that determine the cube enclosing the given point
        // Do bitwise AND to get the euclidean remainder/modulo by 256.
        let rx0 = floor_p.x as isize & Self::MASK;
        let ry0 = floor_p.y as isize & Self::MASK;
        let rz0 = floor_p.z as isize & Self::MASK;
        let rx1 = (rx0 + 1) & Self::MASK;
        let ry1 = (ry0 + 1) & Self::MASK;
        let rz1 = (rz0 + 1) & Self::MASK;

        // The 8 random values at the corners of said cube.
        let c000 = self.randoms[self.hash(rx0, ry0, rz0)];
        let c100 = self.randoms[self.hash(rx1, ry0, rz0)];
        let c010 = self.randoms[self.hash(rx0, ry1, rz0)];
        let c110 = self.randoms[self.hash(rx1, ry1, rz0)];

        let c001 = self.randoms[self.hash(rx0, ry0, rz1)];
        let c101 = self.randoms[self.hash(rx1, ry0, rz1)];
        let c011 = self.randoms[self.hash(rx0, ry1, rz1)];
        let c111 = self.randoms[self.hash(rx1, ry1, rz1)];

        // lerp along X axis
        let x00 = smooth.x.lerp(c000, c100);
        let x10 = smooth.x.lerp(c010, c110);
        let x01 = smooth.x.lerp(c001, c101);
        let x11 = smooth.x.lerp(c011, c111);

        // lerp along Y axis
        let y0 = smooth.y.lerp(x00, x10);
        let y1 = smooth.y.lerp(x01, x11);

        // finally lerp along Z axis
        smooth.z.lerp(y0, y1)
    }
}
/// Common noise patterns
impl ValueNoise {
    #[allow(non_snake_case)]
    /// Fractional brownian noise maker. Aka fractional brownian motion
    pub fn fBm(mut self, lacunarity: f64, gain: f64, layers: usize) -> Self {
        assert!(layers != 0, "fBm: Can't have 0 layers.");
        assert!(0. < gain && gain < 1., "fBm: Gain must be in range (0, 1).");
        // Get the maxiumum possible value of `sum` for later.
        // Equal to `layers` terms in the geometric series where "a=1, r=gain"
        let max = (1. - gain.powi(layers as i32)) / (1. - gain);

        // This callback will compute the fractal sum
        let callback = move |noise: &Self, mut p: Vec3| {
            let mut sum = 0.;
            let mut amplitude = 1.;
            for _ in 0..layers {
                sum += noise.noise(p) * amplitude;
                p *= lacunarity;
                amplitude *= gain;
            }

            // Normalize the sum to the range [0, 1]
            sum / max
        };

        self.callback = Some(Box::new(callback));
        self
    }

    /// bUmPY nOiSE makEr
    pub fn turbulence(mut self, lacunarity: f64, gain: f64, layers: usize) -> Self {
        assert!(layers != 0, "fBm: Can't have 0 layers.");
        assert!(0. < gain && gain < 1., "fBm: Gain must be in range (0, 1).");
        // Get the maxiumum possible value of `sum` for later.
        // Equal to `layers` terms in the geometric series where "a=1, r=gain"
        let max = (1. - gain.powi(layers as i32)) / (1. - gain);

        // This callback will compute the fractal sum
        let callback = move |noise: &Self, mut p: Vec3| {
            let mut sum = 0.;
            let mut amplitude = 1.;
            for _ in 0..layers {
                // mAke thE nOIsE bUmPY by making it signed, then taking its absolute value
                let layer = 2. * noise.noise(p) - 1.;
                sum += layer.abs() * amplitude;
                p *= lacunarity;
                amplitude *= gain;
            }

            // Normalize the sum to the range [0, 1]
            sum / max
        };

        self.callback = Some(Box::new(callback));
        self
    }

    /// Marbled noise
    pub fn marbled(self, lacunarity: f64, gain: f64, layers: usize) -> Self {
        let freq = self.freq;
        // Get an fBm callback to later get fractal noise from.
        let mut ret = self.fBm(lacunarity, gain, layers);
        let fbm = ret.callback.take().unwrap();

        let callback = move |noise: &Self, p: Vec3| {
            let noise = fbm(noise, p);
            // Perturb the phase of the sine function using the noise.
            let noise = 5. * freq * noise + p.z;
            // Normalize sine to range [0, 1]
            (noise.sin() + 1.) * 0.5
        };

        ret.callback = Some(Box::new(callback));
        ret
    }
}
impl Debug for ValueNoise {
    /// This struct's fields are too large to be printed.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueNoise { .. }").finish()
    }
}
impl Texture for ValueNoise {
    fn value(&self, _u: f64, _v: f64, point: Vec3) -> Color {
        Color::default() * self.eval(point)
    }
}

// ===== Blanket Impls ======
impl<T: Texture + Send + Debug> Texture for Arc<T> {
    fn value(&self, u: f64, v: f64, point: Vec3) -> Color {
        // Use fully qualified syntax to prevent recursion
        <T as Texture>::value(self, u, v, point)
    }
}
