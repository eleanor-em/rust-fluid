use gui::graphics;

use rand::Rng;
use gui::graphics::{Backend, VertexProducer, RuntimeParams, RenderData, Vertex, Colour, Index};
use gui::graphics::util::RenderStack;

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
        let mut quads = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 1..50000 {
            quads.push(Quad::new(&mut rng))
        }

        Self {
            quads
        }
    }
}

impl VertexProducer for Producer {
    fn get_data(&mut self, params: RuntimeParams) -> RenderData {
        let mut stack = RenderStack::new();
        for quad in self.quads.iter_mut() {
            stack.push(quad.step(&params));
        }

        stack.build()
    }
}

#[derive(Debug)]
struct Quad {
    t: f32,
    dt0: f32,
    dt1: f32,
    dt2: f32,
    dt3: f32,
    dt4: f32,
    size: f32,
    x: f32,
    y: f32
}

impl Quad {
    const PERIOD: f32 = 40.;

    fn new<R: Rng>(mut rng: &mut R) -> Self {
        let x = rng.gen_range(0.0, 1.0);
        let y = rng.gen_range(0.0, 1.0);
        Self::at(&mut rng, x, y)
    }

    fn at<R: Rng>(rng: &mut R, x: f32, y: f32) -> Self {
        Self {
            t: 0.,
            dt0: rng.gen_range(0.0, Self::PERIOD),
            dt1: rng.gen_range(0.0, Self::PERIOD),
            dt2: rng.gen_range(0.0, Self::PERIOD),
            dt3: rng.gen_range(0.0, Self::PERIOD),
            dt4: rng.gen_range(0.0, Self::PERIOD),
            size: rng.gen_range(0.0, 5.0),
            x, y
        }
    }

    fn step(&mut self, params: &RuntimeParams) -> RenderData {
        self.t += 1.;
        (self.verts(params), self.cols(), self.indices())
    }

    fn verts(&self, params: &RuntimeParams) -> Vec<Vertex> {
        let size = (((self.t * 6.28 / Self::PERIOD + self.dt4).sin() + 1.0) / 2.0 + 1.0) * self.size;

        let left = params.window_width as f32 * self.x - size;
        let right = params.window_width as f32 * self.x + size;
        let top = params.window_height as f32 * self.y - size;
        let bottom = params.window_height as f32 * self.y + size;

        Vertex::from_xyz(&[
            (left, top, 0.0),
            (left, bottom, 0.0),
            (right, top, 0.0),
            (right, bottom, 0.0)
        ])
    }

    fn cols(&self) -> Vec<Colour> {
        let c0 = ((self.t * 6.28 / Self::PERIOD + self.dt0).sin() + 1.0) / 2.0;
        let c1 = ((self.t * 6.28 / Self::PERIOD + self.dt1).sin() + 1.0) / 2.0;
        let c2 = ((self.t * 6.28 / Self::PERIOD + self.dt2).sin() + 1.0) / 2.0;
        let c30 = ((self.t * 6.28 / Self::PERIOD + self.dt3 + self.dt0).sin() + 1.0) / 2.0;
        let c31 = ((self.t * 6.28 / Self::PERIOD + self.dt3 + self.dt1).sin() + 1.0) / 2.0;
        let c32 = ((self.t * 6.28 / Self::PERIOD + self.dt3 + self.dt2).sin() + 1.0) / 2.0;

        Colour::from_rgba(&[
            (c0, 0.0, 0.0, 1.0),
            (0.0, c1, 0.0, 1.0),
            (0.0, 0.0, c2, 1.0),
            (c30, c31, c32, 1.0)
        ])
    }

    fn indices(&self) -> Vec<Index> {
        vec![
            0u16, 1, 2, 2, 3, 1
        ]
    }
}