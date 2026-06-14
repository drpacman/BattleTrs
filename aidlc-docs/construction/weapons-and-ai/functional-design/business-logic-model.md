# Business Logic Model — Unit 2: weapons-and-ai

## 1. Weapon Activation Flow

```
player presses weapon slot key (1-0)
  → resolve slot index → look up ArsenalSlot
  → if slot empty: no-op
  → if Mirror active on THIS board:
      if weapon ∈ {Swap, Mondale, Keating, Ames, Ace, Condor, NiceDay, Susan, Mirror}:
          nullify (no effect on either board)
      else:
          apply weapon effect to SELF (weapon is reflected back)
      remove from arsenal; return
  → else:
      arsenal.remove(kind)
      if duration > 0:
          target.weapon_state.remaining[kind.index()] += duration
          emit WeaponOn { kind, duration }
      apply_instant_effect(kind, &mut target_board, &mut source_board, rng)
      emit WeaponLaunched { kind }
```

Mirror is checked on the **target** board's weapon state.  
`apply_instant_effect` runs only for duration=0 weapons (and for duration>0 weapons that also have an instant component such as RiseUp).

---

## 2. Weapon Duration Countdown

Called inside `game_state::tick()` immediately after `check_and_clear_lines()` returns `N > 0`.

```
for each weapon index i in 0..34:
    if weapon_state.remaining[i] == 0: skip
    weapon_state.remaining[i] = remaining[i].saturating_sub(lines_cleared)
    if weapon_state.remaining[i] == 0:
        apply_weapon_deactivate(i, &mut self)
        emit WeaponOff { kind: WeaponKind::from(i) }
```

---

## 3. Weapon Effect Algorithms (all 34)

### 3A — Board-state weapons

**RiseUp** (instant, also triggered by Lawyers each time opponent clears a line):
```
hole_col = rng.gen_range(0..BOARD_COLS)
for row in 1..BOARD_ROWS (reversed, bottom-up):       // shift all rows up
    board.cells[row-1] = board.cells[row]
board.cells[BOARD_ROWS-1] = [Cell::Struct_; BOARD_COLS]
board.cells[BOARD_ROWS-1][hole_col] = Cell::Empty
// if row 0 was occupied before shift → board.topped_out = true
```

**FlipOut** (instant):
```
for row in 0..BOARD_ROWS:
    board.cells[row].reverse()
```

**Swap** (instant):
```
std::mem::swap(&mut player_board.cells, &mut ernie_board.cells)
// each board keeps its own weapon_state; Bottle/Upbyside side-effects also swap
```

**Upbyside** (timed, 10 lines):
- Rendering: draw board rows in reverse order (row BOARD_ROWS-1 at top)
- Input: Left/Right arrows reversed; CW rotation becomes CCW
- Applied via `weapon_state.is_active(Upbyside)` checks in game_loop input handler and renderer

**Fallout** (timed, 10 lines):
- Columns 2-7 (0-indexed) in all rows act as a "black hole"
- On lock: any piece cell that occupies column 2..=7 is NOT written to the board (cells simply vanish)
- Line completions in the fallout zone never count (columns 2-7 cells are always "empty" for line-fill check)
- `Board::occupied()` returns false for columns 2-7 when Fallout is active (pieces pass through)
- Implementation flag: `weapon_state.is_active(Fallout)` checked in `Board::occupied()` and `lock_piece()`

**Bottle** (timed, 10 lines):
- Middle rows (rows 7..=20 in BOARD_ROWS=28 board) narrow to columns 3-6 only
- `Board::occupied()` returns true for columns 0-2 and 7-9 in rows 7-20 when Bottle is active
- Line-clear check in the neck region only counts columns 3-6 as "fillable"
- Pieces cannot physically enter columns 0-2 or 7-9 in rows 7-20

### 3B — Piece disruption weapons

**FearedWeird** (timed, 3 lines):
- `PieceKind::random()` overridden: only return weird pieces (Dog, RDog, Cap, Wall, Tower, Star, WeirdLong)
- Applied via `weapon_state.is_active(FearedWeird)` in `PieceManager::next_piece()`

**FourByFour** (timed, 10 lines):
- When active, `PieceKind::Box` is replaced by a 4×4 hollow box piece (`PieceKind::FourByFour`)
- PieceKind::FourByFour uses piece 17 (BT_4x4_PIECE) rotation table

