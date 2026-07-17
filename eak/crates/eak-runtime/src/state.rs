//! Engineering State — the runtime's owned model, built by folding the event log.
//!
//! State is the fold of an ordered [`Event`] sequence (determinism, P4). Entity-bearing
//! events are state deltas; the rest are audit-only. The same fold runs live during a
//! run and during [`crate::replay`], guaranteeing identical reconstruction.

use eak_domain::{
    Assumption, AssumptionStatus, Board, BomLineItem, Component, Constraint, Decision,
    DesignIntent, EntityId, Evidence, FunctionalBlock, ModelFidelity, Net, Part, Pin, Placement,
    ProvenanceLink, Requirement, Risk, RiskSeverity, RiskStatus, Track, Violation, ViolationStatus,
    Waiver,
};
use eak_ports::{Event, Seq};
use serde::{Deserialize, Serialize};

/// Advisory, model-authored review metadata attached to a raised [`Violation`] (E6 C1).
///
/// It is deliberately NOT a domain engineering entity: it carries no id of its own, is never
/// validated at a capability seam, and NEVER affects the violation's severity/status/validity or
/// gates a phase. It only *describes*. It is folded from [`Event::ViolationExplained`] into
/// [`EngineeringState::violation_explanations`] so the review/traceability UI can read it back and
/// so replay reconstructs it byte-identically (P4). `reasoning_call_seq` points back at the exact
/// [`Event::ReasoningCall`] that produced the text (provenance-by-construction).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViolationExplanation {
    pub violation: EntityId,
    pub explanation: String,
    pub suggested_fix: String,
    pub reasoning_call_seq: Seq,
}

/// A fidelity trust-tag attached to a derived fact (Band A, increment 2). Like
/// [`ViolationExplanation`] it is advisory-only: it carries no id of its own, is folded into its
/// OWN store ([`EngineeringState::fidelity_tags`]), references a `target` it never mutates, and
/// never gates a phase. It only *describes how well a fact is known* (Map 6). `reasoning_call_seq`
/// points back at the [`Event::ReasoningCall`] that produced it (`None` when the runtime itself
/// tagged the fact, e.g. a first-order floor). Folded so replay reconstructs it byte-identically.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FidelityTag {
    pub target: EntityId,
    pub fidelity: ModelFidelity,
    pub reasoning_call_seq: Option<Seq>,
}

/// The single canonical instance of everything the runtime knows about a design. Entities
/// are kept in insertion (event) order so a run and its replay serialize byte-identically.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EngineeringState {
    pub intent: Option<DesignIntent>,
    pub requirements: Vec<Requirement>,
    pub decisions: Vec<Decision>,
    pub evidence: Vec<Evidence>,
    pub links: Vec<ProvenanceLink>,
    // Phase 2: the machine-checkable layer.
    pub constraints: Vec<Constraint>,
    pub violations: Vec<Violation>,
    pub waivers: Vec<Waiver>,
    // Phase 3: the synthesis (realization) layer beneath verification.
    pub functional_blocks: Vec<FunctionalBlock>,
    pub components: Vec<Component>,
    pub pins: Vec<Pin>,
    pub nets: Vec<Net>,
    // Phase 3 (BOM): concrete parts and the line items binding them to components.
    pub parts: Vec<Part>,
    pub bom_line_items: Vec<BomLineItem>,
    // Phase 3 (PCB): the single board outline plus each component's placement on it.
    pub board: Option<Board>,
    pub placements: Vec<Placement>,
    // Phase 3 (routing): the copper tracks realizing the nets.
    pub tracks: Vec<Track>,
    // E6 (C1): advisory, model-authored explanations of raised violations. A SEPARATE store from
    // `violations` — folding here never touches a `Violation`, so the explainer stays advisory-only.
    pub violation_explanations: Vec<ViolationExplanation>,
    // Band A (Phase 3): first-class, auditable presumptions. An Open + Critical one blocks
    // release at the honesty gate ([`Self::undischarged_critical_assumptions`]).
    pub assumptions: Vec<Assumption>,
    // Band A (increment 2): advisory trust-tags on derived facts (Map 6). A SEPARATE store from
    // any entity — folding here never mutates a tagged entity, so the tag stays advisory-only.
    // Kept in insertion (event) order so a run and its replay serialize byte-identically.
    pub fidelity_tags: Vec<FidelityTag>,
    // Band A (increment 3): first-class, auditable risks (Map 46). TRACKED TRUTH — a Risk is
    // surfaced ([`Self::unaccepted_critical_risks`]) but does NOT block release in v0; the human
    // owns acceptance of residual risk (`00` Principle 11).
    pub risks: Vec<Risk>,
}

