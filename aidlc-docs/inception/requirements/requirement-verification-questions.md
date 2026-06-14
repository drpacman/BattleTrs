# Requirements Clarification Questions — BattleTrisRs

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options fit, choose the last option (Other) and describe your preference after the tag.
Let me know when you're done.

---

## Question 1
How faithful should the Rust port be to the original BattleTris gameplay?

A) Faithful port — preserve all original features: all 34 weapons, die pieces, happy-face pieces, bazaar, funds system, ELO ranking, player database, and AI opponent (Ernie)
B) Core gameplay only — Tetris board, die/happy pieces, funds system, bazaar, and all weapons; skip ELO ranking and player database (no persistent stats)
C) Simplified — standard Tetris pieces + a subset of the most impactful weapons; no die pieces, no ELO, no player database
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
What should the networking architecture look like for the LAN version?

A) Direct peer-to-peer TCP — one player acts as host (listens on a port), the other connects directly; no separate server process needed
B) Keep the server model — one machine (either the Mac or a third machine) runs a lightweight relay server; both clients connect to it
C) Either is fine — decide based on what is simplest to implement and most reliable on LAN
X) Other (please describe after [Answer]: tag below)

[Answer]: B 

---

## Question 3
Which rendering / windowing library should be used for the Rust port?

A) SDL2 (`sdl2` crate) — mature, widely supported on Mac and Windows, pixel-level control, good for 2D games
B) macroquad — simpler API, single-file game loop, cross-platform, good for rapid prototyping
C) Bevy (game engine) — full ECS game engine, more powerful but more setup overhead
D) Terminal / TUI (e.g. `ratatui`) — text-based rendering in the terminal, no graphics library needed
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 4
Should the AI opponent (Ernie) be included in the Rust port?

A) Yes — port the AI opponent so a single player can play against the computer without needing a second machine
B) No — two-player LAN only; skip the AI opponent for now
C) Defer — two-player LAN first; AI can be added later
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 5
What should happen with audio / sound effects?

A) Stub it out — no audio for now; the game runs silently
B) Implement basic audio using a cross-platform Rust library (e.g. `rodio`) if sound assets become available
C) Not important — skip entirely, don't even stub it
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 6
How should the visual style be approached?

A) Recreate the original Motif look faithfully — grey panels, block-based board, 23x23 pixel cells, original color scheme
B) Modern clean 2D — same layout and proportions but with a cleaner/more modern visual style
C) Functional only — get it working first; visual polish can come later
X) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 7
For the two-player LAN setup, how should players find each other / connect?

A) Manual IP entry — one player hosts (sets a listen port), the other enters the host's IP address and port to connect
B) LAN auto-discovery — the game broadcasts/listens for peers on the local network (e.g. UDP broadcast or mDNS)
C) Either — start with manual IP entry; auto-discovery can be added later
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 8
Should the weapons be loaded from an external data file (like the original `btweapons.db`), or compiled in?

A) Compiled in — embed weapon definitions directly in Rust source code (simpler, no external file dependency)
B) External file — keep weapons configurable via a data file (TOML, JSON, or similar)
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 9
What is the priority order for the initial deliverable?

A) Playable two-player LAN game first — networking and game loop working before worrying about visual polish or full weapon set
B) Full feature completeness first — all weapons, full game loop, then networking
C) Core Tetris engine first — get the single-player board working (piece movement, line clearing), then add weapons and networking
X) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Question 10: Security Extension
Should security extension rules be enforced for this project?

A) Yes — enforce all SECURITY rules as blocking constraints (recommended for production-grade applications)
B) No — skip all SECURITY rules (suitable for PoCs, prototypes, and experimental projects like this game port)
X) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 11: Property-Based Testing Extension
Should property-based testing (PBT) rules be enforced for this project?

A) Yes — enforce all PBT rules as blocking constraints (recommended for projects with significant game logic, like piece rotation and collision)
B) Partial — enforce PBT rules only for pure functions (piece rotation, collision detection, line clearing logic)
C) No — skip all PBT rules; standard unit tests are sufficient
X) Other (please describe after [Answer]: tag below)

[Answer]: C
