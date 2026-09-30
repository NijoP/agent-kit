//! eak-app — the Electronics Agent Kit desktop shell (Tauri v2).
//!
//! Thin bridge: the existing Rust kernel is the app's native backend, and every event it commits
//! is forwarded to the webview as an `eak://event` Tauri event through an [`EventSink`]. The whole
//! kernel↔UI integration is the `TauriEventSink` below — everything else is standard Tauri config
//! (generated locally; see ../../README.md). Build/run on a machine with the Tauri prerequisites.

use eak_ports::{EventRecord, EventSink, ReasoningEngine};
use tauri::{AppHandle, Emitter};
use eak_reasoning::FixtureEngine;
use eak_cli::{RunConfig, ReasoningChoice};
use std::path::PathBuf;

/// An [`EventSink`] that forwards every committed kernel event to the webview. It runs on the
/// kernel's worker thread; `AppHandle` is `Send + Sync`, so emitting across the thread boundary is
/// safe. `EventRecord` is `Serialize`, so it travels to the frontend as JSON unchanged.
struct TauriEventSink {
    app: AppHandle,
}

impl EventSink for TauriEventSink {
    fn on_committed(&mut self, record: &EventRecord) {
        // Fire-and-forget: a closed/slow webview must never block the kernel's commit path.
        let _ = self.app.emit("eak://event", record.clone());
    }
}

/// Kick off a real pipeline run on a background thread, streaming its events to the UI live.
#[tauri::command]
fn start_run(app: AppHandle, intent: String) -> Result<(), String> {
    std::thread::spawn(move || {
        let sink: Box<dyn EventSink> = Box::new(TauriEventSink { app });
        // Load the default cassette
        let cassette_path = match std::env::current_dir() {
            Ok(mut dir) => {
                dir.push("eak/crates/eak-cli/fixtures/default_cassette.json");
                dir
            }
            Err(e) => {
                eprintln!("Failed to get current directory: {e}");
                return;
            }
        };
        let reasoning: Box<dyn ReasoningEngine> = match eak_reasoning::FixtureEngine::load(&cassette_path) {
            Ok(engine) => Box::new(engine),
            Err(e) => {
                eprintln!("Failed to load cassette: {e}");
                return;
            }
        };
        // Build config
        let log_path = std::env::temp_dir().join("eak-run.log");
        let cfg = RunConfig {
            intent,
            reasoning: ReasoningChoice::Fixture,
            cassette: Some(cassette_path),
            log: log_path,
            model: "claude-opus-4-8".to_string(),
            seed: 1,
            deterministic_clock: true,
        };
        // Run with sink
        if let Err(e) = eak_cli::run_with_sink(reasoning, &cfg, Some(sink)) {
            eprintln!("Run failed: {e}");
        }
    });
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_run])
        .run(tauri::generate_context!())
        .expect("error while running eak-app");
}
