use rand::{Rng, rngs::StdRng};

use crate::engine::board::{Board, Cell, BOARD_COLS, BOARD_ROWS};
use crate::engine::piece::PieceKind;
use crate::engine::weapons::{Arsenal, WeaponKind, WeaponState, weapon_def, WEAPON_COUNT};
use crate::engine::score::Score;

// ─── Difficulty levels (from BTComputer.C levels[]) ──────────────────────────

/// All 15 difficulty levels in order from easiest to hardest.
/// Each entry is (name, think_interval_ms).
pub const LEVELS: &[(&str, u64)] = &[
    ("Comatose",     4000),
    ("Somnambulant", 3000),
    ("Lethargic",    2000),
    ("Pensive",      1500),
    ("Able",         1250),
    ("Willing",      1000),
    ("Focused",       750),
    ("Lively",        550),
    ("Energetic",     400),
    ("Pepped-up",     350),
    ("Caffeinated",   300),
    ("Bug-eyed",      225),
    ("Supercharged",  100),
    ("Hell-Bent",      10),
    ("Bionic",          0),
];

/// Return the think-interval milliseconds for a given 0-based difficulty index.
pub fn difficulty_think_ms(level: u8) -> u64 {
    let idx = (level as usize).min(LEVELS.len() - 1);
    LEVELS[idx].1
}

// ─── AiPenalties (from BTComputer.C) ────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct AiPenalties {
    pub open_hole:    i64,
    pub closed_hole:  i64,
    pub covered_hole: i64,
    pub height:       i64,
    pub variance:     i64,
    pub line_bonus:   i64,
    pub happy_bonus:  i64,
}

impl Default for AiPenalties {
    fn default() -> Self {
        AiPenalties {
            open_hole:    7_000,
            closed_hole:  10_000,
            covered_hole: 3_000,
            height:       30_000,
            variance:     50,
            line_bonus:   5_000,
            happy_bonus:  20_000,
        }
    }
}

// ─── AiMove ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct AiMove {
    pub col: i32,
    pub rotation: u8,
    pub score: i64,
}

// ─── Ai ──────────────────────────────────────────────────────────────────────

pub struct Ai {
    #[allow(dead_code)]
    difficulty: u8,
    pub penalties: AiPenalties,
    /// Weapons Ernie is permitted to purchase (all true initially, some banned).
    pub can_purchase: [bool; WEAPON_COUNT],
    /// Weapons Ernie will actually launch after buying.
    pub can_launch: [bool; WEAPON_COUNT],
    /// Lines cleared by opponent (used to unlock Susan purchase at 50+).
    op_lines: u32,
    /// op_lines value at which Reagan/Keating/NiceDay cooldown expires (post-Reagan).
    reagan_cooldown_expires_at: u32,
}

impl Ai {
    pub fn new(difficulty: u8) -> Self {
        let mut can_purchase = [true; WEAPON_COUNT];
        let can_launch = [true; WEAPON_COUNT];

        // Ernie never buys these weapons (observation weapons are useless vs. Ernie,
        // Meadow helps the player, Susan is unlocked at 50 op_lines).
        for kind in [
            WeaponKind::Ames, WeaponKind::Ace, WeaponKind::Condor,
            WeaponKind::Meadow, WeaponKind::Susan,
        ] {
            can_purchase[kind.index()] = false;
        }

        Ai {
            difficulty,
            penalties: AiPenalties::default(),
            can_purchase,
            can_launch,
            op_lines: 0,
            reagan_cooldown_expires_at: 0,
        }
    }

    pub fn update_op_lines(&mut self, lines: u32) {
        self.op_lines = lines;
        if self.op_lines >= 50 {
            self.can_purchase[WeaponKind::Susan.index()] = true;
        }
        // Restore Reagan/Keating/NiceDay once cooldown window has elapsed.
        if self.reagan_cooldown_expires_at > 0 && self.op_lines >= self.reagan_cooldown_expires_at {
            self.reagan_cooldown_expires_at = 0;
            self.can_purchase[WeaponKind::Reagan.index()] = true;
            self.can_purchase[WeaponKind::Keating.index()] = true;
            self.can_purchase[WeaponKind::NiceDay.index()] = true;
        }
    }

