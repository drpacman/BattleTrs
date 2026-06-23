use battletris_renderer::{Color, DrawContext};
use battletris_renderer::font::{draw_text, text_w};
use battletris_renderer::layout::{WINDOW_H, WINDOW_W};

use super::{Renderer, SdlBackend};

const PANEL_W: f64 = 500.0;
const PANEL_H: f64 = 260.0;

pub fn draw_connection_screen(
    r: &mut Renderer,
    addr_buf: &str,
    name_buf: &str,
    active_field: usize,
    cursor_visible: bool,
    error: Option<&str>,
) {
    let mut ctx = r.backend();
    let cx = WINDOW_W / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(10, 10, 30));

    let title = "NETWORK GAME";
    draw_text(&mut ctx, title, cx - text_w(title, 4.0) / 2.0, 140.0, Color::rgb(255, 220, 0), 4.0);

    let px = cx - PANEL_W / 2.0;
    let py = 230.0;
    ctx.fill_rect(px, py, PANEL_W, PANEL_H, Color::rgb(25, 25, 50));
    ctx.stroke_rect(px, py, PANEL_W, PANEL_H, Color::rgb(80, 80, 160));

    let addr_label = "SERVER ADDRESS:";
    draw_text(&mut ctx, addr_label, px + 16.0, py + 20.0, Color::rgb(160, 160, 160), 2.0);
    draw_input_field(&mut ctx, px + 16.0, py + 42.0, PANEL_W - 32.0, addr_buf, active_field == 0, cursor_visible);

    let name_label = "YOUR NAME:";
    draw_text(&mut ctx, name_label, px + 16.0, py + 110.0, Color::rgb(160, 160, 160), 2.0);
    draw_input_field(&mut ctx, px + 16.0, py + 132.0, PANEL_W - 32.0, name_buf, active_field == 1, cursor_visible);

    let hint1 = "TAB - switch field    ENTER - connect    ESC - back";
    draw_text(&mut ctx, hint1, cx - text_w(hint1, 1.0) / 2.0, py + PANEL_H + 10.0,
        Color::rgb(100, 100, 100), 1.0);

    if let Some(err) = error {
        draw_text(&mut ctx, err, cx - text_w(err, 2.0) / 2.0, py + PANEL_H + 30.0,
            Color::rgb(220, 60, 60), 2.0);
    }
}

fn draw_input_field(ctx: &mut SdlBackend, x: f64, y: f64, w: f64, text: &str, active: bool, cursor_visible: bool) {
    let h = 40.0;
    let bg = if active { Color::rgb(35, 35, 70) } else { Color::rgb(20, 20, 40) };
    ctx.fill_rect(x, y, w, h, bg);

    let border = if active { Color::rgb(100, 100, 220) } else { Color::rgb(60, 60, 100) };
    ctx.stroke_rect(x, y, w, h, border);

    let display = if text.len() > 40 { &text[text.len() - 40..] } else { text };
    draw_text(ctx, display, x + 6.0, y + 10.0, Color::rgb(220, 220, 220), 2.0);

    if active && cursor_visible {
        let cursor_x = x + 6.0 + text_w(display, 2.0);
        ctx.fill_rect(cursor_x, y + 8.0, 2.0, 24.0, Color::rgb(200, 200, 200));
    }
}

pub fn draw_connecting_screen(r: &mut Renderer, addr: &str) {
    let mut ctx = r.backend();
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(10, 10, 30));

    let t1 = "CONNECTING...";
    draw_text(&mut ctx, t1, cx - text_w(t1, 4.0) / 2.0, cy - 60.0, Color::rgb(255, 220, 0), 4.0);
    draw_text(&mut ctx, addr, cx - text_w(addr, 2.0) / 2.0, cy + 10.0, Color::rgb(160, 160, 160), 2.0);

    let hint = "ESC - cancel";
    draw_text(&mut ctx, hint, cx - text_w(hint, 2.0) / 2.0, cy + 60.0, Color::rgb(80, 80, 80), 2.0);
}

pub fn draw_waiting_screen(r: &mut Renderer, player_name: &str) {
    let mut ctx = r.backend();
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(10, 10, 30));

    let t1 = "WAITING FOR OPPONENT";
    draw_text(&mut ctx, t1, cx - text_w(t1, 3.0) / 2.0, cy - 80.0, Color::rgb(255, 220, 0), 3.0);

    let name_str = format!("Playing as: {player_name}");
    draw_text(&mut ctx, &name_str, cx - text_w(&name_str, 2.0) / 2.0, cy - 20.0,
        Color::rgb(160, 160, 160), 2.0);

    let hint = "ESC - cancel";
    draw_text(&mut ctx, hint, cx - text_w(hint, 2.0) / 2.0, cy + 60.0, Color::rgb(80, 80, 80), 2.0);
}
