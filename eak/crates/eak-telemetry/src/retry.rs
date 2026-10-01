//! Retry policy with bounded exponential backoff and jitter

use crate::models::ProviderState;
use crate::provider_state::ProviderStateManager;
use crate::{Result, TelemetryError};
use rand::Rng;
use regex::Regex;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter_factor: f64,
    pub respect_retry_after: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 60000,
            jitter_factor: 0.2,
            respect_retry_after: true,
        }
    }
}

impl RetryPolicy {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    pub fn calculate_delay(&self, attempt: u32, retry_after: Option<u64>) -> Duration {
        if self.respect_retry_after {
            if let Some(seconds) = retry_after {
                return Duration::from_secs(seconds);
            }
        }

        let base = self.base_delay_ms * 2_u64.pow(attempt);
        let capped = std::cmp::min(base, self.max_delay_ms);

        let jitter = (capped as f64 * self.jitter_factor) as u64;
        let jitter_range = if jitter > 0 { 2 * jitter } else { 0 };

        let mut rng = rand::thread_rng();
        let actual_jitter = rng.gen_range(0..=jitter_range);

        Duration::from_millis(capped.saturating_add(actual_jitter).saturating_sub(jitter))
    }

    pub fn should_retry(
        &self,
        attempt: u32,
        http_status: u16,
        provider_state: &ProviderState,
    ) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        // Don't retry if provider is offline
        if provider_state.max_retries_exhausted {
            return false;
        }

        // Retry on rate limit (429)
        if http_status == 429 {
            return true;
        }

        // Retry on server errors (5xx)
        if (500..600).contains(&http_status) {
            return true;
        }

        // Retry on timeout (408) and too many requests (429)
        if http_status == 408 || http_status == 429 {
            return true;
        }

        false
    }

    pub fn execute_with_retry<F, T, E>(
        &self,
        provider: &str,
        state_manager: &ProviderStateManager,
        mut operation: F,
    ) -> Result<T>
    where
        F: FnMut() -> std::result::Result<T, E>,
        E: std::fmt::Debug + std::error::Error + 'static,
    {
        let mut attempt = 0;

        loop {
            let provider_state = state_manager.get_state(provider);

            // Check if we should even attempt (rate limited)
            if state_manager.is_rate_limited(provider) {
                if let Some(cooldown) = state_manager.get_cooldown_remaining(provider) {
                    return Err(TelemetryError::Config(format!(
                        "Provider {} is rate limited, cooldown: {}s",
                        provider, cooldown
                    )));
                }
            }

            match operation() {
                Ok(result) => {
                    state_manager.record_success(provider);
                    return Ok(result);
                }
                Err(error) => {
                    // Check if it's a rate limit error
                    let http_status = extract_http_status(&error);

                    if http_status == Some(429) {
                        let retry_after = extract_retry_after(&error);
                        state_manager.record_rate_limit(provider, retry_after);

                        if self.should_retry(attempt, 429, &provider_state) {
                            attempt += 1;
                            let delay = self.calculate_delay(attempt, retry_after);
                            std::thread::sleep(delay);
                            continue;
                        } else {
                            state_manager.mark_retries_exhausted(provider);
                            return Err(TelemetryError::Config(format!(
                                "Max retries ({}) exhausted for {}: {:?}",
                                attempt, provider, error
                            )));
                        }
                    }

                    // Check for other retryable errors
                    if let Some(status) = http_status {
                        if self.should_retry(attempt, status, &provider_state) {
                            attempt += 1;
                            let delay = self.calculate_delay(attempt, None);
                            std::thread::sleep(delay);
                            continue;
                        }
                    }

                    // Non-retryable error
                    state_manager.record_error(provider);
                    return Err(TelemetryError::Config(format!(
                        "Operation failed: {:?}",
                        error
                    )));
                }
            }
        }
    }
}

fn extract_http_status<E: std::error::Error>(error: &E) -> Option<u16> {
    let error_str = format!("{:?}", error);

    // Try to extract HTTP status from common error patterns
    for pattern in [r"(\d{3})", r"status[:\s]+(\d{3})", r"HTTP[/\s]+(\d{3})"] {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(&error_str) {
                if let Ok(status) = caps.get(1).unwrap().as_str().parse::<u16>() {
                    return Some(status);
                }
            }
        }
    }

    None
}

fn extract_retry_after<E: std::error::Error>(error: &E) -> Option<u64> {
    let error_str = format!("{:?}", error);

    // Try to extract Retry-After header value
    for pattern in [r"retry[-\s]?after[:\s]+(\d+)", r"Retry-After[:\s]+(\d+)"] {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(&error_str) {
                if let Ok(seconds) = caps.get(1).unwrap().as_str().parse::<u64>() {
                    return Some(seconds);
                }
            }
        }
    }

    None
}
