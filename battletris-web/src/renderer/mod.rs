pub mod overlay;
pub mod playing;
pub mod screens;

use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

use battletris_engine::engine::board::{BoardSnapshot, Cell, BOARD_COLS, BOARD_ROWS};
use battletris_engine::engine::piece::PieceKind;

// ─── Layout constants (identical to SDL2 client) ─────────────────────────────

pub const CELL_PX: f64 = 28.0;
pub const WINDOW_W: f64 = 820.0;
pub const WINDOW_H: f64 = 860.0;

pub const PLAYER_BOARD_X: f64 = 20.0;
pub const PLAYER_BOARD_Y: f64 = 40.0;
pub const STATS_X: f64 = 310.0;
pub const STATS_Y: f64 = 40.0;
pub const OPP_BOARD_X: f64 = 520.0;
pub const OPP_BOARD_Y: f64 = 40.0;

pub const BOARD_PX_W: f64 = CELL_PX * BOARD_COLS as f64;  // 280
pub const BOARD_PX_H: f64 = CELL_PX * BOARD_ROWS as f64;  // 784

// ─── 5×7 bitmap font (ASCII 32–96, same data as SDL2 client) ─────────────────

const GLYPH_W: f64 = 5.0;

#[rustfmt::skip]
const FONT: &[[u8; 7]] = &[
    [0,  0,  0,  0,  0,  0,  0 ], // 32 ' '
    [4,  4,  4,  4,  0,  4,  0 ], // 33 '!'
    [10, 10, 0,  0,  0,  0,  0 ], // 34 '"'
    [10, 31, 10, 31, 10, 0,  0 ], // 35 '#'
    [4,  14, 20, 14, 5,  14, 4 ], // 36 '$'
    [17, 9,  2,  4,  18, 9,  17], // 37 '%'
    [12, 18, 12, 10, 17, 18, 13], // 38 '&'
    [12, 4,  8,  0,  0,  0,  0 ], // 39 '\''
    [2,  4,  8,  8,  8,  4,  2 ], // 40 '('
    [8,  4,  2,  2,  2,  4,  8 ], // 41 ')'
    [0,  10, 4,  31, 4,  10, 0 ], // 42 '*'
    [0,  4,  4,  31, 4,  4,  0 ], // 43 '+'
    [0,  0,  0,  0,  6,  4,  8 ], // 44 ','
    [0,  0,  0,  31, 0,  0,  0 ], // 45 '-'
    [0,  0,  0,  0,  0,  12, 12], // 46 '.'
    [1,  1,  2,  4,  8,  16, 16], // 47 '/'
    [14, 17, 17, 17, 17, 17, 14], // 48 '0'
    [4,  12, 4,  4,  4,  4,  14], // 49 '1'
    [14, 17, 1,  6,  8,  16, 31], // 50 '2'
    [14, 17, 1,  6,  1,  17, 14], // 51 '3'
    [2,  6,  10, 18, 31, 2,  2 ], // 52 '4'
    [31, 16, 16, 30, 1,  17, 14], // 53 '5'
    [14, 16, 16, 30, 17, 17, 14], // 54 '6'
    [31, 1,  2,  4,  8,  8,  8 ], // 55 '7'
    [14, 17, 17, 14, 17, 17, 14], // 56 '8'
    [14, 17, 17, 15, 1,  17, 14], // 57 '9'
    [0,  12, 12, 0,  12, 12, 0 ], // 58 ':'
    [0,  12, 12, 0,  12, 4,  8 ], // 59 ';'
    [2,  4,  8,  16, 8,  4,  2 ], // 60 '<'
    [0,  31, 0,  0,  31, 0,  0 ], // 61 '='
    [8,  4,  2,  1,  2,  4,  8 ], // 62 '>'
    [14, 17, 1,  6,  4,  0,  4 ], // 63 '?'
    [14, 17, 1,  13, 21, 21, 14], // 64 '@'
    [4,  10, 17, 31, 17, 17, 17], // 65 'A'
    [30, 17, 17, 30, 17, 17, 30], // 66 'B'
    [14, 17, 16, 16, 16, 17, 14], // 67 'C'
    [30, 17, 17, 17, 17, 17, 30], // 68 'D'
    [31, 16, 16, 30, 16, 16, 31], // 69 'E'
    [31, 16, 16, 30, 16, 16, 16], // 70 'F'
    [14, 17, 16, 23, 17, 17, 14], // 71 'G'
    [17, 17, 17, 31, 17, 17, 17], // 72 'H'
    [14, 4,  4,  4,  4,  4,  14], // 73 'I'
    [7,  2,  2,  2,  18, 18, 12], // 74 'J'
    [17, 18, 20, 24, 20, 18, 17], // 75 'K'
    [16, 16, 16, 16, 16, 16, 31], // 76 'L'
    [17, 27, 21, 17, 17, 17, 17], // 77 'M'
    [17, 25, 21, 19, 17, 17, 17], // 78 'N'
    [14, 17, 17, 17, 17, 17, 14], // 79 'O'
    [30, 17, 17, 30, 16, 16, 16], // 80 'P'
    [14, 17, 17, 17, 21, 19, 15], // 81 'Q'
    [30, 17, 17, 30, 20, 18, 17], // 82 'R'
    [14, 17, 16, 14, 1,  17, 14], // 83 'S'
    [31, 4,  4,  4,  4,  4,  4 ], // 84 'T'
    [17, 17, 17, 17, 17, 17, 14], // 85 'U'
    [17, 17, 17, 17, 10, 10, 4 ], // 86 'V'
    [17, 17, 17, 21, 21, 27, 17], // 87 'W'
    [17, 17, 10, 4,  10, 17, 17], // 88 'X'
    [17, 17, 10, 4,  4,  4,  4 ], // 89 'Y'
    [31, 1,  2,  4,  8,  16, 31], // 90 'Z'
    [14, 8,  8,  8,  8,  8,  14], // 91 '['
    [16, 8,  8,  4,  2,  1,  1 ], // 92 '\\'
    [14, 2,  2,  2,  2,  2,  14], // 93 ']'
    [4,  10, 17, 0,  0,  0,  0 ], // 94 '^'
    [0,  0,  0,  0,  0,  0,  31], // 95 '_'
    [8,  4,  0,  0,  0,  0,  0 ], // 96 '`'
];

