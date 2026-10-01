//! SQLite database for telemetry persistence

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::models::*;
use crate::{Result, TelemetryError};

static DEFAULT_DB_PATH: &str = "data/telemetry/eak-telemetry.db";

static DATABASE: Lazy<Arc<Mutex<Option<TelemetryDatabase>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn init_global_db(path: Option<&str>) -> Result<()> {
    let path = path.unwrap_or(DEFAULT_DB_PATH);
    let db = TelemetryDatabase::new(path)?;
    *DATABASE.lock().unwrap() = Some(db);
    Ok(())
}

pub fn get_global_db() -> Result<Arc<Mutex<TelemetryDatabase>>> {
    let guard = DATABASE.lock().unwrap();
    guard
        .as_ref()
        .map(|db| Arc::new(Mutex::new(db.clone())))
        .ok_or_else(|| {
            TelemetryError::Config("Database not initialized. Call init_global_db() first.".into())
        })
}

pub struct TelemetryDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl Clone for TelemetryDatabase {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

impl TelemetryDatabase {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Model requests table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS model_requests (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                terminal_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                agent TEXT NOT NULL,
                model_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                request_type TEXT NOT NULL,
                status TEXT NOT NULL,
                http_status INTEGER,
                latency_ms INTEGER NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0,
                input_tokens INTEGER,
                output_tokens INTEGER,
                total_tokens INTEGER,
                error_category TEXT,
                error_message TEXT,
                rate_limited INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_model_requests_timestamp ON model_requests(timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_model_requests_model_id ON model_requests(model_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_model_requests_session_id ON model_requests(session_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_model_requests_terminal_id ON model_requests(terminal_id)",
            [],
        )?;

        // Sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                terminal_id TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                total_requests INTEGER NOT NULL DEFAULT 0,
                successful_requests INTEGER NOT NULL DEFAULT 0,
                failed_requests INTEGER NOT NULL DEFAULT 0,
                total_input_tokens INTEGER NOT NULL DEFAULT 0,
                total_output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                models_used TEXT NOT NULL DEFAULT '[]',
                agents_used TEXT NOT NULL DEFAULT '[]'
            )",
            [],
        )?;

        // Daily summaries table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_summaries (
                date TEXT PRIMARY KEY,
                total_requests INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                total_input_tokens INTEGER NOT NULL DEFAULT 0,
                total_output_tokens INTEGER NOT NULL DEFAULT 0,
                successful_requests INTEGER NOT NULL DEFAULT 0,
                failed_requests INTEGER NOT NULL DEFAULT 0,
                rate_limit_events INTEGER NOT NULL DEFAULT 0,
                model_breakdown TEXT NOT NULL DEFAULT '[]'
            )",
            [],
        )?;

