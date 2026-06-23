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

use battletris_engine::engine::game_state::PlayerInput;
use battletris_engine::protocol::GameMessage;

use game_loop::{run_game_loop, PeerChannels, RenderEvent};
use net::{ConnectError, NetChannels};
use battletris_renderer::bazaar::draw_bazaar;
use battletris_renderer::game_over::draw_game_over;
use battletris_renderer::playing::{draw_playing, draw_quit_confirm};
use renderer::{
    lobby::{draw_connection_screen, draw_connecting_screen, draw_waiting_screen},
    title::draw_title,
    Renderer,
};

// ─── App state machine ────────────────────────────────────────────────────────

enum AppState {
    TitleMenu,
    ConnectionScreen {
        addr_buf: String,
        name_buf: String,
        active_field: usize, // 0=addr, 1=name
        error: Option<String>,
        cursor_blink: bool,
        blink_timer: Instant,
    },
    /// Background thread doing TcpConnect + handshake.
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
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let mut event_pump = sdl.event_pump()?;

    let mut renderer = Renderer::new(video)?;
    let mut app = AppState::TitleMenu;

    // Quit-confirm overlay state (only active while InGame).
    let mut quit_confirming = false;
    let mut last_playing: Option<RenderEvent> = None;
    let mut in_game = false;

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,

