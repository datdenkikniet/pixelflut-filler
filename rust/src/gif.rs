use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    codec::{CodecData, DataProducer},
    color::Color,
    pixelcollector::{PixOffset, PixelCollector},
};

pub struct Gif {
    frame_time: Duration,
    height_offset: i32,
    width_offset: i32,
    path: PathBuf,
    frames: Vec<Vec<u8>>,
    frame_num: usize,
}

impl Gif {
    pub fn new(path: PathBuf, frame_time: Duration, width_offset: i32, height_offset: i32) -> Self {
        Self {
            frame_time,
            height_offset,
            width_offset,
            path,
            frames: Vec::new(),
            frame_num: 0,
        }
    }
}

impl DataProducer for Gif {
    fn do_setup(&mut self, data: &CodecData) -> Result<(), String> {
        let file = std::fs::File::open(self.path.as_path()).unwrap();
        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::RGBA);

        let mut decoder = decoder.read_info(file).unwrap();
        log::info!("Reading all frames");

        let start_time = Instant::now();

        let mut uncompressed_bytes = 0;
        let mut out_bytes = 0;
        let mut frame_num = 0;

        let mut width = None;
        let mut height = None;

        while let Some(frame) = decoder.read_next_frame().unwrap() {
            let x_offset = self.width_offset;
            let y_offset = self.height_offset;

            let (x_max, y_max) = (data.window.get_x() as i32, data.window.get_y() as i32);
            let mut pixel_collector: PixelCollector = data.clone().into();

            // Use the width and height of the first frame for offset calculations
            let (width, height) = if let (Some(width), Some(height)) = (width, height) {
                (width, height)
            } else {
                width = Some(frame.width as i32);
                height = Some(frame.height as i32);
                (frame.width as i32, frame.height as i32)
            };

            let pix_offset = PixOffset {
                x_max,
                x_offset,
                y_max,
                y_offset,
                width,
                height,
            };

            let (width, height) = (frame.width as usize, frame.height as usize);
            for y in 0..height {
                for x in 0..width {
                    let start_pixel = ((y * width) + x) * 4;
                    let pd = &frame.buffer[start_pixel..start_pixel + 4];
                    let (x, y) = pix_offset.do_offset(x as i32, y as i32);

                    pixel_collector.add_pixel_colored(
                        x,
                        y,
                        &Color::from_rgba(pd[0], pd[1], pd[2], Some(pd[3])),
                    );
                }
            }

            let (actual_size, frame) = pixel_collector.into_bytes();
            let len = frame.len();
            out_bytes += frame.len();
            uncompressed_bytes += actual_size;
            self.frames.push(frame);
            log::debug!(
                "Finished frame {}. Ratio: {:.02}",
                frame_num,
                (actual_size as f64) / (len as f64)
            );
            frame_num += 1;
        }

        log::info!(
            "Bytes read: {}, Bytes out: {}, ratio: {:.02}",
            uncompressed_bytes,
            out_bytes,
            (uncompressed_bytes as f64) / (out_bytes as f64)
        );

        log::info!(
            "Loaded {} frames in  {} ms",
            frame_num,
            Instant::now().duration_since(start_time).as_millis()
        );

        Ok(())
    }

    fn get_next_data(&mut self) -> Result<(Vec<u8>, Option<Duration>), crate::codec::RunError> {
        self.frame_num = (self.frame_num + 1) % self.frames.len();
        let frame = &self.frames[self.frame_num];
        Ok((frame.clone(), Some(self.frame_time)))
    }
}
