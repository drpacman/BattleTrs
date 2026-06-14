# Build Notes — Unit 1 (core-engine)

## macOS Build

```bash
# Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and run the game
cd /path/to/BattleTrisRs
cargo run -p battletris-client
```

SDL2 is compiled from source via the `bundled` feature — no separate SDL2 installation needed on macOS.

## Windows Cross-Compile (from macOS)

Install the Windows target and cross-compiler:

```bash
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
```

Build:

```bash
cargo build -p battletris-client --target x86_64-pc-windows-gnu --release
```

The resulting `battletris-client.exe` in `target/x86_64-pc-windows-gnu/release/` is self-contained (SDL2 statically linked via `bundled` feature). No runtime DLLs required.

**Note**: The `bundled` SDL2 feature compiles SDL2 from C source using the C cross-compiler. Build time is ~2–5 minutes on first compile. Requires `mingw-w64` (`brew install mingw-w64` on macOS).

## Running Tests (engine only — no SDL2 required)

```bash
cargo test -p battletris-engine
```

## Development Tips

- The `battletris-engine` crate is pure Rust logic with no SDL2 dependency — tests run in any environment.
- The `battletris-client` crate requires SDL2 (compiled from source via `bundled`).
- The `battletris-server` crate (Unit 3) has no SDL2 dependency.
- To use a system SDL2 instead of bundled (faster rebuilds): change `sdl2 = { version = "0.36", features = ["ttf", "bundled"] }` to `sdl2 = { version = "0.36", features = ["ttf"] }` in `battletris-client/Cargo.toml`.

## Font Note

The client attempts to load a system TTF font at runtime from common paths:
- macOS: `/System/Library/Fonts/Menlo.ttc` (Menlo monospace) or Helvetica fallback
- Windows: `C:\Windows\Fonts\cour.ttf` (Courier New) or Arial fallback
- Linux: DejaVu Sans Mono or Liberation Mono

If no font is found, game board and pieces render correctly but text labels (score, stats) are invisible. The game remains fully playable.
