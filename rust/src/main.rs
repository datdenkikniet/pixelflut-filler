use std::net::TcpStream;
use structopt::StructOpt;

mod color;
use color::Color;

mod window;
use window::Window;

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
    let noisy = opt.noisy;

    let color = if let Some(color) = opt.color {
        color
    } else {
        Color::random()
    };

    let stream = &mut TcpStream::connect(format!("{}:1337", remote))?;

    let mut window = Window::from_stream(stream)?;

    println!(
        "Detect screen with dimensions x: {}, y: {}",
        window.get_x(),
        window.get_y()
    );
    println!("Filling with color {:x}", color);

    window.fill(&color, noisy);

    Ok(())
}