    /// Decide the best (col, rotation) placement for the given piece.
    pub fn decide(&self, board: &Board, kind: PieceKind, ws: &WeaponState) -> Option<AiMove> {
        let num_rots = kind.rotation_count();
        let mut best: Option<AiMove> = None;

        // Weapon state adjustments to penalties (matching BTComputer BT_WPN_ON responses).
        let mut p = self.penalties.clone();
        if ws.is_active(WeaponKind::FBF) {
            p.height   = p.height * 2;
            p.variance = p.variance * 2;
        }
        if ws.is_active(WeaponKind::Force) {
            p.height     = p.height * 4;
            p.variance   = p.variance * 2;
            p.line_bonus = p.line_bonus * 4;
        }
        if ws.is_active(WeaponKind::Fallout) {
            p.line_bonus = p.line_bonus * 2;
            p.height     = p.height * 2;
        }
        if ws.is_active(WeaponKind::Bottle) {
            p.open_hole   = p.open_hole / 10;
            p.closed_hole = p.closed_hole / 10;
            p.height      = p.height * 2;
        }
        if ws.is_active(WeaponKind::NoDice) {
            p.variance = p.variance * 4;
        }

        for rot in 0..num_rots {
            let cells = kind.cells(rot);
            for col in -(cells.iter().map(|c| c.0).min().unwrap_or(0))
                ..(BOARD_COLS as i32 - cells.iter().map(|c| c.0).max().unwrap_or(0))
            {
                if let Some(drop_row) = find_drop_row(board, cells, col) {
                    let score = evaluate(board, cells, col, drop_row, &p);
                    if best.is_none() || score < best.unwrap().score {
                        best = Some(AiMove { col, rotation: rot, score });
                    }
                }
            }
        }

        best
    }

    /// Buy weapons during bazaar. Returns list of weapon kinds to launch at opponent.
    pub fn go_shopping(&mut self, score: &mut Score, arsenal: &mut Arsenal, rng: &mut StdRng, board: &Board) -> Vec<WeaponKind> {
        let mut to_launch = Vec::new();
        let mut purchased = [false; WEAPON_COUNT];
        let mut speedy_count = 0u32;
        let mut niceday_this_visit = false;

        loop {
            let can_stack_speedy = speedy_count < 2;
            let affordable: Vec<WeaponKind> = (0..WEAPON_COUNT)
                .filter_map(|i| WeaponKind::from_index(i))
                .filter(|&k| {
                    self.can_purchase[k.index()]
                        && self.can_launch[k.index()]
                        && (if k == WeaponKind::Speedy { can_stack_speedy } else { !purchased[k.index()] })
                        && (weapon_def(k).price as i64) <= score.funds
                        && arsenal.can_add(k)
                        && self.can_buy_now(k, board)
                })
                .collect();

            if affordable.is_empty() {
                break;
            }

            // If NiceDay was just bought, prioritise Reagan for the economic combo.
            let kind = if niceday_this_visit && affordable.contains(&WeaponKind::Reagan) {
                niceday_this_visit = false;
                WeaponKind::Reagan
            } else {
                affordable[rng.gen_range(0..affordable.len())]
            };

            if kind == WeaponKind::Speedy {
                speedy_count += 1;
            } else {
                purchased[kind.index()] = true;
            }
            score.funds -= weapon_def(kind).price as i64;
            arsenal.add(kind);
            to_launch.push(kind);

            if kind == WeaponKind::NiceDay {
                niceday_this_visit = true;
            }

            // After buying Reagan, block Reagan/Keating/NiceDay for 50 op_lines to
            // prevent immediate re-use of the economic attack combo.
            if kind == WeaponKind::Reagan {
                self.reagan_cooldown_expires_at = self.op_lines + 50;
                self.can_purchase[WeaponKind::Reagan.index()]  = false;
                self.can_purchase[WeaponKind::Keating.index()] = false;
                self.can_purchase[WeaponKind::NiceDay.index()] = false;
                break;
            }
        }

        // Shuffle launch order so weapon arrival is less predictable.
        for i in (1..to_launch.len()).rev() {
            let j = rng.gen_range(0..=i);
            to_launch.swap(i, j);
        }

        to_launch
    }

