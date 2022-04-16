use std::{
    io::{Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{canvas::Canvas, pixelcollector::PixelCollector};

pub struct Gif<T>
where
    T: Read + Write,
{
    canvas: Canvas<T>,
    frame_time: Duration,
    frames: Vec<Vec<u8>>,
}

impl<T> Gif<T>
where
    T: Read + Write,
{
    pub fn new_with(
        canvas: Canvas<T>,
        path: PathBuf,
        frame_time: Duration,
        width_offset: usize,
        height_offset: usize,
    ) -> Self {
        let mut frames = Vec::new();

        let file = std::fs::File::open(path).unwrap();
        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::RGBA);

        let mut decoder = decoder.read_info(file).unwrap();
        println!("Reading all frames");

        let mut frame_number = 0;
        let mut threads = Vec::new();
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            frames.push(Vec::new());
            let frame = frame.clone();

            threads.push(std::thread::spawn(move || {
                let mut pixel_collector = PixelCollector::new_binary();

                let width = frame.width as usize;
                let height = frame.height as usize;

                for y in 0..height {
                    for x in 0..width {
                        let start_pixel = ((y * width) + x) * 4;
                        let pixel_data = &frame.buffer[start_pixel..start_pixel + 4];
                        let x = (x + frame.left as usize + width_offset) as u16;
                        let y = (y + frame.top as usize + height_offset) as u16;

                        if pixel_data[3] != 0 {
                            pixel_collector.add_pixel(x, y, (0, 0, 0, 0x00));
                        }
                    }
                }
                println!("Read frame {}", frame_number);
                (frame_number, pixel_collector.as_bytes())
            }));
            frame_number += 1;
        }

        for thread in threads {
            let (frame_num, frame) = thread.join().unwrap();

            let overwrite = frames.get_mut(frame_num).unwrap();
            *overwrite = frame;
        }

        println!("Done reading");
        Self {
            frames,
            frame_time,
            canvas,
        }
    }

    pub fn send_continuous(&mut self) {
        loop {
            let frames = self.frames.iter();
            let stream = &mut self.canvas.window.get_stream();
            for frame in frames {
                let start_time = Instant::now();
                stream.write(frame).unwrap();
                let end_time = Instant::now();
                let frame_duration = end_time - start_time;
                if frame_duration < self.frame_time {
                    std::thread::sleep(self.frame_time - frame_duration);
                }
            }
        }
    }
}
