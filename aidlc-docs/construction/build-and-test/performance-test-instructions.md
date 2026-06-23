# Performance Test Instructions — BattleTrisRs

## Status: N/A

Performance testing is not applicable for this project phase.

**Rationale**:
- BattleTrisRs is a 1v1 LAN game with a maximum of 2 simultaneous connections per server instance
- The relay server has no database queries on the hot path (board sync is pure in-memory message passing)
- The WASM client runs in a requestAnimationFrame loop at ~60fps — frame budget is ≥16ms which is more than sufficient for the canvas 2D rendering workload
- No SLA or latency targets were specified in the requirements (Q5=C: network only; no performance NFRs defined)

## Informal Playability Targets (Observed)

These are not automated tests, but are verified during manual integration testing:

| Metric | Target | Verification |
|--------|--------|--------------|
| Input-to-render latency | ≤ 2 frames (~33ms) | Subjective feel during play |
| Board sync round-trip | ≤ 100ms on LAN | Piece lock → opponent board update visible in next frame |
| Server memory per game | < 10 MB | `ps aux` while a game is running |
| WASM bundle size (release) | < 2 MB `.wasm` | `ls -lh battletris-web/dist/*.wasm` |

## WASM Bundle Size Check

After `trunk build --release`:

```bash
ls -lh battletris-web/dist/*.wasm
```

A release-optimised WASM binary with `opt-level = "s"` and `lto = true` should be well under 2 MB for this codebase.
