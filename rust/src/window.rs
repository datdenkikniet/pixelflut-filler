use std::io::{Read, Write};

use crate::Error;

pub struct Window<T>
where
    T: Read + Write,
{
    x_width: usize,
    y_height: usize,
    stream: T,
}

impl<T> Window<T>
where
    T: Read + Write,
{
    pub fn get_x(&self) -> usize {
        self.x_width
    }

    pub fn get_y(&self) -> usize {
        self.y_height
    }

    pub fn get_stream(&mut self) -> &mut T {
        &mut self.stream
    }

    pub fn from_stream(mut stream: T) -> Result<Self, Error> {
        stream.write("SIZE\n".as_bytes())?;
        let result = &mut [0u8; 128];

        let len = stream.read(result)?;

        println!("{}", len);

        let mut size_result = if let Ok(result) = String::from_utf8((&result[..len]).to_vec()) {
            result
        } else {
            return Err(Error::SizeParseError(String::from("Could not read size.")));
        };

        size_result.pop();

        let size_result = if size_result.ends_with('\r') {
            &size_result[..size_result.len() - 1]
        } else {
            &size_result
        };

        let parts: Vec<&str> = size_result.split(" ").collect();

        if parts.len() > 2 {
            let (x_width, y_height) = match (parts[1].trim().parse(), parts[2].trim().parse()) {
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
                stream,
            };
            Ok(window_size)
        } else {
            Err(Error::SizeParseError(String::from(
                "Did not receive enough parts.",
            )))
        }
    }
}
