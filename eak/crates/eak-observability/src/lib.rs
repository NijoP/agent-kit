//! EAK Observability Bridge - Real-time OpenCode event consumption and telemetry integration

pub mod bridge;
pub mod events;
pub mod state;
pub mod websocket;

pub use bridge::ObservabilityBridge;
pub use events::OpenCodeEvent;
pub use state::{AgentState, ModelStatus, SessionState, TaskState};
pub use websocket::WebSocketServer;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Telemetry error: {0}")]
    Telemetry(#[from] eak_telemetry::TelemetryError),

    #[error("Process error: {0}")]
    Process(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),
}

pub type Result<T> = std::result::Result<T, ObservabilityError>;
