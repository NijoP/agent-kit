//! WebSocket server for real-time dashboard updates

use crate::state::{GlobalStats, ModelStatus, ObservabilityState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use chrono::Utc;
use futures::{sink::SinkExt, stream::StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DashboardMessage {
    #[serde(rename = "agent_update")]
    AgentUpdate(AgentUpdate),

    #[serde(rename = "session_update")]
    SessionUpdate(SessionUpdate),

    #[serde(rename = "global_stats")]
    GlobalStats(GlobalStats),

    #[serde(rename = "model_health")]
    ModelHealth(Vec<ModelHealthInfo>),

    #[serde(rename = "task_update")]
    TaskUpdate(TaskUpdate),

    #[serde(rename = "rate_limit")]
    RateLimit(RateLimitEvent),

    #[serde(rename = "heartbeat")]
    Heartbeat,

    #[serde(rename = "error")]
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUpdate {
    pub session_id: String,
    pub terminal_id: String,
    pub agent_name: String,
    pub model_id: String,
    pub status: ModelStatus,
    pub current_task: Option<TaskInfo>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limit_count: u64,
    pub total_retries: u64,
    pub avg_latency_ms: f64,
    pub total_tokens: TokenUsageInfo,
    pub current_cooldown: Option<u64>,
    pub last_activity: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_id: String,
    pub description: String,
    pub status: ModelStatus,
    pub started_at: String,
    pub elapsed_seconds: u64,
    pub current_activity: String,
    pub tools_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUpdate {
    pub session_id: String,
    pub terminal_id: String,
    pub started_at: String,
    pub last_activity: String,
    pub agents: Vec<String>,
    pub root_task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub session_id: String,
    pub agent_name: String,
    pub task_id: String,
    pub description: String,
    pub status: ModelStatus,
    pub started_at: String,
    pub elapsed_seconds: u64,
    pub current_activity: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEvent {
    pub session_id: String,
    pub agent_name: String,
    pub model_id: String,
    pub timestamp: String,
    pub retry_count: u32,
    pub cooldown_seconds: u64,
    pub retry_after: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealthInfo {
    pub model_id: String,
    pub agent: String,
    pub role: String,
    pub status: ModelStatus,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limit_count: u64,
    pub avg_latency_ms: f64,
    pub current_cooldown: Option<u64>,
    pub last_request: Option<String>,
    pub last_success: Option<String>,
    pub last_error: Option<String>,
    pub tokens: TokenUsageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageInfo {
    pub total_input: u64,
    pub total_output: u64,
    pub total: u64,
    pub provider_reported: bool,
}

#[allow(dead_code)]
pub struct WebSocketServer {
    state: Arc<ObservabilityState>,
    tx: broadcast::Sender<DashboardMessage>,
    rx: Arc<RwLock<Option<broadcast::Receiver<DashboardMessage>>>>,
}

impl WebSocketServer {
    pub fn new(state: Arc<ObservabilityState>) -> Self {
        let (tx, _rx) = broadcast::channel(1000);
        Self {
            state,
            tx,
            rx: Arc::new(RwLock::new(None)),
        }
    }

    pub fn broadcaster(&self) -> broadcast::Sender<DashboardMessage> {
        self.tx.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DashboardMessage> {
        self.tx.subscribe()
    }

    pub fn handle_ws(
        &self,
        ws: WebSocketUpgrade,
        State(_state): State<Arc<ObservabilityState>>,
    ) -> Response {
        let rx = self.subscribe();
        ws.on_upgrade(move |socket| handle_socket(socket, rx))
    }

    pub fn broadcast_agent_update(&self, session_id: &str, agent_name: &str) {
        if let Some(agent) = self.state.get_agent(session_id, agent_name) {
            let session = self.state.get_session(session_id);
            let terminal_id = session
                .as_ref()
                .map(|s| s.terminal_id.clone())
                .unwrap_or_default();

            let msg = DashboardMessage::AgentUpdate(AgentUpdate {
                session_id: session_id.to_string(),
                terminal_id,
                agent_name: agent.agent_name.clone(),
                model_id: agent.model_id.clone(),
                status: agent.status.clone(),
                current_task: agent.current_task.as_ref().map(|t| TaskInfo {
                    task_id: t.task_id.clone(),
                    description: t.description.clone(),
                    status: t.status.clone(),
                    started_at: t.started_at.to_rfc3339(),
                    elapsed_seconds: Utc::now()
                        .signed_duration_since(t.started_at)
                        .num_seconds()
                        .max(0) as u64,
                    current_activity: t.current_activity.clone(),
                    tools_used: t.tools_used.len() as u32,
                }),
                total_requests: agent.total_requests,
                successful_requests: agent.successful_requests,
                failed_requests: agent.failed_requests,
                rate_limit_count: agent.rate_limit_count,
                total_retries: agent.total_retries,
                avg_latency_ms: agent.avg_latency_ms,
                total_tokens: TokenUsageInfo {
                    total_input: agent.total_tokens.total_input,
                    total_output: agent.total_tokens.total_output,
                    total: agent.total_tokens.total,
                    provider_reported: agent.total_tokens.provider_reported,
                },
                current_cooldown: agent.current_cooldown,
                last_activity: agent.last_activity.map(|d| d.to_rfc3339()),
                last_error: agent.last_error.clone(),
            });

            let _ = self.tx.send(msg);
        }
    }

    pub fn broadcast_session_update(&self, session_id: &str) {
        if let Some(session) = self.state.get_session(session_id) {
            let agents: Vec<String> = session.agents.keys().cloned().collect();

            let msg = DashboardMessage::SessionUpdate(SessionUpdate {
                session_id: session.session_id.clone(),
                terminal_id: session.terminal_id.clone(),
                started_at: session.started_at.to_rfc3339(),
                last_activity: session.last_activity.to_rfc3339(),
                agents,
                root_task_id: session.root_task_id.clone(),
            });

            let _ = self.tx.send(msg);
        }
    }

    pub fn broadcast_global_stats(&self) {
        let stats = self.state.get_global_stats();
        let msg = DashboardMessage::GlobalStats(stats);
        let _ = self.tx.send(msg);
    }

    pub fn broadcast_model_health(&self) {
        let agents = self.state.get_all_agents();
        let health: Vec<ModelHealthInfo> = agents
            .into_iter()
            .map(|a| ModelHealthInfo {
                model_id: a.model_id.clone(),
                agent: a.agent_name.clone(),
                role: get_role_for_agent(&a.agent_name),
                status: a.status.clone(),
                total_requests: a.total_requests,
                successful_requests: a.successful_requests,
                failed_requests: a.failed_requests,
                rate_limit_count: a.rate_limit_count,
                avg_latency_ms: a.avg_latency_ms,
                current_cooldown: a.current_cooldown,
                last_request: a.last_activity.map(|d| d.to_rfc3339()),
                last_success: None, // Would need to track separately
                last_error: a.last_error.clone(),
                tokens: TokenUsageInfo {
                    total_input: a.total_tokens.total_input,
                    total_output: a.total_tokens.total_output,
                    total: a.total_tokens.total,
                    provider_reported: a.total_tokens.provider_reported,
                },
            })
            .collect();

        let msg = DashboardMessage::ModelHealth(health);
        let _ = self.tx.send(msg);
    }

    pub fn broadcast_task_update(&self, session_id: &str, agent_name: &str) {
        if let Some(agent) = self.state.get_agent(session_id, agent_name) {
            if let Some(task) = &agent.current_task {
                let msg = DashboardMessage::TaskUpdate(TaskUpdate {
                    session_id: session_id.to_string(),
                    agent_name: agent_name.to_string(),
                    task_id: task.task_id.clone(),
                    description: task.description.clone(),
                    status: task.status.clone(),
                    started_at: task.started_at.to_rfc3339(),
                    elapsed_seconds: Utc::now()
                        .signed_duration_since(task.started_at)
                        .num_seconds()
                        .max(0) as u64,
                    current_activity: task.current_activity.clone(),
                    completed_at: task.completed_at.map(|d| d.to_rfc3339()),
                    error: task.error.clone(),
                });
                let _ = self.tx.send(msg);
            }
        }
    }

    pub fn broadcast_rate_limit(
        &self,
        session_id: &str,
        agent_name: &str,
        model_id: &str,
        retry_count: u32,
        cooldown: u64,
        retry_after: Option<u64>,
    ) {
        let msg = DashboardMessage::RateLimit(RateLimitEvent {
            session_id: session_id.to_string(),
            agent_name: agent_name.to_string(),
            model_id: model_id.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            retry_count,
            cooldown_seconds: cooldown,
            retry_after,
        });
        let _ = self.tx.send(msg);
    }

    pub fn start_heartbeat(&self) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                let _ = tx.send(DashboardMessage::Heartbeat);
            }
        });
    }
}

async fn handle_socket(socket: WebSocket, mut rx: broadcast::Receiver<DashboardMessage>) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial state
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&DashboardMessage::Heartbeat).unwrap(),
        ))
        .await;

    // Forward broadcast messages to WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages (ping/pong, subscriptions)
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if text == "ping" {
                    // Could send pong back
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

fn get_role_for_agent(agent: &str) -> String {
    match agent {
        "eak-master" => "Orchestrator".to_string(),
        "eak-architect" => "Architect".to_string(),
        "eak-engineer" => "Engineer".to_string(),
        "eak-reviewer" => "Reviewer".to_string(),
        "eak-fast" => "Fast/Inspector".to_string(),
        _ => "Unknown".to_string(),
    }
}
