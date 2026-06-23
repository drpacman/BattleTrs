use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::engine::board::{BoardSnapshot, Cell};
use crate::engine::game_state::{GameEvent, GamePhase, GameState, PlayerInput};
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
        }
    }

    /// Reset per-round peer state when a new game begins (vs-computer rematch).
    pub fn reset_peer_state(&mut self) {
        self.peer_board = None;
        self.peer_disconnected = false;
        self.network_result = None;
        self.did_send_game_over = false;
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
                self.state.ernie_bazaar_done();
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
                let mut rng = StdRng::from_entropy();
                let events = self.state.apply_incoming_weapon(kind, &mut rng);
                for ev in &events {
                    if let GameEvent::WeaponFired { kind: k, reflect: true } = ev {
                        replies.push(GameMessage::WeaponReflected { kind: *k });
                    }
                }
            }

            GameMessage::WeaponReflected { kind } => {
                let mut rng = StdRng::from_entropy();
                self.state.apply_incoming_weapon(kind, &mut rng);
            }

            GameMessage::FundsReceived { amount } => {
                self.state.score.funds += amount;
            }

            GameMessage::GameOver { winner_name, elo_delta_winner, elo_delta_loser, .. } => {
                let i_won = self.player_name.as_deref()
                    .map(|n| n == winner_name)
                    .unwrap_or(false);
                let my_delta = if i_won { elo_delta_winner } else { elo_delta_loser };
                self.network_result = Some((i_won, winner_name, my_delta));
                self.state.phase = GamePhase::GameOver { won: i_won };
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

        let now_loss = matches!(self.state.phase, GamePhase::GameOver { won: false });

        let mut outgoing = events_to_outgoing(&events, &self.state);

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

        (events, outgoing)
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
