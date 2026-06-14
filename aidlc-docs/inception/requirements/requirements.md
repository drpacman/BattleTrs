# Requirements — BattleTrisRs

## Intent Analysis

- **User Request**: Port the historic BattleTris C++ game to Rust, playable by two machines on a local network (Mac + Windows)
- **Request Type**: Migration / Port (C++ → Rust; X11/Motif → SDL2; POSIX-only → cross-platform)
- **Scope Estimate**: System-wide — full rewrite of all subsystems (game engine, rendering, networking, database, AI)
- **Complexity Estimate**: High — multiple subsystems, cross-platform target, faithful feature parity with original

---

## Extension Configuration

| Extension | Enabled | Decided At |
|---|---|---|
| Security Baseline | No | Requirements Analysis (Q10=B — game port/prototype) |
| Property-Based Testing | No | Requirements Analysis (Q11=C — standard unit tests sufficient) |

---

## Functional Requirements

### FR-1: Faithful Game Engine
Port the complete BattleTris gameplay faithfully:
- **Board**: 10 columns × 28 rows (BT_BOARD_WTH × BT_BOARD_HGT)
- **Standard pieces**: 7 Tetris pieces (L, reverse-L, S-right, S-left, I, T, O)
- **Special pieces**: Die piece (1×1, pip values 1-6), Happy-face piece (1×1, 150 funds if cleared on spawn turn)
- **Weird pieces**: 9 types (Dog, RevDog, Cap, Wall, Tower, Star, WeirdLong, FourByFour, LongDong) — active when FEARED_WEIRD or FOUR_BY_FOUR weapon is in effect
- **Funds system**: Lines cleared earn funds equal to the sum of die pip values in that line; double/triple/tetris multiplies accordingly; happy-face awards 150 funds
- **Bazaar**: Triggered when combined lines total is a multiple of 20; both players pause and purchase weapons with accumulated funds
- **Weapon arsenal**: Each player holds up to 10 weapons (BT_ARSENAL_SIZE); launched by pressing number keys 1–10
- **Game over**: First player whose board fills to the top loses

### FR-2: All 34 Weapons
Port all BTWeaponToken weapon types (FEARED_WEIRD through GIMP) with their original effects, prices, and durations. Weapon definitions compiled directly into Rust source (not an external data file). Effects include:
- Board orientation changes (UPBYSIDE, FLIP_OUT, MIRROR)
- Piece disruption (FEARED_WEIRD, FOUR_BY_FOUR, HATTER, BROKEN, PIECE_IT)
- Board state manipulation (SWAP, RISE_UP, FALL_OUT, SLICK)
- Visibility disruption (BLIND)
- Control disruption (NO_SLIDE)
- Fund attacks (LAWYERS, CARTER)
- Drop rate changes (SPEEDY)
- Piece restriction (MISSING, NO_DICE, BOTTLE)

### FR-3: SDL2 Rendering — Modern Clean 2D
- Use the `sdl2` crate for rendering on both macOS and Windows
- Visual style: modern clean 2D — same layout, board proportions, and cell dimensions as the original (23×23 pixel cells, BT_BOX_WTH × BT_BOX_HGT) but with a cleaner, more modern aesthetic
- Render: local player's board, opponent's board (mini/recon view), score panel, funds display, arsenal list, next-piece preview, bazaar screen
- Keyboard input via SDL2 event loop (replacing Xt keyboard callbacks)

### FR-4: Server Relay Model
- One machine runs a lightweight Rust relay server (replacing btserverd + btslaved)
- Both game clients connect to the server by entering its IP address and port manually at startup
- The server relays game traffic between the two paired clients
- Server tracks connected clients and pairs them for a game session
- No auto-discovery; manual IP:port entry on each client

### FR-5: AI Opponent (Ernie)
- Port BTComputer: exhaustive search of all piece placements and rotations
- Board state scoring: penalize holes and height variance (matching original `computeValue()` logic)
- Weapon purchase strategy: port BTCOrders priority queue logic
- Single-player mode: one client connects to the server and challenges Ernie (AI runs inside the server process or as a local client-side AI)
- Multiple difficulty levels (matching original `nLevels()` / `levelName()`)

### FR-6: Player Database and ELO Ranking
- Persistent player records: username, ELO rating (starting at 1200), win/loss counts
- ELO updated after each ranked game (networked games only; single-player vs AI does not affect rank)
- Database stored on the server machine as flat files (replacing BTDB)
- Players identified by username; no authentication (matching original approach)
- Admin CLI (`btref` equivalent) to view and manage player records

### FR-7: Audio Stub
- All audio calls compile and link but produce no output
- No audio library dependency required; stub functions return immediately
- Structure code so audio can be added later without architectural changes

### FR-8: Development Priority — Incremental Build Order
Build in this order to achieve a working game as early as possible:
1. **Phase 1**: Core Tetris engine — board, pieces (18 types), collision, line clearing, funds accumulation; single-player; SDL2 rendering
2. **Phase 2**: Weapons system — all 34 weapons, bazaar, arsenal management; working in single-player or vs-AI
3. **Phase 3**: Networking — server relay, two-player LAN game, opponent board sync, ELO and player database

---

## Non-Functional Requirements

### NFR-1: Cross-Platform
- Must compile and run on macOS (Apple Silicon arm64 and x86_64)
- Must compile and run on Windows 10/11 (x86_64)
- Single Cargo workspace; no platform-specific build scripts beyond SDL2 linking differences
- SDL2 dynamic library distributed with the Windows binary or linked statically

### NFR-2: LAN Playability
- Two machines on a local network (same subnet) must be able to play a full game
- Network latency on LAN is acceptable; no compensation needed for WAN latency
- The relay server can run on either player's machine or a dedicated machine on the LAN

### NFR-3: Idiomatic Rust
- Use safe Rust throughout; `unsafe` only where SDL2 FFI requires it (scoped and documented)
- Prefer Rust standard library types (Vec, HashMap, etc.) over custom containers
- Use `serde` + `bincode` (or equivalent) for network serialization; replace ad-hoc binary framing
- Unit tests for all pure game logic (piece rotation, line clearing, collision detection, score calculation)

### NFR-4: Maintainability
- Clear module boundaries matching the original: `engine`, `renderer`, `network`, `server`, `ai`, `db`
- Weapon effects and piece shapes defined in one place (single source of truth, compiled in)
- All weapon timing/pricing/duration values derived from the original `btweapons.db` data

---

## Constraints

- **No audio assets available**: The original BattleTris sounds are lost; audio is stubbed, not implemented
- **No existing Rust code in workspace**: Full greenfield Rust implementation (the source of truth is the C++ codebase in `BattleTris/`)
- **Motif not used**: The X11/Motif widget layer is not ported; SDL2 replaces it entirely
- **btweapons.db content must be preserved**: All 34 weapon names, descriptions, prices, and durations from the original data file must be faithfully reproduced as compiled-in Rust constants

---

## Success Criteria

1. A Rust binary runs on macOS and on Windows producing a playable Tetris game with all 18 piece types
2. All 34 weapons are implemented and correctly affect gameplay when launched
3. The bazaar screen appears, allows weapon purchase, and correctly deducts funds
4. Two machines on a LAN can play a full game against each other via the relay server
5. The AI opponent (Ernie) provides a playable single-player experience with recognizable strategy
6. Player ELO and win/loss records persist between sessions on the server machine
7. The relay server can be started independently and accepts connections from both Mac and Windows clients
