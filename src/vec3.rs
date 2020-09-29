use std::ops;

use anyhow::{ensure, Result};
use rand::Rng;
use rand_distr::{Distribution, Standard, Uniform};

use crate::CrateRng;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
const ERR_NORMED_0: &str = "Tried to normalize vector of length 0!";
impl Vec3 {
    pub const ORIGIN: Self = Self::new(0., 0., 0.);
    // The standard basis
    pub const UNIT_X: Self = Self::new(1., 0., 0.);
    pub const UNIT_Y: Self = Self::new(0., 1., 0.);
    pub const UNIT_Z: Self = Self::new(0., 0., 1.);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// # Example
    /// ```
    /// # use raytracing::vec3::Vec3;
    /// let a = Vec3::new(1., 2., 3.);
    /// let b = Vec3::normalized(a);
    /// assert_eq!(b.norm(), 1.);
    /// ```
    pub fn normalized(v: Vec3) -> Self {
        let normed = v / v.norm();
        // TODO: Measure perf impact of assert! vs debug_assert!
        debug_assert!(!normed.is_nan(), ERR_NORMED_0);
        normed
    }

    /// # Example
    /// ```
    /// # use raytracing::vec3::Vec3;
    /// let a = Vec3::new(0., 0., 0.);
    /// assert!(Vec3::checked_normalized(a).is_err());
    /// ```
    pub fn checked_normalized(v: Vec3) -> Result<Self> {
        let norm = v.norm();
        ensure!(norm != 0., ERR_NORMED_0);
        Ok(v / norm)
    }

    /// Samples uniformly from the surface of the unit sphere in three dimensions.
    pub fn rand_unit_sphere(rng: &mut CrateRng) -> Self {
        rand_distr::UnitSphere.sample(rng).into()
    }

    /// Samples uniformly from the unit disc in the `x` and `y` dimensions. `z` is 0.
    pub fn rand_unit_disk(rng: &mut CrateRng) -> Self {
        let ret = rand_distr::UnitDisc.sample(rng);
        Self::new(ret[0], ret[1], 0.)
    }

    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }

    pub fn norm_squared(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    /// # Example
    /// ```
    /// # use raytracing::vec3::Vec3;
    /// let a = Vec3::new(4., 8., 10.);
    /// let b = Vec3::new(9., 2., 7.);
    /// assert_eq!(a.dot(b), 122.);
    /// ```
    pub fn dot(&self, rhs: Vec3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// # Example
    /// ```
    /// # use raytracing::vec3::Vec3;
    /// let a = Vec3::new(2., 3., 4.);
    /// let b = Vec3::new(5., 6., 7.);
    /// assert_eq!(a.cross(b), Vec3::new(-3., 6., -3.));
    /// ```
    pub fn cross(&self, rhs: Vec3) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn reflect(&self, normal: Vec3) -> Self {
        let unit_dir = Vec3::normalized(*self);
        unit_dir - 2. * unit_dir.dot(normal) * normal
    }

    pub fn refract(&self, normal: Vec3, eta_i_over_eta_t: f64) -> Self {
        let cos_theta = (-*self).dot(normal);
        let refract_parallel = eta_i_over_eta_t * (*self + cos_theta * normal);
        let refract_perp = -normal * (1. - refract_parallel.norm_squared()).sqrt();
        refract_parallel + refract_perp
    }

    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }
}

impl From<[f64; 3]> for Vec3 {
    fn from(v: [f64; 3]) -> Self {
        Self::new(v[0], v[1], v[2])
    }
}

impl ops::Index<Axis> for Vec3 {
    type Output = f64;

    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}
impl ops::IndexMut<Axis> for Vec3 {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
            Axis::Z => &mut self.z,
        }
    }
}

impl ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

/// Multiply the corresponding fields together
impl ops::Mul for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}
impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}
impl ops::MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}
impl ops::DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

#[derive(Copy, Clone)]
pub enum Axis {
    X,
    Y,
    Z,
}
impl Distribution<Axis> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Axis {
        let distr = Uniform::new(0u8, 3);
        match distr.sample(rng) {
            0 => Axis::X,
            1 => Axis::Y,
            2 => Axis::Z,
            _ => unreachable!(),
        }
    }
}
