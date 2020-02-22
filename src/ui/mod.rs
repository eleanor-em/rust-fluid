use crate::graphics::{Colour, RenderData, RuntimeParams};
use crate::graphics::util::{Quad, RenderQueue, Coord};

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

    fn set_w_h(&mut self, w: u16, h: u16);
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
            border_colour: Colour::white()
        }
    }
}

pub struct Container {
    pub flex_direction: Direction,
    children: Vec<GuiObject>,
    pub style: Style,
    left: i16,
    top: i16,
    w: u16,
    h: u16,
    params: RuntimeParams
}

impl SetPosition for Container {
    fn set_top_left(&mut self, left: i16, top: i16) {
        self.left = left;
        self.top = top;
    }

    fn set_w_h(&mut self, w: u16, h: u16) {
        self.w = w;
        self.h = h;
    }
}

impl Container {
    pub fn new(params: &RuntimeParams) -> Self {
        Self {
            flex_direction: Direction::Row,
            children: vec![],
            style: Style::new(),
            left: 0,
            top: 0,
            w: params.window_width,
            h: params.window_height,
            params: params.clone(),
        }
    }

    pub fn push(&mut self, obj: GuiObject) {
        self.children.push(obj);
    }

    pub fn render(&self) -> RenderData {
        let mut queue = RenderQueue::new();
        let quad = Quad {
            top_left: Coord { x: self.left, y: self.top},
            width: self.w,
            height: self.h,
            colour: self.style.colour.clone()
        };

        queue.push(quad.render());

        queue.build()
    }
}

pub struct Component {
    pub style: Style,
    left: i16,
    top: i16,
    w: u16,
    h: u16,
}

impl SetPosition for Component {
    fn set_top_left(&mut self, left: i16, top: i16) {
        self.left = left;
        self.top = top;
    }

    fn set_w_h(&mut self, w: u16, h: u16) {
        self.w = w;
        self.h = h;
    }
}