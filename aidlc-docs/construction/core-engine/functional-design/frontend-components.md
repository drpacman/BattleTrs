# Frontend Components — Unit 1: core-engine (SDL2 Renderer)

Cell size: **28×28 px** (Q1=B). Board pixel area: 280×784 px.

---

## Window Layout

```
+------------------------------------------------------------------+
| BattleTris                                              [window] |
|  (20px top padding)                                              |
|                                                                  |
|  [Player Board]     [Stats Panel]    [Opponent Board]            |
|  280 × 784          200 × 784         280 × 784                  |
|  x=20, y=40         x=310, y=40       x=520, y=40               |
|                                                                  |
+------------------------------------------------------------------+
  Total window: 820 × 860 px
```

**Window size**: 820 × 860 px (fixed; no resize in Unit 1)

**Background**: Black (`RGB(0, 0, 0)`)

---

## Component 1: TitleScreen

**Visible when**: `GamePhase::Title`

**Elements**:
- Title text "BATTLETRIS" centered, large font
- Version subtitle "v1.0 (Rust Port)"
- Prompt: "Press Enter to play vs Computer" (Unit 1 shows single-player only)
- Prompt: "Press N to play vs Network" (grayed out in Unit 1)

**Interaction**: Enter key → transitions to `GamePhase::Playing` in single-player mode

---

## Component 2: PlayingScreen

**Visible when**: `GamePhase::Playing`

Composed of sub-components:

### 2a. PlayerBoard

Origin: `(20, 40)`. Size: `280 × 784` px.

**Rendering**:
- Black background fill
- Iterate all 28 rows, 10 columns
- For each cell: draw a filled rectangle `(x*28 + 20, y*28 + 40, 28, 28)` with 1px dark border
- Cell color mapping:

| Cell | Color |
|------|-------|
| `Empty` | Black (no draw) |
| `Regular(1)` | Ivory `RGB(255, 255, 230)` |
| `Regular(2)` | Yellow `RGB(255, 220, 0)` |
| `Regular(3)` | Red `RGB(220, 50, 50)` |
| `Regular(4)` | Blue `RGB(50, 100, 220)` |
| `Regular(5)` | Orange `RGB(255, 140, 0)` |
| `Regular(6)` | Green `RGB(50, 200, 50)` |
| `Regular(7)` | Cyan `RGB(0, 200, 220)` |
| `Regular(8)` | Purple `RGB(160, 50, 200)` |
| `Die(pips)` | White background + pip dots |
| `Happy` | White background + smiley face |
| `HappyMissed` | Light gray background + frown face |

**Cell border**: 1px darker shade of cell color. Cells have a small inset appearance.

**Active piece**: Drawn on top of the board using the same color logic, using `active_piece.cells()` absolute positions.

**Ghost piece** (drop preview): Draw active piece cells at the hard-drop landing row using the piece color at 30% opacity.

### 2b. OpponentBoard

Origin: `(520, 40)`. Size: `280 × 784` px.

**Unit 1**: Renders the `BoardSnapshot` from the opponent (or is blank/grey-filled in single-player mode with a border and "CPU" label).

Same rendering rules as PlayerBoard. In network games (Unit 3), this board updates from `RenderEvent::OpponentBoardUpdate(BoardSnapshot)`.

### 2c. StatsPanel

Origin: `(310, 40)`. Size: `200 × 784` px.

**Sections** (top to bottom, with ~10px padding):

```
[NEXT PIECE]
  4×4 preview box at (320, 50)
  shows next_piece rendered in miniature (14px cells)

[PLAYER]
  Score:   12345
  Lines:   42
  Funds:   $320

[LINES TIL BAZ]
  17

[OPPONENT]
  Score:   9876
  Lines:   38
  Funds:   (hidden until Condor/Ames/Ace; shows "???" otherwise)

[ARSENAL]   (Unit 2 — blank in Unit 1, reserved space)
  Slot 0: [empty]
  ...
  Slot 9: [empty]
```

**Font**: Monospace or pixel font. Label+value on same line; left-aligned labels.

