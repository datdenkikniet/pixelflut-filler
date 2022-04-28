use codec::{Codec, CodecData, DataProducer, RunError, SetupError};
use fill::Fill;
use pixelcollector::CompressionKind;
use snake::Snake;
use std::{net::TcpStream, path::PathBuf, time::Duration};
use structopt::StructOpt;

mod codec;
mod color;
mod fill;
mod gif;
mod image;
mod letters;
mod pixelcollector;
mod snake;
mod window;

use crate::{codec::CodecOptions, gif::Gif};
use color::Color;

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    SetupError(SetupError),
    RunError(RunError),
}

impl From<SetupError> for Error {
    fn from(e: SetupError) -> Self {
        Self::SetupError(e)
    }
}

impl From<RunError> for Error {
    fn from(e: RunError) -> Self {
        Self::RunError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(StructOpt)]
#[structopt(
    name = "pixelflut-filler",
    about = "Fill a pixelflut instance's window"
)]
struct Opt {
    /// Whether or not the data should be
    /// transmited "noisly" (i.e. with non-linear
    /// distribution of the coordinates of the sent
    /// pixels)
    #[structopt(short)]
    noisy: bool,

    /// The remote to connect to
    #[structopt(short, default_value = "127.0.0.1")]
    remote: String,

    /// Use the binary protocol
    #[structopt(short = "b", long)]
    use_binary_protocol: bool,

    /// What type of compression to use
    #[structopt(short, long)]
    compression: Option<CompressionKind>,

    /// The command to execute
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Fill the screen with a specific color
    Fill { color: Option<Color> },
    /// Write some text to the screen
    Write(WriteCommand),
    /// Send a gif, repeatedly
    Gif(GifCommand),
    /// Create a snake that wiggles along the screen
    Snake,
}

#[derive(StructOpt)]
struct WriteCommand {
    /// How many times to write the text at random coordinates
    /// and a scaling between 5 and 20 (only valid if 'scale',
    /// 'x' and 'y' are not set)
    #[structopt(short)]
    writes: Option<usize>,
    /// The scale at which to draw the text
    #[structopt(short)]
    scale: Option<usize>,
    /// The X coordinate to draw the text at
    #[structopt(short)]
    x: Option<usize>,
    /// The Y coordinate to draw the text at
    #[structopt(short)]
    y: Option<usize>,
    /// The color to use when drawing. Defaults
    /// to a random color per letter
    #[structopt(short)]
    color: Option<Color>,
    /// The text to write
    #[structopt(default_value = "change me")]
    text: String,
    /// Fill the screen with the specified
    /// color before writing the text. Use 'r'
    /// to use a random color.
    #[structopt(short, long)]
    fill_color: Option<Color>,
}

#[derive(StructOpt)]
struct GifCommand {
    /// The file name of the GIF to send
    file_name: PathBuf,
    /// The target frame time (in milliseconds)
    #[structopt(long, short, default_value = "150")]
    frame_time: u64,
    // Height offset
    #[structopt(long, short, default_value = "0")]
    height_offset: usize,
    // Width offset
    #[structopt(long, short, default_value = "0")]
    width_offset: usize,
}

enum DataProducers {
    Gif(Gif),
    Fill(Fill),
    Snake(Snake),
}

impl DataProducer for DataProducers {
    fn do_setup(&mut self, data: &CodecData) -> Result<(), String> {
        match self {
            DataProducers::Gif(gif) => gif.do_setup(data),
            DataProducers::Fill(fill) => fill.do_setup(data),
            DataProducers::Snake(snake) => snake.do_setup(data),
        }
    }

    fn get_next_data(&mut self) -> Result<(Vec<u8>, Duration), RunError> {
        match self {
            DataProducers::Gif(gif) => gif.get_next_data(),
            DataProducers::Fill(fill) => fill.get_next_data(),
            DataProducers::Snake(snake) => snake.get_next_data(),
        }
    }
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let remote = opt.remote;

    let stream = TcpStream::connect(format!("{}:1337", remote))?;

    let data_producer = match opt.command {
        Command::Gif(gif) => DataProducers::Gif(Gif::new(
            gif.file_name,
            Duration::from_micros(gif.frame_time),
            gif.width_offset,
            gif.height_offset,
        )),
        Command::Fill { color } => {
            DataProducers::Fill(Fill::new(color.unwrap_or(Color::random()), opt.noisy))
        }
        Command::Snake => DataProducers::Snake(Snake::new()),
        _ => todo!(),
    };

    let codec = Codec::new(
        stream,
        data_producer,
        CodecOptions {
            compression_kind: opt.compression,
            binary_px: opt.use_binary_protocol,
        },
    )?;

    println!(
        "Detect screen with dimensions x: {}, y: {}",
        codec.data().window.get_x(),
        codec.data().window.get_y()
    );

    let res = codec.run();
    std::thread::sleep(Duration::from_secs(1));

    res?;
    Ok(())
}
