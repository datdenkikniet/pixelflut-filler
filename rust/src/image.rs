use std::{path::PathBuf, time::Duration};

use image::{io::Reader, Pixel};

use crate::{codec::DataProducer, pixelcollector::PixelCollector};

pub struct Image {
    data: Vec<u8>,
    path: PathBuf,
    width_offset: i32,
    height_offset: i32,
    frame_interval: Option<Duration>,
}

impl Image {
    pub fn new(
        path: PathBuf,
        frame_interval: Option<Duration>,
        width_offset: i32,
        height_offset: i32,
    ) -> Self {
        Self {
            data: Vec::new(),
            path,
            frame_interval,
            width_offset,
            height_offset,
        }
    }
}

impl DataProducer for Image {
    fn do_setup(&mut self, codec: &crate::codec::CodecData) -> Result<(), String> {
        let image_data = Reader::open(self.path.as_path()).map_err(|e| format!("{:?}", e))?;
        let data = image_data.decode().unwrap().to_rgba8();

        let x_offset = if self.width_offset >= 0 {
            self.width_offset
        } else {
            codec.window.get_x() as i32 + self.width_offset - data.width() as i32
        };

        let y_offset = if self.height_offset >= 0 {
            self.height_offset
        } else {
            codec.window.get_y() as i32 + self.height_offset - data.height() as i32
        };

        let mut pixelcollector: PixelCollector = codec.clone().into();
        for (x, y, data) in data.enumerate_pixels() {
            let x = x as i32 + x_offset;
            let y = y as i32 + y_offset;

            if x >= 0 && y >= 0 {
                let (x, y) = (x as u16, y as u16);
                let data = data.channels();
                let color =
                    crate::color::Color::from_rgba(data[0], data[1], data[2], Some(data[3]));
                pixelcollector.add_pixel_colored(x, y, &color);
            }
        }

        self.data = pixelcollector.into_bytes().1;
        Ok(())
    }

    fn get_next_data(&mut self) -> Result<(Vec<u8>, Option<Duration>), crate::codec::RunError> {
        Ok((self.data.clone(), self.frame_interval))
    }
}
