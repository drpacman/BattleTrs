use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::engine::board::{BoardSnapshot, Cell};
use crate::engine::game_state::{GameEvent, GameMode, GamePhase, GameState, PlayerInput, PlayingView};
use crate::engine::weapons::{WeaponKind, check_mirror, MirrorResult};
use crate::protocol::GameMessage;

/// Shared state for a networked (or vs-computer) game session.
///
/// Encapsulates `GameState` plus all peer-tracking fields so both the SDL2
/// game_loop thread and the WASM rAF tick can share the same logic for:
/// - processing incoming peer messages
/// - converting engine events into outgoing messages
/// - board-visibility fog-of-war
pub struct NetworkSession {
    pub state: GameState,
    pub peer_board: Option<BoardSnapshot>,
    pub peer_disconnected: bool,
    /// Set after the server-enriched GameOver reply arrives; carries the winner
    /// name and ELO delta for the results screen.
    pub network_result: Option<(bool, String, i32)>,
    /// Guards against sending the local-defeat notification more than once.
    pub did_send_game_over: bool,
    pub player_name: Option<String>,
    pub opponent_name: Option<String>,
    /// True after we fire Susan — the next ArsenalSwapped we receive is the
    /// peer's reply carrying their arsenal (not a fresh Susan aimed at us).
    pending_susan_reply: bool,
}

impl NetworkSession {
    pub fn new(seed: u64, player_name: Option<String>) -> Self {
        Self {
            state: GameState::new(seed),
            peer_board: None,
            peer_disconnected: false,
            network_result: None,
            did_send_game_over: false,
            player_name,
            opponent_name: None,
            pending_susan_reply: false,
        }
    }

    /// Build a `PlayingView` with fog-of-war and peer metadata applied.
    pub fn playing_view(&self) -> PlayingView {
        let mut view = self.state.to_playing_view();
        view.opponent_board = apply_board_visibility(&self.peer_board, view.opponent_board_accuracy);
        view.peer_disconnected = self.peer_disconnected;
        view.opponent_name = self.opponent_name.clone();
        view.player_name = self.player_name.clone();
        view
    }

    /// Reset per-round peer state when a new game begins (vs-computer rematch).
    pub fn reset_peer_state(&mut self) {
        self.peer_board = None;
        self.peer_disconnected = false;
        self.network_result = None;
        self.did_send_game_over = false;
        self.pending_susan_reply = false;
    }

