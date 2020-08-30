use minifb::{Key, Window, WindowOptions};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use raytracing::shape::Sphere;
use raytracing::{Camera, HitList, Hittable, Ray, Rgb, Screen, Vec3};
use std::f64;
use std::io::{self, Write};

const RESOLUTIONS: &[[usize; 2]] = &[
    [384, 216],   // 0
    [640, 360],   // 1
    [1024, 576],  // 2
    [1280, 720],  // 3
    [1600, 900],  // 4
    [1920, 1080], // 5
];
const DIM: [usize; 2] = RESOLUTIONS[2];
const SAMPLES_PER_PIXEL: u16 = 100;
const MAX_RECURSION: u32 = 2;

fn main() {
    let mut rng = SmallRng::from_entropy();

    let width = DIM[0];
    let height = DIM[1];
    let camera = Camera::default();

    let mut world = HitList::new();
    world.push(Sphere::from([0., 0., -1.], 0.5));
    world.push(Sphere::from([0., -100.5, -1.], 100.));

    let mut screen = Screen::new(width, height);
    for (y, row) in screen.rows_mut().enumerate() {
        if (height - y - 1) % 100 == 0 {
            print!("\rScanlines remaining: {:<3}", height - y - 1);
            io::stdout().flush().unwrap();
        }

        for (x, pix) in row.iter_mut().enumerate() {
            let mut color = [0u32; 3];
            for _ in 0..SAMPLES_PER_PIXEL {
                // Don't do antialiasing when only using 1 sample
                let (rand_i, rand_j): (f64, f64) = if SAMPLES_PER_PIXEL == 1 {
                    (0., 0.)
                } else {
                    (rng.gen(), rng.gen())
                };

                let i = (x as f64 + rand_i) / (width as f64 - 1.);
                let j = (y as f64 + rand_j) / (height as f64 - 1.);
                let j = 1. - j;

                let sample = ray_color(&world, &camera.get_ray(i, j), MAX_RECURSION, &mut rng);
                color[0] += sample.r as u32;
                color[1] += sample.g as u32;
                color[2] += sample.b as u32;
            }

            color[0] /= SAMPLES_PER_PIXEL as u32;
            color[1] /= SAMPLES_PER_PIXEL as u32;
            color[2] /= SAMPLES_PER_PIXEL as u32;
            *pix = Rgb::new(color[0] as u8, color[1] as u8, color[2] as u8);
        }
    }
    println!("\nDone!");

    let mut window = Window::new("Raytracing", width, height, WindowOptions::default()).unwrap();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&screen.encode(), screen.width, screen.height)
            .unwrap();
    }
}

fn ray_color(world: &HitList, ray: &Ray, max_depth: u32, rng: &mut impl Rng) -> Rgb {
    if max_depth == 0 {
        return Rgb::f64(0., 0., 0.);
    }

    if let Some(t) = world.hit(ray, &(0.0..std::f64::INFINITY)) {
        let target = t.point + t.normal + Vec3::rand_unit_ball(rng);
        return 0.5 *
            ray_color(world, &Ray::new(t.point, target - t.point), MAX_RECURSION - 1, rng);
    }

    let unit_dir = Vec3::normalized(ray.dir);
    let t = 0.5 * (unit_dir.y + 1.);
    (1. - t) * Rgb::f64(1., 1., 1.) + t * Rgb::f64(0.5, 0.7, 1.)
}
