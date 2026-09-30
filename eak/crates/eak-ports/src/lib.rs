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
    Assumption, Board, BomLineItem, Bus, ClockDomain, Component, Constraint, Contract, Decision,
    DesignIntent, Discharge, Evidence, FunctionalBlock, Interface, ModelFidelity, Net, Objective,
    Part, Pin, PinAssignment, PinCapability, Placement, PowerDomain, Priority, ProvenanceLink,
    Requirement, RequirementCategory, ReturnPath, Risk, Signal, Subsystem, Track, Tradeoff,
    Violation, Waiver,
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
    /// A [`Contract`] was committed (Band B inc 6): a protocol rule-set (e.g. "I²C", "SPI") that
    /// governs an interface. The protocol name is open-ended (String) — an enum would fabricate a
    /// closed world (P7).
    ContractCommitted {
        contract: Contract,
    },
    /// An [`Interface`] was committed (Band B inc 6): a named collection of signals governed by a
    /// contract. The signals and contract were re-checked at the seam to be committed; whether the
    /// interface satisfies the contract (correct signals, directions, protocol rules) is the
    /// [`InterfaceContractRule`]'s judgement at ERC time.
    InterfaceCommitted {
        interface: Interface,
    },
    /// A [`Bus`] was committed (Band B inc 7): a collection of interfaces (or signals) sharing a
    /// physical bus line under one protocol contract, with a declared topology. The contract and
    /// all members were re-checked at the seam to be committed; whether the bus's topology
    /// satisfies the protocol's structural rules (unique addresses, termination, fan-out) is the
    /// [`BusTopologyRule`]'s judgement at ERC time.
    BusCommitted {
        bus: Bus,
    },
    /// A [`Subsystem`] was committed (Band B inc 8): a hierarchical grouping of blocks exposing
    /// interfaces as its boundary — the unit of reuse and reasoning at scale (Map 14). The blocks
    /// and interfaces were re-checked at the seam to be committed; whether the subsystem's
    /// boundary is *complete* (every cross-boundary pin is exposed) is the
    /// [`SubsystemBoundaryRule`]'s judgement at ERC time.
    SubsystemCommitted {
        subsystem: Subsystem,
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

/// Provider identifier (e.g., "anthropic", "openai", "ollama").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderId(pub String);

impl ProviderId {
    pub fn anthropic() -> ProviderId {
        ProviderId("anthropic".to_string())
    }
    pub fn openai() -> ProviderId {
        ProviderId("openai".to_string())
    }
    pub fn ollama() -> ProviderId {
        ProviderId("ollama".to_string())
    }
    pub fn gemini() -> ProviderId {
        ProviderId("gemini".to_string())
    }
    pub fn kimi() -> ProviderId {
        ProviderId("kimi".to_string())
    }
    pub fn nvidia() -> ProviderId {
        ProviderId("nvidia".to_string())
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ProviderId {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ProviderId(s.to_string()))
    }
}

/// Model identifier (e.g., "claude-opus-4", "gpt-4o", "llama3.1").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ModelId {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ModelId(s.to_string()))
    }
}

/// Capabilities a model may support. Used for capability-based routing and validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    /// Basic text generation (completion/chat)
    TextGeneration,
    /// Streaming response chunks
    Streaming,
    /// Function/tool calling with structured arguments
    ToolCalling,
    /// Structured output (JSON schema constrained)
    StructuredOutput,
    /// Vision/multimodal input (images, etc.)
    Vision,
    /// Audio input/output
    Audio,
    /// Reasoning/effort parameter (e.g., o-series, Claude thinking)
    ReasoningEffort,
    /// System prompt/instructions support
    SystemInstructions,
    /// Parallel tool calls
    ParallelToolCalls,
}

impl ModelCapability {
    /// All known capabilities for iteration.
    pub const ALL: &[ModelCapability] = &[
        ModelCapability::TextGeneration,
        ModelCapability::Streaming,
        ModelCapability::ToolCalling,
        ModelCapability::StructuredOutput,
        ModelCapability::Vision,
        ModelCapability::Audio,
        ModelCapability::ReasoningEffort,
        ModelCapability::SystemInstructions,
        ModelCapability::ParallelToolCalls,
    ];
}

