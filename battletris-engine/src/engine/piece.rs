use rand::{Rng, rngs::StdRng};
use serde::{Deserialize, Serialize};

use crate::engine::board::{Board, Cell};
use crate::engine::weapons::WeaponState;

// ─── Rotation tables ────────────────────────────────────────────────────────
// All cell offsets are (col, row) relative to the piece's (x, y) origin.
// CW rotation for an N×N grid: (col, row) → (N-1-row, col).
// Special pieces (Wall, Star, WeirdLong) use hand-coded state tables derived
// from BTPiece.C state machines.

// El (J/L variant, 3×3, 4 rotations, BT_BLUE=4)
static EL_ROT: [&[(i32,i32)]; 4] = [
    &[(1,0),(1,1),(1,2),(2,2)],          // spawn: vertical bar + foot right
    &[(0,1),(0,2),(1,1),(2,1)],          // CW90
    &[(0,0),(1,0),(1,1),(1,2)],          // CW180
    &[(0,1),(1,1),(2,0),(2,1)],          // CW270
];

// RevEl (J/L mirror, 3×3, 4 rotations, BT_ORANGE=5)
static REVEL_ROT: [&[(i32,i32)]; 4] = [
    &[(0,2),(1,0),(1,1),(1,2)],          // spawn: vertical bar + foot left
    &[(0,0),(0,1),(1,1),(2,1)],          // CW90
    &[(1,0),(1,1),(1,2),(2,0)],          // CW180
    &[(0,1),(1,1),(2,1),(2,2)],          // CW270
];

// SldLft (S-piece, 3×3, 4 rotations, BT_GREEN=6)
static SLDLFT_ROT: [&[(i32,i32)]; 4] = [
    &[(0,1),(1,1),(1,2),(2,2)],          // spawn horizontal
    &[(0,1),(0,2),(1,0),(1,1)],          // vertical
    &[(0,0),(1,0),(1,1),(2,1)],          // horizontal shifted
    &[(1,1),(1,2),(2,0),(2,1)],          // vertical shifted
];

// SldRt (Z-piece, 3×3, 4 rotations, BT_RED=3)
static SLDRT_ROT: [&[(i32,i32)]; 4] = [
    &[(0,2),(1,1),(1,2),(2,1)],          // spawn horizontal
    &[(0,0),(0,1),(1,1),(1,2)],          // vertical
    &[(0,1),(1,0),(1,1),(2,0)],          // horizontal shifted
    &[(1,0),(1,1),(2,1),(2,2)],          // vertical shifted
];

// Long (I-piece, 4×4, 4 rotations, BT_CYAN=7)
static LONG_ROT: [&[(i32,i32)]; 4] = [
    &[(0,1),(1,1),(2,1),(3,1)],          // horizontal at row 1
    &[(2,0),(2,1),(2,2),(2,3)],          // vertical at col 2
    &[(0,2),(1,2),(2,2),(3,2)],          // horizontal at row 2
    &[(1,0),(1,1),(1,2),(1,3)],          // vertical at col 1
];

// Plug (T-piece, 3×3, 4 rotations, BT_PURPLE=8)
static PLUG_ROT: [&[(i32,i32)]; 4] = [
    &[(0,2),(1,1),(1,2),(2,2)],          // prong up
    &[(0,0),(0,1),(0,2),(1,1)],          // prong right
    &[(0,0),(1,0),(1,1),(2,0)],          // prong down
    &[(1,1),(2,0),(2,1),(2,2)],          // prong left
];

// Box_ (O-piece, 3×3, 1 state, BT_YELLOW=2)
static BOX_ROT: [&[(i32,i32)]; 1] = [
    &[(1,1),(1,2),(2,1),(2,2)],
];

// Die (single cell, 3×3, 1 state — color/rendering is special)
static DIE_ROT: [&[(i32,i32)]; 1] = [
    &[(1,1)],
];

