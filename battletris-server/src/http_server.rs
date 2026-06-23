use std::path::PathBuf;
use std::sync::Arc;

use axum::http::{header, HeaderValue};
use axum::routing::get;
use axum::Router;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::server::SharedState;
use crate::ws_listener::ws_upgrade_handler;

pub fn build_router(web_dir: PathBuf, shared: Arc<SharedState>) -> Router {
    Router::new()
        .route("/game", get(ws_upgrade_handler))
        .fallback_service(ServeDir::new(web_dir))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .with_state(shared)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;
    use std::sync::Mutex;

    fn make_shared() -> Arc<SharedState> {
        use crate::db::PlayerDb;
        Arc::new(SharedState::new(Arc::new(Mutex::new(
            PlayerDb::load(&PathBuf::from("/nonexistent_bt_test")),
        ))))
    }

    async fn serve(web_dir: PathBuf, uri: &str) -> axum::response::Response {
        let router = build_router(web_dir, make_shared());
        router
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn missing_file_returns_404() {
        let dir = tempfile::tempdir().unwrap();
        let resp = serve(dir.path().to_path_buf(), "/missing.html").await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn existing_file_has_security_headers() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("index.html"), "<html></html>").unwrap();
        let resp = serve(dir.path().to_path_buf(), "/index.html").await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(resp.headers().get("x-frame-options").unwrap(), "DENY");
    }
}
