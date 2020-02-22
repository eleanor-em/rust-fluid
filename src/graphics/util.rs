use crate::graphics::*;

pub struct RenderStack {
    verts: Vec<Vertex>,
    cols: Vec<Colour>,
    indices: Vec<Index>,
    base: usize
}

impl RenderStack {
    pub fn new() -> Self {
        Self {
            verts: Vec::new(),
            cols: Vec::new(),
            indices: Vec::new(),
            base: 0
        }
    }

    pub fn push(&mut self, mut data: RenderData) {
        assert_eq!(data.0.len(), data.1.len());
        let num_vertices = data.0.len();
        self.verts.append(&mut data.0);
        self.cols.append(&mut data.1);
        self.indices.append(&mut data.2.into_iter().map(|x| x + self.base as u16).collect());
        self.base += num_vertices;
    }

    pub fn build(self) -> RenderData {
        (self.verts, self.cols, self.indices)
    }
}