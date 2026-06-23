use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{ConnectInfo, State};
use axum::response::IntoResponse;

use crate::conn::{GameConn, WsConn};
use crate::server::{handle_client, SharedState};

pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    State(shared): State<Arc<SharedState>>,
) -> impl IntoResponse {
    eprintln!("[WS] upgrade request from {peer}");
    ws.on_upgrade(move |socket| async move {
        let conn: Box<dyn GameConn> = Box::new(WsConn::new(socket));
        handle_client(conn, peer, shared).await;
    })
}
