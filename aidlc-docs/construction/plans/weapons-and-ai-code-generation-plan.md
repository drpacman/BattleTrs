# Code Generation Plan — Unit 2: weapons-and-ai

## Architecture Summary

**Two-board channel pattern (Q3=B):**
- `game_loop` (tick thread) owns player's `GameState`
- `ernie` task (dedicated thread) owns Ernie's `GameState`
- `to_ernie: Sender<GameMessage>` — game_loop → ernie (weapon effects, bazaar events)
- `from_ernie: Receiver<GameMessage>` — ernie → game_loop (board snapshots, weapon launches, game over)
- Unit 3 will replace the ernie task with TCP send/recv tasks — no game_loop restructuring needed

**Key type change:** `Score.funds: u32 → i64` (Reagan Era makes funds negative)

---

## Step Checklist

### battletris-engine changes

- [x] **Step 1** — Extend `Cell` enum + `Board` for weapon geometry
  - Add `Cell::Bug` and `Cell::Twilight` variants to `board.rs`
  - Extend `Board::occupied()` to return true for Fallout zone (cols 2-7) and Bottle zone (cols 0-2, 7-9 in rows 7-20) when respective weapons active; add `weapon_flags: u64` bitmask arg or pass `&WeaponState`
  - Design decision: pass `Option<&WeaponState>` to `occupied()`; `None` = no weapon effects (used in tests)

- [x] **Step 2** — Add Board weapon-effect methods to `board.rs`
  - `Board::rise_up(rng)` — shift all rows up, insert junk row at bottom with random hole
  - `Board::flip_out()` — reverse each row horizontally
  - `Board::force_clear(full_rows)` — zero cleared rows in-place without row rebuild
  - `Board::remove_random_cell(rng) -> bool` — find random non-empty cell, clear it
  - `Board::add_piece_at(cells: &[(i32,i32)], col: i32, row: i32, cell_type: Cell) -> bool` — place piece cells, return false if any overlap
  - `Board::apply_twilight()` — convert all non-empty cells to `Cell::Twilight`

- [x] **Step 3** — Create `battletris-engine/src/engine/weapons.rs` — WeaponKind + WeaponDef
  - `pub enum WeaponKind` (34 variants, repr(usize) for array indexing)
  - `impl WeaponKind { pub fn index(self) -> usize }`
  - `pub struct WeaponDef { kind, name, description, price: u32, duration: u32 }`
  - `pub static WEAPONS: [WeaponDef; 34]` — all 34 entries from btweapons.db + btweaponsp.db
  - `pub fn weapon_def(kind: WeaponKind) -> &'static WeaponDef`

- [x] **Step 4** — Arsenal + WeaponState in `weapons.rs`
  - `pub struct ArsenalSlot { kind: WeaponKind, quantity: u8 }`
  - `pub struct Arsenal { slots: Vec<ArsenalSlot> }` + `impl`: `add`, `remove`, `get`, `is_full`, `slot_count`, `clear`
  - `pub struct WeaponState` with fields from domain-entities.md
  - `impl WeaponState`: `new`, `is_active(kind)`, `activate(kind, duration)`, `tick_lines(n) -> Vec<WeaponKind>` (returns expired weapons), `mondale_rate() -> u8`

- [x] **Step 5** — Instant weapon effects in `weapons.rs`
  - `pub fn apply_weapon_instant(kind: WeaponKind, target: &mut Board, target_ws: &mut WeaponState, source_funds: &mut i64, target_funds: &mut i64, target_next_piece: &mut PieceKind, source_arsenal: &mut Arsenal, target_arsenal: &mut Arsenal, rng: &mut SmallRng)`
  - Covers all duration=0 weapons: RiseUp, FlipOut, Swap (board ref pair), Missing, PieceIt, Bug, Keating, Reagan, NiceDay, Susan, Blind, Twilight, Gimp
  - `swap_boards(a: &mut Board, b: &mut Board)` helper

- [x] **Step 6** — Timed weapon activate/deactivate handlers in `weapons.rs`
  - `pub fn apply_weapon_on(kind: WeaponKind, ws: &mut WeaponState, penalties: Option<&mut AiPenalties>)`
  - `pub fn apply_weapon_off(kind: WeaponKind, ws: &mut WeaponState, penalties: Option<&mut AiPenalties>)`
  - Adjusts `broken_kind`, `slick_dir`, `mondale_rate`, Ai penalty multipliers
  - Mirror check wrapper: `pub fn check_mirror_and_launch(kind, target_ws, source_board, target_board, ...) -> bool` — returns `true` if weapon was nullified or reflected

