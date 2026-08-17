//! Ports (contracts) for Electronics Agent Kit — Use-case ring.
//!
//! Inner rings DEFINE these contracts; outer-ring adapters IMPLEMENT them
//! (`docs/core/contracts.md`, P1). Phase 1 exposes the two adapter boundaries —
//! the [`EventLog`] (implemented by `eak-store`) and the [`ReasoningEngine`]
//! (implemented by `eak-reasoning`) — plus the [`Event`] type they carry.
//!
//! The kernel-internal protocol surfaces (AgentContext, FSM framework, capability
//! handlers) live in `eak-runtime`: they are not implemented by outer adapters, so by
//! the "a contract lives with the ring that needs it" rule they belong to the kernel.

use eak_domain::{
    Assumption, Board, BomLineItem, ClockDomain, Component, Constraint, Decision, DesignIntent,
    Discharge, Evidence, FunctionalBlock, ModelFidelity, Net, Objective, Part, Pin, PinAssignment,
    PinCapability, Placement, PowerDomain, Priority, ProvenanceLink, Requirement,
    RequirementCategory, ReturnPath, Risk, Signal, Track, Tradeoff, Violation, Waiver,
};
use eak_units::PhysicalQuantity;
use serde::{Deserialize, Serialize};

/// Monotonic event sequence number — the address of a fact in history.
pub type Seq = u64;

/// Wall-clock instant (unix epoch milliseconds). Recorded in every event and never
/// re-read on replay (determinism, P4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub i64);

// ===================== Event-log boundary (impl: eak-store) =====================

/// An event with its assigned position and recorded time. The unit of provenance and
/// the basis of deterministic replay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    pub seq: Seq,
    pub timestamp: Timestamp,
    pub event: Event,
}

