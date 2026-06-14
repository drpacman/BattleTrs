# Frontend Components — Unit 2: weapons-and-ai

## Layout Changes from Unit 1

Unit 2 activates the opponent board (Ernie) and adds:
- Arsenal display in the stats panel (replacing placeholder slots)
- Active weapon indicators on each board
- Bazaar screen overlay
- Visual weapon effects (Upbyside, Twilight, Blind, Slick overlay, Gimp flash)

Window size unchanged: **820×860**. Board positions unchanged.

---

## 1. Extended PlayingView (engine → renderer data contract)

```
PlayingView {
    // --- unchanged from Unit 1 ---
    player_board:    BoardSnapshot,
    active_piece:    Option<(PieceKind, Vec<(i32,i32)>)>,
    ghost_cells:     Vec<(i32,i32)>,
    next_piece:      PieceKind,
    score:           ScoreView,
    opponent_board:  Option<BoardSnapshot>,   // NOW ALWAYS Some (Ernie's board)

    // --- new in Unit 2 ---
    player_active_weapons:  Vec<ActiveWeaponView>,   // active weapons on player
    ernie_active_weapons:   Vec<ActiveWeaponView>,   // active weapons on Ernie
    player_arsenal:         Vec<ArsenalSlotView>,    // player's weapons (up to 10)
    ernie_arsenal_count:    usize,                   // how many weapons Ernie holds
    show_opponent_funds:    bool,                    // Ames/Ace/Condor active
    opponent_board_accuracy: f32,                    // 1.0 Condor, 0.8 Ace, 0.0 none
    upbyside_active:        bool,                    // player's board is inverted
    ernie_upbyside:         bool,
    blind_cells:            Vec<(usize,usize)>,      // cells to render as empty on player board
    twilight_active:        bool,                    // player's board all-grey
    gimp_flash:             bool,                    // overlay on Ernie's board
    slick_active:           bool,
}

ActiveWeaponView {
    name:           &'static str,
    remaining_lines: u32,
}

ArsenalSlotView {
    kind:     WeaponKind,
    name:     &'static str,
    quantity: u8,
    key:      char,   // '1'-'9','0'
}
```

ScoreView gains: `ernie_score: u32`, `ernie_lines: u32`, `ernie_funds: Option<i64>` (None unless spy active).

---

## 2. Stats Panel — Extended Arsenal Display

Stats panel (STATS_X=310, width=190) replaces Unit 1's "ARSENAL: 0: --" placeholders.

```
// Arsenal section replaces the placeholder loop
section_label("ARSENAL");
for (i, slot) in view.player_arsenal.iter().enumerate() {
    let key = if i < 9 { b'1' + i as u8 } else { b'0' } as char;
    let quantity_str = if slot.quantity > 1 { format!("({})", slot.quantity) } else { "".into() };
    draw_text(canvas, &format!("{}: {} {}", key, slot.name, quantity_str), sx+5, sy, color, 1);
    sy += 14;
}
if view.player_arsenal.is_empty() {
    draw_text(canvas, "--", sx + 60, sy, dim_color, 2);
    sy += 18;
}
```

Scale 1 (6px per char) is used for arsenal slot text to fit long weapon names in 190px.

---

## 3. Active Weapon Indicators

Drawn below each board, between the board bottom and window bottom.

```
// Player active weapons: below player board (y = PLAYER_BOARD_Y + BOARD_PX_H + 4)
for (i, wpn) in view.player_active_weapons.iter().enumerate() {
    let x = PLAYER_BOARD_X + i as i32 * 90;
    let y = PLAYER_BOARD_Y + BOARD_PX_H as i32 + 4;
    // background chip
    canvas.fill_rect(Rect::new(x, y, 86, 16));
    draw_text(canvas, wpn.name, x+2, y+2, yellow, 1);
    draw_text(canvas, &format!("{}L", wpn.remaining_lines), x+66, y+2, white, 1);
}

// Ernie active weapons: below Ernie board
// Same layout at OPP_BOARD_X
```

Chips are 86×16px. Up to 3 visible before truncation (most games won't have more than 3 simultaneous).

---

## 4. Opponent Board Rendering (Ernie active)

```
pub fn draw_board_with_effects(
    canvas:         &mut Canvas<Window>,
    snapshot:       &BoardSnapshot,
    origin_x:       i32,
    origin_y:       i32,
    twilight:       bool,
    blind_cells:    &[(usize,usize)],
    accuracy:       f32,       // 1.0 = full, 0.8 = 80%, 0.0 = hidden
    rng:            &mut SmallRng,
) {
    for (row,col) in cells:
        let cell = snapshot.cells[row][col]
        let hidden = twilight || blind_cells.contains(&(row,col)) || Cell::Twilight
        if hidden { draw grid dot; continue }
        if accuracy < 1.0 && rng.gen::<f32>() > accuracy { draw wrong color; continue }
        draw_cell / draw_active_piece as usual
}
```

Ernie's board is drawn via `draw_board_with_effects` with `accuracy = view.opponent_board_accuracy` and `twilight = view.ernie_upbyside` (Upbyside inverts draw order for Ernie's board).

---

## 5. Upbyside Visual Effect

