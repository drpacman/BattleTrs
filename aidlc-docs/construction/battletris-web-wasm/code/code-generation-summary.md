# Code Generation Summary — Unit B: battletris-web WASM

**Date**: 2026-06-22  
**Status**: COMPLETED  
**Build**: 0 errors, 0 warnings  
**Tests**: 89/89 pass (all pre-existing; WASM crate has no native tests by design)

---

## Plan Execution Checklist

**Part 1 — Build config**
- [x] `battletris-web/Cargo.toml` — wasm-bindgen, js-sys, web-sys (13 features), battletris-engine, bincode, serde, console_error_panic_hook, rand
- [x] `battletris-web/Trunk.toml` — `target = "index.html"`
- [x] `battletris-web/index.html` — canvas#game-canvas 820×860, dark background, pixelated rendering

**Part 2 — WASM modules**
- [x] `battletris-web/src/input.rs` — InputHandler with owned keydown/keyup closures, drain() returning raw code strings, prevent_default for scroll keys
- [x] `battletris-web/src/transport.rs` — WsTransport with BinaryType::Arraybuffer, Hello sent from on_open, owned closures stored as fields
- [x] `battletris-web/src/renderer/mod.rs` — CanvasRenderer, 5×7 bitmap font (65 glyphs, exact copy from SDL2 font.rs), layout constants, draw_cell/draw_board/draw_active_piece/draw_ghost_piece/draw_next_piece/draw_die_pips/draw_face/draw_text/text_w
- [x] `battletris-web/src/renderer/playing.rs` — draw_playing, draw_board_with_effects (gimp flash, upbyside flip, blind cells, twilight, bug cells), draw_weapon_chips, draw_stats (NEXT piece, PLAYER/OPPONENT panels, arsenal slots, SLICK indicator)
- [x] `battletris-web/src/renderer/overlay.rs` — draw_bazaar (scrollable weapon list, description box, controls bar), draw_quit_confirm
- [x] `battletris-web/src/renderer/screens.rs` — draw_connecting, draw_waiting, draw_name_taken, draw_disconnected, draw_game_over (winner name, score/lines, ELO delta)
- [x] `battletris-web/src/app.rs` — WasmApp, WebPhase enum (Connecting/WaitingForOpponent/InGame/NameTaken/Disconnected), tick() with quit confirm/game-over intercept, process_message() porting game_loop.rs logic, forward_events(), render_in_game(), key_to_input() matching SDL2 scancode map, apply_board_visibility(), ws_url_from_location() with ?server= override
- [x] `battletris-web/src/lib.rs` — start() with panic hook, thread_local APP+TICK, schedule_next_frame() rAF pattern (closure never dropped)

**Part 3 — Build verify**
- [x] `cargo build --workspace` — 0 errors, 0 warnings (native target)
- [x] `cargo build -p battletris-web --target wasm32-unknown-unknown` — 0 errors, 0 warnings
- [x] `cargo test --workspace` — 89/89 pass

---

## Key Implementation Notes

- `ArrayBuffer`/`Uint8Array` are from `js-sys`, not `web-sys` — feature list corrected during build
- `PieceKind` is at `battletris_engine::engine::piece::PieceKind` (not re-exported from game_state)
- `[profile.release]` moved to workspace `Cargo.toml` (package-level profiles are ignored with warning)
- Ghost piece uses `stroke_rect` (outline) matching SDL2's `draw_rect` outline-only approach
- Font rendering: characters uppercased before lookup (ASCII 32–96 range), matching SDL2 font.rs behaviour
- Bazaar is drawn as full-screen overlay (skips draw_playing underneath), matching SDL2 client
- Quit confirm → page reload (cleanest disconnect in a single-page WASM app)
- Game over → page reload on Enter/Space (re-prompts name on fresh start)

---

## Dev Workflow

```bash
# Start server (from workspace root)
cargo run -p battletris-server -- serve --web-dir battletris-web/dist

# In another terminal, build and serve WASM (from battletris-web/)
trunk serve  # hot reload at http://localhost:8080?server=ws://localhost:7001
```
