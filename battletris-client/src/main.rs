mod ernie;
mod game_loop;
mod net;
mod renderer;

use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::{Mod, Scancode};

use battletris_engine::ai::{DEFAULT_DIFFICULTY, LEVELS};
use battletris_engine::engine::game_state::PlayerInput;
use battletris_engine::protocol::GameMessage;

use game_loop::{run_game_loop, PeerChannels, RenderEvent};
use net::{ConnectError, NetChannels};
use battletris_renderer::game_over::draw_game_over;
use battletris_renderer::playing::render_game_view;
use battletris_renderer::screens::{
    validate_player_name, draw_connection_screen, draw_connecting, draw_waiting,
};
use battletris_renderer::title::{draw_difficulty_select, draw_title};
use crate::renderer::Renderer;
// ─── App state machine ────────────────────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
enum InGameSub {
    Active,
    QuitConfirm,
}

enum AppState {
    Title,
    DifficultySelect {
        selected: usize,
    },
    Lobby {
        addr_buf: String,
        name_buf: String,
        active_field: usize, // 0=addr, 1=name
        error: Option<String>,
        cursor_blink: bool,
        blink_timer: Instant,
    },
    Connecting {
        addr_display: String,
        result_rx: mpsc::Receiver<Result<(NetChannels, String), ConnectError>>,
    },
    WaitingForOpponent {
        net: NetChannels,
        player_name: String,
    },
    InGame {
        input_tx: mpsc::Sender<PlayerInput>,
        render_rx: mpsc::Receiver<RenderEvent>,
        sub: InGameSub,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let mut event_pump = sdl.event_pump()?;

    let mut renderer = Renderer::new(video)?;
    let mut app = AppState::Title;
    let mut last_playing: Option<RenderEvent> = None;

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,

                Event::KeyDown { scancode: Some(sc), keymod, repeat: false, .. } => {
                    // Type printable characters into lobby fields.
                    let text_ch = if matches!(app, AppState::Lobby { .. }) {
                        scancode_to_char(sc, keymod)
                    } else {
                        None
                    };
                    if let Some(ch) = text_ch {
                        handle_text_input(&mut app, ch);
                    }

                    match &mut app {
                        AppState::Title => {
                            match sc {
                                Scancode::Return | Scancode::KpEnter => {
                                    app = AppState::DifficultySelect { selected: DEFAULT_DIFFICULTY };
                                }
                                Scancode::S => {
                                    app = start_single_player_game();
                                    last_playing = None;
                                }
                                Scancode::N => {
                                    app = AppState::Lobby {
                                        addr_buf: "127.0.0.1:7001".to_string(),
                                        name_buf: String::new(),
                                        active_field: 0,
                                        error: None,
                                        cursor_blink: true,
                                        blink_timer: Instant::now(),
                                    };
                                }
                                _ => {}
                            }
                        }

                        AppState::DifficultySelect { selected } => {
                            match sc {
                                Scancode::Up => {
                                    if *selected > 0 { *selected -= 1; }
                                }
                                Scancode::Down => {
                                    if *selected < LEVELS.len() - 1 { *selected += 1; }
                                }
                                Scancode::Return | Scancode::KpEnter => {
                                    let difficulty = *selected as u8;
                                    app = start_ernie_game(difficulty);
                                    last_playing = None;
                                }
                                Scancode::Escape => {
                                    app = AppState::Title;
                                }
                                _ => {}
                            }
                        }

                        AppState::Lobby { addr_buf, name_buf, active_field, error, .. } => {
                            match sc {
                                Scancode::Escape => {
                                    app = AppState::Title;
                                }
                                Scancode::Tab => {
                                    *active_field = 1 - *active_field;
                                }
                                Scancode::Return | Scancode::KpEnter => {
                                    let addr_s = addr_buf.trim().to_string();
                                    let name_s = name_buf.trim().to_string();
                                    match validate_connection_input(&addr_s, &name_s) {
                                        Err(e) => { *error = Some(e); }
                                        Ok(addr) => {
                                            app = begin_connect(addr, addr_s, name_s);
                                        }
                                    }
                                }
                                Scancode::Backspace => {
                                    let target = if *active_field == 0 { addr_buf } else { name_buf };
                                    target.pop();
                                    *error = None;
                                }
                                _ => {}
                            }
                        }

                        AppState::Connecting { .. } => {
                            if sc == Scancode::Escape {
                                app = AppState::Title;
                            }
                        }

                        AppState::WaitingForOpponent { .. } => {
                            if sc == Scancode::Escape {
                                app = AppState::Title;
                            }
                        }

                        AppState::InGame { sub, input_tx, .. } => {
                            match sub {
                                InGameSub::QuitConfirm => match sc {
                                    Scancode::Y => {
                                        *sub = InGameSub::Active;
                                        last_playing = None;
                                        let _ = input_tx.send(PlayerInput::QuitToTitle);
                                    }
                                    Scancode::N | Scancode::Escape => {
                                        *sub = InGameSub::Active;
                                    }
                                    _ => {}
                                },
                                InGameSub::Active => {
                                    let in_bazaar = matches!(
                                        &last_playing,
                                        Some(RenderEvent::Playing(v)) if v.bazaar_view.is_some()
                                    );
                                    let in_game_active = matches!(
                                        &last_playing,
                                        Some(RenderEvent::Playing(_))
                                    );
                                    if sc == Scancode::Escape && in_bazaar {
                                        let _ = input_tx.send(PlayerInput::BazaarExit);
                                    } else if sc == Scancode::Escape && in_game_active {
                                        *sub = InGameSub::QuitConfirm;
                                    } else if let Some(input) = scancode_to_input(sc) {
                                        let _ = input_tx.send(input);
                                    }
                                }
                            }
                        }
                    }
                }

                Event::KeyUp { scancode: Some(sc), .. } => {
                    if let AppState::InGame { ref input_tx, ref sub, .. } = app {
                        if *sub != InGameSub::QuitConfirm && sc == Scancode::Down {
                            let _ = input_tx.send(PlayerInput::SoftDropRelease);
                        }
                    }
                }

                _ => {}
            }
        }

        // ── Per-frame state transitions ────────────────────────────────────
        app = transition(app, &mut last_playing);

        // ── Cursor blink ───────────────────────────────────────────────────
        if let AppState::Lobby { cursor_blink, blink_timer, .. } = &mut app {
            if blink_timer.elapsed() >= Duration::from_millis(500) {
                *cursor_blink = !*cursor_blink;
                *blink_timer = Instant::now();
            }
        }

        // ── Render ─────────────────────────────────────────────────────────
        renderer.clear();
        render_frame(&mut renderer, &app, &last_playing);
        renderer.present();

        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

