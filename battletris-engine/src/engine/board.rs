use rand::{Rng, rngs::StdRng};
use serde::{Deserialize, Serialize};

pub const BOARD_COLS: usize = 10;
pub const BOARD_ROWS: usize = 28;

// Fallout zone: middle 6 columns cleared by Fallout weapon on activation
pub const FALLOUT_COL_MIN: usize = 2;
pub const FALLOUT_COL_MAX: usize = 7; // inclusive

// Bottle neck zone: rows 7-20; cols 0-2 and 7-9 become Struct_ walls
pub const BOTTLE_ROW_MIN: usize = 7;
pub const BOTTLE_ROW_MAX: usize = 20; // inclusive
pub const BOTTLE_WALL_COLS: &[usize] = &[0, 1, 2, 7, 8, 9];

/// A single cell on the BattleTris board. Mirrors the original BTBox hierarchy.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Regular(u8),  // color index 1–8
    Die(u8),      // pip count 1–6; fund_value = pips
    Happy,        // ☺ smiley; fund_value = 150
    HappyMissed,  // ☹ frown after happy cell in non-cleared row; fund_value = 0
    Struct_,      // permanent wall cell (Bottle zone / out-of-bounds sentinel)
    Bug,          // invisible solid cell (Bug Report weapon) — occupied but rendered empty
    Twilight,     // cell turned invisible by Twilight Zone weapon — solid but rendered grey
    /// Gimp weapon: existing cell wrapped in uniform distracting appearance; still solid/clearable.
    /// Carries the original cell's fund value so Die/Happy values survive the conversion.
    Gimp(i32),
}

impl Cell {
    pub fn is_empty(self) -> bool {
        matches!(self, Cell::Empty)
    }

    /// Funds contributed when this cell is part of a cleared line (BTBox::value()).
    pub fn fund_value(self) -> i32 {
        match self {
            Cell::Die(pips) => pips as i32,
            Cell::Happy => 150,
            // Gimp preserves the original cell's value (BTGimpBox::value_ in original).
            Cell::Gimp(v) => v,
            _ => 0,
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The 10×28 game board (BTBoardManager equivalent).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    /// cells[row][col], row 0 is top.
    cells: [[Cell; BOARD_COLS]; BOARD_ROWS],
    pub upside_down: bool,
    /// True while Fallout is active: columns FALLOUT_COL_MIN–FALLOUT_COL_MAX have no floor.
    /// Mirrors BTBoardManager::occupied() Fallout branch from the original.
    pub fallout_active: bool,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            cells: [[Cell::Empty; BOARD_COLS]; BOARD_ROWS],
            upside_down: false,
            fallout_active: false,
        }
    }
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether position (col, row) is occupied.
    ///
    /// Mirrors BTBoardManager::occupied().  When Fallout is active the floor is
    /// removed for FALLOUT_COL_MIN–FALLOUT_COL_MAX (cols 2–7), so pieces fall
    /// straight through.  A hard safety bound at BOARD_ROWS+4 ensures pieces
    /// eventually stop regardless (matches original `height_ + BT_PIECE_HEIGHT`).
    pub fn occupied(&self, col: i32, row: i32) -> bool {
        if col < 0 || col >= BOARD_COLS as i32 {
            return true; // side walls
        }
        if row < 0 {
            return true; // ceiling
        }
        if row >= BOARD_ROWS as i32 {
            // Hard safety floor prevents infinite falling.
            if row >= BOARD_ROWS as i32 + 4 {
                return true;
            }
            // During Fallout, center columns (2–7) have no floor.
            if self.fallout_active {
                let c = col as usize;
                if c >= FALLOUT_COL_MIN && c <= FALLOUT_COL_MAX {
                    return false;
                }
            }
            return true; // ledge columns (0–1, 8–9) always have a floor
        }
        !self.cells[row as usize][col as usize].is_empty()
    }

