use battletris_engine::engine::board::{BOARD_COLS, BOARD_ROWS};

pub const CELL_PX: f64 = 28.0;
pub const WINDOW_W: f64 = 820.0;
pub const WINDOW_H: f64 = 860.0;

pub const PLAYER_BOARD_X: f64 = 20.0;
pub const PLAYER_BOARD_Y: f64 = 40.0;
pub const STATS_X: f64 = 310.0;
pub const STATS_Y: f64 = 40.0;
pub const OPP_BOARD_X: f64 = 520.0;
pub const OPP_BOARD_Y: f64 = 40.0;

pub const BOARD_PX_W: f64 = CELL_PX * BOARD_COLS as f64; // 280.0
pub const BOARD_PX_H: f64 = CELL_PX * BOARD_ROWS as f64; // 784.0
