//! Dashboard models and app state

use eak_observability::{ObservabilityBridge, WebSocketServer};
use eak_telemetry::TelemetryDatabase;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<TelemetryDatabase>,
    pub bridge: Arc<ObservabilityBridge>,
    pub ws_server: Arc<WebSocketServer>,
}
