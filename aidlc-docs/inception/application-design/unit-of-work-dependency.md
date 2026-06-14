# Unit-of-Work Dependencies — BattleTrisRs

## Dependency Chain

Units are strictly sequential. Each unit produces artifacts that the next unit consumes.

```
[Unit 1: core-engine] --> [Unit 2: weapons-and-ai] --> [Unit 3: network-and-db]
```

---

## Unit 1: core-engine

### Entry Criteria (all must be true before starting)
- Cargo workspace `Cargo.toml` does not yet exist
- (No prior units — this is the first unit)

### Produces (exit artifacts consumed by later units)

| Artifact | Consumed By |
|----------|-------------|
| `battletris-engine` crate (Board, PieceKind, GameState, Score) | Unit 2, Unit 3 |
| `GameMessage` bare enum in `protocol/mod.rs` | Unit 2 (channel messages), Unit 3 (serialization) |
| `GameState::apply_network_message` stub signature | Unit 3 (implements body) |
| `Ai` stub (random placement) | Unit 2 (replaces with real AI) |
| `battletris-client` channel architecture (game-tick thread + SDL2 main thread) | Unit 2 (adds Ernie task), Unit 3 (adds net tasks) |
| `battletris-server/main.rs` placeholder (compiles) | Unit 3 (replaces with full impl) |
| Windows cross-compile verification | Unit 3 (validates again with full game) |

### Exit Criteria (all must be true before Unit 2 begins)
- [ ] `cargo run -p battletris-client` opens SDL2 window; player can complete a single-player Tetris game
- [ ] Die piece fund accumulation and happy piece (150 funds) visible in UI
- [ ] `cargo test -p battletris-engine` passes (board, collision, line clear, die/happy detection)
- [ ] `cargo build -p battletris-client --target x86_64-pc-windows-gnu` (or equivalent) succeeds
- [ ] All Unit 1 source files committed

---

## Unit 2: weapons-and-ai

### Entry Criteria (all must be true before starting)
- Unit 1 exit criteria all satisfied
- `battletris-engine` lib provides: `Board`, `PieceKind`, `GameState`, `GamePhase`, `Score`, `GameMessage` bare enum
- `battletris-client` provides: channel-based game loop with `PlayerInput` and `RenderEvent` channels

### Produces (exit artifacts consumed by later units)

| Artifact | Consumed By |
|----------|-------------|
| `WeaponKind` (34 variants) + `WeaponDef` compiled-in data | Unit 3 (passed over network in GameMessage) |
| `WeaponState`, `Arsenal`, `BazaarState` in `GameState` | Unit 3 (serialized as part of game state sync) |
| `apply_weapon_activate/deactivate/on_line_cleared` match dispatch | Unit 3 (weapon effects triggered by network messages) |
| `Ai` (full Ernie implementation) | Unit 3 (server can run AI for single-player vs-computer over network, optional) |
| `GameMessage` fields populated (weapon events, board sync) | Unit 3 (adds serde + encode/decode) |
| Ernie task + `GameMessage` channel pattern in `game_loop.rs` | Unit 3 (Ernie task replaced by net send/recv tasks) |
| `renderer::bazaar` | Unit 3 (bazaar still used in networked game) |

### Exit Criteria (all must be true before Unit 3 begins)
- [ ] `cargo run -p battletris-client -- --vs-computer` shows two boards; full game playable vs Ernie
- [ ] Bazaar triggers at correct interval (every 20 combined lines); weapons purchasable
- [ ] All 34 weapon effects apply to the correct board
- [ ] Ernie makes legal moves; purchases weapons from bazaar; provides real opposition
- [ ] `cargo test -p battletris-engine` passes (weapon application, bazaar trigger, AI placement legality, board evaluation)
- [ ] All Unit 2 source files committed

---

## Unit 3: network-and-db

### Entry Criteria (all must be true before starting)
- Unit 2 exit criteria all satisfied
- `battletris-engine::protocol::GameMessage` enum fully populated (all variant fields defined)
- `game_loop.rs` Ernie task uses `GameMessage` channels (ready to swap for net tasks)
- `battletris-server/main.rs` compiles (placeholder)

### Produces (final deliverable)

| Artifact | Role |
|----------|------|
| `protocol::encode` / `protocol::decode` | Wire framing for all TCP communication |
| `battletris-client::net` (NetClient, NetSender, recv_loop) | Client TCP connection to relay server |
| `battletris-server` (run_server, run_session) | Relay server pairing two clients |
| `db::PlayerDb` | Persistent player record storage (JSON flat file) |
| `elo::compute_elo_delta` / `update_elo` | ELO rating updates after each ranked game |
| btref CLI (serve / players / show) | Server management tool |

### Exit Criteria (project complete)
- [ ] Server starts: `cargo run -p battletris-server -- serve --port 7000`
- [ ] Two client machines connect and play a full networked two-player game
- [ ] All 34 weapons work correctly in the networked game
- [ ] Bazaar works correctly in the networked game
- [ ] ELO ratings update after game; `show <name>` reflects updated rating
- [ ] `cargo test -p battletris-engine` passes (protocol encode/decode round-trip)
- [ ] `cargo test -p battletris-server` passes (ELO delta calculation, PlayerDb load/save)
- [ ] `cargo build --target x86_64-pc-windows-gnu` succeeds for the full workspace

---

## Cross-Unit Constraints

| Constraint | Details |
|------------|---------|
| `battletris-engine` has zero I/O deps | No SDL2, tokio, or file I/O in the engine crate at any point across all 3 units |
| SDL2 stays on main thread | Enforced across all 3 units; game-tick thread never calls SDL2 functions |
| `GameState` is never shared across threads | Owned exclusively by game-tick thread; all cross-thread communication via channels |
| `GameMessage` enum is the single protocol boundary | All inter-board communication (local or network) uses `GameMessage`; introduced in Unit 1, fully serializable in Unit 3 |
| Windows builds at each unit boundary | Validated at Unit 1 exit; confirmed again at Unit 3 exit |
