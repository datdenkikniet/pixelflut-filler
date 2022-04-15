use rand::{thread_rng, RngCore};
use std::{net::TcpStream, path::PathBuf, time::Duration};
use structopt::StructOpt;

mod color;
use color::Color;

mod window;
use window::Window;

mod letters;

use crate::{canvas::Canvas, letters::*};

mod canvas;
mod gif;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    SizeParseError(String),
    OptionError(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
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

    /// The command to execute
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Fill the screen with a specific color
    Fill { color: Option<Color> },
    /// Write some text to the screen
    Write(Write),
    /// Send a gif, repeatedly
    Gif(Gif),
}

#[derive(StructOpt)]
struct Write {
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
struct Gif {
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

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let remote = opt.remote;

    let mut stream = TcpStream::connect(format!("{}:1337", remote))?;

    let canvas: &mut Canvas = &mut Window::from_stream(&mut stream)?.into();

    println!(
        "Detect screen with dimensions x: {}, y: {}",
        canvas.get_window().get_x(),
        canvas.get_window().get_y()
    );

    match opt.command {
        Command::Fill { color } => fill_canvas(canvas, opt.noisy, color),
        Command::Write(write) => write_text(canvas, opt.noisy, write)?,
        Command::Gif(gif) => send_gif_loop(stream, gif),
    }

    Ok(())
}

fn fill_canvas(canvas: &mut Canvas, noisy: bool, color: Option<Color>) {
    let fill_color = if let Some(color) = color {
        color
    } else {
        Color::random()
    };

    println!("Filling with color {:x}", fill_color);

    canvas.fill(&fill_color);

    if noisy {
        canvas.send_data_noisy();
    } else {
        canvas.send_data();
    }
}

fn write_text(canvas: &mut Canvas, noisy: bool, write: Write) -> Result<(), Error> {
    let iterations = match (&write.x, &write.y, &write.scale, &write.writes) {
        (Some(..), Some(..), Some(..), None) => 1,
        (None, None, None, Some(writes)) => *writes,
        (None, None, None, None) => 1,
        (..) => {
            return Err(Error::OptionError(format!("Valid combinations are: x, y, scale, and not writes, or writes and not x, y, z, and scale")));
        }
    };

    let mut offset = 0;

    let letters: LetterString = write.text.as_str().into();

    if let Some(fc) = &write.fill_color {
        canvas.fill(fc);
    }

    for _ in 0..iterations {
        let (x, y, scale) =
            if let (Some(x), Some(y), Some(scale)) = (&write.x, &write.y, &write.scale) {
                (*x, *y, *scale)
            } else {
                let random_scale = ((thread_rng().next_u32() as usize) % 15) + 5;
                let random_x = (thread_rng().next_u64() as usize) % canvas.get_window().get_x();
                let random_y = (thread_rng().next_u64() as usize) % canvas.get_window().get_y();
                (random_x, random_y, random_scale)
            };

        letters.iter().for_each(|letter| {
            let mut random_color = Color::random();
            random_color.a = Some(0x7F);
            let color = match &write.color {
                Some(color) => color,
                None => &random_color,
            };
            canvas.draw_letter(&letter, x + offset, y, color, scale);
            offset += (1 + LETTER_WIDTH) * scale;
        });
    }

    if noisy {
        canvas.send_data_noisy();
    } else {
        canvas.send_data();
    }
    Ok(())
}

fn send_gif_loop(stream: TcpStream, gif: Gif) {
    let frame_time = Duration::from_millis(gif.frame_time);

    let mut gif = gif::Gif::new_with(
        stream,
        gif.file_name,
        frame_time,
        gif.width_offset,
        gif.height_offset,
    );
    gif.send_continuous();
}
