use std::net::TcpStream;
use structopt::StructOpt;

mod color;
use color::Color;

mod window;
use window::Window;

use crate::canvas::{Canvas, Pixel};

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
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let remote = opt.remote;

    let color = if let Some(color) = opt.color {
        color
    } else {
        Color::random()
    };

    let stream = &mut TcpStream::connect(format!("{}:1337", remote))?;

    let mut canvas: Canvas = Window::from_stream(stream)?.into();

    println!(
        "Detect screen with dimensions x: {}, y: {}",
        canvas.get_window().get_x(),
        canvas.get_window().get_y()
    );
    println!("Filling with color {:x}", color);

    canvas.fill(&Pixel::from_color(&color));

    if opt.noisy {
        canvas.send_data_noisy();
    } else {
        canvas.send_data();
    }

    Ok(())
}
