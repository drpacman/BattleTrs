pub use battletris_renderer::screens::{draw_connecting, draw_name_taken, draw_waiting};

use battletris_renderer::font::{draw_text, text_w};
use battletris_renderer::layout::{WINDOW_H, WINDOW_W};
use battletris_renderer::widgets::draw_input_field;
use battletris_renderer::{Color, DrawContext};

const PANEL_W: f64 = 500.0;

pub fn draw_connection_screen<D: DrawContext>(
    ctx: &mut D,
    name_buf: &str,
    cursor_visible: bool,
    error: Option<&str>,
) {
    let cx = WINDOW_W / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(10, 10, 30));

    let title = "NETWORK GAME";
    draw_text(ctx, title, cx - text_w(title, 4.0) / 2.0, 140.0, Color::rgb(255, 220, 0), 4.0);

    let panel_h = 140.0;
    let px = cx - PANEL_W / 2.0;
    let py = 270.0;
    ctx.fill_rect(px, py, PANEL_W, panel_h, Color::rgb(25, 25, 50));
    ctx.stroke_rect(px, py, PANEL_W, panel_h, Color::rgb(80, 80, 160));

    draw_text(ctx, "YOUR NAME:", px + 16.0, py + 20.0, Color::rgb(160, 160, 160), 2.0);
    draw_input_field(ctx, px + 16.0, py + 42.0, PANEL_W - 32.0, name_buf, true, cursor_visible);

    let hint = "ENTER - connect";
    draw_text(ctx, hint, cx - text_w(hint, 1.0) / 2.0, py + panel_h + 14.0,
        Color::rgb(100, 100, 100), 1.0);

    if let Some(err) = error {
        draw_text(ctx, err, cx - text_w(err, 2.0) / 2.0, py + panel_h + 34.0,
            Color::rgb(220, 60, 60), 2.0);
    }
}