/// A set of model capabilities, stored as a bitflag for efficient checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapabilitySet(u16);

impl CapabilitySet {
    pub const EMPTY: CapabilitySet = CapabilitySet(0);
    pub const TEXT_GENERATION: CapabilitySet = CapabilitySet(1 << 0);
    pub const STREAMING: CapabilitySet = CapabilitySet(1 << 1);
    pub const TOOL_CALLING: CapabilitySet = CapabilitySet(1 << 2);
    pub const STRUCTURED_OUTPUT: CapabilitySet = CapabilitySet(1 << 3);
    pub const VISION: CapabilitySet = CapabilitySet(1 << 4);
    pub const AUDIO: CapabilitySet = CapabilitySet(1 << 5);
    pub const REASONING_EFFORT: CapabilitySet = CapabilitySet(1 << 6);
    pub const SYSTEM_INSTRUCTIONS: CapabilitySet = CapabilitySet(1 << 7);
    pub const PARALLEL_TOOL_CALLS: CapabilitySet = CapabilitySet(1 << 8);

    pub fn contains(self, cap: ModelCapability) -> bool {
        (self.0 & CapabilitySet::from(cap).0) != 0
    }

    pub fn insert(&mut self, cap: ModelCapability) {
        self.0 |= CapabilitySet::from(cap).0;
    }

    pub fn remove(&mut self, cap: ModelCapability) {
        self.0 &= !CapabilitySet::from(cap).0;
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = ModelCapability> {
        ModelCapability::ALL
            .iter()
            .copied()
            .filter(move |c| self.contains(*c))
    }
}

impl std::iter::FromIterator<ModelCapability> for CapabilitySet {
    fn from_iter<T: IntoIterator<Item = ModelCapability>>(iter: T) -> Self {
        let mut set = CapabilitySet::EMPTY;
        for cap in iter {
            set |= CapabilitySet::from(cap);
        }
        set
    }
}

impl From<ModelCapability> for CapabilitySet {
    fn from(cap: ModelCapability) -> Self {
        match cap {
            ModelCapability::TextGeneration => CapabilitySet::TEXT_GENERATION,
            ModelCapability::Streaming => CapabilitySet::STREAMING,
            ModelCapability::ToolCalling => CapabilitySet::TOOL_CALLING,
            ModelCapability::StructuredOutput => CapabilitySet::STRUCTURED_OUTPUT,
            ModelCapability::Vision => CapabilitySet::VISION,
            ModelCapability::Audio => CapabilitySet::AUDIO,
            ModelCapability::ReasoningEffort => CapabilitySet::REASONING_EFFORT,
            ModelCapability::SystemInstructions => CapabilitySet::SYSTEM_INSTRUCTIONS,
            ModelCapability::ParallelToolCalls => CapabilitySet::PARALLEL_TOOL_CALLS,
        }
    }
}

impl std::ops::BitOr for CapabilitySet {
    type Output = CapabilitySet;
    fn bitor(self, rhs: Self) -> Self::Output {
        CapabilitySet(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CapabilitySet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for CapabilitySet {
    type Output = CapabilitySet;
    fn bitand(self, rhs: Self) -> Self::Output {
        CapabilitySet(self.0 & rhs.0)
    }
}

/// Metadata describing a model's capabilities and limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub provider: ProviderId,
    pub model_id: ModelId,
    pub display_name: String,
    pub capabilities: CapabilitySet,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub supports_parallel_tool_calls: bool,
    /// Optional: pricing info (per 1M tokens) for cost estimation
    pub input_price_per_million: Option<f64>,
    pub output_price_per_million: Option<f64>,
}

/// Configuration for a model provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: ProviderId,
    pub name: String,
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub credential_ref: Option<CredentialRef>,
    pub default_model: Option<ModelId>,
    pub extra_headers: Option<std::collections::HashMap<String, String>>,
}

/// Configuration for a specific model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: ProviderId,
    pub model_id: ModelId,
    pub enabled: bool,
    pub capabilities: CapabilitySet,
    pub parameters: ModelParameters,
    pub metadata: Option<ModelMetadata>,
}

/// Runtime parameters for model inference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModelParameters {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub max_tokens: Option<u32>,
    pub reasoning_effort: Option<String>,
    pub stop_sequences: Option<Vec<String>>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub seed: Option<u64>,
}