// ─── Canvas renderer ──────────────────────────────────────────────────────────

pub struct CanvasRenderer {
    pub ctx: CanvasRenderingContext2d,
}

impl CanvasRenderer {
    pub fn new() -> Result<Self, wasm_bindgen::JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id("game-canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        Ok(Self { ctx })
    }

    pub fn clear(&self) {
        self.ctx.set_fill_style_str("#000000");
        self.ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H);
    }
}

// ─── Color helpers ────────────────────────────────────────────────────────────

pub fn bt_color(index: u8) -> &'static str {
    match index {
        1 => "#ffffe6", // BT_IVORY
        2 => "#ffdc00", // BT_YELLOW
        3 => "#dc3232", // BT_RED
        4 => "#3264dc", // BT_BLUE
        5 => "#ff8c00", // BT_ORANGE
        6 => "#32c832", // BT_GREEN
        7 => "#00c8dc", // BT_CYAN
        8 => "#a032c8", // BT_PURPLE
        _ => "#808080",
    }
}

fn darken_hex(hex: &str) -> String {
    if hex.len() != 7 || !hex.starts_with('#') {
        return hex.to_string();
    }
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0).saturating_sub(40);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0).saturating_sub(40);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0).saturating_sub(40);
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub fn cell_color_str(cell: Cell) -> Option<&'static str> {
    match cell {
        Cell::Empty => None,
        Cell::Regular(c) => Some(bt_color(c)),
        Cell::Die(_) => Some("#ffffff"),
        Cell::Happy => Some("#ffffff"),
        Cell::HappyMissed => Some("#b4b4b4"),
        Cell::Struct_ => Some("#646464"),
        Cell::Bug => None,
        Cell::Twilight => Some("#282828"),
    }
}