/// Every design-significant change in Phase 1. Entity-bearing variants are *state
/// deltas* (folded into Engineering State); the rest are audit/provenance markers.
/// Per P5 every committed transition emits at least one event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    // ---- phase lifecycle (audit) ----
    PhaseEntered {
        phase: String,
        state: String,
    },
    PhaseStateChanged {
        phase: String,
        from: String,
        to: String,
    },
    PhaseCompleted {
        phase: String,
        outcome: String,
    },
    PhaseFailed {
        phase: String,
        reason: String,
    },

    // ---- reasoning boundary (provenance) ----
    ReasoningCall {
        request: ReasoningRequest,
        response: ReasoningResponse,
    },

    // ---- state deltas ----
    IntentCaptured {
        intent: DesignIntent,
    },
    EvidenceReferenced {
        evidence: Evidence,
    },
    DecisionCreated {
        decision: Decision,
    },
    RequirementCommitted {
        requirement: Requirement,
    },
    ProvenanceLinked {
        link: ProvenanceLink,
    },

    // ---- Phase 2: verification state deltas ----
    ConstraintCommitted {
        constraint: Constraint,
    },
    ViolationRaised {
        violation: Violation,
    },
    WaiverGranted {
        waiver: Waiver,
    },

    // ---- Phase 2: verification milestones (audit) ----
    ConstraintsExtracted {
        count: usize,
    },
    VerificationCompleted {
        rule_count: usize,
        /// Open, blocking violations scoped to *this phase's own rules* (not the global
        /// violation count). Audit-only — not folded into `EngineeringState`.
        open_violations: usize,
    },

    // ---- Phase 3: synthesis state deltas ----
    FunctionalBlockCommitted {
        block: FunctionalBlock,
    },
    ComponentCommitted {
        component: Component,
    },
    PinCommitted {
        pin: Pin,
    },
    NetCommitted {
        net: Net,
    },

    // ---- Phase 3 (BOM): bill-of-materials state deltas ----
    PartCommitted {
        part: Part,
    },
    BomLineItemCommitted {
        item: BomLineItem,
    },

    // ---- IR boundary milestones (audit) ----
    RequirementIrProduced {
        schema_version: u32,
        requirement_count: usize,
    },
    EngineeringIrProduced {
        schema_version: u32,
        block_count: usize,
    },
    SchematicIrProduced {
        schema_version: u32,
        net_count: usize,
    },
    BomIrProduced {
        schema_version: u32,
        line_item_count: usize,
    },

    // ---- Phase 3 (PCB): layout state deltas ----
    BoardCommitted {
        board: Board,
    },
    PlacementCommitted {
        placement: Placement,
    },

    // ---- Phase 3 (routing): routing state delta ----
    TrackCommitted {
        track: Track,
    },

    // ---- Phase 3 (PCB): IR boundary milestone (audit) ----
    PcbIrProduced {
        schema_version: u32,
        placement_count: usize,
    },

    // ---- Phase 3 (routing): IR enrichment milestone (audit) ----
    PcbIrEnriched {
        schema_version: u32,
        track_count: usize,
    },

    // ---- Phase 3 (manufacturing): terminal IR / release milestone (audit) ----
    ManufacturingGenerated {
        schema_version: u32,
        place_count: usize,
        copper_count: usize,
        line_item_count: usize,
    },

    // ---- E6 (C1): AI review explainer — advisory-only metadata ----
    /// A plain-English explanation + suggested fix for one raised [`Violation`], proposed through
    /// the reasoning boundary (`violation_explanation_v1`) and committed as ADVISORY metadata.
    ///
    /// INVARIANT (advisory-only, security-critical): this event NEVER changes the violation's
    /// `severity`/`status`/validity, NEVER gates a phase, and NEVER drives a kernel mutation. The
    /// committed kernel data *grounds* the text; the model only *describes* it. It folds into a
    /// SEPARATE advisory store (`EngineeringState::violation_explanations`), never into the
    /// [`Violation`] itself, so no reasoning output can ever reach an engineering decision (P3).
    /// `reasoning_call_seq` points back at the exact [`Event::ReasoningCall`] that produced the
    /// text (provenance-by-construction); `violation` links it to the subject it explains.
    ViolationExplained {
        violation: eak_domain::EntityId,
        explanation: String,
        suggested_fix: String,
        reasoning_call_seq: Seq,
    },

    // ---- Phase 3 (Band A): epistemic state deltas ----
    /// A first-class presumption the reasoning declared, made auditable (Map 10). A state
    /// delta: the fold pushes the [`Assumption`] into `EngineeringState::assumptions`. A
    /// Critical + Open one blocks release at the honesty gate (P4-folded, gate-read).
    AssumptionRaised {
        assumption: Assumption,
    },
    /// An [`Assumption`] was discharged. A state delta: the fold finds it by id, flips its
    /// `status` to `Discharged`, and records the [`Discharge`] on it.
    AssumptionDischarged {
        assumption: eak_domain::EntityId,
        discharge: Discharge,
    },

    // ---- Band A (increment 2): fidelity tag — advisory-only metadata ----
    /// A trust-tag ([`ModelFidelity`]) attached to a derived/predicted fact (Map 6), committed
    /// as ADVISORY metadata through the audit seam — exactly like [`Event::ViolationExplained`].
    ///
    /// INVARIANT (advisory-only, structural): the fold pushes into a SEPARATE store
    /// (`EngineeringState::fidelity_tags`), keyed by `target`; it NEVER mutates the tagged entity,
    /// NEVER gates a phase, and NEVER drives a kernel mutation (P3). `target` is the entity the tag
    /// describes; `reasoning_call_seq`, when present, points back at the [`Event::ReasoningCall`]
    /// that produced the tag (provenance-by-construction), and is `None` when the runtime tags a
    /// fact itself (e.g. a first-order floor).
    FidelityTagged {
        target: eak_domain::EntityId,
        fidelity: ModelFidelity,
        reasoning_call_seq: Option<Seq>,
    },

    // ---- Band A (increment 3): risk posture — tracked truth (Map 46) ----
    /// A first-class [`Risk`] was raised, made auditable (Map 46). A state delta: the fold
    /// pushes the risk into `EngineeringState::risks`. Risk is TRACKED TRUTH — it does NOT
    /// block release in v0; the human owns acceptance of residual risk (`00` Principle 11).
    RiskRaised {
        risk: Risk,
    },
    /// A [`Risk`] was accepted by a named human. A state delta: the fold finds it by id and
    /// flips its `status` to `Accepted` (human authority, Principle 11). `accepted_by` names
    /// the decider (like [`Waiver::decided_by`], Principle 10).
    RiskAccepted {
        risk: eak_domain::EntityId,
        accepted_by: String,
    },

    // ---- Band A (increment 4): objective / tradeoff — the weighed-and-rejected space (Map 11) ----
    /// A first-class [`Objective`] (a weighted design goal) was recorded. A state delta: the fold
    /// pushes the objective into `EngineeringState::objectives`.
    ObjectiveRecorded {
        objective: Objective,
    },
    /// A first-class [`Tradeoff`] was recorded, PRESERVING its rejected space (`00` Principle 7,
    /// exit criterion 3). A state delta: the fold pushes it into `EngineeringState::tradeoffs`. A
    /// [`Decision`] may later cite the tradeoff it resolved via a [`ProvenanceLink`].
    TradeoffRecorded {
        tradeoff: Tradeoff,
    },

    // ---- Band B (Phase 5, increment 1): power architecture — state delta (Map 38) ----
    /// A first-class [`PowerDomain`] (a named power rail) was committed (Band B). A state delta:
    /// the fold pushes the domain into `EngineeringState::power_domains`. The domain names the
    /// [`Net`]s it must hold at `voltage` and the `max_current` its source component can deliver;
    /// whether the load the nets carry exceeds that budget is the [`PowerBalanceRule`]'s judgement
    /// at ERC time, not this commit's (a well-formed but overloaded rail must enter state so the
    /// rule can say so).
    PowerDomainCommitted {
        domain: PowerDomain,
    },
    // ---- Band B (Phase 5, increment 2): clock architecture — state delta (Map 21) ----
    /// A first-class [`ClockDomain`] (a named clock region) was committed (Band B). A state delta:
    /// the fold pushes the domain into `EngineeringState::clock_domains`. The domain names the
    /// [`Net`]s synchronous to a `frequency` sourced by a component; whether a net belongs to two
    /// domains is the [`ClockDomainMembershipRule`]'s judgement at ERC time, not this commit's (a
    /// well-formed domain must enter state so the rule can reason over the crossing).
    ClockDomainCommitted {
        domain: ClockDomain,
    },
    // ---- Band B (Phase 5, increment 3): return-path architecture — state delta (Map 20) ----
    /// A first-class [`ReturnPath`] (the declared return conductor for a controlled net) was
    /// committed (Band B). A state delta: the fold pushes the path into
    /// `EngineeringState::return_paths`. The path names the reference net a controlled net's return
    /// current flows on; whether the controlled net is under-specified (declares an impedance but no
    /// return) is the [`ReturnPathRule`]'s judgement at ERC time, not this commit's — a well-formed
    /// path must enter state so the rule can reason over the return architecture.
    ReturnPathCommitted {
        path: ReturnPath,
    },
    /// A [`PinCapability`] was committed (Band B inc 4): a physical pin's datasheet truth — the set
    /// of mux functions it can carry. Owned, never fabricated (P7). The capability's link to a real
    /// pin was re-checked at the seam; whether an *assignment* honors it is the
    /// [`PinCapabilityRule`]'s judgement at ERC time.
    PinCapabilityCommitted {
        capability: PinCapability,
    },
    /// A [`PinAssignment`] was committed (Band B inc 4): the mux function the design assigns to a
    /// physical pin. The pin link was re-checked at the seam; whether the assignment conflicts with
    /// another on the same pin (`erc-pin-mux-conflict`) or honors the pin's capability
    /// (`erc-pin-capability`) is the rules' judgement at ERC time — a well-formed assignment must
    /// enter state so the conflicts are *reported*, not silently swallowed (master-prompt §31).
    PinAssignmentCommitted {
        assignment: PinAssignment,
    },
    /// A [`Signal`] was committed (Band B inc 5): a named, *directional* logical signal flow
    /// (source → sinks) with a meaning — the schematic's logical layer above the undirected copper
    /// of a [`Net`] (Map 16). The source and every sink were re-checked at the seam to be committed
    /// pins; whether the flow is *legal* (an output/bidirectional source, every sink
    /// input/bidirectional) is the [`SignalDriverSinkRule`]'s judgement at ERC time.
    SignalCommitted {
        signal: Signal,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    Io(String),
    Serialization(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(m) => write!(f, "store io error: {m}"),
            StoreError::Serialization(m) => write!(f, "store serialization error: {m}"),
        }
    }
}
impl std::error::Error for StoreError {}

