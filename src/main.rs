use rust_fluid::graphics;

use rust_fluid::graphics::{Backend, VertexProducer, RuntimeParams, RenderData, Colour};
use rust_fluid::ui::{Container, Border};

fn main() {
    graphics::init().unwrap()
        .show_fps()
        .run(Box::new(Producer::new())).unwrap();
}

struct Producer {
    frame: Container
}

impl Producer {
    fn new() -> Self {
        let mut frame = Container::new();
        frame.style.colour = Colour::rgb8(20, 20, 25);
        frame.style.margin = Border::new(4, 4, 4, 4);
        frame.style.border_width = 4;
        frame.style.border_colour = Colour::white();

        Self {
            frame
        }
    }
}

impl VertexProducer for Producer {
    fn get_data(&mut self, params: RuntimeParams) -> RenderData {
        self.frame.render(&params)
    }
}