- [x] **Step 7** — Extend `score.rs`
  - Change `funds: u32` → `funds: i64`
  - Add `op_score: u32, op_lines: u32` fields
  - `Score::add_funds(amount: u32, mondale_rate: u8) -> (i64, i64)` — returns (kept, taxed)
  - Extend `ScoreView`: add `op_score`, `op_lines`, `ernie_funds: Option<i64>`, change `funds: i64`

- [x] **Step 8** — Extend `game_state.rs` — weapons + bazaar
  - Add `weapon_state: WeaponState` to `GameState`
  - Add `GameMode::VsComputer` variant
  - Add `GamePhase::InBazaar(BazaarState)` variant
  - `BazaarState` struct + impl: `new(funds, carter_active)`, `try_buy(&mut player_score, &mut arsenal) -> bool`, `navigate_up/down`
  - Add `PlayerInput::WeaponSlot(u8)` (slots 1-10), `PlayerInput::BazaarUp`, `BazaarDown`, `BazaarBuy`, `BazaarExit`
  - `GameState::launch_weapon(slot: u8, target: &mut GameState/channel, rng)` — dispatches via weapon activation flow

- [x] **Step 9** — Extend `piece.rs` — weapon-filtered piece selection
  - `PieceKind::random_filtered(ws: &WeaponState, rng: &mut SmallRng) -> PieceKind`
  - Handles: FearedWeird (weird-only), FBF (Box→FourByFour), SoLong (reroll Long), NoDice (reroll Die), Broken (repeat broken_kind), NiceDay (inject Happy next)
  - `PieceKind::num_orientations(self) -> usize` — needed for AI exhaustive search

- [x] **Step 10** — Extend `game_state::tick()` for active weapon behaviours
  - Hatter spin: if `weapon_state.is_active(Hatter)`, attempt CW rotate on each tick
  - Slick drift: separate 150ms sub-timer; if `is_active(Slick)`, move piece in `slick_dir`; reverse on wall
  - Lawyers: when `is_active(Lawyers)` and a `LawliersRiseUp { lines }` event received from opponent, call `board.rise_up()` that many times
  - Upbyside: flip input direction mapping when active
  - Duration countdown: call `weapon_state.tick_lines(n)` after every line clear, generate WeaponOff messages for expired weapons

- [x] **Step 11** — Extend `protocol/mod.rs` with Unit 2 GameMessage variants
  - Add 12 variants with real field data (see business-logic-model.md §6)
  - Keep `encode`/`decode` as `todo!("Unit 3")`

- [x] **Step 12** — Implement board evaluation helpers in `ai/mod.rs`
  - `fn column_tops(board: &Board) -> [usize; BOARD_COLS]` — row of topmost occupied cell per column
  - `fn count_full_rows_sim(board: &Board) -> usize` — count full rows without mutating
  - `fn count_holes(board: &Board, tops: &[usize; BOARD_COLS]) -> (i64, i64, i64)` — (open, closed, covered)
  - `fn count_happy_in_full_rows(board: &Board) -> i64`
  - `fn simulate_place(board: &Board, cells: &[(i32,i32)], col: i32, row: i32) -> Board` — clone + place
  - `fn find_drop_row(board: &Board, cells: &[(i32,i32)], col: i32, ws: &WeaponState) -> Option<i32>`

- [x] **Step 13** — Implement `Ai::evaluate()` and `Ai::decide()` in `ai/mod.rs`
  - `AiPenalties` struct with default values from BTComputer.C
  - `Ai::evaluate(board, cells, col, row, ws) -> i64`
  - `Ai::decide(board: &Board, kind: PieceKind, ws: &WeaponState) -> AiMove`
  - Full implementation of exhaustive orientation × column search
  - Weapon state adjustments: no_tetri flag when FallOut/FBF/Force/etc active (match BTComputer.C)

- [x] **Step 14** — Implement `Ai::go_shopping()` and weapon launch logic in `ai/mod.rs`
  - `Ai::go_shopping(funds: &mut i64, arsenal: &mut Arsenal, rng: &mut SmallRng) -> Vec<WeaponKind>` — returns weapons to launch
  - `Ai::should_launch(op_lines: u32) -> Vec<WeaponKind>` — check commando queue
  - `Ai::update_can_purchase(op_lines: u32)` — enable Susan at op_lines >= 50, etc.
  - Weapon bans at init: Ames, Ace, Condor, Meadow, Susan, Reagan = false
  - Weapons Ernie skips on launch: Hatter, FlipOut, Speedy

- [x] **Step 15** — Unit tests in `battletris-engine`
  - `weapons::tests`: activation sets remaining, duration countdown, mirror nullification, mirror reflection, Arsenal add/remove/stack/full
  - `board::tests`: rise_up inserts junk row, rise_up topped_out detection, flip_out reverses cells, force_clear leaves rows in-place, remove_random_cell clears one cell
  - `ai::tests`: find_drop_row places at floor, evaluate prefers flat boards, decide picks column that completes a line, go_shopping spends funds, column_tops/count_holes correct
  - `score::tests`: mondale_rate=30 keeps 70%, keating transfers funds, reagan negates

