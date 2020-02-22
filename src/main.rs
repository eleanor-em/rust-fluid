use gui::graphics::*;

fn main() {
    // TODO: Cleaner interface for vertices.
    run_show_fps(|| {
        let vertices = vec![
            Vertex3 { position: (-0.5, -0.5, 0.0) },
            Vertex3 { position: (-0.5, 0.5, 0.0) },
            Vertex3 { position: (0.5, -0.5, 0.0) },
            Vertex3 { position: (0.5, 0.5, 0.0) }
        ];

        let colours = vec![
            Colour4 { colour: (1.0, 0.0, 0.0, 1.0) },
            Colour4 { colour: (0.0, 1.0, 0.0, 1.0) },
            Colour4 { colour: (0.0, 0.0, 1.0, 1.0) },
            Colour4 { colour: (1.0, 1.0, 1.0, 1.0) }
        ];

        let indices = vec![
            0u16, 1, 2, 2, 3, 1
        ];

        (vertices, colours, indices)
    });
}