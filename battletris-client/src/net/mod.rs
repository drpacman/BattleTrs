use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::time::Duration;

use battletris_engine::protocol::{self, GameMessage};

pub struct NetChannels {
    pub from_server: Receiver<GameMessage>,
    pub to_server: SyncSender<GameMessage>,
}

#[derive(Debug)]
pub enum ConnectError {
    Io(std::io::Error),
    NameTaken,
    Protocol(String),
}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectError::Io(e) => write!(f, "connection error: {e}"),
            ConnectError::NameTaken => write!(f, "name already in use"),
            ConnectError::Protocol(s) => write!(f, "protocol error: {s}"),
        }
    }
}

/// Connect to the relay server, perform Hello/Welcome handshake, then spin up
/// background IO threads for the game session. Returns the channel pair and
/// the name assigned by the server.
pub fn connect(addr: SocketAddr, name: String) -> Result<(NetChannels, String), ConnectError> {
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(ConnectError::Io)?;
    stream.set_nodelay(true).ok();

    // ── Handshake (Hello → Welcome / NameTaken) ───────────────────────────
    {
        let mut w = BufWriter::new(&stream);
        let hello = protocol::encode(&GameMessage::Hello { name })
            .map_err(|e| ConnectError::Protocol(e.to_string()))?;
        w.write_all(&hello).map_err(ConnectError::Io)?;
        w.flush().map_err(ConnectError::Io)?;
    }

    let assigned_name = {
        let mut r = BufReader::new(&stream);
        match read_framed(&mut r)? {
            GameMessage::Welcome { assigned_name } => assigned_name,
            GameMessage::NameTaken => return Err(ConnectError::NameTaken),
            _ => return Err(ConnectError::Protocol("unexpected handshake response".into())),
        }
    };

    // ── Spin up IO threads ────────────────────────────────────────────────
    let write_stream = stream.try_clone().map_err(ConnectError::Io)?;

    let (to_game_tx, to_game_rx) = mpsc::sync_channel::<GameMessage>(64);
    let (from_game_tx, from_game_rx) = mpsc::sync_channel::<GameMessage>(64);

    // recv thread: server → game_loop
    thread::Builder::new()
        .name("net-recv".into())
        .spawn(move || {
            let mut r = BufReader::new(stream);
            loop {
                match read_framed(&mut r) {
                    Ok(msg) => {
                        if to_game_tx.send(msg).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        })
        .ok();

    // send thread: game_loop → server
    thread::Builder::new()
        .name("net-send".into())
        .spawn(move || {
            let mut w = BufWriter::new(write_stream);
            while let Ok(msg) = from_game_rx.recv() {
                match protocol::encode(&msg) {
                    Ok(bytes) => {
                        if w.write_all(&bytes).is_err() {
                            break;
                        }
                        let _ = w.flush();
                    }
                    Err(_) => break,
                }
            }
        })
        .ok();

    Ok((NetChannels { from_server: to_game_rx, to_server: from_game_tx }, assigned_name))
}

fn read_framed(r: &mut impl Read) -> Result<GameMessage, ConnectError> {
    let mut header = [0u8; 4];
    r.read_exact(&mut header).map_err(ConnectError::Io)?;
    let len = u32::from_be_bytes(header) as usize;
    if len > 1_048_576 {
        return Err(ConnectError::Protocol("frame too large".into()));
    }
    let mut payload = vec![0u8; len];
    r.read_exact(&mut payload).map_err(ConnectError::Io)?;
    let mut full = Vec::with_capacity(4 + len);
    full.extend_from_slice(&header);
    full.extend_from_slice(&payload);
    protocol::decode(&full).map_err(|e| ConnectError::Protocol(e.to_string()))
}
