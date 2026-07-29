import type { EventRecord } from "../contract/v1";

/**
 * Parse one line of `eak-events.jsonl` into an `EventRecord`, preserving u128 `EntityId`s.
 *
 * `EntityId(pub u128)` serializes as a bare number of up to 39 digits. `JSON.parse` would round
 * anything past 2^53, so we first wrap every long integer RUN that sits in value position
 * (`: 123…`) or array-element position (`[123…`, `,123…`) in quotes, turning it into a string
 * BEFORE parsing. 16 digits is the threshold: entity ids here are ~20–39 digits, while seqs,
 * timestamps, counts, and quantity magnitudes (which carry a decimal point, so their fractional
 * digits are preceded by `.` and never match) stay numeric.
 */
export function parseEventLine(line: string): EventRecord {
  const safe = line.replace(/([:[,]\s*)(\d{16,})(?=\s*[,\]}])/g, '$1"$2"');
  return JSON.parse(safe) as EventRecord;
}

/** Parse a whole `.jsonl` blob, skipping blank lines. */
export function parseEventLog(text: string): EventRecord[] {
  return text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.length > 0)
    .map(parseEventLine);
}