// ─── State helpers ────────────────────────────────────────────────────────────

fn start_single_player_game() -> AppState {
    let (input_tx, input_rx) = mpsc::channel::<PlayerInput>();
    let (render_tx, render_rx) = mpsc::sync_channel::<RenderEvent>(2);
    thread::spawn(move || run_game_loop(input_rx, render_tx, None, None, None));
    AppState::InGame { input_tx, render_rx, sub: InGameSub::Active }
}

fn start_ernie_game(difficulty: u8) -> AppState {
    let (input_tx, input_rx) = mpsc::channel::<PlayerInput>();
    let (render_tx, render_rx) = mpsc::sync_channel::<RenderEvent>(2);

    let (to_ernie_tx, to_ernie_rx) = mpsc::sync_channel::<GameMessage>(32);
    let (from_ernie_tx, from_ernie_rx) = mpsc::sync_channel::<GameMessage>(32);
    let ernie_seed = rand::random::<u64>();
    thread::spawn(move || ernie::run_ernie(to_ernie_rx, from_ernie_tx, ernie_seed, difficulty));

    let (_, level_name) = LEVELS[difficulty as usize];
    let opponent_name = format!("Ernie ({})", level_name);
    let peer = Some(PeerChannels {
        from_peer: from_ernie_rx,
        to_peer: to_ernie_tx,
    });
    thread::spawn(move || run_game_loop(input_rx, render_tx, peer, None, Some(opponent_name)));

    AppState::InGame { input_tx, render_rx, sub: InGameSub::Active }
}

fn begin_connect(addr: SocketAddr, addr_display: String, name: String) -> AppState {
    let (tx, rx) = mpsc::channel::<Result<(NetChannels, String), ConnectError>>();
    thread::spawn(move || {
        let result = net::connect(addr, name);
        let _ = tx.send(result);
    });
    AppState::Connecting { addr_display, result_rx: rx }
}

fn validate_connection_input(addr: &str, name: &str) -> Result<SocketAddr, String> {
    if let Some(e) = validate_player_name(name) {
        return Err(e.to_string());
    }
    addr.parse::<SocketAddr>()
        .map_err(|_| "Invalid server address (use host:port)".into())
}

/// Push a printable character into the active lobby field.
fn handle_text_input(app: &mut AppState, ch: char) {
    let AppState::Lobby { addr_buf, name_buf, active_field, error, .. } = app else { return };
    let target = if *active_field == 0 { addr_buf } else { name_buf };
    let max_len = if *active_field == 0 { 64 } else { 16 };
    if target.len() < max_len {
        target.push(ch);
    }
    *error = None;
}

