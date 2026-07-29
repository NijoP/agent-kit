import type {
  Assumption,
  Board,
  BomLineItem,
  Component,
  Constraint,
  DesignIntent,
  EntityId,
  EventRecord,
  FunctionalBlock,
  ModelFidelity,
  Net,
  Objective,
  Part,
  Pin,
  Placement,
  ProvenanceLink,
  ReasoningRequest,
  ReasoningResponse,
  Requirement,
  Risk,
  Track,
  Tradeoff,
  Violation,
  Waiver,
} from "../contract/v1";
import { eventKind } from "../contract/v1";

/** One phase of the pipeline, tracked from the lifecycle events. */
export interface PhaseView {
  name: string;
  state: string;
  status: "pending" | "active" | "done" | "failed";
}

/** A single entry in the agent activity feed (the narrative in the left panel). */
export interface Activity {
  seq: number;
  kind: string;
  /** UI grouping: what surface this belongs to. */
  lane: "you" | "reason" | "commit" | "milestone" | "assumption" | "violation" | "tradeoff" | "risk";
  title: string;
  detail?: string;
  entity?: EntityId;
  severity?: "error" | "warning" | "info" | "pass";
}

export interface FidelityTag {
  target: EntityId;
  fidelity: ModelFidelity;
  reasoningCallSeq: number | null;
}

export interface ReasoningRecord {
  seq: number;
  request: ReasoningRequest;
  response: ReasoningResponse;
}

/**
 * The projection the panels render — the fold of the kernel's committed-event stream. It is a
 * VIEW of owned truth, never truth itself (USER-MANUAL §2). Mirrors the shape of
 * `eak-runtime::EngineeringState`.
 */
export interface ViewModel {
  intent?: DesignIntent;
  requirements: Requirement[];
  constraints: Constraint[];
  blocks: FunctionalBlock[];
  components: Component[];
  pins: Pin[];
  nets: Net[];
  parts: Part[];
  bomLines: BomLineItem[];
  board?: Board;
  placements: Placement[];
  tracks: Track[];
  violations: Violation[];
  waivers: Waiver[];
  assumptions: Assumption[];
  risks: Risk[];
  objectives: Objective[];
  tradeoffs: Tradeoff[];
  fidelityTags: FidelityTag[];
  explanations: Record<EntityId, { explanation: string; suggestedFix: string }>;
  provenance: ProvenanceLink[];
  reasoning: ReasoningRecord[];
  phases: PhaseView[];
  activity: Activity[];
  /** Terminal milestone — set when ManufacturingGenerated arrives. */
  released?: { placeCount: number; copperCount: number; lineItemCount: number };
  lastSeq: number;
  eventCount: number;
}

export function emptyViewModel(): ViewModel {
  return {
    requirements: [],
    constraints: [],
    blocks: [],
    components: [],
    pins: [],
    nets: [],
    parts: [],
    bomLines: [],
    placements: [],
    tracks: [],
    violations: [],
    waivers: [],
    assumptions: [],
    risks: [],
    objectives: [],
    tradeoffs: [],
    fidelityTags: [],
    explanations: {},
    provenance: [],
    reasoning: [],
    phases: [],
    activity: [],
    lastSeq: -1,
    eventCount: 0,
  };
}

/** The manufacturing-gate verdict — mirrors the kernel's blocking rules exactly. */
export interface Gate {
  released: boolean;
  reason?: string;
}
export function gateOf(vm: ViewModel): Gate {
  const blockingViolation = vm.violations.find(
    (v) => v.severity === "Error" && v.status === "Open",
  );
  if (blockingViolation) return { released: false, reason: `open violation · ${blockingViolation.rule}` };
  const openCritical = vm.assumptions.find(
    (a) => a.criticality === "Critical" && a.status === "Open",
  );
  if (openCritical) return { released: false, reason: "undischarged critical assumption" };
  // Released only once the terminal manufacturing IR has been generated.
  if (!vm.released) return { released: false, reason: "in progress" };
  return { released: true };
}

function upsertPhase(phases: PhaseView[], name: string, patch: Partial<PhaseView>): PhaseView[] {
  const idx = phases.findIndex((p) => p.name === name);
  if (idx === -1) return [...phases, { name, state: "", status: "pending", ...patch }];
  const next = phases.slice();
  next[idx] = { ...next[idx], ...patch };
  return next;
}

