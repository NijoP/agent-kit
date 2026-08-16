import type {
  BomLineItem,
  Component,
  EntityId,
  Net,
  NetClass,
  Part,
  Pin,
  Violation,
} from "../contract/v1";
import type { ViewModel } from "./fold";

/**
 * Pure, projection-side derivations over the folded ViewModel. Every function here is a VIEW of
 * owned truth — it computes display geometry (pad rects, ratline airwires, layer membership) from
 * committed facts and never mutates state (vision §10: projections are outputs of the model).
 */

export interface PadProjection {
  id: EntityId;
  component: EntityId;
  refdes: string;
  klass: string;
  x: number;
  y: number;
  w: number;
  h: number;
  cx: number;
  cy: number;
  side: "Top" | "Bottom";
}

/** Component footprints projected as copper pads (v1: courtyard-as-footprint — a stated fidelity boundary). */
export function pads(vm: ViewModel): PadProjection[] {
  return vm.placements.map((p) => {
    const comp = vm.components.find((c) => c.id === p.component);
    const x = p.x.magnitude;
    const y = p.y.magnitude;
    const w = p.width.magnitude;
    const h = p.height.magnitude;
    return {
      id: p.id,
      component: p.component,
      refdes: comp?.refdes ?? "?",
      klass: comp?.class ?? "",
      x,
      y,
      w,
      h,
      cx: x + w / 2,
      cy: y + h / 2,
      side: p.side,
    };
  });
}