    /// Process one incoming peer `GameMessage`, mutating session state.
    ///
    /// Returns any replies that must be forwarded back to the peer (e.g.
    /// `WeaponReflected`).  Platform-specific arms (`Welcome`, `NameTaken`,
    /// vs-computer `LinesCleared`/`BazaarEnd`) are NOT handled here — the
    /// caller matches those before or after this call.
    pub fn process_message(&mut self, msg: GameMessage) -> Vec<GameMessage> {
        let mut replies: Vec<GameMessage> = Vec::new();

        match msg {
            GameMessage::GameStart { opponent_name } => {
                self.peer_disconnected = false;
                if self.opponent_name.is_none() && !opponent_name.is_empty() {
                    self.opponent_name = Some(opponent_name);
                }
            }

            GameMessage::BazaarOpen => {
                self.state.open_bazaar_now();
                // VsComputer: the opponent finishes shopping first, then sends BazaarOpen —
                // mark opponent_done immediately so the local player can exit at will.
                // VsNetwork: server sends BazaarOpen to both players simultaneously —
                // opponent_done stays false until BazaarEnd arrives from the peer.
                if self.state.mode != GameMode::VsNetwork {
                    self.state.bazaar_done();
                }
            }

            GameMessage::BazaarEnd => {
                self.state.bazaar_done();
            }

            GameMessage::PeerDisconnected => {
                self.peer_disconnected = true;
            }

            GameMessage::PeerReconnected => {
                self.peer_disconnected = false;
            }

            GameMessage::GameVoid => {
                self.state.phase = GamePhase::Title;
            }

            GameMessage::BoardUpdate { snapshot } => {
                self.peer_board = Some(snapshot);
            }

            GameMessage::ScoreUpdate { score, lines, funds } => {
                self.state.score.update_opponent(score, lines, funds);
            }

            GameMessage::WeaponLaunched { kind } => {
                if kind == WeaponKind::Swap {
                    // Swap exchanges boards — needs both players' state, so apply_incoming_weapon
                    // can't handle it. We use our last peer_board snapshot instead.
                    let mirror = check_mirror(kind, &self.state.weapon_state);
                    if mirror == MirrorResult::PassThrough {
                        let snap = self.peer_board.clone();
                        if let Some(snap) = snap {
                            self.state.apply_swap_board(&snap);
                            replies.push(GameMessage::BoardUpdate {
                                snapshot: self.state.board.snapshot(),
                            });
                        }
                    }
                    // Mirror::Nullified → do nothing (Swap is in Mirror's nullified list,
                    // so it can never be reflected back).
                } else {
                    let mut rng = StdRng::from_entropy();
                    let events = self.state.apply_incoming_weapon(kind, &mut rng);
                    for ev in &events {
                        if let GameEvent::WeaponFired { kind: k, reflect: true } = ev {
                            replies.push(GameMessage::WeaponReflected { kind: *k });
                        }
                    }
                }
            }

            GameMessage::WeaponReflected { kind } => {
                let mut rng = StdRng::from_entropy();
                self.state.apply_incoming_weapon(kind, &mut rng);
            }

            GameMessage::LinesCleared { count, .. } => {
                // Keep the combined-line counter in sync so the bazaar trigger
                // fires based on both players' lines (matching the original game).
                self.state.score.add_op_lines(count);

                // Lawyers' Delite: each line the opponent (attacker) clears
                // adds a junk row to this player's (victim's) board.
                if self.state.weapon_state.is_active(WeaponKind::Lawyers) {
                    let mut rng = StdRng::from_entropy();
                    for _ in 0..count {
                        if self.state.board.rise_up(&mut rng) {
                            self.state.phase = GamePhase::GameOver { won: false };
                            break;
                        }
                    }
                }
            }

            GameMessage::ArsenalSwapped { arsenal } => {
                if self.pending_susan_reply {
                    // Reply to our own Susan — take the peer's arsenal.
                    self.state.arsenal = arsenal;
                    self.pending_susan_reply = false;
                } else {
                    // Incoming Susan from peer — swap and reply.
                    // Mirror::Nullified: echo the payload back so the firer's pending
                    //   reply resolves to their own arsenal (net zero change).
                    let my_old = if check_mirror(WeaponKind::Susan, &self.state.weapon_state)
                        == MirrorResult::PassThrough
                    {
                        std::mem::replace(&mut self.state.arsenal, arsenal)
                    } else {
                        arsenal // mirrored: return unchanged, own arsenal untouched
                    };
                    replies.push(GameMessage::ArsenalSwapped { arsenal: my_old });
                }
            }

            GameMessage::FundsReceived { amount } => {
                self.state.score.funds += amount;
            }

            GameMessage::GameOver { winner_name, elo_delta_winner, elo_delta_loser, .. } => {
                if self.state.mode == GameMode::VsComputer {
                    // Ernie sends GameOver to signal its own defeat; if we are still
                    // playing, we won.
                    if matches!(self.state.phase, GamePhase::Playing | GamePhase::InBazaar) {
                        self.state.phase = GamePhase::GameOver { won: true };
                    }
                } else {
                    let i_won = self.player_name.as_deref()
                        .map(|n| n == winner_name)
                        .unwrap_or(false);
                    let my_delta = if i_won { elo_delta_winner } else { elo_delta_loser };
                    self.network_result = Some((i_won, winner_name, my_delta));
                    self.state.phase = GamePhase::GameOver { won: i_won };
                }
            }

            GameMessage::PlayerQuit => {
                self.state.phase = GamePhase::Title;
            }

            _ => {}
        }

        replies
    }

