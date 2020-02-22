use gui::graphics;
use gui::graphics::{Backend, Vertex, Colour, RuntimeParams, Index, VertexProducer};
use rand::Rng;
use std::time::Instant;

fn main() {
    // TODO: Cleaner interface for vertices.
    graphics::new().unwrap()
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

        for _ in 1..20000 {
            quads.push(Quad::new(&mut rng))
        }
//        quads.push(Quad::at(&mut rng, 0., 0.));
//        quads.push(Quad::at(&mut rng, 1., 1.));

        Self {
            quads
        }
    }
}

impl VertexProducer for Producer {
    fn get_data(&mut self, params: RuntimeParams) -> (Vec<Vertex>, Vec<Colour>, Vec<u16>) {
        let mut verts = Vec::new();
        let mut cols = Vec::new();
        let mut indices = Vec::new();


        for (i, quad) in self.quads.iter().enumerate() {
            verts.append(&mut quad.verts(&params));
            cols.append(&mut quad.cols());
            indices.append(&mut quad.indices().into_iter().map(|x| x + (i * 4) as u16).collect());
        }

        (verts, cols, indices)
    }
}

#[derive(Debug)]
struct Quad {
    t0: Instant,
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
    fn new<R: Rng>(mut rng: &mut R) -> Self {
        let x = rng.gen_range(0.0, 1.0);
        let y = rng.gen_range(0.0, 1.0);
        Self::at(&mut rng, x, y)
    }

    fn at<R: Rng>(rng: &mut R, x: f32, y: f32) -> Self {
        Self {
            t0: Instant::now(),
            dt0: rng.gen_range(0.0, 1000.0),
            dt1: rng.gen_range(0.0, 1000.0),
            dt2: rng.gen_range(0.0, 1000.0),
            dt3: rng.gen_range(0.0, 1000.0),
            dt4: rng.gen_range(0.0, 1000.0),
            size: rng.gen_range(0.0, 5.0),
            x, y
        }
    }

    fn verts(&self, params: &RuntimeParams) -> Vec<Vertex> {
        let t = (Instant::now().duration_since(self.t0.clone()).as_millis() as f32) / 1000.0 * 6.28;

        let size = (((t + self.dt4).sin() + 1.0) / 2.0 + 1.0) * self.size;

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
        let t = (Instant::now().duration_since(self.t0.clone()).as_millis() as f32) / 1000.0 * 6.28;

        let c0 = ((t + self.dt0).sin() + 1.0) / 2.0;
        let c1 = ((t + self.dt1).sin() + 1.0) / 2.0;
        let c2 = ((t + self.dt2).sin() + 1.0) / 2.0;
        let c30 = ((t + self.dt3 + self.dt0).sin() + 1.0) / 2.0;
        let c31 = ((t + self.dt3 + self.dt1).sin() + 1.0) / 2.0;
        let c32 = ((t + self.dt3 + self.dt2).sin() + 1.0) / 2.0;

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