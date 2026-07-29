import type { ViewModel } from "../store/fold";
import type { EntityId } from "../contract/v1";
import { ratlines } from "../store/selectors";

/**
 * Engineering tasks, derived PURELY from the owned model — the to-do list the engineer never has to
 * hand-maintain. Open critical assumptions, unrouted nets, single-sourced parts, open violations, and
 * undischarged verification items surface here automatically (vision: knowledge is captured, not lost).
 */
export interface Task {
  id: string;
  title: string;
  severity: "error" | "warn" | "info";
  source: string; // which doc/check produced it
  entity?: EntityId;
}

export function tasks(vm: ViewModel): Task[] {
  const out: Task[] = [];

  for (const v of vm.violations.filter((x) => x.status === "Open")) {
    out.push({
      id: `viol-${v.id}`,
      title: `${v.rule}: ${v.message}`,
      severity: v.severity === "Error" ? "error" : "warn",
      source: "verification/checklist.md",
      entity: v.subjects[0],
    });
  }

  for (const a of vm.assumptions.filter((x) => x.status === "Open")) {
    out.push({
      id: `asm-${a.id}`,
      title: `Discharge ${a.criticality === "Critical" ? "CRITICAL " : ""}assumption: ${a.statement}`,
      severity: a.criticality === "Critical" ? "error" : "info",
      source: "verification/checklist.md",
      entity: a.id,
    });
  }

  // unrouted nets: a net whose airwires aren't all realized by a track (heuristic: a net with members
  // but 0 tracks referencing it)
  const routedNets = new Set(vm.tracks.map((t) => t.net));
  for (const n of vm.nets) {
    if (n.members.length > 1 && !routedNets.has(n.id)) {
      out.push({ id: `net-${n.id}`, title: `Route net ${n.name}`, severity: "warn", source: "docs/02-architecture.md", entity: n.id });
    }
  }

  // single-sourced parts (no second source across BOM lines — every part appears once → a supply risk)
  for (const p of vm.parts) {
    out.push({ id: `src-${p.id}`, title: `Add a second source for ${p.mpn}`, severity: "info", source: "bom/bom.md", entity: p.id });
  }

  // ratline sanity (info)
  const rl = ratlines(vm).length;
  if (rl === 0 && vm.nets.length > 0) {
    out.push({ id: "rl-none", title: "All net connections realized", severity: "info", source: "docs/02-architecture.md" });
  }

  return out;
}
