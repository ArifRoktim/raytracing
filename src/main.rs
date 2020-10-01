use std::f64;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use minifb::{Key, Window, WindowOptions};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

use raytracing::config;
use raytracing::{Color, CrateRng, HitList, Hittable, Ray, Screen, Vec3};

fn main() {
    #[allow(non_snake_case)]
    let CFG: &'static _ = config::GLOBAL();

    let mut rng = match CFG.seed {
        Some(seed) => CrateRng::seed_from_u64(seed),
        None => CrateRng::from_entropy(),
    };

    let width = CFG.width.get();
    let height = CFG.height.get();
    let (camera, world) = CFG.scene.create(&mut rng);

    let mut screen = Screen::new(width, height);
    let rows_done = Arc::new(AtomicUsize::new(0));

    let thread_progress = rows_done.clone();
    // Spawn a new thread for monitoring progress.
    let progress = thread::spawn(move || {
        let mut time = Instant::now();
        loop {
            let delta = time.elapsed();
            if delta < CFG.delay {
                thread::sleep(CFG.delay - delta);
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
            let mut rng = CrateRng::seed_from_u64(seed);
            for (x, pix) in row.iter_mut().enumerate() {
                let mut avg = Color::new(0., 0., 0.);
                for _ in 0..CFG.samples.get() {
                    let (rand_i, rand_j): (f64, f64) = if !CFG.antialias {
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
                avg /= CFG.samples.get() as f64;
                *pix = avg;
            }
            counter.fetch_add(1, Ordering::SeqCst);
        });
    let time = time.elapsed();
    progress.join().unwrap();
    eprintln!("\nRending time elapsed: {:.2} seconds", time.as_secs_f64());

    // Display the screen
    let mut window = Window::new("Raytracing", width, height, WindowOptions::default()).unwrap();
    window.limit_update_rate(Some(CFG.delay));
    let buffer = screen.encode();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, screen.width, screen.height)
            .unwrap();
    }
}

/// Iterative version of the diffuse ray calculation.
/// Used because the recursive method blew the stack every time.
fn ray_color(world: &HitList, ray: &Ray, rng: &mut CrateRng) -> Color {
    let mut color = Color::default();
    let mut ray = ray.clone();
    let mut bounces = config::GLOBAL().max_depth.get();

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
