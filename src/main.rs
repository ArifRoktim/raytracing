use minifb::{Key, Window, WindowOptions};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rayon::prelude::*;

use raytracing::material::{Dielectric, Lambertian, Metal};
use raytracing::shape::{MovingSphere, Sphere};
use raytracing::{Camera, Color, CrateRng, HitList, Hittable, Ray, Screen, Vec3, BVH};
use raytracing::config::global as CONFIG;
use std::f64;
use std::io::{self, Write};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::thread;
use std::time::Instant;

fn main() {
    let mut rng = match CONFIG().seed {
        Some(seed) => SmallRng::seed_from_u64(seed),
        None => SmallRng::from_entropy()
    };

    let width = CONFIG().width;
    let height = CONFIG().height;
    let camera = Camera::builder()
        .origin([13., 2., 3.])
        .look_at([0., 0., 0.])
        .vfov_degrees(20.)
        .aspect_ratio(width as f64 / height as f64)
        .aperture(0.1)
        .focus_dist(10.)
        .shutter_time(0., 1.)
        .build();
    let world = random_scene(&mut rng);

    let mut screen = Screen::new(width, height);
    let rows_done = Arc::new(AtomicUsize::new(0));

    let thread_progress = rows_done.clone();
    // Spawn a new thread for monitoring progress.
    let progress = thread::spawn(move || {
        let mut time = Instant::now();
        loop {
            let delta = time.elapsed();
            if delta < CONFIG().delay {
                thread::sleep(CONFIG().delay - delta);
                time = Instant::now();
            }

            let rows = thread_progress.load(Ordering::SeqCst);
            // Clear the line before printing.
            // http://ascii-table.com/ansi-escape-sequences.php
            print!(
                "\x1B[K\rRows remaining: {}/{} ({:.2}%)",
                height - rows,
                height,
                (height - rows) as f64 / height as f64 * 100.,
            );
            io::stdout().flush().unwrap();

            // Exit when threads are done.
            if rows == height {
                break;
            }
        }
    });

    let seed: u64 = rng.gen();
    // Time the render
    let time = Instant::now();
    // Parallelize over each row
    screen
        .par_rows_mut()
        .enumerate()
        .for_each_with(rows_done, |counter, (y, row)| {
            // Complete each row and then increment the counter.

            // Initialize rng based off of row number
            let seed = seed.wrapping_add(1).wrapping_mul(y as u64);
            let mut rng = SmallRng::seed_from_u64(seed);
            for (x, pix) in row.iter_mut().enumerate() {
                let mut avg = Color::new(0., 0., 0.);
                for _ in 0..CONFIG().samples_per_pixel {
                    let (rand_i, rand_j): (f64, f64) = if !CONFIG().antialias {
                        (0., 0.)
                    } else {
                        (rng.gen(), rng.gen())
                    };
                    let i = (x as f64 + rand_i) / (width as f64 - 1.);
                    let j = 1. - (y as f64 + rand_j) / (height as f64 - 1.);

                    let ray = camera.get_ray(i, j, &mut rng);
                    let sample = ray_color(&world, &ray, &mut rng);
                    avg += sample;
                }
                avg /= CONFIG().samples_per_pixel as f64;
                *pix = avg;
            }
            counter.fetch_add(1, Ordering::SeqCst);
        });
    let time = time.elapsed();

    // Display the screen
    let mut window = Window::new("Raytracing", width, height, WindowOptions::default()).unwrap();
    window.limit_update_rate(Some(CONFIG().delay));
    let buffer = screen.encode();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, screen.width, screen.height)
            .unwrap();
    }

    progress.join().unwrap();
    eprintln!("\nRending time elapsed: {:.2} seconds", time.as_secs_f64());
}

/// Iterative version of the diffuse ray calculation.
/// Used because the recursive method blew the stack every time.
fn ray_color(world: &HitList, ray: &Ray, rng: &mut CrateRng) -> Color {
    let mut color = Color::default();
    let mut ray = ray.clone();
    let mut bounces = CONFIG().max_ray_depth;

    // NOTE: Tweak the beginning of the range to deal with shadow acne.
    while let Some(hit) = world.hit(&ray, &(0.001..f64::INFINITY)) {
        if let Some(scatter) = hit.material.scatter(&ray, &hit, rng) {
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
    let sky = (1. - t) * Color::new(1., 1., 1.) + t * Color::new(0.5, 0.7, 1.);

    sky * color
}

fn random_scene(rng: &mut CrateRng) -> HitList {
    let mut world = HitList::new();
    world.push(Sphere::from(
        [0., -1000., 0.],
        1000.,
        Lambertian::from([0.5; 3]),
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
                bvh_list.push(MovingSphere::new(center, center2, 0., 1., 0.2, material));
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
        Lambertian::from([0.4, 0.2, 0.1]),
    ));
    bvh_list.push(Sphere::from(
        [4., 1., 0.],
        1.,
        Metal::from([0.7, 0.6, 0.5], 0.0),
    ));

    let bvh = BVH::from_list(bvh_list, &(0.0..1.), rng);
    world.push(bvh);

    world
}
