use crate::Color;

pub trait DrawContext {
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color);
    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color);
    fn draw_gimp_tile(&mut self, px: f64, py: f64) {
        self.fill_rect(px, py, 28.0, 28.0, Color::rgb(120, 120, 120));
    }
}
