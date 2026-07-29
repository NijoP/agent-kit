/*
 * contract/v1.ts — the frontend mirror of the kernel's serialized `Event` contract.
 *
 * This is hand-mirrored from `eak/crates/eak-ports/src/lib.rs` (Event, EventRecord,
 * ReasoningRequest/Response) and `eak/crates/eak-domain/src/lib.rs` (the entities). The kernel
 * is the source of truth; this file is the projection's view of it. When the kernel's contract
 * changes, this file must change with it — ideally these types are generated (ts-rs) rather than
 * hand-kept. The status bar prints `contract v1` so a drift is visible.
 *
 * ── The u128 gotcha ──────────────────────────────────────────────────────────────────────────
 * `EntityId(pub u128)` serializes as a BARE JSON number up to 39 digits — far beyond JS Number's
 * 2^53 safe integer range. A plain `JSON.parse` silently rounds these, which would break every
 * cross-reference (net.members → pin.id, component.from_block → block.id). So EAK ids are carried
 * as `string` here, and the fixture loader (see events/parse.ts) quotes long integer runs BEFORE
 * parsing. Timestamps/seqs/counts are small and stay numbers.
 */

export type EntityId = string; // a decimal-string u128 — never a JS number
export type Seq = number;

// ── Units ────────────────────────────────────────────────────────────────────────────────────
export interface PhysicalQuantity {
  magnitude: number;
  unit: string; // "Millimetre" | "Watt" | "Ampere" | "Ohm" | ...
  tolerance: unknown; // "None" | { ... } — not surfaced yet
}

// ── Enums (serde serializes unit variants as their name string) ────────────────────────────────
export type RequirementCategory =
  | "Functional"
  | "Electrical"
  | "Mechanical"
  | "Thermal"
  | "Regulatory"
  | "Fabrication"
  | "Cost"
  | "Schedule";
export type Priority = "High" | "Medium" | "Low";
export type RequirementStatus = "Proposed" | "Accepted" | "Satisfied" | "Violated" | "Waived";
export type ComponentClass = "Connector" | "Regulator" | "Ic" | "Resistor" | "Capacitor";
export type PinElectricalType =
  | "PowerIn"
  | "PowerOut"
  | "Input"
  | "Output"
  | "Bidirectional"
  | "Passive"
  | "Ground"
  | "NoConnect";
export type NetClass = "Power" | "Ground" | "Signal";
export type PartLifecycle = "Active" | "Nrnd" | "Eol";
export type BoardSide = "Top" | "Bottom";
export type ViolationSeverity = "Error" | "Warning" | "Info";
export type ViolationStatus = "Open" | "Waived" | "Resolved";
export type AssumptionCriticality = "Critical" | "Normal";
export type AssumptionStatus = "Open" | "Discharged" | "Invalidated";
export type FidelityMethod =
  | "Assumed"
  | "FirstOrderFloor"
  | "Calculated"
  | "Simulated"
  | "Measured";
export type RiskLikelihood = "Low" | "Medium" | "High";
export type RiskSeverity = "Low" | "Medium" | "High" | "Critical";
export type RiskStatus = "Open" | "Mitigated" | "Accepted";

