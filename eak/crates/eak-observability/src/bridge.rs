//! Observability Bridge - Spawns OpenCode processes and consumes JSON events

use crate::events::{AgentEvent, AgentEventType, OpenCodeEvent};
use crate::state::{
    AgentState, ModelStatus, ObservabilityState, OpencodeSessionInfo, SessionState, TaskState,
    ToolUsage,
};
use crate::{ObservabilityError, Result};
use chrono::Utc;
use eak_telemetry::{
    models::{RequestType, TokenUsage as TelemetryTokenUsage},
    TelemetryRecorder,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct ObservabilityBridge {
    state: Arc<ObservabilityState>,
    recorder: Option<Arc<TelemetryRecorder>>,
    processes: Arc<RwLock<HashMap<String, ProcessHandle>>>,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    event_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<AgentEvent>>>>,
}

#[allow(dead_code)]
struct ProcessHandle {
    session_id: String,
    terminal_id: String,
    agent_name: String,
    model_id: String,
    handle: JoinHandle<()>,
    cancel_tx: mpsc::UnboundedSender<()>,
}

#[derive(Debug, Clone)]
struct RunOpenCodeParams {
    session_id: String,
    terminal_id: String,
    agent_name: String,
    model_id: String,
    prompt: String,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    state: Arc<ObservabilityState>,
    recorder: Option<Arc<TelemetryRecorder>>,
    cancel_rx: mpsc::UnboundedReceiver<()>,
}

impl ObservabilityBridge {
    pub fn new(recorder: Option<Arc<TelemetryRecorder>>) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            state: Arc::new(ObservabilityState::new()),
            recorder,
            processes: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
        }
    }

    pub fn state(&self) -> Arc<ObservabilityState> {
        self.state.clone()
    }

    pub async fn start_agent(
        &self,
        terminal_id: String,
        agent_name: String,
        model_id: String,
        prompt: String,
    ) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();

        // Register agent-model mapping
        self.state
            .register_agent_model(&session_id, &agent_name, &model_id);

        // Create initial session state
        let mut session = SessionState::new(session_id.clone(), terminal_id.clone());
        let agent_state = AgentState::new(agent_name.clone(), model_id.clone());
        session.agents.insert(agent_name.clone(), agent_state);
        self.state.update_session(session);

        // Spawn OpenCode process
        let (cancel_tx, mut cancel_rx) = mpsc::unbounded_channel();
        let event_tx = self.event_tx.clone();
        let state = self.state.clone();
        let recorder = self.recorder.clone();

        // Clone values for the spawned task
        let session_id_for_task = session_id.clone();
        let terminal_id_for_task = terminal_id.clone();
        let agent_name_for_task = agent_name.clone();
        let agent_name_for_error = agent_name.clone();
        let model_id_for_task = model_id.clone();
        let prompt_for_task = prompt.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = run_opencode_process(
                session_id_for_task,
                terminal_id_for_task,
                agent_name_for_task,
                model_id_for_task,
                prompt_for_task,
                event_tx,
                state,
                recorder,
                &mut cancel_rx,
            )
            .await
            {
                error!("OpenCode process error for {}: {}", agent_name_for_error, e);
            }
        });

        let process_handle = ProcessHandle {
            session_id: session_id.clone(),
            terminal_id,
            agent_name: agent_name.clone(),
            model_id,
            handle,
            cancel_tx,
        };

        self.processes
            .write()
            .insert(session_id.clone(), process_handle);

        Ok(session_id)
    }

    pub async fn stop_agent(&self, session_id: &str) -> Result<()> {
        let mut processes = self.processes.write();
        if let Some(handle) = processes.remove(session_id) {
            let _ = handle.cancel_tx.send(());
            handle.handle.abort();
        }
        Ok(())
    }

    pub async fn stop_all(&self) {
        let mut processes = self.processes.write();
        for (_, handle) in processes.drain() {
            let _ = handle.cancel_tx.send(());
            handle.handle.abort();
        }
    }

    pub async fn process_events(&self) -> Result<()> {
        let rx = self.event_rx.write().take();
        let Some(mut rx) = rx else {
            return Err(ObservabilityError::Process(
                "Event receiver already taken".into(),
            ));
        };

        while let Some(event) = rx.recv().await {
            self.handle_agent_event(event).await?;
        }

        Ok(())
    }

    async fn handle_agent_event(&self, event: AgentEvent) -> Result<()> {
        let session_id = &event.session_id;
        let agent_name = &event.agent_name;

        // Register agent/model mapping for this OpenCode sessionID (for dashboard queries)
        self.state
            .register_agent_model(session_id, agent_name, &event.model_id);

        // Get or create session
        let mut session = self.state.get_session(session_id).unwrap_or_else(|| {
            let terminal_id = format!("terminal-{}", &session_id[..8]);
            SessionState::new(session_id.clone(), terminal_id)
        });

        // Get or create agent state
        let mut agent = session
            .agents
            .get(agent_name)
            .cloned()
            .unwrap_or_else(|| AgentState::new(agent_name.clone(), event.model_id.clone()));

        // Update agent session ID
        agent.session_id = Some(session_id.clone());

        // Process event based on type
        match event.event_type {
            AgentEventType::TaskStarted => {
                let task_id = Uuid::new_v4().to_string();
                let task = TaskState {
                    task_id: task_id.clone(),
                    description: event
                        .task_description
                        .unwrap_or_else(|| "New task".to_string()),
                    agent_name: agent_name.clone(),
                    model_id: event.model_id.clone(),
                    session_id: session_id.clone(),
                    status: ModelStatus::Working,
                    started_at: event.timestamp,
                    updated_at: event.timestamp,
                    completed_at: None,
                    current_activity: event.activity.unwrap_or_else(|| "Starting".to_string()),
                    tokens_used: TelemetryTokenUsage::default(),
                    tools_used: Vec::new(),
                    error: None,
                    parent_task_id: None,
                };

                agent.current_task = Some(task);
                agent.status = ModelStatus::Working;
                agent.total_requests += 1;
            }

            AgentEventType::TaskProgress => {
                if let Some(task) = &mut agent.current_task {
                    task.status = ModelStatus::Working;
                    task.updated_at = event.timestamp;
                    task.current_activity = event.activity.unwrap_or_else(|| "Working".to_string());
                }
                agent.status = ModelStatus::Working;
            }

            AgentEventType::ToolStarted => {
                agent.status = ModelStatus::ToolExecution;
                if let Some(task) = &mut agent.current_task {
                    task.status = ModelStatus::ToolExecution;
                    task.current_activity = event
                        .activity
                        .unwrap_or_else(|| "Executing tool".to_string());

                    let tool = ToolUsage {
                        tool_name: event
                            .tool_name
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string()),
                        started_at: event.timestamp,
                        completed_at: None,
                        input: event.tool_input.clone().unwrap_or_default(),
                        output: None,
                        success: true,
                        latency_ms: 0,
                    };
                    task.tools_used.push(tool);
                }
            }

            AgentEventType::ToolCompleted => {
                if let Some(task) = &mut agent.current_task {
                    if let Some(tool) = task.tools_used.last_mut() {
                        tool.completed_at = Some(event.timestamp);
                        tool.output = event.tool_output.clone();
                        tool.success = event.error.is_none();
                        tool.latency_ms = event.latency_ms.unwrap_or(0);
                    }
                    task.status = ModelStatus::Working;
                    task.current_activity = event
                        .activity
                        .unwrap_or_else(|| "Tool completed".to_string());
                }
                agent.status = ModelStatus::Working;
            }

            AgentEventType::TaskCompleted => {
                agent.status = ModelStatus::Completed;
                agent.successful_requests += 1;
                agent.last_activity = Some(event.timestamp);

                if let Some(mut task) = agent.current_task.take() {
                    task.status = ModelStatus::Completed;
                    task.completed_at = Some(event.timestamp);
                    task.updated_at = event.timestamp;
                    if let Some(ref tokens) = event.tokens {
                        task.tokens_used = tokens.clone();
                        agent.total_tokens.total_input += tokens.total_input;
                        agent.total_tokens.total_output += tokens.total_output;
                        agent.total_tokens.total += tokens.total;
                    }
                    agent.task_history.push(task);
                }
            }

            AgentEventType::TaskFailed => {
                agent.status = ModelStatus::Failed;
                agent.failed_requests += 1;
                agent.last_error = event.error.clone();
                agent.last_activity = Some(event.timestamp);

                if let Some(mut task) = agent.current_task.take() {
                    task.status = ModelStatus::Failed;
                    task.completed_at = Some(event.timestamp);
                    task.updated_at = event.timestamp;
                    task.error = event.error.clone();
                    agent.task_history.push(task);
                }
            }

            AgentEventType::Idle => {
                agent.status = ModelStatus::Idle;
            }
        }

        // Update latency
        if let Some(latency) = event.latency_ms {
            let total_latency = agent.avg_latency_ms * agent.total_requests as f64 + latency as f64;
            agent.avg_latency_ms = total_latency / (agent.total_requests + 1) as f64;
        }

        // Update tokens
        if let Some(ref tokens) = event.tokens {
            agent.total_tokens.total_input += tokens.total_input;
            agent.total_tokens.total_output += tokens.total_output;
            agent.total_tokens.total += tokens.total;
        }

        // Update session
        session.agents.insert(agent_name.clone(), agent.clone());
        session.last_activity = Utc::now();
        self.state.update_session(session);

        // Record to telemetry if recorder available
        if let Some(recorder) = &self.recorder {
            let _ = recorder.record_request(
                agent_name,
                &event.model_id,
                "nvidia", // provider
                RequestType::Completion,
                || -> std::result::Result<(), std::io::Error> {
                    // Simulate the operation for telemetry recording
                    Ok(())
                },
            );
        }

        // Update global stats
        self.state.update_global_stats(|stats| {
            stats.total_requests += 1;
            if event.event_type == AgentEventType::TaskCompleted {
                stats.successful_requests += 1;
            } else if event.event_type == AgentEventType::TaskFailed {
                stats.failed_requests += 1;
            }
            stats.active_agents = self
                .state
                .get_all_agents()
                .iter()
                .filter(|a| a.is_active())
                .count() as u32;
        });

        Ok(())
    }
}

