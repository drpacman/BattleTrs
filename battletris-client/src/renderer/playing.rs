use sdl2::pixels::Color;
use sdl2::rect::Rect;

use battletris_engine::engine::game_state::PlayingView;
use battletris_engine::engine::board::{BOARD_COLS, BOARD_ROWS};

use super::{
    cell_color, draw_active_piece, draw_ghost_piece, draw_next_piece,
    draw_cell, draw_die_pips, draw_face, draw_text, text_w,
    OPP_BOARD_X, OPP_BOARD_Y, PLAYER_BOARD_X, PLAYER_BOARD_Y,
    STATS_X, STATS_Y, Renderer, CELL_PX,
    BOARD_PX_H, BOARD_PX_W, COL_PANEL,
};

pub fn draw_playing(r: &mut Renderer, view: &PlayingView) {
    // ── Player board ──────────────────────────────────────────────────────
    draw_board_with_effects(r, view, true);

    if let Some((kind, ref cells)) = view.active_piece {
        draw_ghost_piece(&mut r.canvas, kind, &view.ghost_cells, PLAYER_BOARD_X, PLAYER_BOARD_Y, view.upbyside_active);
        draw_active_piece(&mut r.canvas, kind, cells, PLAYER_BOARD_X, PLAYER_BOARD_Y, view.upbyside_active);
    }

    // Gimp flash overlay on player board (if active)
    if view.gimp_flash {
        r.canvas.set_draw_color(Color::RGBA(0, 0, 0, 180));
        let _ = r.canvas.fill_rect(Rect::new(PLAYER_BOARD_X, PLAYER_BOARD_Y, BOARD_PX_W, BOARD_PX_H));
        let text = "GIMP!";
        let tw = text_w(text, 5);
        draw_text(&mut r.canvas, text,
            PLAYER_BOARD_X + (BOARD_PX_W as i32 - tw) / 2,
            PLAYER_BOARD_Y + BOARD_PX_H as i32 / 2 - 18,
            Color::RGB(255, 50, 50), 5);
    }

    // Upbyside label on player board
    if view.upbyside_active {
        draw_text(&mut r.canvas, "UPSIDE DOWN", PLAYER_BOARD_X + 20, PLAYER_BOARD_Y - 18,
            Color::RGB(220, 60, 60), 2);
    }

    // ── Opponent board ────────────────────────────────────────────────────
    draw_board_with_effects(r, view, false);

    // ── Peer disconnected overlay ─────────────────────────────────────────
    if view.peer_disconnected {
        r.canvas.set_draw_color(Color::RGBA(0, 0, 0, 160));
        let _ = r.canvas.fill_rect(Rect::new(0, 0, super::WINDOW_W, super::WINDOW_H));
        let t1 = "OPPONENT DISCONNECTED";
        let t2 = "Waiting 15s to reconnect...";
        let cx = super::WINDOW_W as i32 / 2;
        let cy = super::WINDOW_H as i32 / 2;
        draw_text(&mut r.canvas, t1, cx - text_w(t1, 3) / 2, cy - 30,
            Color::RGB(220, 180, 0), 3);
        draw_text(&mut r.canvas, t2, cx - text_w(t2, 2) / 2, cy + 20,
            Color::RGB(160, 160, 160), 2);
    }

    // ── Active weapon chips ───────────────────────────────────────────────
    draw_weapon_chips(r, &view.player_active_weapons, PLAYER_BOARD_X);
    draw_weapon_chips(r, &view.ernie_active_weapons, OPP_BOARD_X);

    // ── Stats panel ───────────────────────────────────────────────────────
    draw_stats(r, view);
}

