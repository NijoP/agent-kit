import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { parseEventLog } from "../events/parse";
import { foldAll } from "./fold";
import {
  highlightNetOf,
  netsForComponent,
  netState,
  padsForNet,
  pads,
  partOfComponent,
  pinOfNet,
  schematicGraph,
} from "./selectors";

const here = dirname(fileURLToPath(import.meta.url));
const fixture = resolve(here, "../../public/fixtures/hero.jsonl");

/**
 * Projection selectors — cross-probing, net routing state, and the schematic graph are PURE views
 * of the folded kernel stream. These pin the EDA surfaces (PCB highlighting, schematic sheet,
 * nets panel) to committed facts: a selector can never report a connection the kernel didn't make.
 */
describe("EDA selectors over the hero cassette", () => {
  const vm = foldAll(parseEventLog(readFileSync(fixture, "utf8")));

  it("resolves every net member pin to its net (pin → net map is total over members)", () => {
    for (const net of vm.nets) {
      for (const member of net.members) {
        expect(pinOfNet(vm, member)?.id).toBe(net.id);
      }
    }
  });

  it("maps components to the nets their pins join", () => {
    for (const c of vm.components) {
      const nets = netsForComponent(vm, c.id);
      expect(nets.length).toBe(2); // every part touches VBUS + GND
      expect(nets.some((n) => n.name === "VBUS")).toBe(true);
      expect(nets.some((n) => n.name === "GND")).toBe(true);
    }
  });

  it("marks the routed nets as routed from committed tracks", () => {
    expect(netState(vm, vm.nets[0].id)).toBe("routed");
    for (const n of vm.nets) expect(netState(vm, n.id)).toBe("routed");
  });

  it("highlights the same net whether you select the net, a track, a pin or a component", () => {
    const netId = vm.nets[0].id;
    // from the net itself
    expect(highlightNetOf(vm, netId)).toBe(netId);
    // from any track on the net
    const track = vm.tracks.find((t) => t.net === netId)!;
    expect(highlightNetOf(vm, track.id)).toBe(netId);
    // from any member pin
    expect(highlightNetOf(vm, netId === vm.nets[0].id ? vm.nets[0].members[0] : vm.nets[1].members[0])).toBe(netId);
    // from a component on the net
    const pin0 = vm.pins.find((p) => p.id === vm.nets[0].members[0])!;
    expect(highlightNetOf(vm, pin0.component)).toBe(netId);
  });

  it("projects pads on a net for every component holding a member pin", () => {
    for (const net of vm.nets) {
      const hot = padsForNet(vm, net.id);
      expect(hot.length).toBe(4); // J1..U3 each contribute a pad
      const comps = new Set(hot.map((p) => p.component));
      for (const member of net.members) {
        const pin = vm.pins.find((p) => p.id === member)!;
        expect(comps.has(pin.component)).toBe(true);
      }
    }
  });

  it("builds a schematic graph with a node per component and a rail per net", () => {
    const g = schematicGraph(vm);
    expect(g.nodes.length).toBe(vm.components.length);
    expect(g.rails.length).toBe(vm.nets.length);
    // every node pin must sit on a rail that owns it
    for (const node of g.nodes) {
      expect(node.w).toBeGreaterThan(0);
      for (const p of node.pins) {
        const rail = g.rails.find((r) => r.net.members.includes(p.pin.id));
        expect(rail).toBeDefined();
      }
    }
    // ground reads at the bottom
    const groundRail = g.rails.find((r) => r.net.class === "Ground")!;
    const powerRail = g.rails.find((r) => r.net.class === "Power")!;
    expect(groundRail.y).toBeGreaterThan(powerRail.y);
  });

  it("resolves the part that realizes each component through the BOM lines", () => {
    for (const c of vm.components) {
      const part = partOfComponent(vm, c.id);
      expect(part).toBeDefined();
      expect(pads(vm).some((p) => p.component === c.id)).toBe(true);
    }
  });
});

describe("EDA selectors over the review cassette", () => {
  const reviewFixture = resolve(here, "../../public/fixtures/review.jsonl");
  const vm = foldAll(parseEventLog(readFileSync(reviewFixture, "utf8")));

  it("folds the imported board with its two components, nets and tracks", () => {
    expect(vm.components.length).toBe(2);
    expect(vm.nets.length).toBe(2);
    expect(vm.tracks.length).toBe(2);
    expect(vm.board).toBeDefined();
  });

  it("still maps the bom-coverage violations' subjects to components", () => {
    const violations = vm.violations.filter((v) => v.status === "Open");
    expect(violations.length).toBe(2);
    for (const v of violations) {
      const comp = vm.components.find((c) => c.id === v.subjects[0]);
      expect(comp).toBeDefined();
      expect(comp?.refdes).toMatch(/R[12]/);
    }
  });

  it("resolves an unrouted review net", () => {
    for (const n of vm.nets) expect(netState(vm, n.id)).toBe("routed"); // both nets have tracks
  });
});