/// Reference to a credential stored in a secure credential store.
/// Never contains the raw secret — only an opaque reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRef {
    pub store: String,
    pub key: String,
}

impl CredentialRef {
    pub fn new(store: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            store: store.into(),
            key: key.into(),
        }
    }
}

/// Provider registry for managing configured model providers.
/// Allows dynamic registration, lookup, and selection of providers.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProviderRegistry {
    providers: std::collections::HashMap<ProviderId, ProviderConfig>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    /// Register a provider configuration.
    pub fn register(&mut self, config: ProviderConfig) {
        self.providers.insert(config.id.clone(), config);
    }

    /// Remove a provider configuration.
    pub fn remove(&mut self, id: &ProviderId) -> Option<ProviderConfig> {
        self.providers.remove(id)
    }

    /// Get a provider configuration by ID.
    pub fn get(&self, id: &ProviderId) -> Option<&ProviderConfig> {
        self.providers.get(id)
    }

    /// Get a mutable reference to a provider configuration.
    pub fn get_mut(&mut self, id: &ProviderId) -> Option<&mut ProviderConfig> {
        self.providers.get_mut(id)
    }

    /// List all registered provider IDs.
    pub fn list(&self) -> Vec<&ProviderId> {
        self.providers.keys().collect()
    }

    /// List all registered provider configurations.
    pub fn list_configs(&self) -> Vec<&ProviderConfig> {
        self.providers.values().collect()
    }

    /// Enable a provider.
    pub fn enable(&mut self, id: &ProviderId) -> bool {
        if let Some(config) = self.providers.get_mut(id) {
            config.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a provider.
    pub fn disable(&mut self, id: &ProviderId) -> bool {
        if let Some(config) = self.providers.get_mut(id) {
            config.enabled = false;
            true
        } else {
            false
        }
    }

    /// Check if a provider is enabled.
    pub fn is_enabled(&self, id: &ProviderId) -> bool {
        self.providers.get(id).map(|c| c.enabled).unwrap_or(false)
    }

    /// Get the default model for a provider.
    pub fn default_model(&self, id: &ProviderId) -> Option<&ModelId> {
        self.providers
            .get(id)
            .and_then(|c| c.default_model.as_ref())
    }
}

/// Model registry for managing model metadata and capabilities.
/// Allows lookup of models by provider and model ID.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    models: std::collections::HashMap<String, ModelConfig>,
}

fn model_key(provider: &ProviderId, model_id: &ModelId) -> String {
    format!("{}:{}", provider.0, model_id.0)
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: std::collections::HashMap::new(),
        }
    }

    /// Register a model configuration.
    pub fn register(&mut self, config: ModelConfig) {
        let key = model_key(&config.provider, &config.model_id);
        self.models.insert(key, config);
    }

    /// Remove a model configuration.
    pub fn remove(&mut self, provider: &ProviderId, model_id: &ModelId) -> Option<ModelConfig> {
        self.models.remove(&model_key(provider, model_id))
    }

    /// Get a model configuration by provider and model ID.
    pub fn get(&self, provider: &ProviderId, model_id: &ModelId) -> Option<&ModelConfig> {
        self.models.get(&model_key(provider, model_id))
    }

    /// List all models for a provider.
    pub fn list_for_provider(&self, provider: &ProviderId) -> Vec<&ModelConfig> {
        let prefix = format!("{}:", provider.0);
        self.models
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v)
            .collect()
    }

    /// List all models across all providers.
    pub fn list_all(&self) -> Vec<&ModelConfig> {
        self.models.values().collect()
    }

    /// Find models by capability.
    pub fn find_by_capability(&self, capability: ModelCapability) -> Vec<&ModelConfig> {
        self.models
            .values()
            .filter(|m| m.capabilities.contains(capability))
            .collect()
    }

    /// Select the best model for a given capability and provider.
    pub fn select_model(
        &self,
        provider: &ProviderId,
        capability: ModelCapability,
    ) -> Option<&ModelConfig> {
        let prefix = format!("{}:", provider.0);
        self.models
            .iter()
            .filter(|(k, m)| {
                k.starts_with(&prefix) && m.capabilities.contains(capability) && m.enabled
            })
            .max_by_key(|(_, m)| m.capabilities.0.count_ones())
            .map(|(_, m)| m)
    }
}

