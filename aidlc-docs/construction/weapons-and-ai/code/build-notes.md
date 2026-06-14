# Unit 2 Build Notes — weapons-and-ai

## Running

```bash
# Single-player (original Unit 1 mode)
cargo run -p battletris-client

# Vs Ernie (AI opponent on same machine)
cargo run -p battletris-client -- --vs-computer
```

## Key Design Decisions

**`WeaponState.remaining: Vec<u32>` not `[u32; 34]`**
serde only derives for arrays up to [T; 32]. WEAPON_COUNT=34 exceeds this limit.
Vec<u32> with manual Default (`vec![0u32; WEAPON_COUNT]`) solves this cleanly.

**Ernie GameState uses SinglePlayer mode**
Ernie's own `GameState` uses `GameMode::SinglePlayer` so its bazaar auto-closes when Ernie exits (no second party to wait for). Ernie sends `GameMessage::BazaarEnd` to signal game_loop that Ernie's shopping is done.

**Fallout via `lock_piece_filtered`**
Rather than changing `Board::occupied()` (which would require passing WeaponState everywhere), Fallout filters out cells in cols 2-7 at lock time in `lock_piece_filtered()`. The piece still *falls* through those columns; cells are just not placed there.

**Bottle via `Cell::Struct_`**
Bottle zone walls are materialised as `Cell::Struct_` cells by `fill_bottle_walls()` on weapon activation and removed by `clear_bottle_walls()` on deactivation. Normal `occupied()` returns true for those cells automatically, no WeaponState plumbing needed.

**Channel pattern (Q3=B)**
```
main.rs
  ├── game_loop thread  (owns player GameState)
  │     ├── to_ernie: SyncSender<GameMessage>
  │     └── from_ernie: Receiver<GameMessage>
  └── ernie thread      (owns Ernie GameState + Ai)
        ├── from_player: Receiver<GameMessage>   ← same as to_ernie
        └── to_player: SyncSender<GameMessage>   ← same as from_ernie
```
Unit 3 will replace the ernie thread with two TCP tasks (send + recv), matching the same channel pattern — game_loop needs no restructuring.

## Weapon Behaviours Not Fully Tested in Integration

- Gimp flash: event-driven one-shot; game_loop would need transient state to pulse the visual
- Susan/swap arsenal: game_loop receives `ArsenalSwapped` and needs to re-request both arsenals
- Reagan: funds can go negative; `Score.funds: i64` handles this

## Test Coverage

71 tests pass in battletris-engine. Client has no unit tests (renderer code).
