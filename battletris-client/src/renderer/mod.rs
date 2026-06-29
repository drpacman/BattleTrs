use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use battletris_renderer::{Color, DrawContext, GIMP_PNG};

pub use battletris_renderer::layout::{WINDOW_H, WINDOW_W};

// ─── SDL2 DrawContext backend ─────────────────────────────────────────────────

pub struct SdlBackend<'a> {
    pub canvas: &'a mut Canvas<Window>,
    gimp_texture: Option<&'a Texture<'static>>,
}

impl<'a> SdlBackend<'a> {
    pub fn new(canvas: &'a mut Canvas<Window>, gimp_texture: Option<&'a Texture<'static>>) -> Self {
        Self { canvas, gimp_texture }
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

    fn draw_gimp_tile(&mut self, px: f64, py: f64) {
        if let Some(tex) = self.gimp_texture {
            let dst = sdl2::rect::Rect::new(px as i32, py as i32, 28, 28);
            let _ = self.canvas.copy(tex, None, dst);
        } else {
            self.canvas.set_draw_color(sdl2::pixels::Color::RGB(120, 120, 120));
            let _ = self.canvas.fill_rect(sdl2::rect::Rect::new(px as i32, py as i32, 28, 28));
        }
    }
}

// ─── Renderer (window + canvas lifecycle) ────────────────────────────────────

// Field order determines drop order: gimp_texture before texture_creator before canvas.
pub struct Renderer {
    gimp_texture: Option<Texture<'static>>,
    #[allow(dead_code)]
    texture_creator: TextureCreator<WindowContext>,
    pub canvas: Canvas<Window>,
}

impl Renderer {
    pub fn new(video: sdl2::VideoSubsystem) -> Result<Self, Box<dyn std::error::Error>> {
        let window = video
            .window("BattleTris", WINDOW_W as u32, WINDOW_H as u32)
            .position_centered()
            .build()?;
        let canvas = window.into_canvas().accelerated().present_vsync().build()?;
        let texture_creator = canvas.texture_creator();
        let gimp_texture = load_gimp_texture(&texture_creator)
            // SAFETY: texture_creator is owned by the same struct as gimp_texture.
            // Fields are dropped in declaration order (gimp_texture before texture_creator),
            // so the texture is always dropped before its creator.
            .map(|t| unsafe { std::mem::transmute::<Texture<'_>, Texture<'static>>(t) });
        Ok(Self { gimp_texture, texture_creator, canvas })
    }

    pub fn backend(&mut self) -> SdlBackend<'_> {
        SdlBackend::new(&mut self.canvas, self.gimp_texture.as_ref())
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}

// ─── PNG → SDL2 texture ───────────────────────────────────────────────────────

fn load_gimp_texture(texture_creator: &TextureCreator<WindowContext>) -> Option<Texture<'_>> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(GIMP_PNG));
    decoder.set_transformations(png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width, info.height);

    let mut rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => buf[..info.buffer_size()]
            .chunks_exact(3)
            .flat_map(|c| [c[0], c[1], c[2], 255u8])
            .collect(),
        _ => return None,
    };

    // On little-endian (x86/ARM), ABGR8888 means memory layout [R, G, B, A] — matches RGBA output.
    let surface = Surface::from_data(&mut rgba, w, h, w * 4, PixelFormatEnum::ABGR8888).ok()?;
    texture_creator.create_texture_from_surface(&surface).ok()
}