/**
 * Apply one committed event. Pure and additive: given the same event log it always produces the
 * same ViewModel (determinism, mirroring P4). Returns a NEW object so React/Zustand re-render.
 */
export function fold(vm: ViewModel, record: EventRecord): ViewModel {
  const ev = record.event;
  const kind = eventKind(ev);
  const next: ViewModel = { ...vm, lastSeq: record.seq, eventCount: vm.eventCount + 1 };
  const push = (a: Omit<Activity, "seq" | "kind">) =>
    (next.activity = [...next.activity, { seq: record.seq, kind, ...a }]);

  switch (kind) {
    case "IntentCaptured": {
      const { intent } = (ev as any).IntentCaptured;
      next.intent = intent;
      push({ lane: "you", title: intent.statement, detail: `source: ${intent.source}` });
      break;
    }
    case "ReasoningCall": {
      const { request, response } = (ev as any).ReasoningCall;
      next.reasoning = [...vm.reasoning, { seq: record.seq, request, response }];
      const n = response?.candidates?.length ?? 0;
      push({
        lane: "reason",
        title: `Reasoning · ${request.schema_name}`,
        detail: n ? `${n} candidate${n === 1 ? "" : "s"} · model ${request.model_id}` : `model ${request.model_id}`,
      });
      break;
    }
    case "RequirementCommitted": {
      const { requirement } = (ev as any).RequirementCommitted;
      next.requirements = [...vm.requirements, requirement];
      push({
        lane: "commit",
        title: `Requirement · ${requirement.category}`,
        detail: requirement.statement,
        entity: requirement.id,
      });
      break;
    }
    case "ConstraintCommitted": {
      const { constraint } = (ev as any).ConstraintCommitted;
      next.constraints = [...vm.constraints, constraint];
      break;
    }
    case "FunctionalBlockCommitted": {
      const { block } = (ev as any).FunctionalBlockCommitted;
      next.blocks = [...vm.blocks, block];
      push({ lane: "commit", title: `Block · ${block.name}`, detail: block.function, entity: block.id });
      break;
    }
    case "ComponentCommitted": {
      const { component } = (ev as any).ComponentCommitted;
      next.components = [...vm.components, component];
      push({
        lane: "commit",
        title: `Component · ${component.refdes}`,
        detail: `${component.class}`,
        entity: component.id,
      });
      break;
    }
    case "PinCommitted": {
      const { pin } = (ev as any).PinCommitted;
      next.pins = [...vm.pins, pin];
      break;
    }
    case "NetCommitted": {
      const { net } = (ev as any).NetCommitted;
      next.nets = [...vm.nets, net];
      push({
        lane: "commit",
        title: `Net · ${net.name}`,
        detail: `${net.class} · ${net.members.length} pins`,
        entity: net.id,
      });
      break;
    }
    case "PartCommitted": {
      const { part } = (ev as any).PartCommitted;
      next.parts = [...vm.parts, part];
      push({
        lane: "commit",
        title: `Part · ${part.mpn}`,
        detail: `${part.manufacturer} · ${part.lifecycle}`,
        entity: part.id,
      });
      break;
    }
    case "BomLineItemCommitted": {
      const { item } = (ev as any).BomLineItemCommitted;
      next.bomLines = [...vm.bomLines, item];
      break;
    }
    case "BoardCommitted": {
      const { board } = (ev as any).BoardCommitted;
      next.board = board;
      push({
        lane: "commit",
        title: "Board outline",
        detail: `${board.width.magnitude}×${board.height.magnitude} mm · ${board.stack.layers.length} layers`,
        entity: board.id,
      });
      break;
    }
    case "PlacementCommitted": {
      const { placement } = (ev as any).PlacementCommitted;
      next.placements = [...vm.placements, placement];
      break;
    }
    case "TrackCommitted": {
      const { track } = (ev as any).TrackCommitted;
      next.tracks = [...vm.tracks, track];
      break;
    }
    case "ViolationRaised": {
      const { violation } = (ev as any).ViolationRaised;
      next.violations = [...vm.violations, violation];
      push({
        lane: "violation",
        title: `Violation · ${violation.rule}`,
        detail: violation.message,
        entity: violation.id,
        severity:
          violation.severity === "Error" ? "error" : violation.severity === "Warning" ? "warning" : "info",
      });
      break;
    }
    case "WaiverGranted": {
      const { waiver } = (ev as any).WaiverGranted;
      next.waivers = [...vm.waivers, waiver];
      next.violations = vm.violations.map((v) =>
        v.id === waiver.violation ? { ...v, status: "Waived" } : v,
      );
      break;
    }
    case "AssumptionRaised": {
      const { assumption } = (ev as any).AssumptionRaised;
      next.assumptions = [...vm.assumptions, assumption];
      push({
        lane: "assumption",
        title: `Assumption${assumption.criticality === "Critical" ? " · CRITICAL" : ""}`,
        detail: assumption.statement,
        entity: assumption.id,
        severity: assumption.criticality === "Critical" ? "warning" : "info",
      });
      break;
    }
    case "AssumptionDischarged": {
      const { assumption, discharge } = (ev as any).AssumptionDischarged;
      next.assumptions = vm.assumptions.map((a) =>
        a.id === assumption ? { ...a, status: "Discharged", discharge } : a,
      );
      break;
    }
    case "RiskRaised": {
      const { risk } = (ev as any).RiskRaised;
      next.risks = [...vm.risks, risk];
      push({ lane: "risk", title: `Risk · ${risk.severity}`, detail: risk.statement, entity: risk.id });
      break;
    }
    case "RiskAccepted": {
      const { risk, accepted_by } = (ev as any).RiskAccepted;
      next.risks = vm.risks.map((r) =>
        r.id === risk ? { ...r, status: "Accepted", owner: accepted_by } : r,
      );
      break;
    }
    case "ObjectiveRecorded": {
      const { objective } = (ev as any).ObjectiveRecorded;
      next.objectives = [...vm.objectives, objective];
      break;
    }
    case "TradeoffRecorded": {
      const { tradeoff } = (ev as any).TradeoffRecorded;
      next.tradeoffs = [...vm.tradeoffs, tradeoff];
      const chosen = tradeoff.alternatives[tradeoff.chosen];
      push({
        lane: "tradeoff",
        title: `Tradeoff · chose ${chosen?.label ?? "?"}`,
        detail: tradeoff.question,
        entity: tradeoff.id,
      });
      break;
    }
    case "FidelityTagged": {
      const { target, fidelity, reasoning_call_seq } = (ev as any).FidelityTagged;
      next.fidelityTags = [...vm.fidelityTags, { target, fidelity, reasoningCallSeq: reasoning_call_seq }];
      break;
    }
    case "ViolationExplained": {
      const { violation, explanation, suggested_fix } = (ev as any).ViolationExplained;
      next.explanations = { ...vm.explanations, [violation]: { explanation, suggestedFix: suggested_fix } };
      break;
    }
    case "ProvenanceLinked": {
      const { link } = (ev as any).ProvenanceLinked;
      next.provenance = [...vm.provenance, link];
      break;
    }
    // ---- phase lifecycle ----
    case "PhaseEntered": {
      const { phase, state } = (ev as any).PhaseEntered;
      next.phases = upsertPhase(vm.phases, phase, { state, status: "active" });
      break;
    }
    case "PhaseStateChanged": {
      const { phase, to } = (ev as any).PhaseStateChanged;
      next.phases = upsertPhase(vm.phases, phase, { state: to, status: "active" });
      break;
    }
    case "PhaseCompleted": {
      const { phase } = (ev as any).PhaseCompleted;
      next.phases = upsertPhase(vm.phases, phase, { status: "done" });
      break;
    }
    case "PhaseFailed": {
      const { phase, reason } = (ev as any).PhaseFailed;
      next.phases = upsertPhase(vm.phases, phase, { status: "failed", state: reason });
      break;
    }
    case "ManufacturingGenerated": {
      const { place_count, copper_count, line_item_count } = (ev as any).ManufacturingGenerated;
      next.released = { placeCount: place_count, copperCount: copper_count, lineItemCount: line_item_count };
      push({
        lane: "milestone",
        title: "Manufacturing IR generated",
        detail: `${place_count} placements · ${copper_count} copper · ${line_item_count} BOM lines`,
        severity: "pass",
      });
      break;
    }
    // IR milestones + constraints-extracted + verification-completed are audit-only; folded
    // implicitly by leaving state unchanged (they inform the pipeline stepper via phase events).
    default:
      break;
  }
  return next;
}

/** Fold a whole log at once (used by the fold-parity test and skip-to-end). */
export function foldAll(records: EventRecord[]): ViewModel {
  return records.reduce(fold, emptyViewModel());
}
