# Business Logic Model — Unit 1: core-engine

All algorithms sourced from C++ originals. Module: `battletris-engine`.

---

## 1. Piece Spawn Algorithm

**Source**: `BTPieceManager::create()` (BTPieceManager.C:179)

### Normal Spawn (rejection sampling)

```
loop:
    i = rand() % 18 + 1          // random piece type (1–18)
    j = rand_float() in [0, 1)
    if j < keep_prob[i]:
        break and use piece i
```

### Default Keep Probabilities

| Piece(s) | Probability | Notes |
|----------|-------------|-------|
| El, RevEl, SldLft, SldRt, Long, Plug, Box_ | 0.21 | Regular set |
| Die | 1.0 | Always accepted when selected |
| Happy, LongDong | 0.02 | Rare |
| Dog, RevDog, Cap, Wall, Tower, Star, WeirdLong | 0.0 | Only via FearWeird weapon (Unit 2) |
| FourByFour | 0.0 | Only via FourByFour weapon (Unit 2) |

**Implementation note**: `PieceKind::random(rng, weapon_state)` encapsulates this. In Unit 1, weapon_state is a stub that returns all-normal probabilities.

### Die Pip Randomization
When `Die` is selected: `pips = rand() % 6 + 1` (uniform 1–6). Stored in `PieceKind::Die { pips }`.

### Spawn Position
```
spawn_x = SPAWN_X - (piece.bounding_box_width / 2)
spawn_y = SPAWN_Y  // = 0
rotation = 0
```

Where `bounding_box_width` matches the original `rot_` value per piece type:
- 1×1 pieces (Die, Happy): bounding_box_width = 0 → spawn_x = 5
- 3×3 pieces: bounding_box_width = 3 → spawn_x = 5 - 1 = 4
- 4×4 pieces (Long, Cap, WeirdLong): bounding_box_width = 4 → spawn_x = 5 - 2 = 3
- 8-wide (LongDong): bounding_box_width = 8 → spawn_x = 5 - 4 = 1
- No-rotation (Box_, FourByFour): bounding_box_width = 0 → spawn_x = 5

### Game Over Check
Immediately after spawning, call `active_piece.can_move_to(board, 0, 0)`. If false, emit `GameEvent::GameOver { won: false }`.

---

## 2. Gravity Loop (Tick-Based)

**Source**: `BTGame::drop()`, `BTGame::place()`, timeouts (BTGame.C)

The original uses Xt timer callbacks. The Rust port uses a fixed-timestep tick loop at `~60 fps` driven by the game-tick thread. All timing is tracked via elapsed-ms counters.

### Gravity Step

Each call to `GameState::tick(input, elapsed_ms)`:

```
drop_elapsed_ms += elapsed_ms

if piece_state == Dropping:
    if drop_elapsed_ms >= DROP_INTERVAL_MS (512ms):
        drop_elapsed_ms -= DROP_INTERVAL_MS
        attempt gravity step

if piece_state == HardDropping:
    each tick: attempt gravity step until piece locks

if piece_state == LockDelay(elapsed):
    lock_delay_ms += elapsed_ms
    if lock_delay_ms >= LOCK_DELAY_MS (150ms):
        lock piece
    else:
        allow player to still move/rotate (resets lock_delay_ms on move/rotate)
```

### Gravity Step (one row)

```
if active_piece.can_move_to(board, 0, +1):
    active_piece.y += 1
    piece_state = Dropping
else:
    // piece touched ground
    if piece_state == HardDropping:
        lock immediately
    else:
        piece_state = LockDelay(0)
        lock_delay_ms = 0
```

### Hard Drop (from `BTGame::beginDrop`, BTGame.C:716)

```
score += Board::HEIGHT - active_piece.y    // BT_BOARD_HGT - y_
piece_state = HardDropping
drop_elapsed_ms = 0
```

Hard drop score is awarded at the moment the drop begins, before the piece reaches the bottom.

---

## 3. Piece Lock

**Source**: `BTGame::place()` (BTGame.C:765), `BTPiece::landed()` (BTPiece.C:172)

```
for each (ax, ay) in active_piece.cells():
    board.set(ax, ay, Cell::from(active_piece.kind))

result = board.check_and_clear_lines()
emit GameEvent::PieceLocked
if result.count > 0:
    emit GameEvent::LinesCleared { count, funds_earned }
    score.funds += result.funds_earned
    score.lines += result.count
    update lines_until_bazaar
if result.happy_missed:
    emit GameEvent::HappyMissed

spawn next_piece as active_piece
check game over
```

---

## 4. Line Clearing Algorithm

**Source**: `BTBoardManager::checkLines()` (BTBoardManager.C:551)

```
value = 0
count = 0
y = 27  // start from bottom
while y >= 0:
    if all cells board[y][0..9] are occupied:
        // Full line found
        for each cell in row y:
            value += cell.fund_value()
        count++
        remove_row(y)        // shift rows above down by 1
        // do NOT decrement y — recheck same row after shift
    else:
        // Check for missed happy faces
        for each cell in row y:
            if cell == Cell::Happy:
                board.set(x, y, Cell::HappyMissed)
                happy_missed = true
        y -= 1

funds_earned = value * count
return LinesCleared { count, funds_earned, happy_missed }
```

