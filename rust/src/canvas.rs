use std::io::{Read, Write};

use rand::{prelude::SliceRandom, thread_rng};

use crate::{
    color::Color,
    letters::{Letter, LETTER_HEIGHT, LETTER_WIDTH},
    pixelcollector::{CompressionKind, PixelCollector},
    window::Window,
};

struct Position {
    x: usize,
    y: usize,
}

pub struct Canvas<T>
where
    T: Read + Write,
{
    pub window: Window<T>,
    pixels: Vec<Color>,
}

impl<T> From<Window<T>> for Canvas<T>
where
    T: Read + Write,
{
    fn from(window: Window<T>) -> Self {
        Self::new(window)
    }
}

#[allow(unused)]
impl<T> Canvas<T>
where
    T: Read + Write,
{
    pub fn new(window: Window<T>) -> Self {
        let mut vec = Vec::with_capacity(window.get_x() * window.get_y());
        for _ in 0..(window.get_x() * window.get_y()) {
            vec.push(Color::default());
        }

        Self {
            window,
            pixels: vec,
        }
    }

    fn get_pixel_mut(&mut self, x: usize, y: usize) -> &mut Color {
        let index = self.calc_index(x, y);
        &mut self.pixels[index]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: &Color) {
        self.get_pixel_mut(x, y).copy_from(value);
    }

    #[inline]
    fn calc_index(&self, x: usize, y: usize) -> usize {
        ((x % self.window.get_x()) * self.window.get_y()) + (y % self.window.get_y())
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> &Color {
        &self.pixels[self.calc_index(x, y)]
    }

    pub fn fill(&mut self, color: &Color) {
        self.pixels
            .iter_mut()
            .for_each(|pixel| pixel.copy_from(color));
    }

    pub fn send_data(&mut self, binary: bool) {
        let mut pixels = if binary {
            PixelCollector::new_binary(None)
        } else {
            PixelCollector::new_text(None)
        };
        for x in 0..self.window.get_x() - 1 {
            for y in 0..self.window.get_y() - 1 {
                let pixel = self.get_pixel(x, y);
                pixels.add_pixel_raw(x as u16, y as u16, (pixel.r, pixel.g, pixel.b, pixel.a));
            }
        }
        self.window.get_stream().write(&pixels.into_bytes().1).ok();
    }

    pub fn send_data_noisy(&mut self, use_binary_protocol: bool) {
        let mut position_list = Vec::with_capacity(self.window.get_x() * self.get_window().get_y());

        for x in 0..self.window.get_x() {
            for y in 0..self.window.get_y() {
                position_list.push(Position { x, y })
            }
        }

        position_list.shuffle(&mut thread_rng());

        let mut pixel_collector = if use_binary_protocol {
            self.window.get_stream().write_all(b"COMPRESS\n").unwrap();
            PixelCollector::new_binary(Some(CompressionKind::Zstd))
        } else {
            PixelCollector::new_text(None)
        };

        for position in position_list {
            let x = position.x;
            let y = position.y;
            let pixel = self.get_pixel(x, y);
            pixel_collector.add_pixel_colored(x as u16, y as u16, pixel);
        }

        let (actual, bytes) = pixel_collector.into_bytes();

        println!("Ratio: {:.02}", actual as f64 / (bytes.len() as f64));

        self.window.get_stream().write_all(&bytes).unwrap();

        self.window.get_stream().flush();

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    pub fn get_window(&self) -> &Window<T> {
        &self.window
    }

    pub fn draw_letter(
        &mut self,
        letter: &Letter,
        x: usize,
        y: usize,
        color: &Color,
        scale: usize,
    ) {
        for letter_x in 0..LETTER_WIDTH {
            for letter_y in 0..LETTER_HEIGHT {
                for scale_x in 0..scale {
                    for scale_y in 0..scale {
                        let pixel = self.get_pixel_mut(
                            x + (scale * letter_x) + scale_x,
                            y + (scale * letter_y) + scale_y,
                        );
                        if letter[letter_x + (letter_y * LETTER_WIDTH)] == 1 {
                            pixel.copy_from(&color);
                        }
                    }
                }
            }
        }
    }
}
