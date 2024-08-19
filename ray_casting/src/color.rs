use std::fmt;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r.clamp(0, 255),
            g: g.clamp(0, 255),
            b: b.clamp(0, 255),
        }
    }

    pub fn from_hex(hex: u32) -> Color {
        let r = ((hex >> 16) & 0xFF) as u8;
        let g = ((hex >> 8) & 0xFF) as u8;
        let b = (hex & 0xFF) as u8;
        Color { r, g, b }
    }

    pub fn to_hex(&self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, color2: Color) -> Color {
        let r = self.r.saturating_add(color2.r);
        let g = self.g.saturating_add(color2.g);
        let b = self.b.saturating_add(color2.b);

        Color { r, g, b }
    }
}

impl Sub for Color {
    type Output = Color;

    fn sub(self, color2: Color) -> Color {
        let r = self.r.saturating_sub(color2.r);
        let g = self.g.saturating_sub(color2.g);
        let b = self.b.saturating_sub(color2.b);

        Color { r, g, b }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, factor: f32) -> Color {
        let r = (self.r as f32 * factor).clamp(0.0, 255.0) as u8;
        let g = (self.g as f32 * factor).clamp(0.0, 255.0) as u8;
        let b = (self.b as f32 * factor).clamp(0.0, 255.0) as u8;
        Color { r, g, b }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Color(r: {}, g: {}, b: {})", self.r, self.g, self.b)
    }
}
