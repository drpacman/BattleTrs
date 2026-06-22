use web_sys::CanvasRenderingContext2d;

use battletris_engine::engine::board::BoardSnapshot;
use battletris_engine::engine::game_state::PlayingView;

use super::{
    bt_color, cell_color_str, char_step, draw_active_piece, draw_board, draw_cell, draw_die_pips,
    draw_face, draw_ghost_piece, draw_next_piece, draw_text, text_w,
    BOARD_PX_H, BOARD_PX_W, CELL_PX, OPP_BOARD_X, OPP_BOARD_Y, PLAYER_BOARD_X, PLAYER_BOARD_Y,
    STATS_X, STATS_Y,
};

pub fn draw_playing(ctx: &CanvasRenderingContext2d, view: &PlayingView) {
    // ── Player board ──────────────────────────────────────────────────────
    draw_board_with_effects(ctx, view, true);

    if let Some((kind, ref cells)) = view.active_piece {
        draw_ghost_piece(
            ctx, kind, &view.ghost_cells,
            PLAYER_BOARD_X, PLAYER_BOARD_Y, view.upbyside_active,
        );
        draw_active_piece(
            ctx, kind, cells,
            PLAYER_BOARD_X, PLAYER_BOARD_Y, view.upbyside_active,
        );
    }

    if view.gimp_flash {
        ctx.set_fill_style_str("rgba(0, 0, 0, 0.7)");
        ctx.fill_rect(PLAYER_BOARD_X, PLAYER_BOARD_Y, BOARD_PX_W, BOARD_PX_H);
        let text = "GIMP!";
        let tw = text_w(text, 5.0);
        draw_text(
            ctx, text,
            PLAYER_BOARD_X + (BOARD_PX_W - tw) / 2.0,
            PLAYER_BOARD_Y + BOARD_PX_H / 2.0 - 18.0,
            "#ff3232", 5.0,
        );
    }

    if view.upbyside_active {
        draw_text(ctx, "UPSIDE DOWN", PLAYER_BOARD_X + 20.0, PLAYER_BOARD_Y - 18.0, "#dc3c3c", 2.0);
    }

    // ── Opponent board ────────────────────────────────────────────────────
    draw_board_with_effects(ctx, view, false);

    // ── Peer disconnected overlay ─────────────────────────────────────────
    if view.peer_disconnected {
        ctx.set_fill_style_str("rgba(0, 0, 0, 0.63)");
        ctx.fill_rect(0.0, 0.0, super::WINDOW_W, super::WINDOW_H);
        let t1 = "OPPONENT DISCONNECTED";
        let t2 = "Waiting 15s to reconnect...";
        let cx = super::WINDOW_W / 2.0;
        let cy = super::WINDOW_H / 2.0;
        draw_text(ctx, t1, cx - text_w(t1, 3.0) / 2.0, cy - 30.0, "#dcb400", 3.0);
        draw_text(ctx, t2, cx - text_w(t2, 2.0) / 2.0, cy + 20.0, "#a0a0a0", 2.0);
    }

    // ── Weapon chips ──────────────────────────────────────────────────────
    draw_weapon_chips(ctx, &view.player_active_weapons, PLAYER_BOARD_X);
    draw_weapon_chips(ctx, &view.ernie_active_weapons, OPP_BOARD_X);

    // ── Stats panel ───────────────────────────────────────────────────────
    draw_stats(ctx, view);
}

fn draw_board_with_effects(ctx: &CanvasRenderingContext2d, view: &PlayingView, is_player: bool) {
    let origin_x = if is_player { PLAYER_BOARD_X } else { OPP_BOARD_X };
    let origin_y = if is_player { PLAYER_BOARD_Y } else { OPP_BOARD_Y };

    let snapshot: Option<&BoardSnapshot> = if is_player {
        Some(&view.player_board)
    } else {
        view.opponent_board.as_ref()
    };

    let Some(snapshot) = snapshot else {
        ctx.set_fill_style_str("#0f0f0f");
        ctx.fill_rect(origin_x, origin_y, BOARD_PX_W, BOARD_PX_H);
        draw_text(ctx, "WAITING...", origin_x + 50.0, origin_y + 375.0, "#3c3c3c", 3.0);
        return;
    };

    let upbyside = is_player && view.upbyside_active;
    let blind = if is_player { &view.blind_cells[..] } else { &[] };
    let twilight = is_player && view.twilight_active;

    draw_board(ctx, snapshot, origin_x, origin_y, upbyside, blind, twilight);

    if !is_player && view.opponent_gimp_flash {
        ctx.set_fill_style_str("rgba(0, 0, 0, 0.7)");
        ctx.fill_rect(origin_x, origin_y, BOARD_PX_W, BOARD_PX_H);
        let text = "GIMP!";
        let tw = text_w(text, 5.0);
        draw_text(
            ctx, text,
            origin_x + (BOARD_PX_W - tw) / 2.0,
            origin_y + BOARD_PX_H / 2.0 - 18.0,
            "#ff3232", 5.0,
        );
    }
}

fn draw_weapon_chips(
    ctx: &CanvasRenderingContext2d,
    weapons: &[battletris_engine::engine::weapons::ActiveWeaponView],
    board_x: f64,
) {
    let chip_y = PLAYER_BOARD_Y + BOARD_PX_H + 4.0;
    for (i, wpn) in weapons.iter().enumerate().take(3) {
        let x = board_x + i as f64 * 92.0;
        ctx.set_fill_style_str("#3c0064");
        ctx.fill_rect(x, chip_y, 88.0, 16.0);
        let name = if wpn.name.len() > 10 { &wpn.name[..10] } else { wpn.name };
        draw_text(ctx, name, x + 2.0, chip_y + 2.0, "#ffdc50", 1.0);
        let rem_str = format!("{}L", wpn.remaining_lines);
        draw_text(ctx, &rem_str, x + 66.0, chip_y + 2.0, "#c8c8c8", 1.0);
    }
}

