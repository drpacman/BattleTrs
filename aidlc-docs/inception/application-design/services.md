# Services — BattleTrisRs

## Overview

Three services orchestrate the game's runtime behaviour. They replace the original Xt event loop (C++) with a channel-based multi-thread/task architecture in Rust.

---

## Service 1: GameTickService

**Location**: `battletris-client` — dedicated thread (spawned at game start)
**Purpose**: Owns `GameState` and runs the game simulation. The single authority on all game logic. Communicates with the SDL2 thread via `RenderEvent` channel and with the network via `GameMessage` channels.

**Responsibilities**:
- Own the `GameState` struct exclusively (no shared mutable access)
- Drive the game loop at a configurable tick rate (default: 60 ticks/second, ~16ms per tick)
- On each tick:
  1. Drain incoming `PlayerInput` events from the SDL2 thread (via `mpsc`)
  2. Drain incoming `GameMessage` values from `NetReceiver` (via `mpsc`)
  3. Call `GameState::tick(input, elapsed_ms)` → receive `Vec<GameEvent>`
  4. Apply each `GameEvent` (update score, trigger bazaar, record landed piece, etc.)
  5. Convert current state into a `RenderEvent` and send to SDL2 thread
  6. Send any outgoing `GameMessage` values to `NetSender` (opponent sync)
- Manage piece-drop timing: track `ticks_since_drop`; drop piece when threshold reached
- Manage weapon durations: notify `WeaponState` of each line cleared
- Drive bazaar entry/exit synchronization with the opponent (wait for `GameMessage::StartBazaar` / `EndBazaar`)
- For AI mode: call `Ai::choose_placement()` after each piece spawn; schedule AI move sequence

**Key channels**:
```
SDL2 thread  --[PlayerInput]--> GameTickService
GameTickService --[RenderEvent]--> SDL2 thread
Network task --[GameMessage]--> GameTickService
GameTickService --[GameMessage]--> Network task (via NetSender)
```

---

## Service 2: SDL2RenderService

**Location**: `battletris-client` — main thread (SDL2 must live on the main thread on macOS/Windows)
**Purpose**: Owns the SDL2 context. Handles window events and keyboard input; renders each frame when a new `RenderEvent` arrives.

**Responsibilities**:
- Initialize SDL2, create window and canvas, load fonts (SDL2_ttf)
- Run the SDL2 event pump in a loop
- On each iteration:
  1. Poll SDL2 events → translate keyboard events to `PlayerInput` → send to GameTickService
  2. Check for `Quit` event (close window or Escape) → send `PlayerInput::Quit` → break loop
  3. Try-receive the latest `RenderEvent` from GameTickService (non-blocking)
  4. If a new `RenderEvent` arrived: call `Renderer::render(event)`, then `canvas.present()`
- Handle window resize / minimize gracefully
- Apply active weapon overlays (BLIND black screen, UPBYSIDE flip)

**Key inputs/outputs**:
```
Window events → PlayerInput → GameTickService
RenderEvent  ← GameTickService
```

---

## Service 3: RelayServerService

**Location**: `battletris-server` — main tokio runtime
**Purpose**: Relay server that pairs two TCP clients and routes `GameMessage` traffic between them. Manages the player database and ELO updates.

**Responsibilities**:

### Lobby management
- Accept incoming TCP connections (tokio `TcpListener::accept` loop)
- Maintain a `pending: Option<(TcpStream, String)>` for the first client to arrive
- When a second client connects: spawn a `Session` task and clear `pending`
- If a client requests computer play (single-player vs AI): create a local `AiClient` task as the second participant

### Session relay
- Spawn a `run_session(a, b, db)` tokio task per pair
- Read `GameMessage` from client A, write to client B (and vice versa), concurrently
- Intercept `GameMessage::QueryResult(result)` from either client → update ELO in `PlayerDb` → write to disk
- On either client disconnect: send `GameMessage::GameOver` to the other; close session

### Player database service
- Shared `Arc<Mutex<PlayerDb>>` across all session tasks
- `PlayerDb` is a `HashMap<String, PlayerRecord>` backed by a JSON file
- On `update_elo(result)`: compute new ELO ratings using K=32 formula; write updated records to disk atomically (write to `.tmp`, then rename)
- `PlayerList` query: respond with current snapshot of all records

### CLI subcommands (synchronous, blocking)
- `battletris-server players` — load DB, print sorted player table (rank, name, ELO, W/L)
- `battletris-server show <username>` — print one player's full record

---

## Service Interaction Diagram

```
+---------------------------+     PlayerInput      +---------------------+
|   SDL2RenderService       |--------------------->|  GameTickService    |
|   (main thread)           |<---------------------| (dedicated thread)  |
|                           |     RenderEvent      |                     |
+---------------------------+                      +---------------------+
                                                        |         ^
                                          GameMessage   |         | GameMessage
                                          (outgoing)    v         | (incoming)
                                               +------------------+
                                               | NetSender /      |
                                               | recv_loop task   |
                                               | (tokio tasks)    |
                                               +--------+---------+
                                                        |  TCP (LAN)
                                                        v
                                               +------------------+
                                               | RelayServerSvc   |
                                               | battletris-server|
                                               +--------+---------+
                                                        |  TCP (LAN)
                                                        v
                                               +------------------+
                                               | NetSender /      |
                                               | recv_loop task   |
                                               | (tokio tasks)    |
                                               +--------+---------+
                                                        |         ^
                                          GameMessage   |         | GameMessage
                                                        v         |
                                               +------------------+
                                               | GameTickService  |
                                               | (Player 2)       |
                                               +------------------+
```
