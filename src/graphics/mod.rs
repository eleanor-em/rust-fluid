use std::error::Error;
use std::marker::Sized;
use crate::graphics::backends::vk::VulkanBackend;

pub type Index = u16;

pub enum Vertex {
    Xy(f32, f32),
    Xyz(f32, f32, f32)
}

pub enum Colour {
    Rgb(f32, f32, f32),
    Rgba(f32, f32, f32, f32)
}

impl Vertex {
    pub fn from_xy(vertices: &[(f32, f32)]) -> Vec<Self> {
        vertices.into_iter().map(|(x, y)| Self::Xy(*x, *y)).collect()
    }
    pub fn from_xyz(vertices: &[(f32, f32, f32)]) -> Vec<Self> {
        vertices.into_iter().map(|(x, y, z)| Self::Xyz(*x, *y, *z)).collect()
    }
}

impl Colour {
    pub fn from_rgb(vertices: &[(f32, f32, f32)]) -> Vec<Self> {
        vertices.into_iter().map(|(r, g, b)| Self::Rgb(*r, *g, *b)).collect()
    }
    pub fn from_rgba(vertices: &[(f32, f32, f32, f32)]) -> Vec<Self> {
        vertices.into_iter().map(|(r, g, b, a)| Self::Rgba(*r, *g, *b, *a)).collect()
    }
    pub fn red() -> Self {
        Colour::Rgba(1.0, 0.0, 0.0, 1.0)
    }
    pub fn green() -> Self {
        Colour::Rgba(0.0, 1.0, 0.0, 1.0)
    }
    pub fn blue() -> Self {
        Colour::Rgba(0.0, 0.0, 1.0, 1.0)
    }
    pub fn black() -> Self {
        Colour::Rgba(0.0, 0.0, 0.0, 1.0)
    }
    pub fn white() -> Self {
        Colour::Rgba(1.0, 1.0, 1.0, 1.0)
    }
}

pub struct RuntimeParams {
    pub window_width: u32,
    pub window_height: u32,
}

pub type RenderData = (Vec<Vertex>, Vec<Colour>, Vec<Index>);

pub trait VertexProducer {
    fn get_data(&mut self, params: RuntimeParams) -> RenderData;
}

pub trait Backend {
    fn new() -> Result<Self, Box<dyn Error>> where Self: Sized;
    fn show_fps(self) -> Self;
    fn run(self, update_values: Box<dyn VertexProducer>) -> Result<(), Box<dyn Error>>;
}

pub fn init() -> Result<VulkanBackend, Box<dyn Error>> {
    VulkanBackend::new()
}

pub mod backends;
pub mod util;