fn draw_stats(ctx: &CanvasRenderingContext2d, view: &PlayingView) {
    let sv = &view.score;
    let sx = STATS_X;
    let mut sy = STATS_Y;

    ctx.set_fill_style_str("#1e1e1e");
    ctx.fill_rect(sx, sy, 190.0, BOARD_PX_H);

    sy += 10.0;

    draw_text(ctx, "NEXT", sx + 71.0, sy, "#b4b4b4", 2.0);
    sy += 20.0;

    ctx.set_fill_style_str("#141414");
    ctx.fill_rect(sx + 30.0, sy, 60.0, 60.0);
    draw_next_piece(ctx, view.next_piece, sx + 35.0, sy + 5.0);
    sy += 75.0;

    section_label(ctx, sx, &mut sy, "PLAYER");
    stat_row(ctx, sx, &mut sy, "SCORE", &format!("{}", sv.score));
    stat_row(ctx, sx, &mut sy, "LINES", &format!("{}", sv.lines));

    let funds_str = if sv.funds < 0 {
        format!("$-{}", -sv.funds)
    } else {
        format!("${}", sv.funds)
    };
    let funds_color = if sv.funds < 0 { "#dc3c3c" } else { "#ffffff" };
    draw_text(ctx, "FUNDS", sx + 5.0, sy, "#8c8c8c", 2.0);
    draw_text(ctx, &funds_str, sx + 110.0, sy, funds_color, 2.0);
    sy += 18.0;

    sy += 8.0;

    let baz_color = if sv.lines_until_bazaar <= 5 { "#ffdc00" } else { "#b4b4b4" };
    draw_text(ctx, "TIL BAZ", sx + 10.0, sy, "#787878", 2.0);
    sy += 18.0;
    draw_text(ctx, &format!("{}", sv.lines_until_bazaar), sx + 75.0, sy, baz_color, 3.0);
    sy += 34.0;

    section_label(ctx, sx, &mut sy, "OPPONENT");
    stat_row(ctx, sx, &mut sy, "SCORE", &format!("{}", sv.op_score));
    stat_row(ctx, sx, &mut sy, "LINES", &format!("{}", sv.op_lines));
    if sv.show_op_funds {
        let op_str = if sv.op_funds < 0 {
            format!("$-{}", -sv.op_funds)
        } else {
            format!("${}", sv.op_funds)
        };
        stat_row(ctx, sx, &mut sy, "FUNDS", &op_str);
    } else {
        stat_row(ctx, sx, &mut sy, "FUNDS", "???");
    }

    sy += 8.0;

    section_label(ctx, sx, &mut sy, "ARSENAL");
    if view.player_arsenal.is_empty() {
        ctx.set_fill_style_str("#282828");
        ctx.fill_rect(sx + 10.0, sy, 170.0, 18.0);
        draw_text(ctx, "--", sx + 80.0, sy + 2.0, "#3c3c3c", 2.0);
        sy += 22.0;
    } else {
        for slot in &view.player_arsenal {
            ctx.set_fill_style_str("#282828");
            ctx.fill_rect(sx + 5.0, sy, 180.0, 14.0);
            let qty_suffix = if slot.quantity > 1 {
                format!("({})", slot.quantity)
            } else {
                String::new()
            };
            let label = format!("{}: {} {}", slot.key, slot.name, qty_suffix);
            draw_text(ctx, &label, sx + 7.0, sy + 1.0, "#c8c8c8", 1.0);
            sy += 16.0;
        }
    }

    sy += 4.0;
    if view.ernie_arsenal_count > 0 {
        draw_text(
            ctx,
            &format!("OPP ARSENAL: {}", view.ernie_arsenal_count),
            sx + 5.0, sy, "#785050", 1.0,
        );
    }

    if view.slick_active {
        sy += 12.0;
        draw_text(ctx, "SLICK!", sx + 60.0, sy, "#64b4ff", 2.0);
    }
}

fn section_label(ctx: &CanvasRenderingContext2d, sx: f64, sy: &mut f64, label: &str) {
    ctx.set_fill_style_str("#3c0064");
    ctx.fill_rect(sx, *sy, 190.0, 20.0);
    draw_text(ctx, label, sx + 5.0, *sy + 3.0, "#c8c8c8", 2.0);
    *sy += 24.0;
}

fn stat_row(ctx: &CanvasRenderingContext2d, sx: f64, sy: &mut f64, label: &str, value: &str) {
    draw_text(ctx, label, sx + 5.0, *sy, "#8c8c8c", 2.0);
    draw_text(ctx, value, sx + 110.0, *sy, "#ffffff", 2.0);
    *sy += 18.0;
}

// ─── Unused imports silenced ──────────────────────────────────────────────────
// These are imported for draw_board_with_effects but flagged unused by the
// compiler when certain paths aren't hit; allow dead_code rather than removing.
#[allow(dead_code)]
fn _use_cell_helpers() {
    let _ = bt_color;
    let _ = cell_color_str;
    let _ = char_step;
    let _ = CELL_PX;
    let _ = draw_cell;
    let _ = draw_die_pips;
    let _ = draw_face;
}
