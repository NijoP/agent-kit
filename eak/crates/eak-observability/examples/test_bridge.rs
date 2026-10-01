use eak_observability::ObservabilityBridge;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let bridge = Arc::new(ObservabilityBridge::new(None));

    println!("Starting EAK-Fast agent via observability bridge...");

    let session_id = bridge
        .start_agent(
            "test-terminal-1".to_string(),
            "eak-fast".to_string(),
            "nvidia/nvidia/nemotron-3.5-lightning-30b-a3b".to_string(),
            "Reply exactly: EAK-LIVE-TEST-PASS".to_string(),
        )
        .await?;

    println!("Started session: {}", session_id);

    // Process events in background
    let bridge_clone = Arc::clone(&bridge);
    tokio::spawn(async move {
        if let Err(e) = bridge_clone.process_events().await {
            eprintln!("Event processing error: {}", e);
        }
    });

    // Wait for completion
    for i in 0..30 {
        sleep(Duration::from_secs(1)).await;

        let state = bridge.state();
        if let Some(agent) = state.get_agent(&session_id, "eak-fast") {
            println!(
                "Agent status: {:?}, task: {:?}",
                agent.status,
                agent.current_task.as_ref().map(|t| &t.description)
            );
            if matches!(
                agent.status,
                eak_observability::state::ModelStatus::Completed
                    | eak_observability::state::ModelStatus::Failed
            ) {
                println!("Task completed!");
                break;
            }
        }
    }

    // Check final state
    let state = bridge.state();
    if let Some(agent) = state.get_agent(&session_id, "eak-fast") {
        println!("Final agent state:");
        println!("  Status: {:?}", agent.status);
        println!("  Total requests: {}", agent.total_requests);
        println!("  Successful: {}", agent.successful_requests);
        println!("  Failed: {}", agent.failed_requests);
        println!(
            "  Tokens: in={}, out={}, total={}",
            agent.total_tokens.total_input,
            agent.total_tokens.total_output,
            agent.total_tokens.total
        );
        if let Some(task) = &agent.current_task {
            println!("  Current task: {}", task.description);
            println!("  Task status: {:?}", task.status);
        }
        for task in &agent.task_history {
            println!("  History task: {} - {:?}", task.description, task.status);
        }
    }

    // Check telemetry database
    println!("\nChecking telemetry database...");
    let db = eak_telemetry::TelemetryDatabase::new(
        "/home/dev/electronics-agent-kit/data/telemetry/eak-telemetry.db",
    )?;
    let health = db.get_all_model_health()?;
    println!("Model health rows: {}", health.len());
    for h in health {
        println!(
            "  {}: requests={}, success={}, failed={}, tokens={}",
            h.model_id,
            h.total_requests,
            h.successful_requests,
            h.failed_requests,
            h.token_usage.total
        );
    }

    let requests = db.get_recent_requests(10)?;
    println!("Recent requests: {}", requests.len());
    for r in requests {
        println!(
            "  {} {} {} {:?} latency={}ms tokens={:?}",
            r.timestamp, r.agent, r.model_id, r.status, r.latency_ms, r.total_tokens
        );
    }

    bridge.stop_all().await;

    Ok(())
}
