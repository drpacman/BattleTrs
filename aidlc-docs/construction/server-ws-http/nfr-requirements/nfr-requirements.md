# NFR Requirements — Unit A: server-ws-http

**Date**: 2026-06-22 (revised)  
**Security Extension**: Enabled (Q10=A) — scaled to local-network deployment context  
**Deployment context**: LAN game server accessed via IP address (e.g. `http://192.168.1.x:7001`). No TLS. No public internet exposure. Known, trusted players.

---

## Revision Notes

Several NFRs from the initial cut assumed a public internet deployment (HSTS, strict Origin allowlists, per-IP rate limiting). These add complexity without security benefit on a LAN with no external exposure. They have been removed or simplified below.

---

## NFR-A-SEC-01 — Basic HTTP Headers (SECURITY-04, simplified)

**Requirement**: Apply a minimal set of HTTP headers that are meaningful without TLS:

| Header | Value |
|---|---|
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Cache-Control` | `no-store` (index.html); `max-age=31536000, immutable` (content-hashed assets) |

**Dropped**: `Strict-Transport-Security` (no TLS on LAN), `Content-Security-Policy` (adds no meaningful protection on a local IP with no internet exposure), `Referrer-Policy` (irrelevant on LAN).

**Implementation**: Two `SetResponseHeaderLayer` entries in the tower-http layer stack.

**Verification**: Integration test: static file response includes both headers.

---

## NFR-A-SEC-02 — WebSocket Frame Validation (SECURITY-05)

**Requirement**: `WsConn::read_frame` must:
1. Reject non-binary WebSocket frames (text, ping, pong) — return `InvalidData`
2. Reject frames whose payload exceeds `MAX_FRAME_BYTES` (see NFR-A-SEC-03)
3. Reject frames that fail bincode deserialisation — return `InvalidData`

This guards against malformed data from a buggy client, not a malicious attacker.

**Verification**: Unit tests for each rejection path.

---

## NFR-A-SEC-03 — Bounded Deserialisation (SECURITY-13)

**Requirement**:
- `MAX_FRAME_BYTES = 65536` (64 KiB) — a named constant
- Both `WsConn` and `TcpConn` enforce this limit before decoding
- A worst-case `GameMessage` is ≤ 512 bytes; 64 KiB is a generous sanity ceiling

**Verification**: Unit test: frame at limit accepted; 1 byte over rejected.

---

## NFR-A-SEC-04 — No Internal Detail in Error Responses (SECURITY-09, simplified)

**Requirement**: HTTP error responses must not include Rust panic output, file paths, or internal error messages. Simple fixed-string bodies are sufficient:
- 404: `"Not Found"`
- 500: `"Internal Server Error"`

No need for elaborate per-code body restrictions given the LAN context — just no stack traces.

**Verification**: Integration test: missing file → 404 with `"Not Found"` body; no file path in body.

---

## NFR-A-SEC-05 — Clean Error Handling and Resource Release (SECURITY-03, SECURITY-15)

**Requirement**:
1. `run_session` must not panic on read/write errors — match `Err`, log at WARN, return
2. The per-IP slot counter (if any rate limiting is retained) must be decremented on all exit paths — use a `ConnGuard` RAII struct
3. Server continues accepting connections after a relay task exits

**Removed**: Formal per-IP rate limiting (NFR-A-SEC-06 from initial version). The LAN context makes this unnecessary. Connection count tracking is removed entirely — the axum listener accepts connections and relies on OS TCP backlog for any natural limiting.

**Verification**: Integration test: relay session disconnects; server accepts a new pair immediately after.

---

## NFR-A-SEC-06 — Dependency Supply Chain (SECURITY-10)

**Requirement**:
- New crates (`async-trait`, `axum`, `tower-http`, `tokio-tungstenite`, `futures-util`) sourced from crates.io only — no `git = "..."` deps
- `Cargo.lock` updated and committed with new dependency resolution

**Verification**: Checked at code generation; noted in Build and Test stage.

---

## Removed NFRs (with rationale)

| Removed NFR | Original rule | Reason dropped |
|---|---|---|
| HSTS header | SECURITY-04 | No TLS; header is meaningless on `http://` |
| Content-Security-Policy | SECURITY-04 | No public exposure; adds no protection on a LAN IP |
| Origin allowlist / CORS enforcement | SECURITY-08 | On a LAN IP there is no meaningful cross-origin threat surface; a strict allowlist would require every player to know the server IP before connecting — impractical |
| Per-IP rate limiting | SECURITY-11 | Known/trusted LAN players; adds state management complexity for no benefit |

---

## Performance NFRs

### NFR-A-PERF-01 — Server Startup

Both listeners ready within 1 second of process start. If `web_dir` path does not exist, server exits immediately with a clear error message before binding any port.

### NFR-A-PERF-02 — Connection Overhead

No per-connection state beyond the session relay task. No lock held across network operations.

---

## Compliance Summary

| Rule | Status | Notes |
|---|---|---|
| SECURITY-01 | N/A | No new data stores |
| SECURITY-02 | N/A | No API gateway |
| SECURITY-03 | Compliant | NFR-A-SEC-05: no panics; WARN logging |
| SECURITY-04 | Compliant (simplified) | NFR-A-SEC-01: 2 headers; HSTS/CSP dropped — no TLS, LAN deployment |
| SECURITY-05 | Compliant | NFR-A-SEC-02: frame type + size + decode validation |
| SECURITY-06 | N/A | No IAM |
| SECURITY-07 | N/A | No cloud networking |
| SECURITY-08 | Dropped | No meaningful cross-origin threat on LAN IP |
| SECURITY-09 | Compliant (simplified) | NFR-A-SEC-04: no stack traces; fixed body strings |
| SECURITY-10 | Compliant | NFR-A-SEC-06: crates.io only |
| SECURITY-11 | Dropped | LAN deployment; trusted players |
| SECURITY-12 | N/A | No authentication |
| SECURITY-13 | Compliant | NFR-A-SEC-03: MAX_FRAME_BYTES before decode |
| SECURITY-14 | N/A | No cloud monitoring |
| SECURITY-15 | Compliant | NFR-A-SEC-05: RAII cleanup; relay error handling |
