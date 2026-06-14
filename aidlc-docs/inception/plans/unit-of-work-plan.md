# Unit-of-Work Plan — BattleTrisRs

## Scope

Refine and confirm the 3 units of work already defined in the execution plan and application design. Since component boundaries, method signatures, and crate structure are already specified, this planning step focuses on **within-unit sequencing decisions** that are not yet pinned down.

## Plan Checkboxes

- [x] Answer unit scoping questions (below)
- [x] Generate unit-of-work.md — detailed description of each unit
- [x] Generate unit-of-work-dependency.md — inter-unit dependencies and entry/exit criteria
- [x] Generate unit-of-work-story-map.md — story-level breakdown per unit
- [x] Update aidlc-state.md and audit.md

---

## Established Units (confirmed from Application Design)

| Unit | Name | Deliverable |
|------|------|-------------|
| Unit 1 | `core-engine` | Single-player Tetris in SDL2 window: Board + 18 pieces + line clearing + die/happy funds accumulation + score |
| Unit 2 | `weapons-and-ai` | Local vs-Ernie game: all 34 weapons, bazaar, arsenal, Ernie AI opponent |
| Unit 3 | `network-and-db` | Full two-player LAN game: relay server, two-player protocol, PlayerDb, ELO |

---

## Unit Scoping Questions

Please fill in the letter choice after each `[Answer]:` tag. Let me know when you're done.

---

### Question 1: Protocol Type Placement Across Units

`battletris-engine::protocol` (the `GameMessage` enum and encode/decode) is needed in Unit 3.  
Should any protocol scaffolding appear earlier?

A) **Protocol is strictly Unit 3** — `protocol/` module stays empty (or absent) until Unit 3. Unit 1 and Unit 2 have no network types. The game-tick thread in Unit 2 drives both the local board and Ernie's board directly, with no GameMessage passing involved.

B) **Declare `GameMessage` skeleton in Unit 1** — Add a `protocol/` module with the bare `GameMessage` enum (all variants, no serialization yet) in Unit 1. This lets the engine's `GameState::apply_network_message` signature be stable from the start, even though no TCP code exists yet.

C) **Add protocol skeleton in Unit 2** — Introduce the bare `GameMessage` enum and `GameState::apply_network_message` in Unit 2 (when Ernie's moves need to cross the channel boundary in the same way network moves eventually will). Serialization/TCP added in Unit 3.

[Answer]: B

---

### Question 2: Cross-Platform (Windows) Validation Timing

The final game must build for Windows x86_64 as well as macOS. When should Windows build validation happen?

A) **Unit 1** — After the SDL2 rendering skeleton is working on Mac, immediately verify that `cargo build --target x86_64-pc-windows-gnu` (cross-compile) or a Windows CI job compiles cleanly. Catches SDL2 linking and platform-specific issues as early as possible.

B) **Unit 3** — Validate Windows build only when the full game is functionally complete. Windows-specific issues are fixed at that point as a cleanup pass.

C) **Post-completion** — Skip during the 3 units entirely; treat cross-platform verification as a follow-on task after all units are done and the game plays correctly on Mac.

[Answer]: A

---

### Question 3: Unit 2 Local-vs-Ernie Board Architecture

In Unit 2, the deliverable is a local vs-Ernie game (no network). How should the two boards be driven?

A) **Two `GameState` instances in the same tick thread** — The game-tick thread owns both the player's `GameState` and Ernie's `GameState`. Weapon events are applied directly between them without any channel or message passing. Simplest for Unit 2; Unit 3 replaces the Ernie board with a network opponent.

B) **Simulate the channel pattern early** — Even in Unit 2, use `GameMessage` structs to communicate between the two boards (local channel, no TCP). This means the game-tick thread -> "ernie net task" -> Ernie's GameState flow is established in Unit 2, making Unit 3 a smaller diff (just swap the channel endpoints with real TCP).

C) **Either is fine** — Choose whichever produces cleaner code.

[Answer]: B
