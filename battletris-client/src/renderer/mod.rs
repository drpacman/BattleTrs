use sdl2::render::Canvas;
use sdl2::video::Window;

use battletris_renderer::{Color, DrawContext};

pub mod lobby;
pub mod title;

pub use battletris_renderer::layout::{WINDOW_H, WINDOW_W};

// ─── SDL2 DrawContext backend ─────────────────────────────────────────────────

pub struct SdlBackend<'a> {
    pub canvas: &'a mut Canvas<Window>,
}

impl<'a> SdlBackend<'a> {
    pub fn new(canvas: &'a mut Canvas<Window>) -> Self {
        Self { canvas }
    }
}

impl DrawContext for SdlBackend<'_> {
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color) {
        self.canvas.set_draw_color(sdl2::pixels::Color::RGBA(color.r, color.g, color.b, color.a));
        let _ = self.canvas.fill_rect(sdl2::rect::Rect::new(
            x as i32, y as i32,
            w.max(0.0) as u32, h.max(0.0) as u32,
        ));
    }

    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color) {
        self.canvas.set_draw_color(sdl2::pixels::Color::RGBA(color.r, color.g, color.b, color.a));
        let _ = self.canvas.draw_rect(sdl2::rect::Rect::new(
            x as i32, y as i32,
            w.max(0.0) as u32, h.max(0.0) as u32,
        ));
    }
}

// ─── Renderer (window + canvas lifecycle) ────────────────────────────────────

pub struct Renderer {
    pub canvas: Canvas<Window>,
}

impl Renderer {
    pub fn new(video: sdl2::VideoSubsystem) -> Result<Self, Box<dyn std::error::Error>> {
        let window = video
            .window("BattleTris", WINDOW_W as u32, WINDOW_H as u32)
            .position_centered()
            .build()?;
        let canvas = window.into_canvas().accelerated().present_vsync().build()?;
        Ok(Renderer { canvas })
    }

    pub fn backend(&mut self) -> SdlBackend<'_> {
        SdlBackend::new(&mut self.canvas)
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}