// Happy (single cell, 3×3, 1 state — rendering is special)
static HAPPY_ROT: [&[(i32,i32)]; 1] = [
    &[(1,1)],
];

// Dog (diagonal S-like weird piece, 3×3, 4 rotations, BT_ORANGE=5)
static DOG_ROT: [&[(i32,i32)]; 4] = [
    &[(0,0),(1,1),(2,1),(2,2)],
    &[(0,2),(1,1),(1,2),(2,0)],
    &[(0,0),(0,1),(1,1),(2,2)],
    &[(0,2),(1,0),(1,1),(2,0)],
];

// RevDog (diagonal Z-like weird piece, 3×3, 4 rotations, BT_ORANGE=5)
static REVDOG_ROT: [&[(i32,i32)]; 4] = [
    &[(0,1),(0,2),(1,1),(2,2)],
    &[(0,0),(0,2),(1,0),(1,1)],
    &[(0,0),(1,1),(2,0),(2,1)],
    &[(1,1),(1,2),(2,0),(2,2)],
];

// Cap (arch piece, 4×4, 4 rotations, BT_ORANGE=5)
static CAP_ROT: [&[(i32,i32)]; 4] = [
    &[(0,2),(1,1),(2,1),(3,2)],          // arch opening down
    &[(1,0),(1,3),(2,1),(2,2)],          // arch opening left
    &[(0,1),(1,2),(2,2),(3,1)],          // arch opening up
    &[(1,1),(1,2),(2,0),(2,3)],          // arch opening right
];

// Wall (4×4, 4 custom states from BTPiece.C BTWallPiece::rotate)
static WALL_ROT: [&[(i32,i32)]; 4] = [
    &[(0,1),(0,2),(3,1),(3,2)],          // two vertical pillars
    &[(0,2),(1,3),(2,0),(3,1)],          // diagonal 1
    &[(1,0),(1,3),(2,0),(2,3)],          // two horizontal pairs
    &[(0,1),(1,0),(2,3),(3,2)],          // diagonal 2
];

// Tower (T-variant weird piece, 3×3, 4 rotations, BT_ORANGE=5)
static TOWER_ROT: [&[(i32,i32)]; 4] = [
    &[(0,1),(1,1),(2,0),(2,2)],
    &[(0,2),(1,0),(1,1),(2,2)],
    &[(0,0),(0,2),(1,1),(2,1)],
    &[(0,0),(1,1),(1,2),(2,0)],
];

// Star (+ and X alternation, 3×3, 2 custom states)
static STAR_ROT: [&[(i32,i32)]; 2] = [
    &[(0,1),(1,0),(1,2),(2,1)],          // plus shape
    &[(0,0),(0,2),(2,0),(2,2)],          // X shape
];

// WeirdLong (4×4, 6 custom states from BTWeirdLongPiece::rotate)
static WEIRDLONG_ROT: [&[(i32,i32)]; 6] = [
    &[(1,0),(1,1),(2,2),(2,3)],          // two vertical pairs
    &[(0,0),(1,1),(2,2),(3,3)],          // main diagonal
    &[(0,1),(1,1),(2,2),(3,2)],          // step pattern
    &[(0,2),(1,2),(2,1),(3,1)],          // step pattern (reversed)
    &[(0,3),(1,2),(2,1),(3,0)],          // anti-diagonal
    &[(1,2),(1,3),(2,0),(2,1)],          // two vertical pairs (reversed)
];

// FourByFour (hollow 4×4 square, 1 state, BT_RED=3)
static FOURBYFOUR_ROT: [&[(i32,i32)]; 1] = [
    &[(0,0),(1,0),(2,0),(3,0),
      (0,1),(3,1),
      (0,2),(3,2),
      (0,3),(1,3),(2,3),(3,3)],
];

// LongDong (8-cell horizontal bar, 1 state, BT_IVORY=1)
static LONGDONG_ROT: [&[(i32,i32)]; 1] = [
    &[(0,0),(1,0),(2,0),(3,0),(4,0),(5,0),(6,0),(7,0)],
];

