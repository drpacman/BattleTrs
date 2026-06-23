# BattleTris — Rust Port

A modern Rust port of **BattleTris**, the two-player networked Tetris-with-weapons game originally written at Brown University in spring 1994 by Bryan Cantrill, Charlie Hoecker and Mike Shapiro as a CS32 final project.

The original source and its history live in the [`BattleTris`](https://github.com/bcantrill/BattleTris) repo. A fuller account of the original game's creation can be found [here](https://bcantrill.dtrace.org/2026/05/25/a-portentous-reunion/).

---

## What is BattleTris?

BattleTris is Tetris with an economic warfare layer. You play Tetris normally — but every line you clear earns you money (based on the die-pip values locked into those lines), and every 20 combined lines both players are whisked to a **weapons bazaar** where they spend that money on attacks to make the other player's game harder.

Weapons include flipping the opponent's board upside-down, swapping boards entirely, depriving them of long pieces, spying on their board, adding junk rows, and many more. Weapons last for a duration measured in lines cleared.

The first player whose board tops out loses.

---

## Clients

There are two ways to play:

| Client | Platform | Requires |
|---|---|---|
| **Native** (`battletris-client`) | macOS, Windows, Linux | SDL2 |
| **Browser** (`battletris-web`) | Any modern browser | A running server |

Both clients can connect to the same server and play against each other.

---

## Requirements

### Native client

- **Rust** (stable, 2021 edition) — [rustup.rs](https://rustup.rs)
- **SDL2**

#### macOS (Homebrew)

```sh
brew install sdl2
```

The `.cargo/config.toml` already points the linker at `/opt/homebrew/lib` for `aarch64-apple-darwin`.

#### Windows (ARM64, vcpkg)

```powershell
vcpkg install sdl2:arm64-windows
```

The `.cargo/config.toml` already points at `C:/vcpkg/installed/arm64-windows/lib` for `aarch64-pc-windows-msvc`. Copy `SDL2.dll` from `C:/vcpkg/installed/arm64-windows/bin/` next to the built executable when distributing.

### Browser client and server

- **Rust** (stable, 2021 edition)
- **[trunk](https://trunkrs.dev/)** — WASM bundler

```sh
cargo install trunk
rustup target add wasm32-unknown-unknown
```

---

## Building

### Native client

```sh
cargo build --release -p battletris-client
```

The binary is `target/release/battletris-client`.

### Browser client and server

```sh
cd battletris-web && trunk build && cd ..
cargo build --release -p battletris-server
```

Or use the provided script which builds both and starts the server in one step:

```sh
./run_server.sh
```

---

## Running

### Solo practice (native)

```sh
cargo run --release --bin battletris-client
```

Press **S** on the title screen to launch solo practice mode. You start with $10,000 in funds so you can explore all the weapons freely. Press **B** at any time to open the bazaar.

### Vs computer — Ernie (native)

Press **Enter** on the title screen to play against Ernie, the built-in AI opponent.

### Network play

The server handles both native (TCP) and browser (WebSocket) clients simultaneously — a native client and a browser client can play against each other.

#### Start the server

The quickest path for the browser client is the helper script, which rebuilds the WASM bundle and serves it alongside the WebSocket relay:

```sh
./run_server.sh
```

This is equivalent to:

```sh
cd battletris-web && trunk build && cd ..
cargo run --release -p battletris-server -- serve \
    --web-dir battletris-web/dist
```

Default ports:

| Protocol | Port | Used by |
|---|---|---|
| HTTP + WebSocket | 80 | Browser clients |
| TCP | 7001 | Native clients |

Both can be overridden with `--web-port` and `--port`.

#### Browser client

Open `http://<server-address>/` in a browser. Enter your name and press **Enter** to connect. The server address is derived automatically from the page URL — no configuration needed.

#### Native client

Press **N** on the title screen, then enter the server address (e.g. `192.168.1.10:7001`) and your name.

#### ELO rankings

Player results are stored in `players.json` on the server host. To view the leaderboard:

```sh
cargo run --release -p battletris-server -- players
```

---

## Controls

### In game

| Key | Action |
|---|---|
| Left / Right arrow | Move piece |
| Up arrow | Rotate clockwise |
| Z | Rotate counter-clockwise |
| Down arrow | Soft drop |
| Space | Hard drop |
| P | Pause |
| 1 – 9, 0 | Launch weapon from arsenal slot 1–10 |
| Esc | Quit (asks for confirmation in network play) |
| B | Open bazaar immediately (solo practice only) |

### In bazaar

| Key | Action |
|---|---|
| Up / Down arrow | Navigate weapon list |
| Enter | Buy selected weapon |
| Esc | Return to game |

---

## Gameplay

### Pieces and funds

In addition to the standard seven Tetris pieces there are special pieces:

- **Die** — a single cell showing 1–6 pips. When a line is cleared containing die cells, funds increase by the pip value (doubled/tripled/quadrupled for multi-line clears).
- **Happy face** — a single smiley cell. Clearing a line that contains it earns a $150 bonus; if it locks without being cleared it turns into a frown.

### Bazaar

Every 20 combined lines cleared (by both players together) both players enter the bazaar simultaneously. Browse the weapon list with Up/Down and buy with Enter. When done, press Escape to return to the game. In solo practice mode you can also press **B** at any time to open the bazaar.

### Weapons

Weapons are launched in-game by pressing the number key matching their position in your arsenal (shown on screen). Durations are measured in lines cleared by the player affected.

A selection of weapons:

| Weapon | Effect | Duration |
|---|---|---|
| Upbyside-down | Flips opponent's board upside-down | 10 lines |
| Speedy Gonzales | Dramatically speeds up opponent's drop rate | 10 lines |
| The Feared Weird | Opponent only receives bizarre hard-to-place pieces | 3 lines |
| Four-by-Four | Opponent's Box pieces become hollow 4×4 squares | 10 lines |
| Lawyer's Delite | Each line you clear adds a junk row to opponent | 5 lines |
| Rise Up | Adds one junk row to opponent's board | instant |
| Swap Meet | Exchanges your board with opponent's | instant |
| Flip Out | Mirrors opponent's board horizontally | instant |
| Keating Five | Steals all of opponent's funds | instant |
| Mirror-mirror | Reflects incoming weapons back at attacker | 10 lines |

There are 34 weapons in total.

---

## Differences from the original

The 1994 original ran on Solaris/SPARC using X11 and Motif. This port:

- Uses **SDL2** for cross-platform native rendering (macOS, Windows, Linux)
- Adds a **browser client** compiled to WebAssembly — no installation required for players
- Implements a self-contained **relay server** for network play — no separate `btserverd` daemon required; the same server binary handles both TCP and WebSocket connections
- Adds **solo practice mode** (S key) for exploring weapons without an opponent
- Adds **ELO rankings** tracked in a server-side `players.json`
- Preserves all original gameplay mechanics, weapon set, scoring and bazaar timing faithfully

---

## Credits

Original BattleTris (1994): Bryan Cantrill, Charlie Hoecker, Mike Shapiro — Brown University CS32.

---

## Licence

The original BattleTris source ([`BattleTris`](https://github.com/bcantrill/BattleTris)) is MIT licensed. Its copyright notice is reproduced in [`LICENSE-ORIGINAL`](LICENSE-ORIGINAL) in accordance with the licence terms:

> MIT License  
> Copyright (c) 1993-1997 Bryan Cantrill, Charlie Hoecker and Mike Shapiro
