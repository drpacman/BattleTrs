# Technology Stack — BattleTris (C++ Source)

## Programming Languages

| Language | Version | Usage |
|----------|---------|-------|
| C++ | Pre-standard (1994) | All game logic, UI, networking, database |
| C | Via POSIX headers | System calls (sockets, signals, file I/O) |
| Shell | Bourne | autoconf configure script, Makefile rules |

## Frameworks & Libraries

| Library | Purpose | Status for Rust Port |
|---------|---------|----------------------|
| X11 (libX11) | Window system, event loop, drawing | Replace with cross-platform renderer |
| Motif (libXm) | UI widget toolkit | Replace with cross-platform renderer |
| Xt (libXt) | Xt Intrinsics — timer and I/O callbacks | Replace with Rust game loop / tokio |
| XExtensions (libXext) | X11 extensions | Replace |
| POSIX sockets | TCP networking | Port to Rust std::net or tokio |
| POSIX file I/O | Database storage | Port to std::fs |
| POSIX signals | Daemon signal handling | Replace with Rust signal handling |
| Sun audio (/dev/audio) | Sound playback | Stub or replace with rodio/cpal |

## Build Tools

| Tool | Purpose |
|------|---------|
| autoconf | Configure script generation (configure.in) |
| Sun make / GNU make | Build system with per-directory Makefiles |
| clang++ / g++ | C++ compilation (modern platforms) |
| ar | Static library archiving |

## Runtime Platform (Original)

| Component | Original | Port Target |
|-----------|----------|-------------|
| OS | Solaris/SPARC | macOS (arm64/x86_64) + Windows (x86_64) |
| Window system | X11 (Solaris) | macOS native / Windows native (via Rust lib) |
| Network | POSIX TCP | TCP (macOS + Windows — WinSock2 / POSIX) |
| Audio | /dev/audio | Cross-platform audio library (TBD) |
| Compiler | SunCC | rustc + LLVM |

## Data Formats

| Format | Usage |
|--------|-------|
| Binary framed TCP | All network communication (BTToken protocol) |
| Hash flat-file | Player database (BTDB) |
| PPM/XPM/XBM | Art assets |
| Text key=value | btweapons.db (weapon definitions) |
| X resources (.ad) | X11 UI configuration |

## Rust Port — Target Technology Stack

| Component | Recommended Rust Library | Notes |
|-----------|-------------------------|-------|
| Rendering / UI | SDL2 (`sdl2` crate) or `macroquad` | Cross-platform; SDL2 has wide platform support |
| Networking | `tokio` or `std::net` | TCP client/server; tokio for async if needed |
| Audio | `rodio` or `cpal` | Cross-platform audio |
| Serialization | `serde` + `bincode` | Replace custom BTScore/BTBoard serialization |
| Build | Cargo | Standard Rust build system |
| Game loop | SDL2 event loop or `winit` | Replaces Xt event loop |