// ── Entities ───────────────────────────────────────────────────────────────────────────────────
export interface DesignIntent {
  id: EntityId;
  statement: string;
  structured_summary: string;
  source: string;
}
export interface Requirement {
  id: EntityId;
  statement: string;
  category: RequirementCategory;
  priority: Priority;
  acceptance_criterion: string;
  status: RequirementStatus;
  source: EntityId;
  targets: PhysicalQuantity[];
}
export interface Constraint {
  id: EntityId;
  statement: string;
  subject_requirement: EntityId;
  kind: "Max" | "Min" | "Equal";
  bound: PhysicalQuantity;
  source: EntityId;
  status: "Active" | "Superseded";
}
export interface Violation {
  id: EntityId;
  rule: string;
  severity: ViolationSeverity;
  subjects: EntityId[];
  message: string;
  status: ViolationStatus;
}
export interface Waiver {
  id: EntityId;
  violation: EntityId;
  justification: string;
  decided_by: string;
}
export interface FunctionalBlock {
  id: EntityId;
  name: string;
  function: string;
  requirements: EntityId[];
}
export interface Component {
  id: EntityId;
  refdes: string;
  class: ComponentClass;
  value: PhysicalQuantity | null;
  from_block: EntityId;
  origin: "Synthesized" | "Imported";
}
export interface Pin {
  id: EntityId;
  component: EntityId;
  designation: string;
  electrical_type: PinElectricalType;
}
export interface Net {
  id: EntityId;
  name: string;
  class: NetClass;
  members: EntityId[];
  current: PhysicalQuantity | null;
  impedance_target: PhysicalQuantity | null;
  origin: "Logical" | "Physical";
}
export interface Part {
  id: EntityId;
  mpn: string;
  manufacturer: string;
  lifecycle: PartLifecycle;
  datasheet: string;
}
export interface BomLineItem {
  id: EntityId;
  part: EntityId;
  components: EntityId[];
  quantity: number;
}
export interface Layer {
  role: "Signal" | "Plane";
  copper_thickness: PhysicalQuantity;
  dielectric_height: PhysicalQuantity;
  dielectric_er: number;
  loss_tangent: number;
}
export interface Board {
  id: EntityId;
  width: PhysicalQuantity;
  height: PhysicalQuantity;
  stack: { layers: Layer[] };
}
export interface Placement {
  id: EntityId;
  component: EntityId;
  x: PhysicalQuantity;
  y: PhysicalQuantity;
  width: PhysicalQuantity;
  height: PhysicalQuantity;
  side: BoardSide;
}
export interface Track {
  id: EntityId;
  net: EntityId;
  layer: BoardSide;
  width: PhysicalQuantity;
  x1: PhysicalQuantity;
  y1: PhysicalQuantity;
  x2: PhysicalQuantity;
  y2: PhysicalQuantity;
}
export interface Discharge {
  resolution: "GroundedFact" | "EnforcedConstraint" | "AcceptedRisk";
  target: EntityId;
  decided_by: string;
}
export interface Assumption {
  id: EntityId;
  statement: string;
  rests_on: EntityId;
  criticality: AssumptionCriticality;
  status: AssumptionStatus;
  discharge: Discharge | null;
}
export interface ModelFidelity {
  concern: string;
  method: FidelityMethod;
  confidence: number;
  scope: string;
}
export interface Risk {
  id: EntityId;
  statement: string;
  likelihood: RiskLikelihood;
  severity: RiskSeverity;
  mitigation: string;
  residual: RiskSeverity;
  owner: string;
  status: RiskStatus;
}
export interface Objective {
  id: EntityId;
  statement: string;
  weight: number;
  source: EntityId;
}
export interface Alternative {
  label: string;
  description: string;
  scores: number[];
  rejected: boolean;
}
export interface Tradeoff {
  id: EntityId;
  question: string;
  alternatives: Alternative[];
  criteria: string[];
  chosen: number;
  rationale: string;
  decided_by: string;
}
export interface ProvenanceLink {
  id: EntityId;
  from: EntityId;
  to: EntityId;
  relation:
    | "DerivedFrom"
    | "JustifiedBy"
    | "BasedOnReasoning"
    | "Supports"
    | "TracesTo"
    | "Supersedes";
}
export interface ReasoningRequest {
  model_id: string;
  system: string;
  prompt: string;
  schema_name: string;
  temperature: number;
  seed: number;
}
export interface ReasoningResponse {
  candidates: unknown[];
  explanations?: unknown[];
  part_candidates?: unknown[];
  clarifying_questions?: string[];
  raw?: string;
}

// ── The Event union (serde: struct variants are `{ VariantName: {…fields} }`) ─────────────────
export type Event =
  | { PhaseEntered: { phase: string; state: string } }
  | { PhaseStateChanged: { phase: string; from: string; to: string } }
  | { PhaseCompleted: { phase: string; outcome: string } }
  | { PhaseFailed: { phase: string; reason: string } }
  | { ReasoningCall: { request: ReasoningRequest; response: ReasoningResponse } }
  | { IntentCaptured: { intent: DesignIntent } }
  | { EvidenceReferenced: { evidence: unknown } }
  | { DecisionCreated: { decision: unknown } }
  | { RequirementCommitted: { requirement: Requirement } }
  | { ProvenanceLinked: { link: ProvenanceLink } }
  | { ConstraintCommitted: { constraint: Constraint } }
  | { ViolationRaised: { violation: Violation } }
  | { WaiverGranted: { waiver: Waiver } }
  | { ConstraintsExtracted: { count: number } }
  | { VerificationCompleted: { rule_count: number; open_violations: number } }
  | { FunctionalBlockCommitted: { block: FunctionalBlock } }
  | { ComponentCommitted: { component: Component } }
  | { PinCommitted: { pin: Pin } }
  | { NetCommitted: { net: Net } }
  | { PartCommitted: { part: Part } }
  | { BomLineItemCommitted: { item: BomLineItem } }
  | { RequirementIrProduced: { schema_version: number; requirement_count: number } }
  | { EngineeringIrProduced: { schema_version: number; block_count: number } }
  | { SchematicIrProduced: { schema_version: number; net_count: number } }
  | { BomIrProduced: { schema_version: number; line_item_count: number } }
  | { BoardCommitted: { board: Board } }
  | { PlacementCommitted: { placement: Placement } }
  | { TrackCommitted: { track: Track } }
  | { PcbIrProduced: { schema_version: number; placement_count: number } }
  | { PcbIrEnriched: { schema_version: number; track_count: number } }
  | {
      ManufacturingGenerated: {
        schema_version: number;
        place_count: number;
        copper_count: number;
        line_item_count: number;
      };
    }
  | {
      ViolationExplained: {
        violation: EntityId;
        explanation: string;
        suggested_fix: string;
        reasoning_call_seq: Seq;
      };
    }
  | { AssumptionRaised: { assumption: Assumption } }
  | { AssumptionDischarged: { assumption: EntityId; discharge: Discharge } }
  | {
      FidelityTagged: {
        target: EntityId;
        fidelity: ModelFidelity;
        reasoning_call_seq: Seq | null;
      };
    }
  | { RiskRaised: { risk: Risk } }
  | { RiskAccepted: { risk: EntityId; accepted_by: string } }
  | { ObjectiveRecorded: { objective: Objective } }
  | { TradeoffRecorded: { tradeoff: Tradeoff } };

export interface EventRecord {
  seq: Seq;
  timestamp: number;
  event: Event;
}

/** The variant name of an event — the discriminator serde emits as the sole object key. */
export function eventKind(ev: Event): string {
  return Object.keys(ev)[0];
}
