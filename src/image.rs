use std::{path::PathBuf, time::Duration};

use image::{io::Reader, Pixel};

use crate::{
    codec::DataProducer,
    pixelcollector::{PixOffset, PixelCollector},
};

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

        let pix_offset = PixOffset {
            x_max: codec.window.get_x() as i32,
            x_offset: self.width_offset,
            y_max: codec.window.get_y() as i32,
            y_offset: self.height_offset,
            width: data.width() as i32,
            height: data.height() as i32,
        };

        let mut pixelcollector: PixelCollector = codec.clone().into();
        for (x, y, data) in data.enumerate_pixels() {
            let (x, y) = pix_offset.do_offset(x as i32, y as i32);

            let data = data.channels();
            let color = crate::color::Color::from_rgba(data[0], data[1], data[2], Some(data[3]));
            pixelcollector.add_pixel_colored(x, y, &color);
        }

        self.data = pixelcollector.into_bytes().1;
        Ok(())
    }

    fn get_next_data(&mut self) -> Result<(Vec<u8>, Option<Duration>), crate::codec::RunError> {
        Ok((self.data.clone(), self.frame_interval))
    }
}
