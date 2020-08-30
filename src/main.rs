use minifb::{Key, Window, WindowOptions};

use raytracing::{shape::Sphere, Ray, Rgb, Vec3};

fn main() {
    let width = 1280;
    let height = 720;
    // The camera assumes width and height have aspect ratio of 16:9
    let horiz = Vec3::new(4., 0., 0.);
    let vert = Vec3::new(0., 2.25, 0.);
    let lower_left_corner = Vec3::ORIGIN - horiz / 2. - vert / 2. - Vec3::new(0., 0., 1.);

    let mut screen = Screen::new(width, height);
    let mut window = Window::new("Raytracing", width, height, WindowOptions::default()).unwrap();

    for (y, row) in screen.rows_mut().enumerate() {
        if y % 100 == 0 {
            print!("\rScanlines remaining: {}", height - y);
        }
        let j = (height - y - 1) as f64 / (height as f64 - 1.);

        for (x, pix) in row.iter_mut().enumerate() {
            let i = x as f64 / (width as f64 - 1.);
            let ray = Ray::new(Vec3::ORIGIN, lower_left_corner + i * horiz + j * vert);

            *pix = ray_color(&ray);
        }
    }
    println!("\nDone!");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&screen.encode(), screen.width, screen.height)
            .unwrap();
    }
}

fn ray_color(r: &Ray) -> Rgb {
    // Hardcoded sphere
    let sphere = Sphere::new(Vec3::new(0., 0., -1.), 0.5);
    if let Some(t) = sphere.hit(r) {
        let hit = Vec3::normalized(r.at(t) - sphere.center);
        return Rgb::f64(0.5 * (hit.x + 1.), 0.5 * (hit.y + 1.), 0.5 * (hit.z + 1.));
    }
    let unit_dir = Vec3::normalized(r.dir);
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