/// Normalized provider errors — never leak credentials or raw provider responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderError {
    AuthenticationFailed,
    InvalidApiKey,
    ModelUnavailable { model: ModelId },
    RateLimited { retry_after_secs: Option<u64> },
    Timeout,
    NetworkError,
    InvalidRequest { reason: String },
    SchemaViolation { reason: String },
    UnsupportedCapability { capability: ModelCapability },
    ContentFiltered,
    ProviderUnavailable,
    Unknown { code: String, message: String },
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::AuthenticationFailed => write!(f, "authentication failed"),
            ProviderError::InvalidApiKey => write!(f, "invalid API key"),
            ProviderError::ModelUnavailable { model } => write!(f, "model unavailable: {}", model),
            ProviderError::RateLimited { retry_after_secs } => {
                if let Some(secs) = retry_after_secs {
                    write!(f, "rate limited (retry after {}s)", secs)
                } else {
                    write!(f, "rate limited")
                }
            }
            ProviderError::Timeout => write!(f, "request timeout"),
            ProviderError::NetworkError => write!(f, "network error"),
            ProviderError::InvalidRequest { reason } => write!(f, "invalid request: {}", reason),
            ProviderError::SchemaViolation { reason } => write!(f, "schema violation: {}", reason),
            ProviderError::UnsupportedCapability { capability } => {
                write!(f, "unsupported capability: {:?}", capability)
            }
            ProviderError::ContentFiltered => write!(f, "content filtered"),
            ProviderError::ProviderUnavailable => write!(f, "provider unavailable"),
            ProviderError::Unknown { code, message } => {
                write!(f, "provider error {}: {}", code, message)
            }
        }
    }
}
impl std::error::Error for ProviderError {}

/// Factory for creating `ModelProvider` instances from configuration.
pub struct ProviderFactory {
    credential_store: Box<dyn CredentialStore>,
}

impl ProviderFactory {
    pub fn new(credential_store: Box<dyn CredentialStore>) -> Self {
        Self { credential_store }
    }

    /// Resolve a configured credential without exposing the store implementation.
    pub fn resolve_credential(&self, cred_ref: &CredentialRef) -> Option<String> {
        self.credential_store.resolve(cred_ref)
    }

