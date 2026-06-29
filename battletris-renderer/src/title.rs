use battletris_engine::ai::LEVELS;

use crate::color::Color;
use crate::context::DrawContext;
use crate::font::{draw_text, text_w};
use crate::layout::{WINDOW_H, WINDOW_W};

pub fn draw_title<D: DrawContext>(ctx: &mut D) {
    let cx = WINDOW_W / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(20, 0, 40));
    ctx.fill_rect(0.0, 200.0, WINDOW_W, 130.0, Color::rgb(50, 0, 100));

    let title = "BATTLETRIS";
    draw_text(ctx, title, cx - text_w(title, 5.0) / 2.0, 225.0, Color::rgb(255, 220, 0), 5.0);

    let sub = "RUST PORT  V1.0";
    draw_text(ctx, sub, cx - text_w(sub, 2.0) / 2.0, 290.0, Color::rgb(200, 200, 200), 2.0);

    ctx.fill_rect(cx - 200.0, 360.0, 400.0, 2.0, Color::rgb(80, 0, 160));

    let s1 = "ENTER  -  PLAY VS COMPUTER";
    draw_text(ctx, s1, cx - text_w(s1, 2.0) / 2.0, 380.0, Color::WHITE, 2.0);

    let s2 = "S  -  SOLO PRACTICE";
    draw_text(ctx, s2, cx - text_w(s2, 2.0) / 2.0, 415.0, Color::rgb(200, 255, 200), 2.0);

    let s3 = "N  -  NETWORK GAME";
    draw_text(ctx, s3, cx - text_w(s3, 2.0) / 2.0, 450.0, Color::WHITE, 2.0);

    ctx.fill_rect(0.0, 555.0, WINDOW_W, 190.0, Color::rgb(40, 0, 60));

    let c1 = "ARROW KEYS: MOVE / ROTATE CW";
    draw_text(ctx, c1, cx - text_w(c1, 2.0) / 2.0, 575.0, Color::rgb(160, 160, 160), 2.0);

    let c2 = "Z: ROTATE CCW   SPACE: HARD DROP   DOWN: SOFT DROP";
    draw_text(ctx, c2, cx - text_w(c2, 1.0) / 2.0, 608.0, Color::rgb(160, 160, 160), 1.0);

    let c3 = "P: PAUSE   ESC: QUIT   B: OPEN SHOP (SOLO)";
    draw_text(ctx, c3, cx - text_w(c3, 1.0) / 2.0, 622.0, Color::rgb(160, 160, 160), 1.0);

    let c4 = "1-9/0: LAUNCH WEAPON";
    draw_text(ctx, c4, cx - text_w(c4, 1.0) / 2.0, 636.0, Color::rgb(140, 200, 140), 1.0);

    let cred = "ORIGINAL BATTLETRIS (1994)  BROWN UNIV CS32";
    draw_text(ctx, cred, cx - text_w(cred, 1.0) / 2.0, 760.0, Color::rgb(80, 80, 80), 1.0);
}

pub fn draw_difficulty_select<D: DrawContext>(ctx: &mut D, selected: usize) {
    let cx = WINDOW_W / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(20, 0, 40));
    ctx.fill_rect(0.0, 130.0, WINDOW_W, 70.0, Color::rgb(50, 0, 100));

    let title = "VS COMPUTER";
    draw_text(ctx, title, cx - text_w(title, 4.0) / 2.0, 145.0, Color::rgb(255, 220, 0), 4.0);

    let sub = "SELECT DIFFICULTY";
    draw_text(ctx, sub, cx - text_w(sub, 2.0) / 2.0, 215.0, Color::rgb(180, 180, 180), 2.0);

    const ROW_H: f64 = 36.0;
    const LIST_X: f64 = 220.0;
    const LIST_W: f64 = 380.0;
    let list_y = 250.0;

    ctx.fill_rect(
        LIST_X - 10.0, list_y - 6.0, LIST_W + 20.0, ROW_H * LEVELS.len() as f64 + 12.0,
        Color::rgb(15, 0, 30),
    );
    ctx.stroke_rect(
        LIST_X - 10.0, list_y - 6.0, LIST_W + 20.0, ROW_H * LEVELS.len() as f64 + 12.0,
        Color::rgb(80, 0, 140),
    );

    for (i, &(name, ms)) in LEVELS.iter().enumerate() {
        let row_y = list_y + i as f64 * ROW_H;
        let is_sel = i == selected;

        if is_sel {
            ctx.fill_rect(LIST_X - 8.0, row_y - 2.0, LIST_W + 16.0, ROW_H - 2.0, Color::rgb(80, 0, 150));
        }

        let arrow = if is_sel { ">" } else { " " };
        let arrow_col = if is_sel { Color::rgb(255, 220, 0) } else { Color::rgb(60, 0, 80) };
        draw_text(ctx, arrow, LIST_X, row_y + 6.0, arrow_col, 2.0);

        let name_col = if is_sel { Color::WHITE } else { Color::rgb(160, 140, 180) };
        draw_text(ctx, name, LIST_X + 24.0, row_y + 6.0, name_col, 2.0);

        let delay_str = if ms == 0 { "instant".to_string() } else { format!("{ms}ms") };
        let delay_col = if is_sel { Color::rgb(200, 200, 255) } else { Color::rgb(100, 80, 120) };
        let delay_x = LIST_X + LIST_W - text_w(&delay_str, 2.0) - 8.0;
        draw_text(ctx, &delay_str, delay_x, row_y + 6.0, delay_col, 2.0);
    }

    let hint = "UP/DOWN - select    ENTER - start    ESC - back";
    draw_text(
        ctx, hint, cx - text_w(hint, 1.0) / 2.0,
        list_y + LEVELS.len() as f64 * ROW_H + 24.0,
        Color::rgb(100, 80, 120), 1.0,
    );
}