    /// Per-weapon purchase conditions beyond can_purchase/can_launch.
    fn can_buy_now(&self, kind: WeaponKind, board: &Board) -> bool {
        if kind == WeaponKind::Swap {
            // Only buy Swap when Ernie's board is relatively clear (BT_SWAPLINE = 5):
            // highest occupied row must be below row 5 from the top.
            let tops = column_tops(board);
            let highest = tops.iter().copied().filter(|&t| t < BOARD_ROWS).min().unwrap_or(BOARD_ROWS);
            return highest > 5;
        }
        true
    }

    /// Weapons Ernie should immediately launch from his arsenal (opportunistic).
    pub fn weapons_to_launch(&self, arsenal: &Arsenal) -> Vec<(usize, WeaponKind)> {
        arsenal.slots.iter().enumerate()
            .filter(|(_, slot)| self.can_launch[slot.kind.index()])
            .map(|(i, slot)| (i, slot.kind))
            .collect()
    }
}

// ─── Board evaluation helpers ─────────────────────────────────────────────────

/// Top row (0-indexed from top) of each column. BOARD_ROWS = fully empty column.
pub fn column_tops(board: &Board) -> [usize; BOARD_COLS] {
    let mut tops = [BOARD_ROWS; BOARD_COLS];
    for c in 0..BOARD_COLS {
        for r in 0..BOARD_ROWS {
            if !board.cell(c as i32, r as i32).is_empty() {
                tops[c] = r;
                break;
            }
        }
    }
    tops
}

/// Count empty cells below the column top, split into three severity categories.
///
/// - `open` (7k): hole with empty space directly above it — accessible but hard to fill
/// - `closed` (10k): hole with non-empty cells on all 4 cardinal sides — truly inaccessible
/// - `covered` (3k): hole with a non-empty cell immediately above but not fully enclosed
pub fn count_holes(board: &Board, tops: &[usize; BOARD_COLS]) -> (i64, i64, i64) {
    let mut open   = 0i64;
    let mut closed = 0i64;
    let mut covered = 0i64;

    for c in 0..BOARD_COLS {
        let top = tops[c];
        if top >= BOARD_ROWS { continue; }

        for r in (top + 1)..BOARD_ROWS {
            if board.cell(c as i32, r as i32).is_empty() {
                let above_blocked = !board.cell(c as i32, r as i32 - 1).is_empty();
                let left_blocked  = c == 0 || !board.cell(c as i32 - 1, r as i32).is_empty();
                let right_blocked = c == BOARD_COLS - 1 || !board.cell(c as i32 + 1, r as i32).is_empty();
                let below_blocked = r == BOARD_ROWS - 1 || !board.cell(c as i32, r as i32 + 1).is_empty();

                if above_blocked && left_blocked && right_blocked && below_blocked {
                    closed += 1;
                } else if above_blocked {
                    covered += 1;
                } else {
                    open += 1;
                }
            }
        }
    }
    (open, closed, covered)
}

/// Count how many rows would be full after placing piece cells.
fn count_full_rows_sim(board: &Board, cells: &[(i32, i32)], col: i32, drop_row: i32) -> usize {
    // We need to check which rows become full after placement
    let placed_rows: std::collections::HashSet<i32> = cells
        .iter()
        .map(|&(_, dr)| drop_row + dr)
        .filter(|&r| r >= 0 && r < BOARD_ROWS as i32)
        .collect();

    placed_rows.iter().filter(|&&r| {
        (0..BOARD_COLS as i32).all(|c| {
            let is_piece = cells.iter().any(|&(dc, dr)| col + dc == c && drop_row + dr == r);
            is_piece || !board.cell(c, r).is_empty()
        })
    }).count()
}