    /// Create a `ModelProvider` from a provider configuration.
    pub fn create(
        &self,
        config: &ProviderConfig,
    ) -> Result<Box<dyn ModelProvider>, ReasoningError> {
        if !config.enabled {
            return Err(ReasoningError::Provider(
                ProviderError::ProviderUnavailable.to_string(),
            ));
        }

        let provider = match config.id.0.as_str() {
            "openai-compatible" | "openai" | "openrouter" | "groq" | "nvidia" | "together"
            | "fireworks" | "vllm" | "lmstudio" => {
                #[cfg(feature = "live")]
                {
                    let model =
                        match config.default_model.clone() {
                            Some(m) => m,
                            None => return Err(ReasoningError::Provider(
                                ProviderError::InvalidRequest {
                                    reason:
                                        "default_model must be set for OpenAI-compatible provider"
                                            .into(),
                                }
                                .to_string(),
                            )),
                        };

                    let mut config = eak_reasoning::OpenAICompatConfig {
                        provider_id: config.id.clone(),
                        display_name: config.name.clone(),
                        base_url: config
                            .endpoint
                            .clone()
                            .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
                        model: config.default_model.clone().unwrap(),
                        credential_ref: config.credential_ref.clone(),
                        api_key: None,
                        extra_headers: config.extra_headers.clone().unwrap_or_default(),
                        supports_model_listing: true,
                        supports_streaming: true,
                        supports_tool_calling: true,
                        supports_structured_output: false,
                        supports_parallel_tool_calls: false,
                        supports_vision: false,
                        supports_system_instructions: true,
                        supports_reasoning_effort: false,
                        timeout_secs: 120,
                    };

                    // Resolve credential if credential_ref is provided
                    if let Some(cred_ref) = &config.credential_ref {
                        if let Some(api_key) = self.credential_store.resolve(cred_ref) {
                            config.api_key = Some(api_key);
                        }
                    }

                    let engine = eak_reasoning::OpenAICompatEngine::new(config).map_err(|e| {
                        ReasoningError::Provider(
                            eak_ports::ProviderError::InvalidRequest {
                                reason: e.to_string(),
                            }
                            .to_string(),
                        )
                    })?;
                    Ok(Box::new(engine))
                }
                #[cfg(not(feature = "live"))]
                Err(ReasoningError::Provider(
                    ProviderError::ProviderUnavailable.to_string(),
                ))
            }
            "anthropic" => {
                #[cfg(feature = "live")]
                {
                    let model = config
                        .default_model
                        .clone()
                        .unwrap_or_else(|| "claude-opus-4-8".into());
                    let engine = eak_reasoning::AnthropicEngine::from_env(model).map_err(|e| {
                        ReasoningError::Provider(
                            eak_ports::ProviderError::InvalidRequest {
                                reason: e.to_string(),
                            }
                            .to_string(),
                        )
                    })?;
                    Ok(Box::new(engine))
                }
                #[cfg(not(feature = "live"))]
                Err(ReasoningError::Provider(
                    ProviderError::ProviderUnavailable.to_string(),
                ))
            }
            "fixture" => {
                // Fixture engine is created by the caller (e.g., CLI) and passed as a ModelProvider
                Err(ReasoningError::Provider(
                    ProviderError::InvalidRequest {
                        reason: "fixture provider must be created by caller".into(),
                    }
                    .to_string(),
                ))
            }
            _ => Err(ReasoningError::Provider(
                ProviderError::Unknown {
                    code: "unknown_provider".to_string(),
                    message: format!("Unknown provider: {}", config.id),
                }
                .to_string(),
            )),
        }?;

        provider
    }
}

/// Connection test result for a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Test a provider connection.
pub fn test_provider_connection(
    factory: &ProviderFactory,
    config: &ProviderConfig,
) -> ConnectionTestResult {
    let provider = match factory.create(config) {
        Ok(p) => p,
        Err(e) => {
            return ConnectionTestResult {
                success: false,
                message: format!("Failed to create provider: {}", e),
                details: None,
            };
        }
    };

    // Try to list models to test the connection
    match provider.list_models() {
        Ok(models) => ConnectionTestResult {
            success: true,
            message: format!("Successfully connected. Found {} models.", models.len()),
            details: Some(serde_json::json!({
                "models_found": models.len(),
                "models": models.iter().map(|m| m.model_id.0.clone()).collect::<Vec<_>>()
            })),
        },
        Err(e) => ConnectionTestResult {
            success: false,
            message: format!("Connection test failed: {}", e),
            details: None,
        },
    }
}

/// Trait for secure credential storage and resolution.
/// Implementations can read from environment variables, OS keyring, vault, etc.
pub trait CredentialStore: Send + Sync {
    /// Resolve a credential reference to its raw value.
    /// Returns None if the credential is not found.
    fn resolve(&self, cred_ref: &CredentialRef) -> Option<String>;

    /// Test if a credential reference can be resolved.
    fn can_resolve(&self, cred_ref: &CredentialRef) -> bool {
        self.resolve(cred_ref).is_some()
    }
}

/// Default implementation that reads credentials from environment variables.
/// Uses the convention: `{STORE}_{KEY}` (e.g., "OPENAI_API_KEY" for store="OPENAI", key="API_KEY")
pub struct EnvCredentialStore;

impl CredentialStore for EnvCredentialStore {
    fn resolve(&self, cred_ref: &CredentialRef) -> Option<String> {
        let env_key = format!(
            "{}_{}",
            cred_ref.store.to_uppercase(),
            cred_ref.key.to_uppercase()
        );
        std::env::var(&env_key).ok()
    }
}

