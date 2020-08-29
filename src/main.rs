use minifb::{Key, Window, WindowOptions};

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

            let r = (r * 256.) as u8;
            let g = (g * 256.) as u8;
            let b = (b * 256.) as u8;
            *pix = [r, g, b];
        }
    }
    println!("\nDone!");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&screen.encode(), screen.width, screen.height)
            .unwrap();
    }
}

pub type Pixel = [u8; 3];
struct Screen {
    pub width: usize,
    pub height: usize,
    /// Flat buffer of 24-bit pixels with length of `width * height`
    pub buffer: Box<[Pixel]>,
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![[0; 3]; width * height].into_boxed_slice(),
        }
    }

    /// Encodes each Pixel into `0RGB`
    pub fn encode(&self) -> Box<[u32]> {
        self.buffer
            .iter()
            .map(|p| {
                let (r, g, b) = (p[0] as u32, p[1] as u32, p[2] as u32);
                (r << 16) | (g << 8) | b
            })
            .collect()
    }

    pub fn rows_mut(&mut self) -> std::slice::ChunksExactMut<Pixel> {
        self.buffer.chunks_exact_mut(self.width)
    }
}
