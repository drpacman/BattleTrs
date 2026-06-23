# Unit Test Execution — BattleTrisRs

## Run All Unit Tests (native host target)

```bash
cargo test --workspace
```

### Expected Results

| Crate | Tests | Coverage focus |
|-------|-------|---------------|
| `battletris-engine` | 78 | Board logic, piece rotation, scoring, weapons, bazaar, AI, protocol encode/decode |
| `battletris-server` | 11 | ELO calculation, player DB persistence, TCP framing, HTTP router, security headers |
| `battletris-client` | 0 | Rendering is manual-only |
| `battletris-web` | 0 | WASM crate; browser-only APIs cannot run natively |

**Total**: 89 tests, all pass, 0 failures.

---

## Individual Crate Tests

### Engine only

```bash
cargo test -p battletris-engine
```

Key test groups:
- `engine::board::tests` — board cell operations, line clears, special cells (Die, Happy, Twilight, Bug)
- `engine::piece::tests` — rotation states, ghost piece, collision
- `engine::game_state::tests` — gravity, lock delay, soft drop, hard drop, bazaar trigger, game over
- `engine::score::tests` — funds, bazaar line counter
- `engine::weapons::tests` — each weapon's apply logic, reflect, mirror, arsenal management
- `ai::tests` — Ernie evaluation, placement decisions
- `protocol::tests` — framed encode/decode, raw encode/decode, truncation handling

### Server only

```bash
cargo test -p battletris-server
```

Key test groups:
- `elo::tests` — rating delta calculation, floor clamping
- `db::tests` — PlayerDb create/load/save/apply-result
- `conn::tests` — TcpConn round-trip, MAX_FRAME_BYTES constant
- `http_server::tests` — 404 on missing file, security headers on served files

---

## Notes on battletris-web

The WASM crate has no automated unit tests. All browser APIs (`WebSocket`, `CanvasRenderingContext2d`, `KeyboardEvent`, `requestAnimationFrame`) are unavailable in native test runners. Testing is done via:

1. **Compilation check**: `cargo build -p battletris-web --target wasm32-unknown-unknown` — verifies all Rust code compiles to WASM
2. **Manual integration test**: Browser smoke test documented in `integration-test-instructions.md`

---

## Re-run After Changes

After modifying any engine or server code, always re-run the full suite:

```bash
cargo test --workspace 2>&1 | grep "test result"
```

All lines should show `ok` with 0 failed.
