# Functional Design Plan — Unit 1: core-engine

## Source Analysis Summary

Key facts extracted directly from BattleTris C++ source (BTConstants.H, BTBoardManager.C, BTScoreManager.C, BTPieceManager.C, BTGame.C):

| Constant | Value | Source |
|----------|-------|--------|
| Board width | 10 columns | `BT_BOARD_WTH 10` |
| Board height | 28 rows | `BT_BOARD_HGT 28` |
| Cell pixel size (original) | 23×23 px, border 3 px | `BT_BOX_WTH 23`, `BT_BOX_HGT 23`, `BT_BOX_BRDR 3` |
| Piece bounding box | 8×8 map (not all cells used) | `BT_PIECE_WIDTH 8`, `BT_PIECE_HEIGHT 8` |
| Piece spawn | x = 5 − (rot_/2), y = 0 | `BT_DEFAULT_X 5`, `BT_DEFAULT_Y 0` |
| Drop interval (normal) | 512 ms | `BT_DROP_TIME 512` |
| Drop interval (fast/hard) | 10 ms | `BT_FAST_DROP_TIME 10` |
| Slide timer | 150 ms after piece touches ground | `BT_SLIDE_TIME 150` |
| Happy piece fund value | 150 | `BT_HAPPY_VAL 150` |
| Lines until bazaar | 20 combined | `BT_LINES_TIL_BAZ 20` |
| Score from hard drop | `BT_BOARD_HGT − current_y` | BTGame.C:729 |

**Piece probability system (BTPieceManager.C)**:
- Normal set (prob 0.21 each): El, RevEl, SldLft, SldRt, Long, Plug, Box (7 types)
- Die (prob 1.0): always accepted when randomly selected
- Exotic (prob 0.02 each): Happy, LongDong
- Never in normal play (prob 0.0): Dog, RevDog, Cap, Wall, Tower, Star, WeirdLong, FourByFour
  - Weird pieces (Dog→WeirdLong) only appear during **FearWeird** weapon
  - FourByFour only appears during **FourByFour** weapon (replaces Box)

**Funds formula** (`BTBoardManager::checkLines()`):
- `value = sum of cell.value() for each cell in all cleared lines`
- Regular cells: `value() = 0`
- Die cells: `value() = pip_count` (1–6)
- Happy cells (not yet missed): `value() = 150`
- Happy cells (missed = sad): `value() = 0` (still occupies board)
- `funds_earned = value × lines_cleared_simultaneously`

**Score formula**:
- Score is NOT awarded for line clears in BattleTris
- Score only increments on hard drop: `score += BT_BOARD_HGT − y` (up to 28 pts per drop)

**Rotation rules**:
- Die, Happy, Box_, FourByFour: `rot_ = 0` — no rotation
- El, RevEl, SldLft, SldRt, Plug, Dog, RevDog, Tower: `rot_ = 3` — generic 3×3 matrix rotation
- Long, Cap, WeirdLong: `rot_ = 4` — but WeirdLong has 6 custom states
- Wall: `rot_ = 4`, 4 custom states (manually moves cells between corners)
- Star: 2 custom states
- LongDong: `rot_ = 8`, but does NOT use generic rotation (only falls horizontally)
- **No wall kicks** — rotation fails silently if cells are occupied

**Happy piece "missed" mechanic** (`BTBoardManager::checkLines()`):
- If a happy cell is on the board in a row that is NOT cleared, it becomes `landed_` (sad face)
- Sad happy cell: `value() = 0`, still solid, visual changes from ☺ to ☹

---

## Plan Checkboxes

- [x] Analyze unit context and C++ source
- [x] Answer user design questions (below)
- [x] Generate business-logic-model.md
- [x] Generate business-rules.md
- [x] Generate domain-entities.md
- [x] Generate frontend-components.md (SDL2 renderer layout)
- [x] Update aidlc-state.md and audit.md

---

## Design Questions

Please fill in the letter choice after each `[Answer]:` tag. Let me know when you're done.

---

### Question 1: SDL2 Cell Pixel Size

The original renders each board cell at 23×23 px. For the "modern clean 2D" style (Q6=B), what cell size should we use?

This determines the board pixel area:
- A) **24 px** — 240×672 board; compact; fits any modern display
- B) **28 px** — 280×784 board; moderate; close to original proportions at modern resolution
- C) **32 px** — 320×896 board; roomy; crisp on HiDPI/Retina displays

The SDL2 window will include two boards (player + opponent), score panel, and weapon arsenal. Total window width ≈ 2 × board_width + UI_width.

[Answer]: B

---

### Question 2: Lock Delay (Slide Timer Behavior)

The original gives the player 150 ms after a piece touches a surface before it locks (the "slide timer"). During this window the player can still move or rotate the piece.

In the tick-based Rust loop, how should this be handled?

A) **Port the slide timer faithfully** — track a `lock_delay_ms` counter in `GameState`; piece locks only after 150 ms of being unable to fall further. Keyboard input during this window resets the timer (standard lock delay behavior).

B) **Immediate lock** — piece locks the instant it can no longer fall. Simpler; very different feel from the original.

C) **Configurable lock delay** — default 150 ms (matching original) but stored in a constant that can be changed.

[Answer]: A

---

### Question 3: Rotation Table Approach for Special Pieces

Most pieces use a standard matrix rotation (3×3 or 4×4). Five pieces have special rotation logic:
- **WeirdLong**: 6 distinct states, diagonal shape morphs through 6 hand-coded transitions
- **Wall**: 4 states, 2 cells jump between corner positions
- **Star**: 2 states, plus/cross alternation
- **LongDong**: 8 cells in a row; never rotates (only falls horizontally)
- **FourByFour**: hollow 4×4 square; never rotates

For these, should we:

A) **Static pre-computed cell tables** — define all orientations as `const [(i32, i32)]` arrays in `piece.rs`. Each piece's `cells(rotation)` method just indexes into its static table. Simple, testable, idiomatic Rust. (WeirdLong needs 6 entries, Wall 4, Star 2.)

B) **Port C++ state machines** — replicate the original swap-cells logic from BTPiece.C for Wall, Star, WeirdLong. The rotation transitions are computed each time rather than stored. Closer to original code structure.

[Answer]: A

