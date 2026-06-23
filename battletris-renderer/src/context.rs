use crate::Color;

pub trait DrawContext {
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color);
    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color);
}
