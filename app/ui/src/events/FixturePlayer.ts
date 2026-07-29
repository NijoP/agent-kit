import type { EventRecord } from "../contract/v1";
import type { EventSource } from "./EventSource";
import { parseEventLog } from "./parse";

/**
 * Replays a captured `eak-events.jsonl` (real kernel output) as if it were streaming live, so the
 * UI can be developed and demoed in a plain browser — no webkit2gtk, no Tauri toolchain. The
 * cadence is a readability affordance only; the DATA is exactly what the kernel committed. This is
 * why the stream is labelled CASSETTE: a recorded run replayed deterministically (USER-MANUAL §9).
 */
export class FixturePlayer implements EventSource {
  readonly mode = "CASSETTE" as const;

  private records: EventRecord[] = [];
  private listeners = new Set<(r: EventRecord) => void>();
  private cursor = 0;
  private timer: number | null = null;
  private speed = 1;
  private loaded: Promise<void>;
  /** ms between events at 1× — brisk enough to feel alive, slow enough to read (§11.4). */
  private baseDelay = 90;

  constructor(url: string) {
    this.loaded = fetch(url)
      .then((r) => {
        if (!r.ok) throw new Error(`fixture ${url}: ${r.status}`);
        return r.text();
      })
      .then((text) => {
        this.records = parseEventLog(text);
      });
  }

  subscribe(onEvent: (record: EventRecord) => void): () => void {
    this.listeners.add(onEvent);
    return () => this.listeners.delete(onEvent);
  }

  start(): void {
    void this.loaded.then(() => this.tick());
  }

  pause(): void {
    if (this.timer !== null) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }

  restart(): void {
    this.pause();
    this.cursor = 0;
    void this.loaded.then(() => this.tick());
  }

  setSpeed(multiplier: number): void {
    this.speed = Math.max(0.25, multiplier);
  }

  skipToEnd(): void {
    this.pause();
    while (this.cursor < this.records.length) {
      this.emit(this.records[this.cursor++]);
    }
  }

  /** True once the underlying file has been fetched and parsed. */
  ready(): Promise<void> {
    return this.loaded;
  }

  private tick(): void {
    if (this.cursor >= this.records.length) return;
    this.emit(this.records[this.cursor++]);
    this.timer = window.setTimeout(() => this.tick(), this.baseDelay / this.speed);
  }

  private emit(record: EventRecord): void {
    for (const l of this.listeners) l(record);
  }
}
