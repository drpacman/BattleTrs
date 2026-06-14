# Code Generation Plan — Unit 1: core-engine

## Unit Context

**Unit**: core-engine  
**Deliverable**: Single-player Tetris playable in SDL2 window + `GameMessage` bare enum skeleton  
**Workspace root**: `/Users/paulcaporn/workspace/BattleTrisRs`  
**Project type**: Greenfield Rust (Cargo workspace)  
**Stories covered**: U1-C01 through U1-C15  

## Dependencies

- **battletris-engine** crate: none external except `serde`, `bincode`, `rand`
- **battletris-client** crate: `battletris-engine`, `sdl2`, `clap`
- **battletris-server** crate: placeholder only in Unit 1

## Step Sequence

---

### Step 1: Workspace Cargo.toml
- [x] Create `/Users/paulcaporn/workspace/BattleTrisRs/Cargo.toml`
- [x] Declare workspace members: `battletris-engine`, `battletris-client`, `battletris-server`
- [x] Set `resolver = "2"`, `edition = "2021"`
- [x] Add shared workspace dependencies with versions: `serde`, `bincode`, `rand`, `clap`, `sdl2`
- *Stories: project scaffold for all U1 stories*

---

### Step 2: battletris-engine/Cargo.toml
- [x] Create `battletris-engine/Cargo.toml`
- [x] `[lib]` crate type
- [x] Dependencies: `serde = { workspace = true, features = ["derive"] }`, `bincode = { workspace = true }`, `rand = { workspace = true }`
- *Stories: U1-C13 (protocol skeleton), engine foundation*

---

### Step 3: battletris-engine/src/engine/board.rs
- [x] `Cell` enum: `Empty, Regular(u8), Die(u8), Happy, HappyMissed, Struct_, Gimp(i32)`
- [x] `Cell::is_occupied()`, `Cell::is_removable()`, `Cell::fund_value() -> i32`
- [x] `Board` struct: `cells: [[Cell; 10]; 28]`, `upside_down: bool`
- [x] `Board::new()` initializes all cells to `Empty`
- [x] `Board::occupied(x: i32, y: i32) -> bool` — boundary + cell check
- [x] `Board::get(x, y)` / `Board::set(x, y, cell)`
- [x] `Board::check_and_clear_lines() -> LinesCleared` — full algorithm from business-logic-model.md §4
- [x] `Board::snapshot() -> BoardSnapshot`
- [x] `Board::flip_horizontal()` / `Board::flip_vertical()` — stub (panic! "Unit 2")
- [x] `Board::insert_line(hole_col)` / `Board::remove_partial_line()` — stub (panic! "Unit 2")
- [x] `LinesCleared` struct: `count: u32, funds_earned: i32, happy_missed: bool`
- [x] `BoardSnapshot` struct: `width: u8, height: u8, rep: Vec<u8>` with encoding from domain-entities.md
- *Stories: U1-C05 (line clear), U1-C07 (die funds), U1-C08 (happy funds), U1-C09 (funds display)*

---

### Step 4: battletris-engine/src/engine/board_tests.rs (inline #[cfg(test)])
- [x] `test_occupied_boundary` — walls and floor return occupied
- [x] `test_line_clear_basic` — fill row 27, check_and_clear clears it and shifts
- [x] `test_line_clear_funds_die` — line with two die cells (pips=3, pips=4), expects funds=7×1=7
- [x] `test_line_clear_funds_happy` — line with happy cell, expects funds=150
- [x] `test_happy_missed` — happy cell in non-full row converts to HappyMissed
- [x] `test_multi_line_clear` — clear 3 lines simultaneously, funds multiplied by 3
- *Stories: U1-C15 (unit tests)*

---

