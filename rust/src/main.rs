use std::{
    fmt::LowerHex,
    io::{Read, Write},
    net::TcpStream,
};

use rand::thread_rng;
use rand::{seq::SliceRandom, RngCore};

struct Position {
    x: usize,
    y: usize,
}

struct WindowSize {
    x_width: usize,
    y_height: usize,
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: Option<u8>,
}

impl LowerHex for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:02x}", self.r))?;
        f.write_str(&format!("{:02x}", self.g))?;
        f.write_str(&format!("{:02x}", self.b))?;
        if let Some(a) = self.a {
            f.write_str(&format!("{:02x}", a))?;
        }
        Ok(())
    }
}

impl Color {
    fn random() -> Self {
        let random = thread_rng().next_u32();
        Color {
            r: (random & 0xFF) as u8,
            g: ((random & 0xFF00) >> 8) as u8,
            b: ((random & 0xFF0000) >> 16) as u8,
            a: Some(((random & 0xFF000000) >> 24) as u8),
        }
    }
}

#[derive(Debug)]
enum Error {
    IoError(std::io::Error),
    SizeParseError(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

fn main() -> Result<(), Error> {
    let stream = &mut TcpStream::connect("localhost:1337")?;

    let size = parse_size(stream)?;
    let color = Color::random();

    println!(
        "Detect screen with dimensions x: {}, y: {}",
        size.x_width, size.y_height
    );
    println!("Filling with color {:x}", color);

    fill_window(stream, &color, &size, true);

    Ok(())
}

fn fill_window(stream: &mut TcpStream, color: &Color, window_size: &WindowSize, shuffle: bool) {
    let mut positions = Vec::new();

    for x in 0..window_size.x_width {
        for y in 0..window_size.y_height {
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

    stream.write(all_bytes.as_slice()).ok();
}

fn parse_size(stream: &mut TcpStream) -> Result<WindowSize, Error> {
    stream.write("SIZE\n".as_bytes())?;

    let result = &mut [0u8; 128];

    let len = stream.read(result)?;

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

        let window_size = WindowSize { x_width, y_height };
        Ok(window_size)
    } else {
        Err(Error::SizeParseError(String::from(
            "Did not receive enough parts.",
        )))
    }
}