// ─── Shared drawing primitives ────────────────────────────────────────────────

pub fn draw_cell(ctx: &CanvasRenderingContext2d, px: f64, py: f64, color: &str) {
    let dark = darken_hex(color);
    ctx.set_fill_style_str(&dark);
    ctx.fill_rect(px, py, CELL_PX, CELL_PX);
    ctx.set_fill_style_str(color);
    ctx.fill_rect(px + 1.0, py + 1.0, CELL_PX - 2.0, CELL_PX - 2.0);
}

pub fn draw_die_pips(ctx: &CanvasRenderingContext2d, px: f64, py: f64, pips: u8) {
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
    ctx.set_fill_style_str("#141414");
    for &(dx, dy) in positions {
        ctx.fill_rect(px + dx, py + dy, 4.0, 4.0);
    }
}

pub fn draw_face(ctx: &CanvasRenderingContext2d, px: f64, py: f64, happy: bool) {
    ctx.set_fill_style_str("#0000c8");
    ctx.fill_rect(px + 7.0, py + 8.0, 3.0, 3.0);
    ctx.fill_rect(px + 18.0, py + 8.0, 3.0, 3.0);
    ctx.set_fill_style_str("#b40000");
    if happy {
        ctx.fill_rect(px + 7.0,  py + 18.0, 3.0, 2.0);
        ctx.fill_rect(px + 13.0, py + 20.0, 3.0, 2.0);
        ctx.fill_rect(px + 19.0, py + 18.0, 3.0, 2.0);
    } else {
        ctx.fill_rect(px + 7.0,  py + 20.0, 3.0, 2.0);
        ctx.fill_rect(px + 13.0, py + 18.0, 3.0, 2.0);
        ctx.fill_rect(px + 19.0, py + 20.0, 3.0, 2.0);
    }
}

pub fn draw_board(
    ctx: &CanvasRenderingContext2d,
    snapshot: &BoardSnapshot,
    origin_x: f64,
    origin_y: f64,
    upbyside: bool,
    blind_cells: &[(usize, usize)],
    twilight: bool,
) {
    ctx.set_fill_style_str("#1e1e1e");
    ctx.fill_rect(origin_x, origin_y, BOARD_PX_W, BOARD_PX_H);

    for row in 0..BOARD_ROWS {
        for col in 0..BOARD_COLS {
            let cell = snapshot.cells[row][col];
            let px = origin_x + col as f64 * CELL_PX;
            let py = if upbyside {
                origin_y + (BOARD_ROWS - 1 - row) as f64 * CELL_PX
            } else {
                origin_y + row as f64 * CELL_PX
            };

            if blind_cells.contains(&(row, col)) {
                ctx.set_fill_style_str("#191919");
                ctx.fill_rect(px + 13.0, py + 13.0, 2.0, 2.0);
                continue;
            }

            if twilight && !cell.is_empty() {
                ctx.set_fill_style_str("#282828");
                ctx.fill_rect(px, py, CELL_PX, CELL_PX);
                continue;
            }

            if cell == Cell::Bug {
                ctx.set_fill_style_str("#191919");
                ctx.fill_rect(px + 13.0, py + 13.0, 2.0, 2.0);
                continue;
            }

            if let Some(color) = cell_color_str(cell) {
                draw_cell(ctx, px, py, color);
                match cell {
                    Cell::Die(pips) => draw_die_pips(ctx, px, py, pips),
                    Cell::Happy => draw_face(ctx, px, py, true),
                    Cell::HappyMissed => draw_face(ctx, px, py, false),
                    _ => {}
                }
            } else {
                ctx.set_fill_style_str("#191919");
                ctx.fill_rect(px + 13.0, py + 13.0, 2.0, 2.0);
            }
        }
    }
}