// ─── PieceKind ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PieceKind {
    El,
    RevEl,
    SldLft,
    SldRt,
    Long,
    Plug,
    Box_,
    Die { pips: u8 },
    Happy,
    Dog,
    RevDog,
    Cap,
    Wall,
    Tower,
    Star,
    WeirdLong,
    FourByFour,
    LongDong,
}

impl PieceKind {
    /// Cell offsets (col, row) for the given rotation state.
    pub fn cells(self, rotation: u8) -> &'static [(i32, i32)] {
        let rot = rotation as usize;
        match self {
            PieceKind::El          => EL_ROT[rot % 4],
            PieceKind::RevEl       => REVEL_ROT[rot % 4],
            PieceKind::SldLft      => SLDLFT_ROT[rot % 4],
            PieceKind::SldRt       => SLDRT_ROT[rot % 4],
            PieceKind::Long        => LONG_ROT[rot % 4],
            PieceKind::Plug        => PLUG_ROT[rot % 4],
            PieceKind::Box_        => BOX_ROT[0],
            PieceKind::Die { .. }  => DIE_ROT[0],
            PieceKind::Happy       => HAPPY_ROT[0],
            PieceKind::Dog         => DOG_ROT[rot % 4],
            PieceKind::RevDog      => REVDOG_ROT[rot % 4],
            PieceKind::Cap         => CAP_ROT[rot % 4],
            PieceKind::Wall        => WALL_ROT[rot % 4],
            PieceKind::Tower       => TOWER_ROT[rot % 4],
            PieceKind::Star        => STAR_ROT[rot % 2],
            PieceKind::WeirdLong   => WEIRDLONG_ROT[rot % 6],
            PieceKind::FourByFour  => FOURBYFOUR_ROT[0],
            PieceKind::LongDong    => LONGDONG_ROT[0],
        }
    }

    /// Number of distinct rotation states.
    pub fn rotation_count(self) -> u8 {
        match self {
            PieceKind::El | PieceKind::RevEl | PieceKind::SldLft | PieceKind::SldRt
            | PieceKind::Plug | PieceKind::Dog | PieceKind::RevDog | PieceKind::Tower
            | PieceKind::Long | PieceKind::Cap | PieceKind::Wall => 4,
            PieceKind::Star => 2,
            PieceKind::WeirdLong => 6,
            PieceKind::Box_ | PieceKind::Die { .. } | PieceKind::Happy
            | PieceKind::FourByFour | PieceKind::LongDong => 1,
        }
    }

    /// Spawn x = 5 − rot_/2, using the original BT_DEFAULT_X=5 formula.
    /// rot_ is the bounding-box dimension for each piece family.
    pub fn spawn_x(self) -> i32 {
        let rot_ = match self {
            PieceKind::El | PieceKind::RevEl | PieceKind::SldLft | PieceKind::SldRt
            | PieceKind::Plug | PieceKind::Dog | PieceKind::RevDog | PieceKind::Tower
            | PieceKind::Star => 3,
            PieceKind::Long | PieceKind::Cap | PieceKind::Wall | PieceKind::WeirdLong
            | PieceKind::FourByFour => 4,
            PieceKind::LongDong => 8,
            PieceKind::Box_ | PieceKind::Die { .. } | PieceKind::Happy => 0,
        };
        5 - rot_ / 2
    }

    /// BattleTris color index (1–8) for Regular cell rendering.
    pub fn color(self) -> u8 {
        match self {
            PieceKind::El         => 4, // BT_BLUE
            PieceKind::RevEl      => 5, // BT_ORANGE
            PieceKind::SldLft     => 6, // BT_GREEN
            PieceKind::SldRt      => 3, // BT_RED
            PieceKind::Long       => 7, // BT_CYAN
            PieceKind::Plug       => 8, // BT_PURPLE
            PieceKind::Box_       => 2, // BT_YELLOW
            PieceKind::Die { .. } => 0, // rendered specially
            PieceKind::Happy      => 0, // rendered specially
            PieceKind::LongDong   => 1, // BT_IVORY
            PieceKind::FourByFour => 3, // BT_RED
            _                     => 5, // BT_ORANGE for weird pieces
        }
    }

    /// The Cell value to place on the board when this piece locks.
    pub fn locked_cell(self) -> Cell {
        match self {
            PieceKind::Die { pips } => Cell::Die(pips),
            PieceKind::Happy => Cell::Happy,
            other => Cell::Regular(other.color()),
        }
    }

    /// Weapon-aware piece selector. Applies FW, FBF, Broken, SoLong, NoDice, NiceDay filters.
    pub fn random_filtered(ws: &WeaponState, rng: &mut StdRng) -> PieceKind {
        use crate::engine::weapons::WeaponKind;

        // NiceDay: next piece is Happy
        if ws.nice_day_pending {
            return PieceKind::Happy;
        }

        // Broken Record: repeat the locked piece kind
        if ws.is_active(WeaponKind::Broken) {
            if let Some(kind) = ws.broken_kind {
                return kind;
            }
        }

        // Feared Weird: only weird pieces
        if ws.is_active(WeaponKind::FW) {
            let weird = [
                PieceKind::Dog, PieceKind::RevDog, PieceKind::Cap,
                PieceKind::Wall, PieceKind::Tower, PieceKind::Star, PieceKind::WeirdLong,
            ];
            return weird[rng.gen_range(0..weird.len())];
        }

        loop {
            let mut piece = PieceKind::random(rng);
            // FourByFour: Box → FourByFour
            if ws.is_active(WeaponKind::FBF) {
                if piece == PieceKind::Box_ {
                    piece = PieceKind::FourByFour;
                }
            }
            // SoLong: reject Long pieces
            if ws.is_active(WeaponKind::SoLong) && piece == PieceKind::Long {
                continue;
            }
            // NoDice: reject Die pieces
            if ws.is_active(WeaponKind::NoDice) {
                if let PieceKind::Die { .. } = piece {
                    continue;
                }
            }
            return piece;
        }
    }

    /// Rejection-sampling piece selector faithful to BTPieceManager::newPiece().
    ///
    /// Probabilities: Die = 1.0, normal pieces = 0.21, Happy/LongDong = 0.02,
    /// weird pieces = 0.0 (never in normal play — only via FearWeird/FourByFour weapons).
    pub fn random(rng: &mut StdRng) -> PieceKind {
        loop {
            let idx: u8 = rng.gen_range(1..=18);
            let threshold: f64 = match idx {
                1..=7  => 0.21, // El, RevEl, SldLft, SldRt, Long, Plug, Box_
                8      => 1.0,  // Die always accepted
                9      => 0.02, // Happy
                10..=16 => 0.0, // weird pieces — never in normal play
                17     => 0.0,  // FourByFour — only via weapon
                18     => 0.02, // LongDong
                _      => unreachable!(),
            };
            if rng.gen::<f64>() < threshold {
                return match idx {
                    1  => PieceKind::El,
                    2  => PieceKind::RevEl,
                    3  => PieceKind::SldLft,
                    4  => PieceKind::SldRt,
                    5  => PieceKind::Long,
                    6  => PieceKind::Plug,
                    7  => PieceKind::Box_,
                    8  => PieceKind::Die { pips: rng.gen_range(1..=6) },
                    9  => PieceKind::Happy,
                    10 => PieceKind::Dog,
                    11 => PieceKind::RevDog,
                    12 => PieceKind::Cap,
                    13 => PieceKind::Wall,
                    14 => PieceKind::Tower,
                    15 => PieceKind::Star,
                    16 => PieceKind::WeirdLong,
                    17 => PieceKind::FourByFour,
                    18 => PieceKind::LongDong,
                    _  => unreachable!(),
                };
            }
        }
    }
}