**Missing** (instant):
```
candidates = [(r,c) for r,c where board.cells[r][c] != Empty]
if candidates.is_empty(): return
(r,c) = candidates[rng.gen_range(0..candidates.len())]
board.cells[r][c] = Cell::Empty
```

**PieceIt** (instant):
```
kind = PieceKind::random_standard(rng)   // standard 7 pieces only
cells = kind.cells(0)
col_offset = rng.gen_range(0..(BOARD_COLS - piece_w))
// place as high on board as possible (drop simulation from row 0)
if placement doesn't overlap existing cells: write cells to board
else: skip (no effect if board is too full)
```

**Bug** (instant): same as PieceIt but the placed cells are tagged `Cell::Bug` (rendered invisible; treated as occupied for collision)

**Broken** (timed, 5 lines):
- `weapon_state.broken_kind` is set to the current active piece's kind when weapon activates
- `PieceManager::next_piece()` returns `broken_kind` instead of a random piece while active
- On deactivation: `broken_kind` cleared

**SoLong** (timed, 10 lines):
- `PieceKind::random()`: re-roll if result is `PieceKind::Long` (I-piece)

**NoDice** (timed, 35 lines):
- `PieceKind::random()`: re-roll if result is `PieceKind::Die { .. }`

**Hatter** (timed, 5 lines):
- On each game tick, if Hatter is active: attempt CW rotation of active piece; if blocked by wall, skip
- Implemented in `game_state::tick()`: `apply_hatter_spin(&mut active_piece, &board)`

**NiceDay** (instant):
- `ernie_game_state.next_piece = PieceKind::Happy`

### 3C — Physics / controls weapons

**Speedy** (timed, 10 lines):
- `drop_interval_ms` halved: `DROP_INTERVAL_MS / 2 = 256ms`
- Applied via `weapon_state.is_active(Speedy)` in drop-timer logic

**Meadow** (timed, 10 lines):
- `drop_interval_ms` doubled: `DROP_INTERVAL_MS * 2 = 1024ms`

**NoSlide** (timed, 10 lines):
- Left/Right arrow inputs ignored when NoSlide is active
- Checked via `weapon_state.is_active(NoSlide)` in input dispatch

**Slick** (timed, 3 lines):
- On each tick a `SlickTick` event fires (every ~150ms, matching `BT_SLICK_TIMEOUT`):
  - try to move active piece in `weapon_state.slick_dir` direction
  - if blocked (wall or cell): reverse `slick_dir` (now drifts the other way)
- Timer added alongside game tick; fires independently of drop timer

**Force** (timed, 5 lines):
- When a line is cleared with Force active, the cleared row becomes all-empty but rows ABOVE it do NOT shift down
- Implementation: in `check_and_clear_lines()`, if Force active: instead of rebuilding board, zero out cleared rows in-place
```
if weapon_state.is_active(Force):
    for full_row in full_rows:
        board.cells[full_row] = [Cell::Empty; BOARD_COLS]
    // do NOT rebuild from non-cleared rows
```

### 3D — Economic weapons

**Mondale** (timed, 50 lines):
- When a player earns funds `F` and Mondale is active on them:
  - player receives `F * 70 / 100`
  - opponent receives `F * 30 / 100`
  - Applied in `Score::add_funds(amount) -> (kept, taxed)`
- Multiple Mondale stacks accumulate in `mondale_rate` (capped at 90% max)

**Keating** (instant):
```
stolen = target.score.funds
target.score.funds = 0
source.score.funds += stolen
emit FundsStolen { amount: stolen }
```

**Carter** (timed, 20 lines):
- `BazaarState.price_mult = 2` when Carter is active on the affected player at bazaar time
- Checked via `weapon_state.is_active(Carter)` when building BazaarState

**Reagan** (instant):
```
if target.score.funds > 0:
    target.score.funds = 0u32.wrapping_sub(target.score.funds)  // goes negative (stored as i64 in Score)
// Note: Score.funds must be i64 for Unit 2 (Reagan makes it negative)
emit FundsNegated
```

Score.funds type changes from `u32` to `i64` in Unit 2.

### 3E — Visibility / intel weapons

**Ames** (timed, 20 lines):
- When active on the LAUNCHER (not target): display opponent's exact funds in player's stats panel
- `view.show_opponent_funds = true` in PlayingView when Ames/Ace/Condor active on self

**Ace** (timed, 30 lines):
- Like Ames; also reveals opponent's board with 80% cell accuracy (20% cells shown wrong colour)
- `view.opponent_reveal_accuracy = 0.8`

