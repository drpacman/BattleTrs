# Integration Test Instructions — BattleTrisRs Browser Client

## Overview

The browser client feature adds a WebSocket relay path alongside the existing TCP relay. Integration testing validates:

1. **WASM ↔ Server**: Browser client connects, authenticates, pairs, and plays
2. **Desktop ↔ Server**: SDL2 client still works on the unchanged TCP path
3. **Cross-client**: WASM client and SDL2 desktop client can play each other

All integration tests are manual (no browser automation framework is in scope for this project).

---

## Setup

### Terminal 1 — Build the WASM bundle

```bash
cd battletris-web
trunk build
# Produces: battletris-web/dist/
```

### Terminal 2 — Start the server

```bash
# From workspace root
cargo run -p battletris-server -- serve \
    --port 7000 \
    --web-port 7001 \
    --web-dir battletris-web/dist
```

Expected server startup output:
```
[TCP] Listening on 0.0.0.0:7000
[HTTP+WS] Listening on 0.0.0.0:7001 (serving ./battletris-web/dist)
```

---

## Scenario 1: WASM Client vs WASM Client

**Tests**: browser WS path end-to-end

**Steps**:
1. Open browser tab 1: `http://localhost:7001`
2. At the `window.prompt`, enter name `Alice`
3. Canvas shows "CONNECTING..." then "WAITING FOR OPPONENT..."
4. Open browser tab 2 (same browser or different): `http://localhost:7001`
5. At prompt, enter name `Bob`
6. Both tabs: canvas transitions to the game board (GameStart received)
7. Play: use arrow keys to move pieces, Space to hard drop
8. Verify both boards show on each player's screen (board sync via BoardUpdate)
9. Let one player top out — verify GameOver screen shows on both tabs with ELO delta

**Pass criteria**:
- [ ] Both clients connect and pair successfully
- [ ] Boards sync in real time
- [ ] Weapons work (press 1–9 to fire, watch opponent board react)
- [ ] Bazaar opens (server-arbitrated at 20 combined lines)
- [ ] GameOver + ELO delta shown on both clients
- [ ] "ENTER to play again" reloads the page and reconnects

---

## Scenario 2: Desktop Client vs WASM Client (Cross-play)

**Tests**: TCP path and WS path interoperating through the shared relay

**Steps**:
1. Start server as above
2. Open browser: `http://localhost:7001` → enter name `Web`
3. Canvas shows "WAITING FOR OPPONENT..."
4. In another terminal: `cargo run -p battletris-client -- serve 127.0.0.1:7000` (or however the desktop client connects)  
   Alternatively: `./target/release/battletris-client` and choose network play → connect to `127.0.0.1:7000` → enter name `Desktop`
5. Both clients are paired and the game starts
6. Verify game plays correctly with one TCP client and one WS client

**Pass criteria**:
- [ ] TCP client and WS client pair together
- [ ] Boards sync between clients
- [ ] Weapons transmit correctly across protocol boundary (TcpConn ↔ WsConn relay)
- [ ] Game ends correctly for both

---

## Scenario 3: Desktop Client vs Desktop Client (Regression)

**Tests**: original TCP path unaffected by browser changes

**Steps**:
1. Start server: `cargo run -p battletris-server -- serve --web-dir battletris-web/dist`
2. Connect two desktop clients to port 7000
3. Play a game to completion

**Pass criteria**:
- [ ] TCP-only game works exactly as before
- [ ] No regressions introduced by the server refactor (GameConn abstraction)

---

## Scenario 4: Disconnect Handling

**Tests**: PeerDisconnected / GameVoid flow in WASM client

**Steps**:
1. Start a WASM vs WASM game
2. Close one browser tab mid-game
3. Verify the remaining tab shows "OPPONENT DISCONNECTED" overlay with 15s countdown
4. After 15 seconds: verify "GAME VOID" — GameVoid message received, screen resets to waiting

**Pass criteria**:
- [ ] Disconnect overlay appears within 1 second
- [ ] After 15s, client resets to WaitingForOpponent state

---

## Scenario 5: Name Taken

**Tests**: NameTaken message handling in WASM client

**Steps**:
1. Open browser tab 1: `http://localhost:7001` → enter name `Alice` → waiting screen
2. Open browser tab 2: `http://localhost:7001` → enter name `Alice`
3. Tab 2 should show "NAME ALREADY IN USE / RELOAD TO TRY AGAIN"

**Pass criteria**:
- [ ] Second connection with duplicate name shows error screen instead of crashing

---

## Dev Workflow (trunk serve)

When using `trunk serve` for hot-reload development, the WS URL must be overridden:

```
http://localhost:8080?server=ws://localhost:7001
```

The `?server=` parameter bypasses the origin-derived WS URL so the WASM app connects to the game server (port 7001) rather than trunk's dev server (port 8080).
