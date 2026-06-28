use battletris_engine::engine::board::{BoardSnapshot, Cell, BOARD_COLS, BOARD_ROWS};
use battletris_engine::engine::game_state::PlayingView;
use battletris_engine::engine::weapons::ActiveWeaponView;

use crate::color::Color;
use crate::context::DrawContext;
use crate::font::{draw_text, text_w};
use crate::layout::{
    BOARD_PX_H, BOARD_PX_W, CELL_PX, OPP_BOARD_X, OPP_BOARD_Y,
    PLAYER_BOARD_X, PLAYER_BOARD_Y, STATS_X, STATS_Y, WINDOW_H, WINDOW_W,
};
use crate::primitives::{
    cell_color, draw_active_piece, draw_cell, draw_die_pips, draw_face,
    draw_ghost_piece, draw_next_piece,
};

pub fn draw_playing<D: DrawContext>(ctx: &mut D, view: &PlayingView) {
    draw_board_with_effects(ctx, view, true);

    if let Some((kind, ref cells)) = view.active_piece {
        if !view.twilight_active {
            draw_ghost_piece(ctx, kind, &view.ghost_cells, PLAYER_BOARD_X, PLAYER_BOARD_Y, view.upbyside_active);
            draw_active_piece(ctx, kind, cells, PLAYER_BOARD_X, PLAYER_BOARD_Y, view.upbyside_active);
        }
    }

    if view.upbyside_active {
        draw_text(ctx, "UPSIDE DOWN", PLAYER_BOARD_X + 20.0, PLAYER_BOARD_Y - 18.0,
            Color::rgb(220, 60, 60), 2.0);
    }

    draw_board_with_effects(ctx, view, false);

    if view.peer_disconnected {
        ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgba(0, 0, 0, 160));
        let t1 = "OPPONENT DISCONNECTED";
        let t2 = "Waiting 15s to reconnect...";
        let cx = WINDOW_W / 2.0;
        let cy = WINDOW_H / 2.0;
        draw_text(ctx, t1, cx - text_w(t1, 3.0) / 2.0, cy - 30.0, Color::rgb(220, 180, 0), 3.0);
        draw_text(ctx, t2, cx - text_w(t2, 2.0) / 2.0, cy + 20.0, Color::rgb(160, 160, 160), 2.0);
    }

    draw_weapon_chips(ctx, &view.player_active_weapons, PLAYER_BOARD_X);
    draw_weapon_chips(ctx, &view.opponent_active_weapons, OPP_BOARD_X);

    draw_stats(ctx, view);
}