**Condor** (timed, 40 lines):
- Like Ace but 100% accurate board reveal
- `view.opponent_reveal_accuracy = 1.0`

**Blind** (instant):
- Select a random 4×6 rectangle on the target's board
- Mark those cells in `weapon_state.blind_cells`; renderer draws them as `Cell::Empty`
- Actual board data unchanged; cells are still occupied for collision
- Blind cells clear when Blind expires (duration=0 means cells stay hidden until next Blind or game over)
  - Actually duration=0 means the cells stay invisible permanently for the rest of that game (authentic)

**Twilight** (instant):
- All currently occupied cells on the target board become `Cell::Twilight` (invisible but occupied)
- New cells placed after Twilight remain visible until a new Twilight is launched
- `Cell::Twilight` renders as `Cell::Empty` in renderer

**Gimp** (instant):
- Overlay a large "GIMP!" flash text on the opponent's board for 2 seconds
- No gameplay effect; purely visual distraction
- Implemented as a timed renderer overlay (`gimp_flash_until: Option<Instant>`)

### 3F — Arsenal meta weapons

**Mirror** (timed, 10 lines):
- When active and opponent launches a weapon against this player:
  - Weapons nullified (no effect on either board): Swap, Mondale, Keating, Ames, Ace, Condor, NiceDay, Susan, Mirror
  - All other weapons: reflected back onto the launcher
- Checked in `weapon_activation_flow` before applying any weapon effect

**Susan** (instant):
```
std::mem::swap(&mut player.arsenal, &mut ernie.arsenal)
emit ArsenalSwapped
```

---

## 4. Bazaar Flow

```
// Triggered when combined_lines crosses a multiple of 20
GamePhase::Playing → GamePhase::InBazaar(BazaarState::new(player_funds, carter_active))

BazaarState::new():
    weapons = sorted WEAPONS[] by price ascending
    price_mult = if carter_active { 2 } else { 1 }

// Ernie simultaneously enters shopping mode
// Ernie's goShopping() runs: randomly buys weapons, plans launch orders

// Player interaction:
Up/Down     → move cursor
Enter       → attempt purchase: if funds >= effective_price(selected) and arsenal not full
                player.score.funds -= effective_price(selected)
                player.arsenal.add(selected)
Esc         → set bazaar_state.player_done = true

// Both done → resume:
when player_done AND ernie_done:
    GamePhase::InBazaar → GamePhase::Playing
    combined_lines_at_last_baz = combined_lines  // reset trigger
```

---

## 5. Ernie AI — Placement Search

Ernie runs on a dedicated `std::thread`. The Ernie task owns an `Ai` and a private `GameState`. It communicates via `mpsc::channel<GameMessage>` with the game-tick thread.

### 5A. Placement Search (ported from BTComputer::decide + checkMove)

For each piece:
```
fn decide(piece: PieceKind, board: &Board, weapon_state: &WeaponState, penalties: &AiPenalties) -> AiMove {
    let mut best = i64::MAX;
    let mut best_move = AiMove::default();

    let n_orientations = piece.num_orientations();
    for orientation in 0..n_orientations {
        let cells = piece.cells(orientation);
        let min_col = cells.iter().map(|&(c,_)| c).min().unwrap_or(0);
        let max_col = cells.iter().map(|&(c,_)| c).max().unwrap_or(0);
        let piece_width = max_col - min_col + 1;
        
        for spawn_col in 0..=(BOARD_COLS as i32 - piece_width) {
            let col = spawn_col - min_col;
            if let Some(drop_row) = find_drop_row(board, &cells, col) {
                let v = evaluate(board, &cells, col, drop_row, weapon_state, penalties);
                if v < best {
                    best = v;
                    best_move = AiMove { col, orientation };
                }
            }
        }
    }
    best_move
}
```

### 5B. Board Evaluation (ported from BTCBoard::eval)

