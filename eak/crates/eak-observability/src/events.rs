//! OpenCode event types and parsing

use chrono::{DateTime, Utc};
use eak_telemetry::models::TokenUsage as TelemetryTokenUsage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OpenCodeEvent {
    #[serde(rename = "step_start")]
    StepStart(StepStartEvent),

    #[serde(rename = "text")]
    Text(TextEvent),

    #[serde(rename = "tool_use")]
    ToolUse(ToolUseEvent),

    #[serde(rename = "step_finish")]
    StepFinish(StepFinishEvent),

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStartEvent {
    pub timestamp: u64,
    pub session_id: String,
    pub part: StepStartPart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStartPart {
    pub id: String,
    pub message_id: String,
    pub session_id: String,
    pub snapshot: String,
    #[serde(rename = "type")]
    pub part_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEvent {
    pub timestamp: u64,
    pub session_id: String,
    pub part: TextPart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPart {
    pub id: String,
    pub message_id: String,
    pub session_id: String,
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: String,
    pub time: TimeRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseEvent {
    pub timestamp: u64,
    pub session_id: String,
    pub part: ToolUsePart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsePart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub tool: String,
    pub call_id: String,
    pub state: ToolState,
    pub id: String,
    pub session_id: String,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolState {
    pub status: String,
    pub input: serde_json::Value,
    pub output: Option<String>,
    pub metadata: Option<ToolMetadata>,
    pub title: String,
    pub time: TimeRange,
    pub exit: Option<i32>,
    pub truncated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub output: Option<String>,
    pub exit: Option<i32>,
    pub truncated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepFinishEvent {
    pub timestamp: u64,
    pub session_id: String,
    pub part: StepFinishPart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepFinishPart {
    pub id: String,
    pub reason: String,
    pub snapshot: String,
    pub message_id: String,
    pub session_id: String,
    #[serde(rename = "type")]
    pub part_type: String,
    pub tokens: Option<TelemetryTokenUsage>,
    pub cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheUsage {
    pub write: u64,
    pub read: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: u64,
    pub end: u64,
}

impl OpenCodeEvent {
    pub fn parse(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }

    pub fn session_id(&self) -> &str {
        match self {
            OpenCodeEvent::StepStart(e) => &e.session_id,
            OpenCodeEvent::Text(e) => &e.session_id,
            OpenCodeEvent::ToolUse(e) => &e.session_id,
            OpenCodeEvent::StepFinish(e) => &e.session_id,
            OpenCodeEvent::Unknown => "unknown",
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        let ts = match self {
            OpenCodeEvent::StepStart(e) => e.timestamp,
            OpenCodeEvent::Text(e) => e.timestamp,
            OpenCodeEvent::ToolUse(e) => e.timestamp,
            OpenCodeEvent::StepFinish(e) => e.timestamp,
            OpenCodeEvent::Unknown => 0,
        };
        DateTime::from_timestamp_millis(ts as i64).unwrap_or_else(Utc::now)
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            OpenCodeEvent::StepStart(_) => "step_start",
            OpenCodeEvent::Text(_) => "text",
            OpenCodeEvent::ToolUse(_) => "tool_use",
            OpenCodeEvent::StepFinish(_) => "step_finish",
            OpenCodeEvent::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub session_id: String,
    pub agent_name: String,
    pub model_id: String,
    pub event_type: AgentEventType,
    pub timestamp: DateTime<Utc>,
    pub task_description: Option<String>,
    pub activity: Option<String>,
    pub tokens: Option<TelemetryTokenUsage>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_output: Option<String>,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentEventType {
    TaskStarted,
    TaskProgress,
    ToolStarted,
    ToolCompleted,
    TaskCompleted,
    TaskFailed,
    Idle,
}

impl AgentEvent {
    pub fn from_opencode_event(
        event: OpenCodeEvent,
        agent_name: String,
        model_id: String,
    ) -> Option<Self> {
        let session_id = event.session_id().to_string();
        let timestamp = event.timestamp();

        match event {
            OpenCodeEvent::StepStart(_) => Some(AgentEvent {
                session_id,
                agent_name,
                model_id,
                event_type: AgentEventType::TaskStarted,
                timestamp,
                task_description: None,
                activity: Some("Starting task".to_string()),
                tokens: None,
                tool_name: None,
                tool_input: None,
                tool_output: None,
                latency_ms: None,
                error: None,
            }),
            OpenCodeEvent::Text(e) => Some(AgentEvent {
                session_id,
                agent_name,
                model_id,
                event_type: AgentEventType::TaskProgress,
                timestamp,
                task_description: None,
                activity: Some(extract_task_description(&e.part.text)),
                tokens: None,
                tool_name: None,
                tool_input: None,
                tool_output: None,
                latency_ms: Some(e.part.time.end.saturating_sub(e.part.time.start)),
                error: None,
            }),
            OpenCodeEvent::ToolUse(e) => {
                let tool_output = e.part.state.output.clone();
                Some(AgentEvent {
                    session_id,
                    agent_name,
                    model_id,
                    event_type: AgentEventType::ToolStarted,
                    timestamp,
                    task_description: None,
                    activity: Some(format!("Executing tool: {}", e.part.tool)),
                    tokens: None,
                    tool_name: Some(e.part.tool),
                    tool_input: Some(e.part.state.input),
                    tool_output: tool_output.clone(),
                    latency_ms: Some(
                        e.part
                            .state
                            .time
                            .end
                            .saturating_sub(e.part.state.time.start),
                    ),
                    error: if e.part.state.status == "error" {
                        tool_output
                    } else {
                        None
                    },
                })
            }
            OpenCodeEvent::StepFinish(e) => {
                let (event_type, activity) = match e.part.reason.as_str() {
                    "stop" => (AgentEventType::TaskCompleted, "Task completed"),
                    "tool-calls" => (AgentEventType::TaskProgress, "Tool calls completed"),
                    "error" => (AgentEventType::TaskFailed, "Task failed"),
                    _ => (AgentEventType::TaskProgress, "Processing"),
                };

                let is_failed = event_type == AgentEventType::TaskFailed;

                Some(AgentEvent {
                    session_id,
                    agent_name,
                    model_id,
                    event_type,
                    timestamp,
                    task_description: None,
                    activity: Some(activity.to_string()),
                    tokens: e.part.tokens,
                    tool_name: None,
                    tool_input: None,
                    tool_output: None,
                    latency_ms: None,
                    error: if is_failed { Some(e.part.reason) } else { None },
                })
            }
            OpenCodeEvent::Unknown => None,
        }
    }
}

fn extract_task_description(text: &str) -> String {
    // Extract first meaningful line as task description
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        "Processing".to_string()
    } else {
        lines[0].chars().take(100).collect::<String>()
    }
}
