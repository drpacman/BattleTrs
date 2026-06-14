# Dependencies — BattleTris

## Internal Dependencies (C++ modules)

```
game/ (BattleTris binary)
  +-- widget/         (UI rendering)
  +-- sockets/        (TCP networking)
  +-- stdlib/         (List, Block, BTStack, BTRingNode)
  +-- audio/          (sound — stub on non-Solaris)
  +-- signals/        (signal handling)
  +-- db/             (player/network records, config)

daemons/ (btserverd + btslaved)
  +-- sockets/        (TCP networking)
  +-- signals/        (signal handling)
  +-- db/             (BTDB, BTPlayer, BTNetworkEntry, BTGameStats)
  +-- stdlib/         (data structures)

btref/ (admin CLI)
  +-- db/             (BTDB, BTPlayer, BTNetworkEntry)
  +-- stdlib/
```

### game/ depends on widget/
- **Type**: Compile + Runtime
- **Reason**: All UI rendering (board, pieces, score, bazaar) uses Motif/X11 widgets wrapped by widget/

### game/ depends on sockets/
- **Type**: Compile + Runtime
- **Reason**: BTCommManager and BTNetManager use StreamSocket and PacketBuffer for game networking

### game/ depends on stdlib/
- **Type**: Compile + Runtime
- **Reason**: List<T>, Block<T>, BTStack<T>, BTRingNode used throughout game logic

### game/ depends on db/
- **Type**: Compile
- **Reason**: Uses BTNetworkEntry, BTPlayer, BTGameStats data structures for server communication

### daemons/ depends on db/
- **Type**: Compile + Runtime
- **Reason**: btserverd manages BTDB database; all player and network records reside here

### daemons/ depends on sockets/
- **Type**: Compile + Runtime
- **Reason**: All client/slave communication is via StreamSocket

## External Dependencies (Platform Libraries)

### X11 + Motif (libX11, libXm, libXt, libXext)
- **Version**: X11R6, Motif 1.2 / OpenMotif 2.3+
- **Usage**: All UI rendering, event loop, socket callback integration
- **Purpose**: Original windowing and UI framework
- **License**: MIT (X11), LGPL (OpenMotif)
- **Rust Port**: Must be replaced entirely with a cross-platform library

### POSIX Sockets (sys/socket.h)
- **Version**: POSIX.1
- **Usage**: StreamSocket wraps socket(), connect(), bind(), listen(), accept(), send(), recv()
- **Purpose**: TCP networking for all game and server communication
- **License**: Platform (OS-provided)
- **Rust Port**: Port to Rust std::net::TcpStream / TcpListener, or tokio::net

### Sun Audio (/dev/audio)
- **Version**: SunOS 4.x / Solaris audio ABI
- **Usage**: BTSoundManager / DevAudio open /dev/audio, write PCM audio data
- **Purpose**: In-game sound effects
- **License**: Platform (OS-provided)
- **Rust Port**: Stub (no sounds available) or replace with rodio/cpal

### POSIX Signals (signal.h)
- **Version**: POSIX.1
- **Usage**: SigHandler/SigReceiver for daemon signal management (SIGTERM, SIGCHLD, etc.)
- **Purpose**: Clean daemon shutdown and child process management
- **License**: Platform (OS-provided)
- **Rust Port**: Use ctrlc crate or signal-hook for cross-platform signals

### POSIX File I/O (unistd.h, fcntl.h)
- **Version**: POSIX.1
- **Usage**: BTDB uses open/read/write/close/lseek for flat-file hash database
- **Purpose**: Persistent player data storage
- **License**: Platform (OS-provided)
- **Rust Port**: Port to std::fs

## Platform-Specific Issues for Rust Port

### Solaris-isms that must go
- `/dev/audio` — stub completely
- `<sys/filio.h>` — replace with `<sys/ioctl.h>`
- `SIGPOLL` — replace with `SIGIO` (macOS/Linux) or omit (Windows)
- `bzero()`/`bcopy()` — replace with memset/memcpy (or Rust equivalents)
- `struct msghdr.msg_accrights` — not on Linux/macOS/Windows; use `msg_control` + `SCM_RIGHTS`

### macOS-specific
- XQuartz provides X11 at `/opt/X11`
- OpenMotif via Homebrew at `/opt/homebrew`
- BSD socket API (compatible)

### Windows-specific (new for Rust port)
- No X11/Motif — need cross-platform renderer
- WinSock2 (wrapped by Rust std::net/tokio)
- No POSIX signals — use Windows event objects or ctrlc crate
- Path separators, no fork() — server must use threads or async instead
