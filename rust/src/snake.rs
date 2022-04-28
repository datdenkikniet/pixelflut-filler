use std::time::Duration;

use crate::{
    codec::{CodecData, DataProducer, RunError},
    color::Color,
    pixelcollector::PixelCollector,
};

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

pub struct Snake {
    x: isize,
    y: isize,
    direction: Velocity,
    color: Color,
    codec_data: Option<CodecData>,
}

impl Snake {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            codec_data: None,
            direction: Velocity::new(5.),
            color: Color::random(),
        }
    }

    fn change_direction(&mut self) {
        self.direction.change();
        self.color = Color::random();
    }

    fn advance(&mut self) {
        let (x, y) = self.direction.get_delta_p();

        self.x += x as isize;
        self.y += y as isize;

        let width = self.codec_data.as_ref().unwrap().window.get_y() as isize;
        let height = self.codec_data.as_ref().unwrap().window.get_x() as isize;

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
}

impl DataProducer for Snake {
    fn do_setup(&mut self, codec: &crate::codec::CodecData) -> Result<(), String> {
        self.codec_data = Some(codec.clone());
        Ok(())
    }

    fn get_next_data(&mut self) -> Result<(Vec<u8>, Option<Duration>), RunError> {
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

        let mut pixel_collector: PixelCollector = self.codec_data.clone().unwrap().into();

        for x in x_range {
            for y in y_range.clone() {
                pixel_collector.add_pixel_colored(x as u16, y as u16, &self.color);
            }
        }

        let (_bytes, data) = pixel_collector.into_bytes();

        Ok((data, Some(Duration::from_millis(1))))
    }
}
