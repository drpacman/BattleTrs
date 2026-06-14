# Unit of Work — BattleTrisRs

## Overview

Three sequential units deliver BattleTrisRs. Each unit is a complete vertical slice: design → code → test → playable milestone. Unit N must reach its deliverable before Unit N+1 begins.

| Unit | Name | Deliverable |
|------|------|-------------|
| 1 | `core-engine` | Single-player Tetris playable in SDL2 window |
| 2 | `weapons-and-ai` | Local two-board game vs Ernie with all 34 weapons |
| 3 | `network-and-db` | Full two-player LAN game with relay server and ELO |

---

## Unit 1: core-engine

### Goals

Establish the Cargo workspace and implement the foundational game engine. By the end of Unit 1, a human player can play a complete game of single-player Tetris in an SDL2 window on macOS, and the Windows cross-compilation target builds cleanly.

### Scope

**`battletris-engine` (lib) — modules created:**
- `engine::board` — `Board` (10×28 grid), `BoardSnapshot`, `CellColor`, `LinesCleared`
- `engine::piece` — `PieceKind` (18 variants), rotation tables for all variants, `ActivePiece`, `Placement`
- `engine::game_state` — `GameState`, `GamePhase`, `PlayerInput`, `GameEvent`, `GameMode`
- `engine::score` — `Score`, `ScoreView`
- `engine::mod` — re-exports
- `protocol::mod` — `GameMessage` bare enum (all variants declared, no serde derives yet); `GameState::apply_network_message` stub (panics or returns empty vec)

**`battletris-client` (bin) — modules created:**
- `renderer::title` — title/splash screen
- `renderer::playing` — active game: player board, score panel, funds counter, next-piece preview; opponent board shown as blank placeholder
- `renderer::game_over` — win/lose screen stub
- `game_loop` — game-tick thread: owns `GameState`, drives ~60 fps tick, drains `PlayerInput` from SDL2, sends `RenderEvent` to SDL2
- `main` — spawns game-tick thread; runs SDL2 event pump on main thread; wires channels

**`battletris-server` (bin) — placeholder only:**
- `main.rs` compiles and prints "not yet implemented"; no logic

### Key Design Decisions Applied

- **Q1=B**: `protocol/mod.rs` declares `GameMessage` enum with all variants. No `#[derive(Serialize, Deserialize)]`. `encode`/`decode` functions are `todo!()`. This pins `apply_network_message`'s signature for the remainder of development.
- **Q2=A**: After SDL2 skeleton works on macOS, verify `cargo build --target x86_64-pc-windows-gnu` (or equivalent cross-compile target) before declaring Unit 1 complete.
- Channel-based loop (Q5=C): SDL2 on main thread, game-tick on dedicated thread.

### Acceptance Criteria

1. `cargo run -p battletris-client` opens an SDL2 window
2. Arrow keys move and rotate the active piece; space hard-drops
3. Lines clear and disappear; score increments
4. `Die { pips }` pieces earn `pips` funds on clear; `Happy` pieces earn 150 funds; funds counter visible in UI
5. Board fills → game over screen shown
6. `cargo test -p battletris-engine` passes (board collision, line clearing, rotation, die/happy detection)
7. `cargo build -p battletris-client --target x86_64-pc-windows-gnu` (or Windows CI) succeeds

### Files Created (Unit 1)

```
Cargo.toml                                         (workspace)
battletris-engine/
  Cargo.toml
  src/
    lib.rs
    engine/
      mod.rs
      board.rs
      piece.rs
      game_state.rs
      score.rs
    protocol/
      mod.rs                                       (GameMessage bare enum + stubs)
    ai/
      mod.rs                                       (Ai stub: random placement)
battletris-client/
  Cargo.toml
  src/
    main.rs
    game_loop.rs
    renderer/
      mod.rs
      title.rs
      playing.rs
      game_over.rs
battletris-server/
  Cargo.toml
  src/
    main.rs                                        (placeholder)
```

---

## Unit 2: weapons-and-ai

### Goals

Extend the engine with all 34 weapons, the bazaar system, and Ernie AI. By the end of Unit 2, a human player can play a full local game against Ernie with the bazaar, weapon purchasing, and weapon effects.

### Scope

**`battletris-engine` — modules extended/added:**
- `engine::weapons` — `WeaponKind` (34 variants), `WeaponDef` (compiled-in definitions from btweapons.db data), `WeaponState`, `Arsenal`, `ArsenalSlot`, `BazaarState`, `apply_weapon_activate`, `apply_weapon_deactivate`, `apply_weapon_on_line_cleared`
- `engine::game_state` — extended: weapon triggers, bazaar transition (every 20 combined lines), `GamePhase::InBazaar`
- `ai::mod` — `Ai` (exhaustive placement search, `evaluate_board`, `choose_bazaar_purchases`, `AiDifficulty`)
- `protocol::mod` — `GameMessage` variants populated with real field data for weapon and board events used in the local vs-Ernie channel (no serde yet)

**`battletris-client` — modules extended/added:**
- `renderer::playing` — extended: opponent board filled (Ernie), weapon slots display, active weapon effects visualised
- `renderer::bazaar` — bazaar screen: weapon list, prices, available funds, purchase UI
- `game_loop` — extended: Ernie task spawned (`std::thread`); `GameMessage` passed over `std::sync::mpsc` channels between tick thread and Ernie task (Q3=B)
- `main` — extended: `--vs-computer` CLI flag selects local vs-Ernie mode

