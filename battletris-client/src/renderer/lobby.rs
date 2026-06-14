use sdl2::pixels::Color;
use sdl2::rect::Rect;

use super::{draw_text, text_w, Renderer, WINDOW_H, WINDOW_W};

const PANEL_W: u32 = 500;
const PANEL_H: u32 = 260;

pub fn draw_connection_screen(
    r: &mut Renderer,
    addr_buf: &str,
    name_buf: &str,
    active_field: usize, // 0 = address, 1 = name
    cursor_visible: bool,
    error: Option<&str>,
) {
    let cx = WINDOW_W as i32 / 2;

    // Background
    r.canvas.set_draw_color(Color::RGB(10, 10, 30));
    let _ = r.canvas.fill_rect(Rect::new(0, 0, WINDOW_W, WINDOW_H));

    // Title
    let title = "NETWORK GAME";
    draw_text(&mut r.canvas, title, cx - text_w(title, 4) / 2, 140,
        Color::RGB(255, 220, 0), 4);

    // Panel
    let px = cx - PANEL_W as i32 / 2;
    let py = 230i32;
    r.canvas.set_draw_color(Color::RGB(25, 25, 50));
    let _ = r.canvas.fill_rect(Rect::new(px, py, PANEL_W, PANEL_H));
    r.canvas.set_draw_color(Color::RGB(80, 80, 160));
    let _ = r.canvas.draw_rect(Rect::new(px, py, PANEL_W, PANEL_H));

    // Server address field
    let addr_label = "SERVER ADDRESS:";
    draw_text(&mut r.canvas, addr_label, px + 16, py + 20, Color::RGB(160, 160, 160), 2);
    draw_input_field(r, px + 16, py + 42, PANEL_W - 32, addr_buf, active_field == 0, cursor_visible);

    // Player name field
    let name_label = "YOUR NAME:";
    draw_text(&mut r.canvas, name_label, px + 16, py + 110, Color::RGB(160, 160, 160), 2);
    draw_input_field(r, px + 16, py + 132, PANEL_W - 32, name_buf, active_field == 1, cursor_visible);

    // Instructions
    let hint1 = "TAB - switch field    ENTER - connect    ESC - back";
    draw_text(&mut r.canvas, hint1, cx - text_w(hint1, 1) / 2, py + PANEL_H as i32 + 10,
        Color::RGB(100, 100, 100), 1);

    // Error message
    if let Some(err) = error {
        let err_str = err;
        draw_text(&mut r.canvas, err_str, cx - text_w(err_str, 2) / 2, py + PANEL_H as i32 + 30,
            Color::RGB(220, 60, 60), 2);
    }
}

fn draw_input_field(
    r: &mut Renderer,
    x: i32, y: i32, w: u32,
    text: &str,
    active: bool,
    cursor_visible: bool,
) {
    let h = 40u32;
    // Field background
    let bg = if active { Color::RGB(35, 35, 70) } else { Color::RGB(20, 20, 40) };
    r.canvas.set_draw_color(bg);
    let _ = r.canvas.fill_rect(Rect::new(x, y, w, h));

    // Border
    let border = if active { Color::RGB(100, 100, 220) } else { Color::RGB(60, 60, 100) };
    r.canvas.set_draw_color(border);
    let _ = r.canvas.draw_rect(Rect::new(x, y, w, h));

    // Text (truncated to fit)
    let display = if text.len() > 40 { &text[text.len() - 40..] } else { text };
    draw_text(&mut r.canvas, display, x + 6, y + 10, Color::RGB(220, 220, 220), 2);

    // Cursor
    if active && cursor_visible {
        let cursor_x = x + 6 + text_w(display, 2);
        r.canvas.set_draw_color(Color::RGB(200, 200, 200));
        let _ = r.canvas.fill_rect(Rect::new(cursor_x, y + 8, 2, 24));
    }
}

pub fn draw_connecting_screen(r: &mut Renderer, addr: &str) {
    let cx = WINDOW_W as i32 / 2;
    let cy = WINDOW_H as i32 / 2;

    r.canvas.set_draw_color(Color::RGB(10, 10, 30));
    let _ = r.canvas.fill_rect(Rect::new(0, 0, WINDOW_W, WINDOW_H));

    let t1 = "CONNECTING...";
    draw_text(&mut r.canvas, t1, cx - text_w(t1, 4) / 2, cy - 60,
        Color::RGB(255, 220, 0), 4);

    draw_text(&mut r.canvas, addr, cx - text_w(addr, 2) / 2, cy + 10,
        Color::RGB(160, 160, 160), 2);

    let hint = "ESC - cancel";
    draw_text(&mut r.canvas, hint, cx - text_w(hint, 2) / 2, cy + 60,
        Color::RGB(80, 80, 80), 2);
}

pub fn draw_waiting_screen(r: &mut Renderer, player_name: &str) {
    let cx = WINDOW_W as i32 / 2;
    let cy = WINDOW_H as i32 / 2;

    r.canvas.set_draw_color(Color::RGB(10, 10, 30));
    let _ = r.canvas.fill_rect(Rect::new(0, 0, WINDOW_W, WINDOW_H));

    let t1 = "WAITING FOR OPPONENT";
    draw_text(&mut r.canvas, t1, cx - text_w(t1, 3) / 2, cy - 80,
        Color::RGB(255, 220, 0), 3);

    let name_str = format!("Playing as: {player_name}");
    draw_text(&mut r.canvas, &name_str, cx - text_w(&name_str, 2) / 2, cy - 20,
        Color::RGB(160, 160, 160), 2);

    let hint = "ESC - cancel";
    draw_text(&mut r.canvas, hint, cx - text_w(hint, 2) / 2, cy + 60,
        Color::RGB(80, 80, 80), 2);
}
