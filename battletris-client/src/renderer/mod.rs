use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use battletris_engine::engine::{BoardSnapshot, Cell, PieceKind, BOARD_COLS, BOARD_ROWS};

pub mod bazaar;
pub mod font;
pub mod game_over;
pub mod lobby;
pub mod playing;
pub mod title;

pub use font::{draw_text, text_w};

// ─── Layout constants ────────────────────────────────────────────────────────

pub const CELL_PX: u32 = 28;
pub const WINDOW_W: u32 = 820;
pub const WINDOW_H: u32 = 860;

pub const PLAYER_BOARD_X: i32 = 20;
pub const PLAYER_BOARD_Y: i32 = 40;
pub const STATS_X: i32 = 310;
pub const STATS_Y: i32 = 40;
pub const OPP_BOARD_X: i32 = 520;
pub const OPP_BOARD_Y: i32 = 40;

pub const BOARD_PX_W: u32 = CELL_PX * BOARD_COLS as u32;   // 280
pub const BOARD_PX_H: u32 = CELL_PX * BOARD_ROWS as u32;   // 784

// ─── Color palette (matches BT_* constants) ──────────────────────────────────

pub const COL_BLACK: Color = Color::RGB(0, 0, 0);
pub const COL_WHITE: Color = Color::RGB(255, 255, 255);
pub const COL_PANEL: Color = Color::RGB(30, 30, 30);
#[allow(dead_code)]
pub const COL_GRID:  Color = Color::RGB(25, 25, 25);

/// BT color index → SDL2 Color. Index 0 = unused placeholder.
pub fn bt_color(index: u8) -> Color {
    match index {
        1 => Color::RGB(255, 255, 230), // BT_IVORY
        2 => Color::RGB(255, 220,   0), // BT_YELLOW
        3 => Color::RGB(220,  50,  50), // BT_RED
        4 => Color::RGB( 50, 100, 220), // BT_BLUE
        5 => Color::RGB(255, 140,   0), // BT_ORANGE
        6 => Color::RGB( 50, 200,  50), // BT_GREEN
        7 => Color::RGB(  0, 200, 220), // BT_CYAN
        8 => Color::RGB(160,  50, 200), // BT_PURPLE
        _ => Color::RGB(128, 128, 128),
    }
}

pub fn cell_color(cell: Cell) -> Option<Color> {
    match cell {
        Cell::Empty => None,
        Cell::Regular(c) => Some(bt_color(c)),
        Cell::Die(_) => Some(COL_WHITE),
        Cell::Happy => Some(COL_WHITE),
        Cell::HappyMissed => Some(Color::RGB(180, 180, 180)),
        Cell::Struct_ => Some(Color::RGB(100, 100, 100)),
        Cell::Bug => None,
        Cell::Twilight => Some(Color::RGB(40, 40, 40)),
    }
}

// ─── Renderer ────────────────────────────────────────────────────────────────

pub struct Renderer {
    pub canvas: Canvas<Window>,
}

