# Domain Entities — Unit 1: core-engine

All types live in `battletris-engine`. Coordinate convention: `x` = column (0 = left, 9 = right), `y` = row (0 = top, 27 = bottom).

---

## Cell

Represents a single board cell.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Regular(u8),    // color id 1–8 (BT_IVORY through BT_PURPLE)
    Die(u8),        // pip count 1–6; ivory background
    Happy,          // happy face, not yet missed; value = 150
    HappyMissed,    // sad face (missed clear); value = 0; still solid
    Struct_,        // structural cell from Bottle weapon; non-removable; Unit 2+
    Gimp(i32),      // Gimp-covered cell; tracks underlying value; Unit 2+
}

impl Cell {
    pub fn is_occupied(&self) -> bool { !matches!(self, Cell::Empty) }
    pub fn is_removable(&self) -> bool { !matches!(self, Cell::Struct_) }
    pub fn fund_value(&self) -> i32 {
        match self {
            Cell::Die(pips) => *pips as i32,
            Cell::Happy     => 150,
            Cell::Gimp(v)   => *v,
            _               => 0,
        }
    }
}
```

**Note**: `Struct_` and `Gimp` are declared in Unit 1 for stability, implemented in Unit 2.

---

## Board

Fixed 10×28 grid. Row-major: `cells[y][x]`.

```rust
pub struct Board {
    cells: [[Cell; 10]; 28],
    upside_down: bool,   // UpBySide weapon — gravity reverses; Unit 2+
}

impl Board {
    pub const WIDTH: i32 = 10;
    pub const HEIGHT: i32 = 28;

    pub fn occupied(&self, x: i32, y: i32) -> bool;
    pub fn get(&self, x: i32, y: i32) -> Cell;
    pub fn set(&mut self, x: i32, y: i32, cell: Cell);
    pub fn fill_piece(&mut self, piece: &ActivePiece, kind: PieceKind);
    pub fn check_and_clear_lines(&mut self) -> LinesCleared;
    pub fn insert_line(&mut self, hole_col: usize);    // Unit 2 (RiseUp, Lawyers)
    pub fn remove_partial_line(&mut self, row: i32, x1: usize, x2: usize); // Unit 2
    pub fn flip_horizontal(&mut self);                 // Unit 2 (UpBySide)
    pub fn flip_vertical(&mut self);                   // Unit 2 (FlipOut)
    pub fn snapshot(&self) -> BoardSnapshot;
}
```

**`occupied(x, y)` rules** (from `BTBoardManager::occupied`, BTBoardManager.H:71):
- `x < 0 || x >= 10` → occupied (left/right wall)
- `y < 0 || y >= 28` → occupied (top ceiling and floor)
- `cells[y][x].is_occupied()` → occupied
- Otherwise → not occupied

**`check_and_clear_lines()` algorithm** (from `BTBoardManager::checkLines`, BTBoardManager.C:551):
1. Scan rows bottom-up (y = 27 down to 0)
2. For each row: if all 10 cells are occupied → full line
3. On full line: add all `cell.fund_value()` to accumulated `value`; count this line; remove the row (shift rows above down by 1); re-check same y index
4. For each non-full row: if any cell is `Happy`, convert it to `HappyMissed`
5. Final `funds_earned = value × lines_cleared_count`
6. Returns `LinesCleared { count, funds_earned, happy_missed: bool }`

---

## BoardSnapshot

Cheap serializable board copy for network sync and rendering (no `Cell` enum — uses compact byte representation).

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardSnapshot {
    pub width: u8,    // always 10
    pub height: u8,   // always 28
    pub rep: Vec<u8>, // row-major; 0 = empty; 1–8 = regular color;
                      // 101–106 = die pip 1–6; 200 = happy; 201 = happy-missed;
                      // 20 = struct; 23 = gimp
}
```

This mirrors the original `BTBoard` wire format (BTBoard.C) for potential future interop with the C++ version.

---

## PieceKind