### Step 5: battletris-engine/src/engine/piece.rs
- [x] `PieceKind` enum — 18 variants as defined in domain-entities.md
- [x] Per-piece static cell offset tables (const arrays) for all rotations:
  - El: 4 rotations (3×3 generic CW)
  - RevEl: 4 rotations
  - SldLft: 4 rotations
  - SldRt: 4 rotations
  - Long: 2 effective rotations (horizontal / vertical; 4×4 grid)
  - Plug: 4 rotations
  - Dog: 4 rotations
  - RevDog: 4 rotations
  - Cap: 4 rotations (4×4 generic CW)
  - Tower: 4 rotations
  - Wall: 4 static states (hand-coded from BTWallPiece::rotate)
  - Star: 2 static states (plus ↔ cross, from BTStarPiece::rotate)
  - WeirdLong: 6 static states (diagonal morph, from BTWeirdLongPiece::rotate)
  - Box_: 1 rotation (2×2 square)
  - FourByFour: 1 rotation (hollow 4×4 frame)
  - Die: 1 rotation (1×1 at offset (1,1))
  - Happy: 1 rotation (1×1 at offset (1,1))
  - LongDong: 1 rotation (8 cells horizontal)
- [x] `PieceKind::cells(rotation: u8) -> &'static [(i32, i32)]`
- [x] `PieceKind::num_rotations() -> u8`
- [x] `PieceKind::spawn_x_offset() -> i32` — `bounding_box_width / 2` per piece
- [x] `PieceKind::color_id() -> u8` — maps to BT color constant (1–8)
- [x] `PieceKind::is_die() -> Option<u8>`
- [x] `PieceKind::is_happy() -> bool`
- [x] `PieceKind::random(rng, weapon_state) -> PieceKind` — rejection sampling from business-logic-model.md §1
- [x] `ActivePiece` struct: `kind, x: i32, y: i32, rotation: u8`
- [x] `ActivePiece::cells() -> impl Iterator<Item=(i32,i32)>` — adds (x,y) to each offset
- [x] `ActivePiece::can_move_to(board, dx, dy) -> bool`
- [x] `ActivePiece::can_rotate(board, clockwise) -> bool`
- [x] `ActivePiece::rotated(clockwise) -> ActivePiece` — returns new piece with updated rotation
- [x] `Placement` struct: `x: i32, rotation: u8`
- *Stories: U1-C02 (pieces fall), U1-C03 (move/rotate), U1-C04 (lock)*

---

### Step 6: battletris-engine/src/engine/piece_tests.rs (inline #[cfg(test)])
- [x] `test_el_rotation_0` — cells match expected offsets
- [x] `test_el_rotation_cycle` — 4 CW rotations return to original
- [x] `test_long_two_states` — Long has 2 distinct shapes
- [x] `test_longdong_no_rotate` — LongDong num_rotations=1
- [x] `test_die_spawn_offset` — Die spawns at (5,0), 1×1 at (6,1)
- [x] `test_piece_can_move_to_wall` — piece at x=0 cannot move left
- [x] `test_piece_can_move_to_floor` — piece at y=27 cannot move down
- [x] `test_rotation_blocked` — can_rotate returns false if cell occupied
- [x] `test_piece_random_distribution` — 10000 samples, weird pieces never appear, die appears ~6× more than normal pieces
- *Stories: U1-C15 (unit tests)*

---

### Step 7: battletris-engine/src/engine/score.rs
- [x] `Score` struct: `score: u32, lines: u32, funds: i32, op_score: u32, op_lines: u32, op_funds: i32`
- [x] `Score::default()` — all zeros
- [x] `Score::add_hard_drop_score(y_at_drop_start: i32)` — `+= 28 - y`
- [x] `Score::add_funds(amount: i32)` — `funds += amount`
- [x] `Score::add_lines(n: u32)` — `lines += n`
- [x] `ScoreView` struct: `score, lines, funds, op_score, op_lines, lines_until_bazaar: u32`
- [x] `Score::to_view(op_lines: u32) -> ScoreView`
- *Stories: U1-C06 (score), U1-C07-C09 (funds tracking)*

