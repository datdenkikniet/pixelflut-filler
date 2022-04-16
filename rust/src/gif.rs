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
        use_binary_protocol: bool,
    ) -> Self {
        let mut frames = Vec::new();

        let file = std::fs::File::open(path).unwrap();
        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::RGBA);

        let mut decoder = decoder.read_info(file).unwrap();
        println!("Reading all frames");

        let mut frame_number = 0;
        let (tx, rx) = std::sync::mpsc::channel();

        let start_time = Instant::now();

        let mut active_threads = 0;
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            frames.push(Vec::new());
            let frame = frame.clone();

            active_threads += 1;

            let tx = tx.clone();
            std::thread::spawn(move || {
                let mut pixel_collector = if use_binary_protocol {
                    PixelCollector::new_binary()
                } else {
                    PixelCollector::new_text()
                };

                let width = frame.width as usize;
                let height = frame.height as usize;

                for y in 0..height {
                    for x in 0..width {
                        let start_pixel = ((y * width) + x) * 4;
                        let pd = &frame.buffer[start_pixel..start_pixel + 4];
                        let x = (x + frame.left as usize + width_offset) as u16;
                        let y = (y + frame.top as usize + height_offset) as u16;

                        if pd[3] != 0 {
                            pixel_collector.add_pixel_raw(x, y, (pd[0], pd[1], pd[2], pd[3]));
                        }
                    }
                }
                println!("Read frame {}", frame_number);

                tx.send((frame_number, pixel_collector.as_bytes()))
            });
            frame_number += 1;

            while active_threads >= 16 {
                let (frame_num, frame) = rx.recv().unwrap();
                let overwrite = frames.get_mut(frame_num).unwrap();
                *overwrite = frame;
                active_threads -= 1;
            }
        }

        while active_threads > 0 {
            let (frame_num, frame) = rx.recv().unwrap();
            let overwrite = frames.get_mut(frame_num).unwrap();
            *overwrite = frame;
            active_threads -= 1;
        }

        println!(
            "Loaded {} frames in  {} ms",
            frame_number,
            Instant::now().duration_since(start_time).as_millis()
        );

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
