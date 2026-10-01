//! Dashboard API routes

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::Utc;
use eak_telemetry::models::*;
use eak_telemetry::TelemetryError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

use crate::models::AppState;

static INDEX_HTML: &str = include_str!("../static/index.html");

pub async fn index_html() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "eak-dashboard",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

pub async fn get_model_health(
    State(state): State<AppState>,
) -> Result<Json<Vec<ModelHealthResponse>>, DashboardError> {
    let health = state.db.get_all_model_health()?;

    let response: Vec<ModelHealthResponse> = health
        .into_iter()
        .map(|h| ModelHealthResponse {
            model_id: h.model_id,
            agent: h.agent,
            role: h.role,
            last_request: h.last_request.map(|d| d.to_rfc3339()),
            last_success: h.last_success.map(|d| d.to_rfc3339()),
            last_error: h.last_error.map(|d| d.to_rfc3339()),
            total_requests: h.total_requests,
            successful_requests: h.successful_requests,
            failed_requests: h.failed_requests,
            rate_limit_count: h.rate_limit_count,
            total_retries: h.total_retries,
            avg_latency_ms: h.avg_latency_ms,
            current_status: format!("{:?}", h.current_status),
            current_cooldown_seconds: h.current_cooldown_seconds,
            token_usage: TokenUsageResponse {
                total_input: h.token_usage.total_input,
                total_output: h.token_usage.total_output,
                total: h.token_usage.total,
                provider_reported: h.token_usage.provider_reported,
            },
        })
        .collect();

    Ok(Json(response))
}

pub async fn get_global_stats(
    State(state): State<AppState>,
) -> Result<Json<GlobalStatsResponse>, DashboardError> {
    let stats = state.db.get_global_stats()?;

    let response = GlobalStatsResponse {
        total_requests: stats.total_requests,
        successful_requests: stats.successful_requests,
        failed_requests: stats.failed_requests,
        rate_limit_events: stats.rate_limit_events,
        total_tokens: stats.total_tokens,
        active_models: stats.active_models,
        most_used_model: stats.most_used_model,
        fastest_model: stats.fastest_model,
        error_rate: stats.error_rate,
    };

    Ok(Json(response))
}