    pub fn cell(&self, col: i32, row: i32) -> Cell {
        if col < 0 || col >= BOARD_COLS as i32 || row < 0 || row >= BOARD_ROWS as i32 {
            return Cell::Struct_;
        }
        self.cells[row as usize][col as usize]
    }

    pub fn set_cell(&mut self, col: i32, row: i32, cell: Cell) {
        if col >= 0 && col < BOARD_COLS as i32 && row >= 0 && row < BOARD_ROWS as i32 {
            self.cells[row as usize][col as usize] = cell;
        }
    }

    /// Lock a falling piece onto the board.
    pub fn lock_piece(&mut self, cells: &[(i32, i32)], cell_value: Cell) {
        for &(col, row) in cells {
            self.set_cell(col, row, cell_value);
        }
    }

    /// Scan for completed lines, clear them, shift rows down, and return summary.
    ///
    /// Implements BTBoardManager::checkLines():
    /// - full rows are cleared and shifted down in one pass
    /// - happy cells in NON-cleared rows convert to HappyMissed in the same pass
    /// - funds = sum(cell.fund_value() for cleared cells) × number_of_lines_cleared
    pub fn check_and_clear_lines(&mut self) -> LinesCleared {
        // Collect full-row indices (bottom-up order doesn't matter; we'll sort anyway)
        let full_rows: Vec<usize> = (0..BOARD_ROWS)
            .filter(|&r| (0..BOARD_COLS).all(|c| !self.cells[r][c].is_empty()))
            .collect();

        let count = full_rows.len() as u32;
        if count == 0 {
            return LinesCleared::default();
        }

        // Accumulate fund values from cleared rows
        let raw_fund_sum: i32 = full_rows
            .iter()
            .flat_map(|&r| (0..BOARD_COLS).map(move |c| (r, c)))
            .map(|(r, c)| self.cells[r][c].fund_value())
            .sum();
        let funds_earned = raw_fund_sum * count as i32;

        // Convert happy cells in non-cleared rows to HappyMissed
        let full_set: std::collections::HashSet<usize> = full_rows.iter().copied().collect();
        let mut happy_missed = false;
        for r in 0..BOARD_ROWS {
            if full_set.contains(&r) {
                continue;
            }
            for c in 0..BOARD_COLS {
                if self.cells[r][c] == Cell::Happy {
                    self.cells[r][c] = Cell::HappyMissed;
                    happy_missed = true;
                }
            }
        }

        // Rebuild board: collect non-cleared rows in order, pad the top with empty rows.
        // Iterative single-row removal corrupts indices for multi-row clears.
        let new_rows: Vec<[Cell; BOARD_COLS]> = (0..BOARD_ROWS)
            .filter(|r| !full_set.contains(r))
            .map(|r| self.cells[r])
            .collect();

        let pad = BOARD_ROWS - new_rows.len();
        for r in 0..pad {
            self.cells[r] = [Cell::Empty; BOARD_COLS];
        }
        for (i, row) in new_rows.iter().enumerate() {
            self.cells[pad + i] = *row;
        }

        LinesCleared { count, funds_earned, happy_missed }
    }

    /// Rise Up: shift all rows up by 1, insert a junk row at the bottom.
    /// Returns true if topped-out (row 0 was occupied before the shift).
    pub fn rise_up(&mut self, rng: &mut StdRng) -> bool {
        if self.is_topped_out() {
            return true;
        }
        // Shift rows up: row[i] = row[i+1]
        for r in 0..BOARD_ROWS - 1 {
            self.cells[r] = self.cells[r + 1];
        }
        // New bottom row: all solid except one random hole
        let hole_col = rng.gen_range(0..BOARD_COLS);
        let junk_color = rng.gen_range(1u8..=8);
        for c in 0..BOARD_COLS {
            self.cells[BOARD_ROWS - 1][c] = if c == hole_col {
                Cell::Empty
            } else {
                Cell::Regular(junk_color)
            };
        }
        false
    }

