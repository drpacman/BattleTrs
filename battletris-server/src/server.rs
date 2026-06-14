use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use battletris_engine::protocol::{self, GameMessage, ProtocolError};

use crate::db::PlayerDb;
use crate::session;

pub async fn run_server(port: u16, db: Arc<Mutex<PlayerDb>>) {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await
        .unwrap_or_else(|e| panic!("Cannot bind {addr}: {e}"));
    println!("BattleTris relay server listening on {addr}");

    // Pending clients waiting to be paired: (stream, name).
    let pending: Arc<Mutex<Option<(TcpStream, String)>>> = Arc::new(Mutex::new(None));
    // Names of all currently-active (connected) clients.
    let active_names: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => { eprintln!("[SERVER] accept error: {e}"); continue; }
        };
        eprintln!("[SERVER] connection from {addr}");

        let db = Arc::clone(&db);
        let pending = Arc::clone(&pending);
        let active = Arc::clone(&active_names);

        tokio::spawn(async move {
            handle_client(stream, db, pending, active).await;
        });
    }
}

async fn handle_client(
    mut stream: TcpStream,
    db: Arc<Mutex<PlayerDb>>,
    pending: Arc<Mutex<Option<(TcpStream, String)>>>,
    active_names: Arc<Mutex<HashSet<String>>>,
) {
    // Read Hello frame.
    let mut buf = Vec::new();
    let name = match read_hello(&mut stream, &mut buf).await {
        Some(n) => n,
        None => return,
    };

    // Name length validation (BR-NET-02).
    if name.is_empty() || name.len() > 16 {
        eprintln!("[SERVER] invalid name from client, dropping");
        return;
    }

    // Name uniqueness check (BR-NET-01) — check then drop guard before await.
    let name_taken = active_names.lock().unwrap().contains(&name);
    if name_taken {
        eprintln!("[SERVER] name '{name}' already in use — rejecting");
        let _ = write_msg(&mut stream, &GameMessage::NameTaken).await;
        return;
    }

    // Accept the client — insert before await so name is reserved.
    active_names.lock().unwrap().insert(name.clone());
    if write_msg(&mut stream, &GameMessage::Welcome { assigned_name: name.clone() }).await.is_err() {
        active_names.lock().unwrap().remove(&name);
        return;
    }
    eprintln!("[SERVER] '{name}' joined");

    // Pair with a pending client or become the pending client.
    let maybe_other = {
        let mut slot = pending.lock().unwrap();
        slot.take()
    };

    if let Some((other_stream, other_name)) = maybe_other {
        eprintln!("[SERVER] pairing '{name}' with '{other_name}'");
        let name_a = other_name.clone();
        let name_b = name.clone();
        let active = Arc::clone(&active_names);
        tokio::spawn(async move {
            session::run_session(other_stream, name_a.clone(), stream, name_b.clone(), db).await;
            let mut names = active.lock().unwrap();
            names.remove(&name_a);
            names.remove(&name_b);
            eprintln!("[SERVER] session ended for '{name_a}' vs '{name_b}'");
        });
    } else {
        eprintln!("[SERVER] '{name}' waiting for opponent");
        pending.lock().unwrap().replace((stream, name));
    }
}

async fn read_hello(stream: &mut TcpStream, buf: &mut Vec<u8>) -> Option<String> {
    use tokio::io::AsyncReadExt;
    loop {
        match protocol::decode(buf) {
            Ok(GameMessage::Hello { name }) => {
                let consumed = protocol::frame_len(buf);
                buf.drain(..consumed);
                return Some(name);
            }
            Ok(_) => return None, // unexpected first message
            Err(ProtocolError::NeedMoreData) => {}
            Err(_) => return None,
        }
        let mut chunk = [0u8; 4096];
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 { return None; }
        buf.extend_from_slice(&chunk[..n]);
    }
}

async fn write_msg(stream: &mut TcpStream, msg: &GameMessage) -> std::io::Result<()> {
    let bytes = protocol::encode(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    stream.write_all(&bytes).await
}
