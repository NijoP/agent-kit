import type { EventSource } from "../events/EventSource";

/** Honesty badge — REAL / CURATED / CASSETTE / SCAFFOLD (USER-MANUAL §9). */
export function HonestyBadge({ mode }: { mode: EventSource["mode"] }) {
  const map: Record<EventSource["mode"], { color: string; dashed?: boolean }> = {
    REAL: { color: "var(--pass)" },
    CURATED: { color: "var(--warn)" },
    CASSETTE: { color: "var(--cassette)" },
    SCAFFOLD: { color: "var(--scaffold)", dashed: true },
  };
  const { color, dashed } = map[mode];
  return (
    <span
      className="badge"
      style={{
        color,
        borderColor: color,
        borderStyle: dashed ? "dashed" : "solid",
      }}
    >
      <span className="dot" style={{ background: color, animation: "eak-pulse 2s var(--ease) infinite" }} />
      {mode}
    </span>
  );
}

/** Small status pill with a leading glyph. */
export function Pill({
  tone,
  children,
}: {
  tone: "pass" | "warn" | "error" | "muted" | "accent";
  children: React.ReactNode;
}) {
  const color =
    tone === "pass"
      ? "var(--pass)"
      : tone === "warn"
        ? "var(--warn)"
        : tone === "error"
          ? "var(--error)"
          : tone === "accent"
            ? "var(--accent)"
            : "var(--text-muted)";
  return (
    <span className="pill" style={{ color, borderColor: color }}>
      {children}
    </span>
  );
}