    /// Flip Out: mirror the board horizontally (col 0 ↔ col 9).
    pub fn flip_out(&mut self) {
        for row in &mut self.cells {
            let mut mirrored = [Cell::Empty; BOARD_COLS];
            for c in 0..BOARD_COLS {
                mirrored[BOARD_COLS - 1 - c] = row[c];
            }
            *row = mirrored;
        }
    }

    /// Force clear: zero full rows in-place without shifting rows down (Force weapon).
    pub fn force_clear_lines(&mut self) -> LinesCleared {
        let full_rows: Vec<usize> = (0..BOARD_ROWS)
            .filter(|&r| (0..BOARD_COLS).all(|c| !self.cells[r][c].is_empty()))
            .collect();

        let count = full_rows.len() as u32;
        if count == 0 {
            return LinesCleared::default();
        }

        let raw_fund_sum: i32 = full_rows
            .iter()
            .flat_map(|&r| (0..BOARD_COLS).map(move |c| (r, c)))
            .map(|(r, c)| self.cells[r][c].fund_value())
            .sum();
        let funds_earned = raw_fund_sum * count as i32;

        // Mark happy cells in non-cleared rows as missed
        let full_set: std::collections::HashSet<usize> = full_rows.iter().copied().collect();
        let mut happy_missed = false;
        for r in 0..BOARD_ROWS {
            if !full_set.contains(&r) {
                for c in 0..BOARD_COLS {
                    if self.cells[r][c] == Cell::Happy {
                        self.cells[r][c] = Cell::HappyMissed;
                        happy_missed = true;
                    }
                }
            }
        }

        // Zero cleared rows in place — rows do NOT shift
        for &r in &full_rows {
            self.cells[r] = [Cell::Empty; BOARD_COLS];
        }

        LinesCleared { count, funds_earned, happy_missed }
    }

    /// Remove one random non-empty, non-Struct_ cell. Returns false if board is empty.
    pub fn remove_random_cell(&mut self, rng: &mut StdRng) -> bool {
        let occupied: Vec<(usize, usize)> = (0..BOARD_ROWS)
            .flat_map(|r| (0..BOARD_COLS).map(move |c| (r, c)))
            .filter(|&(r, c)| !self.cells[r][c].is_empty() && self.cells[r][c] != Cell::Struct_)
            .collect();
        if occupied.is_empty() {
            return false;
        }
        let (r, c) = occupied[rng.gen_range(0..occupied.len())];
        self.cells[r][c] = Cell::Empty;
        true
    }

    /// Place cells (relative offsets) at (col+dc, row+dr). Returns false if any overlap.
    pub fn add_piece_at(&mut self, cells: &[(i32, i32)], col: i32, row: i32, cell_type: Cell) -> bool {
        // Check overlaps first
        for &(dc, dr) in cells {
            let c = col + dc;
            let r = row + dr;
            if c < 0 || c >= BOARD_COLS as i32 || r < 0 || r >= BOARD_ROWS as i32 {
                return false;
            }
            if !self.cells[r as usize][c as usize].is_empty() {
                return false;
            }
        }
        for &(dc, dr) in cells {
            self.cells[(row + dr) as usize][(col + dc) as usize] = cell_type;
        }
        true
    }

    /// Convert all non-empty, non-Struct_ cells to Cell::Twilight (Twilight Zone weapon).
    pub fn apply_twilight(&mut self) {
        for row in &mut self.cells {
            for cell in row.iter_mut() {
                if !cell.is_empty() && *cell != Cell::Struct_ {
                    *cell = Cell::Twilight;
                }
            }
        }
    }

    /// Convert all non-empty, non-Struct_ cells to Cell::Gimp (Gimp weapon).
    /// Each cell's fund value is captured before conversion so Die/Happy values survive.
    pub fn apply_gimp(&mut self) {
        for row in &mut self.cells {
            for cell in row.iter_mut() {
                if !cell.is_empty() && *cell != Cell::Struct_ {
                    *cell = Cell::Gimp(cell.fund_value());
                }
            }
        }
    }

