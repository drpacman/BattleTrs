# Business Rules — Unit 1: core-engine

Rules are sourced from BTConstants.H, BTBoardManager.C, BTGame.C, and BTPieceManager.C. Each rule cites its origin.

---

## Board Rules

**BR-01 Fixed dimensions**: Board is always exactly 10 columns × 28 rows. Width and height are compile-time constants; no dynamic resizing.
*Source: `BT_BOARD_WTH 10`, `BT_BOARD_HGT 28` (BTConstants.H:89–90)*

**BR-02 Coordinate origin**: (0, 0) is the top-left corner. x increases rightward (0–9). y increases downward (0–27).

**BR-03 Boundary is occupied**: Any position outside the board grid is considered occupied:
- `x < 0` or `x ≥ 10` → left/right wall → occupied
- `y < 0` → top boundary → occupied (pieces cannot exist above row 0)
- `y ≥ 28` → floor → occupied
*Source: `BTBoardManager::occupied()` (BTBoardManager.H:71–86)*

**BR-04 Line full condition**: A row is full if and only if all 10 cells are occupied (non-`Empty`). Structural cells (`Struct_`) count toward fullness and are NOT removable. Structural cells should not normally appear in Unit 1.
*Source: `BTBoardManager::checkLines()` (BTBoardManager.C:576–578)*

**BR-05 Line clear direction**: When a row is cleared, all rows above it (y < cleared_row) shift down by one. Row 0 becomes empty. This applies when gravity is normal (not UpBySide). UpBySide reversal handled in Unit 2.
*Source: `BTBoardManager::removeLine()` (BTBoardManager.C:73–116)*

---

## Piece Rules

**BR-06 Die piece pip range**: Die pips are always in the range [1, 6] inclusive, chosen uniformly at random at spawn time.
*Source: `BTDiePiece::construct()` (BTPiece.C:274): `rand() % 6 + 1`*

**BR-07 Die piece has no rotation**: `Die` and `Happy` pieces cannot rotate. `num_rotations = 1`. Rotation input is silently ignored.
*Source: `BTDiePiece` / `BTHappyPiece` constructors: `rot_ = 0`*

**BR-08 Box pieces have no rotation**: `Box_` and `FourByFour` cannot rotate. `num_rotations = 1`.
*Source: `BTBoxPiece` constructor: `rot_ = 0`; `BTFourByFourPiece` constructor: `rot_ = 0`*

**BR-09 LongDong has no rotation**: The 8-cell horizontal bar cannot rotate. `num_rotations = 1`.

**BR-10 Rotation never wall-kicks**: If a rotation attempt would place any cell of the rotated piece on an occupied cell or boundary, the rotation is silently rejected. The piece keeps its current rotation. No alternate positions are tried.
*Source: `BTPiece::canRotate()` and `BTPiece::rotate()` (BTPiece.C:85–147)*

**BR-11 Spawn x offset**: `spawn_x = 5 - (bounding_box_width / 2)` (integer division). This centers larger pieces over the board.
*Source: `BTGame::place()` (BTGame.C:801)*

**BR-12 Spawn y**: All pieces spawn at `y = 0` (topmost row visible).
*Source: `BT_DEFAULT_Y 0`*

**BR-13 Game over on spawn failure**: If a newly spawned piece cannot be placed at its spawn position (spawn position is occupied), the game ends immediately for that player.
*Source: `BTGame::place()` (BTGame.C:803–806)*

**BR-14 Piece selection is probabilistic**: Piece type is chosen via rejection sampling. Weird pieces (Dog, RevDog, Cap, Wall, Tower, Star, WeirdLong) and FourByFour have probability 0 in normal play; they appear only when specific weapons are active.
*Source: `BTPieceManager::create()` (BTPieceManager.C:179–217)*

---

## Funds Rules

**BR-15 Funds formula**: `funds_earned = (sum of fund_value() for all cells in cleared lines) × lines_cleared_count`
*Source: `BTBoardManager::checkLines()` (BTBoardManager.C:613–614): `short funds = value * lines.inc()`*

**BR-16 Die fund value**: A die cell in a cleared line contributes its pip count (1–6) to `value`.
*Source: `BTBox::value()` for die boxes: returns pip count*

**BR-17 Happy fund value**: A happy cell (not yet missed) in a cleared line contributes exactly 150 to `value`.
*Source: `BT_HAPPY_VAL 150` (BTConstants.H:82); `BTHappyBox::value()`: returns `BT_HAPPY_VAL` unless `landed_`*