---

### Step 8: battletris-engine/src/engine/game_state.rs
- [x] `GamePhase` enum (7 variants from application-design.md)
- [x] `GameMode` enum: `SinglePlayer, VsComputer, NetworkGame`
- [x] `PieceState` enum: `Dropping, LockDelay, HardDropping`
- [x] `PlayerInput` enum (7 variants from domain-entities.md)
- [x] `GameEvent` enum (all variants from domain-entities.md)
- [x] `GameState` struct — all fields from domain-entities.md
- [x] `GameState::new(mode: GameMode, seed: u64) -> GameState` — initializes board, spawns first piece, phase=Title
- [x] `GameState::tick(input: Option<PlayerInput>, elapsed_ms: u32) -> Vec<GameEvent>` — full algorithm from business-logic-model.md §2 and §3:
  1. Process input (move/rotate/hard drop)
  2. Advance drop_elapsed_ms
  3. Apply gravity step if interval elapsed
  4. Handle lock delay countdown
  5. Lock piece when lock_delay expires
  6. Spawn next piece; check game over
  7. Update lines_until_bazaar
  8. Return events
- [x] `GameState::apply_network_message(msg: GameMessage) -> Vec<GameEvent>` — returns `vec![]` stub
- [x] `GameState::hard_drop_score_at(y: i32) -> u32` — `(Board::HEIGHT - y) as u32`
- *Stories: U1-C02 through U1-C10, U1-C13*

---

### Step 9: battletris-engine/src/engine/game_state_tests.rs (inline #[cfg(test)])
- [x] `test_tick_gravity` — tick advances piece by 1 row every 512ms
- [x] `test_tick_hard_drop_scores` — hard drop from y=0 adds 28 to score
- [x] `test_lock_delay` — piece at floor, 149ms tick → still not locked; 1ms more → locked
- [x] `test_lock_delay_reset_on_move` — move during lock delay resets the 150ms timer
- [x] `test_game_over` — fill board to top, tick spawns piece → GameOver event emitted
- [x] `test_bazaar_trigger_single_player` — clear 20 lines → BazaarTriggered event
- *Stories: U1-C15 (unit tests)*

---

### Step 10: battletris-engine/src/protocol/mod.rs
- [x] `GameMessage` enum — all variants declared, no `#[derive(Serialize, Deserialize)]` yet:
  - Lobby: `Hello { name: String }`, `Welcome`, `GameStart`, `Quit`
  - Board sync: `BoardUpdate(BoardSnapshot)`, `ScoreUpdate(ScoreView)`
  - Piece events: `PieceLocked`, `LinesCleared { count: u32, funds: i32 }`
  - Weapons (Unit 2–3): `WeaponFired(u8)`, `BazaarStart`, `BazaarDone`, `WeaponBought(u8)`
  - Admin: `QueryResult(GameResult)`, `PlayerInfo(PlayerRecord)`
  - Game over: `GameOver { won: bool }`
- [x] `GameResult` struct: `winner: String, loser: String, winner_elo_delta: i32`
- [x] `PlayerRecord` struct: `name: String, elo: i32, wins: u32, losses: u32`
- [x] `ProtocolError` enum: `InsufficientData, InvalidFrame, DecodeError(String)`
- [x] `encode(msg: &GameMessage) -> Vec<u8>` — `todo!("Unit 3")`
- [x] `decode(buf: &[u8]) -> Result<GameMessage, ProtocolError>` — `todo!("Unit 3")`
- *Stories: U1-C13 (GameMessage skeleton stable)*

---