/// Append-only, ordered event log (event-sourcing, ADR-0004). The single source of
/// truth; state is its fold. Implemented by `eak-store`.
pub trait EventLog {
    /// Append timestamped events atomically; assigns consecutive [`Seq`]s and returns them.
    fn append(&mut self, events: &[(Timestamp, Event)]) -> Result<Vec<Seq>, StoreError>;
    /// Read the full ordered history (basis of replay and provenance).
    fn read_all(&self) -> Result<Vec<EventRecord>, StoreError>;
    /// The next sequence number that would be assigned.
    fn next_seq(&self) -> Seq;
}

/// A live observer of committed events — the seam a streaming consumer (e.g. a desktop UI)
/// subscribes to. The runtime invokes it once per event, in commit order, immediately after the
/// event has been appended to the [`EventLog`] and folded into state, so an observer sees each
/// change *as it happens* rather than only at the end of a run. A sink is a pure side-observer:
/// it must not mutate engineering state, and the runtime owns at most one — absent a sink the
/// behavior is exactly as before, so determinism and replay are untouched (P4). Fire-and-forget
/// by design: a sink that needs to hand events to another thread should enqueue them (e.g. onto a
/// channel) and return promptly rather than block the commit path.
pub trait EventSink {
    /// Invoked after `record`'s event has been committed (appended + folded).
    fn on_committed(&mut self, record: &EventRecord);
}