export interface Ratline {
  netId: EntityId;
  netName: string;
  netClass: string;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

/**
 * Net airwires: for each net, chain the centroids of the distinct components its member pins belong
 * to (the logical connection the router realizes as copper). Shown on the Ratline layer — the
 * connectivity a hardware engineer reads before/after routing.
 */
export function ratlines(vm: ViewModel): Ratline[] {
  const centroidByComponent = new Map<EntityId, { x: number; y: number }>();
  for (const p of pads(vm)) centroidByComponent.set(p.component, { x: p.cx, y: p.cy });
  const componentOfPin = new Map<EntityId, EntityId>();
  for (const pin of vm.pins) componentOfPin.set(pin.id, pin.component);

  const out: Ratline[] = [];
  for (const net of vm.nets) {
    // distinct components on this net, in first-seen order
    const comps: EntityId[] = [];
    const seen = new Set<EntityId>();
    for (const pinId of net.members) {
      const comp = componentOfPin.get(pinId);
      if (comp && !seen.has(comp) && centroidByComponent.has(comp)) {
        seen.add(comp);
        comps.push(comp);
      }
    }
    for (let i = 0; i + 1 < comps.length; i++) {
      const a = centroidByComponent.get(comps[i])!;
      const b = centroidByComponent.get(comps[i + 1])!;
      out.push({ netId: net.id, netName: net.name, netClass: net.class, x1: a.x, y1: a.y, x2: b.x, y2: b.y });
    }
  }
  return out;
}

export type LayerKey =
  | "outline"
  | "topCopper"
  | "bottomCopper"
  | "silk"
  | "ratline"
  | "tracks"
  | "drill";

export interface LayerDescriptor {
  key: LayerKey;
  name: string;
  color: string; // CSS var
  count: number;
  side?: "Top" | "Bottom";
}

/** The Layers panel model — present layers + how many owned objects each projects. */
export function layerModel(vm: ViewModel): LayerDescriptor[] {
  const top = vm.placements.filter((p) => p.side === "Top").length;
  const bottom = vm.placements.filter((p) => p.side === "Bottom").length;
  const ratlineCount = ratlines(vm).length;
  return [
    { key: "topCopper", name: "Top Copper", color: "var(--ecad-copper-top)", count: top, side: "Top" },
    { key: "bottomCopper", name: "Bottom Copper", color: "var(--ecad-copper-bottom)", count: bottom, side: "Bottom" },
    { key: "tracks", name: "Routing", color: "var(--pass)", count: vm.tracks.length },
    { key: "silk", name: "Silkscreen", color: "var(--ecad-silk)", count: vm.placements.length },
    { key: "ratline", name: "Ratlines", color: "var(--ecad-ratline)", count: ratlineCount },
    { key: "outline", name: "Board Outline", color: "var(--ecad-outline)", count: vm.board ? 1 : 0 },
    { key: "drill", name: "Drill / Pads", color: "var(--ecad-drill)", count: vm.placements.length },
  ];
}

export interface BoardBounds {
  w: number;
  h: number;
}
export function boardBounds(vm: ViewModel): BoardBounds | null {
  if (!vm.board) return null;
  return { w: vm.board.width.magnitude, h: vm.board.height.magnitude };
}

// ── Net classes ────────────────────────────────────────────────────────────────────────────────

/** The CSS var a net's class projects to (used by PCB, schematic, nets panel, inspector). */
export function netClassColor(cls: NetClass): string {
  switch (cls) {
    case "Power":
      return "var(--ecad-net-power)";
    case "Ground":
      return "var(--ecad-net-ground)";
    case "Signal":
      return "var(--ecad-net-signal)";
  }
}

/** Nets carrying an impedance target render as high-speed violet. */
export function netColor(net: Net): string {
  return net.impedance_target ? "var(--ecad-net-highspeed)" : netClassColor(net.class);
}

// ── Object maps (component ⇄ pin ⇄ net ⇄ part) ────────────────────────────────────────────────

export function componentOfPin(vm: ViewModel, pinId: EntityId): Component | undefined {
  const pin = vm.pins.find((p) => p.id === pinId);
  return pin ? vm.components.find((c) => c.id === pin.component) : undefined;
}

export function pinsForComponent(vm: ViewModel, compId: EntityId): Pin[] {
  return vm.pins.filter((p) => p.component === compId);
}

export function netsForComponent(vm: ViewModel, compId: EntityId): Net[] {
  const pinIds = new Set(pinsForComponent(vm, compId).map((p) => p.id));
  return vm.nets.filter((n) => n.members.some((m) => pinIds.has(m)));
}

export function pinOfNet(vm: ViewModel, pinId: EntityId): Net | undefined {
  return vm.nets.find((n) => n.members.includes(pinId));
}

export function netOfTrack(vm: ViewModel, trackId: EntityId): Net | undefined {
  const t = vm.tracks.find((x) => x.id === trackId);
  return t ? vm.nets.find((n) => n.id === t.net) : undefined;
}

export function componentOfPlacement(vm: ViewModel, placementId: EntityId): Component | undefined {
  const p = vm.placements.find((x) => x.id === placementId);
  return p ? vm.components.find((c) => c.id === p.component) : undefined;
}

/** Pads (footprint courtyards) belonging to a component. */
export function componentPads(vm: ViewModel, compId: EntityId): PadProjection[] {
  return pads(vm).filter((p) => p.component === compId);
}

/** Pads on a net — every footprint whose component has a pin on that net. */
export function padsForNet(vm: ViewModel, netId: EntityId): PadProjection[] {
  const pinIds = new Set(vm.nets.find((n) => n.id === netId)?.members ?? []);
  const compIds = new Set<EntityId>();
  for (const pin of vm.pins) if (pinIds.has(pin.id)) compIds.add(pin.component);
  return pads(vm).filter((p) => compIds.has(p.component));
}

export function trackIdsForNet(vm: ViewModel, netId: EntityId): EntityId[] {
  return vm.tracks.filter((t) => t.net === netId).map((t) => t.id);
}

// ── Parts ⇄ components (via BOM lines) ────────────────────────────────────────────────────────

export function bomLineForPart(vm: ViewModel, partId: EntityId): BomLineItem | undefined {
  return vm.bomLines.find((l) => l.part === partId);
}

export function componentsForPart(vm: ViewModel, partId: EntityId): Component[] {
  const line = bomLineForPart(vm, partId);
  return vm.components.filter((c) => line?.components.includes(c.id));
}

export function partOfComponent(vm: ViewModel, compId: EntityId): Part | undefined {
  return vm.parts.find((p) => componentsForPart(vm, p.id).some((c) => c.id === compId));
}

export function partOfPin(vm: ViewModel, pinId: EntityId): Part | undefined {
  const comp = componentOfPin(vm, pinId);
  return comp ? partOfComponent(vm, comp.id) : undefined;
}

// ── Net routing state ──────────────────────────────────────────────────────────────────────────

/** Net routing state, derived from committed tracks (the router's progress — not the ratlines'). */
export type NetState = "trivial" | "routed" | "unrouted";
export function netState(vm: ViewModel, netId: EntityId): NetState {
  const net = vm.nets.find((n) => n.id === netId);
  if (!net || net.members.length < 2) return "trivial";
  return vm.tracks.some((t) => t.net === netId) ? "routed" : "unrouted";
}

// ── Cross-probe resolution ─────────────────────────────────────────────────────────────────────

/**
 * Resolve any selectable entity to the net it belongs to, when one exists. This is the seam the
 * canvas cross-probing uses: select a pad, a component, a pin, a track, or a violation subject and
 * the board/schematic highlight the same net.
 */
export function highlightNetOf(vm: ViewModel, id: EntityId | undefined): EntityId | undefined {
  if (!id) return undefined;
  if (vm.nets.some((n) => n.id === id)) return id;
  if (vm.tracks.some((t) => t.id === id)) return netOfTrack(vm, id)?.id;
  if (vm.pins.some((p) => p.id === id)) return pinOfNet(vm, id)?.id;
  const comp = vm.components.find((c) => c.id === id);
  if (comp) return netsForComponent(vm, comp.id)[0]?.id;
  const placement = vm.placements.find((p) => p.id === id);
  if (placement) {
    const c = componentOfPlacement(vm, placement.id);
    if (c) return netsForComponent(vm, c.id)[0]?.id;
  }
  const viol = vm.violations.find((v) => v.id === id);
  if (viol) return viol.subjects.find((s) => vm.nets.some((n) => n.id === s));
  return undefined;
}

export function trackOf(vm: ViewModel, id: EntityId) {
  return vm.tracks.find((t) => t.id === id);
}
export function violationOf(vm: ViewModel, id: EntityId): Violation | undefined {
  return vm.violations.find((v) => v.id === id);
}

// ── Schematic graph ────────────────────────────────────────────────────────────────────────────

export interface SchematicPin {
  pin: Pin;
  designation: string;
  type: Pin["electrical_type"];
  /** Y offset within the node box (0 = top). */
  y: number;
}
export interface SchematicNode {
  component: Component;
  x: number;
  y: number;
  w: number;
  h: number;
  pins: SchematicPin[];
}
export interface SchematicRail {
  net: Net;
  color: string;
  y: number;
}
export interface SchematicGraph {
  nodes: SchematicNode[];
  rails: SchematicRail[];
  /** Bounding box of the whole projection, in layout units. */
  w: number;
  h: number;
}

const BOX_W = 128;
const BOX_H = 170;
const BOX_GAP = 84;
const ROW_GAP = 70;
const TOP_M = 28;
const RAIL_GAP = 92;
const RAIL_TOP_M = 40;

const PIN_ORDER: Record<Pin["electrical_type"], number> = {
  PowerIn: 0,
  PowerOut: 1,
  Input: 2,
  Bidirectional: 3,
  Output: 4,
  Passive: 5,
  Ground: 6,
  NoConnect: 7,
};

/**
 * Schematic projection — the owned architecture drawn as a schematic sheet: one node box per
 * component, pins on the left edge, and one horizontal rail per net labelled with its name and
 * colored by class. This is PURE geometry derived from committed pins/nets (the schematic can never
 * claim a connection the kernel did not commit).
 */
export function schematicGraph(vm: ViewModel): SchematicGraph {
  const nodes: SchematicNode[] = [];
  const perRow = Math.max(1, Math.ceil(Math.sqrt(vm.components.length * 3)) || 1);

  vm.components.forEach((c, i) => {
    const col = i % perRow;
    const row = Math.floor(i / perRow);
    const sorted = pinsForComponent(vm, c.id).slice().sort((a, b) => PIN_ORDER[a.electrical_type] - PIN_ORDER[b.electrical_type]);
    const pinGap = Math.max(34, (BOX_H - 60) / Math.max(1, sorted.length));
    const pins: SchematicPin[] = sorted.map((pin, pIdx) => ({
      pin,
      designation: pin.designation,
      type: pin.electrical_type,
      y: 34 + pIdx * pinGap,
    }));
    nodes.push({ component: c, x: TOP_M + col * (BOX_W + BOX_GAP), y: TOP_M + row * (BOX_H + ROW_GAP), w: BOX_W, h: BOX_H, pins });
  });

  // Rails below the node grid, ordered power → signal → ground (ground reads at the bottom).
  const classOrder: Record<NetClass, number> = { Power: 0, Signal: 1, Ground: 2 };
  const nets = vm.nets.slice().sort((a, b) => classOrder[a.class] - classOrder[b.class]);
  const gridBottom = nodes.reduce((m, n) => Math.max(m, n.y + n.h), TOP_M);
  const rails: SchematicRail[] = nets.map((net, i) => ({ net, color: netColor(net), y: gridBottom + RAIL_TOP_M + i * RAIL_GAP }));

  const width = TOP_M + (Math.min(perRow, Math.max(vm.components.length, 1)) * (BOX_W + BOX_GAP) - BOX_GAP) + TOP_M;
  const height = (rails.length ? rails[rails.length - 1].y : gridBottom) + 40;
  return { nodes, rails, w: Math.max(width, 60), h: Math.max(height, 60) };
}
