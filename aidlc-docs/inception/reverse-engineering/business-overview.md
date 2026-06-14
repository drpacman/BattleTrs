# Business Overview — BattleTris

## Business Context Diagram

```
+-----------------------------------------------------------------------+
|                         BATTLETRIS SYSTEM                             |
|                                                                       |
|  +------------------+     TCP/IP      +---------------------------+   |
|  |  Player 1 Client |<--------------->|  BattleTris Server        |   |
|  |  (Mac/Linux)     |                 |  (btserverd + btslaved)   |   |
|  +------------------+                 +---------------------------+   |
|                                               |                       |
|  +------------------+     TCP/IP             |  Player DB (BTDB)     |
|  |  Player 2 Client |<---------------------->|  Rankings, Stats      |
|  |  (Mac/Linux/Win) |                         +---------------------------+   |
|  +------------------+                                                |
|                                                                       |
|  OR: Single-player vs. AI (no network required: -X flag)             |
+-----------------------------------------------------------------------+
```

## Business Description

- **Business Description**: BattleTris is a two-player competitive networked Tetris game with a weapons system. Players play Tetris in parallel; scoring lines earns in-game currency ("funds") which is spent at a weapons bazaar. Purchased weapons are deployed against the opponent to disrupt their game. The first player whose board fills up loses. Player records (wins, losses, funds, ELO rank) are persisted in a server-side database.

- **Business Transactions**:
  1. **Player Connection** — Player launches client, connects to btserverd, registers presence on the network, appears in lobby
  2. **Matchmaking / Challenge** — Player views roster of online players, issues a challenge; opponent accepts or declines
  3. **Game Start** — Both players confirmed ready; synchronized game start
  4. **Piece Play** — Player moves/rotates falling pieces; die pieces and happy-face pieces generate funds when lines are cleared
  5. **Line Clearing & Scoring** — Completed lines are removed; score and funds updated and synced to opponent
  6. **Weapons Bazaar** — Every 20 combined lines, both players enter the bazaar to purchase weapons with accumulated funds
  7. **Weapon Launch** — Player presses number key 1-10 to activate a weapon from their arsenal against the opponent
  8. **Weapon Effect** — Active weapons alter the opponent's board or game state for a duration measured in lines
  9. **Game Over** — One player's board fills to the top; loser is identified; stats recorded
  10. **Player-vs-Computer** — Single-player mode against Ernie (AI) using -X flag; no server needed; no ranking

- **Business Dictionary**:
  - **Funds**: In-game currency earned by clearing lines; die pip values are summed for the line
  - **Arsenal**: The player's active set of up to 10 weapons available for launch
  - **Bazaar**: Weapons purchasing screen, triggered every 20 combined lines
  - **Die Piece**: A 1×1 special piece with 1-6 pip value; adds pip value to funds when its line is cleared
  - **Happy Face / Smiley**: A 1×1 special piece worth 150 funds if cleared on the turn it appears; turns into a frown if missed
  - **Weapon Duration**: Weapons last for N lines (the affected player's lines), not wall-clock time
  - **ELO Rank**: Player ranking computed using the Elo system, starting at 1200
  - **Ernie**: The AI opponent's name
  - **btserverd**: Master server daemon — manages player database and spawns slave daemons
  - **btslaved**: Per-connection slave daemon — routes game traffic between paired clients
  - **btref**: Command-line admin tool for manipulating the player database

## Component Level Business Descriptions

### game/ — Game Client
- **Purpose**: The main interactive game application players run on their machines
- **Responsibilities**: Rendering the Tetris board and UI (via X11/Motif), handling keyboard input, managing all game logic (piece movement, line clearing, weapon effects), networking with the opponent via btslaved, AI opponent management

### daemons/ — Server Infrastructure
- **Purpose**: Manages player presence and routes game traffic between opponents
- **Responsibilities**: btserverd accepts connections, maintains player database access, spawns btslaved processes per client pair; btslaved acts as the relay between two matched game clients

### db/ — Player Database
- **Purpose**: Persistent storage of player identities, statistics, and rankings
- **Responsibilities**: Hash-based flat-file database (BTDB) with read/write locking; stores BTPlayer records (name, host, ELO score, win/loss stats); BTNetworkEntry tracks online presence

### widget/ — UI Abstraction Layer
- **Purpose**: Wraps X11/Motif widgets in C++ objects for use by the game client
- **Responsibilities**: Provides BTDisplay, drawing areas, forms, labels, push-buttons, scrolled lists/text, slider, palette management

### sockets/ — Network Abstraction
- **Purpose**: Cross-platform TCP socket library integrated with the Xt event loop
- **Responsibilities**: StreamSocket (TCP), PacketBuffer (framed message buffering), Address abstraction, Xt-integrated callbacks (XtSocketCB)

### stdlib/ — Custom Containers
- **Purpose**: Custom template data structures (pre-STL era)
- **Responsibilities**: List<T>, AbsList<T>, Block<T>, BTStack<T>, BTRingNode (message bus)

### audio/ — Sound System
- **Purpose**: Play in-game sound effects
- **Responsibilities**: Interface to /dev/audio (Solaris-specific; stub on macOS/Linux/Windows)

### signals/ — Signal Handling
- **Purpose**: POSIX signal management for clean daemon operation
- **Responsibilities**: SigHandler, SigReceiver for async signal delivery

### btref/ — Admin Utility
- **Purpose**: Command-line tool for database administration
- **Responsibilities**: View/edit player records, manage the player database

### share/ — Game Assets
- **Purpose**: Static configuration and data files
- **Responsibilities**: btweapons.db (weapon definitions: name, description, price, duration), BattleTris.ad (X11 resource file), art assets (PPM/XPM/XBM images)
