use minifb::{Key, Window, WindowOptions};
use raytracing::shape::Sphere;
use raytracing::{HitList, Hittable, Ray, Rgb, Vec3};
use std::f64;
use std::io::{self, Write};

const RESOLUTIONS: &[[usize; 2]] = &[
    [384, 216],   // 0
    [640, 360],   // 1
    [1280, 720],  // 2
    [1600, 900],  // 3
    [1920, 1080], // 4
];
const DIM: [usize; 2] = RESOLUTIONS[2];

fn main() {
    let width = DIM[0];
    let height = DIM[1];
    // The camera assumes width and height have aspect ratio of 16:9
    let horiz = Vec3::new(4., 0., 0.);
    let vert = Vec3::new(0., 2.25, 0.);
    let lower_left_corner = Vec3::ORIGIN - horiz / 2. - vert / 2. - Vec3::new(0., 0., 1.);

    let mut world = HitList::new();
    world.push(Sphere::from([0., 0., -1.], 0.5));
    world.push(Sphere::from([0., -100.5, -1.], 100.));

    let mut screen = Screen::new(width, height);
    for (y, row) in screen.rows_mut().enumerate() {
        if y % 100 == 0 {
            print!("\rScanlines remaining: {} ", height - y);
            io::stdout().flush().unwrap();
        }
        let j = (height - y - 1) as f64 / (height as f64 - 1.);

        for (x, pix) in row.iter_mut().enumerate() {
            let i = x as f64 / (width as f64 - 1.);
            let ray = Ray::new(Vec3::ORIGIN, lower_left_corner + i * horiz + j * vert);

            *pix = ray_color(&world, &ray);
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

fn ray_color(world: &HitList, ray: &Ray) -> Rgb {
    if let Some(mut t) = world.hit(ray, &(0.0..std::f64::INFINITY)) {
        t.normal *= 0.5;
        t.normal += Vec3::new(0.5, 0.5, 0.5);
        return Rgb::f64(t.normal.x, t.normal.y, t.normal.z);
    }
    let unit_dir = Vec3::normalized(ray.dir);
    let t = 0.5 * (unit_dir.y + 1.);
    (1. - t) * Rgb::f64(1., 1., 1.) + t * Rgb::f64(0.5, 0.7, 1.)
}

struct Screen {
    pub width: usize,
    pub height: usize,
    /// Flat buffer of 24-bit pixels with length of `width * height`
    pub buffer: Box<[Rgb]>,
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![Rgb::default(); width * height].into(),
        }
    }

    /// Encodes each Pixel into `0RGB`
    pub fn encode(&self) -> Box<[u32]> {
        self.buffer
            .iter()
            .map(|p| {
                let (r, g, b) = (p.r as u32, p.g as u32, p.b as u32);
                (r << 16) | (g << 8) | b
            })
            .collect()
    }

    pub fn rows_mut(&mut self) -> std::slice::ChunksExactMut<Rgb> {
        self.buffer.chunks_exact_mut(self.width)
    }
}
