//! Live state tracking for agents and models

use chrono::{DateTime, Duration, Utc};
use eak_telemetry::models::{
    ModelHealth, ModelStatus as TelemetryModelStatus, ProviderState, ProviderStatus, TokenUsage,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelStatus {
    Idle,
    Working,
    ToolExecution,
    Waiting,
    Retrying,
    RateLimited,
    Cooldown,
    Completed,
    Failed,
    Unknown,
}

impl From<TelemetryModelStatus> for ModelStatus {
    fn from(status: TelemetryModelStatus) -> Self {
        match status {
            TelemetryModelStatus::Healthy => ModelStatus::Idle,
            TelemetryModelStatus::Degraded => ModelStatus::Working,
            TelemetryModelStatus::RateLimited => ModelStatus::RateLimited,
            TelemetryModelStatus::Offline => ModelStatus::Failed,
            TelemetryModelStatus::Unknown => ModelStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub task_id: String,
    pub description: String,
    pub agent_name: String,
    pub model_id: String,
    pub session_id: String,
    pub status: ModelStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub current_activity: String,
    pub tokens_used: TokenUsage,
    pub tools_used: Vec<ToolUsage>,
    pub error: Option<String>,
    pub parent_task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsage {
    pub tool_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub input: serde_json::Value,
    pub output: Option<String>,
    pub success: bool,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub agent_name: String,
    pub model_id: String,
    pub session_id: Option<String>,
    pub status: ModelStatus,
    pub current_task: Option<TaskState>,
    pub task_history: Vec<TaskState>,
    pub last_activity: Option<DateTime<Utc>>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limit_count: u64,
    pub total_retries: u64,
    pub total_tokens: TokenUsage,
    pub avg_latency_ms: f64,
    pub current_cooldown: Option<u64>,
    pub last_error: Option<String>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            agent_name: String::new(),
            model_id: String::new(),
            session_id: None,
            status: ModelStatus::Unknown,
            current_task: None,
            task_history: Vec::new(),
            last_activity: None,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            rate_limit_count: 0,
            total_retries: 0,
            total_tokens: TokenUsage::default(),
            avg_latency_ms: 0.0,
            current_cooldown: None,
            last_error: None,
        }
    }
}

impl AgentState {
    pub fn new(agent_name: String, model_id: String) -> Self {
        Self {
            agent_name,
            model_id,
            ..Default::default()
        }
    }

    pub fn elapsed_time(&self) -> Option<Duration> {
        self.current_task
            .as_ref()
            .map(|task| Utc::now().signed_duration_since(task.started_at))
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            ModelStatus::Working
                | ModelStatus::ToolExecution
                | ModelStatus::Waiting
                | ModelStatus::Retrying
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub terminal_id: String,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub agents: HashMap<String, AgentState>,
    pub root_task_id: Option<String>,
}

impl SessionState {
    pub fn new(session_id: String, terminal_id: String) -> Self {
        Self {
            session_id,
            terminal_id,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            agents: HashMap::new(),
            root_task_id: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ObservabilityState {
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    agent_model_map: Arc<RwLock<HashMap<String, (String, String)>>>, // session_id -> (agent_name, model_id)
    // Dynamic mapping from OpenCode sessionID -> bridge session info
    opencode_session_map: Arc<RwLock<HashMap<String, OpencodeSessionInfo>>>,
    global_stats: Arc<RwLock<GlobalStats>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpencodeSessionInfo {
    pub bridge_session_id: String,
    pub terminal_id: String,
    pub agent_name: String,
    pub model_id: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalStats {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_rate_limits: u64,
    pub total_tokens: u64,
    pub active_agents: u32,
}

impl ObservabilityState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create_session(&self, session_id: &str, terminal_id: &str) -> SessionState {
        let mut sessions = self.sessions.write();
        sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionState::new(session_id.to_string(), terminal_id.to_string()))
            .clone()
    }

    pub fn update_session(&self, session: SessionState) {
        let mut sessions = self.sessions.write();
        sessions.insert(session.session_id.clone(), session);
    }

    pub fn get_session(&self, session_id: &str) -> Option<SessionState> {
        let sessions = self.sessions.read();
        sessions.get(session_id).cloned()
    }

    pub fn get_all_sessions(&self) -> Vec<SessionState> {
        let sessions = self.sessions.read();
        sessions.values().cloned().collect()
    }

    pub fn register_agent_model(&self, session_id: &str, agent_name: &str, model_id: &str) {
        let mut map = self.agent_model_map.write();
        map.insert(
            session_id.to_string(),
            (agent_name.to_string(), model_id.to_string()),
        );
    }

    pub fn get_agent_model(&self, session_id: &str) -> Option<(String, String)> {
        let map = self.agent_model_map.read();
        map.get(session_id).cloned()
    }

    // Dynamic OpenCode sessionID mapping methods
    pub fn register_opencode_session(&self, opencode_session_id: &str, info: OpencodeSessionInfo) {
        let mut map = self.opencode_session_map.write();
        map.insert(opencode_session_id.to_string(), info);
    }

    pub fn get_opencode_session(&self, opencode_session_id: &str) -> Option<OpencodeSessionInfo> {
        let map = self.opencode_session_map.read();
        map.get(opencode_session_id).cloned()
    }

    pub fn resolve_session_id(&self, opencode_session_id: &str) -> Option<String> {
        let map = self.opencode_session_map.read();
        map.get(opencode_session_id)
            .map(|info| info.bridge_session_id.clone())
    }

    pub fn update_global_stats(&self, f: impl FnOnce(&mut GlobalStats)) {
        let mut stats = self.global_stats.write();
        f(&mut stats);
    }

    pub fn get_global_stats(&self) -> GlobalStats {
        let stats = self.global_stats.read();
        stats.clone()
    }

    pub fn get_all_agents(&self) -> Vec<AgentState> {
        let sessions = self.sessions.read();
        sessions
            .values()
            .flat_map(|s| s.agents.values().cloned())
            .collect()
    }

    pub fn get_agent(&self, session_id: &str, agent_name: &str) -> Option<AgentState> {
        let sessions = self.sessions.read();
        sessions
            .get(session_id)
            .and_then(|s| s.agents.get(agent_name).cloned())
    }

    pub fn update_agent(&self, session_id: &str, agent: AgentState) {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            session.agents.insert(agent.agent_name.clone(), agent);
            session.last_activity = Utc::now();
        }
    }
}
