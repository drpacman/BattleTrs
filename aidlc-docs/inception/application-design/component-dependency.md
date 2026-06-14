# Component Dependencies — BattleTrisRs

## Crate Dependency Graph

```
battletris-client (bin)
  +-- battletris-engine (lib)     [game logic, types, protocol, AI]
  +-- sdl2                        [rendering + input]
  +-- tokio                       [async net client]
  +-- serde / bincode             [via engine re-export]
  +-- clap                        [startup CLI args: --server, --port, --name]

battletris-server (bin)
  +-- battletris-engine (lib)     [protocol types, PlayerRecord, AI for computer mode]
  +-- tokio                       [async relay server]
  +-- serde / serde_json          [player DB persistence]
  +-- bincode                     [via engine re-export for wire protocol]
  +-- clap                        [subcommands: serve, players, show]

battletris-engine (lib)
  +-- serde                       [derive on GameMessage, BoardSnapshot, etc.]
  +-- bincode                     [encode/decode]
  +-- rand                        [piece randomization]
  +-- (no SDL2, no tokio, no I/O)
```

## Dependency Matrix

| Component | depends on | communication |
|-----------|-----------|---------------|
| GameTickService | Engine (owns GameState) | Direct (owned value) |
| GameTickService | AI (calls choose_placement) | Direct (function call in engine) |
| GameTickService | SDL2RenderService | `mpsc::Sender<RenderEvent>` → render thread |
| GameTickService | NetworkClient | `mpsc::Sender<GameMessage>` (outgoing); `mpsc::Receiver<GameMessage>` (incoming) |
| SDL2RenderService | GameTickService | `mpsc::Sender<PlayerInput>` → tick thread |
| SDL2RenderService | Renderer | Direct (owns Renderer) |
| NetworkClient recv_loop | GameTickService | `mpsc::Sender<GameMessage>` → tick thread |
| NetSender | TCP socket | `tokio::io::AsyncWrite` |
| Server Session | Protocol (GameMessage) | read/write over TCP; intercept QueryResult |
| Server | PlayerDb | `Arc<Mutex<PlayerDb>>` shared across session tasks |

## Communication Patterns

### Pattern 1: Channel-based game loop (Q5=C)

```
[SDL2 main thread]                    [game-tick thread]
     |                                      |
     |--mpsc::Sender<PlayerInput>---------->|  (keyboard events)
     |                                      |
     |<--mpsc::Sender<RenderEvent>----------|  (render snapshots, ~60fps)
     |                                      |
     |  [net recv tokio task]               |
     |       |--mpsc::Sender<GameMessage>-->|  (opponent messages)
     |                                      |--mpsc::Sender<GameMessage>--> net send task
```

- **Non-blocking receive on SDL2 thread**: `render_rx.try_recv()` — no stalling the event pump
- **Blocking send from tick thread**: `render_tx.send(event)` — bounded channel; backpressure if SDL2 falls behind
- **Channel capacity**: `mpsc::sync_channel(2)` for RenderEvent (newest frame wins); unbounded for PlayerInput and GameMessage

### Pattern 2: Tokio tasks for networking (Q4=A)

```
GameTickService
  | net_tx: mpsc::Sender<GameMessage>
  v
[net send task]  -- async write --> TCP stream --> server
                                                     |
[net recv task]  <-- async read -- TCP stream <-- server
  | game_tx: mpsc::Sender<GameMessage>
  v
GameTickService
```

Both network tasks are spawned by `NetClient::connect()` on the tokio runtime. The game-tick thread communicates via `std::sync::mpsc` channels bridged into the async world via `blocking` wrappers or a dedicated tokio handle.

### Pattern 3: Server session relay (Q4=A, relay model Q2=B)

```
Client A                Server Session task               Client B
  |                          |                                |
  |--GameMessage(Lobby)----->|--GameMessage(Lobby)---------->|
  |<-GameMessage(Lobby)------|<--GameMessage(Lobby)----------|
  |                          |                                |
  |--GameMessage(Game)------>|--GameMessage(Game)----------->|
  |<-GameMessage(Game)-------|<--GameMessage(Game)-----------|
  |                          |                                |
  |--QueryResult(result)---->|--update_elo()--> PlayerDb     |
```

The session task uses `tokio::io::copy_bidirectional` as the base relay, with message interception for `QueryResult` frames.

## Module Visibility Rules

```
battletris-engine/src/
  lib.rs         — pub use engine::*, pub use protocol::*, pub use ai::*
  engine/
    board.rs     — pub (Board, BoardSnapshot, CellColor, LinesCleared)
    piece.rs     — pub (PieceKind, ActivePiece, Placement)
    weapons.rs   — pub (WeaponKind, WeaponDef, WeaponState, Arsenal, BazaarState, ArsenalSlot)
    game_state.rs — pub (GameState, GamePhase, GameEvent, GameMode, PlayerInput)
    score.rs     — pub (Score, ScoreView)
  protocol/
    mod.rs       — pub (GameMessage, GameResult, PlayerRecord, encode, decode, ProtocolError)
  ai/
    mod.rs       — pub (Ai, AiDifficulty)

battletris-client/src/
  main.rs        — start SDL2 thread + tokio runtime + game-tick thread
  renderer/
    mod.rs       — pub (Renderer, RenderEvent, PlayingView, BazaarView, LobbyView, GameOverView)
  net/
    mod.rs       — pub (NetClient, NetSender, recv_loop)
  game_loop.rs   — game-tick thread entry point (private)

battletris-server/src/
  main.rs        — clap CLI dispatch
  server.rs      — run_server, run_session (pub(crate))
  db.rs          — PlayerDb (pub(crate))
  elo.rs         — compute_elo_delta (pub(crate))
```

## Key Design Constraints

1. **Engine has zero I/O dependencies** — no SDL2, no tokio, no sockets; testable with `cargo test` on any platform without SDL2 installed
2. **SDL2 stays on the main thread** — macOS and Windows both require SDL2 event loop on the thread that created the window; game-tick thread must not touch SDL2
3. **GameState is never shared** — owned exclusively by game-tick thread; all inter-thread communication is via channels carrying cheap snapshots (`RenderEvent` view structs, `GameMessage` enums)
4. **Server has no SDL2 dependency** — `battletris-server` can be compiled for a headless Linux host if needed (one player's Mac or Windows box acts as the relay server)
5. **Protocol types in engine** — `GameMessage` and `PlayerRecord` live in `battletris-engine::protocol` so both `battletris-client` and `battletris-server` import them without duplication