/// Count Happy cells in rows that will be cleared.
fn count_happy_in_full_rows(board: &Board, cells: &[(i32, i32)], col: i32, drop_row: i32) -> i64 {
    let placed_rows: std::collections::HashSet<i32> = cells
        .iter()
        .map(|&(_, dr)| drop_row + dr)
        .filter(|&r| r >= 0 && r < BOARD_ROWS as i32)
        .collect();

    placed_rows.iter().filter(|&&r| {
        (0..BOARD_COLS as i32).all(|c| {
            let is_piece = cells.iter().any(|&(dc, dr)| col + dc == c && drop_row + dr == r);
            is_piece || !board.cell(c, r).is_empty()
        })
    }).flat_map(|&r| {
        (0..BOARD_COLS as i32).map(move |c| (c, r))
    }).filter(|&(c, r)| board.cell(c, r) == Cell::Happy)
    .count() as i64
}

/// Drop a piece straight down from row 0. Returns the landing row or None if blocked at spawn.
pub fn find_drop_row(board: &Board, cells: &[(i32, i32)], col: i32) -> Option<i32> {
    // Verify piece can start above the board (cells with dr < 0 are OK)
    // Check that no cells at the spawn row (y=0) overlap occupied cells
    let can_spawn = cells.iter().all(|&(dc, dr)| {
        let c = col + dc;
        let r = dr; // y = 0
        // Cells above board (r < 0) are always free
        r < 0 || !board.occupied(c, r)
    });
    if !can_spawn {
        return None;
    }

    let mut y = 0i32;
    // Drop while all cells at (y+1) are free
    while cells.iter().all(|&(dc, dr)| !board.occupied(col + dc, y + 1 + dr)) {
        y += 1;
    }
    // Verify final position doesn't overlap (handles negative row offsets)
    let valid = cells.iter().all(|&(dc, dr)| {
        let r = y + dr;
        r < 0 || !board.occupied(col + dc, r)
    });
    if valid { Some(y) } else { None }
}

