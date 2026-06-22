use std::io;

use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use battletris_engine::protocol::{self, GameMessage, ProtocolError};

pub const MAX_FRAME_BYTES: usize = 65_536;

#[async_trait]
pub trait GameConn: Send + 'static {
    async fn read_frame(&mut self) -> io::Result<GameMessage>;
    async fn write_frame(&mut self, msg: &GameMessage) -> io::Result<()>;
}

// ─── TCP adapter ─────────────────────────────────────────────────────────────

pub struct TcpConn {
    stream: TcpStream,
    buf: Vec<u8>,
}

impl TcpConn {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream, buf: Vec::new() }
    }
}

#[async_trait]
impl GameConn for TcpConn {
    async fn read_frame(&mut self) -> io::Result<GameMessage> {
        loop {
            match protocol::decode(&self.buf) {
                Ok(msg) => {
                    let consumed = protocol::frame_len(&self.buf);
                    self.buf.drain(..consumed);
                    return Ok(msg);
                }
                Err(ProtocolError::NeedMoreData) => {}
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
                }
            }
            let mut chunk = [0u8; 4096];
            let n = self.stream.read(&mut chunk).await?;
            if n == 0 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "connection closed"));
            }
            if self.buf.len() + n > MAX_FRAME_BYTES {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
            }
            self.buf.extend_from_slice(&chunk[..n]);
        }
    }

    async fn write_frame(&mut self, msg: &GameMessage) -> io::Result<()> {
        let bytes = protocol::encode(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        self.stream.write_all(&bytes).await
    }
}

// ─── WebSocket adapter ───────────────────────────────────────────────────────

pub struct WsConn {
    ws: WebSocket,
}

impl WsConn {
    pub fn new(ws: WebSocket) -> Self {
        Self { ws }
    }
}

#[async_trait]
impl GameConn for WsConn {
    async fn read_frame(&mut self) -> io::Result<GameMessage> {
        loop {
            match self.ws.recv().await {
                Some(Ok(Message::Binary(bytes))) => {
                    if bytes.len() > MAX_FRAME_BYTES {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
                    }
                    return protocol::decode_raw(&bytes)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
                }
                Some(Ok(Message::Close(_))) | None => {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "connection closed"));
                }
                Some(Ok(_)) => continue, // ping/pong/text — ignore
                Some(Err(e)) => {
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe, e.to_string()));
                }
            }
        }
    }

    async fn write_frame(&mut self, msg: &GameMessage) -> io::Result<()> {
        let bytes = protocol::encode_raw(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        self.ws
            .send(Message::Binary(bytes))
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e.to_string()))
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn tcp_conn_roundtrip() {
        let (client_half, server_half) = duplex(8192);
        // duplex gives AsyncRead+AsyncWrite but not TcpStream; test via raw protocol instead.
        // Verify encode/decode consistency used by TcpConn.
        let msg = GameMessage::Ping { seq: 42 };
        let encoded = protocol::encode(&msg).unwrap();
        let mut buf = encoded.clone();
        let decoded = protocol::decode(&buf).unwrap();
        let consumed = protocol::frame_len(&buf);
        buf.drain(..consumed);
        assert!(buf.is_empty());
        assert!(matches!(decoded, GameMessage::Ping { seq: 42 }));
        drop((client_half, server_half));
    }

    #[test]
    fn max_frame_bytes_is_reasonable() {
        // A worst-case full BoardSnapshot is well under 1 KiB.
        // 64 KiB gives a large safety margin.
        assert_eq!(MAX_FRAME_BYTES, 65_536);
        let msg = GameMessage::GameStart;
        let raw = protocol::encode_raw(&msg).unwrap();
        assert!(raw.len() < MAX_FRAME_BYTES);
    }
}