/// A secure credential store that uses the OS keyring (stub implementation).
///
/// This is a placeholder that provides the same interface as `EnvCredentialStore`
/// but can be extended to use the `keyring` crate for actual OS keyring integration.
/// Currently falls back to environment variables.
#[derive(Debug, Default, Clone)]
pub struct KeyringCredentialStore;

impl CredentialStore for KeyringCredentialStore {
    fn resolve(&self, cred_ref: &CredentialRef) -> Option<String> {
        // Try keyring first (not implemented yet, falls through to env)
        // In a full implementation, this would use the `keyring` crate:
        // keyring::Entry::new(&cred_ref.store, &cred_ref.key)
        //     .ok()
        //     .and_then(|e| e.get_password().ok())

        // Fallback to environment variable convention
        let env_key = format!(
            "{}_{}",
            cred_ref.store.to_uppercase(),
            cred_ref.key.to_uppercase()
        );
        std::env::var(&env_key).ok()
    }
}

/// Validation result for a provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderValidationResult {
    pub provider_id: ProviderId,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate a single provider configuration without creating a provider.
pub fn validate_provider_config(config: &ProviderConfig) -> ProviderValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Required fields
    if config.id.0.is_empty() {
        errors.push("Provider ID cannot be empty".to_string());
    }
    if config.name.is_empty() {
        errors.push("Provider name cannot be empty".to_string());
    }

    // If enabled, must have default_model
    if config.enabled {
        if config.default_model.is_none() {
            errors.push("Enabled provider must have a default_model".to_string());
        }
        if config.endpoint.is_none() {
            warnings.push("No endpoint specified; will use provider default".to_string());
        }
    }

    // Validate credential reference
    if let Some(cred_ref) = &config.credential_ref {
        if cred_ref.store.is_empty() || cred_ref.key.is_empty() {
            errors.push("Credential reference has empty store or key".to_string());
        }
    } else if config.enabled {
        warnings.push(
            "No credential reference configured; will rely on environment variables".to_string(),
        );
    }

    // Validate base URL format if provided
    if let Some(endpoint) = &config.endpoint {
        if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            errors.push("Endpoint must be a valid HTTP/HTTPS URL".to_string());
        }
    }

    ProviderValidationResult {
        provider_id: config.id.clone(),
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Validate a model configuration.
pub fn validate_model_config(config: &ModelConfig) -> ProviderValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if config.provider.0.is_empty() {
        errors.push("Model provider ID cannot be empty".to_string());
    }
    if config.model_id.0.is_empty() {
        errors.push("Model ID cannot be empty".to_string());
    }

    if !config.enabled {
        warnings.push("Model is disabled".to_string());
    }

    if config.capabilities.is_empty() {
        warnings.push("Model has no declared capabilities".to_string());
    }

    ProviderValidationResult {
        provider_id: config.provider.clone(),
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// A tool definition for function calling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// A tool call requested by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// A tool result returned to the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: serde_json::Value,
    pub is_error: bool,
}

/// How the model should choose tools.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    None,
    Auto,
    Required,
    Specific { name: String },
}

/// Token usage metadata from the provider.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UsageMetadata {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cached_tokens: Option<u32>,
    pub reasoning_tokens: Option<u32>,
}

/// Streaming event from a model provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StreamEvent {
    TextDelta {
        text: String,
    },
    ToolCallDelta {
        index: u32,
        id: Option<String>,
        name: Option<String>,
        arguments: Option<String>,
    },
    ToolCallComplete {
        tool_call: ToolCall,
    },
    Complete {
        usage: Option<UsageMetadata>,
    },
    Error {
        error: ProviderError,
    },
}

/// Extended provider interface supporting streaming, tool calling, and capability introspection.
/// This is the main trait for model providers in EAK V1.
pub trait ModelProvider: ReasoningEngine {
    /// Returns the provider identifier (e.g., "anthropic", "openai", "ollama").
    fn provider_id(&self) -> ProviderId {
        ProviderId(
            self.model_id()
                .split(':')
                .next()
                .unwrap_or("unknown")
                .to_string(),
        )
    }

