use eak_telemetry::TelemetryDatabase;
use std::path::Path;

fn main() {
    let db_path = Path::new("/home/dev/electronics-agent-kit/data/telemetry/eak-telemetry.db");
    println!("Opening database: {:?}", db_path);

    let db = TelemetryDatabase::new(db_path).expect("Failed to open DB");

    // Check model health
    let health = db
        .get_all_model_health()
        .expect("Failed to get model health");
    println!("\n=== Model Health ({} rows) ===", health.len());
    for h in &health {
        println!("  model_id={}, agent={}, role={}, total_requests={}, successful={}, failed={}, rate_limits={}, retries={}, avg_latency={}, status={:?}, cooldown={:?}, last_req={:?}, last_success={:?}, last_error={:?}, tokens={:?}",
            h.model_id, h.agent, h.role, h.total_requests, h.successful_requests, h.failed_requests,
            h.rate_limit_count, h.total_retries, h.avg_latency_ms, h.current_status,
            h.current_cooldown_seconds, h.last_request, h.last_success, h.last_error, h.token_usage
        );
    }

    // Check provider states
    let providers = ["nvidia", "deepseek-ai"];
    println!("\n=== Provider States ===");
    for p in &providers {
        let state = db
            .get_provider_state(p)
            .expect("Failed to get provider state");
        println!("  provider={}: {:?}", p, state);
    }

    // Check global stats
    let stats = db.get_global_stats().expect("Failed to get global stats");
    println!("\n=== Global Stats ===");
    println!("  total_requests={}, successful={}, failed={}, rate_limits={}, total_tokens={}, active_models={}, most_used={:?}, fastest={:?}, error_rate={}",
        stats.total_requests, stats.successful_requests, stats.failed_requests,
        stats.rate_limit_events, stats.total_tokens, stats.active_models,
        stats.most_used_model, stats.fastest_model, stats.error_rate
    );

    // Check recent requests
    let requests = db
        .get_recent_requests(20)
        .expect("Failed to get recent requests");
    println!("\n=== Recent Requests ({} rows) ===", requests.len());
    for r in &requests {
        println!("  id={}, ts={}, terminal={}, session={}, agent={}, model={}, provider={}, type={:?}, status={:?}, http={}, latency={}, retries={}, in_tok={:?}, out_tok={:?}, total_tok={:?}, err_cat={:?}, err_msg={:?}, rate_limited={}",
            r.id, r.timestamp, r.terminal_id, r.session_id, r.agent, r.model_id, r.provider,
            r.request_type, r.status, r.http_status.unwrap_or(0), r.latency_ms, r.retry_count,
            r.input_tokens, r.output_tokens, r.total_tokens, r.error_category, r.error_message, r.rate_limited
        );
    }

    // Check sessions
    let requests_all = db
        .get_recent_requests(500)
        .expect("Failed to get all requests");
    let mut sessions = std::collections::HashMap::new();
    for r in &requests_all {
        let e = sessions.entry(r.session_id.clone()).or_insert((
            r.terminal_id.clone(),
            r.timestamp,
            0u64,
            0u64,
            0u64,
            0u64,
            0u64,
            0u64,
            Vec::new(),
            Vec::new(),
            r.timestamp,
        ));
        e.2 += 1;
        if r.status == eak_telemetry::models::RequestStatus::Success {
            e.3 += 1;
        } else {
            e.4 += 1;
        }
        if let Some(t) = r.input_tokens {
            e.5 += t as u64;
        }
        if let Some(t) = r.output_tokens {
            e.6 += t as u64;
        }
        if let Some(t) = r.total_tokens {
            e.7 += t as u64;
        }
        if !e.8.contains(&r.model_id) {
            e.8.push(r.model_id.clone());
        }
        if !e.9.contains(&r.agent) {
            e.9.push(r.agent.clone());
        }
        e.10 = r.timestamp;
    }

    println!("\n=== Sessions ({} rows) ===", sessions.len());
    for (sid, (tid, start, total, succ, fail, in_tok, out_tok, tot_tok, models, agents, end)) in
        sessions
    {
        println!("  session={}, terminal={}, start={}, end={}, total={}, succ={}, fail={}, in={}, out={}, tot={}, models={:?}, agents={:?}",
            sid, tid, start, end, total, succ, fail, in_tok, out_tok, tot_tok, models, agents);
    }
}
