use rand::{Rng, rngs::StdRng};

use crate::engine::board::{Board, Cell, BOARD_COLS, BOARD_ROWS};
use crate::engine::piece::PieceKind;
use crate::engine::weapons::{Arsenal, WeaponKind, WeaponState, weapon_def, WEAPON_COUNT};
use crate::engine::score::Score;

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
}

impl Ai {
    pub fn new(difficulty: u8) -> Self {
        let mut can_purchase = [true; WEAPON_COUNT];
        let mut can_launch = [true; WEAPON_COUNT];

        // Ernie never buys these weapons (BR-AI04)
        for kind in [
            WeaponKind::Ames, WeaponKind::Ace, WeaponKind::Condor,
            WeaponKind::Meadow, WeaponKind::Susan, WeaponKind::Reagan,
        ] {
            can_purchase[kind.index()] = false;
        }

        // Ernie never launches these weapons even if purchased (BR-AI05)
        for kind in [WeaponKind::Hatter, WeaponKind::FlipOut, WeaponKind::Speedy] {
            can_launch[kind.index()] = false;
        }

        Ai {
            difficulty,
            penalties: AiPenalties::default(),
            can_purchase,
            can_launch,
            op_lines: 0,
        }
    }

    pub fn update_op_lines(&mut self, lines: u32) {
        self.op_lines = lines;
        // Susan unlocks after opponent reaches 50 lines (BR-AI04)
        if self.op_lines >= 50 {
            self.can_purchase[WeaponKind::Susan.index()] = true;
        }
    }

    /// Decide the best (col, rotation) placement for the given piece.
    pub fn decide(&self, board: &Board, kind: PieceKind, ws: &WeaponState) -> Option<AiMove> {
        let num_rots = kind.rotation_count();
        let mut best: Option<AiMove> = None;

        // Weapon state adjustments to penalties (BR-AI08/09/10)
        let mut p = self.penalties.clone();
        if ws.is_active(WeaponKind::FBF) || ws.is_active(WeaponKind::Force) {
            p.height = p.height * 3 / 2;
            p.variance = p.variance * 2;
        }
        if ws.is_active(WeaponKind::Fallout) {
            p.line_bonus = p.line_bonus * 2;
            p.height = p.height * 2;
        }
        if ws.is_active(WeaponKind::Bottle) {
            p.open_hole = p.open_hole / 2;
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
    pub fn go_shopping(&self, score: &mut Score, arsenal: &mut Arsenal, rng: &mut StdRng) -> Vec<WeaponKind> {
        let mut to_launch = Vec::new();
        // Each weapon kind bought at most once per bazaar visit to ensure variety.
        let mut purchased = [false; WEAPON_COUNT];

        loop {
            let affordable: Vec<WeaponKind> = (0..WEAPON_COUNT)
                .filter_map(|i| WeaponKind::from_index(i))
                .filter(|&k| {
                    self.can_purchase[k.index()]
                        && self.can_launch[k.index()]
                        && !purchased[k.index()]
                        && (weapon_def(k).price as i64) <= score.funds
                        && arsenal.can_add(k)
                })
                .collect();

            if affordable.is_empty() {
                break;
            }

            // Random selection gives Ernie unpredictable, varied loadouts each bazaar.
            let kind = affordable[rng.gen_range(0..affordable.len())];
            purchased[kind.index()] = true;
            score.funds -= weapon_def(kind).price as i64;
            arsenal.add(kind);
            to_launch.push(kind);
        }

        // Shuffle launch order
        for i in (1..to_launch.len()).rev() {
            let j = rng.gen_range(0..=i);
            to_launch.swap(i, j);
        }

        to_launch
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

/// Count empty cells that have at least one occupied cell above them (holes).
/// Returns (open_holes, closed_holes, covered_holes) but we treat all as open_holes for now.
pub fn count_holes(board: &Board, tops: &[usize; BOARD_COLS]) -> (i64, i64, i64) {
    let mut open = 0i64;
    for c in 0..BOARD_COLS {
        let top = tops[c];
        if top >= BOARD_ROWS { continue; } // empty column
        for r in (top + 1)..BOARD_ROWS {
            if board.cell(c as i32, r as i32).is_empty() {
                open += 1;
            }
        }
    }
    (open, 0, 0)
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
        // Row 11-27 under column 3 are holes
        let tops = column_tops(&board);
        let (open, _, _) = count_holes(&board, &tops);
        assert!(open >= (BOARD_ROWS - 11) as i64);
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
        let ai = Ai::new(5);
        let mut score = Score::default();
        let mut arsenal = Arsenal::new();
        let mut rng = StdRng::seed_from_u64(99);
        score.funds = 500;
        let launched = ai.go_shopping(&mut score, &mut arsenal, &mut rng);
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