// ===================== Reasoning boundary (impl: eak-reasoning) =====================

/// A structured request for judgement. The prompt is *data the runtime composes*; the
/// schema names the shape the answer must take. `seed`/`temperature`/`model_id` are the
/// decisive parameters recorded for reproducibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningRequest {
    pub model_id: String,
    pub system: String,
    pub prompt: String,
    pub schema_name: String,
    pub temperature: f64,
    pub seed: u64,
}

/// One candidate requirement proposed by the reasoning engine — *judgement only*, not
/// yet validated or committed (the seam is in the agent's deterministic half, P3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateRequirement {
    pub statement: String,
    pub category: RequirementCategory,
    pub priority: Priority,
    pub acceptance_criterion: String,
    pub source_hint: String,
    pub confidence: f64,
    pub rationale: String,
    #[serde(default)]
    pub targets: Vec<PhysicalQuantity>,
}

/// One advisory explanation of a raised [`Violation`] proposed by the reasoning engine under the
/// `violation_explanation_v1` schema — *judgement only*. It is NOT a domain entity and is NEVER
/// validated at a capability seam: it is stored verbatim as advisory metadata (see
/// [`Event::ViolationExplained`]) and can never gate a phase or alter the violation it describes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CandidateExplanation {
    /// Plain-English account of what the violation means and why it was raised.
    pub explanation: String,
    /// A concrete, advisory suggestion for how an engineer might resolve it.
    pub suggested_fix: String,
}

