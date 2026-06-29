use crate::{Color, DrawContext};
use crate::font::{draw_text, text_w};
use crate::layout::{WINDOW_H, WINDOW_W};

/// Returns a static error string if the name is invalid, `None` if valid.
pub fn validate_player_name(name: &str) -> Option<&'static str> {
    if name.is_empty() {
        Some("NAME CANNOT BE EMPTY")
    } else if name.len() > 16 {
        Some("NAME TOO LONG (MAX 16 CHARS)")
    } else {
        None
    }
}

pub fn draw_connecting<D: DrawContext>(ctx: &mut D, addr: &str) {
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(10, 10, 30));

    let t1 = "CONNECTING...";
    draw_text(ctx, t1, cx - text_w(t1, 4.0) / 2.0, cy - 60.0, Color::rgb(255, 220, 0), 4.0);
    draw_text(ctx, addr, cx - text_w(addr, 2.0) / 2.0, cy + 10.0, Color::rgb(160, 160, 160), 2.0);

    let hint = "ESC - cancel";
    draw_text(ctx, hint, cx - text_w(hint, 2.0) / 2.0, cy + 60.0, Color::rgb(80, 80, 80), 2.0);
}

pub fn draw_waiting<D: DrawContext>(ctx: &mut D, player_name: &str) {
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(10, 10, 30));

    let t1 = "WAITING FOR OPPONENT";
    draw_text(ctx, t1, cx - text_w(t1, 3.0) / 2.0, cy - 80.0, Color::rgb(255, 220, 0), 3.0);

    if !player_name.is_empty() {
        let name_str = format!("Playing as: {player_name}");
        draw_text(ctx, &name_str, cx - text_w(&name_str, 2.0) / 2.0, cy - 20.0,
            Color::rgb(160, 160, 160), 2.0);
    }

    let hint = "ESC - cancel";
    draw_text(ctx, hint, cx - text_w(hint, 2.0) / 2.0, cy + 60.0, Color::rgb(80, 80, 80), 2.0);
}

pub fn draw_name_taken<D: DrawContext>(ctx: &mut D) {
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(10, 10, 30));

    let msg = "NAME ALREADY IN USE";
    draw_text(ctx, msg, cx - text_w(msg, 3.0) / 2.0, cy - 20.0, Color::rgb(220, 50, 50), 3.0);

    let sub = "ESC - back to title";
    draw_text(ctx, sub, cx - text_w(sub, 2.0) / 2.0, cy + 20.0, Color::rgb(160, 160, 160), 2.0);
}