    /// Clear all cells in the Fallout zone (cols 2–7) across every row (Fallout weapon, one-time).
    pub fn apply_fallout_wipe(&mut self) {
        for row in &mut self.cells {
            for c in FALLOUT_COL_MIN..=FALLOUT_COL_MAX {
                if row[c] != Cell::Struct_ {
                    row[c] = Cell::Empty;
                }
            }
        }
    }

    /// Randomly remove ~50% of all removable board cells (Blind Cleric weapon).
    pub fn apply_blind(&mut self, rng: &mut StdRng) {
        for row in &mut self.cells {
            for cell in row.iter_mut() {
                if !cell.is_empty() && *cell != Cell::Struct_ && rng.gen_bool(0.5) {
                    *cell = Cell::Empty;
                }
            }
        }
    }

    /// Fill Bottle neck zone walls with Struct_ cells (Bottle Neck weapon activation).
    /// Rows 7-20, cols 0-2 and 7-9 become permanent walls.
    pub fn fill_bottle_walls(&mut self) {
        for r in BOTTLE_ROW_MIN..=BOTTLE_ROW_MAX {
            for &c in BOTTLE_WALL_COLS {
                if self.cells[r][c].is_empty() {
                    self.cells[r][c] = Cell::Struct_;
                }
            }
        }
    }

    /// Remove Struct_ bottle walls (Bottle Neck weapon deactivation).
    pub fn clear_bottle_walls(&mut self) {
        for r in BOTTLE_ROW_MIN..=BOTTLE_ROW_MAX {
            for &c in BOTTLE_WALL_COLS {
                if self.cells[r][c] == Cell::Struct_ {
                    self.cells[r][c] = Cell::Empty;
                }
            }
        }
    }

    /// True if the top row has any non-empty cell — game-over condition.
    pub fn is_topped_out(&self) -> bool {
        (0..BOARD_COLS).any(|c| !self.cells[0][c].is_empty())
    }

    pub fn snapshot(&self) -> BoardSnapshot {
        BoardSnapshot { cells: self.cells }
    }

    /// Replace this board's cells with those from a snapshot (Swap weapon).
    /// Does not touch `upside_down` or `fallout_active` — those are per-player effects.
    pub fn load_snapshot(&mut self, snap: &BoardSnapshot) {
        self.cells = snap.cells;
    }
}

/// Lightweight copy of board state for the renderer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardSnapshot {
    pub cells: [[Cell; BOARD_COLS]; BOARD_ROWS],
}

/// Result of a line-clear pass.
#[derive(Clone, Debug, Default)]
pub struct LinesCleared {
    pub count: u32,
    pub funds_earned: i32,
    pub happy_missed: bool,
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fill_row(board: &mut Board, row: usize, cell: Cell) {
        for col in 0..BOARD_COLS {
            board.cells[row][col] = cell;
        }
    }

    #[test]
    fn occupied_ceiling_wall() {
        let board = Board::new();
        assert!(board.occupied(5, -1));
        assert!(board.occupied(0, -100));
        assert!(!board.occupied(5, 0));
        assert!(!board.occupied(0, 27));
        assert!(board.occupied(-1, 10));
        assert!(board.occupied(10, 10));
        assert!(board.occupied(5, 28));
    }

    #[test]
    fn fallout_removes_floor_for_center_columns() {
        let mut board = Board::new();
        // Without Fallout: floor exists everywhere at row BOARD_ROWS
        assert!(board.occupied(5, BOARD_ROWS as i32));     // center col, has floor
        assert!(board.occupied(1, BOARD_ROWS as i32));     // ledge col, has floor
        board.fallout_active = true;
        // Center columns 2–7: floor removed
        assert!(!board.occupied(2, BOARD_ROWS as i32));
        assert!(!board.occupied(5, BOARD_ROWS as i32));
        assert!(!board.occupied(7, BOARD_ROWS as i32));
        // Ledge columns 0–1 and 8–9: floor still exists
        assert!(board.occupied(0, BOARD_ROWS as i32));
        assert!(board.occupied(1, BOARD_ROWS as i32));
        assert!(board.occupied(8, BOARD_ROWS as i32));
        assert!(board.occupied(9, BOARD_ROWS as i32));
        // Safety bound: at BOARD_ROWS+4 always occupied; below that still open
        assert!(board.occupied(5, BOARD_ROWS as i32 + 4));
        assert!(!board.occupied(5, BOARD_ROWS as i32 + 3));
    }