fn draw_board_with_effects<D: DrawContext>(ctx: &mut D, view: &PlayingView, is_player: bool) {
    let origin_x = if is_player { PLAYER_BOARD_X } else { OPP_BOARD_X };
    let origin_y = if is_player { PLAYER_BOARD_Y } else { OPP_BOARD_Y };

    let snapshot: Option<&BoardSnapshot> = if is_player {
        Some(&view.player_board)
    } else {
        view.opponent_board.as_ref()
    };

    let Some(snapshot) = snapshot else {
        ctx.fill_rect(origin_x, origin_y, BOARD_PX_W, BOARD_PX_H, Color::PANEL);
        for row in 0..BOARD_ROWS {
            for col in 0..BOARD_COLS {
                let px = origin_x + col as f64 * CELL_PX;
                let py = origin_y + row as f64 * CELL_PX;
                ctx.fill_rect(px + 13.0, py + 13.0, 2.0, 2.0, Color::GRID);
            }
        }
        return;
    };

    ctx.fill_rect(origin_x, origin_y, BOARD_PX_W, BOARD_PX_H, Color::PANEL);

    for row in 0..BOARD_ROWS {
        for col in 0..BOARD_COLS {
            let cell = snapshot.cells[row][col];
            let (px, py) = if is_player && view.upbyside_active {
                (
                    origin_x + (BOARD_COLS - 1 - col) as f64 * CELL_PX,
                    origin_y + (BOARD_ROWS - 1 - row) as f64 * CELL_PX,
                )
            } else {
                (
                    origin_x + col as f64 * CELL_PX,
                    origin_y + row as f64 * CELL_PX,
                )
            };

            if is_player && view.blind_cells.contains(&(row, col)) {
                ctx.fill_rect(px + 13.0, py + 13.0, 2.0, 2.0, Color::GRID);
                continue;
            }

            if is_player && view.twilight_active && !cell.is_empty() {
                // Original game: erase cells to board background (black_gc).
                // Render a grid dot so hidden cells are indistinguishable from empty ones.
                ctx.fill_rect(px + 13.0, py + 13.0, 2.0, 2.0, Color::GRID);
                continue;
            }

            if cell == Cell::Bug {
                ctx.fill_rect(px + 13.0, py + 13.0, 2.0, 2.0, Color::GRID);
                continue;
            }

            if matches!(cell, Cell::Gimp(_)) {
                ctx.draw_gimp_tile(px, py);
                continue;
            }

            if let Some(color) = cell_color(cell) {
                draw_cell(ctx, px, py, color);
                match cell {
                    Cell::Die(pips) => draw_die_pips(ctx, px, py, pips),
                    Cell::Happy => draw_face(ctx, px, py, true),
                    Cell::HappyMissed => draw_face(ctx, px, py, false),
                    _ => {}
                }
            } else {
                ctx.fill_rect(px + 13.0, py + 13.0, 2.0, 2.0, Color::GRID);
            }
        }
    }

}

fn draw_weapon_chips<D: DrawContext>(ctx: &mut D, weapons: &[ActiveWeaponView], board_x: f64) {
    let chip_y = PLAYER_BOARD_Y + BOARD_PX_H + 4.0;
    for (i, wpn) in weapons.iter().enumerate().take(3) {
        let x = board_x + i as f64 * 92.0;
        ctx.fill_rect(x, chip_y, 88.0, 16.0, Color::rgb(60, 0, 100));
        let name = if wpn.name.len() > 10 { &wpn.name[..10] } else { wpn.name };
        draw_text(ctx, name, x + 2.0, chip_y + 2.0, Color::rgb(255, 220, 80), 1.0);
        let rem_str = format!("{}L", wpn.remaining_lines);
        draw_text(ctx, &rem_str, x + 66.0, chip_y + 2.0, Color::rgb(200, 200, 200), 1.0);
    }
}