### Key Design Decisions Applied

- **Q3=B**: Local vs-Ernie uses the same `GameMessage` channel pattern as networking will in Unit 3. The game-tick thread sends `GameMessage` over `mpsc::Sender<GameMessage>` to an Ernie task; the Ernie task applies the message to Ernie's `GameState`, runs AI, and sends responses back. Unit 3 replaces the Ernie task with a TCP connection — no game-loop restructuring required.
- All 34 weapon definitions compiled in as `const WeaponDef` arrays (Q8=A); no external file.

### Acceptance Criteria

1. `cargo run -p battletris-client -- --vs-computer` shows two boards: human (left) and Ernie (right)
2. Bazaar triggers after every 20 combined lines cleared; player can buy weapons with funds
3. All 34 weapons purchasable; all weapon effects apply to the correct board
4. Ernie makes legal moves, clears lines, purchases weapons from bazaar
5. Game ends when either board fills; game-over screen shows winner
6. `cargo test -p battletris-engine` passes (weapon application, bazaar trigger, AI placement legality)

### Files Created/Extended (Unit 2)

```
battletris-engine/src/engine/
  weapons.rs                                       (new)
  game_state.rs                                    (extended)
battletris-engine/src/ai/
  mod.rs                                           (full Ai impl)
battletris-engine/src/protocol/
  mod.rs                                           (GameMessage fields populated)
battletris-client/src/
  game_loop.rs                                     (extended: Ernie task + GameMessage channels)
  main.rs                                          (extended: --vs-computer flag)
  renderer/
    playing.rs                                     (extended: opponent board + weapons UI)
    bazaar.rs                                      (new)
```

---

## Unit 3: network-and-db

### Goals

Add the relay server, TCP networking, player database, and ELO system. By the end of Unit 3, two machines on the same LAN can play a full game with ELO tracking.

### Scope

**`battletris-engine::protocol` — fully implemented:**
- Add `#[derive(Serialize, Deserialize)]` to `GameMessage` and all nested types
- Implement `encode(msg: &GameMessage) -> Vec<u8>` (4-byte big-endian length prefix + bincode payload)
- Implement `decode(buf: &[u8]) -> Result<GameMessage, ProtocolError>`
- Add `PlayerRecord`, `GameResult` types

**`battletris-client` — network client added:**
- `net::mod` — `NetClient::connect(addr)`, `NetSender`, `recv_loop` (tokio tasks)
- `renderer::lobby` — "Connecting to server…" and "Waiting for opponent…" screens
- `game_loop` — extended: Ernie task replaced by `NetClient`; `GamePhase::ConnectingToServer` and `GamePhase::WaitingForOpponent` handled
- `main` — extended: `--server`, `--port`, `--name` CLI args; selects network vs local mode

**`battletris-server` — fully implemented:**
- `server::run_server` — tokio `TcpListener`; pairs two clients; spawns `run_session` task per pair
- `server::run_session` — relays `GameMessage` bidirectionally; intercepts `QueryResult` frames for ELO update
- `db::PlayerDb` — `HashMap<String, PlayerRecord>` backed by JSON flat file; `Arc<Mutex<PlayerDb>>` shared across sessions
- `elo::compute_elo_delta` — standard ELO formula, K=32, starting rating 1200
- `main` — clap subcommands: `serve --port <N>`, `players` (list all), `show <name>` (detail one player)

### Key Design Decisions Applied

- TCP framing: 4-byte big-endian length prefix followed by bincode-encoded `GameMessage`. Both client and server use the same `protocol::encode`/`decode`.
- The game-tick thread communicates with the net tasks via `std::sync::mpsc` channels bridged into tokio via `tokio::task::spawn_blocking` or a dedicated tokio handle on the client.
- The channel pattern established in Unit 2 (tick thread ↔ Ernie task via `GameMessage` channels) makes Unit 3 a near-drop-in swap: the Ernie task is replaced by net send/recv tasks.

### Acceptance Criteria

1. Machine A: `cargo run -p battletris-server -- serve --port 7000`
2. Machine B + Mac: `cargo run -p battletris-client -- --server <IP> --port 7000 --name Alice` and `--name Bob`
3. Both clients connect, reach "Waiting for opponent", then enter the game when both are connected
4. Full two-player game plays correctly over LAN (moves, weapons, bazaar, game over)
5. ELO updates after game; `show Alice` and `show Bob` reflect new ratings
6. `cargo test -p battletris-engine` passes (protocol encode/decode round-trip)
7. `cargo test -p battletris-server` passes (ELO delta calculation)

### Files Created/Extended (Unit 3)

```
battletris-engine/src/protocol/
  mod.rs                                           (full impl: serde derives, encode, decode, PlayerRecord, GameResult)
battletris-client/src/
  main.rs                                          (extended: network CLI args)
  game_loop.rs                                     (extended: NetClient integration)
  net/
    mod.rs                                         (new: NetClient, NetSender, recv_loop)
  renderer/
    lobby.rs                                       (new: connecting + waiting screens)
    game_over.rs                                   (extended: show ELO delta)
battletris-server/src/
  main.rs                                          (full clap CLI)
  server.rs                                        (run_server, run_session)
  db.rs                                            (PlayerDb)
  elo.rs                                           (compute_elo_delta, update_elo)
```
