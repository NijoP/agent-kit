//! The Engineering Runtime — Electronics Agent Kit's deterministic kernel (Use-case ring).
//!
//! This crate is the *mechanism* + *policy* of the architecture (P7): shared state, the
//! commit path, the FSM framework, the execution engine, orchestration, and replay. It
//! depends only on inner rings (`eak-ports`, `eak-domain`) — never on an adapter — which
//! the [`dependency_rule`] guard test enforces at build time (P1).

pub mod clock;
pub mod fsm;
pub mod orchestrator;
pub mod protocol;
pub mod replay;
mod runtime_core;
pub mod state;

pub use clock::{Clock, IdSource, LogicalClock, SeededIdSource, SystemClock};
pub use fsm::{ExecutionEngine, Machine, MachineError, PhaseOutcome, StateKind, StepResult};
pub use orchestrator::{LoopBack, Orchestrator, WorkflowPlan};
pub use protocol::{
    Agent, AgentActivation, AgentContext, AgentOutcome, Autonomy, Budget, CapabilityAck,
    CapabilityError, CapabilityRequest,
};
pub use replay::replay;
pub use runtime_core::RuntimeCore;
pub use state::{EngineeringState, FidelityTag, ViolationExplanation};

#[cfg(test)]
mod dependency_rule {
    /// The kernel must depend only on inner rings, never on an adapter or instance crate.
    /// If a future change adds such a dependency, this test fails the build (P1).
    #[test]
    fn kernel_has_no_outward_dependencies() {
        let manifest = include_str!("../Cargo.toml");
        for forbidden in [
            "eak-store",
            "eak-reasoning",
            "eak-phases",
            "eak-cli",
            "eak-engines",
            "eak-compiler",
        ] {
            assert!(
                !manifest.contains(forbidden),
                "dependency rule violated: eak-runtime must not depend on {forbidden}"
            );
        }
    }
}

#[cfg(test)]
mod kernel_tests {
    use super::*;
    use eak_domain::{
        Alternative, Assumption, AssumptionCriticality, AssumptionStatus, Board, BoardSide,
        BomLineItem, Component, ComponentClass, ComponentOrigin, Decision, Discharge,
        DischargeResolution, EntityId, FidelityMethod, FunctionalBlock, LayerStack, ModelFidelity,
        Net, NetClass, NetOrigin, Objective, Part, PartLifecycle, Pin, PinElectricalType,
        Placement, PowerDomain, Priority, Requirement, RequirementCategory, RequirementStatus,
        Risk, RiskLikelihood, RiskSeverity, RiskStatus, Track, Tradeoff,
    };
    use eak_ports::{
        Event, EventLog, EventRecord, ReasoningEngine, ReasoningError, ReasoningRequest,
        ReasoningResponse, Seq, StoreError, Timestamp,
    };
    use eak_units::{PhysicalQuantity, Unit};

    struct MemLog {
        records: Vec<EventRecord>,
    }
    impl EventLog for MemLog {
        fn append(&mut self, events: &[(Timestamp, Event)]) -> Result<Vec<Seq>, StoreError> {
            let mut seqs = Vec::new();
            for (ts, ev) in events {
                let seq = self.records.len() as u64;
                self.records.push(EventRecord {
                    seq,
                    timestamp: *ts,
                    event: ev.clone(),
                });
                seqs.push(seq);
            }
            Ok(seqs)
        }
        fn read_all(&self) -> Result<Vec<EventRecord>, StoreError> {
            Ok(self.records.clone())
        }
        fn next_seq(&self) -> Seq {
            self.records.len() as u64
        }
    }

    struct NullReasoner;
    impl ReasoningEngine for NullReasoner {
        fn model_id(&self) -> String {
            "null".into()
        }
        fn request_judgement(
            &self,
            _req: &ReasoningRequest,
        ) -> Result<ReasoningResponse, ReasoningError> {
            Ok(ReasoningResponse {
                candidates: vec![],
                part_candidates: vec![],
                explanations: vec![],
                clarifying_questions: vec![],
                raw: String::new(),
            })
        }
    }

