# Application Design — BattleTrisRs (Consolidated)

## Summary

BattleTrisRs is a faithful Rust port of BattleTris (1994 C++ Brown University game). The design uses 3 Cargo crates, 6 major components, a channel-based game loop, tokio async networking, and SDL2 rendering.

---

## Cargo Workspace

```
battletris-rs/
+-- Cargo.toml              (workspace)
+-- battletris-engine/      (lib) — pure game logic, AI, protocol types
+-- battletris-client/      (bin) — SDL2 client + game loop + network client
+-- battletris-server/      (bin) — tokio relay server + player DB + CLI
+-- BattleTris/             (C++ source reference — read-only, never modified)
```

---

## 6 Components

| # | Component | Crate | Role |
|---|-----------|-------|------|
| 1 | Engine | `battletris-engine` | Board, PieceKind enum, WeaponKind enum, GameState, Score |
| 2 | AI (Ernie) | `battletris-engine::ai` | Placement search, board scoring, weapon strategy |
| 3 | Protocol | `battletris-engine::protocol` | GameMessage enum, serialization, shared record types |
| 4 | Renderer | `battletris-client::renderer` | SDL2 rendering of all game screens |
| 5 | NetworkClient | `battletris-client::net` | tokio TCP client, send/recv GameMessage |
| 6 | Server | `battletris-server` | tokio relay, player DB, ELO, btref CLI |

---

## Key Architectural Decisions (from Q1–Q6)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Workspace | 3 crates + shared lib (Q1=A) | Engine tested without SDL2/tokio; server has no SDL2 dep |
| Piece representation | `enum PieceKind` (Q2=A) | Exhaustive, zero heap alloc, idiomatic Rust |
| Weapon dispatch | `enum WeaponKind` + match (Q3=A) | All 34 effects in one place; easy to test and trace |
| Server runtime | tokio async (Q4=A) | Idiomatic modern Rust; clean task-per-session model |
| Game loop | Channel-based: tick thread + mpsc + SDL2 thread (Q5=C) | Engine logic decoupled from rendering; SDL2 stays on main thread |
| Game state machine | `enum GamePhase` (Q6=A) | Explicit, exhaustive, compiler-checked transitions |

---

## Critical Types

### `enum GamePhase`
```rust
enum GamePhase {
    Title,
    ConnectingToServer,
    WaitingForOpponent,
    Playing,
    InBazaar,
    Paused,
    GameOver { won: bool },
}
```

### `enum PieceKind` (18 variants)
```rust
enum PieceKind {
    El, RevEl, SldLft, SldRt, Long, Plug, Box_,
    Die { pips: u8 },
    Happy,
    Dog, RevDog, Cap, Wall, Tower, Star, WeirdLong,
    FourByFour, LongDong,
}
```

### `enum WeaponKind` (34 variants)
```rust
enum WeaponKind {
    FearedWeird, FourByFour, Hatter, UpBySide, FallOut, Swap,
    Lawyers, RiseUp, FlipOut, Speedy, Missing, PieceIt, Blind,
    Mondale, Keating, Carter, Reagan, Ames, Ace, Condor,
    NiceDay, SoLong, NoDice, Bug, Bottle, NoSlide, Susan,
    Meadow, Mirror, Twilight, Slick, Broken, Force, Gimp,
}
```

### `enum GameMessage` (network wire type)
Covers all lobby, game-loop, and administration events in one serializable enum. Encoded as length-prefixed `bincode` over TCP.

### `enum RenderEvent`
Cheap snapshots sent from game-tick thread to SDL2 main thread via `mpsc`. Contains all view data needed for a frame; no borrowing of `GameState` across threads.

---

## Thread Model

```
Main thread (SDL2)
  - SDL2 event pump
  - Renderer::render(event)
  - sends: PlayerInput → game-tick thread
  - receives: RenderEvent ← game-tick thread

Game-tick thread
  - owns GameState exclusively
  - calls GameState::tick() at ~60fps
  - sends: RenderEvent → main thread
  - sends: GameMessage → net-send task
  - receives: PlayerInput ← main thread
  - receives: GameMessage ← net-recv task

Net-send task (tokio)
  - receives: GameMessage ← game-tick thread
  - async-writes to TCP stream

Net-recv task (tokio)
  - async-reads from TCP stream
  - sends: GameMessage → game-tick thread

Relay server (separate process, tokio)
  - session task per client pair
  - relays GameMessage bidirectionally
  - intercepts QueryResult for ELO update
```

---

## Unit Mapping

| Unit | Components built | Deliverable |
|------|-----------------|-------------|
| Unit 1: core-engine | Engine (Board, PieceKind, GameState, Score) + Renderer | Single-player Tetris playable in SDL2 window |
| Unit 2: weapons-and-ai | Engine (WeaponKind, WeaponState, BazaarState) + AI + NetworkClient stub | Local vs-Ernie game with all 34 weapons and bazaar |
| Unit 3: network-and-db | Protocol + NetworkClient + Server + PlayerDb | Full two-player LAN game with ELO |

---

## Artifact Index

| File | Purpose |
|------|---------|
| `components.md` | 6 component definitions with responsibilities |
| `component-methods.md` | Method signatures per component |
| `services.md` | 3 services: GameTickService, SDL2RenderService, RelayServerService |
| `component-dependency.md` | Crate graph, channel wiring, module visibility rules |
| `application-design.md` | This consolidated summary |
