use std::{
    io::{Read, Write},
    net::TcpStream,
};

use rand::{prelude::SliceRandom, thread_rng};

use crate::{color::Color, Error};

pub struct Window<'a> {
    x_width: usize,
    y_height: usize,
    tcp_stream: &'a mut TcpStream,
}

struct Position {
    x: usize,
    y: usize,
}

impl<'a> Window<'a> {
    pub fn get_x(&self) -> usize {
        self.x_width
    }

    pub fn get_y(&self) -> usize {
        self.y_height
    }

    pub fn from_stream(tcp_stream: &'a mut TcpStream) -> Result<Self, Error> {
        tcp_stream.write("SIZE\n".as_bytes())?;

        let result = &mut [0u8; 128];

        let len = tcp_stream.read(result)?;

        let mut size_result = if let Ok(result) = String::from_utf8((&result[..len]).to_vec()) {
            result
        } else {
            return Err(Error::SizeParseError(String::from("Could not read size.")));
        };

        size_result.pop();

        let parts: Vec<&str> = size_result.split(" ").collect();

        if parts.len() > 2 {
            let (x_width, y_height) = match (parts[1].parse(), parts[2].parse()) {
                (Ok(x), Ok(y)) => (x, y),
                _ => {
                    return Err(Error::SizeParseError(String::from(
                        "Could not parse width and height.",
                    )));
                }
            };

            let window_size = Window {
                x_width,
                y_height,
                tcp_stream,
            };
            Ok(window_size)
        } else {
            Err(Error::SizeParseError(String::from(
                "Did not receive enough parts.",
            )))
        }
    }

    pub fn fill(&mut self, color: &Color, shuffle: bool) {
        let mut positions = Vec::new();

        for x in 0..self.x_width {
            for y in 0..self.y_height {
                positions.push(Position { x, y })
            }
        }

        if shuffle {
            positions.shuffle(&mut thread_rng());
        }

        let mut all_bytes = Vec::new();

        let color_str = format!("{:x}", color);

        for position in positions {
            let command = format!("PX {} {} {}\n", position.x, position.y, color_str);
            for byte in command.as_bytes().iter() {
                all_bytes.push(*byte)
            }
        }

        self.tcp_stream.write(all_bytes.as_slice()).ok();
    }
}
