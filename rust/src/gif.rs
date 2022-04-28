use std::{
    path::PathBuf,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use crate::{
    codec::{CodecData, DataProducer},
    pixelcollector::PixelCollector,
};

pub struct Gif {
    frame_time: Duration,
    height_offset: usize,
    width_offset: usize,
    path: PathBuf,
    frames: Vec<Vec<u8>>,
    frame_num: usize,
}

impl Gif {
    pub fn new(
        path: PathBuf,
        frame_time: Duration,
        width_offset: usize,
        height_offset: usize,
    ) -> Self {
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
        let width_offset = self.width_offset;
        let height_offset = self.height_offset;

        let file = std::fs::File::open(self.path.as_path()).unwrap();
        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::RGBA);

        let mut decoder = decoder.read_info(file).unwrap();
        println!("Reading all frames");

        let mut frame_number = 0;
        let (tx, rx): (_, Receiver<(usize, (usize, Vec<u8>))>) = std::sync::mpsc::channel();

        let start_time = Instant::now();

        let mut uncompressed_bytes = 0;
        let mut out_bytes = 0;
        let mut active_threads = 0;

        macro_rules! take_frame {
            ($max: literal) => {
                while active_threads > $max {
                    let (frame_num, (actual_size, frame)) = rx.recv().unwrap();
                    let overwrite = self.frames.get_mut(frame_num).unwrap();
                    uncompressed_bytes += actual_size;
                    let len = frame.len();
                    out_bytes += frame.len();
                    *overwrite = frame;
                    active_threads -= 1;
                    println!(
                        "Finished frame {}. Ratio: {:.02}",
                        frame_num,
                        (actual_size as f64) / (len as f64)
                    );
                }
            };
        }

        while let Some(frame) = decoder.read_next_frame().unwrap() {
            self.frames.push(Vec::new());
            let frame = frame.clone();

            active_threads += 1;

            let tx = tx.clone();
            let data = data.clone();
            std::thread::spawn(move || {
                let mut pixel_collector: PixelCollector = data.into();

                let width = frame.width as usize;
                let height = frame.height as usize;

                for y in 0..height {
                    for x in 0..width {
                        let start_pixel = ((y * width) + x) * 4;
                        let pd = &frame.buffer[start_pixel..start_pixel + 4];
                        let x = (x + frame.left as usize + width_offset) as u16;
                        let y = (y + frame.top as usize + height_offset) as u16;

                        pixel_collector.add_pixel_raw(x, y, (pd[0], pd[1], pd[2], Some(pd[3])));
                    }
                }

                tx.send((frame_number, pixel_collector.into_bytes()))
            });
            frame_number += 1;

            take_frame!(15);
        }

        take_frame!(0);

        println!(
            "Bytes read: {}, Bytes out: {}, ratio: {:.02}",
            uncompressed_bytes,
            out_bytes,
            (uncompressed_bytes as f64) / (out_bytes as f64)
        );

        println!(
            "Loaded {} frames in  {} ms",
            frame_number,
            Instant::now().duration_since(start_time).as_millis()
        );

        println!("Done reading");
        Ok(())
    }

    fn get_next_data(&mut self) -> Result<(Vec<u8>, Duration), crate::codec::RunError> {
        self.frame_num = (self.frame_num + 1) % self.frames.len();
        let frame = &self.frames[self.frame_num];
        Ok((frame.clone(), self.frame_time))
    }
}
