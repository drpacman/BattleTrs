use std::sync::mpsc::{Receiver, SyncSender};
use std::time::{Duration, Instant};

use battletris_engine::engine::game_state::{GameMode, GamePhase, PlayerInput, PlayingView};
use battletris_engine::protocol::GameMessage;
use battletris_engine::session::{apply_board_visibility, NetworkSession};

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

pub struct PeerChannels {
    pub from_peer: Receiver<GameMessage>,
    pub to_peer: SyncSender<GameMessage>,
}

pub fn run_game_loop(
    input_rx: Receiver<PlayerInput>,
    render_tx: SyncSender<RenderEvent>,
    peer: Option<PeerChannels>,
    player_name: Option<String>,
    opponent_name: Option<String>,
) {
    let seed = rand::random::<u64>();
    let mut session = NetworkSession::new(seed, player_name);
    session.opponent_name = opponent_name;

    if peer.is_some() {
        session.state.mode = GameMode::VsNetwork;
    }

    // All modes start immediately: solo, vs-Ernie, and vs-network are all
    // spawned only after the match is confirmed, so there is no title-screen wait.
    session.state.tick(Some(PlayerInput::StartGame), 0);

    let mut last_tick = Instant::now();

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
            while let Ok(msg) = ch.from_peer.try_recv() {
                if matches!(msg, GameMessage::PlayerQuit) {
                    session.state.phase = GamePhase::Title;
                } else {
                    let replies = session.process_message(msg);
                    for r in replies {
                        let _ = ch.to_peer.try_send(r);
                    }
                }
            }
        }

        // ── Tick and forward events ───────────────────────────────────────
        let was_pre_game = matches!(session.state.phase, GamePhase::Title | GamePhase::GameOver { .. });

        let (_, outgoing) = session.tick(input, elapsed_ms);

        if let Some(ref ch) = peer {
            for msg in outgoing {
                let _ = ch.to_peer.try_send(msg);
            }
        }

        // ── Notify peer of rematch (Ernie only in practice) ───────────────
        if was_pre_game && matches!(session.state.phase, GamePhase::Playing) {
            if let Some(ref ch) = peer {
                let _ = ch.to_peer.try_send(GameMessage::NewGame);
            }
            session.reset_peer_state();
        }

        // ── Build render event ────────────────────────────────────────────
        let render_ev = match &session.state.phase {
            GamePhase::Title => Some(RenderEvent::Title),

            GamePhase::Playing | GamePhase::InBazaar => {
                let mut view = session.state.to_playing_view();
                view.opponent_board = apply_board_visibility(
                    &session.peer_board,
                    view.opponent_board_accuracy,
                );
                view.peer_disconnected = session.peer_disconnected;
                view.opponent_name = session.opponent_name.clone();
                view.player_name = session.player_name.clone();
                Some(RenderEvent::Playing(view))
            }

            GamePhase::GameOver { won } => {
                let (winner_name, elo_delta) = match &session.network_result {
                    Some((_, name, delta)) => (Some(name.clone()), Some(*delta)),
                    None => (None, None),
                };
                Some(RenderEvent::GameOver {
                    won: *won,
                    score: session.state.score.score,
                    lines: session.state.score.lines,
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