### Remove Row Algorithm

```
remove_row(target_y):
    for y = target_y downto 1:
        for x = 0..9:
            board[y][x] = board[y-1][x]
    for x = 0..9:
        board[0][x] = Cell::Empty
```

**Note**: In Unit 2, UpBySide weapon reverses gravity direction, changing this to shift rows UP instead of down (see `BTBoardManager::removeLine`).

---

## 5. Rotation Algorithm

**Source**: `BTPiece::rotate()` (BTPiece.C:101), `BTPiece::canRotate()` (BTPiece.C:85)

### Generic Rotation (El, RevEl, SldLft, SldRt, Plug, Dog, RevDog, Tower)

Clockwise 3×3 rotation: cell `(dx, dy)` → `(dy, 2 - dx)`

```
can_rotate(board, cw):
    new_cells = apply_rotation(active_piece.cells, cw)
    return all new_cells are unoccupied on board
    // NO wall kicks — if blocked, rotation simply fails

rotate(cw):
    if can_rotate(board, cw):
        active_piece.rotation = (rotation + 1) % num_rotations
        // (or -1 for CCW)
        reset lock_delay_ms = 0  // player action extends lock delay
```

**No wall kicks**: Rotation attempts are all-or-nothing. If any cell of the rotated piece overlaps an occupied cell or boundary, the rotation is silently rejected.

### Long Piece (4×4, 2 effective rotations)

CW rotation: cell `(dx, dy)` → `(dy, 3 - dx)` within 4×4 box. Long has 4 abstract rotation states but only 2 distinct shapes (horizontal / vertical), cycling at 4.

### Cap Piece (4×4, 4 rotations)

Same formula as Long but has 4 distinct shapes.

### Special Static-Table Pieces (Q3=A)

- **WeirdLong**: 6 static orientation arrays (diagonal diagonal shape morphing)
- **Wall**: 4 static orientation arrays (2 cells cycling through corner pairs)
- **Star**: 2 static orientation arrays (plus ↔ cross)

For these, `cells(rotation)` just indexes the static array; no computation at runtime.

### Non-Rotating Pieces

`num_rotations = 1` for: Die, Happy, Box_, FourByFour, LongDong.
`rotate()` returns immediately with no change.

---

## 6. Collision Detection

**Source**: `BTBoardManager::occupied()` (BTBoardManager.H:71)

```
occupied(x, y):
    if x < 0 || x >= 10:  return true   // left/right walls
    if y < 0 || y >= 28:  return true   // top ceiling and floor
    return board.cells[y][x].is_occupied()
```

`ActivePiece::can_move_to(board, dx, dy)`:
```
for each (ax, ay) in self.cells():
    if board.occupied(ax + dx, ay + dy):
        return false
return true
```

---

## 7. Score Accumulation

**Source**: BTGame.C:729, BTScoreManager.C

| Event | Score Change | Funds Change |
|-------|-------------|--------------|
| Hard drop | `+= (28 - y_before_lock)` | none |
| Line clear | none | `+= sum(cell_values) × lines_cleared` |
| Die cell cleared in a line | none | `+pip_value` (part of line total) |
| Happy cell cleared in a line | none | `+150` (part of line total) |
| Happy cell NOT cleared (missed) | none | none (value → 0 forever) |

**Key invariant**: Score is purely from hard drops. Funds are the economy.

---

## 8. Bazaar Trigger

**Source**: `BTScoreManager::receive(BT_LINE / BT_OP_SCORE)` (BTScoreManager.C)

```
lines_until_bazaar = 20 - (my_lines + opponent_lines) % 20
if lines_until_bazaar just wrapped (increased after decrement):
    emit GameEvent::BazaarTriggered
```

In Unit 1 (single-player): opponent lines = 0, bazaar never triggers. Wired properly in Unit 2 (vs-Ernie) and Unit 3 (network).

---

## 9. Game Over Detection

**Source**: `BTGame::place()` (BTGame.C:803)

After spawning a new piece:
```
if not active_piece.can_move_to(board, 0, 0):
    emit GameEvent::GameOver { won: false }
    phase = GamePhase::GameOver { won: false }
```

The opponent's game over (networked game, Unit 3): when `GameMessage::GameOver` is received from the server, `won = true`.

---

## 10. GameState::tick() Contract

```rust
pub fn tick(&mut self, input: Option<PlayerInput>, elapsed_ms: u32) -> Vec<GameEvent>
```

Processes one game tick:
1. Apply player input (if any): move, rotate, begin hard drop
2. Advance timing: add elapsed_ms to drop/lock counters
3. Fire gravity step if drop interval elapsed
4. Lock piece if lock delay elapsed
5. Return all emitted events

All state mutations happen inside `tick()`. The game-tick thread calls this; no external mutation of `GameState` is permitted.

---

## 11. apply_network_message() Stub (Unit 1)

```rust
pub fn apply_network_message(&mut self, msg: GameMessage) -> Vec<GameEvent> {
    // populated in Unit 3
    vec![]
}
```

Signature is stable from Unit 1. Body is a no-op until Unit 3.
