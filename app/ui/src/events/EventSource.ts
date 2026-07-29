import type { EventRecord } from "../contract/v1";

/**
 * The seam the UI subscribes to — the frontend mirror of the kernel's `EventSink` port
 * (`eak-ports::EventSink`). The runtime commits events in seq order and hands each one to a sink;
 * the UI is one such observer. Two implementations satisfy this interface:
 *
 *   • FixturePlayer — replays a captured `eak-events.jsonl` in the browser (development / demo).
 *   • TauriBridge   — subscribes to the live `eak://event` stream from the packaged desktop app.
 *
 * Nothing else in the UI knows which one is running: the store folds whatever a source emits.
 */
export interface EventSource {
  /** The honesty label of this stream, shown in the status bar. */
  readonly mode: "CASSETTE" | "REAL" | "CURATED" | "SCAFFOLD";
  /** Register a listener; returns an unsubscribe fn. Called once per committed event, in order. */
  subscribe(onEvent: (record: EventRecord) => void): () => void;
  /** Begin the run. For a fixture this starts playback; for the bridge it invokes `start_run`. */
  start(intent?: string): void;
  /** Pause emission (fixtures only; a no-op for the live bridge). */
  pause?(): void;
  /** Restart from the first event (fixtures only). */
  restart?(): void;
  /** Playback rate multiplier (fixtures only). */
  setSpeed?(multiplier: number): void;
  /** Emit all remaining events immediately (fixtures only). */
  skipToEnd?(): void;
}
