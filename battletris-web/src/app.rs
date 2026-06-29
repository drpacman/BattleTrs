use rand::SeedableRng;
use rand::rngs::StdRng;

use battletris_engine::ai::{DEFAULT_DIFFICULTY, LEVELS};
use battletris_engine::ernie::ErnieSession;
use battletris_engine::engine::game_state::{GameMode, GamePhase, PlayerInput};
use battletris_engine::protocol::GameMessage;
use battletris_engine::session::NetworkSession;

use battletris_renderer::bazaar::draw_bazaar;
use battletris_renderer::game_over::draw_game_over;
use battletris_renderer::playing::{draw_playing, draw_quit_confirm};
use battletris_renderer::screens::{
    validate_player_name, draw_connecting, draw_connection_screen, draw_name_taken, draw_waiting
};
use battletris_renderer::title::{draw_difficulty_select, draw_title};

use crate::input::InputHandler;
use crate::renderer::CanvasRenderer;
use crate::transport::WsTransport;

// ─── Peer ─────────────────────────────────────────────────────────────────────

enum Peer {
    Network(WsTransport),
    /// Computer opponent. `to_ernie` buffers this frame's session outgoing for
    /// Ernie to process next frame (one-frame lag, imperceptible at 60 fps).
    Computer { ernie: ErnieSession, to_ernie: Vec<GameMessage> },
    Solo,
}

// ─── Phase state machine ──────────────────────────────────────────────────────

enum Phase {
    Title,
    DifficultySelect { selected: usize },
    Lobby {
        name_buf: String,
        error: Option<String>,
    },
    Connecting {
        transport: WsTransport,
        player_name: String,
        addr_display: String,
    },
    WaitingForOpponent {
        transport: WsTransport,
        player_name: String,
    },
    InGame {
        session: NetworkSession,
        peer: Peer,
        sub: InGameSub,
    },
}

#[derive(PartialEq, Clone, Copy)]
enum InGameSub {
    Active,
    QuitConfirm,
    NameTaken,
}

// ─── App ──────────────────────────────────────────────────────────────────────

pub struct WasmApp {
    renderer: CanvasRenderer,
    input: InputHandler,
    /// Wrapped in Option so we can `take()` it in `tick` without a mutable
    /// self-borrow conflict while also calling `self.renderer`.
    phase: Option<Phase>,
    last_ts: f64,
}

impl WasmApp {
    pub fn new() -> Result<Self, wasm_bindgen::JsValue> {
        let renderer = CanvasRenderer::new()?;
        let input = InputHandler::new();

        Ok(Self {
            renderer,
            input,
            phase: Some(Phase::Title),
            last_ts: 0.0,
        })
    }

    pub fn tick(&mut self, ts: f64) {
        let elapsed_ms = if self.last_ts == 0.0 {
            0
        } else {
            (ts - self.last_ts).max(0.0).min(100.0) as u32
        };
        self.last_ts = ts;

        let text_keys = self.input.drain_text();
        let code_keys = self.input.drain();

        let phase = self.phase.take().unwrap();
        self.phase = Some(Self::advance(phase, elapsed_ms, &text_keys, &code_keys));

        self.render(ts);
    }

    // ── Phase transitions ─────────────────────────────────────────────────────

