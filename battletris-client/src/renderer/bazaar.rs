use sdl2::pixels::Color;
use sdl2::rect::Rect;

use battletris_engine::engine::weapons::{BazaarStateView, WEAPON_COUNT, weapon_def};

use super::{draw_text, text_w, Renderer, WINDOW_W, WINDOW_H};

const VISIBLE_ROWS: usize = 24;
const ROW_H: i32 = 20;
const LIST_Y: i32 = 165;

pub fn draw_bazaar(r: &mut Renderer, state: &BazaarStateView) {
    let canvas = &mut r.canvas;

    // ── Background overlay ────────────────────────────────────────────────────
    canvas.set_draw_color(Color::RGB(10, 0, 20));
    let _ = canvas.fill_rect(Rect::new(0, 0, WINDOW_W, WINDOW_H));

    // ── Title bar ─────────────────────────────────────────────────────────────
    canvas.set_draw_color(Color::RGB(80, 0, 140));
    let _ = canvas.fill_rect(Rect::new(0, 80, WINDOW_W, 50));
    let title = "BAZAAR";
    let tw = text_w(title, 4);
    draw_text(canvas, title, (WINDOW_W as i32 - tw) / 2, 91, Color::RGB(255, 220, 100), 4);

    // ── Funds display ─────────────────────────────────────────────────────────
    let funds_str = if state.player_funds < 0 {
        format!("FUNDS: $-{}", -state.player_funds)
    } else {
        format!("FUNDS: ${}", state.player_funds)
    };
    let funds_color = if state.player_funds < 0 {
        Color::RGB(220, 60, 60)
    } else {
        Color::RGB(180, 220, 180)
    };
    draw_text(canvas, &funds_str, 30, 140, funds_color, 2);

    // ── Weapon list ───────────────────────────────────────────────────────────
    let scroll_offset = state.selected.saturating_sub(VISIBLE_ROWS / 2)
        .min(WEAPON_COUNT.saturating_sub(VISIBLE_ROWS));

    for i in scroll_offset..(scroll_offset + VISIBLE_ROWS).min(WEAPON_COUNT) {
        let kind = state.weapons[i];
        let def = weapon_def(kind);
        let price = if state.carter_active { def.price * 2 } else { def.price };
        let affordable = state.player_funds >= price as i64;
        let row_y = LIST_Y + (i - scroll_offset) as i32 * ROW_H;

        // Selected row highlight
        if i == state.selected {
            canvas.set_draw_color(Color::RGB(60, 0, 120));
            let _ = canvas.fill_rect(Rect::new(20, row_y - 1, WINDOW_W - 40, ROW_H as u32));
        }

        // Cursor
        if i == state.selected {
            draw_text(canvas, ">", 24, row_y + 1, Color::RGB(255, 220, 0), 2);
        }

        // Weapon name
        let name_color = if affordable {
            Color::RGB(220, 220, 220)
        } else {
            Color::RGB(70, 70, 70)
        };
        let name = def.name;
        draw_text(canvas, name, 44, row_y + 1, name_color, 2);

        // Price
        let price_str = format!("${}", price);
        let price_color = if affordable {
            Color::RGB(255, 220, 80)
        } else {
            Color::RGB(60, 60, 60)
        };
        let pw = text_w(&price_str, 2);
        draw_text(canvas, &price_str, WINDOW_W as i32 - 40 - pw, row_y + 1, price_color, 2);
    }

    // ── Scroll indicators ─────────────────────────────────────────────────────
    if scroll_offset > 0 {
        draw_text(canvas, "^^ more above ^^", 340, LIST_Y - 16, Color::RGB(120, 120, 120), 1);
    }
    if scroll_offset + VISIBLE_ROWS < WEAPON_COUNT {
        draw_text(canvas, "vv more below vv", 340, LIST_Y + VISIBLE_ROWS as i32 * ROW_H, Color::RGB(120, 120, 120), 1);
    }

    // ── Description box ───────────────────────────────────────────────────────
    canvas.set_draw_color(Color::RGB(25, 10, 40));
    let _ = canvas.fill_rect(Rect::new(20, 690, WINDOW_W - 40, 80));
    canvas.set_draw_color(Color::RGB(80, 0, 140));
    let _ = canvas.draw_rect(Rect::new(20, 690, WINDOW_W - 40, 80));

    let selected_kind = state.weapons[state.selected];
    let selected_def = weapon_def(selected_kind);
    draw_text(canvas, selected_def.name, 30, 698, Color::RGB(255, 200, 80), 2);
    draw_text(canvas, selected_def.description, 30, 720, Color::RGB(180, 180, 180), 2);

    // ── Controls bar ──────────────────────────────────────────────────────────
    canvas.set_draw_color(Color::RGB(30, 0, 50));
    let _ = canvas.fill_rect(Rect::new(0, 790, WINDOW_W, 40));
    draw_text(canvas, "[UP/DN] SELECT", 30, 800, Color::RGB(140, 140, 140), 2);
    draw_text(canvas, "[ENTER] BUY", 270, 800, Color::RGB(140, 140, 140), 2);
    draw_text(canvas, "[ESC] DONE", 500, 800, Color::RGB(140, 140, 140), 2);
}
