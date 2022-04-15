use std::{
    io::Write,
    net::TcpStream,
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct Gif {
    stream: TcpStream,
    frame_time: Duration,
    frames: Vec<String>,
}

impl Gif {
    pub fn new_with(
        stream: TcpStream,
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
            frames.push(String::new());
            let frame = frame.clone();

            threads.push(std::thread::spawn(move || {
                let width = frame.width as usize;
                let height = frame.height as usize;

                let mut string = String::new();
                for y in 0..height {
                    for x in 0..width {
                        let start_pixel = ((y * width) + x) * 4;
                        let pixel_data = &frame.buffer[start_pixel..start_pixel + 4];
                        string.push_str(&format!(
                            "PX {} {} {:02X}{:02X}{:02X}{:02X}\n",
                            x + frame.left as usize + width_offset,
                            y + frame.top as usize + height_offset,
                            pixel_data[0],
                            pixel_data[1],
                            pixel_data[2],
                            pixel_data[3]
                        ))
                    }
                }
                println!("Read frame {}", frame_number);
                (frame_number, string)
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
            stream,
        }
    }

    pub fn send_continuous(&mut self) {
        loop {
            let frames = self.frames.iter();
            let stream = &mut self.stream;
            for frame in frames {
                let start_time = Instant::now();
                stream.write(frame.as_bytes()).ok();
                let end_time = Instant::now();
                let frame_duration = end_time - start_time;
                if frame_duration < self.frame_time {
                    std::thread::sleep(self.frame_time - frame_duration);
                }
            }
        }
    }
}
