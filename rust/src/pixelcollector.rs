use crate::{canvas::Pixel, color::Color};

enum PixelCollectorKind {
    Binary,
    Text,
}

pub struct PixelCollector {
    data: Vec<u8>,
    kind: PixelCollectorKind,
}

impl PixelCollector {
    pub fn new_binary() -> Self {
        Self {
            data: Vec::new(),
            kind: PixelCollectorKind::Binary,
        }
    }

    pub fn new_text() -> Self {
        Self {
            data: Vec::new(),
            kind: PixelCollectorKind::Text,
        }
    }

    pub fn add_colored_pixel(&mut self, x: u16, y: u16, color: Color) {
        let alpha = if let Some(alpha) = color.a {
            alpha
        } else {
            0xFF
        };
        self.add_pixel_raw(x, y, (color.r, color.g, color.b, alpha));
    }

    pub fn add_pixel(&mut self, x: u16, y: u16, pixel: Pixel) {
        self.add_pixel_raw(x, y, (pixel.r, pixel.g, pixel.b, pixel.a))
    }

    pub fn add_pixel_raw(&mut self, x: u16, y: u16, pixel_data: (u8, u8, u8, u8)) {
        let (r, g, b, a) = pixel_data;

        match self.kind {
            PixelCollectorKind::Binary => {
                let data = &mut self.data;
                data.extend_from_slice(b"PXB ");

                x.to_le_bytes().iter().for_each(|b| data.push(*b));
                y.to_le_bytes().iter().for_each(|b| data.push(*b));

                data.push(r);
                data.push(g);
                data.push(b);
                data.push(a);
                data.push(b'\n');
            }
            PixelCollectorKind::Text => {
                self.data.extend_from_slice(
                    format!("PX {} {} {:02X}{:02X}{:02X}{:02X}\n", x, y, r, g, b, a).as_bytes(),
                );
            }
        }
    }

    pub fn as_bytes(self) -> Vec<u8> {
        self.data
    }
}
