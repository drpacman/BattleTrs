use web_sys::CanvasRenderingContext2d;

use battletris_engine::engine::weapons::{weapon_def, BazaarStateView, WEAPON_COUNT};

use super::{draw_text, text_w, WINDOW_H, WINDOW_W};

const VISIBLE_ROWS: usize = 24;
const ROW_H: f64 = 20.0;
const LIST_Y: f64 = 165.0;

pub fn draw_bazaar(ctx: &CanvasRenderingContext2d, state: &BazaarStateView) {
    ctx.set_fill_style_str("#0a0014");
    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H);

    ctx.set_fill_style_str("#50008c");
    ctx.fill_rect(0.0, 80.0, WINDOW_W, 50.0);
    let title = "BAZAAR";
    let tw = text_w(title, 4.0);
    draw_text(ctx, title, (WINDOW_W - tw) / 2.0, 91.0, "#ffdc64", 4.0);

    let funds_str = if state.player_funds < 0 {
        format!("FUNDS: $-{}", -state.player_funds)
    } else {
        format!("FUNDS: ${}", state.player_funds)
    };
    let funds_color = if state.player_funds < 0 { "#dc3c3c" } else { "#b4dcb4" };
    draw_text(ctx, &funds_str, 30.0, 140.0, funds_color, 2.0);

    let scroll_offset = state
        .selected
        .saturating_sub(VISIBLE_ROWS / 2)
        .min(WEAPON_COUNT.saturating_sub(VISIBLE_ROWS));

    for i in scroll_offset..(scroll_offset + VISIBLE_ROWS).min(WEAPON_COUNT) {
        let kind = state.weapons[i];
        let def = weapon_def(kind);
        let price = if state.carter_active { def.price * 2 } else { def.price };
        let affordable = state.player_funds >= price as i64;
        let row_y = LIST_Y + (i - scroll_offset) as f64 * ROW_H;

        if i == state.selected {
            ctx.set_fill_style_str("#3c0078");
            ctx.fill_rect(20.0, row_y - 1.0, WINDOW_W - 40.0, ROW_H);
        }

        if i == state.selected {
            draw_text(ctx, ">", 24.0, row_y + 1.0, "#ffdc00", 2.0);
        }

        let name_color = if affordable { "#dcdcdc" } else { "#464646" };
        draw_text(ctx, def.name, 44.0, row_y + 1.0, name_color, 2.0);

        let price_str = format!("${price}");
        let price_color = if affordable { "#ffdc50" } else { "#3c3c3c" };
        let pw = text_w(&price_str, 2.0);
        draw_text(ctx, &price_str, WINDOW_W - 40.0 - pw, row_y + 1.0, price_color, 2.0);
    }

    if scroll_offset > 0 {
        draw_text(ctx, "^^ more above ^^", 340.0, LIST_Y - 16.0, "#787878", 1.0);
    }
    if scroll_offset + VISIBLE_ROWS < WEAPON_COUNT {
        draw_text(
            ctx, "vv more below vv", 340.0,
            LIST_Y + VISIBLE_ROWS as f64 * ROW_H, "#787878", 1.0,
        );
    }

    ctx.set_fill_style_str("#190a28");
    ctx.fill_rect(20.0, 690.0, WINDOW_W - 40.0, 80.0);
    ctx.set_stroke_style_str("#50008c");
    ctx.set_line_width(1.0);
    ctx.stroke_rect(20.0, 690.0, WINDOW_W - 40.0, 80.0);

    let sel_kind = state.weapons[state.selected];
    let sel_def = weapon_def(sel_kind);
    draw_text(ctx, sel_def.name, 30.0, 698.0, "#ffc850", 2.0);
    draw_text(ctx, sel_def.description, 30.0, 720.0, "#b4b4b4", 2.0);

    ctx.set_fill_style_str("#1e0032");
    ctx.fill_rect(0.0, 790.0, WINDOW_W, 40.0);
    draw_text(ctx, "[UP/DN] SELECT", 30.0, 800.0, "#8c8c8c", 2.0);
    draw_text(ctx, "[ENTER] BUY", 270.0, 800.0, "#8c8c8c", 2.0);
    draw_text(ctx, "[ESC] DONE", 500.0, 800.0, "#8c8c8c", 2.0);
}

pub fn draw_quit_confirm(ctx: &CanvasRenderingContext2d) {
    let box_w = 320.0;
    let box_h = 110.0;
    let bx = (WINDOW_W - box_w) / 2.0;
    let by = (WINDOW_H - box_h) / 2.0;

    ctx.set_fill_style_str("#c8b400");
    ctx.fill_rect(bx - 2.0, by - 2.0, box_w + 4.0, box_h + 4.0);

    ctx.set_fill_style_str("#0f0f0f");
    ctx.fill_rect(bx, by, box_w, box_h);

    let cx = bx + box_w / 2.0;

    let title = "QUIT GAME?";
    draw_text(ctx, title, cx - text_w(title, 3.0) / 2.0, by + 14.0, "#ffdc00", 3.0);

    let yes = "Y - QUIT";
    draw_text(ctx, yes, cx - text_w(yes, 2.0) / 2.0, by + 52.0, "#dc5050", 2.0);

    let no = "N - CONTINUE";
    draw_text(ctx, no, cx - text_w(no, 2.0) / 2.0, by + 76.0, "#50dc50", 2.0);
}
