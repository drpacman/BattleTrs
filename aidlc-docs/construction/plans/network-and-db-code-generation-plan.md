# Code Generation Plan — Unit 3: network-and-db

## Unit Context

**Stories**: U3-C01 through U3-C18  
**Depends on**: Unit 2 complete (engine, weapons, GameMessage channels, client rendering)  
**Workspace root**: /Users/paulcaporn/workspace/BattleTrisRs

## Key Architectural Decisions (from Functional Design)

| Decision | Choice |
|----------|--------|
| Mode selection | Q1=B — title screen menu (1=vs Ernie, 2=Network) |
| Bazaar sync | Q2=B — server broadcasts BazaarOpen (not client-side threshold) |
| Disconnection | Q3=C — 15s reconnect window, then GameVoid (no ELO) |
| Board sync | Q4=A — BoardUpdate on piece lock only |
| Duplicate names | Q5=A — reject with NameTaken |

## Architecture Change Summary

`main.rs` becomes an **AppState machine** (`TitleMenu` → `ConnectionScreen` → `InGame` → back).  
Game threads are **spawned per-game** (not once at startup).  
`ErnieChannels` is renamed `PeerChannels` — game_loop is agnostic to whether the peer is Ernie or a network connection.

---

## Plan Checkboxes

### Step 1 — Protocol: add serde, new variants, encode/decode
- [x] Add `#[derive(Serialize, Deserialize)]` to `GameMessage` in `battletris-engine/src/protocol/mod.rs`
- [x] Add `use serde::{Serialize, Deserialize};`
- [x] Add new `GameMessage` variants: `GameStart`, `BazaarOpen`, `PeerDisconnected`, `PeerReconnected`, `GameVoid`, `NameTaken`
- [x] Rename `Hello.player_name` → `Hello.name`; change `Welcome` to `Welcome { assigned_name: String }`
- [x] Extend `GameOver` with `winner_name: String, elo_delta_winner: i32, elo_delta_loser: i32`
- [x] Update `PlayerRecord`: remove `player_id`, add `draws: u32`, add serde derives
- [x] Implement `encode(msg: &GameMessage) -> Result<Vec<u8>, ProtocolError>` — 4-byte BE length + bincode payload
- [x] Implement `decode(buf: &[u8]) -> Result<GameMessage, ProtocolError>` — check length prefix, bincode deserialize
- [x] Add `NeedMoreData` variant to `ProtocolError`
- [x] Add unit tests: encode/decode round-trip for `BoardUpdate`, `WeaponLaunched`, `GameOver`, `BazaarOpen`

### Step 2 — Engine: suppress local bazaar in VsNetwork mode
- [x] In `battletris-engine/src/engine/game_state.rs` bazaar trigger block: wrap with `&& self.mode != GameMode::VsNetwork`
- [x] Verify existing bazaar test still passes (SinglePlayer mode unaffected)

### Step 3 — Server Cargo.toml
- [x] Add to `battletris-server/Cargo.toml` dependencies: `serde`, `bincode`, `serde_json`, `clap`

### Step 4 — Server: `battletris-server/src/elo.rs`
- [x] Create file
- [x] `pub fn compute_elo_delta(winner_elo: i32, loser_elo: i32) -> (i32, i32)` — K=32, zero-sum formula, floor 100
- [x] Unit tests: delta sign, equal players, strong-beats-weak, floor enforcement

### Step 5 — Server: `battletris-server/src/db.rs`
- [x] Create file with PlayerDb, load/save/get_or_create/apply_result/all_sorted/get
- [x] Unit tests: get_or_create, apply_result, save/reload round-trip

### Step 6 — Server: `battletris-server/src/session.rs`
- [x] Create file with run_session, relay loop, BazaarOpen intercept, GameOver intercept (with correct winner=peer), disconnect→PeerDisconnected→15s→GameVoid

### Step 7 — Server: `battletris-server/src/server.rs`
- [x] Create file with run_server, handle_client, Hello/Welcome/NameTaken handshake, pending/pairing logic

### Step 8 — Server: rewrite `battletris-server/src/main.rs`
- [x] Rewrite with clap: serve/players/show subcommands

### Step 9 — Client Cargo.toml
- [x] Add `tokio = { workspace = true }` to `battletris-client/Cargo.toml`