impl Renderer {
    pub fn new(video: sdl2::VideoSubsystem) -> Result<Self, Box<dyn std::error::Error>> {
        let window = video
            .window("BattleTris", WINDOW_W, WINDOW_H)
            .position_centered()
            .build()?;
        let canvas = window.into_canvas().accelerated().present_vsync().build()?;
        Ok(Renderer { canvas })
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(COL_BLACK);
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}

// ─── Shared drawing helpers ───────────────────────────────────────────────────

/// Draw a filled cell with a 1px darker border.
pub fn draw_cell(canvas: &mut Canvas<Window>, x: i32, y: i32, color: Color) {
    // Border
    let dark = Color::RGB(
        color.r.saturating_sub(40),
        color.g.saturating_sub(40),
        color.b.saturating_sub(40),
    );
    canvas.set_draw_color(dark);
    let _ = canvas.fill_rect(Rect::new(x, y, CELL_PX, CELL_PX));
    // Inner fill (1px inset)
    canvas.set_draw_color(color);
    let _ = canvas.fill_rect(Rect::new(x + 1, y + 1, CELL_PX - 2, CELL_PX - 2));
}

/// Draw a pip dot for Die cells. `px`,`py` = top-left of cell.
pub fn draw_die_pips(canvas: &mut Canvas<Window>, px: i32, py: i32, pips: u8) {
    // Pip positions within a 28×28 cell (3×3 dot at each of 7 slots)
    #[rustfmt::skip]
    let positions: &[(i32,i32)] = match pips {
        1 => &[(12,12)],
        2 => &[(5,5),(19,19)],
        3 => &[(5,5),(12,12),(19,19)],
        4 => &[(5,5),(19,5),(5,19),(19,19)],
        5 => &[(5,5),(19,5),(12,12),(5,19),(19,19)],
        6 => &[(5,5),(19,5),(5,12),(19,12),(5,19),(19,19)],
        _ => &[],
    };
    canvas.set_draw_color(Color::RGB(20, 20, 20));
    for &(dx, dy) in positions {
        let _ = canvas.fill_rect(Rect::new(px + dx, py + dy, 4, 4));
    }
}

/// Draw the smile/frown symbol for Happy/HappyMissed.
pub fn draw_face(canvas: &mut Canvas<Window>, px: i32, py: i32, happy: bool) {
    let eye_color = Color::RGB(0, 0, 200);
    // Eyes
    canvas.set_draw_color(eye_color);
    let _ = canvas.fill_rect(Rect::new(px + 7,  py + 8,  3, 3));
    let _ = canvas.fill_rect(Rect::new(px + 18, py + 8,  3, 3));
    // Mouth
    canvas.set_draw_color(Color::RGB(180, 0, 0));
    if happy {
        // Smile: bottom arc approximated as 3 dots
        let _ = canvas.fill_rect(Rect::new(px + 7,  py + 18, 3, 2));
        let _ = canvas.fill_rect(Rect::new(px + 13, py + 20, 3, 2));
        let _ = canvas.fill_rect(Rect::new(px + 19, py + 18, 3, 2));
    } else {
        // Frown: inverted arc
        let _ = canvas.fill_rect(Rect::new(px + 7,  py + 20, 3, 2));
        let _ = canvas.fill_rect(Rect::new(px + 13, py + 18, 3, 2));
        let _ = canvas.fill_rect(Rect::new(px + 19, py + 20, 3, 2));
    }
}

#[allow(dead_code)]
pub fn draw_board(
    canvas: &mut Canvas<Window>,
    snapshot: &BoardSnapshot,
    origin_x: i32,
    origin_y: i32,
) {
    // Board background
    canvas.set_draw_color(COL_PANEL);
    let _ = canvas.fill_rect(Rect::new(origin_x, origin_y, BOARD_PX_W, BOARD_PX_H));

    for row in 0..BOARD_ROWS {
        for col in 0..BOARD_COLS {
            let cell = snapshot.cells[row][col];
            let px = origin_x + col as i32 * CELL_PX as i32;
            let py = origin_y + row as i32 * CELL_PX as i32;

            if let Some(color) = cell_color(cell) {
                draw_cell(canvas, px, py, color);
                // Special overlays
                match cell {
                    Cell::Die(pips) => draw_die_pips(canvas, px, py, pips),
                    Cell::Happy => draw_face(canvas, px, py, true),
                    Cell::HappyMissed => draw_face(canvas, px, py, false),
                    _ => {}
                }
            } else {
                // Grid dot to show empty cells
                canvas.set_draw_color(COL_GRID);
                let _ = canvas.fill_rect(Rect::new(px + 13, py + 13, 2, 2));
            }
        }
    }
}

/// Overlay active piece cells onto the already-drawn board.
/// When `flip` is true (Upbyside active), rows are rendered in reverse so the piece
/// visually matches the flipped board.
pub fn draw_active_piece(
    canvas: &mut Canvas<Window>,
    kind: PieceKind,
    cells: &[(i32, i32)],
    origin_x: i32,
    origin_y: i32,
    flip: bool,
) {
    let color = match kind {
        PieceKind::Die { .. } => COL_WHITE,
        PieceKind::Happy => COL_WHITE,
        _ => bt_color(kind.color()),
    };
    for &(col, row) in cells {
        if row < 0 { continue; }
        let px = origin_x + col * CELL_PX as i32;
        let py = if flip {
            origin_y + (BOARD_ROWS as i32 - 1 - row) * CELL_PX as i32
        } else {
            origin_y + row * CELL_PX as i32
        };
        draw_cell(canvas, px, py, color);
        match kind {
            PieceKind::Die { pips } => draw_die_pips(canvas, px, py, pips),
            PieceKind::Happy => draw_face(canvas, px, py, true),
            _ => {}
        }
    }
}

/// Draw ghost piece (translucent outline) at given cells.
/// When `flip` is true (Upbyside active), rows are rendered in reverse.
pub fn draw_ghost_piece(
    canvas: &mut Canvas<Window>,
    kind: PieceKind,
    cells: &[(i32, i32)],
    origin_x: i32,
    origin_y: i32,
    flip: bool,
) {
    let base = match kind {
        PieceKind::Die { .. } | PieceKind::Happy => COL_WHITE,
        _ => bt_color(kind.color()),
    };
    // Ghost = dark tinted version of the piece color
    let ghost_color = Color::RGB(
        base.r / 4,
        base.g / 4,
        base.b / 4,
    );
    canvas.set_draw_color(ghost_color);
    for &(col, row) in cells {
        if row < 0 { continue; }
        let px = origin_x + col * CELL_PX as i32;
        let py = if flip {
            origin_y + (BOARD_ROWS as i32 - 1 - row) * CELL_PX as i32
        } else {
            origin_y + row * CELL_PX as i32
        };
        // Outline only (no fill)
        let _ = canvas.draw_rect(Rect::new(px + 1, py + 1, CELL_PX - 2, CELL_PX - 2));
    }
}

/// Modal overlay asking the player to confirm quitting.
pub fn draw_quit_confirm(r: &mut Renderer) {
    let box_w: u32 = 320;
    let box_h: u32 = 110;
    let bx = (WINDOW_W - box_w) as i32 / 2;
    let by = (WINDOW_H - box_h) as i32 / 2;

    // Outer border
    r.canvas.set_draw_color(Color::RGB(200, 180, 0));
    let _ = r.canvas.fill_rect(Rect::new(bx - 2, by - 2, box_w + 4, box_h + 4));

    // Dark background
    r.canvas.set_draw_color(Color::RGB(15, 15, 15));
    let _ = r.canvas.fill_rect(Rect::new(bx, by, box_w, box_h));

    let cx = bx + box_w as i32 / 2;

    let title = "QUIT GAME?";
    draw_text(&mut r.canvas, title, cx - text_w(title, 3) / 2, by + 14, Color::RGB(255, 220, 0), 3);

    let yes = "Y - QUIT";
    draw_text(&mut r.canvas, yes, cx - text_w(yes, 2) / 2, by + 52, Color::RGB(220, 80, 80), 2);

    let no = "N - CONTINUE";
    draw_text(&mut r.canvas, no, cx - text_w(no, 2) / 2, by + 76, Color::RGB(80, 220, 80), 2);
}

/// Draw the next-piece preview in the stats panel.
pub fn draw_next_piece(
    canvas: &mut Canvas<Window>,
    kind: PieceKind,
    origin_x: i32,
    origin_y: i32,
) {
    const PREVIEW_CELL: u32 = 14;
    let color = match kind {
        PieceKind::Die { .. } => COL_WHITE,
        PieceKind::Happy => COL_WHITE,
        _ => bt_color(kind.color()),
    };
    let cells = kind.cells(0);
    for &(col, row) in cells {
        let px = origin_x + col as i32 * PREVIEW_CELL as i32;
        let py = origin_y + row as i32 * PREVIEW_CELL as i32;
        canvas.set_draw_color(color);
        let _ = canvas.fill_rect(Rect::new(px, py, PREVIEW_CELL - 1, PREVIEW_CELL - 1));
    }
}
