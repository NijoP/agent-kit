//! Shared provider rate-limit state

use crate::models::{ProviderState, ProviderStatus};
use chrono::{Duration, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProviderStateManager {
    states: Arc<RwLock<HashMap<String, ProviderState>>>,
    database: Option<crate::database::TelemetryDatabase>,
}

impl ProviderStateManager {
    pub fn new(database: Option<crate::database::TelemetryDatabase>) -> Self {
        let manager = Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            database,
        };

        // Load from database if available
        if let Some(_db) = if let Some(db) = &manager.database {manager.database {
            // Could load all provider states here
        }

        manager
    }

    pub fn get_state(&self, provider: &str) -> ProviderState {
        let states = self.states.read();
        states
            .get(provider)
            .cloned()
            .unwrap_or_else(|| ProviderState {
                provider: provider.to_string(),
                status: ProviderStatus::Available,
                last_rate_limit: None,
                rate_limit_count: 0,
                cooldown_until: None,
                retry_after_seconds: None,
                consecutive_429s: 0,
                max_retries_exhausted: false,
            })
    }

    pub fn record_success(&self, provider: &str) {
        let mut states = self.states.write();
        let state = states
            .entry(provider.to_string())
            .or_insert_with(|| ProviderState {
                provider: provider.to_string(),
                status: ProviderStatus::Available,
                last_rate_limit: None,
                rate_limit_count: 0,
                cooldown_until: None,
                retry_after_seconds: None,
                consecutive_429s: 0,
                max_retries_exhausted: false,
            });

        state.status = ProviderStatus::Available;
        state.consecutive_429s = 0;
        state.max_retries_exhausted = false;
        state.cooldown_until = None;
        state.retry_after_seconds = None;

        self.persist(provider, state);
    }

    pub fn record_rate_limit(&self, provider: &str, retry_after: Option<u64>) {
        let mut states = self.states.write();
        let state = states
            .entry(provider.to_string())
            .or_insert_with(|| ProviderState {
                provider: provider.to_string(),
                status: ProviderStatus::Available,
                last_rate_limit: None,
                rate_limit_count: 0,
                cooldown_until: None,
                retry_after_seconds: None,
                consecutive_429s: 0,
                max_retries_exhausted: false,
            });

        state.status = ProviderStatus::RateLimited;
        state.last_rate_limit = Some(Utc::now());
        state.rate_limit_count += 1;
        state.consecutive_429s += 1;
        state.retry_after_seconds = retry_after;

        if let Some(seconds) = retry_after {
            state.cooldown_until = Some(Utc::now() + Duration::seconds(seconds as i64));
        } else {
            // Default cooldown: exponential backoff based on consecutive 429s
            let base_seconds = 60;
            let max_seconds = 3600;
            let cooldown = std::cmp::min(
                base_seconds * 2_u64.pow(state.consecutive_429s.saturating_sub(1)),
                max_seconds,
            );
            state.cooldown_until = Some(Utc::now() + Duration::seconds(cooldown as i64));
        }

        self.persist(provider, state);
    }

    pub fn record_error(&self, provider: &str) {
        let mut states = self.states.write();
        if let Some(state) = states.get_mut(provider) {
            if state.status == ProviderStatus::RateLimited {
                // Keep rate limited status
            } else {
                state.status = ProviderStatus::Degraded;
            }
        }
    }

    pub fn is_rate_limited(&self, provider: &str) -> bool {
        let states = self.states.read();
        if let Some(state) = states.get(provider) {
            if state.status == ProviderStatus::RateLimited {
                if let Some(cooldown) = state.cooldown_until {
                    if Utc::now() < cooldown {
                        return true;
                    }
                }
                // Cooldown expired
                return false;
            }
        }
        false
    }

    pub fn get_cooldown_remaining(&self, provider: &str) -> Option<u64> {
        let states = self.states.read();
        states.get(provider).and_then(|state| {
            state.cooldown_until.map(|cooldown| {
                let now = Utc::now();
                if now < cooldown {
                    (cooldown - now).num_seconds().max(0) as u64
                } else {
                    0
                }
            })
        })
    }

    pub fn mark_retries_exhausted(&self, provider: &str) {
        let mut states = self.states.write();
        if let Some(state) = states.get_mut(provider) {
            state.max_retries_exhausted = true;
            state.status = ProviderStatus::Offline;
            self.persist(provider, state);
        }
    }

    pub fn reset_provider(&self, provider: &str) {
        let mut states = self.states.write();
        if let Some(state) = states.get_mut(provider) {
            state.status = ProviderStatus::Available;
            state.consecutive_429s = 0;
            state.max_retries_exhausted = false;
            state.cooldown_until = None;
            state.retry_after_seconds = None;
            self.persist(provider, state);
        }
    }

    fn persist(fn persist(&self, provider: &str, state: &ProviderState) {self, _provider: fn persist(&self, provider: &str, state: &ProviderState) {str, state: fn persist(&self, provider: &str, state: &ProviderState) {ProviderState) {
        if let Some(db) = &self.database {
            let _ = db.upsert_provider_state(state);
        }
    }

    pub fn get_all_states(&self) -> Vec<ProviderState> {
        let states = self.states.read();
        states.values().cloned().collect()
    }
}
