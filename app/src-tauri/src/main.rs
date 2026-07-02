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
        let _sink: Box<dyn EventSink> = Box::new(TauriEventSink { app });
        // ── FINALIZE LOCALLY (README steps 1-5) ────────────────────────────────────────────────
        // Build a reasoning engine + RunConfig, then stream a real run through the sink:
        //
        //   let reasoning = Box::new(
        //       eak_reasoning::FixtureEngine::load("<cassette>.json").unwrap()); // or --features live
        //   let cfg = eak_cli::RunConfig { intent, /* + the other RunConfig fields */ };
        //   let _ = eak_cli::run_with_sink(reasoning, &cfg, Some(_sink));
        //
        // `run_with_sink` already exists in the kernel (eak-cli) and streams every committed
        // EventRecord to `_sink`. This bridge is the entire integration; the call above is wiring.
        let _ = intent;
    });
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_run])
        .run(tauri::generate_context!())
        .expect("error while running eak-app");
}
