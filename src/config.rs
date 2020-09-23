use std::time::Duration;

use once_cell::sync::OnceCell;
use rand::Rng;
use structopt::StructOpt;
use strum::VariantNames;
use strum_macros::{EnumString, EnumVariantNames};

use crate::material::{Checkered, Dielectric, Lambertian, Metal};
use crate::shape::{MovingSphere, Sphere};
use crate::{Camera, Color, CrateRng, HitList, Vec3, BVH};

static CONFIG: OnceCell<Config> = OnceCell::new();

#[allow(non_snake_case)]
/// Return a `Config` built from command line args
pub fn GLOBAL() -> &'static Config {
    CONFIG.get_or_init(Config::from_args)
}

#[derive(Clone, Debug, StructOpt)]
pub struct Config {
    #[structopt(short, long, default_value = "1024", display_order = 0)]
    /// Width of render
    pub width: usize,

    #[structopt(short, long, default_value = "576", display_order = 1)]
    /// Height of render
    pub height: usize,

    // Run at 30 fps
    #[structopt(skip = Duration::from_secs_f64(1. / 30.))]
    /// Controls the framerate
    pub delay: Duration,

    #[structopt(
        help = "Disable antialiasing",
        short = "n",
        long = "no-aa",
        // Disable antialiasing if the flag is given at least once
        parse(from_occurrences = invert_bool),
    )]
    /// Controls antialiasing
    pub antialias: bool,

    #[structopt(short, long = "samples", default_value = "100")]
    /// Number of samples per pixel
    pub samples_per_pixel: u16,

    #[structopt(short, long = "max-depth", default_value = "100")]
    /// Maximum ray bounce depth
    pub max_ray_depth: u32,

    #[structopt(short = "r", long = "rng")]
    /// Use a specific seed for the rng.
    pub seed: Option<u64>,

    #[structopt(default_value = "Random", possible_values = Scene::VARIANTS)]
    /// The scene to render
    pub scene: Scene,
}

fn invert_bool(i: u64) -> bool {
    i == 0
}

#[derive(Copy, Clone, Debug, EnumString, EnumVariantNames, PartialEq)]
pub enum Scene {
    Random,
}

impl Scene {
    pub fn create(self, rng: &mut CrateRng) -> (Camera, HitList) {
        #[allow(non_snake_case)]
        let CFG: &'static _ = GLOBAL();

        use Scene::*;
        match self {
            Random => {
                let camera = Camera::builder()
                    .origin([13., 2., 3.])
                    .look_at([0., 0., 0.])
                    .vfov_degrees(20.)
                    .aspect_ratio(CFG.width as f64 / CFG.height as f64)
                    .aperture(0.1)
                    .focus_dist(10.)
                    .shutter_time(0.0..1.0)
                    .build();

                let mut world = HitList::new();
                let checker = Checkered::color([0.2, 0.3, 0.1], [0.9, 0.9, 0.9]);
                world.push(Sphere::from(
                    [0., -1000., 0.],
                    1000.,
                    Lambertian::new(checker),
                ));

                let mut bvh_list = HitList::new();
                for a in -11..11 {
                    for b in -11..11 {
                        let (x, z) = (0.9 * rng.gen::<f64>(), 0.9 * rng.gen::<f64>());
                        let center = Vec3::new(a as f64 + x, 0.2, b as f64 + z);
                        if (center - Vec3::new(4., 0.2, 0.)).norm() <= 0.9 {
                            continue;
                        }
                        let material = rng.gen::<f64>();
                        if material < 0.8 {
                            // diffuse
                            let material = Lambertian::new(Color::rand(rng) * Color::rand(rng));
                            let center2 = center + Vec3::new(0., rng.gen_range(0., 0.5), 0.);
                            bvh_list.push(MovingSphere::new(center, center2, 0.2, material));
                        } else if material < 0.95 {
                            // metal
                            let albedo = Color::rand_range(rng, 0.5, 1.);
                            let fuzz = rng.gen_range(0., 0.5);
                            bvh_list.push(Sphere::new(center, 0.2, Metal::new(albedo, fuzz)));
                        } else {
                            // glass
                            bvh_list.push(Sphere::new(center, 0.2, Dielectric::new(1.5)));
                        }
                    }
                }

                bvh_list.push(Sphere::from([0., 1., 0.], 1., Dielectric::new(1.5)));
                bvh_list.push(Sphere::from(
                    [-4., 1., 0.],
                    1.,
                    Lambertian::new(Color::new(0.4, 0.2, 0.1)),
                ));
                bvh_list.push(Sphere::from(
                    [4., 1., 0.],
                    1.,
                    Metal::from([0.7, 0.6, 0.5], 0.0),
                ));

                let bvh = BVH::from_list(bvh_list, &(0.0..1.), rng);
                world.push(bvh);

                (camera, world)
            }
        }
    }
}

#[cfg(test)]
mod parse_test {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn Random() {
        let scene = "Random".parse::<Scene>().unwrap();
        assert_eq!(scene, Scene::Random);
    }

    #[test]
    #[should_panic]
    fn random() {
        let _scene = "random".parse::<Scene>().unwrap();
    }
}
