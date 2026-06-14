# Application Design Plan — BattleTrisRs

## Scope
Design the high-level component structure for the BattleTrisRs Rust port: 6 modules across 3+ Cargo crates, defining component boundaries, interfaces, and communication patterns. Detailed business logic (piece rotation tables, weapon effect algorithms, AI scoring) is deferred to Functional Design per unit.

## Plan Checkboxes

- [x] Answer user design questions (below)
- [x] Generate components.md — 6 components with responsibilities
- [x] Generate component-methods.md — method signatures per component
- [x] Generate services.md — game loop, match orchestration, server dispatch
- [x] Generate component-dependency.md — crate graph, communication patterns
- [x] Generate application-design.md — consolidated summary
- [x] Update aidlc-state.md and audit.md

---

## Design Questions

Please fill in the letter choice after each `[Answer]:` tag. Let me know when you're done.

---

### Question 1: Cargo Workspace Structure
How should the Rust workspace be organized?

A) **3 crates + shared lib**:
   - `battletris-engine` (lib) — game logic only; no SDL2, no network
   - `battletris-client` (bin) — SDL2 rendering + game loop; depends on engine
   - `battletris-server` (bin) — relay server + player DB; depends on engine for protocol types
   - *(AI lives inside engine; all three units share the engine crate)*

B) **2 crates**:
   - `battletris` (lib) — everything except binaries (engine, weapons, AI, network protocol)
   - `battletris-server` (bin) — server binary; depends on the lib

C) **Single crate with feature flags**:
   - One crate; `--features client` builds the SDL2 client, `--features server` builds the relay server
   - Reduces cross-platform linking complexity (server binary needs no SDL2)

X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 2: Piece Abstraction
How should the 18 piece types be represented in Rust?

A) **Enum** — `enum PieceKind { El, RevEl, SldLft, SldRt, Long, Plug, Box, Die, Happy, Dog, ... }` with a `match` in methods like `cells(rotation)` that returns the cell offsets. Simple, exhaustive, no heap allocation.

B) **Trait objects** — `trait Piece { fn cells(&self, rotation: u8) -> &[(i32, i32)]; ... }` with 18 structs implementing it, stored as `Box<dyn Piece>`. Mirrors the original C++ polymorphic design.

C) **Enum wrapping data** — an enum where each variant carries its rotation table as a static `&[(i32,i32)]` slice, looked up at construction time. Combines enum exhaustiveness with static data tables.

X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 3: Weapon Effect Architecture
How should the 34 weapon effects be dispatched and applied?

A) **Enum match** — `enum WeaponKind` with 34 variants; a `apply_weapon(kind, &mut GameState)` free function with a `match` arm per weapon. All effects in one place; easy to read and test.

B) **Per-weapon handler functions** — a `WeaponDef` struct holds `fn apply(&mut GameState)` and `fn tick(&mut GameState)` function pointers (one pair per weapon). The weapon table is an array of `WeaponDef`.

C) **Trait** — `trait WeaponEffect { fn on_activate(&self, state: &mut GameState); fn on_line_cleared(&self, state: &mut GameState); }` with 34 structs; stored as `Box<dyn WeaponEffect>` in a registry.

X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 4: Server / Networking Runtime
What should power the relay server and client networking?

A) **tokio** — async runtime; each connected client is a tokio task; game relay is async message passing. Clean, idiomatic for modern Rust network code. Requires `tokio` dependency.

B) **std::net + threads** — each client connection spawns an OS thread; `TcpListener` + `TcpStream` from std only. Simpler dependency tree; more than sufficient for 2 concurrent clients.

C) **Either is fine** — choose based on what leads to the clearest code for a 2-client relay server.

X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 5: Game Loop / SDL2 Integration Style
How should the client-side game loop drive both SDL2 events and piece-drop timing?

A) **Fixed-timestep tick loop** — a tight loop polls SDL2 events and advances a `GameState` on a fixed tick (e.g. every 16ms); piece drop is counted in ticks rather than wall-clock timers. Simple; deterministic; easy to test.

B) **SDL2 timer callbacks** — use `sdl2::timer::Timer` (analogous to Xt timeouts in the original) to fire piece-drop and weapon-tick events asynchronously. Closer to the original architecture.

C) **Channel-based loop** — a dedicated game-tick thread sends events over an `mpsc` channel; the SDL2 render thread processes them. Decouples rendering from game logic.

X) Other (please describe after [Answer]: tag below)

[Answer]: C

---

### Question 6: Game State Representation
How should the overall game state machine be modeled?

A) **Explicit state enum** — `enum GamePhase { Title, WaitingForOpponent, Playing, InBazaar, GameOver }` with a top-level `GameState` struct holding the current phase and all sub-state. The game loop matches on `GamePhase`.

B) **Implicit / procedural** — no explicit state enum; the game loop uses boolean flags and function calls to control flow (similar to the original C++ approach with `started_`, `in_baz_`, `paused_`, etc.).

X) Other (please describe after [Answer]: tag below)

[Answer]: A