    #[test]
    fn single_line_clear() {
        let mut board = Board::new();
        fill_row(&mut board, 27, Cell::Regular(1));
        let result = board.check_and_clear_lines();
        assert_eq!(result.count, 1);
        assert_eq!(result.funds_earned, 0); // Regular cells have fund_value = 0
        // Row 27 should now be empty
        assert!((0..BOARD_COLS).all(|c| board.cells[27][c].is_empty()));
    }

    #[test]
    fn die_cell_funds_single_line() {
        let mut board = Board::new();
        // Fill row 27 with Die(3) in every column
        fill_row(&mut board, 27, Cell::Die(3));
        let result = board.check_and_clear_lines();
        assert_eq!(result.count, 1);
        // funds = (3 × 10 cells) × 1 line = 30
        assert_eq!(result.funds_earned, 30);
    }

    #[test]
    fn happy_cell_funds_and_missed() {
        let mut board = Board::new();
        // Row 27: fill with Regular, but put Happy in one cell of row 26 (not cleared)
        fill_row(&mut board, 27, Cell::Regular(1));
        board.cells[26][0] = Cell::Happy;
        let result = board.check_and_clear_lines();
        assert_eq!(result.count, 1);
        assert_eq!(result.funds_earned, 0); // No happy in cleared row
        assert!(result.happy_missed);
        // Row 27 was cleared; row 26 (with the happy cell) shifts down to row 27
        assert_eq!(board.cells[27][0], Cell::HappyMissed);
    }

    #[test]
    fn happy_cell_in_cleared_row_earns_funds() {
        let mut board = Board::new();
        // Row 27: 9 regular + 1 happy
        fill_row(&mut board, 27, Cell::Regular(1));
        board.cells[27][5] = Cell::Happy;
        let result = board.check_and_clear_lines();
        assert_eq!(result.count, 1);
        // funds = 150 × 1 line = 150
        assert_eq!(result.funds_earned, 150);
        assert!(!result.happy_missed);
    }

    #[test]
    fn multi_line_clear_multiplies_funds() {
        let mut board = Board::new();
        // Rows 26 and 27: all Die(2)
        fill_row(&mut board, 26, Cell::Die(2));
        fill_row(&mut board, 27, Cell::Die(2));
        let result = board.check_and_clear_lines();
        assert_eq!(result.count, 2);
        // funds = (2×10 + 2×10) raw_sum = 40; × 2 lines = 80
        assert_eq!(result.funds_earned, 80);
    }

    #[test]
    fn rows_shift_down_after_clear() {
        let mut board = Board::new();
        // Put a marker cell at row 25, then fill rows 26 and 27
        board.cells[25][0] = Cell::Regular(7);
        fill_row(&mut board, 26, Cell::Regular(1));
        fill_row(&mut board, 27, Cell::Regular(1));
        board.check_and_clear_lines();
        // Marker should have shifted down by 2 rows
        assert_eq!(board.cells[27][0], Cell::Regular(7));
        assert!(board.cells[25][0].is_empty());
    }

    #[test]
    fn topped_out_detection() {
        let mut board = Board::new();
        assert!(!board.is_topped_out());
        board.cells[0][5] = Cell::Regular(1);
        assert!(board.is_topped_out());
    }

