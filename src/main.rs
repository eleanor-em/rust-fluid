use rust_fluid::graphics;

use rand::Rng;
use rust_fluid::graphics::{Backend, VertexProducer, RuntimeParams, RenderData, Colour};
use rust_fluid::graphics::util::{RenderStack, Quad, Coord};

fn main() {
    graphics::init().unwrap()
        .show_fps()
        .run(Box::new(Producer::new())).unwrap();
}

struct Producer {
    quads: Vec<Quad>
}

impl Producer {
    fn new() -> Self {
        let quads = Vec::new();

        Self {
            quads
        }
    }
}

impl VertexProducer for Producer {
    fn get_data(&mut self, params: RuntimeParams) -> RenderData {
        if self.quads.len() == 0 {
            let mut rng = rand::thread_rng();

            for _ in 1..10000 {
                let x = rng.gen_range(0, params.window_width) as i16;
                let y = rng.gen_range(0, params.window_height) as i16;
                let w = rng.gen_range(1, 64);
                let h = rng.gen_range(1, 64);
                let col = Colour::Rgba(rng.gen_range(0., 1.),
                                       rng.gen_range(0., 1.),
                                       rng.gen_range(0., 1.),
                                       rng.gen_range(0., 1.));
                self.quads.push(Quad {
                    top_left: Coord { x, y },
                    width: w,
                    height: h,
                    colour: col
                })
            }
        }
        let mut stack = RenderStack::new();
        for quad in self.quads.iter_mut() {
            stack.push(quad.render());
        }

        stack.build()
    }
}