When `view.upbyside_active` is true for the player's board:
- Draw rows in reverse: row BOARD_ROWS-1 at canvas Y=PLAYER_BOARD_Y, row 0 at canvas Y=PLAYER_BOARD_Y+BOARD_PX_H-CELL_PX
- Active piece Y coord is inverted: `canvas_y = PLAYER_BOARD_Y + (BOARD_ROWS-1 - piece_row) * CELL_PX`
- Active-weapon indicator text: draw "UPSIDE DOWN" label in red on the board header

```
if view.upbyside_active {
    draw_text(canvas, "UPSIDE DOWN", PLAYER_BOARD_X+40, PLAYER_BOARD_Y-18, red, 2);
}
```

---

## 6. Twilight Visual Effect

When `view.twilight_active` is true for the player's board:
- All board cells render as `Cell::Empty` (grid dot only)
- The active piece itself is still visible (renderer draws it from `active_piece` not the snapshot)
- Indicate state: draw "TWILIGHT" label above board in dark purple

---

## 7. Gimp Flash Overlay

When `view.gimp_flash` is true:
```
// Draw 80% opaque dark overlay on Ernie's board
canvas.set_draw_color(Color::RGBA(0, 0, 0, 200));
canvas.fill_rect(Rect::new(OPP_BOARD_X, OPP_BOARD_Y, BOARD_PX_W, BOARD_PX_H));
// Draw "GIMP!" in large text, centred
let text = "GIMP!";
draw_text(canvas, text, OPP_BOARD_X + (BOARD_PX_W as i32 - text_w(text,5))/2,
          OPP_BOARD_Y + BOARD_PX_H as i32/2 - 18, Color::RGB(255,50,50), 5);
```

Flash duration: 2 seconds tracked by `gimp_flash_until: Option<Instant>` in game_loop.

---

## 8. Bazaar Screen (new: renderer/bazaar.rs)

Full-window overlay drawn on top of the playing background.

### Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  ████████████████████████████████████████████████████████████████████  │
│  █                     BAZAAR                                        █  │  y=80
│  ████████████████████████████████████████████████████████████████████  │
│                                                                         │
│   FUNDS: $1234                                                y=130     │
│                                                                         │
│   ►  1. The Feared Weird .............. $400                  y=165     │
│      2. Four-by-Four .................. $425                  y=185     │
│      3. The Mad Hatter ................ $375                  y=205     │
│      ...                                                                 │
│      34. The Gimp ..................... $25                              │
│                                                                         │
│   ── Description ──────────────────────────────────            y=700    │
│   Gives your opponent bizarre, disjointed pieces.                       │
│   None of the pieces are easily placed...                               │
│                                                                         │
│   [ENTER] Buy    [UP/DOWN] Select    [ESC] Done               y=800    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Dimensions
- Overlay background: full window 820×860, dark purple `RGB(10,0,20)`
- Title bar: y=80, full-width, height=50
- Weapon list: y=165, each row 20px, max 24 rows visible; scrolls when > 24 entries
- Selector cursor: `►` character (or filled rect) at x=30
- Description box: y=700, height=70, scale=1 text
- Controls bar: y=800

### Scroll Logic
```
const VISIBLE_ROWS: usize = 24;
let scroll_offset = selected.saturating_sub(VISIBLE_ROWS / 2)
    .min(34usize.saturating_sub(VISIBLE_ROWS));
for i in scroll_offset..(scroll_offset + VISIBLE_ROWS).min(34) {
    draw weapon row at y = 165 + (i - scroll_offset) * 20
}
```

### Colour coding
- Affordable weapon (funds >= effective_price): white text
- Unaffordable: grey `RGB(80,80,80)`
- Selected row: highlighted with background `RGB(60,0,120)`
- Cursor: `►` in yellow

### Input Handling (bazaar phase)
| Key | Action |
|-----|--------|
| Up | selected = selected.saturating_sub(1) |
| Down | selected = (selected + 1).min(33) |
| Enter | attempt purchase; re-render |
| Esc | player_done = true; close bazaar |

### Purchase Feedback
- On successful buy: brief flash of green on purchased weapon row (2 frames)
- Show updated funds after each purchase
- Weapon quantity shown: if already owned, "(2)" suffix in grey

---

## 9. ScoreView Extension

```
ScoreView (Unit 2 additions):
    ernie_score:      u32,
    ernie_lines:      u32,
    ernie_funds:      Option<i64>,   // Some when spy weapon active
    funds:            i64,           // was u32 in Unit 1, now i64 for Reagan
    funds_until_baz:  i32,           // NOT used in Unit 2; bazaar is line-triggered
```

The stats panel `FUNDS` row:
- If `funds < 0`: render in red with negative sign: `$-450` (Reagan Era active)
- If `funds >= 0`: render normally in white

---

## 10. `--vs-computer` Flag Wiring

`main.rs` parses `std::env::args()`. If `--vs-computer` is present:
- Set `game_mode = GameMode::VsComputer`
- Spawn Ernie task thread with `mpsc::channel<GameMessage>` pair
- Pass Ernie's sender to game_loop for dispatching weapon/board messages
- Pass Ernie's receiver to the Ernie task for receiving state updates

Without `--vs-computer` flag: game runs in single-player mode (Unit 1 behaviour).
