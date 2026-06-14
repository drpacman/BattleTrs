# Business Rules — Unit 2: weapons-and-ai

## Weapon Activation Rules

**BR-W01** — Weapons with `WeaponDef.duration > 0` set `weapon_state.remaining[kind.index()] += duration` on the TARGET board when activated. Duration counts down in lines cleared by the target player.

**BR-W02** — Weapons with `duration = 0` apply their effect immediately (instant) and do NOT increment any `remaining` counter. They do not appear as "active" in `weapon_state`.

**BR-W03** — Multiple applications of the same timed weapon STACK: remaining lines are accumulated, not replaced (`remaining += duration`).

**BR-W04** — The duration countdown fires ONCE per `check_and_clear_lines` call, decrementing by the number of lines just cleared. A 4-line tetris decrements by 4.

**BR-W05** — A weapon that reaches `remaining = 0` triggers a `WeaponOff` event and immediately ceases affecting the board.

---

## Mirror Rules

**BR-M01** — Mirror is active on the DEFENDING player's board (the launch TARGET). It intercepts weapons aimed at that board.

**BR-M02** — When Mirror is active, the following weapons are NULLIFIED (no effect on either board): `Swap`, `Mondale`, `Keating`, `Ames`, `Ace`, `Condor`, `NiceDay`, `Susan`, `Mirror`.

**BR-M03** — All other weapons when Mirror is active are REFLECTED: the effect is applied to the LAUNCHER's board instead of the target's board.

**BR-M04** — The weapon is consumed from the arsenal regardless of Mirror reflection or nullification.

---

## Arsenal Rules

**BR-A01** — Arsenal capacity is 10 slots (`BT_ARSENAL_SIZE = 10`). An eleventh weapon cannot be bought if all 10 slots are occupied by distinct weapon kinds, unless one slot already holds that kind (stacking is allowed).

**BR-A02** — Stacking: buying a weapon already in the arsenal increments its quantity rather than creating a new slot.

**BR-A03** — A player cannot buy a weapon if `score.funds < effective_price(kind)`.

**BR-A04** — Arsenal slots 1-10 are mapped to keys 1-9 and 0 (slot 10 = key 0) during play.

**BR-A05** — Lazy Susan (BT_SUSAN) swaps the two players' arsenals instantly. The swap is NOT mirrored or nullified by Mirror on the target.

---

## Board-Effect Rules

**BR-BE01** — Rise Up: if any cell in row 0 was occupied before the shift, the board is immediately topped-out (game over).

**BR-BE02** — Flip Out: mirrors the board horizontally. Cells in column 0 move to column 9, column 1 to 8, etc. Active piece position is also mirrored.

**BR-BE03** — Swap: both boards exchange their entire cell grid. Each board retains its OWN `weapon_state` (timed effects stay attached to the board, not the player). Bottle and Upbyside active states also transfer with the board.

**BR-BE04** — Fallout zone: columns 2 through 7 inclusive (0-indexed) act as a black hole while Fallout is active. Piece cells landing in those columns are destroyed (not placed). Line completion checks treat those columns as always empty.

**BR-BE05** — Bottle neck zone: rows 7 through 20 inclusive narrow the playfield to columns 3 through 6. Cells outside that range in those rows are treated as permanently occupied walls. Line clears in the neck only require columns 3-6 to be filled.

**BR-BE06** — Force: when active, cleared lines are zeroed in place (row becomes empty). Rows above the cleared line do NOT shift downward.

---

## Piece-Disruption Rules

**BR-PD01** — Feared Weird overrides the piece picker: only the seven "weird" piece kinds (Dog, RDog, Cap, Wall, Tower, Star, WeirdLong) can spawn while active.

**BR-PD02** — Four-by-Four overrides: if the random picker would produce `Box`, it produces `FourByFour` (piece 17) instead.

**BR-PD03** — FW and FBF can both be active simultaneously; FBF takes priority over FW for Box selection; FW takes effect for all non-Box pieces.

**BR-PD04** — Broken Record: the first piece that spawns AFTER the weapon is activated becomes the "locked" piece kind. That kind repeats for the weapon's 5-line duration.

**BR-PD05** — So Long: if the random picker produces `Long`, re-roll. Re-roll until a non-Long result is obtained.

**BR-PD06** — No Dice: if the picker produces `Die { .. }`, re-roll. Re-roll until a non-Die result.

**BR-PD07** — PieceIt and Bug place a piece on the TARGET's board. If the chosen column/placement overlaps an existing cell, the weapon has no effect (not guaranteed to land).

**BR-PD08** — Bug placed cells are stored as `Cell::Bug`. They are occupied for collision but rendered as `Cell::Empty` (invisible). They can be cleared by completing a line.

---

## Economic Rules

**BR-E01** — Score.funds is stored as `i64` (can be negative after Reagan).

