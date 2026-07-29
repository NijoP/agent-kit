import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { parseEventLog } from "../events/parse";
import { foldAll, gateOf } from "./fold";

const here = dirname(fileURLToPath(import.meta.url));
const fixture = resolve(here, "../../public/fixtures/hero.jsonl");

/**
 * Fold-parity: folding the REAL captured kernel stream must reproduce exactly the entity counts
 * the kernel reported (`eak run` printed 4 requirements; the histogram confirmed the rest). This
 * pins the frontend fold to the kernel's `EngineeringState::apply` so the projection can never
 * silently drift from owned truth.
 */
describe("fold parity with the hero cassette", () => {
  const records = parseEventLog(readFileSync(fixture, "utf8"));
  const vm = foldAll(records);

  it("folds every committed event", () => {
    expect(records.length).toBe(151);
    expect(vm.eventCount).toBe(151);
  });

  it("reproduces the kernel's committed entity counts", () => {
    expect(vm.requirements.length).toBe(4);
    expect(vm.blocks.length).toBe(4);
    expect(vm.components.length).toBe(4);
    expect(vm.pins.length).toBe(8);
    expect(vm.nets.length).toBe(2);
    expect(vm.parts.length).toBe(2);
    expect(vm.placements.length).toBe(4);
    expect(vm.tracks.length).toBe(6);
    expect(vm.board).toBeDefined();
  });

  it("preserves u128 ids as strings so cross-references resolve", () => {
    // Every net member must resolve to a real committed pin id — this only holds if the big
    // u128 ids survived parsing without precision loss (the whole reason ids are strings).
    const pinIds = new Set(vm.pins.map((p) => p.id));
    for (const net of vm.nets) {
      for (const member of net.members) {
        expect(typeof member).toBe("string");
        expect(pinIds.has(member)).toBe(true);
      }
    }
    // Every placement references a real component.
    const compIds = new Set(vm.components.map((c) => c.id));
    for (const pl of vm.placements) expect(compIds.has(pl.component)).toBe(true);
  });

  it("reports the manufacturing gate as RELEASED (no blocking findings)", () => {
    const gate = gateOf(vm);
    expect(vm.released).toBeDefined();
    expect(gate.released).toBe(true);
  });
});