                Event::KeyDown { scancode: Some(sc), keymod, repeat: false, .. } => {
                    // Type printable characters into connection screen fields.
                    // Evaluate the character before taking a mutable borrow of app.
                    let text_ch = if matches!(app, AppState::ConnectionScreen { .. }) {
                        scancode_to_char(sc, keymod)
                    } else {
                        None
                    };
                    if let Some(ch) = text_ch {
                        handle_text_input(&mut app, ch);
                    }

                    if quit_confirming {
                        match sc {
                            Scancode::Y => {
                                quit_confirming = false;
                                in_game = false;
                                last_playing = None;
                                if let AppState::InGame { ref input_tx, .. } = app {
                                    let _ = input_tx.send(PlayerInput::QuitToTitle);
                                }
                            }
                            Scancode::N | Scancode::Escape => {
                                quit_confirming = false;
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match &mut app {
                        AppState::TitleMenu => {
                            match sc {
                                Scancode::Return | Scancode::KpEnter => {
                                    app = start_ernie_game();
                                    in_game = false;
                                    last_playing = None;
                                }
                                Scancode::S => {
                                    app = start_single_player_game();
                                    in_game = false;
                                    last_playing = None;
                                }
                                Scancode::N => {
                                    app = AppState::ConnectionScreen {
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

                        AppState::ConnectionScreen { addr_buf, name_buf, active_field, error, .. } => {
                            match sc {
                                Scancode::Escape => {
                                    app = AppState::TitleMenu;
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
                                app = AppState::TitleMenu;
                            }
                        }

                        AppState::WaitingForOpponent { .. } => {
                            if sc == Scancode::Escape {
                                app = AppState::TitleMenu;
                            }
                        }

                        AppState::InGame { ref input_tx, .. } => {
                            let in_bazaar = matches!(
                                &last_playing,
                                Some(RenderEvent::Playing(v)) if v.bazaar_view.is_some()
                            );
                            if sc == Scancode::Escape && in_bazaar {
                                let _ = input_tx.send(PlayerInput::BazaarExit);
                            } else if sc == Scancode::Escape && in_game {
                                quit_confirming = true;
                            } else if let Some(input) = scancode_to_input(sc) {
                                let _ = input_tx.send(input);
                            }
                        }
                    }
                }

                Event::KeyUp { scancode: Some(sc), .. } => {
                    if !quit_confirming {
                        if let AppState::InGame { ref input_tx, .. } = app {
                            if sc == Scancode::Down {
                                let _ = input_tx.send(PlayerInput::SoftDropRelease);
                            }
                        }
                    }
                }

                _ => {}
            }
        }

        // ── Per-frame state transitions ────────────────────────────────────
        app = transition(app, &mut in_game, &mut last_playing, &mut quit_confirming);

        // ── Cursor blink ───────────────────────────────────────────────────
        if let AppState::ConnectionScreen { cursor_blink, blink_timer, .. } = &mut app {
            if blink_timer.elapsed() >= Duration::from_millis(500) {
                *cursor_blink = !*cursor_blink;
                *blink_timer = Instant::now();
            }
        }

        // ── Render ─────────────────────────────────────────────────────────
        renderer.clear();
        render_frame(&mut renderer, &app, &last_playing, quit_confirming);
        renderer.present();

        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

// ─── State helpers ────────────────────────────────────────────────────────────

fn start_single_player_game() -> AppState {
    let (input_tx, input_rx) = mpsc::channel::<PlayerInput>();
    let (render_tx, render_rx) = mpsc::sync_channel::<RenderEvent>(2);
    thread::spawn(move || run_game_loop(input_rx, render_tx, None, None));
    AppState::InGame { input_tx, render_rx }
}

fn start_ernie_game() -> AppState {
    let (input_tx, input_rx) = mpsc::channel::<PlayerInput>();
    let (render_tx, render_rx) = mpsc::sync_channel::<RenderEvent>(2);

    let (to_ernie_tx, to_ernie_rx) = mpsc::sync_channel::<GameMessage>(32);
    let (from_ernie_tx, from_ernie_rx) = mpsc::sync_channel::<GameMessage>(32);
    let ernie_seed = rand::random::<u64>();
    thread::spawn(move || ernie::run_ernie(to_ernie_rx, from_ernie_tx, ernie_seed));

    let peer = Some(PeerChannels {
        from_peer: from_ernie_rx,
        to_peer: to_ernie_tx,
    });
    thread::spawn(move || run_game_loop(input_rx, render_tx, peer, None));

    AppState::InGame { input_tx, render_rx }
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
    if name.is_empty() {
        return Err("Player name cannot be empty".into());
    }
    if name.len() > 16 {
        return Err("Name must be 16 chars or less".into());
    }
    addr.parse::<SocketAddr>()
        .map_err(|_| "Invalid server address (use host:port)".into())
}

/// Push a printable character into the active connection screen field.
fn handle_text_input(app: &mut AppState, ch: char) {
    let AppState::ConnectionScreen { addr_buf, name_buf, active_field, error, .. } = app else { return };
    let target = if *active_field == 0 { addr_buf } else { name_buf };
    let max_len = if *active_field == 0 { 64 } else { 16 };
    if target.len() < max_len {
        target.push(ch);
    }
    *error = None;
}

/// Per-frame transitions that don't require user input.
fn transition(
    app: AppState,
    in_game: &mut bool,
    last_playing: &mut Option<RenderEvent>,
    quit_confirming: &mut bool,
) -> AppState {
    match app {
        AppState::Connecting { addr_display, result_rx } => {
            match result_rx.try_recv() {
                Ok(Ok((net, player_name))) => {
                    AppState::WaitingForOpponent { net, player_name }
                }
                Ok(Err(ConnectError::NameTaken)) => {
                    AppState::ConnectionScreen {
                        addr_buf: addr_display,
                        name_buf: String::new(),
                        active_field: 1,
                        error: Some("Name already in use — pick another".into()),
                        cursor_blink: true,
                        blink_timer: Instant::now(),
                    }
                }
                Ok(Err(e)) => {
                    AppState::ConnectionScreen {
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
                    AppState::ConnectionScreen {
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
            // Poll for GameStart from server
            match net.from_server.try_recv() {
                Ok(GameMessage::GameStart) => {
                    // Spawn the game loop with network channels
                    let (input_tx, input_rx) = mpsc::channel::<PlayerInput>();
                    let (render_tx, render_rx) = mpsc::sync_channel::<RenderEvent>(2);

                    let name_clone = player_name.clone();
                    let peer = Some(PeerChannels {
                        from_peer: net.from_server,
                        to_peer: net.to_server,
                    });
                    thread::spawn(move || {
                        run_game_loop(input_rx, render_tx, peer, Some(name_clone))
                    });

                    *in_game = false;
                    *last_playing = None;
                    AppState::InGame { input_tx, render_rx }
                }
                Ok(_) => AppState::WaitingForOpponent { net, player_name },
                Err(_) => AppState::WaitingForOpponent { net, player_name },
            }
        }

        AppState::InGame { input_tx: _, ref render_rx } => {
            // Drain render channel
            let mut latest: Option<RenderEvent> = None;
            while let Ok(ev) = render_rx.try_recv() {
                latest = Some(ev);
            }

            let back_to_title = matches!(latest, Some(RenderEvent::Title));

            match &latest {
                Some(RenderEvent::Playing(_)) => {
                    *in_game = true;
                    *last_playing = latest;
                }
                Some(RenderEvent::Title) => {
                    *in_game = false;
                    *quit_confirming = false;
                    *last_playing = None;
                }
                Some(RenderEvent::GameOver { .. }) => {
                    *in_game = false;
                    *quit_confirming = false;
                    *last_playing = latest;
                }
                None => {}
            }

            // When the game loop returns to its own title phase, go back to the
            // real title menu so all key bindings (S, Enter, N) work again.
            if back_to_title {
                AppState::TitleMenu
            } else {
                app
            }
        }

        other => other,
    }
}

// ─── Rendering ────────────────────────────────────────────────────────────────

fn render_frame(
    renderer: &mut Renderer,
    app: &AppState,
    last_playing: &Option<RenderEvent>,
    quit_confirming: bool,
) {
    if quit_confirming {
        if let Some(RenderEvent::Playing(ref view)) = last_playing {
            let baz = view.bazaar_view.clone();
            let mut ctx = renderer.backend();
            draw_playing(&mut ctx, view);
            if let Some(ref b) = baz { draw_bazaar(&mut ctx, b); }
            draw_quit_confirm(&mut ctx);
        } else {
            draw_quit_confirm(&mut renderer.backend());
        }
        return;
    }

    match app {
        AppState::TitleMenu => draw_title(renderer),

        AppState::ConnectionScreen { addr_buf, name_buf, active_field, error, cursor_blink, .. } => {
            draw_connection_screen(renderer, addr_buf, name_buf, *active_field, *cursor_blink, error.as_deref());
        }

        AppState::Connecting { addr_display, .. } => {
            draw_connecting_screen(renderer, addr_display);
        }

        AppState::WaitingForOpponent { player_name, .. } => {
            draw_waiting_screen(renderer, player_name);
        }

        AppState::InGame { .. } => {
            match last_playing {
                Some(RenderEvent::Playing(ref view)) => {
                    let baz = view.bazaar_view.clone();
                    let mut ctx = renderer.backend();
                    draw_playing(&mut ctx, view);
                    if let Some(ref b) = baz { draw_bazaar(&mut ctx, b); }
                }
                Some(RenderEvent::GameOver { won, score, lines, winner_name, elo_delta }) => {
                    draw_game_over(
                        &mut renderer.backend(),
                        *won, *score, *lines,
                        winner_name.as_deref(),
                        *elo_delta,
                    );
                }
                _ => draw_title(renderer),
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
