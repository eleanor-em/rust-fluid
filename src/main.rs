use rust_fluid::graphics;

use rust_fluid::graphics::{GfxProvider, Colour};
use rust_fluid::ui::{Border, Frame};

fn main() {
    let frame = Frame::new()
        .colour(Colour::rgb8(20, 20, 25))
        .margin(Border::new(4, 4, 4, 4))
        .border_width(4)
        .border_colour(Colour::white());

    graphics::init().unwrap()
        .show_fps()
        .run(Box::new(frame)).unwrap();
}