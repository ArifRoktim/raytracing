use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;
use std::ops::Range;

use rand::Rng;

use crate::shape::Dummy;
use crate::{Axis, CrateRng, Material, Ray, Vec3};

pub struct Hit<'a> {
    pub point: Vec3,
    /// A unit-length normal vector
    pub normal: Vec3,
    /// Time of hit
    pub time: f64,
    /// Hit the front face or back face of object
    pub front_face: bool,
    /// The material that was hit
    pub material: &'a dyn Material,
    pub u: f64,
    pub v: f64,
}
impl<'a> Hit<'a> {
    pub fn new(
        point: Vec3,
        normal: Vec3,
        t: f64,
        front_face: bool,
        material: &'a dyn Material,
        u: f64,
        v: f64,
    ) -> Self {
        Self {
            point,
            normal,
            time: t,
            front_face,
            material,
            u,
            v,
        }
    }

    pub fn ray(
        point: Vec3,
        mut normal: Vec3,
        t: f64,
        ray: &Ray,
        material: &'a dyn Material,
        u: f64,
        v: f64,
    ) -> Self {
        // Dot product is negative when ray hits back face
        let front_face = ray.dir.dot(normal) < 0.;
        // Make suface normal always point against incident ray
        if !front_face {
            normal *= -1.;
        }
        Self::new(point, normal, t, front_face, material, u, v)
    }
}

pub trait Hittable: Sync + Debug {
    /// Returns the hit determined by a ray. If there is no hit or the hit's time isn't contained
    /// by `hit_time`, returns `None`.
    fn hit(&self, ray: &Ray, hit_time: &Range<f64>) -> Option<Hit>;
    /// Returns the bounding box for the `Hittable`.  
    /// `shutter_time` affects the bounding_box of moving `Hittable`s (e.g. `MovingSphere`).
    fn bounding_box(&self, shutter_time: &Range<f64>) -> Option<AABB>;
}

#[derive(Default, Debug)]
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
    fn hit(&self, ray: &Ray, hit_time: &Range<f64>) -> Option<Hit> {
        let mut range = hit_time.clone();
        let mut closest = None;
        for obj in &self.0 {
            if let Some(hit) = obj.hit(ray, &range) {
                range.end = hit.time;
                closest = Some(hit);
            }
        }
        closest
    }

    fn bounding_box(&self, shutter_time: &Range<f64>) -> Option<AABB> {
        if self.0.is_empty() {
            return None;
        }

        let mut ret_bound: Option<AABB> = None;
        for obj in &self.0 {
            if let Some(bound_box) = obj.bounding_box(shutter_time) {
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

    pub fn hit(&self, ray: &Ray, hit_time: &Range<f64>) -> bool {
        let mut range = hit_time.clone();

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

    fn compare_axis(&self, other: &AABB, axis: Axis) -> Ordering {
        self.min[axis].partial_cmp(&other.min[axis]).unwrap()
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
    pub fn new(bound_box: AABB, left: Box<dyn Hittable>, right: Box<dyn Hittable>) -> Self {
        Self {
            bound_box,
            left,
            right,
        }
    }

    pub fn from(
        left: Box<dyn Hittable>,
        right: Box<dyn Hittable>,
        shutter_time: &Range<f64>,
    ) -> Self {
        let l_box = left.bounding_box(shutter_time);
        let r_box = right.bounding_box(shutter_time);

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
        shutter_time: &Range<f64>,
        rng: &mut CrateRng,
    ) -> Self {
        let err_msg = "No bounding box in BVH construction!";

        // Only 1 available hittable for BVH node. Make the other one a dummy hittable.
        if hitlist.len() == 1 {
            // Make the left node the Dummy so less work is done in BVH::hit()
            let left = Box::new(Dummy {});
            let right = hitlist.pop().unwrap();
            let bound_box = right.bounding_box(shutter_time).expect(err_msg);
            return Self::new(bound_box, left, right);
        }

        let (left, right);
        if hitlist.len() == 2 {
            left = hitlist.pop().unwrap();
            right = hitlist.pop().unwrap();
        } else {
            hitlist.sort_unstable_by(|a, b| {
                let axis = rng.gen();
                let a = a.bounding_box(shutter_time).expect(err_msg);
                let b = b.bounding_box(shutter_time).expect(err_msg);
                a.compare_axis(&b, axis)
            });
            let second_half = hitlist.split_off(hitlist.len() / 2);
            left = Box::new(Self::inner_list(hitlist, shutter_time, rng));
            right = Box::new(Self::inner_list(second_half, shutter_time, rng));
        }

        Self::from(left, right, shutter_time)
    }
}
impl Hittable for BVH {
    fn hit(&self, ray: &Ray, hit_time: &Range<f64>) -> Option<Hit> {
        if !self.bound_box.hit(ray, hit_time) {
            return None;
        }

        let mut range = hit_time.clone();
        let hit_left = if let Some(hit) = self.left.hit(ray, &range) {
            // Change range so next hit must be closer
            range.end = hit.time;
            Some(hit)
        } else {
            None
        };

        if let Some(right_hit) = self.right.hit(ray, &range) {
            return Some(right_hit);
        }

        hit_left
    }

    fn bounding_box(&self, _shutter_time: &Range<f64>) -> Option<AABB> {
        Some(self.bound_box.clone())
    }
}