// ─── ActivePiece ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivePiece {
    pub kind: PieceKind,
    pub x: i32,
    pub y: i32,
    pub rotation: u8,
}

impl ActivePiece {
    pub fn new(kind: PieceKind) -> Self {
        ActivePiece {
            x: kind.spawn_x(),
            y: 0,
            rotation: 0,
            kind,
        }
    }

    /// Absolute board positions of this piece's cells.
    pub fn absolute_cells(&self) -> Vec<(i32, i32)> {
        self.kind
            .cells(self.rotation)
            .iter()
            .map(|&(dc, dr)| (self.x + dc, self.y + dr))
            .collect()
    }

    /// True if the piece at the given (x, y, rotation) would not overlap occupied cells.
    pub fn can_place_at(&self, board: &Board, nx: i32, ny: i32, rot: u8) -> bool {
        self.kind
            .cells(rot)
            .iter()
            .all(|&(dc, dr)| !board.occupied(nx + dc, ny + dr))
    }

    /// Try to move left; returns true on success.
    pub fn try_move_left(&mut self, board: &Board) -> bool {
        if self.can_place_at(board, self.x - 1, self.y, self.rotation) {
            self.x -= 1;
            true
        } else {
            false
        }
    }

    /// Try to move right; returns true on success.
    pub fn try_move_right(&mut self, board: &Board) -> bool {
        if self.can_place_at(board, self.x + 1, self.y, self.rotation) {
            self.x += 1;
            true
        } else {
            false
        }
    }