fn draw_board_with_effects(r: &mut Renderer, view: &PlayingView, is_player: bool) {
    let origin_x = if is_player { PLAYER_BOARD_X } else { OPP_BOARD_X };
    let origin_y = if is_player { PLAYER_BOARD_Y } else { OPP_BOARD_Y };

    let snapshot = if is_player {
        Some(&view.player_board)
    } else {
        view.opponent_board.as_ref()
    };

    let Some(snapshot) = snapshot else {
        // Opponent not connected
        r.canvas.set_draw_color(Color::RGB(15, 15, 15));
        let _ = r.canvas.fill_rect(Rect::new(origin_x, origin_y, BOARD_PX_W, BOARD_PX_H));
        draw_text(&mut r.canvas, "CPU", origin_x + 110, origin_y + 375, Color::RGB(60, 60, 60), 3);
        return;
    };

    // Board background
    r.canvas.set_draw_color(COL_PANEL);
    let _ = r.canvas.fill_rect(Rect::new(origin_x, origin_y, BOARD_PX_W, BOARD_PX_H));

    for row in 0..BOARD_ROWS {
        for col in 0..BOARD_COLS {
            let cell = snapshot.cells[row][col];
            let px = origin_x + col as i32 * CELL_PX as i32;
            let py = if is_player && view.upbyside_active {
                // Upbyside: rows rendered reversed
                origin_y + (BOARD_ROWS - 1 - row) as i32 * CELL_PX as i32
            } else {
                origin_y + row as i32 * CELL_PX as i32
            };

            // Blind cells: render as empty
            if is_player && view.blind_cells.contains(&(row, col)) {
                r.canvas.set_draw_color(Color::RGB(25, 25, 25));
                let _ = r.canvas.fill_rect(Rect::new(px + 13, py + 13, 2, 2));
                continue;
            }

            // Twilight: all cells render as grey
            if is_player && view.twilight_active && !cell.is_empty() {
                r.canvas.set_draw_color(Color::RGB(40, 40, 40));
                let _ = r.canvas.fill_rect(Rect::new(px, py, CELL_PX, CELL_PX));
                continue;
            }

            // Bug cells: render as empty (invisible but solid)
            use battletris_engine::engine::board::Cell;
            if cell == Cell::Bug {
                r.canvas.set_draw_color(Color::RGB(25, 25, 25));
                let _ = r.canvas.fill_rect(Rect::new(px + 13, py + 13, 2, 2));
                continue;
            }

            if let Some(color) = cell_color(cell) {
                draw_cell(&mut r.canvas, px, py, color);
                match cell {
                    Cell::Die(pips) => draw_die_pips(&mut r.canvas, px, py, pips),
                    Cell::Happy => draw_face(&mut r.canvas, px, py, true),
                    Cell::HappyMissed => draw_face(&mut r.canvas, px, py, false),
                    _ => {}
                }
            } else {
                // Grid dot for empty cells
                r.canvas.set_draw_color(Color::RGB(25, 25, 25));
                let _ = r.canvas.fill_rect(Rect::new(px + 13, py + 13, 2, 2));
            }
        }
    }

    // Gimp flash on opponent board (when player fired Gimp at them)
    if !is_player && view.opponent_gimp_flash {
        r.canvas.set_draw_color(Color::RGBA(0, 0, 0, 180));
        let _ = r.canvas.fill_rect(Rect::new(origin_x, origin_y, BOARD_PX_W, BOARD_PX_H));
        let text = "GIMP!";
        let tw = text_w(text, 5);
        draw_text(&mut r.canvas, text,
            origin_x + (BOARD_PX_W as i32 - tw) / 2,
            origin_y + BOARD_PX_H as i32 / 2 - 18,
            Color::RGB(255, 50, 50), 5);
    }
}

fn draw_weapon_chips(r: &mut Renderer, weapons: &[battletris_engine::engine::weapons::ActiveWeaponView], board_x: i32) {
    let chip_y = PLAYER_BOARD_Y + BOARD_PX_H as i32 + 4;
    for (i, wpn) in weapons.iter().enumerate().take(3) {
        let x = board_x + i as i32 * 92;
        // Chip background
        r.canvas.set_draw_color(Color::RGB(60, 0, 100));
        let _ = r.canvas.fill_rect(Rect::new(x, chip_y, 88, 16));
        // Name (truncated to fit 88px at scale=1)
        let name = if wpn.name.len() > 10 { &wpn.name[..10] } else { wpn.name };
        draw_text(&mut r.canvas, name, x + 2, chip_y + 2, Color::RGB(255, 220, 80), 1);
        // Remaining lines
        let rem_str = format!("{}L", wpn.remaining_lines);
        draw_text(&mut r.canvas, &rem_str, x + 66, chip_y + 2, Color::RGB(200, 200, 200), 1);
    }
}