impl EngineeringState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fold one event. State-delta variants mutate; audit variants are ignored here
    /// (they still live in the log for provenance).
    pub fn apply(&mut self, event: &Event) {
        match event {
            Event::IntentCaptured { intent } => self.intent = Some(intent.clone()),
            Event::RequirementCommitted { requirement } => {
                self.requirements.push(requirement.clone())
            }
            Event::DecisionCreated { decision } => self.decisions.push(decision.clone()),
            Event::EvidenceReferenced { evidence } => self.evidence.push(evidence.clone()),
            Event::ProvenanceLinked { link } => self.links.push(link.clone()),
            Event::ConstraintCommitted { constraint } => self.constraints.push(constraint.clone()),
            Event::ViolationRaised { violation } => self.violations.push(violation.clone()),
            Event::WaiverGranted { waiver } => {
                // A waiver is itself a recorded fact AND it transitions its target violation.
                // Folding both here keeps replay byte-identical to the live run (P4).
                if let Some(v) = self
                    .violations
                    .iter_mut()
                    .find(|v| v.id == waiver.violation)
                {
                    v.status = ViolationStatus::Waived;
                }
                self.waivers.push(waiver.clone());
            }
            Event::FunctionalBlockCommitted { block } => self.functional_blocks.push(block.clone()),
            Event::ComponentCommitted { component } => self.components.push(component.clone()),
            Event::PinCommitted { pin } => self.pins.push(pin.clone()),
            Event::NetCommitted { net } => self.nets.push(net.clone()),
            Event::PartCommitted { part } => self.parts.push(part.clone()),
            Event::BomLineItemCommitted { item } => self.bom_line_items.push(item.clone()),
            Event::BoardCommitted { board } => self.board = Some(board.clone()),
            Event::PlacementCommitted { placement } => self.placements.push(placement.clone()),
            Event::TrackCommitted { track } => self.tracks.push(track.clone()),
            // E6 (C1): advisory review metadata. State-bearing (so the UI/replay can read it), but
            // it folds into its OWN store and never mutates the referenced `Violation` — the
            // advisory-only invariant is structural, not merely a convention.
            Event::ViolationExplained {
                violation,
                explanation,
                suggested_fix,
                reasoning_call_seq,
            } => self.violation_explanations.push(ViolationExplanation {
                violation: *violation,
                explanation: explanation.clone(),
                suggested_fix: suggested_fix.clone(),
                reasoning_call_seq: *reasoning_call_seq,
            }),
            // Band A (Phase 3): epistemic state deltas. Raise pushes; discharge finds by id,
            // flips status -> Discharged, and records the discharge on the assumption. Folding
            // both here keeps replay byte-identical to the live run (P4).
            Event::AssumptionRaised { assumption } => self.assumptions.push(assumption.clone()),
            Event::AssumptionDischarged {
                assumption,
                discharge,
            } => {
                if let Some(a) = self.assumptions.iter_mut().find(|a| a.id == *assumption) {
                    a.status = AssumptionStatus::Discharged;
                    a.discharge = Some(discharge.clone());
                }
            }
            // Band A (increment 2): advisory fidelity metadata. State-bearing (so the UI/replay
            // can read it), but it folds into its OWN store and NEVER mutates the tagged entity —
            // the advisory-only invariant is structural, not merely a convention (mirrors
            // `ViolationExplained`).
            Event::FidelityTagged {
                target,
                fidelity,
                reasoning_call_seq,
            } => self.fidelity_tags.push(FidelityTag {
                target: *target,
                fidelity: fidelity.clone(),
                reasoning_call_seq: *reasoning_call_seq,
            }),
            // Band A (increment 3): risk posture. Raise pushes; accept finds by id and flips
            // status -> Accepted (human authority, Principle 11). Folding both here keeps replay
            // byte-identical to the live run (P4).
            Event::RiskRaised { risk } => self.risks.push(risk.clone()),
            Event::RiskAccepted { risk, .. } => {
                if let Some(r) = self.risks.iter_mut().find(|r| r.id == *risk) {
                    r.status = RiskStatus::Accepted;
                }
            }
            // Audit-only events (phase lifecycle, reasoning calls, IR-boundary milestones)
            // carry no state and are intentionally not folded. AUDIT: any NEW state-bearing
            // event variant MUST get an explicit arm above, or replay will silently diverge.
            _ => {}
        }
    }

    pub fn requirement(&self, id: EntityId) -> Option<&Requirement> {
        self.requirements.iter().find(|r| r.id == id)
    }

    pub fn decision(&self, id: EntityId) -> Option<&Decision> {
        self.decisions.iter().find(|d| d.id == id)
    }

    pub fn evidence_item(&self, id: EntityId) -> Option<&Evidence> {
        self.evidence.iter().find(|e| e.id == id)
    }

    pub fn constraint(&self, id: EntityId) -> Option<&Constraint> {
        self.constraints.iter().find(|c| c.id == id)
    }

    pub fn violation(&self, id: EntityId) -> Option<&Violation> {
        self.violations.iter().find(|v| v.id == id)
    }

    pub fn functional_block(&self, id: EntityId) -> Option<&FunctionalBlock> {
        self.functional_blocks.iter().find(|b| b.id == id)
    }

    pub fn component(&self, id: EntityId) -> Option<&Component> {
        self.components.iter().find(|c| c.id == id)
    }

    pub fn pin(&self, id: EntityId) -> Option<&Pin> {
        self.pins.iter().find(|p| p.id == id)
    }

    pub fn net(&self, id: EntityId) -> Option<&Net> {
        self.nets.iter().find(|n| n.id == id)
    }

    pub fn part(&self, id: EntityId) -> Option<&Part> {
        self.parts.iter().find(|p| p.id == id)
    }

    pub fn bom_line_item(&self, id: EntityId) -> Option<&BomLineItem> {
        self.bom_line_items.iter().find(|i| i.id == id)
    }

    pub fn board(&self) -> Option<&Board> {
        self.board.as_ref()
    }

    pub fn placement(&self, id: EntityId) -> Option<&Placement> {
        self.placements.iter().find(|p| p.id == id)
    }

    pub fn track(&self, id: EntityId) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == id)
    }

    /// Advisory explanations recorded for a given violation (E6 C1). Read-only metadata — it never
    /// participates in any gate; the workflow gate remains [`Self::open_blocking_violations`].
    pub fn explanations_for(&self, violation: EntityId) -> Vec<&ViolationExplanation> {
        self.violation_explanations
            .iter()
            .filter(|e| e.violation == violation)
            .collect()
    }

    /// Open, blocking (error-severity) violations — the workflow gate (P13).
    pub fn open_blocking_violations(&self) -> Vec<&Violation> {
        self.violations.iter().filter(|v| v.is_blocking()).collect()
    }

    pub fn assumption(&self, id: EntityId) -> Option<&Assumption> {
        self.assumptions.iter().find(|a| a.id == id)
    }

    /// Advisory fidelity tags recorded for a given target, in insertion order (Band A, increment
    /// 2). Read-only metadata — it never participates in any gate; it only makes visible how well
    /// a derived fact is known (Map 6).
    pub fn fidelity_for(&self, target: EntityId) -> Vec<&ModelFidelity> {
        self.fidelity_tags
            .iter()
            .filter(|t| t.target == target)
            .map(|t| &t.fidelity)
            .collect()
    }

    /// Undischarged CRITICAL assumptions — the honesty gate (Band A, exit criterion 1). A
    /// release is refused while this is non-empty (mirror of [`Self::open_blocking_violations`]).
    pub fn undischarged_critical_assumptions(&self) -> Vec<&Assumption> {
        self.assumptions
            .iter()
            .filter(|a| a.is_blocking())
            .collect()
    }

    pub fn risk(&self, id: EntityId) -> Option<&Risk> {
        self.risks.iter().find(|r| r.id == id)
    }

    /// Risks whose RESIDUAL severity is `Critical` and that have NOT yet been accepted by a
    /// human. Band A (increment 3): this is a READ, not a gate — Risk is tracked truth and does
    /// NOT block release in v0. It surfaces the risk posture for the human, who owns acceptance
    /// of the residual (`00` Principle 11).
    pub fn unaccepted_critical_risks(&self) -> Vec<&Risk> {
        self.risks
            .iter()
            .filter(|r| r.residual == RiskSeverity::Critical && r.status != RiskStatus::Accepted)
            .collect()
    }

    /// Deterministic serialization used to assert byte-identity between a run and its
    /// replay (the Phase-1 exit criterion).
    pub fn canonical_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("engineering state serializes")
    }
}
