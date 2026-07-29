import type { EntityId } from "../contract/v1";
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
