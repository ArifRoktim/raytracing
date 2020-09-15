use crate::{Axis, CrateRng, Material, Ray, Vec3};
use rand::Rng;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;
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

pub trait Hittable: Send + Sync + Debug {
    fn hit(&self, _ray: &Ray, _range: &Range<f64>) -> Option<Hit> {
        None
    }

    fn bounding_box(&self, _range: &Range<f64>) -> Option<AABB> {
        None
    }
}

#[derive(Default, Debug)]
pub struct HitList(pub Vec<Box<dyn Hittable>>);
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

    fn bounding_box(&self, range: &Range<f64>) -> Option<AABB> {
        if self.0.is_empty() {
            return None;
        }

        let mut ret_bound: Option<AABB> = None;
        for obj in &self.0 {
            if let Some(bound_box) = obj.bounding_box(range) {
                // Compute bounding box
                if let Some(ret) = &mut ret_bound {
                    *ret = ret.surrounding(&bound_box);
                } else {
                    ret_bound = Some(bound_box);
                }
            } else {
                // Hittable doesn't have a bounding box, so not possible for
                // the list to have one.
                return None;
            }
        }

        ret_bound
    }
}

/// Axis-Aligned Bounding Box
#[derive(Clone, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}
impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn surrounding(&self, other: &AABB) -> Self {
        let small = Vec3::new(
            self.min.x.min(other.min.x),
            self.min.y.min(other.min.y),
            self.min.z.min(other.min.z),
        );
        let big = Vec3::new(
            self.max.x.max(other.max.x),
            self.max.y.max(other.max.y),
            self.max.z.max(other.max.z),
        );

        AABB::new(small, big)
    }

    pub fn hit(&self, ray: &Ray, range: &Range<f64>) -> bool {
        let mut range = range.clone();

        let mut hit = |axis| {
            let inv_dir = 1.0 / ray.dir[axis];
            let mut t0 = (self.min[axis] - ray.origin[axis]) * inv_dir;
            let mut t1 = (self.max[axis] - ray.origin[axis]) * inv_dir;
            if inv_dir < 0. {
                mem::swap(&mut t0, &mut t1);
            }
            range.start = range.start.max(t0);
            range.end = range.end.min(t1);

            range.end > range.start
        };

        if !hit(Axis::X) || !hit(Axis::Y) || !hit(Axis::Z) {
            return false;
        }

        true
    }

    fn rand_axis_compare(rng: &mut CrateRng) -> Box<dyn Fn(&AABB, &AABB) -> Ordering> {
        let axis: Axis = rng.gen();
        Box::new(match axis {
            Axis::X => |a: &AABB, b: &AABB| a.compare_x(b),
            Axis::Y => |a: &AABB, b: &AABB| a.compare_y(b),
            Axis::Z => |a: &AABB, b: &AABB| a.compare_z(b),
        })
    }
    fn compare_x(&self, other: &AABB) -> Ordering {
        self.min.x.partial_cmp(&other.min.x).unwrap()
    }
    fn compare_y(&self, other: &AABB) -> Ordering {
        self.min.y.partial_cmp(&other.min.y).unwrap()
    }
    fn compare_z(&self, other: &AABB) -> Ordering {
        self.min.z.partial_cmp(&other.min.z).unwrap()
    }
}

/// Bounding Volume Heirarchy
#[derive(Debug)]
pub struct BVH {
    bound_box: AABB,
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
}
impl BVH {
    pub fn new(
        bound_box: AABB,
        left: Box<dyn Hittable>,
        right: Box<dyn Hittable>,
    ) -> Self {
        Self {
            bound_box,
            left,
            right,
        }
    }

    pub fn from(
        left: Box<dyn Hittable>,
        right: Box<dyn Hittable>,
        shutter_range: &Range<f64>,
    ) -> Self {
        let l_box = left.bounding_box(shutter_range);
        let r_box = right.bounding_box(shutter_range);

        let bound_box = match (l_box, r_box) {
            (Some(l_box), Some(r_box)) => l_box.surrounding(&r_box),
            _ => panic!("No bounding box in BVH construction!"),
        };
        Self::new(bound_box, left, right)
    }

    /// Construct the BVH
    pub fn from_list(hitlist: HitList, shutter_time: &Range<f64>, rng: &mut CrateRng) -> Self {
        Self::inner_list(hitlist.0, shutter_time, rng)
    }

    // Recursively create the tree
    fn inner_list(
        mut hitlist: Vec<Box<dyn Hittable>>,
        shutter_range: &Range<f64>,
        rng: &mut CrateRng,
    ) -> Self {
        let err_msg = "No bounding box in BVH construction!";

        // Only 1 available hittable for BVH node. Make the other one a dummy hittable.
        if hitlist.len() == 1 {
            // Make the left node the Dummy so less work is done in BVH::hit()
            let left = Box::new(Dummy);
            let right = hitlist.pop().unwrap();
            let bound_box = right.bounding_box(shutter_range).expect(err_msg);
            return Self::new(bound_box, left, right);
        }

        let (left, right);
        if hitlist.len() == 2 {
            left = hitlist.pop().unwrap();
            right = hitlist.pop().unwrap();
        } else {
            hitlist.sort_unstable_by(|a, b| {
                let a = a.bounding_box(shutter_range).expect(err_msg);
                let b = b.bounding_box(shutter_range).expect(err_msg);
                let cmp = AABB::rand_axis_compare(rng);
                cmp(&a, &b)
            });
            let second_half = hitlist.split_off(hitlist.len() / 2);
            left = Box::new(Self::inner_list(hitlist, shutter_range, rng));
            right = Box::new(Self::inner_list(second_half, shutter_range, rng));
        }

        Self::from(left, right, shutter_range)
    }
}
impl Hittable for BVH {
    fn hit(&self, ray: &Ray, range: &Range<f64>) -> Option<Hit> {
        if !self.bound_box.hit(ray, range) {
            return None;
        }

        let mut range = range.clone();
        let hit_left = if let Some(hit) = self.left.hit(ray, &range) {
            // Change range so next hit must be closer
            range.end = hit.t;
            Some(hit)
        } else {
            None
        };

        if let Some(right_hit) = self.right.hit(ray, &range) {
            return Some(right_hit)
        }

        hit_left
    }

    fn bounding_box(&self, _range: &Range<f64>) -> Option<AABB> {
        Some(self.bound_box.clone())
    }
}

// ===Shapes===
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
pub struct Dummy;
impl Hittable for Dummy {
    /// Dummy will never return a hit.
    fn hit(&self, _ray: &Ray, _range: &Range<f64>) -> Option<Hit> {
        None
    }
}
