use gui::graphics;
use gui::graphics::{Backend, Vertex, Colour, RuntimeParams, RenderParams};

fn main() {
    // TODO: Cleaner interface for vertices.
    graphics::new().unwrap()
        .show_fps()
        .run(&producer).unwrap();
}

fn producer(params: RuntimeParams) -> RenderParams {
    let size = 100;
    let left = (params.window_width / 2 - size) as f32;
    let right = (params.window_width / 2 + size) as f32;
    let top = (params.window_height / 2 - size) as f32;
    let bottom = (params.window_height / 2 + size) as f32;
    let vertices = Vertex::from_v3(&[
        (left, top, 0.0),
        (left, bottom, 0.0),
        (right, top, 0.0),
        (right, bottom, 0.0)
    ]);

    let colours = Colour::from_rgba(&[
        (1.0, 0.0, 0.0, 1.0),
        (0.0, 1.0, 0.0, 1.0),
        (0.0, 0.0, 1.0, 1.0),
        (1.0, 1.0, 1.0, 1.0)
    ]);

    let indices = vec![
        0u16, 1, 2, 2, 3, 1
    ];

    (vertices, colours, indices)
}