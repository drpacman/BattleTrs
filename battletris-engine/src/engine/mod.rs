pub mod board;
pub mod game_state;
pub mod piece;
pub mod score;
pub mod weapons;

pub use board::{Board, BoardSnapshot, Cell, LinesCleared, BOARD_COLS, BOARD_ROWS};
pub use game_state::{
    GameEvent, GameMode, GamePhase, GameState, PieceState, PlayerInput, PlayingView,
    DROP_INTERVAL_MS, FAST_DROP_INTERVAL_MS, LOCK_DELAY_MS, LINES_UNTIL_BAZAAR,
};
pub use piece::{ActivePiece, PieceKind};
pub use score::{Score, ScoreView};
pub use weapons::{
    Arsenal, ArsenalSlot, ArsenalSlotView, ActiveWeaponView,
    BazaarState, BazaarStateView,
    WeaponKind, WeaponState, weapon_def, WEAPON_COUNT, ARSENAL_SIZE,
    MirrorResult, check_mirror,
};