pub async fn get_provider_states(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProviderStateResponse>>, DashboardError> {
    let states = state.db.get_all_model_health()?;

    // Aggregate by provider
    let mut provider_map: HashMap<String, ProviderStateResponse> = HashMap::new();

    for health in states {
        let provider = health
            .model_id
            .split('/')
            .next()
            .unwrap_or("unknown")
            .to_string();

        let entry = provider_map
            .entry(provider.clone())
            .or_insert(ProviderStateResponse {
                provider: provider.clone(),
                status: "Unknown".to_string(),
                last_rate_limit: None,
                rate_limit_count: 0,
                cooldown_until: None,
                retry_after_seconds: None,
                consecutive_429s: 0,
                max_retries_exhausted: false,
                models: Vec::new(),
            });

        entry.rate_limit_count += health.rate_limit_count;
        entry.models.push(health.model_id);

        if health.current_status == ModelStatus::RateLimited {
            entry.status = "RateLimited".to_string();
        } else if entry.status == "Unknown" && health.current_status == ModelStatus::Healthy {
            entry.status = "Available".to_string();
        }
    }

    let response: Vec<ProviderStateResponse> = provider_map.into_values().collect();
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct RecentRequestsQuery {
    limit: Option<usize>,
}

pub async fn get_recent_requests(
    State(state): State<AppState>,
    Query(params): Query<RecentRequestsQuery>,
) -> Result<Json<Vec<ModelRequestResponse>>, DashboardError> {
    let limit = params.limit.unwrap_or(100);
    let requests = state.db.get_recent_requests(limit)?;

    let response: Vec<ModelRequestResponse> = requests
        .into_iter()
        .map(|r| ModelRequestResponse {
            id: r.id.to_string(),
            timestamp: r.timestamp.to_rfc3339(),
            terminal_id: r.terminal_id,
            session_id: r.session_id,
            agent: r.agent,
            model_id: r.model_id,
            provider: r.provider,
            request_type: format!("{:?}", r.request_type),
            status: format!("{:?}", r.status),
            http_status: r.http_status,
            latency_ms: r.latency_ms,
            retry_count: r.retry_count,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            total_tokens: r.total_tokens,
            error_category: r.error_category.map(|e| format!("{:?}", e)),
            error_message: r.error_message,
            rate_limited: r.rate_limited,
        })
        .collect();

    Ok(Json(response))
}

pub async fn get_requests_by_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<Json<Vec<ModelRequestResponse>>, DashboardError> {
    let requests = state.db.get_requests_by_model(&model_id, 100)?;

    let response: Vec<ModelRequestResponse> = requests
        .into_iter()
        .map(|r| ModelRequestResponse {
            id: r.id.to_string(),
            timestamp: r.timestamp.to_rfc3339(),
            terminal_id: r.terminal_id,
            session_id: r.session_id,
            agent: r.agent,
            model_id: r.model_id,
            provider: r.provider,
            request_type: format!("{:?}", r.request_type),
            status: format!("{:?}", r.status),
            http_status: r.http_status,
            latency_ms: r.latency_ms,
            retry_count: r.retry_count,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            total_tokens: r.total_tokens,
            error_category: r.error_category.map(|e| format!("{:?}", e)),
            error_message: r.error_message,
            rate_limited: r.rate_limited,
        })
        .collect();

    Ok(Json(response))
}

pub async fn get_session_summary(
    State(state): State<AppState>,
) -> Result<Json<Vec<SessionSummaryResponse>>, DashboardError> {
    // For now, return recent requests grouped by session
    let requests = state.db.get_recent_requests(500)?;

    let mut sessions: HashMap<String, SessionSummaryResponse> = HashMap::new();

    for req in requests {
        let entry = sessions
            .entry(req.session_id.clone())
            .or_insert(SessionSummaryResponse {
                session_id: req.session_id.clone(),
                terminal_id: req.terminal_id,
                start_time: req.timestamp.to_rfc3339(),
                end_time: None,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_tokens: 0,
                models_used: Vec::new(),
                agents_used: Vec::new(),
            });

        entry.total_requests += 1;
        if req.status == RequestStatus::Success {
            entry.successful_requests += 1;
        } else {
            entry.failed_requests += 1;
        }

        if let Some(tokens) = req.input_tokens {
            entry.total_input_tokens += tokens as u64;
        }
        if let Some(tokens) = req.output_tokens {
            entry.total_output_tokens += tokens as u64;
        }
        if let Some(tokens) = req.total_tokens {
            entry.total_tokens += tokens as u64;
        }

        if !entry.models_used.contains(&req.model_id) {
            entry.models_used.push(req.model_id);
        }
        if !entry.agents_used.contains(&req.agent) {
            entry.agents_used.push(req.agent);
        }

        entry.end_time = Some(req.timestamp.to_rfc3339());
    }

    let response: Vec<SessionSummaryResponse> = sessions.into_values().collect();
    Ok(Json(response))
}

pub async fn get_daily_summary(
    State(state): State<AppState>,
) -> Result<Json<Vec<DailySummaryResponse>>, DashboardError> {
    // Return a placeholder for now - would need more complex aggregation
    Ok(Json(Vec::new()))
}

#[derive(Serialize)]
pub struct ModelHealthResponse {
    model_id: String,
    agent: String,
    role: String,
    last_request: Option<String>,
    last_success: Option<String>,
    last_error: Option<String>,
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    rate_limit_count: u64,
    total_retries: u64,
    avg_latency_ms: f64,
    current_status: String,
    current_cooldown_seconds: Option<u64>,
    token_usage: TokenUsageResponse,
}

#[derive(Serialize)]
pub struct TokenUsageResponse {
    total_input: u64,
    total_output: u64,
    total: u64,
    provider_reported: bool,
}

#[derive(Serialize)]
pub struct GlobalStatsResponse {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    rate_limit_events: u64,
    total_tokens: u64,
    active_models: u32,
    most_used_model: Option<String>,
    fastest_model: Option<String>,
    error_rate: f64,
}

#[derive(Serialize)]
pub struct ProviderStateResponse {
    provider: String,
    status: String,
    last_rate_limit: Option<String>,
    rate_limit_count: u64,
    cooldown_until: Option<String>,
    retry_after_seconds: Option<u64>,
    consecutive_429s: u32,
    max_retries_exhausted: bool,
    models: Vec<String>,
}

#[derive(Serialize)]
pub struct ModelRequestResponse {
    id: String,
    timestamp: String,
    terminal_id: String,
    session_id: String,
    agent: String,
    model_id: String,
    provider: String,
    request_type: String,
    status: String,
    http_status: Option<u16>,
    latency_ms: u64,
    retry_count: u32,
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    total_tokens: Option<u32>,
    error_category: Option<String>,
    error_message: Option<String>,
    rate_limited: bool,
}

#[derive(Serialize)]
pub struct SessionSummaryResponse {
    session_id: String,
    terminal_id: String,
    start_time: String,
    end_time: Option<String>,
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_input_tokens: u64,
    total_output_tokens: u64,
    total_tokens: u64,
    models_used: Vec<String>,
    agents_used: Vec<String>,
}

#[derive(Serialize)]
pub struct DailySummaryResponse {
    date: String,
    total_requests: u64,
    total_tokens: u64,
    total_input_tokens: u64,
    total_output_tokens: u64,
    successful_requests: u64,
    failed_requests: u64,
    rate_limit_events: u64,
    model_breakdown: Vec<ModelDailyStatsResponse>,
}

#[derive(Serialize)]
pub struct ModelDailyStatsResponse {
    model_id: String,
    requests: u64,
    tokens: u64,
    input_tokens: u64,
    output_tokens: u64,
    avg_latency_ms: f64,
    errors: u64,
    rate_limits: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("Database error: {0}")]
    Database(#[from] eak_telemetry::TelemetryError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl IntoResponse for DashboardError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            DashboardError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DashboardError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "error": self.to_string(),
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
