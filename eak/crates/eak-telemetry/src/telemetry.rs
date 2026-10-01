//! Telemetry recorder for model requests

use crate::database::TelemetryDatabase;
use crate::models::*;
use crate::provider_state::ProviderStateManager;
use crate::retry::RetryPolicy;
use crate::Result;
use chrono::Utc;
use regex::Regex;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

pub struct TelemetryRecorder {
    database: Arc<TelemetryDatabase>,
    provider_state: Arc<ProviderStateManager>,
    retry_policy: RetryPolicy,
    terminal_id: String,
    session_id: String,
}

impl TelemetryRecorder {
    pub fn new(database: Arc<TelemetryDatabase>, terminal_id: String, session_id: String) -> Self {
        let provider_state = Arc::new(ProviderStateManager::new(Some((*database).clone())));

        Self {
            database,
            provider_state,
            retry_policy: RetryPolicy::default(),
            terminal_id,
            session_id,
        }
    }

    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    pub fn record_request<F, T, E>(
        &self,
        agent: &str,
        model_id: &str,
        provider: &str,
        request_type: RequestType,
        mut operation: F,
    ) -> Result<T>
    where
        F: FnMut() -> std::result::Result<T, E>,
        E: std::fmt::Debug + std::error::Error + 'static,
    {
        let start = Instant::now();
        let request_id = Uuid::new_v4();
        let timestamp = Utc::now();

        // Execute with retry logic
        let result = self
            .retry_policy
            .execute_with_retry(provider, &self.provider_state, operation)

        let latency = start.elapsed().as_millis() as u64;

        // Extract HTTP status and error info from result
        let (status, http_status, error_category, error_message, rate_limited) = match &result {
            Ok(_) => (RequestStatus::Success, Some(200), None, None, false),
            Err(e) => {
                let err_str = format!("{:?}", e);
                if err_str.contains("rate limited") || err_str.contains("429") {
                    (
                        RequestStatus::RateLimited,
                        Some(429),
                        Some(ErrorCategory::RateLimit),
                        Some(err_str),
                        true,
                    )
                } else {
                    (
                        RequestStatus::Failed,
                        extract_http_status_str(&err_str),
                        categorize_error_str(&err_str),
                        Some(err_str),
                        false,
                    )
                }
            }
        };

        // Create telemetry record
        let record = ModelRequest {
            id: request_id,
            timestamp,
            terminal_id: self.terminal_id.clone(),
            session_id: self.session_id.clone(),
            agent: agent.to_string(),
            model_id: model_id.to_string(),
            provider: provider.to_string(),
            request_type,
            status: status.clone(),
            http_status,
            latency_ms: latency,
            retry_count: 0,     // Could track this more precisely
            input_tokens: None, // Would be filled by actual provider response
            output_tokens: None,
            total_tokens: None,
            error_category,
            error_message,
            rate_limited,
        };

        // Persist to database
        self.database.record_request(&record)?;

        // Update model health cache
        self.update_model_health(&record)?;

        result
    }

    fn update_model_health(&self, record: &ModelRequest) -> Result<()> {
        let mut health = self
            .database
            .get_all_model_health()?
            .into_iter()
            .find(|h| h.model_id == record.model_id)
            .unwrap_or_else(|| ModelHealth {
                model_id: record.model_id.clone(),
                agent: record.agent.clone(),
                role: Self::get_role_for_agent(&record.agent),
                last_request: None,
                last_success: None,
                last_error: None,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                rate_limit_count: 0,
                total_retries: 0,
                avg_latency_ms: 0.0,
                current_status: ModelStatus::Unknown,
                current_cooldown_seconds: None,
                token_usage: TokenUsage::default(),
            });

        health.last_request = Some(record.timestamp);
        health.total_requests += 1;

        // Update running average latency
        let total_latency =
            health.avg_latency_ms * (health.total_requests - 1) as f64 + record.latency_ms as f64;
        health.avg_latency_ms = total_latency / health.total_requests as f64;

        match record.status {
            RequestStatus::Success => {
                health.successful_requests += 1;
                health.last_success = Some(record.timestamp);
                health.current_status = ModelStatus::Healthy;
            }
            RequestStatus::RateLimited => {
                health.failed_requests += 1;
                health.rate_limit_count += 1;
                health.last_error = Some(record.timestamp);
                health.current_status = ModelStatus::RateLimited;
                if let Some(cooldown) = self.provider_state.get_cooldown_remaining(&record.provider)
                {
                    health.current_cooldown_seconds = Some(cooldown);
                }
            }
            RequestStatus::Failed | RequestStatus::Timeout => {
                health.failed_requests += 1;
                health.last_error = Some(record.timestamp);
                if health.failed_requests > health.successful_requests * 2 {
                    health.current_status = ModelStatus::Degraded;
                }
            }
            RequestStatus::Cancelled => {
                // Don't count as success or failure
            }
        }

        self.database.upsert_model_health(&health)?;
        Ok(())
    }

    fn get_role_for_agent(agent: &str) -> String {
        match agent {
            "eak-master" => "Architect/Orchestrator".to_string(),
            "eak-architect" => "Architect".to_string(),
            "eak-engineer" => "Engineer".to_string(),
            "eak-reviewer" => "Reviewer".to_string(),
            "eak-fast" => "Fast/Inspector".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn get_model_health(&self) -> Result<Vec<ModelHealth>> {
        self.database.get_all_model_health()
    }

    pub fn get_global_stats(&self) -> Result<GlobalStats> {
        self.database.get_global_stats()
    }

    pub fn get_provider_states(&self) -> Vec<ProviderState> {
        self.provider_state.get_all_states()
    }

    pub fn is_provider_rate_limited(&self, provider: &str) -> bool {
        self.provider_state.is_rate_limited(provider)
    }

    pub fn get_provider_cooldown(&self, provider: &str) -> Option<u64> {
        self.provider_state.get_cooldown_remaining(provider)
    }
}

fn extract_http_status_str(error_str: &str) -> Option<u16> {
    for pattern in [r"(\d{3})", r"status[:\s]+(\d{3})", r"HTTP[/\s]+(\d{3})"] {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(error_str) {
                if let Ok(status) = caps.get(1).unwrap().as_str().parse::<u16>() {
                    return Some(status);
                }
            }
        }
    }
    None
}

fn categorize_error_str(error_str: &str) -> Option<ErrorCategory> {
    let error_lower = error_str.to_lowercase();

    if error_lower.contains("auth")
        || error_lower.contains("unauthorized")
        || error_lower.contains("401")
    {
        Some(ErrorCategory::Authentication)
    } else if error_lower.contains("rate")
        || error_lower.contains("429")
        || error_lower.contains("quota")
    {
        Some(ErrorCategory::RateLimit)
    } else if error_lower.contains("network")
        || error_lower.contains("timeout")
        || error_lower.contains("connection")
    {
        Some(ErrorCategory::Network)
    } else if error_lower.contains("invalid")
        || error_lower.contains("400")
        || error_lower.contains("bad request")
    {
        Some(ErrorCategory::InvalidRequest)
    } else if error_lower.contains("unavailable")
        || error_lower.contains("503")
        || error_lower.contains("not found")
    {
        Some(ErrorCategory::ModelUnavailable)
    } else if error_lower.contains("filter")
        || error_lower.contains("content")
        || error_lower.contains("safety")
    {
        Some(ErrorCategory::ContentFiltered)
    } else {
        Some(ErrorCategory::Unknown)
    }
}