Enum with 18 variants. Each variant knows its cell offsets per rotation.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PieceKind {
    El, RevEl, SldLft, SldRt, Long, Plug, Box_,
    Die { pips: u8 },   // pips 1–6 chosen at spawn
    Happy,
    Dog, RevDog, Cap, Wall, Tower, Star, WeirdLong,
    FourByFour, LongDong,
}

impl PieceKind {
    pub fn cells(&self, rotation: u8) -> &'static [(i32, i32)];
    pub fn num_rotations(&self) -> u8;
    pub fn spawn_x_offset(&self) -> i32;  // rot_ / 2 subtracted from spawn x=5
    pub fn color_id(&self) -> u8;
    pub fn is_die(&self) -> Option<u8>;
    pub fn is_happy(&self) -> bool;
    pub fn random(rng: &mut impl Rng, weapon_state: &WeaponState) -> PieceKind;
}
```

### Rotation Table Design (Q3=A: static pre-computed arrays)

Cell offsets are `(dx, dy)` from the piece's `(x, y)` position. All tables are `&'static [(i32, i32)]`.

**Coordinate transform for CW rotation** (derived from BTBoardManager.C `rotate()`):
- Cell at `(dx, dy)` within a `rot × rot` bounding box → CW → `(dy, rot-1-dx)` — where `rot` is the bounding box side length.

**Rotation counts and approach per piece**:

| Piece | rot_ | Rotations | Approach |
|-------|------|-----------|----------|
| El | 3 | 4 | Generic 3×3 CW |
| RevEl | 3 | 4 | Generic 3×3 CW |
| SldLft | 3 | 4 | Generic 3×3 CW |
| SldRt | 3 | 4 | Generic 3×3 CW |
| Plug | 3 | 4 | Generic 3×3 CW |
| Dog | 3 | 4 | Generic 3×3 CW |
| RevDog | 3 | 4 | Generic 3×3 CW |
| Tower | 3 | 4 | Generic 3×3 CW |
| Long | 4 | 2 | Generic 4×4 CW (2 distinct, cycles at 4) |
| Cap | 4 | 4 | Generic 4×4 CW |
| LongDong | 8 | 1 | No rotation (8-wide horizontal bar) |
| Star | — | 2 | Static 2 states: plus / cross |
| Wall | — | 4 | Static 4 states: 2 cells cycling through corners |
| WeirdLong | — | 6 | Static 6 states: diagonal diagonal morphing |
| Box_ | — | 1 | No rotation (2×2 square) |
| FourByFour | — | 1 | No rotation (hollow 4×4 frame) |
| Die | — | 1 | No rotation (1×1) |
| Happy | — | 1 | No rotation (1×1) |

**Static table format** (Unit 1 code generation will fill these in):
```rust
// El piece — orientation 0: vertical bar + foot-right
const EL_ROT0: &[(i32,i32)] = &[(1,0),(1,1),(1,2),(2,2)];
// ... orientations 1, 2, 3 derived via generic 3×3 CW formula
```

**WeirdLong** has 6 orientations because the original has `orientations_ = 6` and 6 custom `rotate()` cases. Orientations 0–5 cycle and must be computed from the C++ state machine transitions and stored as 6 static arrays.

---

## ActivePiece

The piece currently falling.

```rust
pub struct ActivePiece {
    pub kind: PieceKind,
    pub x: i32,
    pub y: i32,
    pub rotation: u8,
}

impl ActivePiece {
    pub fn cells(&self) -> impl Iterator<Item = (i32, i32)>;  // absolute board positions
    pub fn can_move_to(&self, board: &Board, dx: i32, dy: i32) -> bool;
    pub fn can_rotate(&self, board: &Board, clockwise: bool) -> bool;
}
```

**Spawn position** (from BTGame.C:801):
`x = 5 - (kind.spawn_x_offset())`, `y = 0`, `rotation = 0`

For LongDong (bounding box 8 wide): `x = 5 - 8/2 = 1`, cells fill columns 1–8.

---

## Placement

Used by AI for exhaustive search (Unit 2). Declared in Unit 1 for stable API.