**Colors**: White text on dark gray panel background `RGB(30, 30, 30)`.

**"Lines til baz" counter**: Large, yellow when ≤ 5 remaining.

### 2d. NextPiecePreview

Rendered inside the StatsPanel, above player stats.

- 4×4 cell grid (112×112 px at 28px cells; shrink to 14px cells for preview = 56×56 px)
- Piece drawn centered in the preview box
- Border: thin white rectangle

---

## Component 3: GameOverScreen

**Visible when**: `GamePhase::GameOver { won }`

**Elements**:
- Full-window overlay with semi-transparent dark background
- Large text: "YOU WIN!" (green) or "GAME OVER" (red)
- Stats: final score, lines cleared, max funds reached
- Prompt: "Press Enter to play again" / "Press Esc for title"

---

## Component 4: ConnectingScreen (Unit 3 stub)

**Visible when**: `GamePhase::ConnectingToServer` or `GamePhase::WaitingForOpponent`

**Unit 1**: Not rendered; phases don't exist in single-player. Stub exists as a variant but `Renderer::render()` returns early for these phases in Unit 1.

---

## RenderEvent

The game-tick thread sends `RenderEvent` to the SDL2 main thread via `mpsc::Sender<RenderEvent>`. The SDL2 thread calls `Renderer::render(event)` on each event.

```rust
pub enum RenderEvent {
    Title,
    Playing(PlayingView),
    GameOver { won: bool, score: u32, lines: u32 },
    // ConnectingToServer,     // Unit 3
    // WaitingForOpponent,     // Unit 3
    // InBazaar(BazaarView),   // Unit 2
}

pub struct PlayingView {
    pub player_board: BoardSnapshot,
    pub active_piece: Option<(PieceKind, Vec<(i32,i32)>)>,  // kind + absolute cells
    pub ghost_cells: Vec<(i32,i32)>,
    pub next_piece: PieceKind,
    pub score: ScoreView,
    pub opponent_board: Option<BoardSnapshot>,  // None in single-player
}
```

**Channel capacity**: `sync_channel(2)` for `RenderEvent` — newest frame wins; the SDL2 thread drains with `try_recv()` and takes the last event before rendering. No frame-by-frame synchronization needed.

---

## SDL2 Thread Loop (main.rs)

```
create SDL2 window (820×860)
create Renderer
spawn game-tick thread (sends RenderEvent via render_tx)
spawn tokio runtime (net tasks, Unit 3)

loop:
    for event in sdl2.event_pump():
        if keyboard → translate to PlayerInput → send to game-tick thread
        if Quit → break

    if let Ok(render_event) = render_rx.try_recv():
        renderer.render(render_event)

    sdl2.window.present()
    sleep(1ms)  // cap at ~1000 fps; actual frame rate governed by tick thread
```

---

## Color Palette Summary

| BT Constant | Color | RGB |
|-------------|-------|-----|
| BT_IVORY (1) | Ivory | (255, 255, 230) |
| BT_YELLOW (2) | Yellow | (255, 220, 0) |
| BT_RED (3) | Red | (220, 50, 50) |
| BT_BLUE (4) | Blue | (50, 100, 220) |
| BT_ORANGE (5) | Orange | (255, 140, 0) |
| BT_GREEN (6) | Green | (50, 200, 50) |
| BT_CYAN (7) | Cyan | (0, 200, 220) |
| BT_PURPLE (8) | Purple | (160, 50, 200) |
| Die background | White | (255, 255, 255) |
| Happy background | White | (255, 255, 255) |
| HappyMissed | Light gray | (180, 180, 180) |
| Board background | Black | (0, 0, 0) |
| Panel background | Dark gray | (30, 30, 30) |

---

## Keyboard Mapping

| Key | PlayerInput |
|-----|-------------|
| Left arrow | MoveLeft |
| Right arrow | MoveRight |
| Up arrow | RotateCW |
| Z | RotateCCW |
| Down arrow | SoftDrop |
| Space | HardDrop |
| 0–9 | LaunchWeapon(n) (Unit 2+) |
| P | Pause |
| Esc | Quit to title |
