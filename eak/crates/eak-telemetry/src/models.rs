//! Telemetry data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub terminal_id: String,
    pub session_id: String,
    pub agent: String,
    pub model_id: String,
    pub provider: String,
    pub request_type: RequestType,
    pub status: RequestStatus,
    pub http_status: Option<u16>,
    pub latency_ms: u64,
    pub retry_count: u32,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub error_category: Option<ErrorCategory>,
    pub error_message: Option<String>,
    pub rate_limited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    Completion,
    Streaming,
    ToolCall,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestStatus {
    Success,
    Failed,
    RateLimited,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    Authentication,
    RateLimit,
    Network,
    InvalidRequest,
    ModelUnavailable,
    ContentFiltered,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub terminal_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
    pub models_used: Vec<String>,
    pub agents_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: String,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limit_events: u64,
    pub model_breakdown: Vec<ModelDailyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDailyStats {
    pub model_id: String,
    pub requests: u64,
    pub tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub avg_latency_ms: f64,
    pub errors: u64,
    pub rate_limits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    pub model_id: String,
    pub agent: String,
    pub role: String,
    pub last_request: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub last_error: Option<DateTime<Utc>>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limit_count: u64,
    pub total_retries: u64,
    pub avg_latency_ms: f64,
    pub current_status: ModelStatus,
    pub current_cooldown_seconds: Option<u64>,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelStatus {
    Healthy,
    Degraded,
    RateLimited,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub total_input: u64,
    pub total_output: u64,
    pub total: u64,
    pub provider_reported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderState {
    pub provider: String,
    pub status: ProviderStatus,
    pub last_rate_limit: Option<DateTime<Utc>>,
    pub rate_limit_count: u64,
    pub cooldown_until: Option<DateTime<Utc>>,
    pub retry_after_seconds: Option<u64>,
    pub consecutive_429s: u32,
    pub max_retries_exhausted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderStatus {
    Available,
    RateLimited,
    Degraded,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limit_events: u64,
    pub total_tokens: u64,
    pub active_models: u32,
    pub most_used_model: Option<String>,
    pub fastest_model: Option<String>,
    pub error_rate: f64,
}
