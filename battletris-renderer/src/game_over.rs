use crate::color::Color;
use crate::context::DrawContext;
use crate::font::{draw_text, text_w};
use crate::layout::{WINDOW_H, WINDOW_W};

pub fn draw_game_over<D: DrawContext>(
    ctx: &mut D,
    won: bool,
    score: u32,
    lines: u32,
    winner_name: Option<&str>,
    elo_delta: Option<i32>,
) {
    let cx = WINDOW_W / 2.0;
    let cy = WINDOW_H / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgba(0, 0, 0, 200));

    let panel_h = if winner_name.is_some() { 390.0 } else { 340.0 };
    ctx.fill_rect(cx - 220.0, cy - 195.0, 440.0, panel_h, Color::rgb(20, 10, 40));
    ctx.stroke_rect(cx - 220.0, cy - 195.0, 440.0, panel_h, Color::rgb(80, 0, 160));

    let (result_text, result_color) = if won {
        ("YOU WIN!", Color::rgb(50, 220, 50))
    } else {
        ("GAME OVER", Color::rgb(220, 50, 50))
    };
    draw_text(ctx, result_text, cx - text_w(result_text, 4.0) / 2.0, cy - 170.0, result_color, 4.0);

    let mut y = cy - 100.0;
    if let Some(name) = winner_name {
        let label = if won { format!("You beat {name}!") } else { format!("{name} wins") };
        draw_text(ctx, &label, cx - text_w(&label, 2.0) / 2.0, y, Color::rgb(200, 200, 200), 2.0);
        y += 26.0;
    }

    let score_str = format!("SCORE: {score}");
    let lines_str = format!("LINES: {lines}");
    draw_text(ctx, &score_str, cx - text_w(&score_str, 3.0) / 2.0, y, Color::rgb(255, 220, 0), 3.0);
    y += 30.0;
    draw_text(ctx, &lines_str, cx - text_w(&lines_str, 3.0) / 2.0, y, Color::rgb(200, 200, 200), 3.0);
    y += 40.0;

    if let Some(delta) = elo_delta {
        let (delta_str, delta_col) = if delta >= 0 {
            (format!("ELO: +{delta}"), Color::rgb(80, 220, 80))
        } else {
            (format!("ELO: {delta}"), Color::rgb(220, 80, 80))
        };
        draw_text(ctx, &delta_str, cx - text_w(&delta_str, 3.0) / 2.0, y, delta_col, 3.0);
        y += 40.0;
    }

    let p1 = "ENTER  -  PLAY AGAIN";
    let p2 = "ESC  -  TITLE SCREEN";
    draw_text(ctx, p1, cx - text_w(p1, 2.0) / 2.0, y,        Color::rgb(180, 180, 180), 2.0);
    draw_text(ctx, p2, cx - text_w(p2, 2.0) / 2.0, y + 25.0, Color::rgb(180, 180, 180), 2.0);
}
