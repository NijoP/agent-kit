//! The Engineering Runtime kernel — the sole mutator of Engineering State (P2).
//!
//! Every event funnels through the single [`RuntimeCore::commit`] path: stamp (clock) ->
//! append (event log) -> fold (state). The runtime implements [`AgentContext`], so agents
//! reach reasoning and mutation only through it. Capability handlers re-validate proposals
//! at the seam (P3) before committing.

use crate::clock::{Clock, IdSource};
use crate::protocol::{AgentContext, Autonomy, CapabilityAck, CapabilityError, CapabilityRequest};
use crate::state::EngineeringState;
use eak_domain::{
    Assumption, Board, BomLineItem, ClockDomain, Component, ComponentOrigin, Constraint, Decision,
    DesignIntent, Discharge, EntityId, Evidence, FunctionalBlock, Net, NetOrigin, Objective, Part,
    Pin, Placement, PowerDomain, ProvenanceLink, Requirement, ReturnPath, Risk, Track, Tradeoff,
    Violation, Waiver,
};
use eak_ports::{
    Event, EventLog, EventRecord, EventSink, ReasoningEngine, ReasoningError, ReasoningRequest,
    ReasoningResponse, Seq, StoreError, Timestamp,
};

pub struct RuntimeCore {
    pub state: EngineeringState,
    log: Box<dyn EventLog>,
    reasoning: Box<dyn ReasoningEngine>,
    ids: Box<dyn IdSource>,
    clock: Box<dyn Clock>,
    autonomy: Autonomy,
    /// Optional live observer of committed events (a UI stream, telemetry, …). `None` restores
    /// the pre-observer behavior exactly; a sink only observes, never mutates (P4).
    sink: Option<Box<dyn EventSink>>,
}

impl RuntimeCore {
    pub fn new(
        log: Box<dyn EventLog>,
        reasoning: Box<dyn ReasoningEngine>,
        ids: Box<dyn IdSource>,
        clock: Box<dyn Clock>,
        autonomy: Autonomy,
    ) -> Self {
        Self {
            state: EngineeringState::new(),
            log,
            reasoning,
            ids,
            clock,
            autonomy,
            sink: None,
        }
    }

    /// Attach a live [`EventSink`] that observes every committed event as it happens (builder
    /// form). Absent a sink the runtime behaves exactly as before; a sink never affects what is
    /// committed or folded — it only observes — so determinism and replay are unchanged (P4). This
    /// is the seam a desktop UI subscribes to for live streaming.
    pub fn with_sink(mut self, sink: Box<dyn EventSink>) -> Self {
        self.sink = Some(sink);
        self
    }

    /// Attach or replace the live [`EventSink`] after construction (same guarantees as
    /// [`with_sink`](Self::with_sink)).
    pub fn set_sink(&mut self, sink: Box<dyn EventSink>) {
        self.sink = Some(sink);
    }

    /// Read-only access to the log (for replay / inspection).
    pub fn log(&self) -> &dyn EventLog {
        self.log.as_ref()
    }

    /// The single commit path (P2): stamp -> append -> fold. All event production
    /// converges here so every change is recorded and reproducible.
    fn commit(&mut self, events: Vec<Event>) -> Result<Vec<Seq>, StoreError> {
        let stamped: Vec<(Timestamp, Event)> = events
            .iter()
            .map(|e| (self.clock.now(), e.clone()))
            .collect();
        let seqs = self.log.append(&stamped)?;
        for e in &events {
            self.state.apply(e);
        }
        // Notify a live observer (if any) once per event, in commit order, AFTER the fold — so a
        // streaming consumer sees each change against consistent state. Purely observational: the
        // sink cannot alter what was committed, so replay and determinism are untouched (P4).
        if let Some(sink) = self.sink.as_mut() {
            for ((timestamp, event), seq) in stamped.iter().zip(seqs.iter()) {
                sink.on_committed(&EventRecord {
                    seq: *seq,
                    timestamp: *timestamp,
                    event: event.clone(),
                });
            }
        }
        Ok(seqs)
    }

    /// Seed the phase with the engineer's intent (trusted input, not model output).
    pub fn capture_intent(
        &mut self,
        statement: &str,
        source: &str,
    ) -> Result<EntityId, StoreError> {
        let id = self.ids.fresh();
        let intent = DesignIntent {
            id,
            statement: statement.to_string(),
            structured_summary: statement.to_string(),
            source: source.to_string(),
        };
        self.commit(vec![Event::IntentCaptured { intent }])?;
        Ok(id)
    }