fn draw_stats<D: DrawContext>(ctx: &mut D, view: &PlayingView) {
    let sv = &view.score;
    let sx = STATS_X;
    let mut sy = STATS_Y;

    ctx.fill_rect(sx, sy, 190.0, BOARD_PX_H, Color::PANEL);
    sy += 10.0;

    draw_text(ctx, "NEXT", sx + 71.0, sy, Color::rgb(180, 180, 180), 2.0);
    sy += 20.0;

    ctx.fill_rect(sx + 30.0, sy, 60.0, 60.0, Color::rgb(20, 20, 20));
    draw_next_piece(ctx, view.next_piece, sx + 35.0, sy + 5.0);
    sy += 75.0;

    let player_label = view.player_name.as_deref().unwrap_or("PLAYER");
    section_label(ctx, sx, &mut sy, player_label);
    stat_row(ctx, sx, &mut sy, "SCORE", &format!("{}", sv.score));
    stat_row(ctx, sx, &mut sy, "LINES", &format!("{}", sv.lines));

    let funds_str = if sv.funds < 0 {
        format!("$-{}", -sv.funds)
    } else {
        format!("${}", sv.funds)
    };
    let funds_color = if sv.funds < 0 { Color::rgb(220, 60, 60) } else { Color::WHITE };
    draw_text(ctx, "FUNDS", sx + 5.0, sy, Color::rgb(140, 140, 140), 2.0);
    draw_text(ctx, &funds_str, sx + 110.0, sy, funds_color, 2.0);
    sy += 18.0;

    sy += 8.0;

    let baz_color = if sv.lines_until_bazaar <= 5 {
        Color::rgb(255, 220, 0)
    } else {
        Color::rgb(180, 180, 180)
    };
    draw_text(ctx, "TIL BAZ", sx + 10.0, sy, Color::rgb(120, 120, 120), 2.0);
    sy += 18.0;
    draw_text(ctx, &format!("{}", sv.lines_until_bazaar), sx + 75.0, sy, baz_color, 3.0);
    sy += 34.0;

    let opp_label = view.opponent_name.as_deref().unwrap_or("OPPONENT");
    section_label(ctx, sx, &mut sy, opp_label);
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
        ctx.fill_rect(sx + 10.0, sy, 170.0, 18.0, Color::rgb(40, 40, 40));
        draw_text(ctx, "--", sx + 80.0, sy + 2.0, Color::rgb(60, 60, 60), 2.0);
        sy += 22.0;
    } else {
        for slot in &view.player_arsenal {
            ctx.fill_rect(sx + 5.0, sy, 180.0, 14.0, Color::rgb(40, 40, 40));
            let qty_suffix = if slot.quantity > 1 {
                format!("({})", slot.quantity)
            } else {
                String::new()
            };
            let label = format!("{}: {} {}", slot.key, slot.name, qty_suffix);
            draw_text(ctx, &label, sx + 7.0, sy + 1.0, Color::rgb(200, 200, 200), 1.0);
            sy += 16.0;
        }
    }

    sy += 4.0;
    if view.opponent_arsenal_count > 0 {
        draw_text(ctx, &format!("OPP ARSENAL: {}", view.opponent_arsenal_count),
            sx + 5.0, sy, Color::rgb(120, 80, 80), 1.0);
    }

    if view.slick_active {
        sy += 12.0;
        draw_text(ctx, "SLICK!", sx + 60.0, sy, Color::rgb(100, 180, 255), 2.0);
    }
}

fn section_label<D: DrawContext>(ctx: &mut D, sx: f64, sy: &mut f64, label: &str) {
    ctx.fill_rect(sx, *sy, 190.0, 20.0, Color::rgb(60, 0, 100));
    draw_text(ctx, label, sx + 5.0, *sy + 3.0, Color::rgb(200, 200, 200), 2.0);
    *sy += 24.0;
}

fn stat_row<D: DrawContext>(ctx: &mut D, sx: f64, sy: &mut f64, label: &str, value: &str) {
    draw_text(ctx, label, sx + 5.0, *sy, Color::rgb(140, 140, 140), 2.0);
    draw_text(ctx, value, sx + 110.0, *sy, Color::WHITE, 2.0);
    *sy += 18.0;
}

pub fn draw_quit_confirm<D: DrawContext>(ctx: &mut D) {
    let box_w = 320.0;
    let box_h = 110.0;
    let bx = (WINDOW_W - box_w) / 2.0;
    let by = (WINDOW_H - box_h) / 2.0;

    ctx.fill_rect(bx - 2.0, by - 2.0, box_w + 4.0, box_h + 4.0, Color::rgb(200, 180, 0));
    ctx.fill_rect(bx, by, box_w, box_h, Color::rgb(15, 15, 15));

    let cx = bx + box_w / 2.0;
    let title = "QUIT GAME?";
    draw_text(ctx, title, cx - text_w(title, 3.0) / 2.0, by + 14.0, Color::rgb(255, 220, 0), 3.0);

    let yes = "Y - QUIT";
    draw_text(ctx, yes, cx - text_w(yes, 2.0) / 2.0, by + 52.0, Color::rgb(220, 80, 80), 2.0);

    let no = "N - CONTINUE";
    draw_text(ctx, no, cx - text_w(no, 2.0) / 2.0, by + 76.0, Color::rgb(80, 220, 80), 2.0);
}
