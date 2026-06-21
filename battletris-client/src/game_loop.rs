use std::sync::mpsc::{Receiver, SyncSender};
use std::time::{Duration, Instant};

use rand::Rng;
use battletris_engine::engine::board::{BoardSnapshot, Cell};
use battletris_engine::engine::game_state::{GameMode, GamePhase, GameState, PlayerInput, PlayingView};
use battletris_engine::engine::score::ScoreView;
use battletris_engine::engine::weapons::ActiveWeaponView;
use battletris_engine::protocol::GameMessage;

pub enum RenderEvent {
    Title,
    Playing(PlayingView),
    GameOver {
        won: bool,
        score: u32,
        lines: u32,
        winner_name: Option<String>,
        elo_delta: Option<i32>,
    },
}

/// Channel pair connecting game_loop to either the Ernie AI task or the network relay.
pub struct PeerChannels {
    pub from_peer: Receiver<GameMessage>,
    pub to_peer: SyncSender<GameMessage>,
    pub is_network: bool,
}


/// Game-tick thread entry point.
///
/// `player_name` — required for network mode so the game_loop can match its own
///   name against the winner_name in the server-enriched GameOver message.
pub fn run_game_loop(
    input_rx: Receiver<PlayerInput>,
    render_tx: SyncSender<RenderEvent>,
    peer: Option<PeerChannels>,
    player_name: Option<String>,
) {
    let seed = rand::random::<u64>();
    let mut state = GameState::new(seed);

    let is_network = peer.as_ref().map(|p| p.is_network).unwrap_or(false);

    if is_network {
        state.mode = GameMode::VsNetwork;
    } else if peer.is_some() {
        state.mode = GameMode::VsComputer;
    }

    let mut last_tick = Instant::now();

    let mut peer_board: Option<BoardSnapshot> = None;
    let mut peer_score_view: Option<ScoreView> = None;
    let peer_active_weapons: Vec<ActiveWeaponView> = vec![];
    let peer_arsenal_count: usize = 0;

    // Network-only state
    let mut peer_disconnected = false;
    // game_loop is only spawned after main.rs already received GameStart, so always ready.
    let mut game_started = true;
    let mut network_result: Option<(bool, String, i32)> = None; // (i_won, winner_name, my_delta)
    let mut did_send_game_over = false; // prevent duplicate sends

    // Skip the in-engine title screen for modes that don't need it.
    if is_network || peer.is_none() {
        state.tick(Some(PlayerInput::StartGame), 0);
    }

    loop {
        // ── Drain inputs ──────────────────────────────────────────────────
        let mut input: Option<PlayerInput> = None;
        while let Ok(i) = input_rx.try_recv() {
            input = Some(i);
        }

        let now = Instant::now();
        let elapsed_ms = now.duration_since(last_tick).as_millis() as u32;
        last_tick = now;

        // ── Process peer messages ─────────────────────────────────────────
        if let Some(ref ch) = peer {
            let to_peer = &ch.to_peer;
            while let Ok(msg) = ch.from_peer.try_recv() {
                process_peer_message(
                    &mut state,
                    msg,
                    &mut peer_board,
                    &mut peer_score_view,
                    Some(to_peer),
                    is_network,
                    &mut peer_disconnected,
                    &mut game_started,
                    &mut network_result,
                    player_name.as_deref(),
                );
            }
        }

        if !game_started {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // ── Tick player state ─────────────────────────────────────────────
        let was_pre_game = matches!(state.phase, GamePhase::Title | GamePhase::GameOver { .. });
        let was_playing = matches!(state.phase, GamePhase::Playing | GamePhase::InBazaar);
        let events = state.tick(input, elapsed_ms);
        let now_game_over_loss = matches!(state.phase, GamePhase::GameOver { won: false });

        // In network mode, when local board tops out send GameOver to server so
        // the server can compute ELO and notify both players.
        if is_network && was_playing && now_game_over_loss && !did_send_game_over {
            did_send_game_over = true;
            if let Some(ref ch) = peer {
                let _ = ch.to_peer.try_send(GameMessage::GameOver {
                    winner_id: 0,
                    final_score_p1: state.score.score,
                    final_score_p2: 0,
                    winner_name: String::new(),
                    elo_delta_winner: 0,
                    elo_delta_loser: 0,
                });
            }
        }

        // ── Notify peer of new game (vs-computer only) ───────────────────
        if was_pre_game && matches!(state.phase, GamePhase::Playing) {
            if let Some(ref ch) = peer {
                if !is_network {
                    let _ = ch.to_peer.try_send(GameMessage::NewGame);
                }
            }
            peer_board = None;
            peer_score_view = None;
            peer_disconnected = false;
            network_result = None;
            did_send_game_over = false;
        }

        // ── Bazaar sync (vs-computer only) ────────────────────────────────
        if peer.is_some() && !is_network {
            use battletris_engine::engine::game_state::GameEvent;
            if events.iter().any(|e| matches!(e, GameEvent::BazaarTriggered)) {
                state.ernie_bazaar_done();
            }
        }
        // Network mode: bazaar triggered by BazaarOpen from server.

        // ── Forward player actions to peer ────────────────────────────────
        if let Some(ref ch) = peer {
            use battletris_engine::engine::game_state::GameEvent;
            for event in &events {
                match event {
                    GameEvent::WeaponFired { kind, reflect: false } => {
                        let _ = ch.to_peer.try_send(GameMessage::WeaponLaunched { kind: *kind });
                    }
                    GameEvent::WeaponFired { .. } => {}
                    GameEvent::LinesCleared(n) => {
                        let _ = ch.to_peer.try_send(GameMessage::LinesCleared {
                            count: *n as u32,
                            funds_earned: 0,
                        });
                    }
                    GameEvent::PieceLocked => {
                        // Send board snapshot so the peer can render our board.
                        let _ = ch.to_peer.try_send(GameMessage::BoardUpdate {
                            snapshot: state.board.snapshot(),
                        });
                    }
                    _ => {}
                }
            }

            if !is_network {
                let _ = ch.to_peer.try_send(GameMessage::ScoreUpdate {
                    score: state.score.score,
                    lines: state.score.lines,
                    funds: state.score.funds,
                });
            }
        }

        // ── Single player: apply fired weapons to self (for testing) ─────
        if peer.is_none() {
            use battletris_engine::engine::game_state::GameEvent;
            use rand::SeedableRng;
            use rand::rngs::StdRng;
            for event in &events {
                if let GameEvent::WeaponFired { kind, reflect: false } = event {
                    let mut rng = StdRng::from_entropy();
                    state.apply_incoming_weapon(*kind, &mut rng);
                }
            }
        }

        // ── Build render event ────────────────────────────────────────────
        let render_ev = match &state.phase {
            GamePhase::Title => Some(RenderEvent::Title),

            GamePhase::Playing | GamePhase::InBazaar => {
                let mut view = state.to_playing_view();
                view.opponent_board = apply_board_visibility(&peer_board, view.opponent_board_accuracy);
                view.ernie_active_weapons = peer_active_weapons.clone();
                view.ernie_arsenal_count = peer_arsenal_count;
                view.peer_disconnected = peer_disconnected;
                if let Some(ref sv) = peer_score_view {
                    view.score.op_score = sv.score;
                    view.score.op_lines = sv.lines;
                    if view.score.show_op_funds {
                        view.score.op_funds = sv.funds;
                    }
                }
                Some(RenderEvent::Playing(view))
            }

            GamePhase::GameOver { won } => {
                let (winner_name, elo_delta) = if is_network {
                    if let Some((_, ref name, delta)) = network_result {
                        (Some(name.clone()), Some(delta))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };
                Some(RenderEvent::GameOver {
                    won: *won,
                    score: state.score.score,
                    lines: state.score.lines,
                    winner_name,
                    elo_delta,
                })
            }

            GamePhase::Paused
            | GamePhase::ConnectingToServer
            | GamePhase::WaitingForOpponent => None,
        };

        if let Some(ev) = render_ev {
            let _ = render_tx.try_send(ev);
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}

fn apply_board_visibility(board: &Option<BoardSnapshot>, accuracy: f32) -> Option<BoardSnapshot> {
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

#[allow(clippy::too_many_arguments)]
fn process_peer_message(
    state: &mut GameState,
    msg: GameMessage,
    peer_board: &mut Option<BoardSnapshot>,
    peer_score_view: &mut Option<ScoreView>,
    to_peer: Option<&SyncSender<GameMessage>>,
    is_network: bool,
    peer_disconnected: &mut bool,
    game_started: &mut bool,
    network_result: &mut Option<(bool, String, i32)>,
    player_name: Option<&str>,
) {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    match msg {
        // ── Network session management ─────────────────────────────────────
        GameMessage::GameStart => {
            *game_started = true;
            *peer_disconnected = false;
        }

        GameMessage::BazaarOpen => {
            if is_network {
                state.open_bazaar_now();
                state.ernie_bazaar_done();
            }
        }

        GameMessage::PeerDisconnected => {
            *peer_disconnected = true;
        }

        GameMessage::PeerReconnected => {
            *peer_disconnected = false;
        }

        GameMessage::GameVoid => {
            state.phase = GamePhase::Title;
        }

        // ── Board and score sync ───────────────────────────────────────────
        GameMessage::BoardUpdate { snapshot } => {
            *peer_board = Some(snapshot);
        }

        GameMessage::ScoreUpdate { score, lines, funds } => {
            let mut sv = ScoreView::default();
            sv.score = score;
            sv.lines = lines;
            sv.funds = funds;
            *peer_score_view = Some(sv);
        }

        // ── Weapons ────────────────────────────────────────────────────────
        GameMessage::WeaponLaunched { kind } => {
            let mut rng = StdRng::from_entropy();
            let events = state.apply_incoming_weapon(kind, &mut rng);
            use battletris_engine::engine::game_state::GameEvent;
            for ev in &events {
                if let GameEvent::WeaponFired { kind: k, reflect: true } = ev {
                    if let Some(ch) = to_peer {
                        let _ = ch.try_send(GameMessage::WeaponReflected { kind: *k });
                    }
                }
            }
        }

        GameMessage::WeaponReflected { kind } => {
            let mut rng = StdRng::from_entropy();
            state.apply_incoming_weapon(kind, &mut rng);
        }

        // ── Bazaar (vs-computer) ───────────────────────────────────────────
        GameMessage::BazaarEnd => {
            if !is_network {
                state.ernie_bazaar_done();
            }
        }

        // ── Lines cleared (vs-computer bazaar trigger) ─────────────────────
        GameMessage::LinesCleared { count, .. } => {
            if !is_network && state.score.add_op_lines(count) {
                state.open_bazaar_now();
                state.ernie_bazaar_done();
            }
        }

        GameMessage::FundsReceived { amount } => {
            state.score.funds += amount;
        }

        // ── Game over (server-enriched, sent to both players) ─────────────
        GameMessage::GameOver { winner_name, elo_delta_winner, elo_delta_loser, .. } => {
            if is_network {
                let i_won = player_name.map(|n| n == winner_name).unwrap_or(false);
                let my_delta = if i_won { elo_delta_winner } else { elo_delta_loser };
                *network_result = Some((i_won, winner_name, my_delta));
                // Force both clients to GameOver phase (winner's engine is still in Playing)
                state.phase = GamePhase::GameOver { won: i_won };
            } else {
                state.phase = GamePhase::GameOver { won: true };
            }
        }

        _ => {}
    }
}
