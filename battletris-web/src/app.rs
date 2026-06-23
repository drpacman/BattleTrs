use battletris_engine::engine::game_state::{GameMode, GamePhase, PlayerInput};
use battletris_engine::protocol::GameMessage;
use battletris_engine::session::{apply_board_visibility, NetworkSession};

use battletris_renderer::bazaar::draw_bazaar;
use battletris_renderer::game_over::draw_game_over;
use battletris_renderer::playing::{draw_playing, draw_quit_confirm};

use crate::input::InputHandler;
use crate::renderer::CanvasRenderer;
use crate::renderer::screens::{
    draw_connecting, draw_connection_screen, draw_disconnected, draw_name_taken, draw_waiting,
};
use crate::transport::WsTransport;

// ─── Phase state machine ──────────────────────────────────────────────────────

enum Phase {
    Lobby {
        addr_buf: String,
        name_buf: String,
        active_field: usize,
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
    Disconnected,
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
                addr_buf: ws_url_from_location(),
                name_buf: String::new(),
                active_field: 1, // focus the name field by default
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
            Phase::Lobby { mut addr_buf, mut name_buf, mut active_field, mut error } => {
                for key in text_keys {
                    match key.as_str() {
                        "Tab" => {
                            active_field = 1 - active_field;
                        }
                        "Backspace" => {
                            if active_field == 0 { addr_buf.pop(); } else { name_buf.pop(); }
                            error = None;
                        }
                        "Enter" => {
                            let name = name_buf.trim().to_string();
                            let addr = addr_buf.trim().to_string();
                            if name.is_empty() {
                                error = Some("NAME CANNOT BE EMPTY".into());
                            } else if name.len() > 16 {
                                error = Some("NAME TOO LONG (MAX 16 CHARS)".into());
                            } else {
                                match WsTransport::connect(&addr, &name) {
                                    Ok(transport) => {
                                        return Phase::Connecting {
                                            transport,
                                            player_name: name,
                                            addr_display: addr,
                                        };
                                    }
                                    Err(_) => {
                                        error = Some("INVALID SERVER ADDRESS".into());
                                    }
                                }
                            }
                        }
                        k if k.len() == 1 => {
                            let target = if active_field == 0 { &mut addr_buf } else { &mut name_buf };
                            let max_len = if active_field == 0 { 64 } else { 16 };
                            if target.len() < max_len {
                                target.push_str(k);
                            }
                            error = None;
                        }
                        _ => {}
                    }
                }
                Phase::Lobby { addr_buf, name_buf, active_field, error }
            }

            // ── Connecting ────────────────────────────────────────────────────
            Phase::Connecting { transport, player_name, addr_display } => {
                if transport.is_disconnected() {
                    Phase::Lobby {
                        addr_buf: addr_display,
                        name_buf: player_name,
                        active_field: 0,
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
                for msg in transport.drain_incoming() {
                    if matches!(msg, GameMessage::GameStart) {
                        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
                        let mut session = NetworkSession::new(seed, Some(player_name.clone()));
                        session.state.mode = GameMode::VsNetwork;
                        session.state.tick(Some(PlayerInput::StartGame), 0);
                        return Phase::InGame {
                            session,
                            transport,
                            sub: InGameSub::Active,
                        };
                    }
                }
                Phase::WaitingForOpponent { transport, player_name }
            }

            // ── InGame ────────────────────────────────────────────────────────
            Phase::InGame { mut session, transport, mut sub } => {
                if transport.is_disconnected() && sub == InGameSub::Active {
                    sub = InGameSub::Disconnected;
                }

                for msg in transport.drain_incoming() {
                    match msg {
                        GameMessage::Welcome { assigned_name } => {
                            session.player_name = Some(assigned_name);
                        }
                        GameMessage::NameTaken => {
                            sub = InGameSub::NameTaken;
                        }
                        GameMessage::GameStart => {
                            session.peer_disconnected = false;
                            session.state.tick(Some(PlayerInput::StartGame), 0);
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

                // NameTaken / Disconnected: no further key processing or ticking.
                if matches!(sub, InGameSub::NameTaken | InGameSub::Disconnected) {
                    return Phase::InGame { session, transport, sub };
                }

                // sub is Copy — match copies the discriminant so we can freely
                // reassign `sub` inside arms without borrow-checker conflicts.
                let mut input: Option<PlayerInput> = None;
                for key in code_keys {
                    match sub {
                        InGameSub::QuitConfirm => match key.as_str() {
                            "KeyY" => { let _ = web_sys::window().unwrap().location().reload(); }
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
                        InGameSub::NameTaken | InGameSub::Disconnected => unreachable!(),
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
            Phase::Lobby { addr_buf, name_buf, active_field, error } => {
                let mut ctx = self.renderer.backend();
                draw_connection_screen(
                    &mut ctx,
                    addr_buf,
                    name_buf,
                    *active_field,
                    cursor_visible,
                    error.as_deref(),
                );
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
                    InGameSub::Disconnected => {
                        let mut ctx = self.renderer.backend();
                        draw_disconnected(&mut ctx);
                    }
                    InGameSub::QuitConfirm => {
                        let mut view = session.state.to_playing_view();
                        view.opponent_board = apply_board_visibility(
                            &session.peer_board,
                            view.opponent_board_accuracy,
                        );
                        view.peer_disconnected = session.peer_disconnected;
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
    let window = web_sys::window().unwrap();
    let location = window.location();

    if let Ok(search) = location.search() {
        for part in search.split('&') {
            let part = part.trim_start_matches('?');
            if part.starts_with("server=") {
                return part["server=".len()..].to_string();
            }
        }
    }

    let origin = location.origin().unwrap_or_default();
    origin.replace("https://", "wss://").replace("http://", "ws://") + "/game"
}