```rust
pub struct Placement {
    pub x: i32,
    pub rotation: u8,
}
```

---

## LinesCleared

Result of `Board::check_and_clear_lines()`.

```rust
pub struct LinesCleared {
    pub count: u32,       // 0–4 (or more with certain weapons)
    pub funds_earned: i32,
    pub happy_missed: bool,
}
```

---

## Score

Tracks all numeric game state for one player's view. Signed `funds` because Reagan weapon negates them.

```rust
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Score {
    pub score: u32,        // from hard drops only
    pub lines: u32,        // total lines cleared
    pub funds: i32,        // signed: Reagan negates, Keating zeros, Mondale taxes
    pub op_score: u32,
    pub op_lines: u32,
    pub op_funds: i32,
}

pub struct ScoreView {
    pub score: u32,
    pub lines: u32,
    pub funds: i32,
    pub op_score: u32,
    pub op_lines: u32,
    pub lines_until_bazaar: u32,
}
```

**Score increment from hard drop** (BTGame.C:729):
`score += Board::HEIGHT as u32 - current_y as u32`

---

## GamePhase

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GamePhase {
    Title,
    ConnectingToServer,
    WaitingForOpponent,
    Playing,
    InBazaar,
    Paused,
    GameOver { won: bool },
}
```

---

## PlayerInput

```rust
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PlayerInput {
    MoveLeft,
    MoveRight,
    RotateCW,
    RotateCCW,
    SoftDrop,
    HardDrop,
    LaunchWeapon(u8),   // weapon slot 0–9; Unit 2+
}
```

---

## GameEvent

Events emitted by `GameState::tick()` for the renderer and network layer to consume.

```rust
#[derive(Clone, Debug)]
pub enum GameEvent {
    PieceSpawned,
    PieceMoved,
    PieceLocked,
    LinesCleared { count: u32, funds_earned: i32 },
    HappyMissed,
    GameOver { won: bool },
    FundsChanged { new_funds: i32 },
    BazaarTriggered,         // Unit 2+
    WeaponActivated(u8),     // Unit 2+
    WeaponDeactivated(u8),   // Unit 2+
}
```

---

## PieceState

Sub-state of the active piece within `GamePhase::Playing`.

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceState {
    Dropping,         // piece is falling; drop timer active
    LockDelay(u32),   // ms remaining before lock; piece touched ground
    HardDropping,     // fast-drop in progress
}
```

---

## GameState (top-level)

```rust
pub struct GameState {
    pub phase: GamePhase,
    pub board: Board,
    pub active_piece: Option<ActivePiece>,
    pub next_piece: PieceKind,
    pub piece_state: PieceState,
    pub score: Score,
    pub drop_elapsed_ms: u32,   // ms since last gravity step
    pub lock_delay_ms: u32,     // ms elapsed since piece touched ground
    pub mode: GameMode,
    pub rng: SmallRng,
    // weapon_state: WeaponState,   // Unit 2
    // bazaar_state: BazaarState,   // Unit 2
}

pub enum GameMode {
    SinglePlayer,
    VsComputer,
    NetworkGame,
}
```

---

## Timing Constants

```rust
pub const DROP_INTERVAL_MS: u32 = 512;    // BT_DROP_TIME
pub const FAST_DROP_INTERVAL_MS: u32 = 10; // BT_FAST_DROP_TIME
pub const LOCK_DELAY_MS: u32 = 150;        // BT_SLIDE_TIME
pub const SPAWN_X: i32 = 5;               // BT_DEFAULT_X
pub const SPAWN_Y: i32 = 0;               // BT_DEFAULT_Y
pub const BOARD_WIDTH: i32 = 10;
pub const BOARD_HEIGHT: i32 = 28;
pub const HAPPY_FUND_VALUE: i32 = 150;    // BT_HAPPY_VAL
pub const LINES_UNTIL_BAZAAR: u32 = 20;   // BT_LINES_TIL_BAZ
pub const ELO_START: i32 = 1200;
pub const ARSENAL_SIZE: usize = 10;       // BT_ARSENAL_SIZE
```