    fn advance(
        phase: Phase,
        elapsed_ms: u32,
        text_keys: &[String],
        code_keys: &[String],
    ) -> Phase {
        match phase {
            // ── Title ─────────────────────────────────────────────────────────
            Phase::Title => {
                for key in code_keys {
                    match key.as_str() {
                        "Enter" | "NumpadEnter" => {
                            return Phase::DifficultySelect { selected: DEFAULT_DIFFICULTY };
                        }
                        "KeyS" => return start_solo_game(),
                        "KeyN" => return Phase::Lobby { name_buf: String::new(), error: None },
                        _ => {}
                    }
                }
                Phase::Title
            }

            // ── DifficultySelect ──────────────────────────────────────────────
            Phase::DifficultySelect { mut selected } => {
                for key in code_keys {
                    match key.as_str() {
                        "ArrowUp" => { if selected > 0 { selected -= 1; } }
                        "ArrowDown" => { if selected + 1 < LEVELS.len() { selected += 1; } }
                        "Enter" | "NumpadEnter" => return start_ernie_game(selected as u8),
                        "Escape" => return Phase::Title,
                        _ => {}
                    }
                }
                Phase::DifficultySelect { selected }
            }

            // ── Lobby ─────────────────────────────────────────────────────────
            Phase::Lobby { mut name_buf, mut error } => {
                for key in text_keys {
                    match key.as_str() {
                        "Backspace" => { name_buf.pop(); error = None; }
                        "Enter" => {
                            let name = name_buf.trim().to_string();
                            if let Some(e) = validate_player_name(&name) {
                                error = Some(e.into());
                            } else {
                                let addr = ws_url_from_location();
                                match WsTransport::connect(&addr, &name) {
                                    Ok(transport) => {
                                        return Phase::Connecting {
                                            transport,
                                            player_name: name,
                                            addr_display: addr,
                                        };
                                    }
                                    Err(_) => {
                                        error = Some("CONNECTION FAILED".into());
                                    }
                                }
                            }
                        }
                        k if k.len() == 1 => {
                            if name_buf.len() < 16 { name_buf.push_str(k); }
                            error = None;
                        }
                        _ => {}
                    }
                }
                for key in code_keys {
                    if key == "Escape" { return Phase::Title; }
                }
                Phase::Lobby { name_buf, error }
            }

            // ── Connecting ────────────────────────────────────────────────────
            Phase::Connecting { transport, player_name, addr_display } => {
                for msg in transport.drain_incoming() {
                    match msg {
                        GameMessage::NameTaken => {
                            return Phase::Lobby {
                                name_buf: player_name,
                                error: Some("NAME ALREADY IN USE".into()),
                            };
                        }
                        GameMessage::GameStart { opponent_name } => {
                            return start_network_game(transport, player_name, opponent_name);
                        }
                        _ => {}
                    }
                }
                if transport.is_disconnected() {
                    Phase::Lobby {
                        name_buf: player_name,
                        error: Some("CONNECTION FAILED".into()),
                    }
                } else if transport.is_connected() {
                    Phase::WaitingForOpponent { transport, player_name }
                } else {
                    Phase::Connecting { transport, player_name, addr_display }
                }
            }

            // ── WaitingForOpponent ────────────────────────────────────────────
            Phase::WaitingForOpponent { transport, player_name } => {
                let mut name_taken = false;
                for msg in transport.drain_incoming() {
                    match msg {
                        GameMessage::GameStart { opponent_name } => {
                            return start_network_game(transport, player_name, opponent_name);
                        }
                        GameMessage::NameTaken => { name_taken = true; }
                        _ => {}
                    }
                }
                if name_taken {
                    return Phase::Lobby {
                        name_buf: player_name,
                        error: Some("NAME ALREADY IN USE".into()),
                    };
                }
                if transport.is_disconnected() {
                    return Phase::Lobby {
                        name_buf: player_name,
                        error: Some("CONNECTION FAILED".into()),
                    };
                }
                for key in code_keys {
                    if key == "Escape" {
                        return Phase::Title;
                    }
                }
                Phase::WaitingForOpponent { transport, player_name }
            }

            // ── InGame ────────────────────────────────────────────────────────
            Phase::InGame { mut session, mut peer, mut sub } => {
                // Network: drain transport and classify messages.
                let mut game_msgs: Vec<GameMessage> = Vec::new();
                if let Peer::Network(transport) = &mut peer {
                    // Drain before checking disconnect — PlayerQuit and other
                    // intentional-close signals arrive in the same frame as the
                    // WebSocket close event.
                    for msg in transport.drain_incoming() {
                        match msg {
                            GameMessage::Welcome { assigned_name } => {
                                session.player_name = Some(assigned_name);
                            }
                            GameMessage::NameTaken => { sub = InGameSub::NameTaken; }
                            GameMessage::GameStart { .. } => {
                                session.peer_disconnected = false;
                                session.state.tick(Some(PlayerInput::StartGame), 0);
                            }
                            GameMessage::PlayerQuit => {
                                return Phase::Lobby {
                                    name_buf: session.player_name.clone().unwrap_or_default(),
                                    error: Some("OPPONENT QUIT".into()),
                                };
                            }
                            GameMessage::PeerDisconnected => {
                                return Phase::Lobby {
                                    name_buf: session.player_name.clone().unwrap_or_default(),
                                    error: None,
                                };
                            }
                            other => game_msgs.push(other),
                        }
                    }
                    if transport.is_disconnected() {
                        return Phase::Lobby {
                            name_buf: session.player_name.clone().unwrap_or_default(),
                            error: Some("CONNECTION LOST".into()),
                        };
                    }
                }

                // NameTaken: no key processing or ticking.
                if sub == InGameSub::NameTaken {
                    return Phase::InGame { session, peer, sub };
                }

                let mut input: Option<PlayerInput> = None;
                let mut rematch = false;
                for key in code_keys {
                    match sub {
                        InGameSub::QuitConfirm => match key.as_str() {
                            "KeyY" => {
                                if let Peer::Network(transport) = &mut peer {
                                    transport.send(&GameMessage::PlayerQuit);
                                    transport.close();
                                }
                                return Phase::Title;
                            }
                            "KeyN" | "Escape" => { sub = InGameSub::Active; }
                            _ => {}
                        },
                        InGameSub::Active => match (&session.state.phase, key.as_str()) {
                            (GamePhase::GameOver { .. }, "Enter" | "NumpadEnter" | "Space") => {
                                rematch = true;
                                break;
                            }
                            (GamePhase::GameOver { .. }, _) => {}
                            (GamePhase::Playing, "Escape") => { sub = InGameSub::QuitConfirm; }
                            (GamePhase::Playing, _) => {
                                if let Some(pi) = playing_key(key) { input = Some(pi); }
                            }
                            (GamePhase::InBazaar, _) => {
                                if let Some(pi) = bazaar_key(key) { input = Some(pi); }
                            }
                            _ => {}
                        },
                        InGameSub::NameTaken => unreachable!(),
                    }
                }

                if rematch {
                    match &mut peer {
                        Peer::Network(_) => return Phase::Title,
                        Peer::Computer { to_ernie, .. } => {
                            to_ernie.push(GameMessage::NewGame);
                            session.reset_peer_state();
                            session.state.tick(Some(PlayerInput::StartGame), 0);
                        }
                        Peer::Solo => {
                            session.state.tick(Some(PlayerInput::StartGame), 0);
                        }
                    }
                }

                // Advance by peer type.
                match &mut peer {
                    Peer::Network(transport) => {
                        let (to_send, _) = session.advance_frame(game_msgs.drain(..), input, elapsed_ms);
                        for msg in to_send { transport.send(&msg); }
                    }
                    Peer::Computer { ernie, to_ernie } => {
                        // Ernie processes last frame's player outgoing, then the player
                        // processes this frame's Ernie responses. One-frame lag (≈16ms).
                        let from_ernie = ernie.step(to_ernie, elapsed_ms);
                        to_ernie.clear();
                        let (new_to_ernie, _) = session.advance_frame(from_ernie, input, elapsed_ms);
                        *to_ernie = new_to_ernie;
                    }
                    Peer::Solo => {
                        let (to_self, _) = session.advance_frame(std::iter::empty(), input, elapsed_ms);
                        let mut rng = StdRng::from_entropy();
                        for msg in to_self {
                            if let GameMessage::WeaponLaunched { kind } = msg {
                                session.state.apply_incoming_weapon(kind, &mut rng);
                            }
                        }
                    }
                }

                Phase::InGame { session, peer, sub }
            }
        }
    }