### Step 11: battletris-engine/src/ai/mod.rs
- [x] `AiDifficulty` enum: `Easy, Medium, Hard`
- [x] `Ai` struct: `difficulty: AiDifficulty`
- [x] `Ai::new(difficulty) -> Ai`
- [x] `Ai::choose_placement(board, piece, weapon_state) -> Placement` — stub: returns `Placement { x: Board::WIDTH/2, rotation: 0 }`
- [x] `Ai::evaluate_board(board) -> f32` — stub: returns 0.0
- [x] `Ai::choose_bazaar_purchases(available, funds, score) -> Vec<WeaponKind>` — stub: returns `vec![]`
- *Stories: U1-C13 (Ai stub for stable API)*

---

### Step 12: battletris-engine/src/lib.rs and engine/mod.rs
- [x] `battletris-engine/src/lib.rs` — `pub mod engine; pub mod protocol; pub mod ai; pub use engine::*; pub use protocol::*; pub use ai::*;`
- [x] `battletris-engine/src/engine/mod.rs` — re-exports: `Board, Cell, BoardSnapshot, LinesCleared, PieceKind, ActivePiece, Placement, GameState, GamePhase, GameMode, PieceState, PlayerInput, GameEvent, Score, ScoreView`
- [x] Verify `cargo check -p battletris-engine` passes
- *Stories: all U1 engine stories*

---

### Step 13: battletris-client/Cargo.toml
- [x] `[[bin]]` crate type
- [x] Dependencies: `battletris-engine = { path = "../battletris-engine" }`, `sdl2 = { workspace = true, features = ["bundled"] }`, `clap = { workspace = true, features = ["derive"] }`
- *Stories: client scaffold*

---

### Step 14: battletris-client/src/renderer/mod.rs and render types
- [x] `RenderEvent` enum: `Title, Playing(PlayingView), GameOver { won: bool, score: u32, lines: u32 }`
- [x] `PlayingView` struct: `player_board: BoardSnapshot, active_piece: Option<(PieceKind, Vec<(i32,i32)>)>, ghost_cells: Vec<(i32,i32)>, next_piece: PieceKind, score: ScoreView, opponent_board: Option<BoardSnapshot>`
- [x] `Renderer` struct: owns `sdl2::render::Canvas<sdl2::video::Window>`, `sdl2::ttf::Font`
- [x] `Renderer::new(sdl2_ctx) -> Result<Renderer>`
- [x] `Renderer::render(&mut self, event: &RenderEvent)`
- *Stories: U1-C01 (title screen), U1-C11 (next piece preview)*

---

### Step 15: battletris-client/src/renderer/title.rs
- [x] `draw_title(canvas: &mut Canvas<Window>)` — BATTLETRIS text centered, start prompt
- *Stories: U1-C01 (title screen)*

---

### Step 16: battletris-client/src/renderer/playing.rs
- [x] `draw_playing(canvas: &mut Canvas<Window>, view: &PlayingView)`
- [x] `draw_board(canvas, snapshot, origin_x, origin_y)` — iterates all cells, maps `Cell` encoding to RGB
- [x] `draw_active_piece(canvas, cells, kind)` — draws piece on top of board
- [x] `draw_ghost_piece(canvas, cells, kind)` — 30% opacity version at hard-drop landing
- [x] `draw_next_piece(canvas, kind, origin)` — 4×4 preview at 14px cell size
- [x] `draw_stats(canvas, score_view)` — score, lines, funds, lines-until-bazaar
- [x] `draw_die_cell(canvas, x, y, pips)` — white background + pip dots pattern
- [x] `draw_happy_cell(canvas, x, y, missed)` — white/gray background + face
- *Stories: U1-C02 to U1-C10, U1-C11, U1-C12*

---

### Step 17: battletris-client/src/renderer/game_over.rs
- [x] `draw_game_over(canvas, won: bool, score: u32, lines: u32)` — overlay with result + stats
- *Stories: U1-C10 (game over)*

---