    /// Try to move down one row; returns true on success.
    pub fn try_move_down(&mut self, board: &Board) -> bool {
        if self.can_place_at(board, self.x, self.y + 1, self.rotation) {
            self.y += 1;
            true
        } else {
            false
        }
    }

    /// Try to rotate CW; fails silently (no wall kicks) if blocked.
    pub fn try_rotate_cw(&mut self, board: &Board) -> bool {
        let next_rot = (self.rotation + 1) % self.kind.rotation_count();
        if self.can_place_at(board, self.x, self.y, next_rot) {
            self.rotation = next_rot;
            true
        } else {
            false
        }
    }

    /// Try to rotate CCW; fails silently if blocked.
    pub fn try_rotate_ccw(&mut self, board: &Board) -> bool {
        let count = self.kind.rotation_count();
        let next_rot = (self.rotation + count - 1) % count;
        if self.can_place_at(board, self.x, self.y, next_rot) {
            self.rotation = next_rot;
            true
        } else {
            false
        }
    }

    /// Hard-drop y: the lowest row this piece can occupy.
    pub fn ghost_y(&self, board: &Board) -> i32 {
        let mut gy = self.y;
        while self.can_place_at(board, self.x, gy + 1, self.rotation) {
            gy += 1;
        }
        gy
    }
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn seeded() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    #[test]
    fn el_rotation_cycle_returns_to_start() {
        let cells0 = PieceKind::El.cells(0);
        let cells4 = PieceKind::El.cells(4); // wraps via % 4
        assert_eq!(cells0, cells4);
    }

    #[test]
    fn box_does_not_rotate() {
        let c0 = PieceKind::Box_.cells(0);
        let c1 = PieceKind::Box_.cells(1);
        assert_eq!(c0, c1);
        assert_eq!(PieceKind::Box_.rotation_count(), 1);
    }

    #[test]
    fn weirdlong_has_six_states() {
        assert_eq!(PieceKind::WeirdLong.rotation_count(), 6);
        // All 6 states must be distinct
        let states: Vec<_> = (0..6).map(|r| PieceKind::WeirdLong.cells(r)).collect();
        for i in 0..6 {
            for j in (i+1)..6 {
                assert_ne!(states[i], states[j], "WeirdLong states {i} and {j} are identical");
            }
        }
    }

    #[test]
    fn star_has_two_distinct_states() {
        let s0 = PieceKind::Star.cells(0);
        let s1 = PieceKind::Star.cells(1);
        assert_ne!(s0, s1);
        assert_eq!(PieceKind::Star.cells(2), s0); // wraps
    }