**BR-E02** — A player with negative funds cannot make any purchases. Purchases require `funds >= effective_price`.

**BR-E03** — Mondale tax rate is 30%. Formula: `kept = amount * 70 / 100; taxed = amount - kept`. If multiple Mondale effects are stacked, the effective rate is `1 - 0.7^n` (each stack takes 30% of what remains), up to a maximum tax of 90%.

**BR-E04** — Carter Year doubles effective_price for ALL weapons in the bazaar for the affected player. Does not affect funds earned mid-game.

**BR-E05** — Keating Five is instant: target's funds drop to 0 and the full amount is transferred to the launcher's funds immediately.

**BR-E06** — Reagan Era is instant: `target.funds = -target.funds`. If target already has 0 or negative funds, the effect is zero.

---

## Bazaar Rules

**BR-B01** — Bazaar opens when `combined_lines` (both players' line totals summed) crosses a multiple of 20. This is detected in `Score::add_lines()` returning `true`.

**BR-B02** — BOTH player and Ernie must signal done (`BazaarEnd`) before the game resumes. Neither plays while the other is still in the bazaar.

**BR-B03** — Ernie's bazaar timer fires after 3000ms (`BT_BAZAAR_TIMEOUT`). Ernie always finishes shopping within that window.

**BR-B04** — The bazaar weapon list shows all 34 weapons sorted by price ascending. Carter-affected prices are displayed.

**BR-B05** — A player can buy multiple weapons in one bazaar visit until funds are exhausted or arsenal is full.

**BR-B06** — Pressing Esc in the bazaar immediately marks the player as done (no further purchases).

---

## Spy-Weapon Rules

**BR-S01** — Ames, Ace, and Condor reveal information about the OPPONENT to the LAUNCHER (not the target). They affect the launcher's PlayingView, not the target's gameplay.

**BR-S02** — Ames: reveals exact opponent funds for 20 lines. Board not revealed.

**BR-S03** — Ace: reveals opponent board at 80% cell accuracy (20% of non-empty cells are rendered with wrong colour). Funds revealed. Duration: 30 lines.

**BR-S04** — Condor: reveals opponent board at 100% accuracy. Funds revealed. Duration: 40 lines.

**BR-S05** — In the local vs-Ernie game, Ernie's board is always visible (since Ernie can't hide it). These weapons still affect the PlayingView flag for protocol consistency with Unit 3.

---

## AI Rules

**BR-AI01** — Ernie exhaustively searches all legal (column, orientation) placements for each piece, computing the board evaluation value for every reachable position.

**BR-AI02** — Ernie selects the placement with the MINIMUM evaluation value (lower = better).

**BR-AI03** — Ernie's think interval at "Focused" difficulty is 750ms. One piece is evaluated and placed per interval.

**BR-AI04** — Ernie NEVER purchases: Ames, Ace, Condor, Meadow, Susan, Reagan (at game start). Susan becomes purchasable after opponent reaches 50 lines.

**BR-AI05** — Ernie NEVER launches: Hatter, FlipOut, Speedy (he ignores these even if purchased).

**BR-AI06** — Ernie does not move pieces while the bazaar is open. He waits for `BazaarEnd` from the player before resuming.

**BR-AI07** — After Reagan, Ernie waits 50 opponent lines before buying another Reagan, Keating, or NiceDay (prevents economic spam).

**BR-AI08** — Ernie's board evaluation increases height penalty and variance penalty when FourByFour or Force is active on his board (faithful port of `BT_WPN_ON` handler in BTComputer.C).

**BR-AI09** — Ernie's board evaluation increases line bonus and height penalty when Fallout is active (must build bridges, not rely on tetris lines).

**BR-AI10** — Ernie reduces hole penalty when Bottle is active (plays differently inside the neck).

---

## Visibility Rules

**BR-V01** — Blind cells (`blind_cells` list in WeaponState) persist for the rest of the current game after the weapon fires. They are not time-limited.

**BR-V02** — Twilight marks all currently occupied cells as `Cell::Twilight` at the instant the weapon fires. New cells placed after Twilight are normally visible. A second Twilight makes new cells invisible again.

**BR-V03** — Bug cells (`Cell::Bug`) are invisible (rendered as empty) but fully solid for collision and line-clear counting.

**BR-V04** — Gimp overlay clears after 2 seconds (`Duration::from_secs(2)`). No gameplay impact.

---

## Slick / Hatter Timing Rules

**BR-T01** — Slick Willy has its OWN tick timer at 150ms interval (BT_SLICK_TIMEOUT), independent of the 512ms drop timer. Each Slick tick: move piece one cell in `slick_dir`; reverse `slick_dir` on wall/cell collision.

**BR-T02** — Hatter spin fires on every GAME tick (not its own timer): attempt CW rotation; ignore if blocked.

**BR-T03** — Slick and Hatter can be active simultaneously. Both effects apply each tick.