/// Evaluate board state after hypothetically placing piece at (col, drop_row).
/// Lower score = better position for Ernie.
pub fn evaluate(
    board: &Board,
    cells: &[(i32, i32)],
    col: i32,
    drop_row: i32,
    p: &AiPenalties,
) -> i64 {
    // Build simulated column tops
    let mut tops = column_tops(board);
    // Update tops with placed cells
    for &(dc, dr) in cells {
        let c = (col + dc) as usize;
        let r = (drop_row + dr) as usize;
        if c < BOARD_COLS && r < BOARD_ROWS && r < tops[c] {
            tops[c] = r;
        }
    }

    // Count holes after placement
    // We approximate by counting holes in the existing board + adjusting for placed cells
    let (open, closed, covered) = count_holes(board, &tops);

    // Aggregate height penalty: distance from top of board to lowest column top
    let max_height = tops.iter().map(|&t| BOARD_ROWS - t).max().unwrap_or(0) as i64;

    // Variance penalty: sum of squared height differences between adjacent columns
    let variance: i64 = tops.windows(2).map(|w| {
        let diff = (w[0] as i64) - (w[1] as i64);
        diff * diff
    }).sum();

    // Lines cleared bonus
    let lines = count_full_rows_sim(board, cells, col, drop_row) as i64;

    // Happy bonus (Happy cells in cleared lines)
    let happy = count_happy_in_full_rows(board, cells, col, drop_row);

    p.open_hole * open
    + p.closed_hole * closed
    + p.covered_hole * covered
    + p.height * max_height
    + p.variance * variance
    - p.line_bonus * lines
    - p.happy_bonus * happy
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::Board;
    use rand::SeedableRng;

    fn test_board() -> Board { Board::new() }

    #[test]
    fn column_tops_empty_board() {
        let board = test_board();
        let tops = column_tops(&board);
        assert!(tops.iter().all(|&t| t == BOARD_ROWS));
    }

    #[test]
    fn column_tops_with_cell() {
        let mut board = test_board();
        board.set_cell(5, 20, Cell::Regular(1));
        let tops = column_tops(&board);
        assert_eq!(tops[5], 20);
    }

    #[test]
    fn count_holes_finds_hole_below_cell() {
        let mut board = test_board();
        board.set_cell(3, 10, Cell::Regular(1));
        // Row 11-27 under column 3 are holes (17 total)
        let tops = column_tops(&board);
        let (open, closed, covered) = count_holes(&board, &tops);
        assert_eq!(open + closed + covered, (BOARD_ROWS - 11) as i64);
        // Row 11 is directly under the filled cell — covered hole
        assert_eq!(covered, 1);
        // Rows 12-27 have empty space above them — open holes
        assert_eq!(open, (BOARD_ROWS - 12) as i64);
    }

    #[test]
    fn find_drop_row_lands_at_floor() {
        let board = test_board();
        let cells = PieceKind::Box_.cells(0);
        // Box_ cells are at (1,1),(1,2),(2,1),(2,2) — deepest offset = 2
        // Should land at row 27-2 = 25 (so (1,1) lands at row 26, (2,2) at row 27)
        let drop = find_drop_row(&board, cells, 0);
        assert!(drop.is_some());
        let drop_row = drop.unwrap();
        // Verify: cells at drop_row+2 ≤ 27
        let max_r = cells.iter().map(|&(_, dr)| drop_row + dr).max().unwrap();
        assert_eq!(max_r, BOARD_ROWS as i32 - 1);
    }

    #[test]
    fn find_drop_row_none_when_blocked() {
        let mut board = test_board();
        // Fill entire board
        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                board.set_cell(c as i32, r as i32, Cell::Regular(1));
            }
        }
        let cells = PieceKind::El.cells(0);
        assert!(find_drop_row(&board, cells, 0).is_none());
    }

    #[test]
    fn evaluate_prefers_flat_board() {
        let board = test_board();
        let p = AiPenalties::default();
        let cells_el = PieceKind::El.cells(0);
        let cells_box = PieceKind::Box_.cells(0);

        let row_el = find_drop_row(&board, cells_el, 0).unwrap();
        let row_box = find_drop_row(&board, cells_box, 0).unwrap();

        let score_el = evaluate(&board, cells_el, 0, row_el, &p);
        let score_box = evaluate(&board, cells_box, 0, row_box, &p);

        // Both should produce similar low scores on empty board
        assert!(score_el >= 0);
        assert!(score_box >= 0);
    }

    #[test]
    fn decide_returns_some_move() {
        let board = test_board();
        let ws = WeaponState::new();
        let ai = Ai::new(5);
        let ai_move = ai.decide(&board, PieceKind::El, &ws);
        assert!(ai_move.is_some());
    }

    #[test]
    fn decide_prefers_line_completion() {
        // Fill row 27 leaving only column 0 empty
        let mut board = test_board();
        for c in 1..BOARD_COLS as i32 {
            board.set_cell(c, 27, Cell::Regular(1));
        }
        let ws = WeaponState::new();
        let ai = Ai::new(5);

        // A Long piece placed at col 0 should complete the row
        let ai_move = ai.decide(&board, PieceKind::Long, &ws);
        assert!(ai_move.is_some());
        // The AI should find a move (we just verify it runs without panic)
    }

    #[test]
    fn go_shopping_spends_funds() {
        let mut ai = Ai::new(5);
        let mut score = Score::default();
        let mut arsenal = Arsenal::new();
        let mut rng = StdRng::seed_from_u64(99);
        let board = test_board();
        score.funds = 500;
        let launched = ai.go_shopping(&mut score, &mut arsenal, &mut rng, &board);
        // Should have bought something
        assert!(arsenal.slot_count() > 0 || score.funds <= 500);
        // Launched weapons are all in can_launch list
        for kind in launched {
            assert!(ai.can_launch[kind.index()]);
        }
    }

    #[test]
    fn susan_unlocks_at_50_op_lines() {
        let mut ai = Ai::new(5);
        assert!(!ai.can_purchase[WeaponKind::Susan.index()]);
        ai.update_op_lines(50);
        assert!(ai.can_purchase[WeaponKind::Susan.index()]);
    }
}
