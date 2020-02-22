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

pub struct Coord {
    pub x: i16,
    pub y: i16
}

pub struct Quad {
    pub top_left: Coord,
    pub width: u16,
    pub height: u16,
    pub colour: Colour
}

impl Quad {
    pub fn render(&self) -> RenderData {
        let left = self.top_left.x as f32;
        let top = self.top_left.y as f32;
        let w = self.width as f32;
        let h = self.height as f32;
        let vertices = Vertex::from_xy(&[
            (left, top),
            (left, top + h),
            (left + w, top),
            (left + w, top + h)
        ]);
        let cols = vec![self.colour.clone(), self.colour.clone(), self.colour.clone(), self.colour.clone()];
        let indices = vec![
            0u16, 1, 2, 2, 3, 1
        ];

        (vertices, cols, indices)
    }
}