**BR-18 Happy missed — no funds**: Once a happy cell is converted to `HappyMissed` (sad face), it contributes 0 to `value` in all future line clears.
*Source: `BTHappyBox::value()`: `if (landed_) return 0`*

**BR-19 Regular cells contribute 0 funds**: Non-special cells have `fund_value() = 0`. Clearing lines of regular pieces earns no funds.

**BR-20 Funds can be negative**: The Reagan weapon (`BT_REAGAN`) negates `funds_ *= -1` on activation. Funds are stored as `i32`.
*Source: `BTScoreManager::receive(BT_WPN_ON/BT_REAGAN)` (BTScoreManager.C:125)*

---

## Score Rules

**BR-21 Score from hard drop only**: Score is NOT awarded for line clears. Score is awarded only when a hard drop begins: `score += 28 - y_at_start_of_drop`.
*Source: `BTGame::beginDrop()` (BTGame.C:729): `score_manager_->rep_.score_ += BT_BOARD_HGT - y_`*

**BR-22 Maximum hard drop score**: A piece hard-dropped from the very top (y=0) earns at most 28 points. Hard drop from y=27 earns 1 point.

---

## Timing Rules

**BR-23 Base drop interval**: 512 ms per gravity step in normal play.
*Source: `BT_DROP_TIME 512` (BTConstants.H:93)*

**BR-24 Hard drop interval**: 10 ms per gravity step during hard drop.
*Source: `BT_FAST_DROP_TIME 10` (BTConstants.H:92)*

**BR-25 Lock delay**: After a piece touches a surface (cannot fall further), a 150 ms lock delay begins. The piece locks only after this delay expires.
*Source: `BT_SLIDE_TIME 150` (BTConstants.H:94)*

**BR-26 Lock delay reset on move/rotate**: Any successful move or rotation during the lock delay resets the 150 ms timer (Q2=A). This allows the player to continue adjusting the piece after it touches ground.

**BR-27 No speed levels**: There is no level-based drop speed increase. `DROP_INTERVAL_MS` is constant at 512 ms throughout the game. Drop speed changes only via the Speedy weapon (Unit 2).

---

## Happy Piece Rules

**BR-28 Happy-missed detection occurs during line check**: When `check_and_clear_lines()` finds a row that is NOT full, it checks every cell in that row. Any `Happy` cell is immediately converted to `HappyMissed`.
*Source: `BTBoardManager::checkLines()` (BTBoardManager.C:589–594)*

**BR-29 HappyMissed is still solid**: A `HappyMissed` cell occupies its position and contributes to line fills. It just earns 0 funds.

**BR-30 NiceDay weapon forces happy pieces**: When the NiceDay weapon is active, the next N pieces are forced to be Happy. (Unit 2 weapon effect; BTPieceManager tracks `hap_on_`.)

---

## Bazaar Rules

**BR-31 Bazaar trigger condition**: The bazaar triggers whenever `(my_lines + opponent_lines) % 20` transitions from a small number back to 19 (i.e., the combined total crosses a multiple of 20).
*Source: `BTScoreManager::receive(BT_LINE / BT_OP_SCORE)` (BTScoreManager.C:189–193)*

**BR-32 Bazaar in single-player**: In Unit 1 (single-player mode), opponent lines = 0, so the bazaar triggers every 20 lines cleared by the player. (The bazaar screen itself is Unit 2.)

---

## WeaponState Rules (Stubs in Unit 1)

**BR-33 WeaponState stub**: In Unit 1, `WeaponState` is declared but contains no active weapons. All weapon checks in `PieceKind::random()` and `Board::check_and_clear_lines()` default to no active effects.

---

## Idiot Detection (Ernie Feedback)

**BR-34 Idiot condition**: A player (or Ernie) is flagged as an "idiot" if a piece is placed such that an empty gap is created that is completely surrounded on three sides (left, right, and top) by occupied cells. This flag is sent as feedback to the AI in Unit 2.
*Source: `BTBoardManager::landed()` (BTBoardManager.C:499–549): `idiot_ = 1; reason_ = BT_BAD_MOVE`*

**BR-35 Near-death condition**: If the highest occupied cell is in row 7 or above (7 rows from the top), a near-death flag is raised. This drives the jeopardy sound and can influence Ernie's strategy.
*Source: BTBoardManager.C:601: `if (min < 8) { idiot_ = 1; reason_ = BT_NEAR_DEATH; }`*