    fn handle_create_requirement(
        &mut self,
        requirement: Requirement,
        decision: Decision,
        evidence: Vec<Evidence>,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): the runtime, not the model, commits. Re-validate domain invariants.
        requirement
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        if self.autonomy == Autonomy::Supervised {
            return Err(CapabilityError::Rejected(
                "supervised autonomy requires human approval (HITL deferred to a later phase)"
                    .into(),
            ));
        }

        let mut events = Vec::new();
        for ev in evidence {
            events.push(Event::EvidenceReferenced { evidence: ev });
        }
        events.push(Event::DecisionCreated { decision });
        events.push(Event::RequirementCommitted { requirement });
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_create_constraint(
        &mut self,
        constraint: Constraint,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the constraint and its subject before committing.
        constraint
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        if constraint.subject_requirement.is_null() {
            return Err(CapabilityError::Rejected(
                "constraint has no subject requirement".into(),
            ));
        }
        // Referential integrity at the seam: the subject must be a committed requirement,
        // so a constraint can never dangle (P3, P5).
        if self
            .state
            .requirement(constraint.subject_requirement)
            .is_none()
        {
            return Err(CapabilityError::Rejected(format!(
                "constraint subject requirement {} does not exist",
                constraint.subject_requirement.short()
            )));
        }

        let mut events = vec![Event::ConstraintCommitted { constraint }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_raise_violation(
        &mut self,
        violation: Violation,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // A violation that names no subjects would be untraceable — reject it (P13).
        if violation.subjects.is_empty() {
            return Err(CapabilityError::Rejected(
                "violation names no subjects (would be untraceable)".into(),
            ));
        }

        let mut events = vec![Event::ViolationRaised { violation }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_grant_waiver(&mut self, waiver: Waiver) -> Result<CapabilityAck, CapabilityError> {
        // Accepting a violation is a design-significant judgement (P10): in supervised mode
        // it needs human approval, which is deferred to a later phase.
        if self.autonomy == Autonomy::Supervised {
            return Err(CapabilityError::Rejected(
                "supervised autonomy requires human approval (HITL deferred to a later phase)"
                    .into(),
            ));
        }
        // The target must exist — a waiver for an unknown violation is meaningless.
        if self.state.violation(waiver.violation).is_none() {
            return Err(CapabilityError::Rejected(format!(
                "waiver targets unknown violation {}",
                waiver.violation.short()
            )));
        }

        let seqs = self
            .commit(vec![Event::WaiverGranted { waiver }])
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_create_functional_block(
        &mut self,
        block: FunctionalBlock,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the block and its requirement links before committing.
        block
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // A block that realizes no requirement is untraceable to intent (P3) — reject it.
        if block.requirements.is_empty() {
            return Err(CapabilityError::Rejected(
                "functional block realizes no requirement (would be untraceable)".into(),
            ));
        }
        // Referential integrity at the seam: every referenced requirement must exist, so
        // the block-to-requirement trace can never dangle (P3, P5).
        for rid in &block.requirements {
            if self.state.requirement(*rid).is_none() {
                return Err(CapabilityError::Rejected(format!(
                    "functional block references unknown requirement {}",
                    rid.short()
                )));
            }
        }

        let mut events = vec![Event::FunctionalBlockCommitted { block }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_realize_component(
        &mut self,
        component: Component,
        pins: Vec<Pin>,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the component and its originating block before committing.
        component
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // The block-origin rule is *origin-dependent* (P3), mirroring `handle_create_net`'s
        // Logical/Physical branch. A `Synthesized` component is minted from a committed
        // [`FunctionalBlock`] (Schematic Planning realizes it from intent), so a null or unknown
        // `from_block` is a synthesis defect — existing behavior, verbatim. An `Imported` component
        // is parsed from a finished board's footprint, which declares no functional decomposition, so
        // it MAY carry a null `from_block`; the seam never fabricates a block (that would invent intent
        // the board never stated). Either way, any block it DOES reference must be committed — Imported
        // relaxes existence only, never the no-phantom rule.
        match component.origin {
            ComponentOrigin::Synthesized => {
                // A component minted from no block is untraceable to intent (P3) — reject it.
                if component.from_block == EntityId::NULL {
                    return Err(CapabilityError::Rejected(
                        "component has no originating functional block".into(),
                    ));
                }
                // Referential integrity at the seam: the originating block must exist (P3, P5).
                if self.state.functional_block(component.from_block).is_none() {
                    return Err(CapabilityError::Rejected(format!(
                        "component originates from unknown functional block {}",
                        component.from_block.short()
                    )));
                }
            }
            ComponentOrigin::Imported => {
                // A null `from_block` is legitimate (an imported board declares no block); but any
                // block it DOES reference must be a committed one, so an import can never point at a
                // phantom block (P3, P5) — existence relaxed, no-phantom rule intact.
                if component.from_block != EntityId::NULL
                    && self.state.functional_block(component.from_block).is_none()
                {
                    return Err(CapabilityError::Rejected(format!(
                        "imported component references unknown functional block {}",
                        component.from_block.short()
                    )));
                }
            }
        }
        // A component with no pins can never join a net and would pass every ERC rule
        // vacuously — reject it (P13: no silently-inert entities).
        if pins.is_empty() {
            return Err(CapabilityError::Rejected("component has no pins".into()));
        }

        // One atomic realization: the component, then a pin event each, then the links.
        let mut events = vec![Event::ComponentCommitted { component }];
        for pin in pins {
            events.push(Event::PinCommitted { pin });
        }
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_create_net(
        &mut self,
        net: Net,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the net and its membership before committing.
        net.validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // The ≥1-pin rule is an *origin-dependent* seam invariant (P3, P13), not a domain one.
        // A `Logical` net is synthesised from the schematic, which joins pins, so a member-less
        // logical net is a synthesis defect and is rejected — existing behavior, verbatim. A
        // `Physical` net is imported from a finished board's copper, which carries no parsed pins
        // yet (import order is CreateBoard -> CreateNet -> RouteNet, so the realizing tracks do not
        // exist at net-creation), so it MAY be member-less; the seam never fabricates a pin to
        // satisfy the logical rule. Either way, any member that IS listed must be a committed pin.
        if matches!(net.origin, NetOrigin::Logical) && net.members.is_empty() {
            return Err(CapabilityError::Rejected(
                "net joins no pins (carries no connectivity)".into(),
            ));
        }
        // Referential integrity at the seam: every member that is listed must be a committed pin,
        // so connectivity can never reference a phantom terminal (P3, P5) — enforced for both
        // origins, so a `Physical` net can be empty but never fabricates a pin.
        for pid in &net.members {
            if self.state.pin(*pid).is_none() {
                return Err(CapabilityError::Rejected(format!(
                    "net references unknown pin {}",
                    pid.short()
                )));
            }
        }

        let mut events = vec![Event::NetCommitted { net }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_create_part(
        &mut self,
        part: Part,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the part (non-empty manufacturer part number) before
        // committing — an unorderable part must never enter the BOM.
        part.validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;

        let mut events = vec![Event::PartCommitted { part }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_create_bom_line_item(
        &mut self,
        item: BomLineItem,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate intrinsic invariants — non-empty coverage, quantity
        // equal to the component count, and no component listed twice.
        item.validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        if item.part.is_null() {
            return Err(CapabilityError::Rejected(
                "BOM line item has no part".into(),
            ));
        }
        // Referential integrity at the seam: the ordered part must be committed, so a line
        // can never bind a phantom part (P3, P5).
        if self.state.part(item.part).is_none() {
            return Err(CapabilityError::Rejected(format!(
                "BOM line item references unknown part {}",
                item.part.short()
            )));
        }
        // Referential integrity + single-sourcing: every covered component must exist and
        // must not already be claimed by another line — a component is ordered exactly once,
        // so the BOM can never silently double-source or contradict itself (P5).
        for cid in &item.components {
            if self.state.component(*cid).is_none() {
                return Err(CapabilityError::Rejected(format!(
                    "BOM line item references unknown component {}",
                    cid.short()
                )));
            }
            if self
                .state
                .bom_line_items
                .iter()
                .any(|l| l.components.contains(cid))
            {
                return Err(CapabilityError::Rejected(format!(
                    "BOM line item double-covers component {} (already on another line)",
                    cid.short()
                )));
            }
        }

        let mut events = vec![Event::BomLineItemCommitted { item }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_create_board(
        &mut self,
        board: Board,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the outline (positive dimensions + at least one layer)
        // before committing.
        board
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // A design has exactly one outline; a second board would make placement DRC ambiguous
        // (which board is being fit against?) — reject it (P5).
        if self.state.board.is_some() {
            return Err(CapabilityError::Rejected(
                "design already has a board outline".into(),
            ));
        }

        let mut events = vec![Event::BoardCommitted { board }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_place_component(
        &mut self,
        placement: Placement,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the courtyard (positive extent) before committing.
        placement
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // A placement with no component is untraceable to the schematic (P3) — reject it.
        if placement.component.is_null() {
            return Err(CapabilityError::Rejected(
                "placement has no component".into(),
            ));
        }
        // Referential integrity at the seam: the placed component must be realized (P3, P5).
        if self.state.component(placement.component).is_none() {
            return Err(CapabilityError::Rejected(format!(
                "placement references unknown component {}",
                placement.component.short()
            )));
        }
        // A placement is meaningless without an outline to fit against — require the board
        // first, so layout can never precede the floor plan (P5).
        if self.state.board.is_none() {
            return Err(CapabilityError::Rejected(
                "cannot place a component before the board outline exists".into(),
            ));
        }
        // Single-placement: a component sits at exactly one spot on one side, so a second
        // placement of the same component would contradict itself — reject it (P5).
        if self
            .state
            .placements
            .iter()
            .any(|p| p.component == placement.component)
        {
            return Err(CapabilityError::Rejected(
                "component is already placed".into(),
            ));
        }

        let mut events = vec![Event::PlacementCommitted { placement }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_route_net(
        &mut self,
        track: Track,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the trace (positive width, finite endpoints) before
        // committing.
        track
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // A track that realizes no net is untraceable to the schematic (P3) — reject it.
        if track.net.is_null() {
            return Err(CapabilityError::Rejected("track realizes no net".into()));
        }
        // Referential integrity at the seam: the realized net must be committed, so a track can
        // never realize a phantom net (P3, P5).
        if self.state.net(track.net).is_none() {
            return Err(CapabilityError::Rejected(format!(
                "track realizes unknown net {}",
                track.net.short()
            )));
        }
        // A track is copper on a substrate — require the board first, so routing can never
        // precede the floor plan (P5).
        if self.state.board.is_none() {
            return Err(CapabilityError::Rejected(
                "cannot route a net before the board outline exists".into(),
            ));
        }
        // A net is realized by a DAISY-CHAIN of one or more tracks (one segment per consecutive
        // member pad), so the seam does NOT cap the track count per net — that would reject the
        // chain's later segments. Idempotency is the Routing Planning machine's own concern: it
        // skips any net already realized by ≥1 track on re-entry, so a DRC loop-back never
        // re-mints (each segment still carries a fresh, unique id, so replay stays exact — P4).

        let mut events = vec![Event::TrackCommitted { track }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    /// Does `id` resolve to ANY committed entity an assumption may trace to (P3)? v0 accepts a
    /// requirement, decision, functional block, or constraint — the traceable spine so far.
    fn resolves_to_committed_entity(&self, id: EntityId) -> bool {
        self.state.requirement(id).is_some()
            || self.state.decision(id).is_some()
            || self.state.functional_block(id).is_some()
            || self.state.constraint(id).is_some()
    }

    fn handle_raise_assumption(
        &mut self,
        assumption: Assumption,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the assumption (non-empty statement, discharge/status
        // consistency) before committing — the model is never trusted.
        assumption
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // An assumption that rests on nothing would be untraceable — reject it (P3).
        if assumption.rests_on.is_null() {
            return Err(CapabilityError::Rejected(
                "assumption rests on no entity (would be untraceable)".into(),
            ));
        }
        // Referential integrity at the seam: `rests_on` must be a committed entity (requirement /
        // decision / functional block), so an assumption can never dangle (P3, P5).
        if self.state.requirement(assumption.rests_on).is_none()
            && self.state.decision(assumption.rests_on).is_none()
            && self.state.functional_block(assumption.rests_on).is_none()
        {
            return Err(CapabilityError::Rejected(format!(
                "assumption rests on unknown entity {}",
                assumption.rests_on.short()
            )));
        }
        // A raised assumption is never born discharged — it must start Open.
        if !assumption.is_open() {
            return Err(CapabilityError::Rejected(
                "a raised assumption must be Open".into(),
            ));
        }

        let mut events = vec![Event::AssumptionRaised { assumption }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_discharge_assumption(
        &mut self,
        assumption: EntityId,
        discharge: Discharge,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The target assumption must exist and still be Open (cannot re-discharge, nor discharge
        // an Invalidated one) — the seam guards the lifecycle (P3).
        match self.state.assumption(assumption) {
            None => {
                return Err(CapabilityError::Rejected(format!(
                    "discharge targets unknown assumption {}",
                    assumption.short()
                )));
            }
            Some(a) if !a.is_open() => {
                return Err(CapabilityError::Rejected("assumption is not open".into()));
            }
            Some(_) => {}
        }
        // The discharge target must resolve to some committed entity, so a discharge can never
        // point at a phantom (P3). v0 does NOT yet tighten AcceptedRisk -> Risk (Risk lands in a
        // later increment); the target is a free EntityId that must merely resolve.
        if discharge.target.is_null() || !self.resolves_to_committed_entity(discharge.target) {
            return Err(CapabilityError::Rejected(format!(
                "discharge target {} does not exist",
                discharge.target.short()
            )));
        }

        let seqs = self
            .commit(vec![Event::AssumptionDischarged {
                assumption,
                discharge,
            }])
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_raise_risk(
        &mut self,
        risk: Risk,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the risk (non-empty statement + owner) before committing —
        // the model is never trusted. A risk with no accountable owner cannot be owned/accepted
        // (Principle 11), and `validate()` rejects it.
        risk.validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;

        let mut events = vec![Event::RiskRaised { risk }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_accept_risk(
        &mut self,
        risk: EntityId,
        accepted_by: String,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The target risk must exist — accepting a risk that was never raised is meaningless
        // (P3). Acceptance is a human-authority act (Principle 11); folding flips status ->
        // Accepted. v0 does NOT gate release on risk, so no further lifecycle guard is required.
        if self.state.risk(risk).is_none() {
            return Err(CapabilityError::Rejected(format!(
                "accept targets unknown risk {}",
                risk.short()
            )));
        }
        let seqs = self
            .commit(vec![Event::RiskAccepted { risk, accepted_by }])
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_record_objective(
        &mut self,
        objective: Objective,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the objective (non-empty statement) before committing.
        objective
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // Referential integrity at the seam: an objective's `source` must be a committed entity, so
        // a weighted goal can never dangle (P3, P5).
        if objective.source.is_null() || !self.resolves_to_committed_entity(objective.source) {
            return Err(CapabilityError::Rejected(format!(
                "objective is rooted in unknown source {}",
                objective.source.short()
            )));
        }

        let mut events = vec![Event::ObjectiveRecorded { objective }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_record_tradeoff(
        &mut self,
        tradeoff: Tradeoff,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the tradeoff — >=2 alternatives, `chosen` in range and NOT
        // rejected, and >=1 rejected alternative PRESERVED (exit criterion 3). The model is never
        // trusted to have kept its own rejected space; the runtime re-checks it before committing.
        tradeoff
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;

        let mut events = vec![Event::TradeoffRecorded { tradeoff }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_create_power_domain(
        &mut self,
        domain: PowerDomain,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the power domain (non-empty name, positive voltage, positive
        // max current, >=1 net) before committing — the model is never trusted to have formed a
        // well-shaped rail.
        domain
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // A rail with no supplying component is untraceable to intent (P3) — reject it.
        if domain.source_component.is_null() {
            return Err(CapabilityError::Rejected(
                "power domain has no source component".into(),
            ));
        }
        // Referential integrity at the seam: the source component must be committed (P3, P5).
        if self.state.component(domain.source_component).is_none() {
            return Err(CapabilityError::Rejected(format!(
                "power domain source component {} does not exist",
                domain.source_component.short()
            )));
        }
        // Referential integrity at the seam: every rail net must be a committed net, so the domain
        // can never reference a phantom rail (P3, P5).
        for nid in &domain.nets {
            if self.state.net(*nid).is_none() {
                return Err(CapabilityError::Rejected(format!(
                    "power domain references unknown net {}",
                    nid.short()
                )));
            }
        }

        let mut events = vec![Event::PowerDomainCommitted { domain }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_create_clock_domain(
        &mut self,
        domain: ClockDomain,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the clock domain (non-empty name, positive finite frequency,
        // >=1 member net) before committing — the model is never trusted to have formed a
        // well-shaped clock region.
        domain
            .validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // A clock with no sourcing component is untraceable to intent (P3) — reject it.
        if domain.source_component.is_null() {
            return Err(CapabilityError::Rejected(
                "clock domain has no source component".into(),
            ));
        }
        // Referential integrity at the seam: the source component must be committed (P3, P5).
        if self.state.component(domain.source_component).is_none() {
            return Err(CapabilityError::Rejected(format!(
                "clock domain source component {} does not exist",
                domain.source_component.short()
            )));
        }
        // Referential integrity at the seam: every member net must be a committed net, so the
        // domain can never reference a phantom clock net (P3, P5).
        for nid in &domain.members {
            if self.state.net(*nid).is_none() {
                return Err(CapabilityError::Rejected(format!(
                    "clock domain references unknown net {}",
                    nid.short()
                )));
            }
        }

        let mut events = vec![Event::ClockDomainCommitted { domain }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }

    fn handle_create_return_path(
        &mut self,
        path: ReturnPath,
        links: Vec<ProvenanceLink>,
    ) -> Result<CapabilityAck, CapabilityError> {
        // The seam (P3): re-validate the return path (non-empty name, non-null net, non-null
        // reference plane, net != reference plane) before committing — the model is never trusted
        // to have formed a well-shaped return relationship.
        path.validate()
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        // Referential integrity at the seam: the controlled net must be a committed net (P3, P5).
        if self.state.net(path.net).is_none() {
            return Err(CapabilityError::Rejected(format!(
                "return path references unknown net {}",
                path.net.short()
            )));
        }
        // Referential integrity at the seam: the reference plane must be a committed net — the
        // return path can never reference a phantom return conductor (P3, P5).
        if self.state.net(path.reference_plane).is_none() {
            return Err(CapabilityError::Rejected(format!(
                "return path references unknown reference plane {}",
                path.reference_plane.short()
            )));
        }

        let mut events = vec![Event::ReturnPathCommitted { path }];
        for link in links {
            events.push(Event::ProvenanceLinked { link });
        }
        let seqs = self
            .commit(events)
            .map_err(|e| CapabilityError::Rejected(e.to_string()))?;
        Ok(CapabilityAck { committed: seqs })
    }
}

impl AgentContext for RuntimeCore {
    fn autonomy(&self) -> Autonomy {
        self.autonomy
    }

    fn fresh_id(&mut self) -> EntityId {
        self.ids.fresh()
    }

    fn design_intent(&self) -> Option<DesignIntent> {
        self.state.intent.clone()
    }

    fn requirements(&self) -> Vec<Requirement> {
        self.state.requirements.clone()
    }

    fn provenance_links(&self) -> Vec<ProvenanceLink> {
        self.state.links.clone()
    }

    fn constraints(&self) -> Vec<Constraint> {
        self.state.constraints.clone()
    }

    fn violations(&self) -> Vec<Violation> {
        self.state.violations.clone()
    }

    fn explained_violations(&self) -> Vec<EntityId> {
        self.state
            .violation_explanations
            .iter()
            .map(|e| e.violation)
            .collect()
    }

    fn functional_blocks(&self) -> Vec<FunctionalBlock> {
        self.state.functional_blocks.clone()
    }

    fn components(&self) -> Vec<Component> {
        self.state.components.clone()
    }

    fn pins(&self) -> Vec<Pin> {
        self.state.pins.clone()
    }

    fn nets(&self) -> Vec<Net> {
        self.state.nets.clone()
    }

    fn parts(&self) -> Vec<Part> {
        self.state.parts.clone()
    }

    fn bom_line_items(&self) -> Vec<BomLineItem> {
        self.state.bom_line_items.clone()
    }

    fn board(&self) -> Option<Board> {
        self.state.board.clone()
    }

    fn placements(&self) -> Vec<Placement> {
        self.state.placements.clone()
    }

    fn tracks(&self) -> Vec<Track> {
        self.state.tracks.clone()
    }

    fn assumptions(&self) -> Vec<Assumption> {
        self.state.assumptions.clone()
    }

    fn undischarged_critical_assumptions(&self) -> Vec<Assumption> {
        self.state
            .undischarged_critical_assumptions()
            .into_iter()
            .cloned()
            .collect()
    }

    fn risks(&self) -> Vec<Risk> {
        self.state.risks.clone()
    }

    fn objectives(&self) -> Vec<Objective> {
        self.state.objectives.clone()
    }

    fn tradeoffs(&self) -> Vec<Tradeoff> {
        self.state.tradeoffs.clone()
    }

    fn power_domains(&self) -> Vec<PowerDomain> {
        self.state.power_domains.clone()
    }

    fn clock_domains(&self) -> Vec<ClockDomain> {
        self.state.clock_domains.clone()
    }

    fn return_paths(&self) -> Vec<ReturnPath> {
        self.state.return_paths.clone()
    }

    fn reason(
        &mut self,
        mut req: ReasoningRequest,
    ) -> Result<(Seq, ReasoningResponse), ReasoningError> {
        req.model_id = self.reasoning.model_id();
        let response = self.reasoning.request_judgement(&req)?;
        let event = Event::ReasoningCall {
            request: req,
            response: response.clone(),
        };
        let seqs = self
            .commit(vec![event])
            .map_err(|e| ReasoningError::Provider(e.to_string()))?;
        let seq = *seqs.first().expect("reasoning call produced one event");
        Ok((seq, response))
    }

    fn invoke(&mut self, req: CapabilityRequest) -> Result<CapabilityAck, CapabilityError> {
        match req {
            CapabilityRequest::CreateRequirement {
                requirement,
                decision,
                evidence,
                links,
            } => self.handle_create_requirement(requirement, decision, evidence, links),
            CapabilityRequest::CreateConstraint { constraint, links } => {
                self.handle_create_constraint(constraint, links)
            }
            CapabilityRequest::RaiseViolation { violation, links } => {
                self.handle_raise_violation(violation, links)
            }
            CapabilityRequest::GrantWaiver { waiver } => self.handle_grant_waiver(waiver),
            CapabilityRequest::CreateFunctionalBlock { block, links } => {
                self.handle_create_functional_block(block, links)
            }
            CapabilityRequest::RealizeComponent {
                component,
                pins,
                links,
            } => self.handle_realize_component(component, pins, links),
            CapabilityRequest::CreateNet { net, links } => self.handle_create_net(net, links),
            CapabilityRequest::CreatePart { part, links } => self.handle_create_part(part, links),
            CapabilityRequest::CreateBomLineItem { item, links } => {
                self.handle_create_bom_line_item(item, links)
            }
            CapabilityRequest::CreateBoard { board, links } => {
                self.handle_create_board(board, links)
            }
            CapabilityRequest::PlaceComponent { placement, links } => {
                self.handle_place_component(placement, links)
            }
            CapabilityRequest::RouteNet { track, links } => self.handle_route_net(track, links),
            CapabilityRequest::RaiseAssumption { assumption, links } => {
                self.handle_raise_assumption(assumption, links)
            }
            CapabilityRequest::DischargeAssumption {
                assumption,
                discharge,
            } => self.handle_discharge_assumption(assumption, discharge),
            CapabilityRequest::RaiseRisk { risk, links } => self.handle_raise_risk(risk, links),
            CapabilityRequest::AcceptRisk { risk, accepted_by } => {
                self.handle_accept_risk(risk, accepted_by)
            }
            CapabilityRequest::RecordObjective { objective, links } => {
                self.handle_record_objective(objective, links)
            }
            CapabilityRequest::RecordTradeoff { tradeoff, links } => {
                self.handle_record_tradeoff(tradeoff, links)
            }
            CapabilityRequest::CreatePowerDomain { domain, links } => {
                self.handle_create_power_domain(domain, links)
            }
            CapabilityRequest::CreateClockDomain { domain, links } => {
                self.handle_create_clock_domain(domain, links)
            }
            CapabilityRequest::CreateReturnPath { path, links } => {
                self.handle_create_return_path(path, links)
            }
        }
    }

    fn emit(&mut self, events: Vec<Event>) -> Result<Vec<Seq>, StoreError> {
        self.commit(events)
    }
}
