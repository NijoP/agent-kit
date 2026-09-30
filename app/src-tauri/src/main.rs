//! eak-app — the Electronics Agent Kit desktop shell (Tauri v2).
//!
//! Thin bridge: the existing Rust kernel is the app's native backend, and every event it commits
//! is forwarded to the webview as an `eak://event` Tauri event through an [`EventSink`]. The whole
//! kernel↔UI integration is the `TauriEventSink` below — everything else is standard Tauri config
//! (generated locally; see ../../README.md). Build/run on a machine with the Tauri prerequisites.

use eak_ports::{EventRecord, EventSink};
use tauri::{AppHandle, Emitter};

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
fn start_run(app: AppHandle, intent: String) {
    std::thread::spawn(move || {
        use eak_cli::RunConfig;
        use eak_reasoning::FixtureEngine;

        // Build a deterministic fixture engine (no API key needed). In a full deployment this
        // would load a cassette; for now we use a single canned response so the pipeline has
        // a reasoning engine to drive its phases.
        let reasoning = Box::new(FixtureEngine::single(eak_reasoning::ReasoningResponse {
            candidates: vec![],
            part_candidates: vec![],
            explanations: vec![],
            clarifying_questions: vec![],
            raw: String::new(),
        }));

        // Build the run config from the supplied intent string.
        let cfg = RunConfig {
            intent,
            reasoning: eak_cli::ReasoningChoice::Fixture,
            cassette: None,
            log: std::path::PathBuf::from(std::env::temp_dir().join("eak_run_log.json")),
            model: String::new(),
            seed: 42,
            deterministic_clock: true,
        };

        // Run the full 15-phase workflow, streaming every EventRecord to the Tauri sink.
        let _ = eak_cli::run_with_sink(reasoning, &cfg, Some(Box::new(TauriEventSink { app })));
    });
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_run])
        .run(tauri::generate_context!())
        .expect("error while running eak-app");
}
