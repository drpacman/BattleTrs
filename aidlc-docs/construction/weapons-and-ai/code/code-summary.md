# Unit 2 Code Summary — weapons-and-ai

## Files Changed / Created

| File | Status | Description |
|------|--------|-------------|
| `battletris-engine/src/engine/board.rs` | Extended | Cell::Bug/Twilight, board effect methods |
| `battletris-engine/src/engine/piece.rs` | Extended | random_filtered weapon-aware piece selector |
| `battletris-engine/src/engine/score.rs` | Rewritten | funds: i64, Mondale tax, op stats |
| `battletris-engine/src/engine/game_state.rs` | Rewritten | WeaponState, InBazaar phase, weapon tick |
| `battletris-engine/src/engine/weapons.rs` | NEW | WeaponKind, WeaponDef, WEAPONS[34], Arsenal, WeaponState, instant/timed effects, BazaarState |
| `battletris-engine/src/engine/mod.rs` | Extended | pub mod weapons |
| `battletris-engine/src/ai/mod.rs` | Rewritten | Ai struct, AiPenalties, evaluate(), decide(), go_shopping() |
| `battletris-engine/src/protocol/mod.rs` | Rewritten | 14 typed GameMessage variants |
| `battletris-client/src/main.rs` | Extended | --vs-computer flag, ernie thread spawn |
| `battletris-client/src/game_loop.rs` | Rewritten | ErnieChannels, weapon forwarding, InBazaar routing |
| `battletris-client/src/ernie.rs` | NEW | Ernie task: GameState + Ai + 750ms think loop |
| `battletris-client/src/renderer/bazaar.rs` | NEW | draw_bazaar() scrollable overlay |
| `battletris-client/src/renderer/playing.rs` | Rewritten | draw_board_with_effects, weapon chips, arsenal, effects |
| `battletris-client/src/renderer/mod.rs` | Extended | pub mod bazaar, Bug/Twilight cell_color |

## Test Coverage (71 tests)

| Module | Tests |
|--------|-------|
| engine::board | 13 (rise_up, flip_out, bottle, bug, twilight, etc.) |
| engine::piece | 9 (rotation, ghost, random_filtered) |
| engine::score | 5 (Mondale, tax, funds negative, bazaar trigger) |
| engine::game_state | 10 (gravity, lock, bazaar, ai_place) |
| engine::weapons | 20 (activation, mirror, arsenal, bazaar, all 34 weapons indexed) |
| ai | 9 (column_tops, holes, evaluate, decide, go_shopping) |

## Architecture Diagram

```
main.rs
 ├── Keycode → PlayerInput channel
 ├── RenderEvent channel (capacity 2)
 ├── game_loop thread
 │    ├── GameState (player)
 │    │    ├── Board (10×28)
 │    │    ├── WeaponState (34-slot Vec<u32>)
 │    │    ├── Arsenal (Vec<ArsenalSlot>)
 │    │    └── Score (funds: i64)
 │    ├── to_ernie: SyncSender<GameMessage>
 │    └── from_ernie: Receiver<GameMessage>
 └── ernie thread (--vs-computer only)
      ├── GameState (Ernie, SinglePlayer mode)
      ├── Ai (penalties + can_purchase[34] + can_launch[34])
      └── 750ms think loop → ai_place_piece()
```

## Weapon Reference

34 weapons across 4 categories (as indexed in WeaponKind repr(usize)):
- **Board attack** (0-11): RiseUp, Lawyers, Ames, Ace, Condor, Fallout, Meadow, Upbyside, Force, Bottle, Swap, Mondale
- **Piece manipulation** (12-22): Hatter, NiceDay, SoLong, Carter, Broken, FW, FBF, NoDice, Keating, Reagan, Gimp
- **Player support** (23-28): Spy, Bug, Blind, PieceIt, Missing, Susan
- **Visual effect** (29-33): Twilight, Slick, Speedy, FlipOut, Mirror