/// One candidate part selection proposed by the reasoning engine under the `part_candidates_v1`
/// schema (E6 C2) — *judgement only*, exactly like [`CandidateRequirement`]. The model names the
/// manufacturer part number it believes realizes a given component class; that claim is UNTRUSTED.
/// The deterministic half of the part-selection agent re-validates every proposal against the
/// authoritative `PartCatalog` (in `eak-engines`) and REJECTS any MPN the catalog does not carry
/// for that class — a rejected proposal is never committed and never enters state (the seam, P3).
/// The committed [`Part`] is always built from the trusted catalog record, so the model's free
/// text can never reach engineering state even when its proposal is accepted; it only *chooses*
/// among catalogued parts, it does not *author* one.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CandidatePart {
    /// The component class this proposal is for (e.g. "Ic", "Connector", "Regulator"). Matched
    /// against the schematic's classes case-insensitively; an unrecognized class is simply ignored.
    pub component_class: String,
    /// The manufacturer part number the model proposes for that class — the claim the kernel checks
    /// against the catalog. A non-catalog MPN here is the canonical rejected proposal.
    pub mpn: String,
    /// Advisory rationale for the choice (recorded for the reasoning/traceability panel).
    #[serde(default)]
    pub rationale: String,
    /// The model's self-reported confidence in the selection (advisory).
    #[serde(default)]
    pub confidence: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReasoningResponse {
    pub candidates: Vec<CandidateRequirement>,
    /// Advisory violation explanations (E6 C1). Additive and defaulted, so a
    /// `requirement_candidates_v1` response that omits it deserializes unchanged.
    #[serde(default)]
    pub explanations: Vec<CandidateExplanation>,
    /// Candidate part selections (E6 C2, `part_candidates_v1`). Additive and defaulted, so a
    /// requirement or explanation response that omits it deserializes unchanged.
    #[serde(default)]
    pub part_candidates: Vec<CandidatePart>,
    #[serde(default)]
    pub clarifying_questions: Vec<String>,
    #[serde(default)]
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReasoningError {
    Provider(String),
    Schema(String),
    Unavailable,
}

impl std::fmt::Display for ReasoningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasoningError::Provider(m) => write!(f, "reasoning provider error: {m}"),
            ReasoningError::Schema(m) => write!(f, "reasoning schema violation: {m}"),
            ReasoningError::Unavailable => write!(f, "reasoning engine unavailable"),
        }
    }
}
impl std::error::Error for ReasoningError {}

/// The single boundary to stochastic judgement (P3). Implemented by `eak-reasoning`
/// (fixture + live Anthropic). Phase 1 uses the synchronous request form; the spec's
/// stream/cancel operations are deferred.
pub trait ReasoningEngine {
    /// Stable identifier of the engine/model in use (e.g. `"fixture"` or
    /// `"anthropic:claude-opus-4-8"`), recorded with each call for reproducibility.
    fn model_id(&self) -> String;
    fn request_judgement(
        &self,
        req: &ReasoningRequest,
    ) -> Result<ReasoningResponse, ReasoningError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use eak_domain::{EntityId, Priority, RequirementCategory, RequirementStatus};