    #[test]
    fn rise_up_inserts_junk_row() {
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(1);
        let mut board = Board::new();
        board.cells[10][0] = Cell::Regular(1);
        let topped = board.rise_up(&mut rng);
        assert!(!topped);
        // The marker should have moved up one row (row 9 now)
        assert_eq!(board.cells[9][0], Cell::Regular(1));
        // Bottom row should be junk (not all empty)
        let bottom_has_cells = (0..BOARD_COLS).any(|c| !board.cells[BOARD_ROWS-1][c].is_empty());
        assert!(bottom_has_cells);
    }

    #[test]
    fn rise_up_returns_topped_when_row0_occupied() {
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(2);
        let mut board = Board::new();
        board.cells[0][3] = Cell::Regular(2);
        let topped = board.rise_up(&mut rng);
        assert!(topped);
    }

    #[test]
    fn flip_out_mirrors_cells() {
        let mut board = Board::new();
        board.cells[5][0] = Cell::Regular(1);
        board.cells[5][9] = Cell::Regular(2);
        board.flip_out();
        assert_eq!(board.cells[5][9], Cell::Regular(1));
        assert_eq!(board.cells[5][0], Cell::Regular(2));
    }

    #[test]
    fn force_clear_does_not_shift() {
        let mut board = Board::new();
        fill_row(&mut board, 27, Cell::Regular(1));
        board.cells[26][0] = Cell::Regular(3);
        let result = board.force_clear_lines();
        assert_eq!(result.count, 1);
        // Row 27 is zeroed in place
        assert!((0..BOARD_COLS).all(|c| board.cells[27][c].is_empty()));
        // Row 26 marker did NOT shift (force doesn't shift rows)
        assert_eq!(board.cells[26][0], Cell::Regular(3));
    }

    #[test]
    fn remove_random_cell_clears_one() {
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(3);
        let mut board = Board::new();
        board.cells[10][5] = Cell::Regular(1);
        let removed = board.remove_random_cell(&mut rng);
        assert!(removed);
        assert!(board.cells[10][5].is_empty());
    }

    #[test]
    fn add_piece_at_places_and_detects_overlap() {
        let mut board = Board::new();
        let cells = [(0i32, 0i32), (1, 0)];
        let placed = board.add_piece_at(&cells, 3, 10, Cell::Regular(1));
        assert!(placed);
        assert_eq!(board.cells[10][3], Cell::Regular(1));
        assert_eq!(board.cells[10][4], Cell::Regular(1));
        // Second placement at same location should fail
        let placed2 = board.add_piece_at(&cells, 3, 10, Cell::Regular(2));
        assert!(!placed2);
    }

    #[test]
    fn apply_twilight_converts_cells() {
        let mut board = Board::new();
        board.cells[5][5] = Cell::Regular(1);
        board.cells[10][3] = Cell::Happy;
        board.apply_twilight();
        assert_eq!(board.cells[5][5], Cell::Twilight);
        assert_eq!(board.cells[10][3], Cell::Twilight);
        assert!(board.cells[0][0].is_empty()); // empty cells unchanged
    }

    #[test]
    fn bottle_walls_fill_and_clear() {
        let mut board = Board::new();
        board.fill_bottle_walls();
        // Check one wall cell is Struct_
        assert_eq!(board.cells[BOTTLE_ROW_MIN][0], Cell::Struct_);
        assert_eq!(board.cells[BOTTLE_ROW_MAX][9], Cell::Struct_);
        // Neck columns are unaffected
        assert!(board.cells[BOTTLE_ROW_MIN][3].is_empty());
        // Clear walls
        board.clear_bottle_walls();
        assert!(board.cells[BOTTLE_ROW_MIN][0].is_empty());
    }

    #[test]
    fn bug_cell_is_solid() {
        assert!(!Cell::Bug.is_empty());
        assert_eq!(Cell::Bug.fund_value(), 0);
    }

    #[test]
    fn twilight_cell_is_solid() {
        assert!(!Cell::Twilight.is_empty());
    }
}
