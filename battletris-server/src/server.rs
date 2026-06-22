use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;

use battletris_engine::protocol::GameMessage;

use crate::conn::{GameConn, TcpConn};
use crate::db::PlayerDb;
use crate::session;

// ─── Shared state ────────────────────────────────────────────────────────────

pub struct SharedState {
    pub pending: Mutex<Option<(Box<dyn GameConn>, String)>>,
    pub active_names: Mutex<HashSet<String>>,
    pub db: Arc<Mutex<PlayerDb>>,
}

impl SharedState {
    pub fn new(db: Arc<Mutex<PlayerDb>>) -> Self {
        Self {
            pending: Mutex::new(None),
            active_names: Mutex::new(HashSet::new()),
            db,
        }
    }
}

// ─── TCP listener ─────────────────────────────────────────────────────────────

pub async fn run_tcp_listener(port: u16, shared: Arc<SharedState>) {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await
        .unwrap_or_else(|e| panic!("Cannot bind {addr}: {e}"));
    println!("BattleTris TCP relay listening on {addr}");

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => { eprintln!("[TCP] accept error: {e}"); continue; }
        };
        eprintln!("[TCP] connection from {peer}");
        let conn: Box<dyn GameConn> = Box::new(TcpConn::new(stream));
        let shared = Arc::clone(&shared);
        tokio::spawn(async move {
            handle_client(conn, peer, shared).await;
        });
    }
}

// ─── Client handshake + pairing ──────────────────────────────────────────────

pub async fn handle_client(
    mut conn: Box<dyn GameConn>,
    peer: SocketAddr,
    shared: Arc<SharedState>,
) {
    let name = match read_hello(&mut conn).await {
        Some(n) => n,
        None => return,
    };

    if name.is_empty() || name.len() > 16 {
        eprintln!("[SERVER] invalid name from {peer}, dropping");
        return;
    }

    let name_taken = shared.active_names.lock().unwrap().contains(&name);
    if name_taken {
        eprintln!("[SERVER] name '{name}' already in use — rejecting {peer}");
        let _ = conn.write_frame(&GameMessage::NameTaken).await;
        return;
    }

    shared.active_names.lock().unwrap().insert(name.clone());
    if conn.write_frame(&GameMessage::Welcome { assigned_name: name.clone() }).await.is_err() {
        shared.active_names.lock().unwrap().remove(&name);
        return;
    }
    eprintln!("[SERVER] '{name}' joined from {peer}");

    let maybe_other = shared.pending.lock().unwrap().take();

    if let Some((other_conn, other_name)) = maybe_other {
        eprintln!("[SERVER] pairing '{name}' with '{other_name}'");
        let name_a = other_name.clone();
        let name_b = name.clone();
        let shared2 = Arc::clone(&shared);
        tokio::spawn(async move {
            session::run_session(other_conn, name_a.clone(), conn, name_b.clone(), shared2.db.clone()).await;
            let mut names = shared2.active_names.lock().unwrap();
            names.remove(&name_a);
            names.remove(&name_b);
            eprintln!("[SERVER] session ended for '{name_a}' vs '{name_b}'");
        });
    } else {
        eprintln!("[SERVER] '{name}' waiting for opponent");
        shared.pending.lock().unwrap().replace((conn, name));
    }
}

async fn read_hello(conn: &mut Box<dyn GameConn>) -> Option<String> {
    match conn.read_frame().await {
        Ok(GameMessage::Hello { name }) => Some(name),
        _ => None,
    }
}

// ─── Web server ───────────────────────────────────────────────────────────────

pub async fn run_web_server(web_port: u16, web_dir: PathBuf, shared: Arc<SharedState>) {
    use crate::http_server::build_router;
    use std::net::SocketAddr;

    let addr: SocketAddr = format!("0.0.0.0:{web_port}").parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await
        .unwrap_or_else(|e| panic!("Cannot bind web port {addr}: {e}"));
    println!("BattleTris web server listening on http://{addr}");

    let router = build_router(web_dir, shared);
    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("web server error");
}