```
fn evaluate(board, cells, col, row, weapon_state, penalties) -> i64 {
    // 1. Simulate: clone board, place piece cells
    let mut sim = board.snapshot();
    for &(dc, dr) in cells {
        let c = (col + dc) as usize;
        let r = (row + dr) as usize;
        if r < BOARD_ROWS { sim.cells[r][c] = Cell::Regular(1); }
    }
    
    // 2. Count lines cleared (unless Force active)
    let lines = if weapon_state.is_active(WeaponKind::Force) { 0 }
                else { count_full_rows(&sim) };
    
    // 3. Measure column tops (row index of topmost occupied cell; BOARD_ROWS if empty)
    let tops = column_tops(&sim);
    let global_top = *tops.iter().min().unwrap_or(&BOARD_ROWS);
    
    // 4. Compute variance (sum of squared adjacent column height differences)
    let variance: i64 = tops.windows(2)
        .map(|w| (w[0] as i64 - w[1] as i64).pow(2))
        .sum();
    
    // 5. Count holes (open, closed, covered)
    let (open_h, closed_h, covered_h) = count_holes(&sim, &tops);
    
    // 6. Happy bonus: if a Happy cell would be cleared by this placement
    let happy = count_happy_in_full_rows(&sim);
    
    // 7. Height penalty: if board top above BT_MIDLINE (row 14)
    let height_pen = if global_top < BT_MIDLINE { penalties.height_penalty } else { 0 };
    
    penalties.open_hole_penalty    * open_h
    + penalties.closed_hole_penalty  * closed_h
    + penalties.covered_hole_penalty * covered_h
    + penalties.variance_penalty     * variance
    + height_pen
    - penalties.line_bonus           * lines as i64
    - penalties.happy_bonus          * happy as i64
}
```

Hole counting rules:
- **Open hole**: empty cell that has ≥ 1 occupied cell directly above it AND ≥ 1 empty adjacent cell (not fully enclosed)
- **Closed hole**: empty cell with occupied cells on all four sides
- **Covered hole**: open or closed hole with ≥ 2 occupied cells in the same column above it

### 5C. Ernie Shopping (ported from BTComputer::goShopping)

Ernie's bazaar strategy (simplified faithful port):
```
fn go_shopping(funds: u32, arsenal: &mut Arsenal, ai: &mut Ai, rng: &mut SmallRng) {
    // Disallow Swap if Ernie's board is already in trouble (top_ > BT_SWAPLINE=5)
    if ernie_board_top > BT_SWAPLINE {
        ai.can_purchase[Susan] = false;
    }
    
    loop {
        // Pick next weapon to buy (random among purchasable)
        let next = ai.next_weapon.unwrap_or_else(|| random_purchasable(ai, rng));
        let price = effective_price(next, ai.carter_active);
        
        if price > funds || arsenal.is_full() { break; }
        
        arsenal.add(next);
        funds -= price;
        ai.schedule_launch(next);  // queue for future launch
        ai.next_weapon = next_in_combo(next, ai);
        
        if combo_complete(ai) { break; }
    }
    ai.ernie_done = true;
}
```

Combo logic: Ernie buys weapon clusters (NiceDay→Reagan, Speedy→Speedy→...) and launches them together after opponent clears a trigger line count.

### 5D. Ernie Weapon Launch (ported from BTComputer::activateCommando)

Checked each time Ernie evaluates a piece:
```
for order in ai.launch_queue:
    if order.trigger_line <= ai.op_lines:
        launch_weapon(order.kind, &mut ernie_arsenal, channel)
        remove order from queue
```

Weapons Ernie never launches: `Hatter`, `FlipOut`, `Speedy` (ignores them).

---

## 6. GameMessage Protocol Extension (Unit 2)

New variants added to `protocol::GameMessage` for weapon + board events:

```
WeaponLaunched { kind: WeaponKind }
WeaponOn       { kind: WeaponKind, duration: u32 }
WeaponOff      { kind: WeaponKind }
RiseUp
BoardFull      { cells: Box<[[Cell; BOARD_COLS]; BOARD_ROWS]> }   // Swap, Condor
FundsStolen    { amount: u32 }
FundsNegated
FundsTax       { rate_pct: u8 }
PieceForced    { kind: PieceKind }      // NiceDay
CellRemoved    { col: usize, row: usize } // Missing
PieceAdded     { col: usize, row: usize, kind: PieceKind } // PieceIt, Bug
ArsenalSwapped { new_slots: Vec<(WeaponKind, u8)> }
BazaarStart
BazaarEnd
OpponentView   { board_accuracy: f32, funds: Option<i64> }   // Ames/Ace/Condor
```

No serde derives yet (Unit 3 adds them).

---

## 7. Combined Lines Trigger

Bazaar triggers when `combined_lines` crosses a multiple of 20 (unchanged from Unit 1).
`Score.add_lines()` already returns a `bool` indicating whether to open the bazaar; Unit 2 acts on that signal by transitioning to `GamePhase::InBazaar`.