/// Per-frame transitions that don't require user input.
fn transition(mut app: AppState, last_playing: &mut Option<RenderEvent>) -> AppState {
    match app {
        AppState::Connecting { addr_display, result_rx } => {
            match result_rx.try_recv() {
                Ok(Ok((net, player_name))) => {
                    AppState::WaitingForOpponent { net, player_name }
                }
                Ok(Err(ConnectError::NameTaken)) => {
                    AppState::Lobby {
                        addr_buf: addr_display,
                        name_buf: String::new(),
                        active_field: 1,
                        error: Some("Name already in use — pick another".into()),
                        cursor_blink: true,
                        blink_timer: Instant::now(),
                    }
                }
                Ok(Err(e)) => {
                    AppState::Lobby {
                        addr_buf: addr_display,
                        name_buf: String::new(),
                        active_field: 0,
                        error: Some(e.to_string()),
                        cursor_blink: true,
                        blink_timer: Instant::now(),
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    AppState::Connecting { addr_display, result_rx }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    AppState::Lobby {
                        addr_buf: addr_display,
                        name_buf: String::new(),
                        active_field: 0,
                        error: Some("Connection thread panicked".into()),
                        cursor_blink: true,
                        blink_timer: Instant::now(),
                    }
                }
            }
        }

        AppState::WaitingForOpponent { net, player_name } => {
            match net.from_server.try_recv() {
                Ok(GameMessage::GameStart { opponent_name }) => {
                    let (input_tx, input_rx) = mpsc::channel::<PlayerInput>();
                    let (render_tx, render_rx) = mpsc::sync_channel::<RenderEvent>(2);

                    let name_clone = player_name.clone();
                    let peer = Some(PeerChannels {
                        from_peer: net.from_server,
                        to_peer: net.to_server,
                    });
                    thread::spawn(move || {
                        run_game_loop(input_rx, render_tx, peer, Some(name_clone), Some(opponent_name))
                    });

                    *last_playing = None;
                    AppState::InGame { input_tx, render_rx, sub: InGameSub::Active }
                }
                Ok(_) => AppState::WaitingForOpponent { net, player_name },
                Err(_) => AppState::WaitingForOpponent { net, player_name },
            }
        }

        AppState::InGame { input_tx: _, ref render_rx, ref mut sub } => {
            let mut latest: Option<RenderEvent> = None;
            while let Ok(ev) = render_rx.try_recv() {
                latest = Some(ev);
            }

            let back_to_title = matches!(&latest, Some(RenderEvent::Title));

            match &latest {
                Some(RenderEvent::Playing(_)) => {
                    *last_playing = latest;
                }
                Some(RenderEvent::Title) => {
                    *sub = InGameSub::Active;
                    *last_playing = None;
                }
                Some(RenderEvent::GameOver { .. }) => {
                    *sub = InGameSub::Active;
                    *last_playing = latest;
                }
                None => {}
            }

            if back_to_title {
                AppState::Title
            } else {
                app
            }
        }

        other => other,
    }
}

// ─── Rendering ────────────────────────────────────────────────────────────────

fn render_frame(renderer: &mut Renderer, app: &AppState, last_playing: &Option<RenderEvent>) {
    let mut ctx = renderer.backend();
    match app {
        AppState::Title => draw_title(&mut ctx),

        AppState::DifficultySelect { selected } => draw_difficulty_select(&mut ctx, *selected),

        AppState::Lobby { addr_buf, name_buf, active_field, error, cursor_blink, .. } => {
            draw_connection_screen(
                &mut ctx,
                Some((addr_buf, *active_field == 0)),
                name_buf, *active_field == 1,
                *cursor_blink, error.as_deref(),
            );
        }

        AppState::Connecting { addr_display, .. } => draw_connecting(&mut ctx, addr_display),

        AppState::WaitingForOpponent { player_name, .. } => draw_waiting(&mut ctx, player_name),

        AppState::InGame { sub, .. } => {
            match last_playing {
                Some(RenderEvent::Playing(ref view)) => {
                    render_game_view(&mut ctx, view, *sub == InGameSub::QuitConfirm);
                }
                Some(RenderEvent::GameOver { won, score, lines, winner_name, elo_delta }) => {
                    draw_game_over(&mut ctx, *won, *score, *lines, winner_name.as_deref(), *elo_delta);
                }
                _ => draw_title(&mut ctx),
            }
        }
    }
}

// ─── Input mapping ────────────────────────────────────────────────────────────

/// Map a scancode + modifier to a printable character for text-entry fields.
fn scancode_to_char(sc: Scancode, keymod: Mod) -> Option<char> {
    let shift = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
    match sc {
        Scancode::A => Some(if shift { 'A' } else { 'a' }),
        Scancode::B => Some(if shift { 'B' } else { 'b' }),
        Scancode::C => Some(if shift { 'C' } else { 'c' }),
        Scancode::D => Some(if shift { 'D' } else { 'd' }),
        Scancode::E => Some(if shift { 'E' } else { 'e' }),
        Scancode::F => Some(if shift { 'F' } else { 'f' }),
        Scancode::G => Some(if shift { 'G' } else { 'g' }),
        Scancode::H => Some(if shift { 'H' } else { 'h' }),
        Scancode::I => Some(if shift { 'I' } else { 'i' }),
        Scancode::J => Some(if shift { 'J' } else { 'j' }),
        Scancode::K => Some(if shift { 'K' } else { 'k' }),
        Scancode::L => Some(if shift { 'L' } else { 'l' }),
        Scancode::M => Some(if shift { 'M' } else { 'm' }),
        Scancode::N => Some(if shift { 'N' } else { 'n' }),
        Scancode::O => Some(if shift { 'O' } else { 'o' }),
        Scancode::P => Some(if shift { 'P' } else { 'p' }),
        Scancode::Q => Some(if shift { 'Q' } else { 'q' }),
        Scancode::R => Some(if shift { 'R' } else { 'r' }),
        Scancode::S => Some(if shift { 'S' } else { 's' }),
        Scancode::T => Some(if shift { 'T' } else { 't' }),
        Scancode::U => Some(if shift { 'U' } else { 'u' }),
        Scancode::V => Some(if shift { 'V' } else { 'v' }),
        Scancode::W => Some(if shift { 'W' } else { 'w' }),
        Scancode::X => Some(if shift { 'X' } else { 'x' }),
        Scancode::Y => Some(if shift { 'Y' } else { 'y' }),
        Scancode::Z => Some(if shift { 'Z' } else { 'z' }),
        Scancode::Num0 | Scancode::Kp0 => Some('0'),
        Scancode::Num1 | Scancode::Kp1 => Some('1'),
        Scancode::Num2 | Scancode::Kp2 => Some('2'),
        Scancode::Num3 | Scancode::Kp3 => Some('3'),
        Scancode::Num4 | Scancode::Kp4 => Some('4'),
        Scancode::Num5 | Scancode::Kp5 => Some('5'),
        Scancode::Num6 | Scancode::Kp6 => Some('6'),
        Scancode::Num7 | Scancode::Kp7 => Some('7'),
        Scancode::Num8 | Scancode::Kp8 => Some('8'),
        Scancode::Num9 | Scancode::Kp9 => Some('9'),
        Scancode::Period | Scancode::KpPeriod => Some('.'),
        Scancode::Semicolon => Some(if shift { ':' } else { ';' }),
        Scancode::Minus => Some(if shift { '_' } else { '-' }),
        Scancode::Space => Some(' '),
        _ => None,
    }
}

fn scancode_to_input(sc: Scancode) -> Option<PlayerInput> {
    match sc {
        Scancode::Left      => Some(PlayerInput::MoveLeft),
        Scancode::Right     => Some(PlayerInput::MoveRight),
        Scancode::Up        => Some(PlayerInput::RotateCW),
        Scancode::Z         => Some(PlayerInput::RotateCCW),
        Scancode::Down      => Some(PlayerInput::SoftDrop),
        Scancode::Space     => Some(PlayerInput::HardDrop),
        Scancode::Return    => Some(PlayerInput::StartGame),
        Scancode::P         => Some(PlayerInput::Pause),
        Scancode::Escape    => Some(PlayerInput::QuitToTitle),
        Scancode::Num0      => Some(PlayerInput::LaunchWeapon(9)),
        Scancode::Num1      => Some(PlayerInput::LaunchWeapon(0)),
        Scancode::Num2      => Some(PlayerInput::LaunchWeapon(1)),
        Scancode::Num3      => Some(PlayerInput::LaunchWeapon(2)),
        Scancode::Num4      => Some(PlayerInput::LaunchWeapon(3)),
        Scancode::Num5      => Some(PlayerInput::LaunchWeapon(4)),
        Scancode::Num6      => Some(PlayerInput::LaunchWeapon(5)),
        Scancode::Num7      => Some(PlayerInput::LaunchWeapon(6)),
        Scancode::Num8      => Some(PlayerInput::LaunchWeapon(7)),
        Scancode::Num9      => Some(PlayerInput::LaunchWeapon(8)),
        Scancode::KpEnter   => Some(PlayerInput::BazaarBuy),
        Scancode::PageUp    => Some(PlayerInput::BazaarUp),
        Scancode::PageDown  => Some(PlayerInput::BazaarDown),
        Scancode::B         => Some(PlayerInput::OpenBazaar),
        _                   => None,
    }
}
