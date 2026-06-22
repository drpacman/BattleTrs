# BattleTris — Rust Port

A modern Rust port of **BattleTris**, the two-player networked Tetris-with-weapons game originally written at Brown University in spring 1994 by Bryan Cantrill, Charlie Hoecker and Mike Shapiro as a CS32 final project.

The original source and its history live in the [`BattleTris`](https://github.com/bcantrill/BattleTris) repo. A fuller account of the original game's creation can be found [here](https://bcantrill.dtrace.org/2026/05/25/a-portentous-reunion/).

---

## What is BattleTris?

BattleTris is Tetris with an economic warfare layer. You play Tetris normally — but every line you clear earns you money (based on the die-pip values locked into those lines), and every 20 combined lines both players are whisked to a **weapons bazaar** where they spend that money on attacks to make the other player's game harder.

Weapons include flipping the opponent's board upside-down, swapping boards entirely, depriving them of long pieces, spying on their board, adding junk rows, and many more. Weapons last for a duration measured in lines cleared.

The first player whose board tops out loses.

---

## Requirements

- **Rust** (stable, 2021 edition) — [rustup.rs](https://rustup.rs)
- **SDL2** — used for rendering and input

### macOS (Homebrew)

```sh
brew install sdl2
```

The `.cargo/config.toml` in this repo already points the linker at `/opt/homebrew/lib` for `aarch64-apple-darwin`.

### Windows (ARM64, vcpkg)

```powershell
vcpkg install sdl2:arm64-windows
```

The `.cargo/config.toml` already points the linker at `C:/vcpkg/installed/arm64-windows/lib` for `aarch64-pc-windows-msvc`. Copy `SDL2.dll` from `C:/vcpkg/installed/arm64-windows/bin/` next to the built executable when distributing.

---

## Building

```sh
cargo build --release
```

The client binary is `target/release/battletris-client`.

---

## Running

### Solo practice

```sh
cargo run --release --bin battletris-client
```

Press **S** on the title screen to launch solo practice mode. You start with $10,000 in funds so you can explore all the weapons freely. You can launch the Bazaar at any time by pressing **B**.

### Vs computer (Ernie)

Press **Enter** on the title screen to play against Ernie, the built-in AI opponent.

### Network play

Start the server on a host both players can reach:

```sh
cargo run --release --bin battletris-server
```

Each player then runs the client and presses **N** to connect:

```sh
cargo run --release --bin battletris-client
```

You will be prompted for the server address and your player name. ELO rankings are tracked in `players.json` on the server host.

---

## Controls

| Key | Action |
|---|---|
| Left / Right arrow | Move piece |
| Up arrow | Rotate clockwise |
| Z | Rotate counter-clockwise |
| Down arrow | Soft drop |
| Space | Hard drop |
| P | Pause |
| Esc | Quit to title |
| 1 – 9, 0 | Launch weapon from arsenal slot 1–10 |
| Up/Down in bazaar | Navigate weapon list |
| Enter (numpad) | Buy selected weapon |
| B | Open bazaar immediately (solo practice only) |

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

- Uses **SDL2** for cross-platform rendering (macOS, Windows, Linux)
- Implements a self-contained **TCP server** for network play — no separate `btserverd` daemon required
- Adds **solo practice mode** (S key) for exploring weapons without an opponent
- Preserves all original gameplay mechanics, weapon set, scoring and bazaar timing faithfully

---

## Credits

Original BattleTris (1994): Bryan Cantrill, Charlie Hoecker, Mike Shapiro — Brown University CS32.

---

## Licence

The original BattleTris source (in [`BattleTris/`](https://github.com/bcantrill/BattleTris)) is MIT licensed. Its copyright notice is reproduced in [`LICENSE-ORIGINAL`](LICENSE-ORIGINAL) in accordance with the licence terms:

> MIT License  
> Copyright (c) 1993-1997 Bryan Cantrill, Charlie Hoecker and Mike Shapiro