    #[test]
    fn commit_then_replay_reconstructs_identical_state() {
        let mut core = RuntimeCore::new(
            Box::new(MemLog { records: vec![] }),
            Box::new(NullReasoner),
            Box::new(SeededIdSource::new(42)),
            Box::new(LogicalClock::new()),
            Autonomy::Autonomous,
        );
        core.capture_intent("USB-C powered IoT sensor node, < 5 W", "engineer")
            .unwrap();
        let src = core.state.intent.as_ref().unwrap().id;
        let rid = core.fresh_id();
        let did = core.fresh_id();
        let req = Requirement {
            id: rid,
            statement: "Operating power shall not exceed 5 W".into(),
            category: RequirementCategory::Electrical,
            priority: Priority::High,
            acceptance_criterion: "measured power < 5 W".into(),
            status: RequirementStatus::Accepted,
            source: src,
            targets: vec![],
        };
        let dec = Decision {
            id: did,
            subject: rid,
            rationale: "derived from intent".into(),
            decider: "test".into(),
            reasoning_call_seq: None,
            evidence: vec![],
            confidence: 1.0,
        };
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: req,
            decision: dec,
            evidence: vec![],
            links: vec![],
        })
        .unwrap();

        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
        assert_eq!(core.state.requirements.len(), 1);
    }

    #[test]
    fn event_sink_observes_every_committed_event_live_in_order() {
        use std::cell::RefCell;
        use std::rc::Rc;

        // A sink that clones each committed record into a shared buffer the instant it commits.
        struct CollectingSink(Rc<RefCell<Vec<EventRecord>>>);
        impl eak_ports::EventSink for CollectingSink {
            fn on_committed(&mut self, record: &EventRecord) {
                self.0.borrow_mut().push(record.clone());
            }
        }

        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut core = RuntimeCore::new(
            Box::new(MemLog { records: vec![] }),
            Box::new(NullReasoner),
            Box::new(SeededIdSource::new(7)),
            Box::new(LogicalClock::new()),
            Autonomy::Autonomous,
        )
        .with_sink(Box::new(CollectingSink(Rc::clone(&seen))));

        // The sink must already hold the intent event the moment capture_intent commits — i.e.
        // mid-run, not batched at the end of a workflow.
        core.capture_intent("USB-C powered IoT sensor node, < 5 W", "engineer")
            .unwrap();
        assert_eq!(seen.borrow().len(), 1, "sink saw the intent event live");
        assert!(matches!(
            seen.borrow()[0].event,
            Event::IntentCaptured { .. }
        ));

        // A multi-event commit (evidence?/decision/requirement) keeps streaming in order.
        let src = core.state.intent.as_ref().unwrap().id;
        let rid = core.fresh_id();
        let did = core.fresh_id();
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: Requirement {
                id: rid,
                statement: "Operating power shall not exceed 5 W".into(),
                category: RequirementCategory::Electrical,
                priority: Priority::High,
                acceptance_criterion: "measured power < 5 W".into(),
                status: RequirementStatus::Accepted,
                source: src,
                targets: vec![],
            },
            decision: Decision {
                id: did,
                subject: rid,
                rationale: "derived from intent".into(),
                decider: "test".into(),
                reasoning_call_seq: None,
                evidence: vec![],
                confidence: 1.0,
            },
            evidence: vec![],
            links: vec![],
        })
        .unwrap();

        // The streamed events must match the authoritative log exactly, seq-aligned and in order —
        // proving every committed event reached the observer once, live (the UI's contract).
        let logged = core.log().read_all().unwrap();
        let streamed = seen.borrow();
        assert_eq!(streamed.len(), logged.len());
        assert!(streamed.len() >= 2, "intent + at least the requirement");
        for (streamed_rec, logged_rec) in streamed.iter().zip(logged.iter()) {
            assert_eq!(streamed_rec.seq, logged_rec.seq);
            assert_eq!(streamed_rec.event, logged_rec.event);
        }
    }

    fn new_core() -> RuntimeCore {
        RuntimeCore::new(
            Box::new(MemLog { records: vec![] }),
            Box::new(NullReasoner),
            Box::new(SeededIdSource::new(7)),
            Box::new(LogicalClock::new()),
            Autonomy::Autonomous,
        )
    }

    /// The B1 ruling: the ≥1-pin connectivity rule is an *origin-dependent* seam invariant. A
    /// `Physical` (imported-copper) net may be member-less; a `Logical` (synthesised) net may not;
    /// and neither origin may reference a pin that was never committed (Physical relaxes the count,
    /// never the no-phantom-terminal rule). No board is needed — `CreateNet` gates on the net alone.
    #[test]
    fn create_net_seam_applies_origin_dependent_connectivity_invariant() {
        let mut core = new_core();
        let phys_id = core.fresh_id();
        let logi_id = core.fresh_id();
        let phantom_id = core.fresh_id();

        // Physical + member-less: ACCEPTED — imported copper carries no parsed pins yet.
        core.invoke(CapabilityRequest::CreateNet {
            net: Net {
                id: phys_id,
                name: "GND".into(),
                class: NetClass::Ground,
                members: vec![],
                current: None,
                impedance_target: None,
                origin: NetOrigin::Physical,
            },
            links: vec![],
        })
        .expect("a member-less Physical net is accepted at the seam");
        assert_eq!(core.state.nets.len(), 1);

        // Logical + member-less: REJECTED — the synthesis connectivity invariant is intact.
        let err = core
            .invoke(CapabilityRequest::CreateNet {
                net: Net {
                    id: logi_id,
                    name: "VCC".into(),
                    class: NetClass::Power,
                    members: vec![],
                    current: None,
                    impedance_target: None,
                    origin: NetOrigin::Logical,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Physical but listing a pin that was never committed: REJECTED — no fabricated terminal.
        let err = core
            .invoke(CapabilityRequest::CreateNet {
                net: Net {
                    id: phantom_id,
                    name: "SDA".into(),
                    class: NetClass::Signal,
                    members: vec![EntityId(0xDEAD_BEEF)],
                    current: None,
                    impedance_target: None,
                    origin: NetOrigin::Physical,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Only the accepted Physical net folded into state.
        assert_eq!(core.state.nets.len(), 1);
    }

    /// The F2 ruling (ADR-0017): the block-origin rule at `RealizeComponent` is *origin-dependent*,
    /// mirroring the B1 net test above. An `Imported` component (parsed from a finished board, which
    /// declares no functional decomposition) may carry a null `from_block`; a `Synthesized` one may
    /// not; and an `Imported` one that DOES name a block must name a committed one (existence relaxed,
    /// no-phantom rule intact). The ≥1-pin rule holds for both. No board is needed — `RealizeComponent`
    /// gates on the component + pins alone (placement is a separate seam).
    #[test]
    fn realize_component_seam_applies_origin_dependent_block_invariant() {
        let mut core = new_core();
        let imported_id = core.fresh_id();
        let imported_pin = core.fresh_id();
        let synth_id = core.fresh_id();
        let synth_pin = core.fresh_id();
        let phantom_comp = core.fresh_id();
        let phantom_pin = core.fresh_id();

        let pin = |id: EntityId, comp: EntityId| Pin {
            id,
            component: comp,
            designation: "1".into(),
            electrical_type: PinElectricalType::Passive,
        };

        // Imported + null block + a real pin: ACCEPTED — an imported board declares no block, and the
        // seam never fabricates one.
        core.invoke(CapabilityRequest::RealizeComponent {
            component: Component {
                id: imported_id,
                refdes: "R1".into(),
                class: ComponentClass::Resistor,
                value: None,
                from_block: EntityId::NULL,
                origin: ComponentOrigin::Imported,
            },
            pins: vec![pin(imported_pin, imported_id)],
            links: vec![],
        })
        .expect("an Imported component with a null block is accepted at the seam");
        assert_eq!(core.state.components.len(), 1);

        // Synthesized + null block: REJECTED — the synthesis traceability invariant is intact.
        let err = core
            .invoke(CapabilityRequest::RealizeComponent {
                component: Component {
                    id: synth_id,
                    refdes: "U1".into(),
                    class: ComponentClass::Ic,
                    value: None,
                    from_block: EntityId::NULL,
                    origin: ComponentOrigin::Synthesized,
                },
                pins: vec![pin(synth_pin, synth_id)],
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Imported but naming a block that was never committed: REJECTED — no phantom block.
        let err = core
            .invoke(CapabilityRequest::RealizeComponent {
                component: Component {
                    id: phantom_comp,
                    refdes: "U2".into(),
                    class: ComponentClass::Ic,
                    value: None,
                    from_block: EntityId(0xDEAD_BEEF),
                    origin: ComponentOrigin::Imported,
                },
                pins: vec![pin(phantom_pin, phantom_comp)],
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Only the accepted Imported component folded into state.
        assert_eq!(core.state.components.len(), 1);
    }

    #[test]
    fn phase3_synthesis_commits_fold_and_replay_byte_identically() {
        let mut core = new_core();
        // Commit a real requirement first: the seam now enforces that a block's referenced
        // requirements exist, so the synthesis chain must start from committed intent.
        core.capture_intent("USB-C powered IoT sensor node, 5 V rail", "engineer")
            .unwrap();
        let src = core.state.intent.as_ref().unwrap().id;
        let rid = core.fresh_id();
        let did = core.fresh_id();
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: Requirement {
                id: rid,
                statement: "Device shall regulate to 5 V".into(),
                category: RequirementCategory::Electrical,
                priority: Priority::High,
                acceptance_criterion: "rail measures 5 V".into(),
                status: RequirementStatus::Accepted,
                source: src,
                targets: vec![],
            },
            decision: Decision {
                id: did,
                subject: rid,
                rationale: "from intent".into(),
                decider: "test".into(),
                reasoning_call_seq: None,
                evidence: vec![],
                confidence: 1.0,
            },
            evidence: vec![],
            links: vec![],
        })
        .unwrap();
        let block = FunctionalBlock {
            id: core.fresh_id(),
            name: "5V regulation".into(),
            function: "step USB-C VBUS down to 5 V".into(),
            requirements: vec![rid],
        };
        let bid = block.id;
        core.invoke(CapabilityRequest::CreateFunctionalBlock {
            block,
            links: vec![],
        })
        .unwrap();

        let comp = Component {
            id: core.fresh_id(),
            refdes: "U1".into(),
            class: ComponentClass::Regulator,
            value: None,
            from_block: bid,
            origin: ComponentOrigin::Synthesized,
        };
        let cid = comp.id;
        let pin = Pin {
            id: core.fresh_id(),
            component: cid,
            designation: "VOUT".into(),
            electrical_type: PinElectricalType::PowerOut,
        };
        let pid = pin.id;
        core.invoke(CapabilityRequest::RealizeComponent {
            component: comp,
            pins: vec![pin],
            links: vec![],
        })
        .unwrap();

        let nid = core.fresh_id();
        core.invoke(CapabilityRequest::CreateNet {
            net: Net {
                id: nid,
                name: "+5V".into(),
                class: NetClass::Power,
                members: vec![pid],
                current: None,
                impedance_target: None,
                origin: NetOrigin::Logical,
            },
            links: vec![],
        })
        .unwrap();

        assert_eq!(core.state.functional_blocks.len(), 1);
        assert_eq!(core.state.components.len(), 1);
        assert_eq!(core.state.pins.len(), 1);
        assert_eq!(core.state.nets.len(), 1);
        assert!(core.state.functional_block(bid).is_some());
        assert!(core.state.component(cid).is_some());
        assert!(core.state.pin(pid).is_some());

        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    /// Drive the full chain down into the BOM layer: requirement -> block -> component, then
    /// a [`Part`] and a [`BomLineItem`] binding the part to that real component. Asserts the
    /// fold landed and that replay reconstructs byte-identical state (the Phase-1 exit
    /// criterion, extended to the BOM deltas).
    #[test]
    fn phase3_bom_commits_fold_and_replay_byte_identically() {
        let mut core = new_core();
        core.capture_intent("USB-C powered IoT sensor node, 3.3 V rail", "engineer")
            .unwrap();
        let src = core.state.intent.as_ref().unwrap().id;
        let rid = core.fresh_id();
        let did = core.fresh_id();
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: Requirement {
                id: rid,
                statement: "Device shall regulate to 3.3 V".into(),
                category: RequirementCategory::Electrical,
                priority: Priority::High,
                acceptance_criterion: "rail measures 3.3 V".into(),
                status: RequirementStatus::Accepted,
                source: src,
                targets: vec![],
            },
            decision: Decision {
                id: did,
                subject: rid,
                rationale: "from intent".into(),
                decider: "test".into(),
                reasoning_call_seq: None,
                evidence: vec![],
                confidence: 1.0,
            },
            evidence: vec![],
            links: vec![],
        })
        .unwrap();

        let block = FunctionalBlock {
            id: core.fresh_id(),
            name: "3V3 regulation".into(),
            function: "step VBUS down to 3.3 V".into(),
            requirements: vec![rid],
        };
        let bid = block.id;
        core.invoke(CapabilityRequest::CreateFunctionalBlock {
            block,
            links: vec![],
        })
        .unwrap();

        let comp = Component {
            id: core.fresh_id(),
            refdes: "U1".into(),
            class: ComponentClass::Regulator,
            value: None,
            from_block: bid,
            origin: ComponentOrigin::Synthesized,
        };
        let cid = comp.id;
        let pin = Pin {
            id: core.fresh_id(),
            component: cid,
            designation: "VOUT".into(),
            electrical_type: PinElectricalType::PowerOut,
        };
        core.invoke(CapabilityRequest::RealizeComponent {
            component: comp,
            pins: vec![pin],
            links: vec![],
        })
        .unwrap();

        // The BOM layer: a concrete part, then a line binding it to the real component.
        let part = Part {
            id: core.fresh_id(),
            mpn: "LM1117-3.3".into(),
            manufacturer: "Texas Instruments".into(),
            lifecycle: PartLifecycle::Eol,
            datasheet: "https://ti.com/lm1117".into(),
        };
        let part_id = part.id;
        core.invoke(CapabilityRequest::CreatePart {
            part,
            links: vec![],
        })
        .unwrap();

        let item = BomLineItem {
            id: core.fresh_id(),
            part: part_id,
            components: vec![cid],
            quantity: 1,
        };
        let item_id = item.id;
        core.invoke(CapabilityRequest::CreateBomLineItem {
            item,
            links: vec![],
        })
        .unwrap();

        assert_eq!(core.state.parts.len(), 1);
        assert_eq!(core.state.bom_line_items.len(), 1);
        assert!(core.state.part(part_id).is_some());
        assert!(core.state.bom_line_item(item_id).is_some());

        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    /// Drive the full chain down into the PCB layer: requirement -> block -> component, then
    /// a [`Board`] outline and a [`Placement`] of that real component on it. Asserts the fold
    /// landed (board + placement) and that replay reconstructs byte-identical state (the
    /// Phase-1 exit criterion, extended to the PCB deltas).
    #[test]
    fn phase3_pcb_commits_fold_and_replay_byte_identically() {
        let mut core = new_core();
        core.capture_intent(
            "USB-C powered IoT sensor node, fits a 50x40 mm board",
            "engineer",
        )
        .unwrap();
        let src = core.state.intent.as_ref().unwrap().id;
        let rid = core.fresh_id();
        let did = core.fresh_id();
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: Requirement {
                id: rid,
                statement: "Device shall fit a 50x40 mm outline".into(),
                category: RequirementCategory::Electrical,
                priority: Priority::High,
                acceptance_criterion: "board <= 50x40 mm".into(),
                status: RequirementStatus::Accepted,
                source: src,
                targets: vec![],
            },
            decision: Decision {
                id: did,
                subject: rid,
                rationale: "from intent".into(),
                decider: "test".into(),
                reasoning_call_seq: None,
                evidence: vec![],
                confidence: 1.0,
            },
            evidence: vec![],
            links: vec![],
        })
        .unwrap();

        let block = FunctionalBlock {
            id: core.fresh_id(),
            name: "3V3 regulation".into(),
            function: "step VBUS down to 3.3 V".into(),
            requirements: vec![rid],
        };
        let bid = block.id;
        core.invoke(CapabilityRequest::CreateFunctionalBlock {
            block,
            links: vec![],
        })
        .unwrap();

        let comp = Component {
            id: core.fresh_id(),
            refdes: "U1".into(),
            class: ComponentClass::Regulator,
            value: None,
            from_block: bid,
            origin: ComponentOrigin::Synthesized,
        };
        let cid = comp.id;
        let pin = Pin {
            id: core.fresh_id(),
            component: cid,
            designation: "VOUT".into(),
            electrical_type: PinElectricalType::PowerOut,
        };
        core.invoke(CapabilityRequest::RealizeComponent {
            component: comp,
            pins: vec![pin],
            links: vec![],
        })
        .unwrap();

        // The PCB layer: the board outline must precede any placement (seam ordering, P5).
        let board = Board {
            id: core.fresh_id(),
            width: PhysicalQuantity::new(50.0, Unit::Millimetre),
            height: PhysicalQuantity::new(40.0, Unit::Millimetre),
            stack: LayerStack::standard_two_layer(),
        };
        let board_id = board.id;
        core.invoke(CapabilityRequest::CreateBoard {
            board,
            links: vec![],
        })
        .unwrap();

        let placement = Placement {
            id: core.fresh_id(),
            component: cid,
            x: PhysicalQuantity::new(5.0, Unit::Millimetre),
            y: PhysicalQuantity::new(5.0, Unit::Millimetre),
            width: PhysicalQuantity::new(10.0, Unit::Millimetre),
            height: PhysicalQuantity::new(8.0, Unit::Millimetre),
            side: BoardSide::Top,
        };
        let placement_id = placement.id;
        core.invoke(CapabilityRequest::PlaceComponent {
            placement,
            links: vec![],
        })
        .unwrap();

        assert!(core.state.board.is_some());
        assert_eq!(core.state.placements.len(), 1);
        assert!(core.state.board().is_some());
        assert_eq!(core.state.board().unwrap().id, board_id);
        assert!(core.state.placement(placement_id).is_some());

        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    /// The BOM seam mirrors the synthesis seam: a line with zero quantity, an empty component
    /// list, an unknown part, or an unknown component is rejected before the commit path.
    #[test]
    fn bom_line_item_handler_rejects_untraceable_proposals_at_the_seam() {
        let mut core = new_core();
        // Pre-mint every id (fresh_id borrows the core mutably, so it cannot run inside an
        // `invoke` argument).
        let fake_part = core.fresh_id();
        let fake_comp = core.fresh_id();
        let pid = core.fresh_id();
        let (id1, id2, id3, id4) = (
            core.fresh_id(),
            core.fresh_id(),
            core.fresh_id(),
            core.fresh_id(),
        );

        // A committed part to isolate the component-integrity check below.
        core.invoke(CapabilityRequest::CreatePart {
            part: Part {
                id: pid,
                mpn: "RC0402FR-0710KL".into(),
                manufacturer: "Yageo".into(),
                lifecycle: PartLifecycle::Active,
                datasheet: "https://yageo.com/rc0402".into(),
            },
            links: vec![],
        })
        .unwrap();

        // Zero quantity.
        let err = core
            .invoke(CapabilityRequest::CreateBomLineItem {
                item: BomLineItem {
                    id: id1,
                    part: pid,
                    components: vec![fake_comp],
                    quantity: 0,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // No components covered.
        let err = core
            .invoke(CapabilityRequest::CreateBomLineItem {
                item: BomLineItem {
                    id: id2,
                    part: pid,
                    components: vec![],
                    quantity: 1,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Unknown part.
        let err = core
            .invoke(CapabilityRequest::CreateBomLineItem {
                item: BomLineItem {
                    id: id3,
                    part: fake_part,
                    components: vec![fake_comp],
                    quantity: 1,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Known part but unknown component.
        let err = core
            .invoke(CapabilityRequest::CreateBomLineItem {
                item: BomLineItem {
                    id: id4,
                    part: pid,
                    components: vec![fake_comp],
                    quantity: 1,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Nothing landed in the BOM line-item store.
        assert!(core.state.bom_line_items.is_empty());
    }

    #[test]
    fn phase3_handlers_reject_untraceable_proposals_at_the_seam() {
        let mut core = new_core();
        let (id1, id2, id3) = (core.fresh_id(), core.fresh_id(), core.fresh_id());

        // A block realizing no requirement is untraceable to intent (P3).
        let err = core
            .invoke(CapabilityRequest::CreateFunctionalBlock {
                block: FunctionalBlock {
                    id: id1,
                    name: "orphan".into(),
                    function: "f".into(),
                    requirements: vec![],
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // A component minted from no block has no upstream trace.
        let err = core
            .invoke(CapabilityRequest::RealizeComponent {
                component: Component {
                    id: id2,
                    refdes: "R1".into(),
                    class: ComponentClass::Resistor,
                    value: None,
                    from_block: EntityId::NULL,
                    origin: ComponentOrigin::Synthesized,
                },
                pins: vec![],
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // A net joining no pins carries no connectivity.
        let err = core
            .invoke(CapabilityRequest::CreateNet {
                net: Net {
                    id: id3,
                    name: "GND".into(),
                    class: NetClass::Ground,
                    members: vec![],
                    current: None,
                    impedance_target: None,
                    origin: NetOrigin::Logical,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Nothing was committed: every proposal was rejected before the commit path.
        assert!(core.state.functional_blocks.is_empty());
        assert!(core.state.components.is_empty());
        assert!(core.state.nets.is_empty());
    }

    #[test]
    fn phase3_pcb_handlers_reject_untraceable_proposals_at_the_seam() {
        let mut core = new_core();

        // Placing a component before any board outline exists is rejected (board precedes
        // placement — there is nothing to fit against).
        let (pid0, cid0) = (core.fresh_id(), core.fresh_id());
        let err = core
            .invoke(CapabilityRequest::PlaceComponent {
                placement: Placement {
                    id: pid0,
                    component: cid0,
                    x: PhysicalQuantity::new(0.0, Unit::Millimetre),
                    y: PhysicalQuantity::new(0.0, Unit::Millimetre),
                    width: PhysicalQuantity::new(1.0, Unit::Millimetre),
                    height: PhysicalQuantity::new(1.0, Unit::Millimetre),
                    side: BoardSide::Top,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Commit a board, then reject a second — a design has exactly one outline.
        let board0 = core.fresh_id();
        core.invoke(CapabilityRequest::CreateBoard {
            board: Board {
                id: board0,
                width: PhysicalQuantity::new(50.0, Unit::Millimetre),
                height: PhysicalQuantity::new(50.0, Unit::Millimetre),
                stack: LayerStack::standard_two_layer(),
            },
            links: vec![],
        })
        .unwrap();
        let board1 = core.fresh_id();
        let err = core
            .invoke(CapabilityRequest::CreateBoard {
                board: Board {
                    id: board1,
                    width: PhysicalQuantity::new(20.0, Unit::Millimetre),
                    height: PhysicalQuantity::new(20.0, Unit::Millimetre),
                    stack: LayerStack::standard_two_layer(),
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Placing an unrealized component is rejected (referential integrity).
        let (pid1, cid1) = (core.fresh_id(), core.fresh_id());
        let err = core
            .invoke(CapabilityRequest::PlaceComponent {
                placement: Placement {
                    id: pid1,
                    component: cid1,
                    x: PhysicalQuantity::new(1.0, Unit::Millimetre),
                    y: PhysicalQuantity::new(1.0, Unit::Millimetre),
                    width: PhysicalQuantity::new(1.0, Unit::Millimetre),
                    height: PhysicalQuantity::new(1.0, Unit::Millimetre),
                    side: BoardSide::Top,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Build a real requirement -> block -> component, place it once, then reject a second
        // placement of the same component (a component is placed exactly once).
        core.capture_intent("intent", "engineer").unwrap();
        let src = core.state.intent.as_ref().unwrap().id;
        let rid = core.fresh_id();
        let did = core.fresh_id();
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: Requirement {
                id: rid,
                statement: "Device shall do a thing".into(),
                category: RequirementCategory::Functional,
                priority: Priority::High,
                acceptance_criterion: "it does the thing".into(),
                status: RequirementStatus::Accepted,
                source: src,
                targets: vec![],
            },
            decision: Decision {
                id: did,
                subject: rid,
                rationale: "from intent".into(),
                decider: "test".into(),
                reasoning_call_seq: None,
                evidence: vec![],
                confidence: 1.0,
            },
            evidence: vec![],
            links: vec![],
        })
        .unwrap();
        let block_id = core.fresh_id();
        core.invoke(CapabilityRequest::CreateFunctionalBlock {
            block: FunctionalBlock {
                id: block_id,
                name: "blk".into(),
                function: "f".into(),
                requirements: vec![rid],
            },
            links: vec![],
        })
        .unwrap();
        let comp_id = core.fresh_id();
        let pin_id = core.fresh_id();
        core.invoke(CapabilityRequest::RealizeComponent {
            component: Component {
                id: comp_id,
                refdes: "U1".into(),
                class: ComponentClass::Ic,
                value: None,
                from_block: block_id,
                origin: ComponentOrigin::Synthesized,
            },
            pins: vec![Pin {
                id: pin_id,
                component: comp_id,
                designation: "VDD".into(),
                electrical_type: PinElectricalType::PowerIn,
            }],
            links: vec![],
        })
        .unwrap();

        let place = |id: EntityId| CapabilityRequest::PlaceComponent {
            placement: Placement {
                id,
                component: comp_id,
                x: PhysicalQuantity::new(2.0, Unit::Millimetre),
                y: PhysicalQuantity::new(2.0, Unit::Millimetre),
                width: PhysicalQuantity::new(3.0, Unit::Millimetre),
                height: PhysicalQuantity::new(3.0, Unit::Millimetre),
                side: BoardSide::Top,
            },
            links: vec![],
        };
        let p1 = core.fresh_id();
        core.invoke(place(p1)).unwrap();
        let p2 = core.fresh_id();
        let err = core.invoke(place(p2)).unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Exactly one placement landed, on the single committed board.
        assert_eq!(core.state.placements.len(), 1);
        assert!(core.state.board.is_some());
    }

    /// The routing seam mirrors the placement seam: a track is rejected before any board
    /// exists and when it realizes an unknown net. A net is realized by a daisy-chain, so a
    /// well-formed track for a committed net on a committed board commits — and a second
    /// segment for the same net is accepted too (the seam no longer caps the per-net count).
    #[test]
    fn phase3_routing_handler_rejects_untraceable_proposals_at_the_seam() {
        let mut core = new_core();
        let mm = |v: f64| PhysicalQuantity::new(v, Unit::Millimetre);
        let track = |id: EntityId, net: EntityId| CapabilityRequest::RouteNet {
            track: Track {
                id,
                net,
                layer: BoardSide::Top,
                width: mm(0.25),
                x1: mm(1.0),
                y1: mm(1.0),
                x2: mm(9.0),
                y2: mm(1.0),
            },
            links: vec![],
        };

        // Build a real requirement -> block -> component (+ pin) -> net to route.
        core.capture_intent("intent", "engineer").unwrap();
        let src = core.state.intent.as_ref().unwrap().id;
        let rid = core.fresh_id();
        let did = core.fresh_id();
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: Requirement {
                id: rid,
                statement: "Device shall do a thing".into(),
                category: RequirementCategory::Functional,
                priority: Priority::High,
                acceptance_criterion: "it does the thing".into(),
                status: RequirementStatus::Accepted,
                source: src,
                targets: vec![],
            },
            decision: Decision {
                id: did,
                subject: rid,
                rationale: "from intent".into(),
                decider: "test".into(),
                reasoning_call_seq: None,
                evidence: vec![],
                confidence: 1.0,
            },
            evidence: vec![],
            links: vec![],
        })
        .unwrap();
        let block_id = core.fresh_id();
        core.invoke(CapabilityRequest::CreateFunctionalBlock {
            block: FunctionalBlock {
                id: block_id,
                name: "blk".into(),
                function: "f".into(),
                requirements: vec![rid],
            },
            links: vec![],
        })
        .unwrap();
        let comp_id = core.fresh_id();
        let pin_id = core.fresh_id();
        core.invoke(CapabilityRequest::RealizeComponent {
            component: Component {
                id: comp_id,
                refdes: "U1".into(),
                class: ComponentClass::Ic,
                value: None,
                from_block: block_id,
                origin: ComponentOrigin::Synthesized,
            },
            pins: vec![Pin {
                id: pin_id,
                component: comp_id,
                designation: "VDD".into(),
                electrical_type: PinElectricalType::PowerIn,
            }],
            links: vec![],
        })
        .unwrap();
        let net_id = core.fresh_id();
        core.invoke(CapabilityRequest::CreateNet {
            net: Net {
                id: net_id,
                name: "VBUS".into(),
                class: NetClass::Power,
                members: vec![pin_id],
                current: None,
                impedance_target: None,
                origin: NetOrigin::Logical,
            },
            links: vec![],
        })
        .unwrap();

        // Routing before any board outline exists is rejected (board precedes routing).
        let t0 = core.fresh_id();
        let err = core.invoke(track(t0, net_id)).unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Commit the board, then routing a phantom net is rejected (referential integrity).
        let board_id = core.fresh_id();
        core.invoke(CapabilityRequest::CreateBoard {
            board: Board {
                id: board_id,
                width: mm(50.0),
                height: mm(50.0),
                stack: LayerStack::standard_two_layer(),
            },
            links: vec![],
        })
        .unwrap();
        let t1 = core.fresh_id();
        let phantom = core.fresh_id();
        let err = core.invoke(track(t1, phantom)).unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Routing the real net succeeds; a net is realized by a daisy-chain, so a SECOND track
        // for the same net is also accepted (the seam no longer caps the per-net track count).
        let t2 = core.fresh_id();
        core.invoke(track(t2, net_id)).unwrap();
        let t3 = core.fresh_id();
        core.invoke(track(t3, net_id)).unwrap();

        // Both segments landed; replay reconstructs byte-identical state.
        assert_eq!(core.state.tracks.len(), 2);
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    // ===================== Band A (increment 1): Assumption seam =====================
    //
    // TDD (written before the seam handlers exist): the epistemic honesty object flows
    // through the same seam as every Phase-2/3 object. RaiseAssumption re-validates and
    // checks `rests_on` referential integrity (P3); DischargeAssumption requires an Open
    // target and a resolvable discharge target. The fold pushes on raise and flips
    // status/discharge on discharge — and the whole log replays byte-identically (P4).

    /// Commit a real requirement so an assumption has a committed entity to rest on. Returns
    /// the requirement id (the resolvable `rests_on`/discharge target for the seam checks).
    fn seed_requirement(core: &mut RuntimeCore) -> EntityId {
        core.capture_intent("USB-C powered IoT sensor node, 3.3 V rail", "engineer")
            .unwrap();
        let src = core.state.intent.as_ref().unwrap().id;
        let rid = core.fresh_id();
        let did = core.fresh_id();
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: Requirement {
                id: rid,
                statement: "Device shall regulate to 3.3 V".into(),
                category: RequirementCategory::Electrical,
                priority: Priority::High,
                acceptance_criterion: "rail measures 3.3 V".into(),
                status: RequirementStatus::Accepted,
                source: src,
                targets: vec![],
            },
            decision: Decision {
                id: did,
                subject: rid,
                rationale: "from intent".into(),
                decider: "test".into(),
                reasoning_call_seq: None,
                evidence: vec![],
                confidence: 1.0,
            },
            evidence: vec![],
            links: vec![],
        })
        .unwrap();
        rid
    }

    #[test]
    fn raise_assumption_seam_accepts_valid_then_folds_and_replays_byte_identically() {
        let mut core = new_core();
        let rid = seed_requirement(&mut core);

        let aid = core.fresh_id();
        core.invoke(CapabilityRequest::RaiseAssumption {
            assumption: Assumption {
                id: aid,
                statement: "the USB-C source can deliver 5 V @ 3 A".into(),
                rests_on: rid,
                criticality: AssumptionCriticality::Critical,
                status: AssumptionStatus::Open,
                discharge: None,
            },
            links: vec![],
        })
        .expect("a well-formed assumption resting on a committed requirement is accepted");

        // The fold pushed exactly one Open assumption and the query reads it back.
        assert_eq!(core.state.assumptions.len(), 1);
        let a = core.state.assumption(aid).expect("assumption folded by id");
        assert!(a.is_open());
        assert!(a.is_blocking());
        assert_eq!(core.state.undischarged_critical_assumptions().len(), 1);

        // Byte-identical replay with the assumption in the log (P4).
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    #[test]
    fn raise_assumption_seam_rejects_malformed_and_dangling_proposals() {
        let mut core = new_core();
        let rid = seed_requirement(&mut core);
        let (a_empty, a_null, a_phantom, a_born_discharged) = (
            core.fresh_id(),
            core.fresh_id(),
            core.fresh_id(),
            core.fresh_id(),
        );

        // (1) Empty statement — domain validate() rejects.
        let err = core
            .invoke(CapabilityRequest::RaiseAssumption {
                assumption: Assumption {
                    id: a_empty,
                    statement: "   ".into(),
                    rests_on: rid,
                    criticality: AssumptionCriticality::Normal,
                    status: AssumptionStatus::Open,
                    discharge: None,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (2) Null rests_on — an untraceable assumption is rejected at the seam (P3).
        let err = core
            .invoke(CapabilityRequest::RaiseAssumption {
                assumption: Assumption {
                    id: a_null,
                    statement: "rests on nothing".into(),
                    rests_on: EntityId::NULL,
                    criticality: AssumptionCriticality::Critical,
                    status: AssumptionStatus::Open,
                    discharge: None,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (3) Dangling rests_on — resolves to no committed entity (P3).
        let err = core
            .invoke(CapabilityRequest::RaiseAssumption {
                assumption: Assumption {
                    id: a_phantom,
                    statement: "rests on a phantom".into(),
                    rests_on: EntityId(0xDEAD_BEEF),
                    criticality: AssumptionCriticality::Critical,
                    status: AssumptionStatus::Open,
                    discharge: None,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (4) Born non-Open — a raised assumption must be Open (never born discharged).
        let err = core
            .invoke(CapabilityRequest::RaiseAssumption {
                assumption: Assumption {
                    id: a_born_discharged,
                    statement: "already discharged at birth".into(),
                    rests_on: rid,
                    criticality: AssumptionCriticality::Critical,
                    status: AssumptionStatus::Discharged,
                    discharge: Some(Discharge {
                        resolution: DischargeResolution::GroundedFact,
                        target: rid,
                        decided_by: "test".into(),
                    }),
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Nothing entered the log: every malformed/dangling proposal was rejected pre-commit.
        assert!(core.state.assumptions.is_empty());
    }

    #[test]
    fn discharge_assumption_seam_flips_status_records_resolution_and_replays() {
        let mut core = new_core();
        let rid = seed_requirement(&mut core);

        let aid = core.fresh_id();
        core.invoke(CapabilityRequest::RaiseAssumption {
            assumption: Assumption {
                id: aid,
                statement: "the ambient stays below 60 C".into(),
                rests_on: rid,
                criticality: AssumptionCriticality::Critical,
                status: AssumptionStatus::Open,
                discharge: None,
            },
            links: vec![],
        })
        .unwrap();
        assert_eq!(core.state.undischarged_critical_assumptions().len(), 1);

        // Discharge it against the committed requirement (a resolvable target).
        core.invoke(CapabilityRequest::DischargeAssumption {
            assumption: aid,
            discharge: Discharge {
                resolution: DischargeResolution::EnforcedConstraint,
                target: rid,
                decided_by: "engineer".into(),
            },
        })
        .expect("discharging an open assumption against a committed entity is accepted");

        // The fold flipped status -> Discharged and recorded the resolution.
        let a = core
            .state
            .assumption(aid)
            .expect("still present after discharge");
        assert_eq!(a.status, AssumptionStatus::Discharged);
        assert!(!a.is_open());
        assert!(!a.is_blocking());
        let d = a
            .discharge
            .as_ref()
            .expect("discharge recorded on the assumption");
        assert_eq!(d.resolution, DischargeResolution::EnforcedConstraint);
        assert_eq!(d.target, rid);
        // No critical assumption is undischarged any more.
        assert!(core.state.undischarged_critical_assumptions().is_empty());

        // Byte-identical replay across raise + discharge (P4).
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    #[test]
    fn discharge_assumption_seam_rejects_unknown_target_non_open_and_dangling_discharge() {
        let mut core = new_core();
        let rid = seed_requirement(&mut core);

        // (1) Target assumption does not exist.
        let err = core
            .invoke(CapabilityRequest::DischargeAssumption {
                assumption: EntityId(0xDEAD_BEEF),
                discharge: Discharge {
                    resolution: DischargeResolution::GroundedFact,
                    target: rid,
                    decided_by: "engineer".into(),
                },
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Raise a real critical assumption to exercise the remaining rejections.
        let aid = core.fresh_id();
        core.invoke(CapabilityRequest::RaiseAssumption {
            assumption: Assumption {
                id: aid,
                statement: "the connector is genuine USB-C".into(),
                rests_on: rid,
                criticality: AssumptionCriticality::Critical,
                status: AssumptionStatus::Open,
                discharge: None,
            },
            links: vec![],
        })
        .unwrap();

        // (2) Discharge target is dangling (resolves to no committed entity).
        let err = core
            .invoke(CapabilityRequest::DischargeAssumption {
                assumption: aid,
                discharge: Discharge {
                    resolution: DischargeResolution::GroundedFact,
                    target: EntityId(0xFEED_FACE),
                    decided_by: "engineer".into(),
                },
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));
        // Still open — the rejected discharge committed nothing.
        assert!(core.state.assumption(aid).unwrap().is_open());

        // A legitimate discharge to move it out of Open.
        core.invoke(CapabilityRequest::DischargeAssumption {
            assumption: aid,
            discharge: Discharge {
                resolution: DischargeResolution::GroundedFact,
                target: rid,
                decided_by: "engineer".into(),
            },
        })
        .unwrap();

        // (3) Re-discharging an already-discharged (non-Open) assumption is rejected.
        let err = core
            .invoke(CapabilityRequest::DischargeAssumption {
                assumption: aid,
                discharge: Discharge {
                    resolution: DischargeResolution::AcceptedRisk,
                    target: rid,
                    decided_by: "engineer".into(),
                },
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Exactly one assumption, discharged once — the rejected calls changed nothing.
        assert_eq!(core.state.assumptions.len(), 1);
        assert_eq!(
            core.state.assumption(aid).unwrap().status,
            AssumptionStatus::Discharged
        );
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    // ===================== Band A (increment 2): ModelFidelity =====================
    //
    // TDD (written before the fold arm / accessor exist): a `ModelFidelity` is a trust-tag
    // on a derived fact (Map 6), committed as ADVISORY metadata via the audit `emit` seam —
    // NOT a `CapabilityRequest` — exactly like `Event::ViolationExplained` (E6 C1). Folding
    // `Event::FidelityTagged` pushes into the SEPARATE `fidelity_tags` store keyed by target
    // and NEVER mutates the tagged entity, so the tag can never usurp an object's authority.
    // The accessor `fidelity_for(target)` reads it back and the whole log replays
    // byte-identically (P4). Delivers exit criterion 2.

    #[test]
    fn fidelity_tag_emitted_folds_into_its_own_store_and_replays_byte_identically() {
        let mut core = new_core();
        // A committed requirement is the derived fact we tag (any committed target works —
        // the tag references a target, it does not own one).
        let target = seed_requirement(&mut core);
        let requirements_before = core.state.requirements.clone();

        // Committed as advisory metadata through the audit seam, exactly like ViolationExplained.
        core.emit(vec![Event::FidelityTagged {
            target,
            fidelity: ModelFidelity {
                concern: "worst-case rail droop".into(),
                method: FidelityMethod::FirstOrderFloor,
                confidence: 0.6,
                scope: "the 3.3 V rail".into(),
            },
            reasoning_call_seq: None,
        }])
        .expect("advisory fidelity tag commits through the audit seam");

        // The fold landed in the SEPARATE fidelity store, keyed by target.
        assert_eq!(core.state.fidelity_tags.len(), 1);
        let tags = core.state.fidelity_for(target);
        assert_eq!(tags.len(), 1, "fidelity_for reads the tag back by target");
        assert_eq!(tags[0].method, FidelityMethod::FirstOrderFloor);
        assert_eq!(tags[0].confidence, 0.6);
        assert_eq!(tags[0].concern, "worst-case rail droop");

        // ADVISORY-ONLY (structural, P3): tagging the requirement did NOT mutate it.
        assert_eq!(
            core.state.requirements, requirements_before,
            "a fidelity tag never mutates the entity it describes"
        );

        // Byte-identical replay with the fidelity tag in the log (P4).
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    #[test]
    fn fidelity_for_returns_all_tags_for_a_target_in_insertion_order() {
        // Several concerns may tag the same derived fact; fidelity_for returns all of them,
        // and only them (a tag on a different target is not returned), in insertion order.
        let mut core = new_core();
        let target = seed_requirement(&mut core);
        let other = core.fresh_id();

        core.emit(vec![
            Event::FidelityTagged {
                target,
                fidelity: ModelFidelity {
                    concern: "thermal".into(),
                    method: FidelityMethod::Assumed,
                    confidence: 0.2,
                    scope: "junction".into(),
                },
                reasoning_call_seq: None,
            },
            Event::FidelityTagged {
                target: other,
                fidelity: ModelFidelity {
                    concern: "unrelated".into(),
                    method: FidelityMethod::Measured,
                    confidence: 1.0,
                    scope: "elsewhere".into(),
                },
                reasoning_call_seq: None,
            },
            Event::FidelityTagged {
                target,
                fidelity: ModelFidelity {
                    concern: "electrical".into(),
                    method: FidelityMethod::Calculated,
                    confidence: 0.9,
                    scope: "rail".into(),
                },
                reasoning_call_seq: None,
            },
        ])
        .expect("advisory fidelity tags commit through the audit seam");

        let tags = core.state.fidelity_for(target);
        assert_eq!(tags.len(), 2, "only this target's tags, not the other's");
        // Insertion order preserved: `thermal` before `electrical`.
        assert_eq!(tags[0].concern, "thermal");
        assert_eq!(tags[1].concern, "electrical");

        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    // ===================== Band A (increment 3): Risk seam =====================
    //
    // TDD (written before the seam handlers / fold arms exist): the auditable risk-posture
    // object (Map 46) flows through the same seam as every other object. `RaiseRisk`
    // re-validates (non-empty statement + owner) and links its provenance; `AcceptRisk`
    // targets an existing risk by id and folds its `status -> Accepted` — the HUMAN owns
    // acceptance of residual risk (`00` Principle 11). Risk is TRACKED TRUTH: it does NOT
    // block release in v0; the read query `unaccepted_critical_risks()` surfaces the posture.
    // The whole log replays byte-identically (P4). No new `DomainError` variant.

    /// A helper building an Open risk with a resolvable-by-caller residual severity.
    fn open_risk(id: EntityId, statement: &str, owner: &str, residual: RiskSeverity) -> Risk {
        Risk {
            id,
            statement: statement.into(),
            likelihood: RiskLikelihood::Medium,
            severity: RiskSeverity::High,
            mitigation: "add a TVS diode on the USB-C VBUS".into(),
            residual,
            owner: owner.into(),
            status: RiskStatus::Open,
        }
    }

    #[test]
    fn raise_risk_seam_accepts_valid_then_folds_and_replays_byte_identically() {
        let mut core = new_core();
        // A committed requirement gives the risk's provenance link a real anchor (mirrors how
        // an assumption rests on a committed entity); the seam links but does not gate on it.
        let rid = seed_requirement(&mut core);
        let risk_id = core.fresh_id();
        let link_id = core.fresh_id();

        core.invoke(CapabilityRequest::RaiseRisk {
            risk: open_risk(
                risk_id,
                "an ESD strike on the USB-C connector could latch up the MCU",
                "hardware lead",
                RiskSeverity::Low,
            ),
            links: vec![eak_domain::ProvenanceLink {
                id: link_id,
                from: risk_id,
                to: rid,
                relation: eak_domain::RelationType::TracesTo,
            }],
        })
        .expect("a well-formed risk with a valid owner is accepted at the seam");

        // The fold pushed exactly one Open risk and the accessor reads it back by id.
        assert_eq!(core.state.risks.len(), 1);
        let r = core.state.risk(risk_id).expect("risk folded by id");
        assert_eq!(r.status, RiskStatus::Open);
        assert_eq!(r.owner, "hardware lead");

        // Byte-identical replay with the risk (and its link) in the log (P4).
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    #[test]
    fn raise_risk_seam_rejects_malformed_proposals_and_commits_nothing() {
        let mut core = new_core();
        seed_requirement(&mut core);
        let (r_empty_stmt, r_empty_owner) = (core.fresh_id(), core.fresh_id());

        // (1) Empty statement — domain validate() rejects; nothing enters the log.
        let err = core
            .invoke(CapabilityRequest::RaiseRisk {
                risk: open_risk(r_empty_stmt, "   ", "hardware lead", RiskSeverity::Low),
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (2) Empty owner — a risk with no accountable owner cannot be owned/accepted
        // (Principle 11) and is rejected at the seam.
        let err = core
            .invoke(CapabilityRequest::RaiseRisk {
                risk: open_risk(
                    r_empty_owner,
                    "ESD on the connector",
                    "  ",
                    RiskSeverity::High,
                ),
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Nothing entered the log: every malformed proposal was rejected pre-commit (P3).
        assert!(core.state.risks.is_empty());
    }

    #[test]
    fn accept_risk_seam_flips_status_to_accepted_and_replays() {
        let mut core = new_core();
        seed_requirement(&mut core);
        let risk_id = core.fresh_id();

        core.invoke(CapabilityRequest::RaiseRisk {
            risk: open_risk(
                risk_id,
                "thermal runaway under sustained full load",
                "thermal owner",
                RiskSeverity::Critical,
            ),
            links: vec![],
        })
        .unwrap();
        // Before acceptance, a Critical-residual, non-Accepted risk is surfaced by the read query.
        assert_eq!(core.state.unaccepted_critical_risks().len(), 1);

        // The HUMAN accepts the residual risk (Principle 11) — the fold flips status -> Accepted.
        core.invoke(CapabilityRequest::AcceptRisk {
            risk: risk_id,
            accepted_by: "founder".into(),
        })
        .expect("accepting an existing risk is accepted at the seam");

        let r = core
            .state
            .risk(risk_id)
            .expect("still present after accept");
        assert_eq!(r.status, RiskStatus::Accepted);
        // Once accepted, it no longer counts as unaccepted-critical (human owns the residual).
        assert!(core.state.unaccepted_critical_risks().is_empty());

        // Byte-identical replay across raise + accept (P4).
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    #[test]
    fn accept_risk_seam_rejects_unknown_target_and_commits_nothing() {
        let mut core = new_core();
        seed_requirement(&mut core);

        // Accepting a risk that was never raised is meaningless — reject it (P3).
        let err = core
            .invoke(CapabilityRequest::AcceptRisk {
                risk: EntityId(0xDEAD_BEEF),
                accepted_by: "founder".into(),
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));
        assert!(core.state.risks.is_empty());
    }

    #[test]
    fn risk_does_not_block_release_but_the_read_query_surfaces_the_posture() {
        // Risk is TRACKED TRUTH in v0: an open Critical-residual risk is NOT a release blocker
        // (unlike an undischarged critical Assumption). The read query merely surfaces the posture
        // for the human, who owns acceptance (Principle 11). This test pins that boundary:
        // `unaccepted_critical_risks` filters on residual == Critical AND status != Accepted, and
        // it is a READ, not a gate.
        let mut core = new_core();
        seed_requirement(&mut core);
        let (crit_id, low_id) = (core.fresh_id(), core.fresh_id());

        // A Critical-residual open risk: surfaced.
        core.invoke(CapabilityRequest::RaiseRisk {
            risk: open_risk(
                crit_id,
                "unmitigated brownout",
                "power owner",
                RiskSeverity::Critical,
            ),
            links: vec![],
        })
        .unwrap();
        // A Low-residual open risk: NOT surfaced by the critical read query.
        core.invoke(CapabilityRequest::RaiseRisk {
            risk: open_risk(
                low_id,
                "minor cosmetic silkscreen error",
                "layout owner",
                RiskSeverity::Low,
            ),
            links: vec![],
        })
        .unwrap();

        assert_eq!(core.state.risks.len(), 2);
        let surfaced = core.state.unaccepted_critical_risks();
        assert_eq!(
            surfaced.len(),
            1,
            "only the Critical-residual, non-Accepted risk is surfaced"
        );
        assert_eq!(surfaced[0].id, crit_id);

        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    // ===================== Band A (increment 4): Objective / Tradeoff seam =====================
    //
    // TDD (written before the seam handlers / fold arms / readers exist): the weighed-and-
    // rejected space, preserved (Map 11, exit criterion 3). `RecordObjective` re-validates a
    // weighted goal and links its provenance to a committed source; `RecordTradeoff` re-validates
    // the tradeoff (>=2 alternatives, `chosen` in range and NOT rejected, >=1 rejected preserved)
    // and links its provenance. Both fold into their own stores, replay byte-identically (P4), and
    // a `Decision` may cite the `Tradeoff` it resolved via a ProvenanceLink (ties the choice to its
    // rejected space, `02` Map 11). No new `DomainError` variant.

    /// A well-formed 3-alternative tradeoff, mirroring the domain helper — `chosen == 0` is not
    /// rejected and two rejected alternatives are preserved. `id` is caller-supplied so the seam
    /// tests can mint a fresh id.
    fn well_formed_tradeoff(id: EntityId) -> Tradeoff {
        Tradeoff {
            id,
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
                Alternative {
                    label: "charge-pump".into(),
                    description: "switched-capacitor charge pump".into(),
                    scores: vec![0.3, 0.5],
                    rejected: true,
                },
            ],
            criteria: vec!["efficiency".into(), "simplicity".into()],
            chosen: 0,
            rationale: "efficiency dominates at this load; the LDO's droop is unacceptable".into(),
            decided_by: "hardware lead".into(),
        }
    }

    #[test]
    fn record_objective_seam_accepts_valid_then_folds_and_replays_byte_identically() {
        let mut core = new_core();
        // A committed requirement is a real `source` anchor for the objective's provenance.
        let rid = seed_requirement(&mut core);
        let obj_id = core.fresh_id();
        let link_id = core.fresh_id();

        core.invoke(CapabilityRequest::RecordObjective {
            objective: Objective {
                id: obj_id,
                statement: "minimize board area".into(),
                weight: 0.7,
                source: rid,
            },
            links: vec![eak_domain::ProvenanceLink {
                id: link_id,
                from: obj_id,
                to: rid,
                relation: eak_domain::RelationType::TracesTo,
            }],
        })
        .expect("a well-formed objective rooted in a committed source is accepted at the seam");

        // The fold pushed exactly one objective and the reader/accessor reads it back by id.
        assert_eq!(core.state.objectives.len(), 1);
        let o = core
            .state
            .objective(obj_id)
            .expect("objective folded by id");
        assert_eq!(o.statement, "minimize board area");
        assert_eq!(o.weight, 0.7);

        // Byte-identical replay with the objective (and its link) in the log (P4).
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    #[test]
    fn record_objective_seam_rejects_empty_statement_and_commits_nothing() {
        let mut core = new_core();
        let rid = seed_requirement(&mut core);
        let obj_id = core.fresh_id();

        // Empty statement — domain validate() rejects; nothing enters the log (P3).
        let err = core
            .invoke(CapabilityRequest::RecordObjective {
                objective: Objective {
                    id: obj_id,
                    statement: "   ".into(),
                    weight: 0.7,
                    source: rid,
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));
        assert!(core.state.objectives.is_empty());
    }

    #[test]
    fn record_tradeoff_seam_accepts_valid_then_folds_and_replays_byte_identically() {
        let mut core = new_core();
        let _rid = seed_requirement(&mut core);
        let td_id = core.fresh_id();

        core.invoke(CapabilityRequest::RecordTradeoff {
            tradeoff: well_formed_tradeoff(td_id),
            links: vec![],
        })
        .expect("a well-formed tradeoff preserving the rejected space is accepted at the seam");

        // The fold pushed exactly one tradeoff and the accessor reads it back by id, WITH its
        // rejected space intact (exit criterion 3: the design remembers what it did not choose).
        assert_eq!(core.state.tradeoffs.len(), 1);
        let t = core.state.tradeoff(td_id).expect("tradeoff folded by id");
        assert_eq!(t.chosen, 0);
        assert!(!t.alternatives[t.chosen].rejected, "chosen is not rejected");
        let rejected_preserved = t.alternatives.iter().filter(|a| a.rejected).count();
        assert_eq!(rejected_preserved, 2, "the rejected space is preserved");

        // Byte-identical replay with the tradeoff in the log (P4).
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    #[test]
    fn record_tradeoff_seam_rejects_malformed_proposals_and_commits_nothing() {
        let mut core = new_core();
        seed_requirement(&mut core);
        let (t_one_alt, t_chosen_oob, t_chosen_rejected, t_no_rejected) = (
            core.fresh_id(),
            core.fresh_id(),
            core.fresh_id(),
            core.fresh_id(),
        );

        // (1) Fewer than two alternatives — nothing was weighed against anything.
        let mut single = well_formed_tradeoff(t_one_alt);
        single.alternatives = vec![Alternative {
            label: "buck".into(),
            description: "the only option".into(),
            scores: vec![0.9],
            rejected: false,
        }];
        single.chosen = 0;
        let err = core
            .invoke(CapabilityRequest::RecordTradeoff {
                tradeoff: single,
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (2) `chosen` out of range.
        let mut oob = well_formed_tradeoff(t_chosen_oob);
        oob.chosen = 3; // len is 3.
        let err = core
            .invoke(CapabilityRequest::RecordTradeoff {
                tradeoff: oob,
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (3) `chosen` points at a REJECTED alternative (self-contradiction).
        let mut chosen_rejected = well_formed_tradeoff(t_chosen_rejected);
        chosen_rejected.chosen = 1; // index 1 is rejected == true.
        let err = core
            .invoke(CapabilityRequest::RecordTradeoff {
                tradeoff: chosen_rejected,
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (4) No rejected alternative preserved — the rejected space was thrown away (exit
        // criterion 3 is violated), so the seam rejects it.
        let mut no_rejected = well_formed_tradeoff(t_no_rejected);
        for alt in &mut no_rejected.alternatives {
            alt.rejected = false;
        }
        no_rejected.chosen = 0;
        let err = core
            .invoke(CapabilityRequest::RecordTradeoff {
                tradeoff: no_rejected,
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Nothing entered the log: every malformed proposal was rejected pre-commit (P3).
        assert!(core.state.tradeoffs.is_empty());
    }

    #[test]
    fn a_decision_may_cite_the_tradeoff_it_resolved_via_a_provenance_link() {
        // Map 11: the choice is tied to its rejected space. A recorded Tradeoff is cited by a
        // Decision through a ProvenanceLink, so the design can always trace WHY the chosen option
        // won over the preserved-and-rejected ones.
        let mut core = new_core();
        let rid = seed_requirement(&mut core);
        let td_id = core.fresh_id();
        core.invoke(CapabilityRequest::RecordTradeoff {
            tradeoff: well_formed_tradeoff(td_id),
            links: vec![],
        })
        .unwrap();

        // A Decision (justifying the chosen alternative) cites the Tradeoff it resolved.
        let req_id = core.fresh_id();
        let did = core.fresh_id();
        let link_id = core.fresh_id();
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: Requirement {
                id: req_id,
                statement: "The 3.3 V rail shall use a buck converter".into(),
                category: RequirementCategory::Electrical,
                priority: Priority::High,
                acceptance_criterion: "topology is a buck".into(),
                status: RequirementStatus::Accepted,
                source: rid,
                targets: vec![],
            },
            decision: Decision {
                id: did,
                subject: rid,
                rationale: "buck chosen per the topology tradeoff".into(),
                decider: "hardware lead".into(),
                reasoning_call_seq: None,
                evidence: vec![],
                confidence: 1.0,
            },
            evidence: vec![],
            links: vec![eak_domain::ProvenanceLink {
                id: link_id,
                from: did,
                to: td_id,
                relation: eak_domain::RelationType::JustifiedBy,
            }],
        })
        .unwrap();

        // The link resolves: it connects the committed Decision to the committed Tradeoff.
        let link = core
            .state
            .links
            .iter()
            .find(|l| l.id == link_id)
            .expect("the decision->tradeoff link was committed");
        assert_eq!(link.from, did);
        assert_eq!(link.to, td_id);
        assert!(
            core.state.decision(did).is_some(),
            "the citing decision is committed"
        );
        assert!(
            core.state.tradeoff(td_id).is_some(),
            "the cited tradeoff is committed"
        );

        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    // ===================== Band B (increment 1): PowerDomain seam =====================
    //
    // TDD (written before the seam handler exists): the power rail object flows through the same
    // seam as every Phase-2/3/Band-A object. CreatePowerDomain re-validates (non-empty name,
    // positive voltage, positive max current, >=1 net), rejects a null/unknown source component,
    // and rejects any net that was never committed (P3 referential integrity — a domain can never
    // reference a phantom rail). The fold pushes the committed domain into state; the log replays
    // byte-identically (P4). This is the entry point for Band B's power-balance ERC rule.

    /// Seed a real component (a regulator) and a committed power net, so a power domain has real
    /// referential anchors. Returns (component id, net id) — the resolvable seam targets.
    fn seed_component_and_net(core: &mut RuntimeCore) -> (EntityId, EntityId) {
        let rid = seed_requirement(core);
        let block_id = core.fresh_id();
        core.invoke(CapabilityRequest::CreateFunctionalBlock {
            block: FunctionalBlock {
                id: block_id,
                name: "3V3 regulation".into(),
                function: "step VBUS down to 3.3 V".into(),
                requirements: vec![rid],
            },
            links: vec![],
        })
        .unwrap();
        let comp_id = core.fresh_id();
        let pin_id = core.fresh_id();
        core.invoke(CapabilityRequest::RealizeComponent {
            component: Component {
                id: comp_id,
                refdes: "U1".into(),
                class: ComponentClass::Regulator,
                value: None,
                from_block: block_id,
                origin: ComponentOrigin::Synthesized,
            },
            pins: vec![Pin {
                id: pin_id,
                component: comp_id,
                designation: "VOUT".into(),
                electrical_type: PinElectricalType::PowerOut,
            }],
            links: vec![],
        })
        .unwrap();
        let net_id = core.fresh_id();
        core.invoke(CapabilityRequest::CreateNet {
            net: Net {
                id: net_id,
                name: "+3V3".into(),
                class: NetClass::Power,
                members: vec![pin_id],
                current: Some(PhysicalQuantity::new(0.5, Unit::Ampere)),
                impedance_target: None,
                origin: NetOrigin::Logical,
            },
            links: vec![],
        })
        .unwrap();
        (comp_id, net_id)
    }

    #[test]
    fn create_power_domain_seam_accepts_valid_then_folds_and_replays_byte_identically() {
        let mut core = new_core();
        let (source, net_id) = seed_component_and_net(&mut core);
        let dom_id = core.fresh_id();

        core.invoke(CapabilityRequest::CreatePowerDomain {
            domain: PowerDomain {
                id: dom_id,
                name: "VDD_CORE".into(),
                voltage: PhysicalQuantity::new(3.3, Unit::Volt),
                source_component: source,
                max_current: PhysicalQuantity::new(2.0, Unit::Ampere),
                nets: vec![net_id],
            },
            links: vec![],
        })
        .expect("a well-formed power domain on committed component + net is accepted at the seam");

        // The fold pushed exactly one domain and the accessor reads it back by id.
        assert_eq!(core.state.power_domains.len(), 1);
        let d = core
            .state
            .power_domain(dom_id)
            .expect("power domain folded by id");
        assert_eq!(d.name, "VDD_CORE");
        assert_eq!(d.source_component, source);

        // Byte-identical replay with the power domain in the log (P4).
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    #[test]
    fn create_power_domain_seam_rejects_malformed_and_dangling_proposals() {
        let mut core = new_core();
        let (source, net_id) = seed_component_and_net(&mut core);
        let (d_blank, d_null_src, d_phantom_src, d_phantom_net) = (
            core.fresh_id(),
            core.fresh_id(),
            core.fresh_id(),
            core.fresh_id(),
        );

        // (1) Blank rail name — domain validate() rejects.
        let err = core
            .invoke(CapabilityRequest::CreatePowerDomain {
                domain: PowerDomain {
                    id: d_blank,
                    name: "   ".into(),
                    voltage: PhysicalQuantity::new(3.3, Unit::Volt),
                    source_component: source,
                    max_current: PhysicalQuantity::new(2.0, Unit::Ampere),
                    nets: vec![net_id],
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (2) Null source — a rail with no supplying component is untraceable (P3).
        let err = core
            .invoke(CapabilityRequest::CreatePowerDomain {
                domain: PowerDomain {
                    id: d_null_src,
                    name: "VDD_GHOST".into(),
                    voltage: PhysicalQuantity::new(3.3, Unit::Volt),
                    source_component: EntityId::NULL,
                    max_current: PhysicalQuantity::new(2.0, Unit::Ampere),
                    nets: vec![net_id],
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (3) Dangling source — resolves to no committed component (P3).
        let err = core
            .invoke(CapabilityRequest::CreatePowerDomain {
                domain: PowerDomain {
                    id: d_phantom_src,
                    name: "VDD_PHANTOM".into(),
                    voltage: PhysicalQuantity::new(3.3, Unit::Volt),
                    source_component: EntityId(0xDEAD_BEEF),
                    max_current: PhysicalQuantity::new(2.0, Unit::Ampere),
                    nets: vec![net_id],
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // (4) Dangling rail net — references a net never committed (P3): no phantom rail.
        let err = core
            .invoke(CapabilityRequest::CreatePowerDomain {
                domain: PowerDomain {
                    id: d_phantom_net,
                    name: "VDD_OK".into(),
                    voltage: PhysicalQuantity::new(3.3, Unit::Volt),
                    source_component: source,
                    max_current: PhysicalQuantity::new(2.0, Unit::Ampere),
                    nets: vec![EntityId(0xDEAD_BEEF)],
                },
                links: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, CapabilityError::Rejected(_)));

        // Nothing entered the log: every malformed/dangling proposal was rejected pre-commit.
        assert!(core.state.power_domains.is_empty());
    }
}