    // ── Rendering ─────────────────────────────────────────────────────────────

    fn render(&mut self, ts: f64) {
        self.renderer.clear();

        let cursor_visible = (ts as u64 / 500) % 2 == 0;
        let mut ctx = self.renderer.backend();
                
        match self.phase.as_ref().unwrap() {
            Phase::Title => {
                draw_title(&mut ctx);
            }

            Phase::DifficultySelect { selected } => {
                draw_difficulty_select(&mut ctx, *selected);
            }

            Phase::Lobby { name_buf, error } => {
                draw_connection_screen(&mut ctx, None, name_buf, true, cursor_visible, error.as_deref());
            }

            Phase::Connecting { addr_display, .. } => {
                draw_connecting(&mut ctx, addr_display);
            }

            Phase::WaitingForOpponent { player_name, .. } => {
                draw_waiting(&mut ctx, player_name);
            }

            Phase::InGame { session, sub, .. } => {
                match sub {
                    InGameSub::NameTaken => {
                        draw_name_taken(&mut ctx);
                    }
                    InGameSub::QuitConfirm => {
                        let view = session.playing_view();
                        draw_playing(&mut ctx, &view);
                        draw_quit_confirm(&mut ctx);
                    }
                    InGameSub::Active => {
                        let in_baz = matches!(session.state.phase, GamePhase::InBazaar);
                        let game_phase = session.state.phase.clone();

                        match game_phase {
                            GamePhase::Playing | GamePhase::InBazaar => {
                                let view = session.playing_view();
                                if in_baz {
                                    if let Some(ref bv) = view.bazaar_view {
                                        draw_bazaar(&mut ctx, bv);
                                    }
                                } else {
                                    draw_playing(&mut ctx, &view);
                                }
                            }
                            GamePhase::GameOver { won } => {
                                let (winner_name, elo_delta) = match &session.network_result {
                                    Some((_, name, delta)) => (Some(name.as_str()), Some(*delta)),
                                    None => (None, None),
                                };
                                draw_game_over(
                                    &mut ctx,
                                    won,
                                    session.state.score.score,
                                    session.state.score.lines,
                                    winner_name,
                                    elo_delta,
                                );
                            }
                            _ => {
                                draw_waiting(&mut ctx, "");
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Game starters ────────────────────────────────────────────────────────────

fn start_network_game(transport: WsTransport, player_name: String, opponent_name: String) -> Phase {
    let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
    let mut session = NetworkSession::new(seed, Some(player_name));
    session.opponent_name = Some(opponent_name);
    session.state.mode = GameMode::VsNetwork;
    session.state.tick(Some(PlayerInput::StartGame), 0);
    Phase::InGame { session, peer: Peer::Network(transport), sub: InGameSub::Active }
}

fn start_ernie_game(difficulty: u8) -> Phase {
    let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
    let ernie_seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
    let level_name = LEVELS[difficulty as usize].0;

    let mut session = NetworkSession::new(seed, None);
    session.state.mode = GameMode::VsComputer;
    session.state.tick(Some(PlayerInput::StartGame), 0);
    session.opponent_name = Some(format!("Ernie ({level_name})"));

    let ernie = ErnieSession::new(ernie_seed, difficulty);
    Phase::InGame {
        session,
        peer: Peer::Computer { ernie, to_ernie: Vec::new() },
        sub: InGameSub::Active,
    }
}

fn start_solo_game() -> Phase {
    let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
    let mut session = NetworkSession::new(seed, None);
    session.state.tick(Some(PlayerInput::StartGame), 0);
    Phase::InGame { session, peer: Peer::Solo, sub: InGameSub::Active }
}

// ─── Input mapping ────────────────────────────────────────────────────────────

fn bazaar_key(code: &str) -> Option<PlayerInput> {
    match code {
        "ArrowUp" | "PageUp" => Some(PlayerInput::BazaarUp),
        "ArrowDown" | "PageDown" => Some(PlayerInput::BazaarDown),
        "Enter" | "NumpadEnter" => Some(PlayerInput::BazaarBuy),
        "Escape" => Some(PlayerInput::BazaarExit),
        _ => None,
    }
}

fn playing_key(code: &str) -> Option<PlayerInput> {
    match code {
        "ArrowLeft" => Some(PlayerInput::MoveLeft),
        "ArrowRight" => Some(PlayerInput::MoveRight),
        "ArrowUp" => Some(PlayerInput::RotateCW),
        "KeyZ" => Some(PlayerInput::RotateCCW),
        "ArrowDown" => Some(PlayerInput::SoftDrop),
        "UP:ArrowDown" => Some(PlayerInput::SoftDropRelease),
        "Space" => Some(PlayerInput::HardDrop),
        "Enter" | "NumpadEnter" => Some(PlayerInput::StartGame),
        "KeyP" => Some(PlayerInput::Pause),
        "KeyB" => Some(PlayerInput::OpenBazaar),
        "Digit0" => Some(PlayerInput::LaunchWeapon(9)),
        "Digit1" => Some(PlayerInput::LaunchWeapon(0)),
        "Digit2" => Some(PlayerInput::LaunchWeapon(1)),
        "Digit3" => Some(PlayerInput::LaunchWeapon(2)),
        "Digit4" => Some(PlayerInput::LaunchWeapon(3)),
        "Digit5" => Some(PlayerInput::LaunchWeapon(4)),
        "Digit6" => Some(PlayerInput::LaunchWeapon(5)),
        "Digit7" => Some(PlayerInput::LaunchWeapon(6)),
        "Digit8" => Some(PlayerInput::LaunchWeapon(7)),
        "Digit9" => Some(PlayerInput::LaunchWeapon(8)),
        _ => None,
    }
}

// ─── Utilities ────────────────────────────────────────────────────────────────

fn ws_url_from_location() -> String {
    let hostname = web_sys::window()
        .unwrap()
        .location()
        .hostname()
        .unwrap_or_default();
    format!("ws://{hostname}/game")
}
