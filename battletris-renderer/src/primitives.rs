use battletris_engine::engine::board::{Cell, BOARD_COLS, BOARD_ROWS};
use battletris_engine::engine::piece::PieceKind;

use crate::color::Color;
use crate::context::DrawContext;
use crate::layout::CELL_PX;

pub fn bt_color(index: u8) -> Color {
    match index {
        1 => Color::rgb(255, 255, 230), // BT_IVORY
        2 => Color::rgb(255, 220,   0), // BT_YELLOW
        3 => Color::rgb(220,  50,  50), // BT_RED
        4 => Color::rgb( 50, 100, 220), // BT_BLUE
        5 => Color::rgb(255, 140,   0), // BT_ORANGE
        6 => Color::rgb( 50, 200,  50), // BT_GREEN
        7 => Color::rgb(  0, 200, 220), // BT_CYAN
        8 => Color::rgb(160,  50, 200), // BT_PURPLE
        _ => Color::rgb(128, 128, 128),
    }
}

pub(crate) fn piece_color(kind: PieceKind) -> Color {
    match kind {
        PieceKind::Die { .. } | PieceKind::Happy => Color::WHITE,
        _ => bt_color(kind.color()),
    }
}

pub(crate) fn cell_pos(col: i32, row: i32, origin_x: f64, origin_y: f64, flip: bool) -> (f64, f64) {
    if flip {
        (
            origin_x + (BOARD_COLS as i32 - 1 - col) as f64 * CELL_PX,
            origin_y + (BOARD_ROWS as i32 - 1 - row) as f64 * CELL_PX,
        )
    } else {
        (
            origin_x + col as f64 * CELL_PX,
            origin_y + row as f64 * CELL_PX,
        )
    }
}

pub fn cell_color(cell: Cell) -> Option<Color> {
    match cell {
        Cell::Empty => None,
        Cell::Regular(c) => Some(bt_color(c)),
        Cell::Die(_) => Some(Color::WHITE),
        Cell::Happy => Some(Color::WHITE),
        Cell::HappyMissed => Some(Color::rgb(180, 180, 180)),
        Cell::Struct_ => Some(Color::rgb(100, 100, 100)),
        Cell::Bug => None,
        Cell::Twilight => None,
        Cell::Gimp(_) => Some(Color::rgb(120, 120, 120)),
    }
}

pub fn draw_cell<D: DrawContext>(ctx: &mut D, px: f64, py: f64, color: Color) {
    ctx.fill_rect(px, py, CELL_PX, CELL_PX, color.darken());
    ctx.fill_rect(px + 1.0, py + 1.0, CELL_PX - 2.0, CELL_PX - 2.0, color);
}

pub fn draw_die_pips<D: DrawContext>(ctx: &mut D, px: f64, py: f64, pips: u8) {
    #[rustfmt::skip]
    let positions: &[(f64, f64)] = match pips {
        1 => &[(12.0, 12.0)],
        2 => &[(5.0, 5.0), (19.0, 19.0)],
        3 => &[(5.0, 5.0), (12.0, 12.0), (19.0, 19.0)],
        4 => &[(5.0, 5.0), (19.0, 5.0), (5.0, 19.0), (19.0, 19.0)],
        5 => &[(5.0, 5.0), (19.0, 5.0), (12.0, 12.0), (5.0, 19.0), (19.0, 19.0)],
        6 => &[(5.0, 5.0), (19.0, 5.0), (5.0, 12.0), (19.0, 12.0), (5.0, 19.0), (19.0, 19.0)],
        _ => &[],
    };
    let pip_color = Color::rgb(20, 20, 20);
    for &(dx, dy) in positions {
        ctx.fill_rect(px + dx, py + dy, 4.0, 4.0, pip_color);
    }
}

pub fn draw_face<D: DrawContext>(ctx: &mut D, px: f64, py: f64, happy: bool) {
    let eye_color = Color::rgb(0, 0, 200);
    ctx.fill_rect(px + 7.0,  py + 8.0, 3.0, 3.0, eye_color);
    ctx.fill_rect(px + 18.0, py + 8.0, 3.0, 3.0, eye_color);
    let mouth_color = Color::rgb(180, 0, 0);
    if happy {
        ctx.fill_rect(px + 7.0,  py + 18.0, 3.0, 2.0, mouth_color);
        ctx.fill_rect(px + 13.0, py + 20.0, 3.0, 2.0, mouth_color);
        ctx.fill_rect(px + 19.0, py + 18.0, 3.0, 2.0, mouth_color);
    } else {
        ctx.fill_rect(px + 7.0,  py + 20.0, 3.0, 2.0, mouth_color);
        ctx.fill_rect(px + 13.0, py + 18.0, 3.0, 2.0, mouth_color);
        ctx.fill_rect(px + 19.0, py + 20.0, 3.0, 2.0, mouth_color);
    }
}

pub fn draw_active_piece<D: DrawContext>(
    ctx: &mut D,
    kind: PieceKind,
    cells: &[(i32, i32)],
    origin_x: f64,
    origin_y: f64,
    flip: bool,
) {
    let color = piece_color(kind);
    for &(col, row) in cells {
        if row < 0 || row >= BOARD_ROWS as i32 { continue; }
        let (px, py) = cell_pos(col, row, origin_x, origin_y, flip);
        draw_cell(ctx, px, py, color);
        match kind {
            PieceKind::Die { pips } => draw_die_pips(ctx, px, py, pips),
            PieceKind::Happy => draw_face(ctx, px, py, true),
            _ => {}
        }
    }
}

pub fn draw_ghost_piece<D: DrawContext>(
    ctx: &mut D,
    kind: PieceKind,
    cells: &[(i32, i32)],
    origin_x: f64,
    origin_y: f64,
    flip: bool,
) {
    let ghost_color = piece_color(kind).quarter();
    for &(col, row) in cells {
        if row < 0 || row >= BOARD_ROWS as i32 { continue; }
        let (px, py) = cell_pos(col, row, origin_x, origin_y, flip);
        ctx.stroke_rect(px + 1.0, py + 1.0, CELL_PX - 2.0, CELL_PX - 2.0, ghost_color);
    }
}

pub fn draw_next_piece<D: DrawContext>(ctx: &mut D, kind: PieceKind, origin_x: f64, origin_y: f64) {
    const PREVIEW_CELL: f64 = 14.0;
    let color = piece_color(kind);
    let cells = kind.cells(0);
    for &(col, row) in cells {
        let px = origin_x + col as f64 * PREVIEW_CELL;
        let py = origin_y + row as f64 * PREVIEW_CELL;
        ctx.fill_rect(px, py, PREVIEW_CELL - 1.0, PREVIEW_CELL - 1.0, color);
    }
}
