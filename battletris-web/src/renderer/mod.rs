pub mod screens;

use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

use battletris_renderer::{Color, DrawContext};
use battletris_renderer::layout::{WINDOW_H, WINDOW_W};

// ─── Canvas DrawContext backend ───────────────────────────────────────────────

pub struct CanvasBackend<'a> {
    pub ctx: &'a CanvasRenderingContext2d,
}

impl<'a> CanvasBackend<'a> {
    pub fn new(ctx: &'a CanvasRenderingContext2d) -> Self {
        Self { ctx }
    }
}

impl DrawContext for CanvasBackend<'_> {
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color) {
        let css = color.to_css();
        self.ctx.set_fill_style_str(&css);
        self.ctx.fill_rect(x, y, w, h);
    }

    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color) {
        let css = color.to_css();
        self.ctx.set_stroke_style_str(&css);
        self.ctx.set_line_width(1.0);
        self.ctx.stroke_rect(x, y, w, h);
    }
}

// ─── Canvas renderer (canvas lifecycle + clear) ───────────────────────────────

pub struct CanvasRenderer {
    pub ctx: CanvasRenderingContext2d,
}

impl CanvasRenderer {
    pub fn new() -> Result<Self, wasm_bindgen::JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id("game-canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        Ok(Self { ctx })
    }

    pub fn backend(&self) -> CanvasBackend<'_> {
        CanvasBackend::new(&self.ctx)
    }

    pub fn clear(&self) {
        self.ctx.set_fill_style_str("#000000");
        self.ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H);
    }
}
