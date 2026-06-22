# Build Instructions — BattleTrisRs (Browser Client Feature)

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust toolchain | stable ≥ 1.75 | All crates |
| `wasm32-unknown-unknown` target | (bundled with toolchain) | WASM compilation |
| `trunk` | ≥ 0.21 | WASM packaging and dev server |
| SDL2 | system library | Desktop client (battletris-client) |

### Install wasm32 target (if not already present)

```bash
rustup target add wasm32-unknown-unknown
```

### Install trunk

```bash
cargo install trunk
```

> Trunk is not included in the workspace. It must be installed separately. Installation takes 2–5 minutes the first time.

---

## Unit A — Server (battletris-server)

The server requires no extra tooling beyond the standard Rust toolchain.

```bash
# Build release binary
cargo build -p battletris-server --release

# Output: target/release/battletris-server
```

**Start the server (development)**:

```bash
# TCP game on :7000, HTTP+WS on :7001, serving dist/ as static files
cargo run -p battletris-server -- serve \
    --port 7000 \
    --web-port 7001 \
    --web-dir battletris-web/dist
```

The `--web-dir` path must exist before the server starts (it rejects a missing directory at boot).

---

## Unit B — WASM Client (battletris-web)

### Development build + dev server (hot reload)

```bash
cd battletris-web
trunk serve
# Serves at http://localhost:8080
# Connect to game server via ?server= param:
#   http://localhost:8080?server=ws://192.168.1.10:7001
```

### Production build (generates dist/)

```bash
cd battletris-web
trunk build --release
# Output: battletris-web/dist/
#   index.html   (wasm-bindgen patched entry point)
#   battletris_web_bg.wasm  (optimised WASM binary)
#   battletris_web.js       (glue JS)
#   battletris_web_bg.wasm.d.ts (TypeScript stubs)
```

The `dist/` directory is what the server's `--web-dir` argument points at.

### WASM-only compilation check (no trunk needed)

```bash
# Verifies the Rust code compiles to WASM — does not produce a usable bundle
cargo build -p battletris-web --target wasm32-unknown-unknown
```

---

## Full Workspace — Compile All Crates (native)

```bash
# From workspace root — builds all crates for the host target
cargo build --workspace
```

Expected output: `Finished dev profile` with 0 errors, 0 warnings.

---

## Desktop Client (battletris-client) — unchanged

```bash
cargo build -p battletris-client --release
# Output: target/release/battletris-client
# Requires SDL2 system library (libsdl2-dev on Linux, SDL2.framework on macOS)
```

---

## Build Artifacts

| Artifact | Location | Description |
|----------|----------|-------------|
| `battletris-server` | `target/release/battletris-server` | Game relay + HTTP server |
| `battletris-client` | `target/release/battletris-client` | SDL2 desktop client |
| WASM bundle | `battletris-web/dist/` | Browser WASM client assets |

---

## Troubleshooting

### `web-sys` feature not found
- Symptom: `feature 'ArrayBuffer' not found`
- Cause: `ArrayBuffer`/`Uint8Array` are from `js-sys`, not `web-sys`
- Status: **Fixed** — correct imports use `js_sys::ArrayBuffer`

### `--web-dir` directory doesn't exist
- Symptom: server exits immediately at startup
- Fix: run `trunk build` in `battletris-web/` before starting the server

### `trunk serve` CORS error connecting to game server
- Symptom: WS connection refused when using trunk dev server
- Fix: pass `?server=ws://<server-ip>:7001` in the browser URL
