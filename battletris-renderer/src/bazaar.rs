use battletris_engine::engine::weapons::{weapon_def, BazaarStateView, WEAPON_COUNT};

use crate::color::Color;
use crate::context::DrawContext;
use crate::font::{draw_text, text_w};
use crate::layout::{WINDOW_H, WINDOW_W};

const VISIBLE_ROWS: usize = 24;
const ROW_H: f64 = 20.0;
const LIST_Y: f64 = 165.0;

pub fn draw_bazaar<D: DrawContext>(ctx: &mut D, state: &BazaarStateView) {
    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(10, 0, 20));

    ctx.fill_rect(0.0, 80.0, WINDOW_W, 50.0, Color::rgb(80, 0, 140));
    let title = "BAZAAR";
    let tw = text_w(title, 4.0);
    draw_text(ctx, title, (WINDOW_W - tw) / 2.0, 91.0, Color::rgb(255, 220, 100), 4.0);

    let funds_str = if state.player_funds < 0 {
        format!("FUNDS: $-{}", -state.player_funds)
    } else {
        format!("FUNDS: ${}", state.player_funds)
    };
    let funds_color = if state.player_funds < 0 {
        Color::rgb(220, 60, 60)
    } else {
        Color::rgb(180, 220, 180)
    };
    draw_text(ctx, &funds_str, 30.0, 140.0, funds_color, 2.0);

    let scroll_offset = state.selected
        .saturating_sub(VISIBLE_ROWS / 2)
        .min(WEAPON_COUNT.saturating_sub(VISIBLE_ROWS));

    for i in scroll_offset..(scroll_offset + VISIBLE_ROWS).min(WEAPON_COUNT) {
        let kind = state.weapons[i];
        let def = weapon_def(kind);
        let price = if state.carter_active { def.price * 2 } else { def.price };
        let affordable = state.player_funds >= price as i64;
        let row_y = LIST_Y + (i - scroll_offset) as f64 * ROW_H;

        if i == state.selected {
            ctx.fill_rect(20.0, row_y - 1.0, WINDOW_W - 40.0, ROW_H, Color::rgb(60, 0, 120));
            draw_text(ctx, ">", 24.0, row_y + 1.0, Color::rgb(255, 220, 0), 2.0);
        }

        let name_color = if affordable { Color::rgb(220, 220, 220) } else { Color::rgb(70, 70, 70) };
        draw_text(ctx, def.name, 44.0, row_y + 1.0, name_color, 2.0);

        let price_str = format!("${}", price);
        let price_color = if affordable { Color::rgb(255, 220, 80) } else { Color::rgb(60, 60, 60) };
        let pw = text_w(&price_str, 2.0);
        draw_text(ctx, &price_str, WINDOW_W - 40.0 - pw, row_y + 1.0, price_color, 2.0);
    }

    if scroll_offset > 0 {
        draw_text(ctx, "^^ more above ^^", 340.0, LIST_Y - 16.0, Color::rgb(120, 120, 120), 1.0);
    }
    if scroll_offset + VISIBLE_ROWS < WEAPON_COUNT {
        draw_text(ctx, "vv more below vv", 340.0,
            LIST_Y + VISIBLE_ROWS as f64 * ROW_H, Color::rgb(120, 120, 120), 1.0);
    }

    ctx.fill_rect(20.0, 690.0, WINDOW_W - 40.0, 80.0, Color::rgb(25, 10, 40));
    ctx.stroke_rect(20.0, 690.0, WINDOW_W - 40.0, 80.0, Color::rgb(80, 0, 140));

    let sel_def = weapon_def(state.weapons[state.selected]);
    draw_text(ctx, sel_def.name, 30.0, 698.0, Color::rgb(255, 200, 80), 2.0);
    draw_text(ctx, sel_def.description, 30.0, 720.0, Color::rgb(180, 180, 180), 2.0);

    ctx.fill_rect(0.0, 790.0, WINDOW_W, 40.0, Color::rgb(30, 0, 50));
    if state.player_done {
        let msg = "WAITING FOR OPPONENT...";
        let mw = text_w(msg, 2.0);
        draw_text(ctx, msg, (WINDOW_W - mw) / 2.0, 800.0, Color::rgb(200, 180, 60), 2.0);
    } else {
        draw_text(ctx, "[UP/DN] SELECT", 30.0, 800.0, Color::rgb(140, 140, 140), 2.0);
        draw_text(ctx, "[ENTER] BUY", 270.0, 800.0, Color::rgb(140, 140, 140), 2.0);
        draw_text(ctx, "[ESC] DONE", 500.0, 800.0, Color::rgb(140, 140, 140), 2.0);
    }
}
