//! EAK Telemetry - Shared telemetry and rate-limit infrastructure for EAK multi-model development

pub mod database;
pub mod models;
pub mod provider_state;
pub mod retry;
pub mod telemetry;

pub use database::TelemetryDatabase;
pub use models::*;
pub use provider_state::ProviderStateManager;
pub use retry::RetryPolicy;
pub use telemetry::TelemetryRecorder;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider rate limited: {0}")]
    RateLimited(String),
}

pub type Result<T> = std::result::Result<T, TelemetryError>;
