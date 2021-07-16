use std::io::Write;

use rand::{prelude::SliceRandom, thread_rng};

use crate::{
    color::Color,
    letters::{Letter, LETTER_HEIGHT, LETTER_WIDTH},
    window::Window,
};

struct Position {
    x: usize,
    y: usize,
}

pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

impl Pixel {
    pub fn from_color(color: &Color) -> Self {
        let a = match color.a {
            Some(value) => value,
            None => 0xFF,
        };
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a,
        }
    }

    fn copy(&mut self, value: &Pixel) {
        self.r = value.r;
        self.g = value.g;
        self.b = value.b;
        self.a = value.a;
    }
}

pub struct Canvas<'a> {
    window: Window<'a>,
    pixels: Vec<Pixel>,
}

impl<'a> From<Window<'a>> for Canvas<'a> {
    fn from(window: Window<'a>) -> Self {
        Self::new(window)
    }
}

impl<'a> Canvas<'a> {
    pub fn new(window: Window<'a>) -> Self {
        let mut vec = Vec::with_capacity(window.get_x() * window.get_y());
        for _ in 0..(window.get_x() * window.get_y()) {
            vec.push(Pixel::default());
        }

        Self {
            window,
            pixels: vec,
        }
    }

    fn get_pixel_mut(&mut self, x: usize, y: usize) -> &mut Pixel {
        let index = self.calc_index(x, y);
        &mut self.pixels[index]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: &Pixel) {
        self.get_pixel_mut(x, y).copy(value);
    }

    #[inline]
    fn calc_index(&self, x: usize, y: usize) -> usize {
        ((x % self.window.get_x()) * self.window.get_y()) + (y % self.window.get_y())
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> &Pixel {
        &self.pixels[self.calc_index(x, y)]
    }

    pub fn fill(&mut self, color: &Color) {
        self.pixels
            .iter_mut()
            .for_each(|pixel| pixel.copy(&Pixel::from_color(color)));
    }

    pub fn send_data(&mut self) {
        let mut all_bytes = Vec::new();
        for x in 0..self.window.get_x() - 1 {
            for y in 0..self.window.get_y() - 1 {
                let pixel = self.get_pixel(x, y);
                let command = format!(
                    "PX {} {} {:02X}{:02X}{:02X}{:02X}\n",
                    x, y, pixel.r, pixel.g, pixel.b, pixel.a
                );
                for byte in command.as_bytes().iter() {
                    all_bytes.push(*byte)
                }
            }
        }
        self.window.get_tcp_stream().write(&all_bytes).ok();
    }

    pub fn send_data_noisy(&mut self) {
        let mut all_bytes = Vec::new();

        let mut position_list = Vec::with_capacity(self.window.get_x() * self.get_window().get_y());

        for x in 0..self.window.get_x() {
            for y in 0..self.window.get_y() {
                position_list.push(Position { x, y })
            }
        }

        position_list.shuffle(&mut thread_rng());

        for position in position_list {
            let x = position.x;
            let y = position.y;
            let pixel = self.get_pixel(x, y);
            let command = format!(
                "PX {} {} {:02X}{:02X}{:02X}{:02X}\n",
                x, y, pixel.r, pixel.g, pixel.b, pixel.a
            );
            for byte in command.as_bytes().iter() {
                all_bytes.push(*byte)
            }
        }

        self.window.get_tcp_stream().write(&all_bytes).ok();
    }

    pub fn get_window(&self) -> &Window {
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
                            pixel.copy(&Pixel::from_color(color));
                        }
                    }
                }
            }
        }
    }
}
