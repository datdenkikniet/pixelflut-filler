use std::{
    io::{Read, Write},
    time::{Duration, Instant},
};

use crate::{pixelcollector::CompressionKind, window::Window};

#[derive(Debug)]
pub enum SetupError {
    IoError(std::io::Error),
    SizeParseError(String),
    EnableCompressionError,
    DataProducerError(String),
}

impl From<std::io::Error> for SetupError {
    fn from(err: std::io::Error) -> Self {
        SetupError::IoError(err)
    }
}

#[derive(Debug)]
pub enum RunError {
    ClientDataFinished,
    Io(std::io::Error),
}

impl From<std::io::Error> for RunError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Clone)]
pub struct CodecOptions {
    pub compression_kind: Option<CompressionKind>,
    pub binary_px: bool,
}

#[derive(Clone)]
pub struct CodecData {
    pub window: Window,
    pub options: CodecOptions,
}

pub trait DataProducer {
    fn do_setup(&mut self, codec: &CodecData) -> Result<(), String>;
    fn get_next_data(&mut self) -> Result<(Vec<u8>, Option<Duration>), RunError>;
}

pub struct Codec<T, D>
where
    T: Read + Write,
    D: DataProducer,
{
    socket: T,
    data_producer: D,
    data: CodecData,
}

impl<T, D> Codec<T, D>
where
    T: Read + Write,
    D: DataProducer,
{
    pub fn data(&self) -> &CodecData {
        &self.data
    }

    pub fn new(
        mut socket: T,
        mut data_producer: D,
        options: CodecOptions,
    ) -> Result<Self, SetupError> {
        socket.write("SIZE\n".as_bytes())?;
        let result = &mut [0u8; 128];

        std::thread::sleep(std::time::Duration::from_millis(500));

        let len = socket.read(result)?;

        let mut size_result = if let Ok(result) = String::from_utf8((&result[..len]).to_vec()) {
            result
        } else {
            return Err(SetupError::SizeParseError(String::from(
                "Could not read size.",
            )));
        };

        size_result.pop();

        let size_result = if size_result.ends_with('\r') {
            &size_result[..size_result.len() - 1]
        } else {
            &size_result
        };

        let parts: Vec<&str> = size_result.split(" ").collect();

        let window = if parts.len() > 2 {
            let (x_width, y_height) = match (parts[1].trim().parse(), parts[2].trim().parse()) {
                (Ok(x), Ok(y)) => (x, y),
                _ => {
                    return Err(SetupError::SizeParseError(String::from(
                        "Could not parse width and height.",
                    )));
                }
            };

            let window = Window { x_width, y_height };
            window
        } else {
            return Err(SetupError::SizeParseError(String::from(
                "Did not receive enough parts.",
            )));
        };

        if let Some(compression) = options.compression_kind {
            socket.write_all(&compression.compression_string())?;

            let mut compress_resp = [0u8; 12];

            socket.read(&mut compress_resp)?;

            if compress_resp.starts_with(b"COMPRESS\r\n")
                && compress_resp.starts_with(b"COMPRESS\n")
            {
                return Err(SetupError::EnableCompressionError);
            }
        }

        let data = CodecData { window, options };

        data_producer
            .do_setup(&data)
            .map_err(|e| SetupError::DataProducerError(e))?;

        Ok(Self {
            socket,
            data,
            data_producer,
        })
    }

    pub fn run(mut self) -> Result<(), RunError> {
        loop {
            let (data, next_data) = self.data_producer.get_next_data()?;
            let start = Instant::now();

            self.socket.write_all(&data)?;

            let send_duration = Instant::now().duration_since(start);

            if let Some(next_data) = next_data {
                if next_data > send_duration {
                    let sleep_duration = next_data - send_duration;
                    std::thread::sleep(sleep_duration);
                }
            } else {
                break Ok(());
            }
        }
    }
}
