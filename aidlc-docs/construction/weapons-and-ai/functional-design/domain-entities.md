# Domain Entities — Unit 2: weapons-and-ai

## WeaponKind

34-variant enum in `battletris-engine::engine::weapons`, mirroring `BTWeaponToken` in enum-index order.

```
WeaponKind index → variant:
 0  FearedWeird      8  FlipOut      16 Reagan     24 Bottle
 1  FourByFour       9  Speedy       17 Ames        25 NoSlide
 2  Hatter          10  Missing      18 Ace         26 Susan
 3  Upbyside        11  PieceIt      19 Condor      27 Meadow
 4  Fallout         12  Blind        20 NiceDay     28 Mirror
 5  Swap            13  Mondale      21 SoLong      29 Twilight
 6  Lawyers         14  Keating      22 NoDice      30 Slick
 7  RiseUp          15  Carter       23  Bug        31 Broken
                                                    32 Force
                                                    33 Gimp
```

`WeaponKind::index() -> usize` — canonical enum index for array lookup.

---

## WeaponDef

Compiled-in definition for each weapon. All 34 are `static WEAPONS: [WeaponDef; 34]`.

```
WeaponDef {
    kind:        WeaponKind,
    name:        &'static str,
    description: &'static str,
    price:       u32,    // base price in funds
    duration:    u32,    // active duration in lines cleared by affected player; 0 = instant
}
```

### Complete Table (source: btweapons.db + btweaponsp.db)

| Variant | Name | Price | Duration |
|---------|------|-------|----------|
| FearedWeird | The Feared Weird | 400 | 3 |
| FourByFour | Four-by-Four | 425 | 10 |
| Hatter | The Mad Hatter | 375 | 5 |
| Upbyside | Upbyside-down | 125 | 10 |
| Fallout | Fallout | 250 | 10 |
| Swap | Swap Meet | 1200 | 0 |
| Lawyers | Lawyer's Delite | 350 | 5 |
| RiseUp | Rise Up | 75 | 0 |
| FlipOut | Flip Out | 15 | 0 |
| Speedy | Speedy Gonzales | 275 | 10 |
| Missing | Missing Pieces | 50 | 0 |
| PieceIt | Piece It Together | 100 | 0 |
| Blind | The Blind Cleric | 400 | 0 |
| Mondale | Mondale '96 | 150 | 50 |
| Keating | Keating Five | 425 | 0 |
| Carter | Carter Years | 250 | 20 |
| Reagan | Reagan Era | 425 | 0 |
| Ames | William Ames | 50 | 20 |
| Ace | Ace of Spies | 100 | 30 |
| Condor | The Condor | 225 | 40 |
| NiceDay | Have a Nice Day | 50 | 0 |
| SoLong | So Long | 100 | 10 |
| NoDice | No Dice | 600 | 35 |
| Bug | Bug Report | 320 | 0 |
| Bottle | Bottle Neck | 150 | 10 |
| NoSlide | Slide Denied | 125 | 10 |
| Susan | Lazy Susan | 600 | 0 |
| Meadow | Meadow | 475 | 10 |
| Mirror | Mirror Mirror | 500 | 10 |
| Twilight | The Twilight Zone | 450 | 0 |
| Slick | Slick Willy | 650 | 3 |
| Broken | Broken Record | 325 | 5 |
| Force | The Force | 325 | 5 |
| Gimp | The Gimp | 25 | 0 |

---

## ArsenalSlot

```
ArsenalSlot {
    kind:     WeaponKind,
    quantity: u8,    // ≥ 1; slot is removed when quantity hits 0
}
```

---

## Arsenal

```
Arsenal {
    slots: Vec<ArsenalSlot>,    // up to BT_ARSENAL_SIZE = 10 entries
}
```

Methods: `add(kind)`, `remove(kind) -> bool`, `slot_at(idx)`, `total_weapons() -> usize`, `is_full() -> bool`.

Stacking: adding a weapon already present increments its quantity instead of occupying a new slot.

---

## WeaponState

Tracks all active timed weapons on one player's board. Stored inside `GameState`.

```
WeaponState {
    remaining: [u32; 34],         // remaining_lines per weapon index; 0 = not active
    slick_dir: i32,               // +1 = right, -1 = left (Slick Willy)
    broken_kind: Option<PieceKind>, // piece that repeats (Broken Record)
    blind_cells: Vec<(usize,usize)>, // cells hidden by Blind Cleric
    mondale_rate: u8,             // 0 or 30 (percent) accumulated from active Mondale stacks
}
```

Helper: `is_active(kind: WeaponKind) -> bool` — `remaining[kind.index()] > 0`.

Duration countdown: on every line clear by this board's player, decrement all non-zero `remaining` entries by `lines_cleared`. Entries that reach 0 generate a `WeaponOff` event.

---

## BazaarState

Transient state during a bazaar pause. Lives in `GamePhase::InBazaar(BazaarState)`.

```
BazaarState {
    weapons:      Vec<WeaponKind>,   // all 34, sorted by price ascending
    selected:     usize,             // cursor index into weapons[]
    player_funds: u32,               // player's current funds (post-Reagan if applied)
    price_mult:   u8,                // 1 normally, 2 if Carter is active on player
    player_done:  bool,
    ernie_done:   bool,
}
```

`effective_price(kind) -> u32`: returns `WEAPONS[kind].price * price_mult as u32`.

---

## AiPenalties

Ernie's board evaluation weights. Ported directly from `BTComputer.C` `#define` constants.

```
AiPenalties {
    open_hole_penalty:    i64,   // default  7_000
    closed_hole_penalty:  i64,   // default 10_000
    covered_hole_penalty: i64,   // default  3_000
    height_penalty:       i64,   // default 30_000 (applied when top > midline row 14)
    line_bonus:           i64,   // default  5_000
    happy_bonus:          i64,   // default 20_000
    variance_penalty:     i64,   // default     50
}
```

---

## AiMove

```
AiMove {
    col:         i32,    // target column (spawn-offset adjusted)
    orientation: usize,  // rotation state index
}
```

---

## Ai

Ernie's runtime state. Owned by the Ernie task thread.

```
Ai {
    difficulty:       u8,             // 6 = "Focused" (750 ms); range 0-14
    delay_ms:         u64,            // think interval derived from difficulty
    penalties:        AiPenalties,
    can_purchase:     [bool; 34],     // per weapon; some start false
    can_launch:       [bool; 34],
    next_weapon:      Option<WeaponKind>,
    op_lines:         u32,            // opponent's total lines cleared
    bazaar_count:     u32,
    combo_cost:       Option<u32>,    // cost accumulator for buy-then-launch combos
    carter_active:    bool,
    upsidedown:       bool,
    no_nice_day:      u32,
    lawyers_active:   bool,
    arsenal:          Arsenal,
}
```

Initial `can_purchase` bans: `Ames, Ace, Condor, Meadow, Susan, Reagan` are `false` at game start.
Susan becomes purchasable once `op_lines >= 50`.