### Step 18: battletris-client/src/game_loop.rs
- [x] `run_game_loop(render_tx: SyncSender<RenderEvent>, input_rx: Receiver<PlayerInput>)` — entry point for game-tick thread
- [x] Creates `GameState::new(GameMode::SinglePlayer, seed)`
- [x] Tick loop at ~60fps (16ms sleep per iteration):
  1. Try-receive all pending `PlayerInput` from `input_rx`
  2. Call `game_state.tick(last_input, elapsed_ms)`
  3. On `GameEvent::PieceLocked` or board change: build `PlayingView` and send `RenderEvent::Playing(view)` via `render_tx`
  4. On `GameEvent::GameOver`: send `RenderEvent::GameOver { ... }`
- [x] `compute_ghost_y(board, piece) -> i32` — finds landing row for ghost piece
- [x] `build_playing_view(state) -> PlayingView` — constructs view from game state
- *Stories: U1-C02 to U1-C10, U1-C12 (opponent board blank)*

---

### Step 19: battletris-client/src/main.rs
- [x] `#[derive(Parser)] struct Args` — `--vs-computer` flag (Unit 2), `--server`, `--port`, `--name` (Unit 3; present but ignored in Unit 1)
- [x] SDL2 init: create window (820×860), canvas, event pump, ttf context
- [x] `std::sync::mpsc::sync_channel::<RenderEvent>(2)` for render channel
- [x] `std::sync::mpsc::channel::<PlayerInput>()` for input channel
- [x] Spawn game-tick thread: `std::thread::spawn(move || run_game_loop(render_tx, input_rx))`
- [x] SDL2 event loop (main thread):
  - Poll events → translate keyboard to `PlayerInput` → send via `input_tx`
  - `try_recv()` on `render_rx` → call `renderer.render(&event)`
  - `canvas.present()`
  - `std::thread::sleep(Duration::from_millis(1))`
- *Stories: U1-C01 through U1-C10 (end-to-end playability)*

---

### Step 20: battletris-server/Cargo.toml and src/main.rs (placeholder)
- [x] `battletris-server/Cargo.toml` — minimal: `[[bin]]`, no deps beyond `battletris-engine`
- [x] `battletris-server/src/main.rs` — `fn main() { println!("battletris-server: not yet implemented (Unit 3)"); }`
- [x] Verify `cargo check -p battletris-server` passes
- *Stories: server placeholder that compiles (required by U1 exit criteria)*

---

### Step 21: Cross-platform Windows build check
- [x] Document Windows cross-compile command in `aidlc-docs/construction/core-engine/code/build-notes.md`
- [x] Command: `cargo build --target x86_64-pc-windows-gnu -p battletris-client` (requires `mingw-w64` toolchain)
- [x] If cross-compile toolchain not available locally: document CI step instead
- *Stories: U1-C14 (Windows cross-compile)*

---

### Step 22: Code documentation summary
- [x] Create `aidlc-docs/construction/core-engine/code/code-summary.md`
- [x] List all files created with one-line description
- [x] Note key design decisions implemented
- [x] Note any deviations from functional design (if any)
- *Documentation artifact*

---

## Story Traceability

| Story | Step(s) |
|-------|---------|
| U1-C01 Title screen | Step 15, 19 |
| U1-C02 Piece fall | Step 5, 8, 18, 19 |
| U1-C03 Move/rotate | Step 5, 8, 18, 19 |
| U1-C04 Piece lock | Step 5, 8, 18 |
| U1-C05 Line clear | Step 3, 8 |
| U1-C06 Score | Step 7, 8 |
| U1-C07 Die funds | Step 3, 7 |
| U1-C08 Happy funds | Step 3, 7 |
| U1-C09 Funds display | Step 7, 16 |
| U1-C10 Game over | Step 8, 17, 19 |
| U1-C11 Next piece | Step 14, 16 |
| U1-C12 Opponent board placeholder | Step 14, 18 |
| U1-C13 GameMessage skeleton | Step 10, 11 |
| U1-C14 Windows cross-compile | Step 21 |
| U1-C15 Unit tests | Step 4, 6, 9 |

## Total: 22 steps across 3 crates