        // Provider state table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS provider_state (
                provider TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                last_rate_limit TEXT,
                rate_limit_count INTEGER NOT NULL DEFAULT 0,
                cooldown_until TEXT,
                retry_after_seconds INTEGER,
                consecutive_429s INTEGER NOT NULL DEFAULT 0,
                max_retries_exhausted INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        // Model health table (cached)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS model_health (
                model_id TEXT PRIMARY KEY,
                agent TEXT NOT NULL,
                role TEXT NOT NULL,
                last_request TEXT,
                last_success TEXT,
                last_error TEXT,
                total_requests INTEGER NOT NULL DEFAULT 0,
                successful_requests INTEGER NOT NULL DEFAULT 0,
                failed_requests INTEGER NOT NULL DEFAULT 0,
                rate_limit_count INTEGER NOT NULL DEFAULT 0,
                total_retries INTEGER NOT NULL DEFAULT 0,
                avg_latency_ms REAL NOT NULL DEFAULT 0.0,
                current_status TEXT NOT NULL DEFAULT 'Unknown',
                current_cooldown_seconds INTEGER,
                token_usage TEXT NOT NULL DEFAULT '{}'
            )",
            [],
        )?;

        Ok(())
    }

    pub fn record_request(&self, request: &ModelRequest) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO model_requests (
                id, timestamp, terminal_id, session_id, agent, model_id, provider,
                request_type, status, http_status, latency_ms, retry_count,
                input_tokens, output_tokens, total_tokens, error_category,
                error_message, rate_limited
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                request.id.to_string(),
                request.timestamp.to_rfc3339(),
                request.terminal_id,
                request.session_id,
                request.agent,
                request.model_id,
                request.provider,
                serde_json::to_string(&request.request_type)?,
                serde_json::to_string(&request.status)?,
                request.http_status,
                request.latency_ms as i64,
                request.retry_count as i64,
                request.input_tokens.map(|v| v as i64),
                request.output_tokens.map(|v| v as i64),
                request.total_tokens.map(|v| v as i64),
                request
                    .error_category
                    .as_ref()
                    .map(|e| serde_json::to_string(e))
                    .transpose()?,
                request.error_message,
                request.rate_limited as i64,
            ],
        )?;
        Ok(())
    }

    pub fn get_recent_requests(&self, limit: usize) -> Result<Vec<ModelRequest>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, terminal_id, session_id, agent, model_id, provider,
                    request_type, status, http_status, latency_ms, retry_count,
                    input_tokens, output_tokens, total_tokens, error_category,
                    error_message, rate_limited
             FROM model_requests
             ORDER BY timestamp DESC
             LIMIT ?",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ModelRequest {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                terminal_id: row.get(2)?,
                session_id: row.get(3)?,
                agent: row.get(4)?,
                model_id: row.get(5)?,
                provider: row.get(6)?,
                request_type: serde_json::from_str(&row.get::<_, String>(7)?)
                    .unwrap_or(RequestType::Completion),
                status: serde_json::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or(RequestStatus::Failed),
                http_status: row.get(9)?,
                latency_ms: row.get::<_, i64>(10)? as u64,
                retry_count: row.get::<_, i64>(11)? as u32,
                input_tokens: row.get::<_, Option<i64>>(12)?.map(|v| v as u32),
                output_tokens: row.get::<_, Option<i64>>(13)?.map(|v| v as u32),
                total_tokens: row.get::<_, Option<i64>>(14)?.map(|v| v as u32),
                error_category: row
                    .get::<_, Option<String>>(15)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                error_message: row.get(16)?,
                rate_limited: row.get::<_, i64>(17)? != 0,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_requests_by_model(&self, model_id: &str, limit: usize) -> Result<Vec<ModelRequest>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, terminal_id, session_id, agent, model_id, provider,
                    request_type, status, http_status, latency_ms, retry_count,
                    input_tokens, output_tokens, total_tokens, error_category,
                    error_message, rate_limited
             FROM model_requests
             WHERE model_id = ?
             ORDER BY timestamp DESC
             LIMIT ?",
        )?;

        let rows = stmt.query_map(params![model_id, limit as i64], |row| {
            Ok(ModelRequest {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                terminal_id: row.get(2)?,
                session_id: row.get(3)?,
                agent: row.get(4)?,
                model_id: row.get(5)?,
                provider: row.get(6)?,
                request_type: serde_json::from_str(&row.get::<_, String>(7)?)
                    .unwrap_or(RequestType::Completion),
                status: serde_json::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or(RequestStatus::Failed),
                http_status: row.get(9)?,
                latency_ms: row.get::<_, i64>(10)? as u64,
                retry_count: row.get::<_, i64>(11)? as u32,
                input_tokens: row.get::<_, Option<i64>>(12)?.map(|v| v as u32),
                output_tokens: row.get::<_, Option<i64>>(13)?.map(|v| v as u32),
                total_tokens: row.get::<_, Option<i64>>(14)?.map(|v| v as u32),
                error_category: row
                    .get::<_, Option<String>>(15)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                error_message: row.get(16)?,
                rate_limited: row.get::<_, i64>(17)? != 0,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_provider_state(&self, provider: &str) -> Result<Option<ProviderState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT provider, status, last_rate_limit, rate_limit_count, cooldown_until,
                    retry_after_seconds, consecutive_429s, max_retries_exhausted
             FROM provider_state
             WHERE provider = ?",
        )?;

        let result = stmt
            .query_row(params![provider], |row| {
                Ok(ProviderState {
                    provider: row.get(0)?,
                    status: serde_json::from_str(&row.get::<_, String>(1)?)
                        .unwrap_or(ProviderStatus::Available),
                    last_rate_limit: row.get::<_, Option<String>>(2)?.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|d| d.with_timezone(&Utc))
                    }),
                    rate_limit_count: row.get(3)?,
                    cooldown_until: row.get::<_, Option<String>>(4)?.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|d| d.with_timezone(&Utc))
                    }),
                    retry_after_seconds: row.get(5)?,
                    consecutive_429s: row.get(6)?,
                    max_retries_exhausted: row.get::<_, i64>(7)? != 0,
                })
            })
            .optional()?;

        Ok(result)
    }

    pub fn upsert_provider_state(&self, state: &ProviderState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO provider_state (provider, status, last_rate_limit, rate_limit_count, cooldown_until, retry_after_seconds, consecutive_429s, max_retries_exhausted)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(provider) DO UPDATE SET
                status = excluded.status,
                last_rate_limit = excluded.last_rate_limit,
                rate_limit_count = excluded.rate_limit_count,
                cooldown_until = excluded.cooldown_until,
                retry_after_seconds = excluded.retry_after_seconds,
                consecutive_429s = excluded.consecutive_429s,
                max_retries_exhausted = excluded.max_retries_exhausted",
            params![
                state.provider,
                serde_json::to_string(&state.status)?,
                state.last_rate_limit.map(|d| d.to_rfc3339()),
                state.rate_limit_count as i64,
                state.cooldown_until.map(|d| d.to_rfc3339()),
                state.retry_after_seconds.map(|v| v as i64),
                state.consecutive_429s as i64,
                state.max_retries_exhausted as i64,
            ],
        )?;
        Ok(())
    }

    pub fn upsert_model_health(&self, health: &ModelHealth) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO model_health (model_id, agent, role, last_request, last_success, last_error,
                total_requests, successful_requests, failed_requests, rate_limit_count,
                total_retries, avg_latency_ms, current_status, current_cooldown_seconds, token_usage)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(model_id) DO UPDATE SET
                agent = excluded.agent,
                role = excluded.role,
                last_request = excluded.last_request,
                last_success = excluded.last_success,
                last_error = excluded.last_error,
                total_requests = excluded.total_requests,
                successful_requests = excluded.successful_requests,
                failed_requests = excluded.failed_requests,
                rate_limit_count = excluded.rate_limit_count,
                total_retries = excluded.total_retries,
                avg_latency_ms = excluded.avg_latency_ms,
                current_status = excluded.current_status,
                current_cooldown_seconds = excluded.current_cooldown_seconds,
                token_usage = excluded.token_usage",
            params![
                health.model_id,
                health.agent,
                health.role,
                health.last_request.map(|d| d.to_rfc3339()),
                health.last_success.map(|d| d.to_rfc3339()),
                health.last_error.map(|d| d.to_rfc3339()),
                health.total_requests as i64,
                health.successful_requests as i64,
                health.failed_requests as i64,
                health.rate_limit_count as i64,
                health.total_retries as i64,
                health.avg_latency_ms,
                serde_json::to_string(&health.current_status)?,
                health.current_cooldown_seconds.map(|v| v as i64),
                serde_json::to_string(&health.token_usage)?,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_model_health(&self) -> Result<Vec<ModelHealth>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT model_id, agent, role, last_request, last_success, last_error,
                    total_requests, successful_requests, failed_requests, rate_limit_count,
                    total_retries, avg_latency_ms, current_status, current_cooldown_seconds, token_usage
             FROM model_health"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ModelHealth {
                model_id: row.get(0)?,
                agent: row.get(1)?,
                role: row.get(2)?,
                last_request: row.get::<_, Option<String>>(3)?.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                }),
                last_success: row.get::<_, Option<String>>(4)?.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                }),
                last_error: row.get::<_, Option<String>>(5)?.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                }),
                total_requests: row.get::<_, i64>(6)? as u64,
                successful_requests: row.get::<_, i64>(7)? as u64,
                failed_requests: row.get::<_, i64>(8)? as u64,
                rate_limit_count: row.get::<_, i64>(9)? as u64,
                total_retries: row.get::<_, i64>(10)? as u64,
                avg_latency_ms: row.get(11)?,
                current_status: serde_json::from_str(&row.get::<_, String>(12)?)
                    .unwrap_or(ModelStatus::Unknown),
                current_cooldown_seconds: row.get::<_, Option<i64>>(13)?.map(|v| v as u64),
                token_usage: serde_json::from_str(&row.get::<_, String>(14)?).unwrap_or_default(),
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_global_stats(&self) -> Result<GlobalStats> {
        let conn = self.conn.lock().unwrap();

        let total_requests: i64 =
            conn.query_row("SELECT COUNT(*) FROM model_requests", [], |r| r.get(0))?;
        let successful: i64 = conn.query_row(
            "SELECT COUNT(*) FROM model_requests WHERE status = 'Success'",
            [],
            |r| r.get(0),
        )?;
        let failed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM model_requests WHERE status = 'Failed'",
            [],
            |r| r.get(0),
        )?;
        let rate_limited: i64 = conn.query_row(
            "SELECT COUNT(*) FROM model_requests WHERE status = 'RateLimited'",
            [],
            |r| r.get(0),
        )?;

        let total_tokens: i64 = conn.query_row(
            "SELECT COALESCE(SUM(total_tokens), 0) FROM model_requests",
            [],
            |r| r.get(0),
        )?;

        let active_models: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT model_id) FROM model_requests WHERE timestamp > datetime('now', '-24 hours')", [], |r| r.get(0)
        )?;

        let most_used: Option<String> = conn.query_row(
            "SELECT model_id FROM model_requests GROUP BY model_id ORDER BY COUNT(*) DESC LIMIT 1", [], |r| r.get(0)
        ).optional()?;

        let fastest: Option<String> = conn.query_row(
            "SELECT model_id FROM model_requests WHERE status = 'Success' GROUP BY model_id ORDER BY AVG(latency_ms) ASC LIMIT 1", [], |r| r.get(0)
        ).optional()?;

        let error_rate = if total_requests > 0 {
            failed as f64 / total_requests as f64
        } else {
            0.0
        };

        Ok(GlobalStats {
            total_requests: total_requests as u64,
            successful_requests: successful as u64,
            failed_requests: failed as u64,
            rate_limit_events: rate_limited as u64,
            total_tokens: total_tokens as u64,
            active_models: active_models as u32,
            most_used_model: most_used,
            fastest_model: fastest,
            error_rate,
        })
    }
}