    /// Tick the engine and build all outgoing messages for this frame.
    ///
    /// Returns `(events, outgoing)`:
    /// - `events`: raw engine events for any caller-specific handling
    ///   (e.g. vs-computer bazaar sync, WASM quit-confirm check).
    /// - `outgoing`: messages ready to send to the peer / server:
    ///   `WeaponLaunched`, `LinesCleared`, `BoardUpdate`, `ScoreUpdate`,
    ///   and `GameOver` (local defeat notification) when applicable.
    pub fn tick(&mut self, input: Option<PlayerInput>, elapsed_ms: u32) -> (Vec<GameEvent>, Vec<GameMessage>) {
        let was_playing = matches!(self.state.phase, GamePhase::Playing | GamePhase::InBazaar);

        let events = self.state.tick(input, elapsed_ms);

        let mut outgoing = events_to_outgoing(&events, &self.state);

        // Susan weapon: replace WeaponLaunched with an ArsenalSwapped handshake.
        // WeaponLaunched carries no payload; the round-trip via ArsenalSwapped is the
        // only way to exchange arsenals without a continuously-synced peer_arsenal field.
        outgoing.retain(|m| !matches!(m, GameMessage::WeaponLaunched { kind: WeaponKind::Susan }));
        for ev in &events {
            if let GameEvent::WeaponFired { kind: WeaponKind::Susan, reflect: false } = ev {
                self.pending_susan_reply = true;
                outgoing.push(GameMessage::ArsenalSwapped {
                    arsenal: self.state.arsenal.clone(),
                });
            }
        }

        // Swap weapon: firer's board also becomes what the peer had.
        // Both sides independently swap with their own peer_board snapshot so the exchange
        // happens without a round-trip.  A BoardUpdate is appended so the peer immediately
        // learns the firer's new (formerly-peer) board state.
        for ev in &events {
            if let GameEvent::WeaponFired { kind: WeaponKind::Swap, reflect: false } = ev {
                let snap = self.peer_board.clone();
                if let Some(snap) = snap {
                    self.state.apply_swap_board(&snap);
                }
                outgoing.push(GameMessage::BoardUpdate { snapshot: self.state.board.snapshot() });
            }
        }

        // Computed after Swap (which can top-out and set GameOver).
        let now_loss = matches!(self.state.phase, GamePhase::GameOver { won: false });

        // Notify peer when we press Done in the bazaar.
        // Using the event (not a before/after state snapshot) because the bazaar may
        // close immediately when opponent_done was already true — making bazaar None and
        // causing a snapshot of player_done to read false from the missing struct.
        if self.state.mode != GameMode::SinglePlayer
            && events.iter().any(|e| matches!(e, GameEvent::BazaarPlayerDone))
        {
            outgoing.push(GameMessage::BazaarEnd);
        }

        if was_playing {
            outgoing.push(GameMessage::ScoreUpdate {
                score: self.state.score.score,
                lines: self.state.score.lines,
                funds: self.state.score.funds,
            });
        }

        // Notify peer/server of local defeat so ELO can be computed.
        if was_playing && now_loss && !self.did_send_game_over {
            self.did_send_game_over = true;
            outgoing.push(GameMessage::GameOver {
                winner_id: 0,
                final_score_p1: self.state.score.score,
                final_score_p2: 0,
                winner_name: String::new(),
                elo_delta_winner: 0,
                elo_delta_loser: 0,
            });
        }

        // Spy weapons (Ace, Ames, Condor): the firer gains board visibility
        // immediately; the message must NOT be forwarded to the peer.
        let outgoing = {
            let mut filtered = Vec::with_capacity(outgoing.len());
            for msg in outgoing {
                if let GameMessage::WeaponLaunched { kind } = msg {
                    if matches!(kind, WeaponKind::Ace | WeaponKind::Ames | WeaponKind::Condor) {
                        let mut rng = StdRng::from_entropy();
                        self.state.apply_incoming_weapon(kind, &mut rng);
                        continue;
                    }
                }
                filtered.push(msg);
            }
            filtered
        };

        (events, outgoing)
    }

    /// Single entry-point for one game frame on both native and WASM.
    ///
    /// Processes `peer_messages` through `process_message`, ticks the engine
    /// with `input` and `elapsed_ms`, and returns all messages that should be
    /// forwarded to the peer plus the raw engine events.
    ///
    /// Platform-specific messages (connection handshake, navigation signals)
    /// should be consumed by the caller before passing the remainder here.
    pub fn advance_frame(
        &mut self,
        peer_messages: impl IntoIterator<Item = GameMessage>,
        input: Option<PlayerInput>,
        elapsed_ms: u32,
    ) -> (Vec<GameMessage>, Vec<GameEvent>) {
        let mut to_peer = Vec::new();
        for msg in peer_messages {
            let replies = self.process_message(msg);
            to_peer.extend(replies);
        }
        let (events, outgoing) = self.tick(input, elapsed_ms);
        to_peer.extend(outgoing);
        (to_peer, events)
    }
}

/// Apply fog-of-war to an opponent board based on weapon accuracy [0.0, 1.0].
/// Returns `None` when accuracy is 0 (full blind), `Some(snapshot)` otherwise
/// with cells randomly blanked proportionally to `(1 - accuracy)`.
pub fn apply_board_visibility(board: &Option<BoardSnapshot>, accuracy: f32) -> Option<BoardSnapshot> {
    if accuracy <= 0.0 {
        return None;
    }
    let Some(snapshot) = board else { return None };
    if accuracy >= 1.0 {
        return Some(snapshot.clone());
    }
    let mut rng = rand::thread_rng();
    let mut cells = snapshot.cells;
    for row in cells.iter_mut() {
        for cell in row.iter_mut() {
            if !cell.is_empty() && rng.gen::<f32>() > accuracy {
                *cell = Cell::Empty;
            }
        }
    }
    Some(BoardSnapshot { cells })
}

fn events_to_outgoing(events: &[GameEvent], state: &GameState) -> Vec<GameMessage> {
    let mut out = Vec::new();
    for event in events {
        match event {
            GameEvent::WeaponFired { kind, reflect: false } => {
                out.push(GameMessage::WeaponLaunched { kind: *kind });
            }
            GameEvent::LinesCleared(n) => {
                out.push(GameMessage::LinesCleared { count: *n as u32, funds_earned: 0 });
            }
            GameEvent::PieceLocked => {
                out.push(GameMessage::BoardUpdate { snapshot: state.board.snapshot() });
            }
            _ => {}
        }
    }
    out
}
