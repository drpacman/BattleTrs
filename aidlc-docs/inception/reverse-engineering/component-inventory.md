# Component Inventory — BattleTris

## Source Modules

| Module | Type | Purpose |
|--------|------|---------|
| game/ | Application | Main game client binary (BattleTris) |
| daemons/ | Server | btserverd (master) + btslaved (per-pair relay) |
| db/ | Shared Library | BTDB player/network database |
| widget/ | Platform Library | X11/Motif UI widget wrappers |
| sockets/ | Shared Library | TCP socket abstraction + Xt event loop integration |
| stdlib/ | Shared Library | Custom template containers (List, Block, BTStack, BTRingNode) |
| audio/ | Platform Library | Solaris /dev/audio interface (stub everywhere else) |
| signals/ | Shared Library | POSIX signal handling |
| btref/ | Utility | Admin CLI for player database management |
| share/ | Data | Weapon database, X resources, art assets |
| art/ | Data | PPM/XPM/XBM image files |
| man/ | Documentation | Unix man pages |

## Binaries Produced

| Binary | Source | Purpose |
|--------|--------|---------|
| BattleTris | game/ | Game client (run by each player) |
| btserverd | daemons/btserverd.C | Master server daemon (port 4404) |
| btslaved | daemons/btslaved.C | Per-connection relay slave daemon |
| btref | btref/ | Admin CLI for player DB |

## Piece Types Inventory (18 total)

| Piece ID | Constant | Description | Notes |
|----------|----------|-------------|-------|
| 1 | BT_EL_PIECE | L-piece | Standard Tetris |
| 2 | BT_REL_PIECE | Reverse L-piece | Standard Tetris |
| 3 | BT_SL_RT_PIECE | S-piece (right) | Standard Tetris |
| 4 | BT_SL_LF_PIECE | S-piece (left) | Standard Tetris |
| 5 | BT_LONG_PIECE | I-piece (long) | Standard Tetris |
| 6 | BT_PLUG_PIECE | T-piece | Standard Tetris |
| 7 | BT_BOX_PIECE | O-piece (box) | Standard Tetris |
| 8 | BT_DIE_PIECE | Die (1×1, 1-6 pips) | Earns funds when line cleared |
| 9 | BT_HAP_PIECE | Happy face (1×1) | 150 funds if cleared same turn |
| 10 | BT_DOG_PIECE | Dog piece | Weird (FEARED_WEIRD weapon) |
| 11 | BT_RDOG_PIECE | Reverse dog piece | Weird |
| 12 | BT_CAP_PIECE | Cap piece | Weird |
| 13 | BT_WALL_PIECE | Wall piece | Weird, special rotation |
| 14 | BT_TOWER_PIECE | Tower piece | Weird |
| 15 | BT_STAR_PIECE | Star piece | Weird, special rotation |
| 16 | BT_WLONG_PIECE | Weird long piece | Weird, special rotation |
| 17 | BT_4x4_PIECE | 4×4 block | FOUR_BY_FOUR weapon |
| 18 | BT_LONG_DONG_PIECE | Long dong piece | Special weapon piece |

## Weapons Inventory (34 BTWeaponToken values, loaded from btweapons.db)

Active weapons tracked in `BTActive[BT_MAX_WEAPONS]` array. Duration measured in lines cleared by the affected player.

| # | Token | Name | Effect Category |
|---|-------|------|-----------------|
| 0 | BT_FEARED_WEIRD | Feared Weird | Piece disruption |
| 1 | BT_FOUR_BY_FOUR | Four by Four | Piece disruption |
| 2 | BT_HATTER | Hatter | Board disruption |
| 3 | BT_UPBYSIDE | Up By Side | Board orientation |
| 4 | BT_FALL_OUT | Fall Out | Board boundary |
| 5 | BT_SWAP | Swap | Board swap |
| 6 | BT_LAWYERS | Lawyers | Fund drain |
| 7 | BT_RISE_UP | Rise Up | Board fill |
| 8 | BT_FLIP_OUT | Flip Out | Board mirror |
| 9 | BT_SPEEDY | Speedy | Drop rate |
| 10 | BT_MISSING | Missing | Piece restriction |
| 11 | BT_PIECE_IT | Piece It | Piece disruption |
| 12 | BT_BLIND | Blind | Visibility |
| 13 | BT_MONDALE | Mondale | Political |
| 14 | BT_KEATING | Keating | Political |
| 15 | BT_CARTER | Carter | Fund drain |
| 16 | BT_REAGAN | Reagan | Political |
| 17 | BT_AMES | Ames | Political |
| 18 | BT_ACE | Ace | Named |
| 19 | BT_CONDOR | Condor | Mystery |
| 20 | BT_NICE_DAY | Nice Day | Ironic piece |
| 21 | BT_SO_LONG | So Long | Named |
| 22 | BT_NO_DICE | No Dice | Remove die pieces |
| 23 | BT_BUG | Bug | Named |
| 24 | BT_BOTTLE | Bottle | Piece bottleneck |
| 25 | BT_NO_SLIDE | No Slide | Control disable |
| 26 | BT_SUSAN | Susan | Named |
| 27 | BT_MEADOW | Meadow | Named |
| 28 | BT_MIRROR | Mirror | Control mirror |
| 29 | BT_TWILIGHT | Twilight | Named |
| 30 | BT_SLICK | Slick | Board slipperiness |
| 31 | BT_BROKEN | Broken | Piece disruption |
| 32 | BT_FORCE | Force | Named |
| 33 | BT_GIMP | Gimp | Named |

## Timer Callbacks (Game Loop Events)

| ID | Constant | Purpose |
|----|----------|---------|
| 0 | BT_DROP_TIMEOUT | Piece falling tick (base: 512ms, fast: 10ms) |
| 1 | BT_SLIDE_TIMEOUT | Horizontal movement animation (150ms) |
| 2 | BT_SLICK_TIMEOUT | SLICK weapon — board rotation effect |
| 3 | BT_HATTER_TIMEOUT | HATTER weapon — piece removal tick |
| 4 | BT_JEOPARDY_TIMEOUT | BLIND weapon — jeopardy/darkness effect |

## Colors Inventory

9 standard piece colors (plus dark variants and special cell types):
- BT_BLACK(0), BT_IVORY(1), BT_YELLOW(2), BT_RED(3), BT_BLUE(4), BT_ORANGE(5), BT_GREEN(6), BT_CYAN(7), BT_PURPLE(8)
- Special: BT_STRUCT(20), BT_HAPPY(21), BT_UNHAPPY(22), BT_GIMP_ID(23), BT_DIE_1..6(24-29)

## Total Count

| Category | Count |
|----------|-------|
| Source modules | 10 |
| Binaries | 4 |
| C++ source files (.C) | ~75 |
| Header files (.H) | ~80 |
| Piece types | 18 |
| Weapon types | 34 |
| Game timer types | 5 |
| Wire protocol tokens | ~50 (BTToken + BTWeaponToken) |
