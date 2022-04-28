use std::time::Duration;

use rand::{prelude::SliceRandom, thread_rng};

use crate::{
    codec::{CodecData, DataProducer, RunError},
    color::Color,
    pixelcollector::PixelCollector,
};

pub struct Fill {
    color: Color,
    data: Vec<u8>,
    sent: bool,
    noisy: bool,
}

impl Fill {
    pub fn new(color: Color, noisy: bool) -> Self {
        Self {
            color,
            data: Vec::new(),
            sent: false,
            noisy,
        }
    }
}

impl DataProducer for Fill {
    fn do_setup(&mut self, codec: &CodecData) -> Result<(), String> {
        self.data.clear();
        let x = codec.window.get_x() as u16;
        let y = codec.window.get_y() as u16;

        let mut position_list = Vec::with_capacity(x as usize * y as usize);

        for x in 0..x {
            for y in 0..y {
                position_list.push((x, y))
            }
        }

        if self.noisy {
            position_list.shuffle(&mut thread_rng());
        }

        let mut pixel_collector: PixelCollector = codec.clone().into();
        for (x, y) in position_list {
            pixel_collector.add_pixel_colored(x as u16, y as u16, &self.color);
        }

        self.data = pixel_collector.into_bytes().1;

        Ok(())
    }

    fn get_next_data(&mut self) -> Result<(Vec<u8>, Option<Duration>), RunError> {
        Ok((self.data.clone(), None))
    }
}
