use web_sys::CanvasRenderingContext2d;

use super::{draw_text, text_w, WINDOW_H, WINDOW_W};

pub fn draw_connecting(ctx: &CanvasRenderingContext2d) {
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;
    let msg = "CONNECTING...";
    draw_text(ctx, msg, cx - text_w(msg, 3.0) / 2.0, cy - 10.0, "#646464", 3.0);
}

pub fn draw_waiting(ctx: &CanvasRenderingContext2d) {
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;
    let msg = "WAITING FOR OPPONENT...";
    draw_text(ctx, msg, cx - text_w(msg, 2.0) / 2.0, cy - 10.0, "#a0a0a0", 2.0);
}

pub fn draw_name_taken(ctx: &CanvasRenderingContext2d) {
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;
    let msg = "NAME ALREADY IN USE";
    draw_text(ctx, msg, cx - text_w(msg, 3.0) / 2.0, cy - 20.0, "#dc3232", 3.0);
    let sub = "RELOAD TO TRY AGAIN";
    draw_text(ctx, sub, cx - text_w(sub, 2.0) / 2.0, cy + 20.0, "#a0a0a0", 2.0);
}

pub fn draw_disconnected(ctx: &CanvasRenderingContext2d) {
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;
    let msg = "DISCONNECTED";
    draw_text(ctx, msg, cx - text_w(msg, 3.0) / 2.0, cy - 20.0, "#dc3232", 3.0);
    let sub = "RELOAD TO RECONNECT";
    draw_text(ctx, sub, cx - text_w(sub, 2.0) / 2.0, cy + 20.0, "#a0a0a0", 2.0);
}

pub fn draw_game_over(
    ctx: &CanvasRenderingContext2d,
    won: bool,
    score: u32,
    lines: u32,
    winner_name: Option<&str>,
    elo_delta: Option<i32>,
) {
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;

    ctx.set_fill_style_str("rgba(0, 0, 0, 0.78)");
    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H);

    let panel_h = if winner_name.is_some() { 390.0 } else { 340.0 };

    ctx.set_fill_style_str("#140a28");
    ctx.fill_rect(cx - 220.0, cy - 195.0, 440.0, panel_h);
    ctx.set_stroke_style_str("#5000a0");
    ctx.set_line_width(1.0);
    ctx.stroke_rect(cx - 220.0, cy - 195.0, 440.0, panel_h);

    let (result_text, result_color) = if won {
        ("YOU WIN!", "#32dc32")
    } else {
        ("GAME OVER", "#dc3232")
    };
    draw_text(
        ctx, result_text,
        cx - text_w(result_text, 4.0) / 2.0, cy - 170.0,
        result_color, 4.0,
    );

    let mut y = cy - 100.0;
    if let Some(name) = winner_name {
        let label = if won {
            format!("You beat {name}!")
        } else {
            format!("{name} wins")
        };
        draw_text(ctx, &label, cx - text_w(&label, 2.0) / 2.0, y, "#c8c8c8", 2.0);
        y += 26.0;
    }

    let score_str = format!("SCORE: {score}");
    let lines_str = format!("LINES: {lines}");
    draw_text(ctx, &score_str, cx - text_w(&score_str, 3.0) / 2.0, y, "#ffdc00", 3.0);
    y += 30.0;
    draw_text(ctx, &lines_str, cx - text_w(&lines_str, 3.0) / 2.0, y, "#c8c8c8", 3.0);
    y += 40.0;

    if let Some(delta) = elo_delta {
        let (delta_str, delta_col) = if delta >= 0 {
            (format!("ELO: +{delta}"), "#50dc50")
        } else {
            (format!("ELO: {delta}"), "#dc5050")
        };
        draw_text(ctx, &delta_str, cx - text_w(&delta_str, 3.0) / 2.0, y, delta_col, 3.0);
        y += 40.0;
    }

    let p1 = "ENTER - PLAY AGAIN";
    let p2 = "ESC - QUIT";
    draw_text(ctx, p1, cx - text_w(p1, 2.0) / 2.0, y, "#b4b4b4", 2.0);
    draw_text(ctx, p2, cx - text_w(p2, 2.0) / 2.0, y + 25.0, "#b4b4b4", 2.0);
}
