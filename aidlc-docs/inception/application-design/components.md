# Components — BattleTrisRs

## Cargo Workspace Layout

```
battletris-rs/              (workspace root)
+-- battletris-engine/      (lib crate)  — pure game logic, no SDL2, no network I/O
+-- battletris-client/      (bin crate)  — SDL2 client + game loop + network client
+-- battletris-server/      (bin crate)  — tokio relay server + player DB
```

---

## Component 1: Engine — `battletris-engine`

**Crate**: `battletris-engine` (lib)
**Purpose**: The entire game simulation — the source of truth for all game rules, state, and logic. Pure Rust with zero platform dependencies (no SDL2, no tokio, no I/O).

**Modules**:
- `engine::board` — Board grid and all operations on it
- `engine::piece` — PieceKind enum and rotation/shape data
- `engine::weapons` — WeaponKind enum, WeaponDef constants, active weapon state
- `engine::game_state` — GameState struct, GamePhase enum, tick logic
- `engine::score` — Score, funds accumulation, line clearing rewards
- `engine::protocol` — GameMessage enum (network wire types), BoardSnapshot, ArsenalSnapshot

**Responsibilities**:
- Define the 10×28 `Board` and all mutations (fill, check_lines, flip, insert_line, swap, clear)
- Define `PieceKind` enum (18 variants) with cell offsets for each rotation, color, and special flags (is_die, is_happy, is_weird)
- Define `WeaponKind` enum (34 variants) and the `WeaponDef` constants table (name, description, price, duration)
- Own `WeaponState` (which weapons are active, how many lines remain on each)
- Apply weapon effects via `apply_weapon(kind, active, &mut GameState)` free function
- Define `GameState` (owns Board, current piece, score, weapon state, phase)
- Drive the game simulation via `GameState::tick(input)` returning a list of `GameEvent`s
- Define `GamePhase` enum for the game state machine
- Define all network-serializable message types (`GameMessage`, `BoardSnapshot`, etc.) using `serde`

**Key types**:
```
Board, PieceKind, WeaponKind, WeaponDef, WeaponState,
GameState, GamePhase, GameEvent, PlayerInput,
Score, ArsenalSlot, Arsenal, BazaarState,
GameMessage, BoardSnapshot, ArsenalSnapshot
```

---

## Component 2: AI — `battletris-engine::ai`

**Crate**: `battletris-engine` (module `ai`)
**Purpose**: The "Ernie" AI opponent. Evaluates all possible piece placements, scores board states, and strategises weapon purchases. Runs inside the engine crate so it shares all game types without crossing crate boundaries.

**Responsibilities**:
- Enumerate all valid placements for a given `PieceKind` on a given `Board` (all rotations × all column positions)
- Score each resulting board state using a penalty function (holes, height variance, covered holes)
- Return the best `Placement` (column, rotation) for the current piece
- Maintain a weapon purchase priority queue (`AiOrders`) to decide what to buy at the bazaar and when to launch
- Support multiple difficulty levels (adjusting search depth, penalty weights, and move delay)

---

## Component 3: Protocol — `battletris-engine::protocol`

**Crate**: `battletris-engine` (module `protocol`)
**Purpose**: All types that cross the network wire, shared by both client and server without duplication.

**Responsibilities**:
- Define `GameMessage` enum covering all lobby, game-loop, and meta events (replaces BTToken + BTWeaponToken)
- Provide `encode(msg) -> Vec<u8>` and `decode(bytes) -> Result<GameMessage>` using `serde` + `bincode`
- Define `BoardSnapshot` (serializable board state for opponent sync)
- Define `ArsenalSnapshot` (serializable arsenal for opponent display)
- Define `PlayerRecord` (username, ELO, wins, losses) for server-side DB and client display

---

## Component 4: Renderer — `battletris-client::renderer`

**Crate**: `battletris-client` (module `renderer`)
**Purpose**: All SDL2 rendering. Receives `RenderEvent` values from the game-tick thread and produces pixels. Owns the SDL2 context.

**Responsibilities**:
- Initialize SDL2 window (title, size, vsync)
- Render the local player's board (filled cells, active piece, ghost piece)
- Render the opponent's board (from `BoardSnapshot` — mini view or full recon view)
- Render the score panel (score, lines, funds)
- Render the next-piece preview
- Render the active arsenal (up to 10 weapon slots, numbered 1–10)
- Render the bazaar screen (available weapons, prices, player funds)
- Render the title screen, lobby/connection screen, game-over screen
- Render weapon effect overlays (e.g. black screen for BLIND, upside-down board for UPBYSIDE)
- Handle all SDL2 Color, Font (TTF), and Rect operations

---

## Component 5: NetworkClient — `battletris-client::net`

**Crate**: `battletris-client` (module `net`)
**Purpose**: Client-side tokio networking. Maintains the TCP connection to the relay server and bridges the game-tick thread with the network.

**Responsibilities**:
- Connect to relay server by `SocketAddr` (entered manually by user at startup)
- Run a tokio task that continuously reads `GameMessage` values from the server and sends them to the game-tick thread via `mpsc`
- Accept `GameMessage` values from the game-tick thread to send to the server
- Handle disconnection and fatal errors gracefully (send `GameEvent::NetworkError` to game-tick thread)
- Provide length-prefixed framing over TCP (4-byte big-endian length prefix + bincode payload)

---

## Component 6: Server — `battletris-server`

**Crate**: `battletris-server` (bin)
**Purpose**: The relay server. Replaces btserverd + btslaved. Accepts two client connections, pairs them, relays game messages between them, and maintains the player database.

**Responsibilities**:
- Listen on a configurable `SocketAddr` (default `0.0.0.0:4404`)
- Accept client connections using tokio; maintain a lobby of waiting clients
- Pair two clients into a `Session`; spawn a tokio task per session to relay `GameMessage` values bidirectionally
- Record game results (`BT_QUER_RESULT` equivalent) and update ELO in the player database
- Persist player database to disk (JSON or bincode flat file) on every ELO update
- Expose a `btref`-equivalent subcommand (CLI via `clap`) to list players and show stats
- Support the AI client mode: if only one human connects and requests computer play, the server spawns a local AI `GameState` and drives it as the second "player"
