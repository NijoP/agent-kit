import type { EventRecord } from "../contract/v1";
import type { EventSource } from "./EventSource";

/**
 * The live event source for the PACKAGED desktop app. It mirrors the existing spine in
 * `app/src-tauri/src/main.rs`: the Rust `TauriEventSink` forwards every committed `EventRecord`
 * as an `eak://event` Tauri event, and `start_run` kicks off a real pipeline run. This class is
 * the frontend half of that bridge — the store folds its events exactly as it folds a fixture's.
 *
 * It is loaded lazily (dynamic import of `@tauri-apps/api`) so the browser dev build, which has no
 * Tauri runtime, never touches it. `main.tsx` selects FixturePlayer in the browser and this bridge
 * only when `window.__TAURI__` is present.
 *
 * ⚠ Contract note (u128): events crossing the Tauri IPC are JSON-deserialized by the webview, so
 * the same u128→Number precision loss as the fixture path applies. The kernel-side fix is to
 * serialize `EntityId` as a string across the sink boundary; until then, ids must be re-stringified
 * here too. Tracked against `contract v1`.
 */
export class TauriBridge implements EventSource {
  readonly mode = "REAL" as const;
  private listeners = new Set<(r: EventRecord) => void>();
  private unlisten: (() => void) | null = null;

  subscribe(onEvent: (record: EventRecord) => void): () => void {
    this.listeners.add(onEvent);
    void this.ensureListening();
    return () => this.listeners.delete(onEvent);
  }

  start(intent?: string): void {
    void import("@tauri-apps/api/core").then(({ invoke }) => {
      invoke("start_run", { intent: intent ?? "" });
    });
  }

  private async ensureListening(): Promise<void> {
    if (this.unlisten) return;
    const { listen } = await import("@tauri-apps/api/event");
    this.unlisten = await listen<EventRecord>("eak://event", (e) => {
      const record = e.payload;
      for (const l of this.listeners) l(record);
    });
  }
}
