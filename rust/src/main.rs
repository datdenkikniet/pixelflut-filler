use rand::{thread_rng, RngCore};
use std::{net::TcpStream, time::Instant};
use structopt::StructOpt;

mod color;
use color::Color;

mod window;
use window::Window;

mod letters;

use crate::{canvas::Canvas, letters::*};

mod canvas;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    SizeParseError(String),
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
    /// Whether or not the fill should be noisy
    #[structopt(short)]
    noisy: bool,

    /// The color to fill
    #[structopt(short)]
    color: Option<Color>,

    /// The remote to connect to
    #[structopt(short, default_value = "127.0.0.1")]
    remote: String,

    /// How many times to draw "change me"
    #[structopt(short, default_value = "1")]
    writes: u32,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let remote = opt.remote;

    let mut fill_color = if let Some(color) = opt.color {
        color
    } else {
        Color::random()
    };
    fill_color.a = Some(0x0F);

    let stream = &mut TcpStream::connect(format!("{}:1337", remote))?;

    let mut canvas: Canvas = Window::from_stream(stream)?.into();

    println!(
        "Detect screen with dimensions x: {}, y: {}",
        canvas.get_window().get_x(),
        canvas.get_window().get_y()
    );
    println!("Filling with color {:x}", fill_color);

    canvas.fill(&fill_color);

    let mut offset = 0;

    let letters: LetterString = "change me".into();

    let rendering = Instant::now();

    for _ in 0..opt.writes {
        let random_scale = ((thread_rng().next_u32() as usize) % 20) + 2;
        let random_x = (thread_rng().next_u64() as usize) % canvas.get_window().get_x();
        let random_y = (thread_rng().next_u64() as usize) % canvas.get_window().get_y();
        letters.iter().for_each(|letter| {
            let mut color = Color::random();
            color.a = Some(0x7F);
            canvas.draw_letter(&letter, random_x + offset, random_y, &color, random_scale);
            offset += (1 + LETTER_WIDTH) * random_scale;
        });
    }

    let rendered = Instant::now();

    println!("{}", (rendered - rendering).as_millis());

    if opt.noisy {
        canvas.send_data_noisy();
    } else {
        canvas.send_data();
    }

    Ok(())
}
