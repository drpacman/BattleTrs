# Unit 2 Functional Design Plan — weapons-and-ai

## Checklist

- [x] Step 1: Collect answers to clarifying questions — Q1=A, Q2=A, Q3=A, Q4=A
- [x] Step 2: Generate domain-entities.md (WeaponKind, WeaponDef, Arsenal, BazaarState, AiDifficulty, WeaponState)
- [x] Step 3: Generate business-logic-model.md (weapon effect algorithms, Ernie AI placement search, bazaar flow, fund economy)
- [x] Step 4: Generate business-rules.md (34 weapon rules, AI strategy rules, bazaar purchase rules, mirror reflection rules)
- [x] Step 5: Generate frontend-components.md (bazaar screen, opponent board with weapons, arsenal display, active-weapon indicators)

---

## Clarifying Questions

**Q1 — Weapon effect fidelity**

The 34 weapons divide into six implementation tiers:

| Tier | Weapons | Typical effort |
|------|---------|----------------|
| A — Board-state changes | Rise Up, Lawyers D., Fallout, Swap, Flip Out, Upbyside, Bottle | Rebuild/shift board rows |
| B — Piece disruption | FW (weird pieces), FBF (4×4 box), Missing, Piece It, Bug, Broken, So Long, No Dice, Hatter, Nice Day | Filter or replace PieceKind |
| C — Physics / controls | Speedy, Meadow, No Slide, Slick, Force | Modify drop interval or input handling |
| D — Economic | Mondale, Keating, Carter, Reagan | Modify funds / bazaar prices |
| E — Visibility / intel | Blind, Twilight, Gimp, Ames, Ace, Condor | Hide/show cells or opponent stats |
| F — Arsenal meta | Mirror, Lazy Susan | Redirect or swap arsenals |

Which scope do you want for Unit 2?

A) **All 34 faithfully** — every weapon implemented to match the original C++ behaviour
B) **Tiers A–D** (26 weapons) — board, piece, physics, economic; skip E–F as stubbed no-ops for now  
C) **Simplified versions of all 34** — implement every weapon but accept minor behavioural differences where the faithful version would require significant extra SDL2 work (e.g., Twilight = grey cells instead of invisible cells)

[Answer]: 

---

**Q2 — Ernie AI algorithm depth**

`BTComputer.C` uses a recursive exhaustive placement search (`checkMove`) with six penalty/bonus weights:
- `BT_OPEN_HOLE_PENALTY 7000`, `BT_CLOSED_HOLE_PENALTY 10000`, `BT_COVERED_HOLE_PENALTY 3000`
- `BT_HEIGHT_PENALTY 30000`, `BT_VARIANCE_PENALTY 50`, `BT_LINE_BONUS 5000`, `BT_HAPPY_BONUS 20000`

Which AI implementation should Unit 2 use?

A) **Port BTComputer's algorithm faithfully** — exhaustive DFS over all (column, rotation) placements; evaluate with the six original penalty/bonus weights; single default difficulty (750ms think interval = "Focused Ernie")
B) **Greedy column scan** — for each rotation, try every column; score by (lines completed − holes created − max height); much simpler, still plays a real game
C) **Random legal placement** — picks a legal (column, rotation) at random; Ernie will lose often but the game-loop integration is simple and testable

[Answer]: 

---

**Q3 — Bazaar UI interaction model**

Which interaction style for the bazaar screen?

A) **Scrollable list with keyboard navigation** — full weapon name, price, description visible; Up/Down selects; Enter buys; Esc exits; sorted by price ascending (matches btweaponsp.db display logic)
B) **Numbered slots (1–0 keys)** — weapons listed in their canonical enum order (0–33); player presses the slot number to buy; simpler to implement
C) **Auto-timed** — bazaar screen shows for 5 seconds; if player doesn't act the screen closes (matches what Ernie does, good for single-player feel)

[Answer]: 

---

**Q4 — Weapon duration unit**

The original measures active-weapon duration in **lines cleared by the affected player** (e.g., "Speedy lasts 10 lines"). Should the Rust port use the same unit?

A) **Lines cleared** — faithful to original; duration counts down as the affected player clears lines (exactly as btweaponsp.db field 2 specifies)
B) **Real time (seconds)** — simpler to implement; duration in seconds derived from lines × 8s/line estimate; loses authenticity but avoids tracking per-player line counts inside weapon state
C) **Pieces placed** — each piece the affected player places counts as one tick; simpler than tracking line clears

[Answer]: 

---

## Source Data Already Extracted

### Weapon Definitions (from btweapons.db + btweaponsp.db)

| # | Token | Name | Price | Duration (lines) |
|---|-------|------|-------|----------|
| 0 | FW | The Feared Weird | 400 | 3 |
| 1 | FBF | Four-by-Four | 425 | 10 |
| 2 | Hatter | The Mad Hatter | 375 | 5 |
| 3 | Upbyside | Upbyside-down | 125 | 10 |
| 4 | Fallout | Fallout | 250 | 10 |
| 5 | Swap | Swap Meet | 1200 | 0 |
| 6 | Lawyers | Lawyer's Delite | 350 | 5 |
| 7 | RiseUp | Rise Up | 75 | 0 |
| 8 | FlipOut | Flip Out | 15 | 0 |
| 9 | Speedy | Speedy Gonzales | 275 | 10 |
| 10 | Missing | Missing Pieces | 50 | 0 |
| 11 | PieceIt | Piece It Together | 100 | 0 |
| 12 | Blind | The Blind Cleric | 400 | 0 |
| 13 | Mondale | Mondale '96 | 150 | 50 |
| 14 | Keating | Keating Five | 425 | 0 |
| 15 | Carter | Carter Years | 250 | 20 |
| 16 | Reagan | Reagan Era | 425 | 0 |
| 17 | Ames | William Ames | 50 | 20 |
| 18 | Ace | Ace of Spies | 100 | 30 |
| 19 | Condor | The Condor | 225 | 40 |
| 20 | NiceDay | Have a Nice Day | 50 | 0 |
| 21 | SoLong | So Long | 100 | 10 |
| 22 | NoDice | No Dice | 600 | 35 |
| 23 | Bug | Bug Report | 320 | 0 |
| 24 | Bottle | Bottle Neck | 150 | 10 |
| 25 | NoSlide | Slide Denied | 125 | 10 |
| 26 | Susan | Lazy Susan | 600 | 0 |
| 27 | Meadow | Meadow | 475 | 10 |
| 28 | Mirror | Mirror Mirror | 500 | 10 |
| 29 | Twilight | The Twilight Zone | 450 | 0 |
| 30 | Slick | Slick Willy | 650 | 3 |
| 31 | Broken | Broken Record | 325 | 5 |
| 32 | Force | The Force | 325 | 5 |
| 33 | Gimp | The Gimp | 25 | 0 |

### AI Constants (from BTComputer.C)
- Default difficulty: 750ms per move ("Focused" Ernie)
- Open hole penalty: 7000
- Closed hole penalty: 10000
- Covered hole penalty: 3000
- Height penalty: 30000
- Line bonus: 5000
- Happy bonus: 20000
- Variance penalty: 50
- Weapons Ernie never buys: Ames, Ace, Condor, Meadow, Susan, Reagan
- Weapons Ernie ignores on launch: Hatter, FlipOut, Speedy
- Ernie enables Susan purchase after opponent reaches 50 lines

### Arsenal Size (from BTConstants)
- `BT_ARSENAL_SIZE = 10` — each player holds up to 10 weapon slots
- Multiple of same weapon allowed (quantity shown in UI)
