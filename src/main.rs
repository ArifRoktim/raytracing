use minifb::{Key, Window, WindowOptions};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use raytracing::material::{Dielectric, Lambertian, Metal};
use raytracing::shape::Sphere;
use raytracing::{Albedo, Camera, HitList, Hittable, Ray, Rgb, Screen, Vec3};
use std::f64;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const UPDATE_DELAY: Duration = Duration::from_millis((1. / 30. * 1000.) as u64);
const RESOLUTIONS: &[[usize; 2]] = &[
    [384, 216],   // 0
    [640, 360],   // 1
    [1024, 576],  // 2
    [1280, 720],  // 3
    [1600, 900],  // 4
    [1920, 1080], // 5
];
const DIM: [usize; 2] = RESOLUTIONS[2];
const ANTIALIASING: bool = true;
const SAMPLES_PER_PIXEL: u16 = 100;
const MAX_RAY_BOUNCES: u32 = 100;

fn main() {
    let mut rng = thread_rng();

    let width = DIM[0];
    let height = DIM[1];
    let camera = Camera::from(
        [13., 2., 3.],
        [0., 0., 0.],
        None,
        20.,
        width as f64 / height as f64,
        0.1,
        Some(10.),
    );
    let world = random_scene(&mut rng);

    let mut screen = Screen::new(width, height);
    let mut time = Instant::now();
    for (y, row) in screen.rows_mut().enumerate() {
        if time.elapsed() > UPDATE_DELAY {
            let percent = (height - y - 1) as f64 / height as f64 * 100.;
            // http://ascii-table.com/ansi-escape-sequences.php
            print!(
                "\x1B[K\rScanlines remaining: {}/{} ({:.2}%)",
                height - y - 1,
                height,
                percent
            );
            io::stdout().flush().unwrap();
            time = Instant::now();
        }

        for (x, pix) in row.iter_mut().enumerate() {
            let mut color = [0u32; 3];
            for _ in 0..SAMPLES_PER_PIXEL {
                let (rand_i, rand_j): (f64, f64) = if !ANTIALIASING {
                    (0., 0.)
                } else {
                    (rng.gen(), rng.gen())
                };

                let i = (x as f64 + rand_i) / (width as f64 - 1.);
                let j = (y as f64 + rand_j) / (height as f64 - 1.);
                let j = 1. - j;

                let sample = ray_color(&world, &camera.get_ray(i, j), MAX_RAY_BOUNCES);
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
    window.limit_update_rate(Some(UPDATE_DELAY));
    let buffer = screen.encode(true);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, screen.width, screen.height)
            .unwrap();
    }
}

/// Iterative version of the diffuse ray calculation.
/// Used because the recursive method blew the stack every time.
fn ray_color(world: &HitList, ray: &Ray, mut bounces: u32) -> Rgb {
    let mut color = Albedo::default();
    let mut ray = ray.clone();

    // NOTE: Tweak the beginning of the range to deal with shadow acne.
    while let Some(hit) = world.hit(&ray, &(0.001..f64::INFINITY)) {
        if let Some(scatter) = hit.material.scatter(&ray, &hit) {
            color *= scatter.albedo;
            ray = scatter.ray;
        } else {
            // Ray got absorbed so no light is reflected.
            color *= 0.;
            break;
        }

        bounces -= 1;
        if bounces == 0 {
            color *= 0.;
            break;
        }
    }

    // Calculate color of the sky
    let unit_dir = Vec3::normalized(ray.dir);
    let t = 0.5 * (unit_dir.y + 1.);
    let sky = (1. - t) * Rgb::f64(1., 1., 1.) + t * Rgb::f64(0.5, 0.7, 1.);

    sky * color
}

fn random_scene(rng: &mut ThreadRng) -> HitList {
    let mut world = HitList::new();
    world.push(Sphere::from(
        [0., -1000., 0.],
        1000.,
        Lambertian::from([0.5; 3]),
    ));

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
                let albedo = Albedo::rand(rng) * Albedo::rand(rng);
                world.push(Sphere::new(center, 0.2, Lambertian::new(albedo)));
            } else if material < 0.98 {
                // metal
                let albedo = Albedo::rand_range(rng, 0.5, 1.);
                let fuzz = rng.gen_range(0., 0.5);
                world.push(Sphere::new(center, 0.2, Metal::new(albedo, fuzz)));
            } else {
                // glass
                world.push(Sphere::new(center, 0.2, Dielectric::new(1.5)));
            }
        }
    }

    world.push(Sphere::from([0., 1., 0.], 1., Dielectric::new(1.5)));
    world.push(Sphere::from(
        [-4., 1., 0.],
        1.,
        Lambertian::from([0.4, 0.2, 0.1]),
    ));
    world.push(Sphere::from(
        [4., 1., 0.],
        1.,
        Metal::from([0.7, 0.6, 0.5], 0.0),
    ));

    world
}