    #[test]
    fn can_place_at_collision() {
        let mut board = Board::new();
        // Block the bottom row entirely
        for col in 0..10_i32 {
            board.set_cell(col, 27, Cell::Regular(1));
        }
        let piece = ActivePiece::new(PieceKind::El);
        // Piece at spawn (x=4, y=0) should be fine
        assert!(piece.can_place_at(&board, 4, 0, 0));
        // Should not be placeable inside the filled row
        // El at (4, 26) rotation 0 has cells at (4+1, 26+2)=(5,28)? No: row 26+2=28 is floor.
        // Let's test a simpler collision: try placing El at y=25 where (1,2) → (5,27) which is filled
        assert!(!piece.can_place_at(&board, 4, 25, 0)); // cell (4+1, 25+2) = (5,27) occupied
    }

    #[test]
    fn ghost_y_stops_at_floor() {
        let board = Board::new();
        let piece = ActivePiece::new(PieceKind::Plug);
        let gy = piece.ghost_y(&board);
        // Plug rot0 cells: (0,2),(1,1),(1,2),(2,2). deepest row offset = 2.
        // Bottom of board = row 27. So gy + 2 <= 27 → gy <= 25.
        assert!(gy <= 25);
        // And piece at gy+1 should be blocked
        assert!(!piece.can_place_at(&board, piece.x, gy + 1, piece.rotation));
    }

    #[test]
    fn rotation_cw_and_ccw_cancel() {
        let board = Board::new();
        let mut piece = ActivePiece::new(PieceKind::Long);
        let orig_rot = piece.rotation;
        piece.try_rotate_cw(&board);
        piece.try_rotate_ccw(&board);
        assert_eq!(piece.rotation, orig_rot);
    }

    #[test]
    fn random_distribution_no_weird_pieces() {
        let mut rng = seeded();
        let mut counts = [0u32; 19]; // index 0 unused

        for _ in 0..10_000 {
            match PieceKind::random(&mut rng) {
                PieceKind::El          => counts[1]  += 1,
                PieceKind::RevEl       => counts[2]  += 1,
                PieceKind::SldLft      => counts[3]  += 1,
                PieceKind::SldRt       => counts[4]  += 1,
                PieceKind::Long        => counts[5]  += 1,
                PieceKind::Plug        => counts[6]  += 1,
                PieceKind::Box_        => counts[7]  += 1,
                PieceKind::Die { .. }  => counts[8]  += 1,
                PieceKind::Happy       => counts[9]  += 1,
                PieceKind::Dog         => counts[10] += 1,
                PieceKind::RevDog      => counts[11] += 1,
                PieceKind::Cap         => counts[12] += 1,
                PieceKind::Wall        => counts[13] += 1,
                PieceKind::Tower       => counts[14] += 1,
                PieceKind::Star        => counts[15] += 1,
                PieceKind::WeirdLong   => counts[16] += 1,
                PieceKind::FourByFour  => counts[17] += 1,
                PieceKind::LongDong    => counts[18] += 1,
            }
        }

        // Weird pieces must never appear in normal play
        for i in 10..=17 {
            assert_eq!(counts[i], 0, "weird piece index {i} appeared in normal play");
        }

        // Die should appear most often (~40% of pieces)
        assert!(counts[8] > 3000, "Die too rare: {}", counts[8]);

        // Normal pieces should each appear roughly 8% of the time (~800 each)
        for i in 1..=7 {
            assert!(counts[i] > 400, "normal piece {i} too rare: {}", counts[i]);
        }
    }

    #[test]
    fn longdong_no_rotation() {
        assert_eq!(PieceKind::LongDong.rotation_count(), 1);
        assert_eq!(PieceKind::LongDong.cells(0).len(), 8);
    }

    #[test]
    fn fourbyfour_twelve_cells() {
        assert_eq!(PieceKind::FourByFour.cells(0).len(), 12);
    }
}
