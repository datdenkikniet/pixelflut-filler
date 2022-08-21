use std::str::FromStr;

use crate::{codec::CodecData, color::Color};

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

#[derive(Debug, Clone)]
pub struct PixOffset {
    pub x_max: i32,
    pub x_offset: i32,
    pub y_max: i32,
    pub y_offset: i32,
    pub width: i32,
    pub height: i32,
}

impl PixOffset {
    pub fn do_offset(&self, x: i32, y: i32) -> (i32, i32) {
        let x_offset = if self.x_offset >= 0 {
            self.x_offset
        } else {
            self.x_max as i32 + self.x_offset - self.width as i32
        };

        let y_offset = if self.y_offset >= 0 {
            self.y_offset
        } else {
            self.y_max as i32 + self.y_offset - self.height as i32
        };

        (x + x_offset, y + y_offset)
    }
}

pub struct PixelCollector {
    kind: PixelCollectorKind,
    compression_kind: Option<CompressionKind>,
    pixels: Vec<(u16, u16, Color)>,
    max_x: i32,
    max_y: i32,
}

impl From<CodecData> for PixelCollector {
    fn from(codec: CodecData) -> Self {
        Self {
            kind: if codec.options.binary_px {
                PixelCollectorKind::Binary
            } else {
                PixelCollectorKind::Text
            },
            compression_kind: codec.options.compression_kind.clone(),
            pixels: Vec::new(),
            max_x: codec.window.get_x() as i32,
            max_y: codec.window.get_y() as i32,
        }
    }
}

impl PixelCollector {
    pub fn add_pixel_colored(&mut self, x: i32, y: i32, color: &Color) {
        let Color { a, .. } = color;

        if a.is_none() || a == &Some(0) {
            return;
        }

        if x >= 0 && x < self.max_x && y >= 0 && y < self.max_y {
            self.pixels.push((x as u16, y as u16, *color));
        }
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
                CompressionKind::Zstd => (data.len(), zstd::encode_all(&data[..], 1).unwrap()),
            },
            None => {
                log::debug!("Not compressing data");
                (data.len(), data)
            }
        }
    }
}
