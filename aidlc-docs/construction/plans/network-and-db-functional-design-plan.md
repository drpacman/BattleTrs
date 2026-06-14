# Functional Design Plan — Unit 3: network-and-db

## Unit Context

**Goal**: Add TCP relay server, client networking, player database, and ELO system.  
Deliverable: two machines on the same LAN can play a full BattleTris game.

**Stories covered**: U3-C01 through U3-C18  
**Depends on**: Unit 2 complete (engine, weapons, channel pattern in place)

## Plan Checkboxes

- [x] Analyze unit context (unit-of-work.md, unit-of-work-story-map.md, application-design.md)
- [x] Collect answers to design questions below
- [x] Generate business-logic-model.md
- [x] Generate domain-entities.md
- [x] Generate business-rules.md
- [x] Update aidlc-state.md and audit.md

---

## Design Questions

Please answer each question by filling in the letter after `[Answer]:`.  
Let me know when you are done.

---

## Question 1
How should the player choose between local vs-Ernie mode and two-player network mode?

Currently the client always starts in vs-computer mode. In Unit 3 we need to support both.

A) **CLI flags distinguish modes** — `cargo run -p battletris-client` (no flags) launches vs-Ernie as today; adding `--server <IP> --port <N> --name <NAME>` switches to network mode. Both modes always available.

B) **Title screen menu** — The title screen gains a menu: "1 = vs Ernie  2 = Network game". If "2" selected, a connection screen appears asking for IP, port, and name (typed in-game, no CLI args needed).

C) **Network-only in Unit 3** — Remove vs-Ernie from the client for now; Unit 3 client is always in network mode. vs-Ernie can be restored later.

D) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 2
How should the bazaar be synchronized between two networked players?

Each player's bazaar triggers when combined lines (own + opponent) cross a 20-line multiple. This is already tracked locally in both the vs-Ernie and network-message paths.

A) **Fully independent (current approach extended)** — Each client tracks its own `combined_lines` counter using its own lines plus `LinesCleared` messages received from opponent. Bazaar triggers locally when the threshold is crossed, exactly as it does today vs Ernie. No server coordination needed. (Simplest; already works.)

B) **Server-arbitrated trigger** — Server counts combined lines from both clients' `LinesCleared` messages and broadcasts an explicit `BazaarOpen` message to both clients simultaneously. Ensures perfect sync but adds server complexity.

C) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 3
What should happen if one player disconnects or quits mid-game?

A) **Opponent wins, ELO updates** — The disconnecting player is treated as having forfeited. The server notifies the remaining player they won, and ELO is updated (the forfeiter loses rating, the winner gains rating).

B) **Game void, no ELO change** — Disconnection voids the game. No ELO update for either player. Both clients return to the title/lobby screen.

C) **Wait briefly, then void** — Server waits up to 15 seconds for reconnection. If the player reconnects within the window, the game resumes. Otherwise, the game is voided (no ELO update).

D) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Question 4
What opponent board information should be sent over the network?

A) **Board snapshot on piece lock only** — `BoardUpdate { snapshot }` is sent each time a piece locks in place, same as the current vs-Ernie behavior. The opponent sees a static view of the locked board (no falling piece visible).

B) **Board snapshot on lock + active piece position** — On piece lock send `BoardUpdate { snapshot }`, AND on each move/rotation send `PieceUpdate { kind, cells }` so the opponent sees the currently-falling piece in near-real-time.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 5
What happens when two players connect using the same username?

A) **Second player rejected** — The server refuses the connection and sends an error message. The second player must choose a different name.

B) **Auto-suffix disambiguation** — The server accepts the connection and renames the second player automatically (e.g., `Alice` becomes `Alice_2`). The player is informed of their actual name.

C) **Allow duplicates** — Usernames need not be unique. Each connection gets a separate session; `PlayerDb` records are matched by exact name. Two "Alice" entries can coexist.

D) Other (please describe after [Answer]: tag below)

[Answer]: A