pub fn draw_active_piece(
    ctx: &CanvasRenderingContext2d,
    kind: PieceKind,
    cells: &[(i32, i32)],
    origin_x: f64,
    origin_y: f64,
    flip: bool,
) {
    let color = match kind {
        PieceKind::Die { .. } | PieceKind::Happy => "#ffffff",
        _ => bt_color(kind.color()),
    };
    for &(col, row) in cells {
        if row < 0 {
            continue;
        }
        let px = origin_x + col as f64 * CELL_PX;
        let py = if flip {
            origin_y + (BOARD_ROWS as i32 - 1 - row) as f64 * CELL_PX
        } else {
            origin_y + row as f64 * CELL_PX
        };
        draw_cell(ctx, px, py, color);
        match kind {
            PieceKind::Die { pips } => draw_die_pips(ctx, px, py, pips),
            PieceKind::Happy => draw_face(ctx, px, py, true),
            _ => {}
        }
    }
}

pub fn draw_ghost_piece(
    ctx: &CanvasRenderingContext2d,
    kind: PieceKind,
    cells: &[(i32, i32)],
    origin_x: f64,
    origin_y: f64,
    flip: bool,
) {
    let base = match kind {
        PieceKind::Die { .. } | PieceKind::Happy => "#ffffff",
        _ => bt_color(kind.color()),
    };
    // Ghost: dark tinted (divide each component by 4)
    let ghost_color = darken_hex_quarter(base);
    ctx.set_stroke_style_str(&ghost_color);
    ctx.set_line_width(1.0);
    for &(col, row) in cells {
        if row < 0 {
            continue;
        }
        let px = origin_x + col as f64 * CELL_PX;
        let py = if flip {
            origin_y + (BOARD_ROWS as i32 - 1 - row) as f64 * CELL_PX
        } else {
            origin_y + row as f64 * CELL_PX
        };
        ctx.stroke_rect(px + 1.0, py + 1.0, CELL_PX - 2.0, CELL_PX - 2.0);
    }
}

pub fn draw_next_piece(
    ctx: &CanvasRenderingContext2d,
    kind: PieceKind,
    origin_x: f64,
    origin_y: f64,
) {
    const PREVIEW_CELL: f64 = 14.0;
    let color = match kind {
        PieceKind::Die { .. } | PieceKind::Happy => "#ffffff",
        _ => bt_color(kind.color()),
    };
    ctx.set_fill_style_str(color);
    let cells = kind.cells(0);
    for &(col, row) in cells {
        let px = origin_x + col as f64 * PREVIEW_CELL;
        let py = origin_y + row as f64 * PREVIEW_CELL;
        ctx.fill_rect(px, py, PREVIEW_CELL - 1.0, PREVIEW_CELL - 1.0);
    }
}

fn darken_hex_quarter(hex: &str) -> String {
    if hex.len() != 7 || !hex.starts_with('#') {
        return hex.to_string();
    }
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0) / 4;
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0) / 4;
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0) / 4;
    format!("#{r:02x}{g:02x}{b:02x}")
}

// ─── Text rendering ───────────────────────────────────────────────────────────

pub fn char_step(scale: f64) -> f64 {
    (GLYPH_W + 1.0) * scale
}

pub fn text_w(text: &str, scale: f64) -> f64 {
    text.chars().count() as f64 * char_step(scale)
}

pub fn draw_text(ctx: &CanvasRenderingContext2d, text: &str, x: f64, y: f64, color: &str, scale: f64) {
    ctx.set_fill_style_str(color);
    let step = char_step(scale);
    for (ci, ch) in text.chars().enumerate() {
        let c = ch.to_ascii_uppercase() as usize;
        if c < 32 || c > 96 {
            continue;
        }
        let glyph = &FONT[c - 32];
        let gx = x + ci as f64 * step;
        for (row, &bits) in glyph.iter().enumerate() {
            for col in 0..5_usize {
                let bit_pos = 4 - col;
                if bits & (1u8 << bit_pos) != 0 {
                    ctx.fill_rect(
                        gx + col as f64 * scale,
                        y + row as f64 * scale,
                        scale,
                        scale,
                    );
                }
            }
        }
    }
}
