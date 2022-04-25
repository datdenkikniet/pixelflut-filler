use std::io::{Read, Write};

use crate::{canvas::Canvas, color::Color, window::Window};

#[derive(Debug)]
pub struct Velocity {
    direction: f32,
    speed: f32,
}

impl Velocity {
    fn new(speed: f32) -> Self {
        Self {
            direction: 1.0,
            speed,
        }
    }

    fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    fn get_delta_p(&self) -> (isize, isize) {
        let x = (self.direction.cos() * self.speed) as isize;
        let y = (self.direction.sin() * self.speed) as isize;

        (x, y)
    }

    fn change(&mut self) {
        self.direction = rand::random::<f32>() * std::f32::consts::PI * 2.;

        loop {
            let (x, y) = self.get_delta_p();
            if x == 0 && y == 0 {
                self.direction = rand::random::<f32>() * std::f32::consts::PI * 2.;
            } else {
                break;
            }
        }
    }
}

pub struct Snake<T>
where
    T: Read + Write,
{
    canvas: Canvas<T>,
    x: isize,
    y: isize,
    direction: Velocity,
    color: Color,
}

impl<T> Snake<T>
where
    T: Read + Write,
{
    pub fn new(canvas: Canvas<T>) -> Self {
        Self {
            canvas,
            x: 0,
            y: 0,
            direction: Velocity::new(2.5),
            color: Color::random(),
        }
    }

    fn window(&mut self) -> &mut Window<T> {
        &mut self.canvas.window
    }

    fn change_direction(&mut self) {
        self.direction.change();
        self.color = Color::random();
    }

    fn advance(&mut self) {
        let (x, y) = self.direction.get_delta_p();

        self.x += x as isize;
        self.y += y as isize;

        let width = self.window().get_x() as isize;
        let height = self.window().get_y() as isize;

        if self.x >= width {
            self.x = width - 1;
            self.change_direction();
        } else if self.x < 0 {
            self.x = 1;
            self.change_direction();
        }

        if self.y >= height {
            self.y = height - 1;
            self.change_direction();
        } else if self.y < 0 {
            self.y = 0;
            self.change_direction();
        }
    }

    pub fn run(mut self) {
        loop {
            let tail = (self.x, self.y);
            self.advance();

            let x_range = if self.x < tail.0 {
                self.x..tail.0
            } else {
                tail.0..self.x
            };

            let y_range = if self.y < tail.1 {
                self.y..tail.1
            } else {
                tail.1..self.y
            };

            let mut data = Vec::new();

            for x in x_range {
                for y in y_range.clone() {
                    data.extend_from_slice(b"PB");

                    (x as u16).to_le_bytes().iter().for_each(|b| data.push(*b));
                    (y as u16).to_le_bytes().iter().for_each(|b| data.push(*b));

                    data.push(self.color.r);
                    data.push(self.color.g);
                    data.push(self.color.b);
                    data.push(self.color.a.unwrap_or(0xFF));
                }
            }
            self.window().get_stream().write_all(&data).unwrap();
            // std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
