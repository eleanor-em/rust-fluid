use rust_fluid::graphics;

use rust_fluid::graphics::{Backend, VertexProducer, RuntimeParams, RenderData, Colour};
use rust_fluid::ui::Container;

fn main() {
    graphics::init().unwrap()
        .show_fps()
        .run(Box::new(Producer::new())).unwrap();
}

struct Producer {
    container: Option<Container>
}

impl Producer {
    fn new() -> Self {
        Self {
            container: None
        }
    }

    fn create_ui(params: &RuntimeParams) -> Container {
        let mut frame = Container::new(&params);
        frame.style.colour = Colour::rgb8(20, 20, 25);

        frame
    }
}

impl VertexProducer for Producer {
    fn get_data(&mut self, params: RuntimeParams) -> RenderData {
        let frame = self.container.get_or_insert_with(|| Self::create_ui(&params));

        frame.render()
    }
}