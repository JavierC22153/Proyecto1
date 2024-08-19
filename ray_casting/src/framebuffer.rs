use crate::color::Color;
use crate::bmp::write_bmp_file;

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Color>,
    pub background_color: Color,
    pub current_color: Color,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> FrameBuffer {
        let default_color = Color::new(255, 255, 255);
        let buffer = vec![default_color; width * height];
        FrameBuffer {
            width,
            height,
            buffer,
            background_color: default_color,
            current_color: default_color,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background_color);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.buffer[self.width * y + x] = self.current_color;
        }
    }

    pub fn draw_rectangle(&mut self, x: usize, y: usize, width: usize, height: usize) {
        // Asegúrate de que el rectángulo esté dentro del búfer
        let x_end = (x + width).min(self.width);
        let y_end = (y + height).min(self.height);

        for i in x..x_end {
            for j in y..y_end {
                self.buffer[self.width * j + i] = self.current_color;
            }
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn get_color(&self, x: usize, y: usize) -> Color {
        if x < self.width && y < self.height {
            self.buffer[self.width * y + x]
        } else {
            self.background_color // Return background color if out of bounds
        }
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn write_to_bmp(&self, file_path: &str) -> std::io::Result<()> {
        let buffer: Vec<u32> = self.buffer.iter().map(|c| c.to_hex()).collect();
        write_bmp_file(file_path, &buffer, self.width, self.height)
    }

    pub fn cast_buffer(&self) -> Vec<u32> {
        self.buffer.iter().map(|color| color.to_hex()).collect()
    }
}