    #[test]
    fn event_roundtrips_through_json() {
        let ev = Event::RequirementCommitted {
            requirement: Requirement {
                id: EntityId(7),
                statement: "Operating power shall not exceed 5 W".into(),
                category: RequirementCategory::Electrical,
                priority: Priority::High,
                acceptance_criterion: "measured power < 5 W".into(),
                status: RequirementStatus::Accepted,
                source: EntityId(1),
                targets: vec![PhysicalQuantity::new(5.0, eak_units::Unit::Watt)],
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band A (increment 1): Assumption events =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its
    // on-disk form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn assumption_raised_event_roundtrips_through_json() {
        use eak_domain::{Assumption, AssumptionCriticality, AssumptionStatus, EntityId};
        let ev = Event::AssumptionRaised {
            assumption: Assumption {
                id: EntityId(9),
                statement: "the USB-C source can deliver 5 V @ 3 A".into(),
                rests_on: EntityId(3),
                criticality: AssumptionCriticality::Critical,
                status: AssumptionStatus::Open,
                discharge: None,
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn assumption_discharged_event_roundtrips_through_json() {
        use eak_domain::{Discharge, DischargeResolution, EntityId};
        let ev = Event::AssumptionDischarged {
            assumption: EntityId(9),
            discharge: Discharge {
                resolution: DischargeResolution::EnforcedConstraint,
                target: EntityId(4),
                decided_by: "engineer".into(),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band A (increment 3): Risk events =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn risk_raised_event_roundtrips_through_json() {
        use eak_domain::{EntityId, Risk, RiskLikelihood, RiskSeverity, RiskStatus};
        let ev = Event::RiskRaised {
            risk: Risk {
                id: EntityId(11),
                statement: "an ESD strike on the USB-C connector could latch up the MCU".into(),
                likelihood: RiskLikelihood::Medium,
                severity: RiskSeverity::High,
                mitigation: "add a TVS diode on the USB-C VBUS".into(),
                residual: RiskSeverity::Low,
                owner: "hardware lead".into(),
                status: RiskStatus::Open,
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn risk_accepted_event_roundtrips_through_json() {
        use eak_domain::EntityId;
        let ev = Event::RiskAccepted {
            risk: EntityId(11),
            accepted_by: "hardware lead".into(),
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band A (increment 4): Objective / Tradeoff events =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4). The `Tradeoff` event carries the
    // preserved rejected space (Map 11, exit criterion 3), so the round-trip proves that space
    // survives serialization intact.

    #[test]
    fn objective_recorded_event_roundtrips_through_json() {
        use eak_domain::{EntityId, Objective};
        let ev = Event::ObjectiveRecorded {
            objective: Objective {
                id: EntityId(19),
                statement: "minimize board area".into(),
                weight: 0.7,
                source: EntityId(3),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn tradeoff_recorded_event_roundtrips_through_json() {
        use eak_domain::{Alternative, EntityId, Tradeoff};
        let ev = Event::TradeoffRecorded {
            tradeoff: Tradeoff {
                id: EntityId(20),
                question: "Which regulator topology for the 3.3 V rail?".into(),
                alternatives: vec![
                    Alternative {
                        label: "buck".into(),
                        description: "switching buck converter".into(),
                        scores: vec![0.9, 0.6],
                        rejected: false,
                    },
                    Alternative {
                        label: "ldo".into(),
                        description: "linear low-dropout regulator".into(),
                        scores: vec![0.4, 0.9],
                        rejected: true,
                    },
                ],
                criteria: vec!["efficiency".into(), "simplicity".into()],
                chosen: 0,
                rationale: "efficiency dominates at this load".into(),
                decided_by: "hardware lead".into(),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band B (increment 1): PowerDomain event =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn power_domain_committed_event_roundtrips_through_json() {
        use eak_domain::{EntityId, PowerDomain};
        use eak_units::{PhysicalQuantity, Unit};
        let ev = Event::PowerDomainCommitted {
            domain: PowerDomain {
                id: EntityId(30),
                name: "3V3".into(),
                voltage: PhysicalQuantity::new(3.3, Unit::Volt),
                source_component: EntityId(4),
                max_current: PhysicalQuantity::new(1.0, Unit::Ampere),
                nets: vec![EntityId(31), EntityId(32)],
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band B (increment 2): ClockDomain event =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn clock_domain_committed_event_roundtrips_through_json() {
        use eak_domain::{ClockDomain, EntityId};
        use eak_units::{PhysicalQuantity, Unit};
        let ev = Event::ClockDomainCommitted {
            domain: ClockDomain {
                id: EntityId(40),
                name: "SYS".into(),
                frequency: PhysicalQuantity::new(48.0, Unit::Megahertz),
                source_component: EntityId(4),
                members: vec![EntityId(41), EntityId(42)],
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band B (increment 3): ReturnPath event =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn return_path_committed_event_roundtrips_through_json() {
        use eak_domain::{EntityId, ReturnPath};
        let ev = Event::ReturnPathCommitted {
            path: ReturnPath {
                id: EntityId(50),
                name: "SYS_CLK ret on GND".into(),
                net: EntityId(41),
                reference_plane: EntityId(60),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band B (increment 4): Pin-Function / Mux events =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn pin_capability_committed_event_roundtrips_through_json() {
        use eak_domain::{EntityId, PinCapability};
        let ev = Event::PinCapabilityCommitted {
            capability: PinCapability {
                id: EntityId(70),
                pin: EntityId(71),
                functions: vec!["SPI1_MOSI".into(), "UART1_TX".into()],
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn pin_assignment_committed_event_roundtrips_through_json() {
        use eak_domain::{EntityId, PinAssignment};
        let ev = Event::PinAssignmentCommitted {
            assignment: PinAssignment {
                id: EntityId(72),
                pin: EntityId(71),
                function: "SPI1_MOSI".into(),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band B (increment 5): Signal event =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn signal_committed_event_roundtrips_through_json() {
        use eak_domain::{EntityId, Signal};
        let ev = Event::SignalCommitted {
            signal: Signal {
                id: EntityId(80),
                name: "SYS_CLK".into(),
                source: EntityId(81),
                sinks: vec![EntityId(82), EntityId(83)],
                semantics: "system clock".into(),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }
}
