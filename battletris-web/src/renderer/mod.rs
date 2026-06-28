pub mod screens;

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use battletris_renderer::{Color, DrawContext, GIMP_PNG};
use battletris_renderer::layout::{WINDOW_H, WINDOW_W};

// ─── Canvas DrawContext backend ───────────────────────────────────────────────

pub struct CanvasBackend<'a> {
    pub ctx: &'a CanvasRenderingContext2d,
    gimp_canvas: Option<&'a HtmlCanvasElement>,
}

impl<'a> CanvasBackend<'a> {
    pub fn new(ctx: &'a CanvasRenderingContext2d, gimp_canvas: Option<&'a HtmlCanvasElement>) -> Self {
        Self { ctx, gimp_canvas }
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

    fn draw_gimp_tile(&mut self, px: f64, py: f64) {
        if let Some(src) = self.gimp_canvas {
            let _ = self.ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                src, px, py, 28.0, 28.0,
            );
        } else {
            self.fill_rect(px, py, 28.0, 28.0, Color::rgb(120, 120, 120));
        }
    }
}

// ─── Canvas renderer (canvas lifecycle + clear) ───────────────────────────────

pub struct CanvasRenderer {
    pub ctx: CanvasRenderingContext2d,
    // Off-screen canvas holding the decoded Gimp tile.
    // drawImage from it uses source-over compositing, unlike put_image_data which
    // replaces pixels wholesale (including setting alpha=0 over already-drawn content).
    gimp_canvas: Option<HtmlCanvasElement>,
}

impl CanvasRenderer {
    pub fn new() -> Result<Self, wasm_bindgen::JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id("game-canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;
        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        let gimp_canvas = load_gimp_canvas(&document);
        Ok(Self { ctx, gimp_canvas })
    }

    pub fn backend(&self) -> CanvasBackend<'_> {
        CanvasBackend::new(&self.ctx, self.gimp_canvas.as_ref())
    }

    pub fn clear(&self) {
        self.ctx.set_fill_style_str("#000000");
        self.ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H);
    }
}

// ─── PNG → offscreen HtmlCanvasElement ───────────────────────────────────────

fn load_gimp_canvas(document: &web_sys::Document) -> Option<HtmlCanvasElement> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(GIMP_PNG));
    decoder.set_transformations(png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width, info.height);

    let rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => buf[..info.buffer_size()]
            .chunks_exact(3)
            .flat_map(|c| [c[0], c[1], c[2], 255u8])
            .collect(),
        _ => return None,
    };

    let image_data = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(rgba.as_slice()), w, h,
    ).ok()?;

    let offscreen = document
        .create_element("canvas").ok()?
        .dyn_into::<HtmlCanvasElement>().ok()?;
    offscreen.set_width(w);
    offscreen.set_height(h);
    let off_ctx = offscreen
        .get_context("2d").ok()??
        .dyn_into::<CanvasRenderingContext2d>().ok()?;
    off_ctx.put_image_data(&image_data, 0.0, 0.0).ok()?;

    Some(offscreen)
}
