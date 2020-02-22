use crate::graphics::{Colour, RenderData, RuntimeParams};
use crate::graphics::util::{Quad, RenderStack, Coord};

pub enum Direction {
    Row,
    Column,
}

pub enum GuiObject {
    Container(Container),
    Component(Component),
}

pub struct Border {
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
    pub left: i16
}

impl Border {
    pub fn zero() -> Self {
        Self::new(0, 0, 0, 0)
    }
    pub fn new(top: i16, right: i16, bottom: i16, left: i16) -> Self {
        Self { top, right, bottom, left }
    }
}

trait SetPosition {
    fn set_top_left(&mut self, left: i16, top: i16);

    fn set_w_h(&mut self, w: f32, h: f32);
}

pub struct Style {
    pub flex: u8,
    pub colour: Colour,
    pub padding: Border,
    pub margin: Border,
    pub border_width: u16,
    pub border_colour: Colour,
}

impl Style {
    pub fn new() -> Self {
        Self {
            flex: 1,
            colour: Colour::white(),
            padding: Border::zero(),
            margin: Border::zero(),
            border_width: 0,
            border_colour: Colour::black()
        }
    }
}

pub struct Container {
    pub flex_direction: Direction,
    children: Vec<GuiObject>,
    pub style: Style,
    left: i16,
    top: i16,
    w: f32,
    h: f32,
}

impl SetPosition for Container {
    fn set_top_left(&mut self, left: i16, top: i16) {
        self.left = left;
        self.top = top;
    }

    fn set_w_h(&mut self, w: f32, h: f32) {
        self.w = w;
        self.h = h;
    }
}

impl Container {
    pub fn new() -> Self {
        Self {
            flex_direction: Direction::Row,
            children: vec![],
            style: Style::new(),
            left: 0,
            top: 0,
            w: 1.,
            h: 1.,
        }
    }

    pub fn push(&mut self, obj: GuiObject) {
        self.children.push(obj);
    }

    pub fn render(&self, params: &RuntimeParams) -> RenderData {
        let mut stack = RenderStack::new();
        let border_quad = Quad {
            top_left: Coord {
                x: self.left + self.style.margin.left,
                y: self.top + self.style.margin.top
            },
            width: ((self.w * params.window_width as f32) as i16
                - self.style.margin.right
                - self.style.margin.left) as u16,
            height: ((self.h * params.window_height as f32) as i16
                - self.style.margin.bottom
                - self.style.margin.top) as u16,
            colour: self.style.border_colour.clone()
        };

        let mut content_quad = border_quad.clone();
        content_quad.colour = self.style.colour.clone();
        content_quad.top_left.x += self.style.border_width as i16;
        content_quad.top_left.y += self.style.border_width as i16;
        content_quad.width -= self.style.border_width * 2;
        content_quad.height -= self.style.border_width * 2;

        stack.push(content_quad.render());
        stack.push(border_quad.render());

        stack.build()
    }
}

pub struct Component {
    pub style: Style,
    left: i16,
    top: i16,
    w: f32,
    h: f32,
}

impl SetPosition for Component {
    fn set_top_left(&mut self, left: i16, top: i16) {
        self.left = left;
        self.top = top;
    }

    fn set_w_h(&mut self, w: f32, h: f32) {
        self.w = w;
        self.h = h;
    }
}