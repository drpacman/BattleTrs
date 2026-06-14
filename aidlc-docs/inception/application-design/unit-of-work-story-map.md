# Unit-of-Work Story Map — BattleTrisRs

## Note on User Stories

The User Stories stage was intentionally skipped (features fully specified; no UX persona ambiguity). The capability stories below are derived directly from the requirements (FR-1 through FR-8, NFR-1 through NFR-4) and serve as acceptance-level checkpoints for each unit. They are not formal Agile user stories tied to a backlog.

---

## Unit 1: core-engine — Capability Stories

**Theme**: A working Tetris game engine and SDL2 rendering foundation.

| ID | Capability | Requirement |
|----|-----------|-------------|
| U1-C01 | Launch game → see title/splash screen with game name | FR-3 (SDL2 rendering) |
| U1-C02 | Start single-player game → pieces appear and fall | FR-1 (piece types) |
| U1-C03 | Arrow keys move piece left/right; up/Z rotates; space hard-drops | FR-1 (gameplay) |
| U1-C04 | Piece locks when it cannot fall further | FR-1 (collision) |
| U1-C05 | Complete line(s) clear and rows above fall down | FR-1 (line clearing) |
| U1-C06 | Score increments with each cleared line | FR-6 (scoring) |
| U1-C07 | `Die { pips }` piece cleared → earn `pips` funds | FR-1 (die pip funds) |
| U1-C08 | `Happy` piece cleared → earn 150 funds | FR-1 (happy funds) |
| U1-C09 | Funds counter visible in the playing UI | FR-1 (funds display) |
| U1-C10 | Board fills to top → game over screen shown | FR-1 (game over) |
| U1-C11 | Next-piece preview visible in UI | FR-3 (UI layout) |
| U1-C12 | Opponent board area present in UI (blank in Unit 1) | FR-3 (two-board layout) |
| U1-C13 | `GameMessage` enum compiles with all variants declared | FR-4 (Q1=B: protocol skeleton) |
| U1-C14 | Windows x86_64 cross-compile builds cleanly | NFR-1 (cross-platform) |
| U1-C15 | `cargo test -p battletris-engine` passes for board/piece/collision logic | NFR-3 (unit tests) |

---

## Unit 2: weapons-and-ai — Capability Stories

**Theme**: Full BattleTris gameplay with the Ernie AI opponent, bazaar, and all 34 weapons.

| ID | Capability | Requirement |
|----|-----------|-------------|
| U2-C01 | `--vs-computer` flag starts local vs-Ernie game | FR-5 (Ernie AI) |
| U2-C02 | Two boards displayed: human (left), Ernie (right) | FR-3 (two-board layout) |
| U2-C03 | After 20 combined lines cleared → bazaar screen appears | FR-1 (bazaar trigger) |
| U2-C04 | Bazaar shows available weapons with names, prices, descriptions | FR-1 (bazaar UI) |
| U2-C05 | Player can purchase weapon(s) with accumulated funds | FR-1 (bazaar purchase) |
| U2-C06 | Purchased weapon appears in player's arsenal | FR-1 (arsenal) |
| U2-C07 | Player can launch weapon from arsenal against opponent | FR-2 (weapon launch) |
| U2-C08 | All 34 weapon effects apply to the target board correctly | FR-2 (all weapons) |
| U2-C09 | Time-limited weapons expire after their duration | FR-2 (weapon duration) |
| U2-C10 | Ernie chooses legal piece placements (exhaustive search) | FR-5 (placement AI) |
| U2-C11 | Ernie evaluates board and picks best placement | FR-5 (board scoring) |
| U2-C12 | Ernie participates in bazaar and purchases weapons | FR-5 (weapon strategy) |
| U2-C13 | Ernie launches purchased weapons against player | FR-5 (AI attacks) |
| U2-C14 | Weapon effects use `GameMessage` channels between boards | Q3=B (channel pattern) |
| U2-C15 | `cargo test -p battletris-engine` passes for weapon and AI logic | NFR-3 (unit tests) |

---

## Unit 3: network-and-db — Capability Stories

**Theme**: Two-player LAN game with relay server, ELO ranking, and player database.

| ID | Capability | Requirement |
|----|-----------|-------------|
| U3-C01 | Server starts with `battletris-server serve --port <N>` | FR-4 (relay server) |
| U3-C02 | Client connects to server by entering IP:port as CLI args | FR-4 (manual IP entry) |
| U3-C03 | "Connecting to server…" screen shown while connecting | FR-3 (lobby UI) |
| U3-C04 | "Waiting for opponent…" screen shown until second client connects | FR-3 (lobby UI) |
| U3-C05 | Two clients connected → game starts on both machines simultaneously | FR-4 (matchmaking) |
| U3-C06 | Piece moves and rotations sync correctly to opponent's view | FR-4 (board sync) |
| U3-C07 | Weapon effects deliver correctly to opponent's board over network | FR-2 + FR-4 |
| U3-C08 | Bazaar phase syncs between clients; both see same event trigger | FR-1 + FR-4 |
| U3-C09 | Game ends when one board fills; both clients see correct winner | FR-1 + FR-4 |
| U3-C10 | ELO ratings update after ranked networked game (K=32, start 1200) | FR-6 (ELO) |
| U3-C11 | `battletris-server players` lists all known players and ratings | FR-6 (player DB) |
| U3-C12 | `battletris-server show <name>` displays a player's record and ELO | FR-6 (player DB) |
| U3-C13 | Player records persist across server restarts (JSON flat file) | FR-6 (persistence) |
| U3-C14 | New player record auto-created on first login | FR-6 (player DB) |
| U3-C15 | `protocol::encode` / `decode` round-trips all `GameMessage` variants | NFR-3 (unit tests) |
| U3-C16 | `battletris-server` ELO delta calculation tested | NFR-3 (unit tests) |
| U3-C17 | Full workspace builds for Windows x86_64 | NFR-1 (cross-platform) |
| U3-C18 | Two machines on same subnet can play a full game | NFR-2 (LAN playability) |

---

## Story Summary

| Unit | Stories | Theme |
|------|---------|-------|
| Unit 1: core-engine | U1-C01 – U1-C15 (15 stories) | Tetris engine + SDL2 rendering + protocol skeleton |
| Unit 2: weapons-and-ai | U2-C01 – U2-C15 (15 stories) | 34 weapons + bazaar + Ernie AI + channel pattern |
| Unit 3: network-and-db | U3-C01 – U3-C18 (18 stories) | Relay server + LAN gameplay + ELO + player DB |
| **Total** | **48 capability stories** | Full BattleTrisRs port |
