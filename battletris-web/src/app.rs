use battletris_engine::engine::game_state::{GameMode, GamePhase, PlayerInput};
use battletris_engine::protocol::GameMessage;
use battletris_engine::session::{apply_board_visibility, NetworkSession};

use battletris_renderer::bazaar::draw_bazaar;
use battletris_renderer::game_over::draw_game_over;
use battletris_renderer::playing::{draw_playing, draw_quit_confirm};

use crate::input::InputHandler;
use crate::renderer::CanvasRenderer;
use crate::renderer::screens::{
    draw_connecting, draw_connection_screen, draw_name_taken, draw_waiting,
};
use crate::transport::WsTransport;

// ─── Phase state machine ──────────────────────────────────────────────────────

enum Phase {
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
        transport: WsTransport,
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
            phase: Some(Phase::Lobby {
                name_buf: String::new(),
                error: None,
            }),
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

        // Take the phase out so we can mutate self freely inside `advance`.
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
            // ── Lobby ─────────────────────────────────────────────────────────
            Phase::Lobby { mut name_buf, mut error } => {
                for key in text_keys {
                    match key.as_str() {
                        "Backspace" => { name_buf.pop(); error = None; }
                        "Enter" => {
                            let name = name_buf.trim().to_string();
                            if name.is_empty() {
                                error = Some("NAME CANNOT BE EMPTY".into());
                            } else if name.len() > 16 {
                                error = Some("NAME TOO LONG (MAX 16 CHARS)".into());
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
                Phase::Lobby { name_buf, error }
            }

            // ── Connecting ────────────────────────────────────────────────────
            Phase::Connecting { transport, player_name, addr_display } => {
                // Drain messages before inspecting connection state. On a fast
                // server, Hello → GameStart (or NameTaken + close) can all arrive
                // between two rAF frames, so we must process the queue first or
                // the disconnect check fires before we see the real outcome.
                for msg in transport.drain_incoming() {
                    match msg {
                        GameMessage::NameTaken => {
                            return Phase::Lobby {
                                name_buf: player_name,
                                error: Some("NAME ALREADY IN USE".into()),
                            };
                        }
                        GameMessage::GameStart { opponent_name } => {
                            return start_game(transport, player_name, opponent_name);
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
                        GameMessage::GameStart { opponent_name } => { return start_game(transport, player_name, opponent_name); }
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
                Phase::WaitingForOpponent { transport, player_name }
            }

            // ── InGame ────────────────────────────────────────────────────────
            Phase::InGame { mut session, transport, mut sub } => {
                // Drain messages before checking disconnect state: PlayerQuit (and
                // other intentional-close messages) arrive in the same rAF frame as
                // the WebSocket close event, so we must inspect the queue first or
                // the disconnect check fires and obscures the real reason.
                for msg in transport.drain_incoming() {
                    match msg {
                        GameMessage::Welcome { assigned_name } => {
                            session.player_name = Some(assigned_name);
                        }
                        GameMessage::NameTaken => {
                            sub = InGameSub::NameTaken;
                        }
                        GameMessage::GameStart { .. } => {
                            session.peer_disconnected = false;
                            session.state.tick(Some(PlayerInput::StartGame), 0);
                        }
                        GameMessage::PlayerQuit => {
                            let name = session.player_name.clone().unwrap_or_default();
                            return Phase::Lobby {
                                name_buf: name,
                                error: Some("OPPONENT QUIT".into()),
                            };
                        }
                        GameMessage::PeerDisconnected => {
                            let name = session.player_name.clone().unwrap_or_default();
                            return Phase::Lobby { name_buf: name, error: None };
                        }
                        GameMessage::GameVoid => {
                            session.state.phase = GamePhase::Title;
                        }
                        other => {
                            let replies = session.process_message(other);
                            for r in replies {
                                transport.send(&r);
                            }
                        }
                    }
                }

                if transport.is_disconnected() {
                    let name = session.player_name.clone().unwrap_or_default();
                    return Phase::Lobby {
                        name_buf: name,
                        error: Some("CONNECTION LOST".into()),
                    };
                }

                // NameTaken: no key processing or ticking.
                if sub == InGameSub::NameTaken {
                    return Phase::InGame { session, transport, sub };
                }

                // sub is Copy — match copies the discriminant so we can freely
                // reassign `sub` inside arms without borrow-checker conflicts.
                let mut input: Option<PlayerInput> = None;
                for key in code_keys {
                    match sub {
                        InGameSub::QuitConfirm => match key.as_str() {
                            "KeyY" => {
                                transport.send(&GameMessage::PlayerQuit);
                                // Explicit close flushes the send buffer before
                                // the closing handshake, ensuring PlayerQuit
                                // reaches the server before the socket drops.
                                transport.close();
                                return Phase::Lobby {
                                    name_buf: session.player_name.clone().unwrap_or_default(),
                                    error: None,
                                };
                            }
                            "KeyN" | "Escape" => { sub = InGameSub::Active; }
                            _ => {}
                        },
                        InGameSub::Active => match (&session.state.phase, key.as_str()) {
                            (GamePhase::GameOver { .. }, "Enter" | "NumpadEnter" | "Space") => {
                                let _ = web_sys::window().unwrap().location().reload();
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

                let (_, outgoing) = session.tick(input, elapsed_ms);
                for msg in outgoing {
                    transport.send(&msg);
                }

                Phase::InGame { session, transport, sub }
            }
        }
    }

    // ── Rendering ─────────────────────────────────────────────────────────────

    fn render(&mut self, ts: f64) {
        self.renderer.clear();

        // Cursor blinks every 500 ms using the rAF timestamp.
        let cursor_visible = (ts as u64 / 500) % 2 == 0;

        match self.phase.as_ref().unwrap() {
            Phase::Lobby { name_buf, error } => {
                let mut ctx = self.renderer.backend();
                draw_connection_screen(&mut ctx, name_buf, cursor_visible, error.as_deref());
            }

            Phase::Connecting { addr_display, .. } => {
                let mut ctx = self.renderer.backend();
                draw_connecting(&mut ctx, addr_display);
            }

            Phase::WaitingForOpponent { player_name, .. } => {
                let mut ctx = self.renderer.backend();
                draw_waiting(&mut ctx, player_name);
            }

            Phase::InGame { session, sub, .. } => {
                match sub {
                    InGameSub::NameTaken => {
                        let mut ctx = self.renderer.backend();
                        draw_name_taken(&mut ctx);
                    }
                    InGameSub::QuitConfirm => {
                        let mut view = session.state.to_playing_view();
                        view.opponent_board = apply_board_visibility(
                            &session.peer_board,
                            view.opponent_board_accuracy,
                        );
                        view.peer_disconnected = session.peer_disconnected;
                        view.opponent_name = session.opponent_name.clone();
                        view.player_name = session.player_name.clone();
                        let mut ctx = self.renderer.backend();
                        draw_playing(&mut ctx, &view);
                        draw_quit_confirm(&mut ctx);
                    }
                    InGameSub::Active => {
                        let in_baz = matches!(session.state.phase, GamePhase::InBazaar);
                        let game_phase = session.state.phase.clone();

                        match game_phase {
                            GamePhase::Playing | GamePhase::InBazaar => {
                                let mut view = session.state.to_playing_view();
                                view.opponent_board = apply_board_visibility(
                                    &session.peer_board,
                                    view.opponent_board_accuracy,
                                );
                                view.peer_disconnected = session.peer_disconnected;
                                view.opponent_name = session.opponent_name.clone();
                                view.player_name = session.player_name.clone();
                                let mut ctx = self.renderer.backend();
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
                                    &mut self.renderer.backend(),
                                    won,
                                    session.state.score.score,
                                    session.state.score.lines,
                                    winner_name,
                                    elo_delta,
                                );
                            }
                            _ => {
                                let mut ctx = self.renderer.backend();
                                draw_waiting(&mut ctx, "");
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn start_game(transport: WsTransport, player_name: String, opponent_name: String) -> Phase {
    let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
    let mut session = NetworkSession::new(seed, Some(player_name));
    session.opponent_name = Some(opponent_name);
    session.state.mode = GameMode::VsNetwork;
    session.state.tick(Some(PlayerInput::StartGame), 0);
    Phase::InGame { session, transport, sub: InGameSub::Active }
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