### Step 10 — Client: `battletris-client/src/net/mod.rs`
- [x] Create `src/net/mod.rs` with blocking BufReader/BufWriter IO
- [x] NetChannels, ConnectError, connect() function
- [x] Background recv thread (server → game_loop) and send thread (game_loop → server)

### Step 11 — Client renderer: `battletris-client/src/renderer/lobby.rs`
- [x] Create file with draw_connection_screen, draw_connecting_screen, draw_waiting_screen

### Step 12 — Client renderer: extend `battletris-client/src/renderer/title.rs`
- [x] Made "N - NETWORK GAME" entry active (white text)

### Step 13 — Client renderer: extend `battletris-client/src/renderer/game_over.rs`
- [x] Added winner_name and elo_delta parameters to draw_game_over
- [x] Renders ELO delta in green/red when network game

### Step 14 — Client: `RenderEvent` and `game_loop.rs` changes
- [x] Renamed `ErnieChannels` → `PeerChannels`; fields `from_ernie/to_ernie` → `from_peer/to_peer`
- [x] Added `is_network: bool` to PeerChannels, `player_name: Option<String>` to run_game_loop
- [x] BazaarOpen, GameVoid, PeerDisconnected, PeerReconnected, GameStart message handling
- [x] Network game over: client sends GameOver when board tops out; server responds enriched
- [x] Added winner_name and elo_delta to RenderEvent::GameOver

### Step 15 — Client: rewrite `battletris-client/src/main.rs`
- [x] AppState machine: TitleMenu, ConnectionScreen, Connecting, WaitingForOpponent, InGame
- [x] Enter → spawn Ernie game; N → ConnectionScreen
- [x] Background connect thread; Connecting state polls result
- [x] WaitingForOpponent polls for GameStart, then spawns network game_loop
- [x] Scancode-to-char keyboard input for connection screen fields
- [x] Quit-confirm overlay still works in InGame state

### Step 16 — Client renderer: overlay for peer disconnect
- [x] Added `peer_disconnected: bool` to `PlayingView` in `game_state.rs`
- [x] game_loop sets `view.peer_disconnected` when PeerDisconnected received
- [x] playing.rs renders "OPPONENT DISCONNECTED" overlay when flag set

### Step 17 — Protocol unit tests (`battletris-engine`)
- [x] Round-trip tests for BoardUpdate, WeaponLaunched, GameOver (with ELO), BazaarOpen
- [x] Truncated buffer returns NeedMoreData

### Step 18 — Server unit tests (`battletris-server`)
- [x] ELO: sign check, equal ≈16, strong-beats-weak small gain, floor clamped
- [x] PlayerDb: get_or_create, apply_result, save/reload round-trip

### Step 19 — Build and verify
- [x] `cargo build --workspace` — zero errors, zero warnings
- [x] `cargo test --workspace` — 83 tests pass (76 engine + 7 server)
- [ ] Manual smoke-test: `cargo run -p battletris-server -- serve --port 7000` + two `cargo run -p battletris-client` windows
- [ ] `cargo run -p battletris-server -- players` shows registered players

---

## Story Coverage

| Story | Step |
|-------|------|
| U3-C01 Server starts with serve --port | Step 8 |
| U3-C02 Client connects by IP:port CLI args | Step 10, 15 |
| U3-C03 Connecting screen | Step 11, 15 |
| U3-C04 Waiting for opponent screen | Step 11, 15 |
| U3-C05 Game starts when both connected | Step 6, 14 |
| U3-C06 Board sync to opponent | Step 1 (protocol), Step 6 (relay) |
| U3-C07 Weapons over network | Step 6 (relay) |
| U3-C08 Bazaar sync between clients | Step 6 (BazaarOpen), Step 14 |
| U3-C09 Game over both clients | Step 6, 14 |
| U3-C10 ELO updates K=32 | Step 4, 6 |
| U3-C11 players command | Step 8 |
| U3-C12 show command | Step 8 |
| U3-C13 Records persist | Step 5 |
| U3-C14 Auto-create player | Step 5, 7 |
| U3-C15 encode/decode round-trips | Step 1, 17 |
| U3-C16 ELO delta tested | Step 4, 18 |
| U3-C17 Windows build | Step 19 |
| U3-C18 LAN playability | Step 19 |
