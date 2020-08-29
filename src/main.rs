use minifb::{Key, Window, WindowOptions};

use raytracing::color::Rgb;

fn main() {
    let mut screen = Screen::new(800, 600);
    let mut window = Window::new("Raytracing", 800, 600, WindowOptions::default()).unwrap();

    let (width, height) = (screen.width, screen.height);
    for (y, row) in screen.rows_mut().enumerate() {
        if y % 100 == 0 {
            print!("\rScanlines remaining: {}", height - y);
        }

        for (x, pix) in row.iter_mut().enumerate() {
            let r = x as f64 / (width as f64 - 1.);
            let g = 1. - (y as f64 / (height as f64 - 1.));
            let b = 0.25;

            *pix = Rgb::f64(r, g, b);
        }
    }
    println!("\nDone!");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&screen.encode(), screen.width, screen.height)
            .unwrap();
    }
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