async fn run_opencode_process(
    params: RunOpenCodeParams,
) -> Result<()> {
    // Build OpenCode command
    let mut cmd = Command::new("opencode");
    cmd.arg("run")
        .arg("--format")
        .arg("json")
        .arg("--agent")
        .arg(&agent_name)
        .arg("--model")
        .arg(&model_id)
        .arg(&prompt)
        .current_dir("/home/dev/electronics-agent-kit")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    info!(
        "Starting OpenCode process for {} (session: {})",
        agent_name, session_id
    );

    let mut child = cmd
        .spawn()
        .map_err(|e| ObservabilityError::Process(format!("Failed to spawn OpenCode: {}", e)))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ObservabilityError::Process("Failed to capture stdout".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| ObservabilityError::Process("Failed to capture stderr".into()))?;

    let event_tx_stdout = event_tx.clone();
    let event_tx_stderr = event_tx.clone();
    let session_id_stdout = session_id.clone();
    let session_id_stderr = session_id.clone();
    let agent_name_stdout = agent_name.clone();
    let agent_name_stderr = agent_name.clone();
    let model_id_stdout = model_id.clone();
    let model_id_stderr = model_id.clone();

    // Process stdout (JSON events)
    let stdout_handle = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut line_count = 0;

        while let Ok(Some(line)) = lines.next_line().await {
            line_count += 1;
            if line_count <= 5 || line_count % 10 == 0 {
                info!("OpenCode stdout line {}: {}", line_count, line);
            }
            if line.trim().is_empty() {
                continue;
            }

            // Parse OpenCode event
            if let Ok(opencode_event) = OpenCodeEvent::parse(&line) {
                info!(
                    "Parsed OpenCode event: type={}, session_id={}",
                    opencode_event.event_type(),
                    opencode_event.session_id()
                );
                // Extract OpenCode sessionID from the event
                let opencode_session_id = opencode_event.session_id().to_string();

                // Try to resolve the bridge session ID from OpenCode sessionID
                let bridge_session_id = state.resolve_session_id(&opencode_session_id);

                // If this is a new OpenCode sessionID, register the mapping
                if bridge_session_id.is_none() {
                    // Register the mapping using the bridge's session info
                    let info = OpencodeSessionInfo {
                        bridge_session_id: session_id_stdout.clone(),
                        terminal_id: terminal_id.clone(),
                        agent_name: agent_name_stdout.clone(),
                        model_id: model_id_stdout.clone(),
                        registered_at: Utc::now(),
                    };
                    state.register_opencode_session(&opencode_session_id, info);
                    info!(
                        "Registered new OpenCode session mapping: {} -> {}",
                        opencode_session_id, session_id_stdout
                    );
                }

                // Resolve the bridge session ID (should now exist)
                let bridge_session_id = state
                    .resolve_session_id(&opencode_session_id)
                    .unwrap_or_else(|| session_id_stdout.clone());

                // Determine agent and model from session mapping
                let (mapped_agent, mapped_model) = state
                    .get_agent_model(&bridge_session_id)
                    .unwrap_or((agent_name_stdout.clone(), model_id_stdout.clone()));

                if let Some(agent_event) =
                    AgentEvent::from_opencode_event(opencode_event, mapped_agent, mapped_model)
                {
                    info!(
                        "Sending agent event: type={:?}, session_id={}",
                        agent_event.event_type, agent_event.session_id
                    );
                    let _ = event_tx_stdout.send(agent_event);
                }
            } else {
                warn!("Failed to parse OpenCode event: {}", line);
            }
        }
        info!(
            "OpenCode stdout reader finished, total lines: {}",
            line_count
        );
    });

    // Process stderr (for errors)
    let stderr_handle = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.contains("429")
                || line.contains("rate limit")
                || line.contains("Too Many Requests")
            {
                warn!("Rate limit detected in stderr: {}", line);
                let agent_event = AgentEvent {
                    session_id: session_id_stderr.clone(),
                    agent_name: agent_name_stderr.clone(),
                    model_id: model_id_stderr.clone(),
                    event_type: AgentEventType::TaskFailed,
                    timestamp: Utc::now(),
                    task_description: None,
                    activity: Some("Rate limited".to_string()),
                    tokens: None,
                    tool_name: None,
                    tool_input: None,
                    tool_output: None,
                    latency_ms: None,
                    error: Some(line),
                };
                let _ = event_tx_stderr.send(agent_event);
            }
        }
    });

    // Wait for process or cancellation
    tokio::select! {
        _ = cancel_rx.recv() => {
            info!("Cancellation requested for {}", agent_name);
            let _ = child.kill().await;
        }
        result = child.wait() => {
            match result {
                Ok(status) => {
                    info!("OpenCode process for {} exited with: {}", agent_name, status);
                }
                Err(e) => {
                    error!("OpenCode process for {} error: {}", agent_name, e);
                }
            }
        }
    }

    // Cleanup
    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    Ok(())
}
