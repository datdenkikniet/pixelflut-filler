use std::str::FromStr;

use zstd::{
    stream::raw::{Encoder, Operation},
    zstd_safe::{InBuffer, OutBuffer},
};

use crate::color::Color;

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
    kind: PixelCollectorKind,
    compression_kind: Option<CompressionKind>,
    pixels: Vec<(u16, u16, Color)>,
}

impl PixelCollector {
    pub fn new_binary(compression_kind: Option<CompressionKind>) -> Self {
        Self {
            kind: PixelCollectorKind::Binary,
            compression_kind,
            pixels: Vec::new(),
        }
    }

    pub fn new_text(compression_kind: Option<CompressionKind>) -> Self {
        Self {
            kind: PixelCollectorKind::Text,
            compression_kind,
            pixels: Vec::new(),
        }
    }

    pub fn add_pixel_colored(&mut self, x: u16, y: u16, color: &Color) {
        let Color { a, .. } = color;

        if a.is_none() || a == &Some(0) {
            return;
        }

        self.pixels.push((x, y, *color));
    }

    pub fn add_pixel_raw(&mut self, x: u16, y: u16, color_data: (u8, u8, u8, Option<u8>)) {
        let (r, g, b, a) = color_data;
        self.add_pixel_colored(x, y, &Color::from_rgba(r, g, b, a))
    }

    fn compress_zstd(in_data: Vec<u8>) -> (usize, Vec<u8>) {
        let in_buffer = &mut InBuffer::around(&in_data);
        let out = &mut vec![0u8; in_data.len() + 1024];
        let out_buffer = &mut OutBuffer::around(out);
        let mut encoder = Encoder::new(1).unwrap();

        encoder.run(in_buffer, out_buffer).unwrap();
        encoder.finish(out_buffer, true).unwrap();

        let data = out_buffer.as_slice().iter().map(|v| *v).collect();

        (in_data.len(), data)
    }

    pub fn into_bytes(mut self) -> (usize, Vec<u8>) {
        self.pixels.sort_unstable_by(|c1, c2| c1.2.cmp(&c2.2));

        let mut data = Vec::with_capacity(self.pixels.len() * 4);

        for (x, y, color) in self.pixels.iter() {
            match self.kind {
                PixelCollectorKind::Binary => {
                    data.extend_from_slice(b"PB");

                    x.to_le_bytes().iter().for_each(|b| data.push(*b));
                    y.to_le_bytes().iter().for_each(|b| data.push(*b));

                    data.push(color.r);
                    data.push(color.g);
                    data.push(color.b);
                    data.push(color.a.unwrap_or(0xFF));
                }
                PixelCollectorKind::Text => {
                    data.extend_from_slice(
                        format!(
                            "PX {} {} {:02X}{:02X}{:02X}{:02X}\n",
                            x,
                            y,
                            color.r,
                            color.g,
                            color.b,
                            color.a.unwrap_or(0xFF)
                        )
                        .as_bytes(),
                    );
                }
            }
        }

        match self.compression_kind {
            Some(comp) => match comp {
                CompressionKind::Zstd => Self::compress_zstd(data),
            },
            None => (data.len(), data),
        }
    }
}
