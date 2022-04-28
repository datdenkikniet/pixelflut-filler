use std::str::FromStr;

use zstd::{
    stream::raw::{Encoder, Operation},
    zstd_safe::{InBuffer, OutBuffer},
};

use crate::{canvas::Pixel, color::Color};

enum PixelCollectorKind {
    Binary,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionKind {
    Zstd,
}

impl FromStr for CompressionKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s.to_lowercase().as_str() {
            "zstd" => Self::Zstd,
            _ => return Err("unknown compression type"),
        };
        Ok(kind)
    }
}

impl ToString for CompressionKind {
    fn to_string(&self) -> String {
        match self {
            CompressionKind::Zstd => "zstd".to_string(),
        }
    }
}

impl CompressionKind {
    pub fn compression_string(&self) -> Vec<u8> {
        // let val = match self {
        //     CompressionKind::Zstd => "ZSTD",
        // };

        format!("COMPRESS\n")
            .as_bytes()
            .iter()
            .map(|u| *u)
            .collect()
    }
}

pub struct PixelCollector {
    data: Vec<u8>,
    kind: PixelCollectorKind,
    compression_kind: Option<CompressionKind>,
}

impl PixelCollector {
    pub fn new_binary(compression_kind: Option<CompressionKind>) -> Self {
        Self {
            data: Vec::new(),
            kind: PixelCollectorKind::Binary,
            compression_kind,
        }
    }

    pub fn new_text(compression_kind: Option<CompressionKind>) -> Self {
        Self {
            data: Vec::new(),
            kind: PixelCollectorKind::Text,
            compression_kind,
        }
    }

    #[allow(unused)]
    pub fn add_colored_pixel(&mut self, x: u16, y: u16, color: Color) {
        let alpha = if let Some(alpha) = color.a {
            alpha
        } else {
            0xFF
        };
        self.add_pixel_raw(x, y, (color.r, color.g, color.b, alpha));
    }

    #[allow(unused)]
    pub fn add_pixel(&mut self, x: u16, y: u16, pixel: Pixel) {
        self.add_pixel_raw(x, y, (pixel.r, pixel.g, pixel.b, pixel.a))
    }

    pub fn add_pixel_raw(&mut self, x: u16, y: u16, pixel_data: (u8, u8, u8, u8)) {
        let (r, g, b, a) = pixel_data;

        match self.kind {
            PixelCollectorKind::Binary => {
                let data = &mut self.data;
                data.extend_from_slice(b"PB");

                x.to_le_bytes().iter().for_each(|b| data.push(*b));
                y.to_le_bytes().iter().for_each(|b| data.push(*b));

                data.push(r);
                data.push(g);
                data.push(b);
                data.push(a);
            }
            PixelCollectorKind::Text => {
                self.data.extend_from_slice(
                    format!("PX {} {} {:02X}{:02X}{:02X}{:02X}\n", x, y, r, g, b, a).as_bytes(),
                );
            }
        }
    }

    fn compress_zstd(self) -> (usize, Vec<u8>) {
        let in_buffer = &mut InBuffer::around(&self.data);
        let out = &mut vec![0u8; self.data.len() + 1024];
        let out_buffer = &mut OutBuffer::around(out);
        let mut encoder = Encoder::new(1).unwrap();

        encoder.run(in_buffer, out_buffer).ok();
        encoder.finish(out_buffer, true).ok();

        let len = out_buffer.pos();
        let data = out.into_iter().take(len).map(|v| *v).collect();

        (self.data.len(), data)
    }

    pub fn as_bytes(self) -> (usize, Vec<u8>) {
        match self.compression_kind {
            Some(comp) => match comp {
                CompressionKind::Zstd => self.compress_zstd(),
            },
            None => (self.data.len(), self.data),
        }
    }
}
