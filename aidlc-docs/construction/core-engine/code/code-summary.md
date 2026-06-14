# Code Summary — Unit 1 (core-engine)

## Files Generated

### battletris-engine (library crate)

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root: re-exports `engine`, `protocol`, `ai` modules |
| `src/engine/mod.rs` | Module re-exports for all engine types |
| `src/engine/board.rs` | `Board` (10×28 cells), `Cell` enum, `LinesCleared`, `BoardSnapshot`. Includes `check_and_clear_lines()` with fund accumulation and HappyMissed conversion. |
| `src/engine/piece.rs` | `PieceKind` (18 variants), all static rotation tables, `ActivePiece` with move/rotate/ghost logic, `PieceKind::random()` (rejection sampling) |
| `src/engine/score.rs` | `Score` and `ScoreView`; hard-drop scoring formula; bazaar-trigger detection |
| `src/engine/game_state.rs` | `GameState` tick loop: input → gravity → lock-delay → spawn → game-over; all phase enums |
| `src/protocol/mod.rs` | `GameMessage` enum (all 17 variants), `ProtocolError`, `GameResult`, `PlayerRecord`; `encode`/`decode` as `todo!()` stubs |
| `src/ai/mod.rs` | `Ai` struct stub with stable `decide()` signature |

### battletris-client (binary crate)

| File | Purpose |
|------|---------|
| `src/main.rs` | SDL2 init, channels (`mpsc`), game-tick thread spawn, event pump loop, key→input mapping |
| `src/game_loop.rs` | Game-tick thread: `run_game_loop()` at ~1 kHz, non-blocking input drain, `RenderEvent` emission |
| `src/renderer/mod.rs` | `Renderer` struct (SDL2 canvas + optional TTF fonts), shared drawing helpers: `draw_board()`, `draw_active_piece()`, `draw_ghost_piece()`, `draw_next_piece()`, `draw_die_pips()`, `draw_face()`, `draw_text()` |
| `src/renderer/title.rs` | `draw_title()` — purple title screen with controls reminder |
| `src/renderer/playing.rs` | `draw_playing()` — player board, ghost, active piece, opponent board (or CPU placeholder), stats panel |
| `src/renderer/game_over.rs` | `draw_game_over()` — overlay with WIN/LOSE, final stats, replay prompt |

### battletris-server (binary crate)

| File | Purpose |
|------|---------|
| `src/main.rs` | Placeholder `main()` — TCP relay implemented in Unit 3 |

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Cell pixel size | 28px | Q1=B; matches BT_BOX_WTH=23 at modern scale |
| Lock delay | 150ms (faithful port) | Q2=A; matches BT_SLIDE_TIME; resets on input |
| Rotation tables | Static const arrays | Q3=A; idiomatic Rust; fully testable |
| Score formula | Hard-drop only | From BTGame.C:729: `score += 28 - y` |
| Funds formula | `raw_sum × line_count` | From BTBoardManager::checkLines() |
| Piece probability | Rejection sampling | Die=1.0, Normal=0.21, Exotic=0.02, Weird=0.0 |
| Channel pattern | `sync_channel(2)` for render, unbounded for input | Decouples 1kHz game tick from 60fps render |
| SDL2 fonts | System font detection + graceful degradation | No bundled font binary needed |
| Windows build | `sdl2 bundled` feature | Single static binary, no DLL runtime |
| GameMessage in Unit 1 | Bare enum, encode/decode = `todo!()` | Q1=B; stable API surface without serde overhead |

## Test Coverage

| Module | Tests | Coverage |
|--------|-------|---------|
| `board.rs` | 7 tests | Wall boundaries, line clear, die/happy funds, HappyMissed, multi-line, row shift, top-out |
| `piece.rs` | 9 tests | Rotation cycles, no-rotate pieces, WeirdLong 6 states, Star 2 states, collision, ghost, CCW cancel, distribution |
| `game_state.rs` | 6 tests | Gravity at 512ms, hard-drop score, lock delay, move resets timer, game-over, bazaar at 20 lines |
| **Total** | **22 tests** (25 with subtests) | All pass with zero warnings |

## Architecture Invariants

- `battletris-engine` has **no I/O or platform dependencies** — pure Rust logic, runs everywhere
- `GameState::apply_network_message()` is a stable stub: signature won't change in Unit 3
- All 18 piece rotation tables are `const` static slices — no allocation on piece selection
- `Board::occupied(col, row < 0)` always returns `true` (faithful ceiling-wall rule from BTBoardManager)
- Hard drop teleports piece to ghost_y and locks immediately (no step-by-step fall in Rust port)
- `PieceKind::random()` uses rejection sampling matching BTPieceManager — weird pieces never appear in normal play
