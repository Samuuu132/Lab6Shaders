use raylib::prelude::*;

/// Convierte un color 0..1 (se recorta) a bytes RGBA.
pub fn to_rgba(rgb: Vector3) -> [u8; 4] {
    [
        (rgb.x.clamp(0.0, 1.0) * 255.0) as u8,
        (rgb.y.clamp(0.0, 1.0) * 255.0) as u8,
        (rgb.z.clamp(0.0, 1.0) * 255.0) as u8,
        255,
    ]
}

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // RGBA
    pub background: Color,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize, background: Color) -> Self {
        let mut fb = Framebuffer { width, height, pixels: vec![0; width * height * 4], background };
        fb.clear();
        fb
    }

    pub fn clear(&mut self) {
        let b = self.background;
        for p in self.pixels.chunks_exact_mut(4) {
            p.copy_from_slice(&[b.r, b.g, b.b, 255]);
        }
    }


    pub fn save_png(&self, path: &str) {
        let mut img = Image::gen_image_color(self.width as i32, self.height as i32, Color::BLACK);
        for y in 0..self.height {
            for x in 0..self.width {
                let i = (y * self.width + x) * 4;
                let c = Color::new(self.pixels[i], self.pixels[i + 1], self.pixels[i + 2], 255);
                img.draw_pixel(x as i32, y as i32, c);
            }
        }
        img.export_image(path);
    }
}