    /// Returns the model identifier (e.g., "claude-opus-4", "gpt-4o", "llama3.1").
    fn model_id_parsed(&self) -> ModelId {
        ModelId(
            self.model_id()
                .split(':')
                .nth(1)
                .unwrap_or(&self.model_id())
                .to_string(),
        )
    }

    /// Returns the model's capability set.
    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet::TEXT_GENERATION
    }

    /// Returns model metadata if available.
    fn metadata(&self) -> Option<ModelMetadata> {
        None
    }

    /// Synchronous request (default implementation uses ReasoningEngine).
    fn request(&self, req: &ReasoningRequest) -> Result<ReasoningResponse, ReasoningError> {
        self.request_judgement(req)
    }

    /// Streaming request — returns a stream of events.
    /// Default implementation returns an error indicating streaming not supported.
    fn stream_request(
        &self,
        _req: &ReasoningRequest,
    ) -> Result<Box<dyn Iterator<Item = Result<StreamEvent, ReasoningError>> + Send>, ReasoningError>
    {
        Err(ReasoningError::Provider(
            ProviderError::UnsupportedCapability {
                capability: ModelCapability::Streaming,
            }
            .to_string(),
        ))
    }

    /// Cancel an in-flight request (if supported by provider).
    fn cancel(&self) -> Result<(), ReasoningError> {
        Err(ReasoningError::Provider(
            ProviderError::UnsupportedCapability {
                capability: ModelCapability::Streaming,
            }
            .to_string(),
        ))
    }

    /// Check if the provider supports a specific capability.
    fn supports(&self, capability: ModelCapability) -> bool {
        self.capabilities().contains(capability)
    }

    /// List available models from this provider (if supported).
    fn list_models(&self) -> Result<Vec<ModelMetadata>, ReasoningError> {
        Err(ReasoningError::Provider(
            ProviderError::UnsupportedCapability {
                capability: ModelCapability::TextGeneration,
            }
            .to_string(),
        ))
    }
}

/// Note: explicit `ModelProvider` implementations are provided for `FixtureEngine` and all
/// test reasoners. A blanket impl is not used to avoid conflicts with custom implementations.
///
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

/// Provider configuration persistence and validation.
pub mod provider_config;

// Tool execution boundary between model providers and the kernel.
// pub mod tool_execution;

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

    // ===================== Band B (increment 6): Interface / Contract events =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn contract_committed_event_roundtrips_through_json() {
        use eak_domain::{Contract, EntityId};
        let ev = Event::ContractCommitted {
            contract: Contract {
                id: EntityId(90),
                protocol: "I2C".into(),
                name: "I2C Bus 1".into(),
                constraints: vec!["unique addresses".into()],
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn interface_committed_event_roundtrips_through_json() {
        use eak_domain::{EntityId, Interface};
        let ev = Event::InterfaceCommitted {
            interface: Interface {
                id: EntityId(91),
                name: "I2C1".into(),
                signals: vec![EntityId(92), EntityId(93)],
                contract: EntityId(90),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band B (increment 7): Bus event =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn bus_committed_event_roundtrips_through_json() {
        use eak_domain::{Bus, BusTopology, EntityId};
        let ev = Event::BusCommitted {
            bus: Bus {
                id: EntityId(110),
                name: "I2C_BUS_1".into(),
                contract: EntityId(100),
                members: vec![EntityId(101), EntityId(102)],
                topology: BusTopology::MultiDrop,
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }

    // ===================== Band B (increment 8): Subsystem event =====================
    //
    // TDD: every new state-delta Event variant carries a serde round-trip test so its on-disk
    // form is pinned and replay-from-log is byte-stable (P4).

    #[test]
    fn subsystem_committed_event_roundtrips_through_json() {
        use eak_domain::{EntityId, Subsystem};
        let ev = Event::SubsystemCommitted {
            subsystem: Subsystem {
                id: EntityId(120),
                name: "MCU_SUBSYSTEM".into(),
                blocks: vec![EntityId(121), EntityId(122)],
                interfaces: vec![EntityId(123), EntityId(124)],
                boundary: "MCU + peripherals".into(),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(ev, back);
    }
}