fn draw_stats(r: &mut Renderer, view: &PlayingView) {
    let sv = &view.score;
    let sx = STATS_X;
    let mut sy = STATS_Y;

    // Panel background
    r.canvas.set_draw_color(COL_PANEL);
    let _ = r.canvas.fill_rect(Rect::new(sx, sy, 190, BOARD_PX_H));

    sy += 10;

    draw_text(&mut r.canvas, "NEXT", sx + 71, sy, Color::RGB(180, 180, 180), 2);
    sy += 20;

    r.canvas.set_draw_color(Color::RGB(20, 20, 20));
    let _ = r.canvas.fill_rect(Rect::new(sx + 30, sy, 60, 60));
    draw_next_piece(&mut r.canvas, view.next_piece, sx + 35, sy + 5);
    sy += 75;

    // ── Player stats ──────────────────────────────────────────────────────
    section_label(r, sx, &mut sy, "PLAYER");
    stat_row(r, sx, &mut sy, "SCORE", &format!("{}", sv.score));
    stat_row(r, sx, &mut sy, "LINES", &format!("{}", sv.lines));

    // Funds: red when negative (Reagan Era)
    let funds_str = if sv.funds < 0 {
        format!("$-{}", -sv.funds)
    } else {
        format!("${}", sv.funds)
    };
    let funds_color = if sv.funds < 0 { Color::RGB(220, 60, 60) } else { Color::RGB(255, 255, 255) };
    draw_text(&mut r.canvas, "FUNDS", sx + 5, sy, Color::RGB(140, 140, 140), 2);
    draw_text(&mut r.canvas, &funds_str, sx + 110, sy, funds_color, 2);
    sy += 18;

    sy += 8;

    // ── Bazaar countdown ──────────────────────────────────────────────────
    let baz_color = if sv.lines_until_bazaar <= 5 {
        Color::RGB(255, 220, 0)
    } else {
        Color::RGB(180, 180, 180)
    };
    draw_text(&mut r.canvas, "TIL BAZ", sx + 10, sy, Color::RGB(120, 120, 120), 2);
    sy += 18;
    draw_text(&mut r.canvas, &format!("{}", sv.lines_until_bazaar), sx + 75, sy, baz_color, 3);
    sy += 34;

    // ── Opponent stats ────────────────────────────────────────────────────
    section_label(r, sx, &mut sy, "OPPONENT");
    stat_row(r, sx, &mut sy, "SCORE", &format!("{}", sv.op_score));
    stat_row(r, sx, &mut sy, "LINES", &format!("{}", sv.op_lines));
    if sv.show_op_funds {
        let op_str = if sv.op_funds < 0 { format!("$-{}", -sv.op_funds) } else { format!("${}", sv.op_funds) };
        stat_row(r, sx, &mut sy, "FUNDS", &op_str);
    } else {
        stat_row(r, sx, &mut sy, "FUNDS", "???");
    }

    sy += 8;

    // ── Arsenal ───────────────────────────────────────────────────────────
    section_label(r, sx, &mut sy, "ARSENAL");
    if view.player_arsenal.is_empty() {
        r.canvas.set_draw_color(Color::RGB(40, 40, 40));
        let _ = r.canvas.fill_rect(Rect::new(sx + 10, sy, 170, 18));
        draw_text(&mut r.canvas, "--", sx + 80, sy + 2, Color::RGB(60, 60, 60), 2);
        sy += 22;
    } else {
        for slot in &view.player_arsenal {
            r.canvas.set_draw_color(Color::RGB(40, 40, 40));
            let _ = r.canvas.fill_rect(Rect::new(sx + 5, sy, 180, 14));
            let qty_suffix = if slot.quantity > 1 { format!("({})", slot.quantity) } else { String::new() };
            let label = format!("{}: {} {}", slot.key, slot.name, qty_suffix);
            draw_text(&mut r.canvas, &label, sx + 7, sy + 1, Color::RGB(200, 200, 200), 1);
            sy += 16;
        }
    }

    // Ernie arsenal count (if watching opponent)
    sy += 4;
    if view.ernie_arsenal_count > 0 {
        draw_text(&mut r.canvas, &format!("CPU ARSENAL: {}", view.ernie_arsenal_count),
            sx + 5, sy, Color::RGB(120, 80, 80), 1);
    }

    // Slick indicator
    if view.slick_active {
        sy += 12;
        draw_text(&mut r.canvas, "SLICK!", sx + 60, sy, Color::RGB(100, 180, 255), 2);
    }
}

fn section_label(r: &mut Renderer, sx: i32, sy: &mut i32, label: &str) {
    r.canvas.set_draw_color(Color::RGB(60, 0, 100));
    let _ = r.canvas.fill_rect(Rect::new(sx, *sy, 190, 20));
    draw_text(&mut r.canvas, label, sx + 5, *sy + 3, Color::RGB(200, 200, 200), 2);
    *sy += 24;
}

fn stat_row(r: &mut Renderer, sx: i32, sy: &mut i32, label: &str, value: &str) {
    draw_text(&mut r.canvas, label, sx + 5, *sy, Color::RGB(140, 140, 140), 2);
    draw_text(&mut r.canvas, value, sx + 110, *sy, Color::RGB(255, 255, 255), 2);
    *sy += 18;
}
