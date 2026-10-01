//! EAK Dashboard - Localhost telemetry dashboard for multi-model development

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use eak_observability::{ObservabilityBridge, WebSocketServer};
use eak_telemetry::TelemetryDatabase;
use include_dir::{include_dir, Dir};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod models;
pub mod routes;

use models::AppState;
use routes::*;

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

const DEFAULT_DB_PATH: &str = "data/telemetry/eak-telemetry.db";
const DEFAULT_PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let db_path = args.get(1).map(|s| s.as_str()).unwrap_or(DEFAULT_DB_PATH);
    let port: u16 = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    tracing::info!("Starting EAK Dashboard");
    tracing::info!("Database: {}", db_path);
    tracing::info!("Port: {}", port);

    // Initialize database
    let db = Arc::new(TelemetryDatabase::new(db_path)?);

    // Initialize observability bridge
    let bridge = Arc::new(ObservabilityBridge::new(None));
    let ws_server = Arc::new(WebSocketServer::new(bridge.state()));

    // Start heartbeat
    ws_server.start_heartbeat();

    // Create app state
    let app_state = AppState {
        db,
        bridge: bridge.clone(),
        ws_server: ws_server.clone(),
    };

    // Build router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index_html))
        .route("/api/health", get(health_check))
        .route("/api/models", get(get_model_health))
        .route("/api/global", get(get_global_stats))
        .route("/api/providers", get(get_provider_states))
        .route("/api/requests/recent", get(get_recent_requests))
        .route("/api/requests/model/:model_id", get(get_requests_by_model))
        .route("/api/session/summary", get(get_session_summary))
        .route("/api/daily", get(get_daily_summary))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .layer(cors)
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Dashboard available at http://{}", addr);
    tracing::info!("WebSocket available at ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn websocket_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    state.ws_server.handle_ws(ws, State(state.bridge.state()))
}