---

### battletris-client changes

- [x] **Step 16** — Create `battletris-client/src/renderer/bazaar.rs`
  - `pub fn draw_bazaar(r: &mut Renderer, state: &BazaarStateView)`
  - Full-window overlay: title bar, scrollable 34-weapon list, description box, controls bar
  - Colour-coded affordability, selected row highlight, scroll offset logic
  - `BazaarStateView` = a render-safe struct passed from game_loop (no Arc/Mutex)

- [x] **Step 17** — Extend `renderer/playing.rs`
  - Update `PlayingView` fields for weapon chips, arsenal slots, effects flags
  - `draw_active_weapon_chips(canvas, weapons, origin_x, y)` — chips below each board
  - Extend `draw_stats()` to render real arsenal slot names (scale=1)
  - Add `draw_board_with_effects()` replacing `draw_board()` for weapon visual effects
  - Upbyside row-inversion rendering, Twilight grey-out, Gimp flash overlay
  - Opponent board accuracy rendering (Ace: 80% cell reveal)

- [x] **Step 18** — Create `battletris-client/src/ernie.rs` — Ernie task
  - `pub fn run_ernie(to_game: Sender<GameMessage>, from_game: Receiver<GameMessage>)`
  - Owns `GameState` for Ernie, `Ai` instance, `SmallRng`
  - Main loop: sleep 750ms (think interval), then `ai.decide()`, apply move, tick Ernie's GameState
  - Receives inbound GameMessage and dispatches: WeaponOn/WeaponLaunched apply to Ernie's game_state, BazaarStart triggers go_shopping, BazaarEnd resumes play
  - Sends outbound: `BoardState` after each piece placed, `WeaponLaunched` when Ernie fires, `BazaarEnd` after shopping, `GameOver` when Ernie's board fills

- [x] **Step 19** — Extend `game_loop.rs`
  - Accept optional `(Sender<GameMessage>, Receiver<GameMessage>)` for Ernie channel
  - Add weapon slot key handling in `keycode_to_input()` (keys 1-9, 0)
  - Add bazaar navigation key handling
  - After tick: drain `from_ernie` channel, apply Ernie events (board update, weapons, game over)
  - Forward player weapon launches to `to_ernie` channel
  - Bazaar phase: route bazaar input, send `BazaarStart` to Ernie, wait for both done
  - Build combined `PlayingView` with `ernie_board_snapshot` (latest received)

- [x] **Step 20** — Extend `main.rs`
  - Parse `--vs-computer` flag from `std::env::args()`
  - If present: create Ernie channel pair, spawn `std::thread::spawn(|| ernie::run_ernie(...))`, pass channels to game_loop
  - If absent: single-player mode (existing Unit 1 behaviour)
  - Add `mod ernie;` declaration

- [x] **Step 21** — Documentation: `aidlc-docs/construction/weapons-and-ai/code/`
  - `build-notes.md` — note that `--vs-computer` enables vs-Ernie mode
  - `code-summary.md` — architecture diagram, test coverage table

- [x] **Step 22** — Full build + test verification
  - `cargo test -p battletris-engine` passes (target: ≥ 40 tests including Unit 1's 25)
  - `cargo build -p battletris-client` passes, 0 errors, 0 warnings
  - `cargo run -p battletris-client -- --vs-computer` launches two-board game vs Ernie

---

## File Manifest

```
battletris-engine/src/
  engine/
    board.rs               (extended: Cell::Bug/Twilight, weapon geometry, board effects)
    piece.rs               (extended: random_filtered, num_orientations)
    score.rs               (extended: i64 funds, Mondale, op stats)
    game_state.rs          (extended: WeaponState, InBazaar phase, weapon tick logic)
    weapons.rs             (NEW: WeaponKind, WeaponDef, WEAPONS const, Arsenal, WeaponState impl, instant/timed effects)
    mod.rs                 (re-export weapons)
  ai/
    mod.rs                 (full Ai: evaluate, decide, go_shopping, launch logic)
  protocol/
    mod.rs                 (extended: 12 new GameMessage variants)

battletris-client/src/
  main.rs                  (extended: --vs-computer flag, Ernie thread spawn)
  game_loop.rs             (extended: weapon keys, Ernie channel, bazaar phase)
  ernie.rs                 (NEW: Ernie task — owns GameState, AI loop, channel dispatch)
  renderer/
    playing.rs             (extended: weapon chips, effects, real arsenal, combined view)
    bazaar.rs              (NEW: scrollable bazaar screen)
    mod.rs                 (re-export bazaar)
```
