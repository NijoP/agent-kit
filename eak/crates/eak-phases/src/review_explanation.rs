//! Review Explanation machine (instance) — the AI-review explainer (E6, increment C1).
//!
//! Runs AFTER a verification phase (e.g. DRC) has committed its [`Violation`]s. For each raised
//! violation it asks the reasoning boundary (`violation_explanation_v1`) for a plain-English
//! account of what the violation means plus a suggested fix, then commits that judgement as an
//! ADVISORY [`Event::ViolationExplained`] linked to the violation by id and back to the exact
//! reasoning call by seq (provenance-by-construction).
//!
//! ADVISORY-ONLY INVARIANT (security-critical, P3). The model text produced here is stored as
//! metadata and NOTHING ELSE. It never changes a violation's `severity`, `status`, or validity; it
//! never gates this or any other phase (this machine always reports [`StepResult::Done`], even when
//! the reasoning engine is unavailable, so a reasoning outage can never block the pipeline); and it
//! never drives a [`CapabilityRequest`] or kernel mutation.
//!
//! The committed kernel violation *grounds* the request; the LLM only *describes* it. The event
//! folds into the SEPARATE `EngineeringState::violation_explanations` store, never into the
//! `Violation`, so the invariant is structural rather than a mere convention. The workflow gate
//! stays exactly what it was: open, blocking (error-severity) violations.

use eak_domain::Violation;
use eak_ports::{Event, ReasoningRequest};
use eak_runtime::{AgentContext, Machine, MachineError, StepResult};
use std::collections::HashSet;

/// Schema name of the advisory explainer request. Kept in lockstep with the fixture and live
/// (`AnthropicEngine`) adapters, which key/branch on this exact string.
pub const VIOLATION_EXPLANATION_SCHEMA: &str = "violation_explanation_v1";

pub struct ReviewExplanationMachine;

impl ReviewExplanationMachine {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ReviewExplanationMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the deterministic reasoning request that grounds ONE violation. Pure function of the
/// violation's kernel-owned facts (rule, severity, message, implicated subjects), so the fixture
/// key is reproducible offline and the same violation always keys to the same cassette entry (P4).
/// Traceability to the originating requirement is NOT re-derived here — it already flows through
/// the violation's own subject provenance links (`Violation -> subject -> … -> Requirement`) plus
/// the [`Event::ViolationExplained`] link this machine commits.
pub fn explanation_request(violation: &Violation) -> ReasoningRequest {
    let prompt = build_prompt(violation);
    let seed = stable_seed(&prompt);
    ReasoningRequest {
        model_id: String::new(), // filled in by the runtime with the real engine id
        system: "You are a senior hardware design reviewer. Explain each design-rule violation in \
                 plain English and suggest one concrete, safe fix. Your commentary is ADVISORY for \
                 a human reviewer only — you never modify the design."
            .into(),
        prompt,
        schema_name: VIOLATION_EXPLANATION_SCHEMA.into(),
        temperature: 0.0,
        seed,
    }
}

impl Machine for ReviewExplanationMachine {
    fn name(&self) -> &str {
        "ReviewExplanation"
    }

    fn initial(&self) -> String {
        "Idle".into()
    }

    fn step(
        &mut self,
        state: &str,
        ctx: &mut dyn AgentContext,
    ) -> Result<StepResult, MachineError> {
        match state {
            "Idle" => Ok(StepResult::Continue("Explaining".into())),

            "Explaining" => {
                let violations = ctx.violations();
                // Idempotent re-entry (e.g. a DRC loop-back re-runs the review): never re-explain a
                // violation already explained — so re-running is deterministic (one explanation each).
                let already: HashSet<_> = ctx.explained_violations().into_iter().collect();

                for v in &violations {
                    if already.contains(&v.id) {
                        continue;
                    }

                    // ---- reasoning half: ask for the advisory explanation (judgement only) ----
                    // Best-effort: a reasoning error is NOT a phase failure — advisory commentary
                    // must never gate the pipeline, so we simply skip this violation and continue.
                    let request = explanation_request(v);
                    let (call_seq, response) = match ctx.reason(request) {
                        Ok(pair) => pair,
                        Err(_) => continue,
                    };
                    // The request grounds exactly ONE violation, so the first advisory explanation
                    // is the one for it. An empty/degenerate answer is dropped (nothing to record).
                    let Some(exp) = response.explanations.into_iter().next() else {
                        continue;
                    };
                    if exp.explanation.trim().is_empty() && exp.suggested_fix.trim().is_empty() {
                        continue;
                    }

                    // ---- advisory commit via the audit seam ----
                    // `emit` records the fact through the single commit path (stamp -> append ->
                    // fold). It carries NO CapabilityRequest and folds into the separate
                    // `violation_explanations` store, so it can never mutate `v` or gate a phase.
                    ctx.emit(vec![Event::ViolationExplained {
                        violation: v.id,
                        explanation: exp.explanation,
                        suggested_fix: exp.suggested_fix,
                        reasoning_call_seq: call_seq,
                    }])
                    .map_err(|e| MachineError::Internal(e.to_string()))?;
                }

                // Always Done: the explainer is advisory, so it succeeds whether it explained every
                // violation, some, or none. It NEVER reports Failed on the basis of model output.
                Ok(StepResult::Done)
            }

            other => Err(MachineError::Internal(format!("unknown state {other}"))),
        }
    }
}

/// Grounding prompt: the violation's kernel-owned facts, nothing model-authored.
fn build_prompt(v: &Violation) -> String {
    let subjects = v
        .subjects
        .iter()
        .map(|s| s.short())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "A design-rule check RAISED the following violation. Explain, in plain English, what it \
         means and why it was flagged, then suggest one concrete fix. This is advisory commentary \
         for a human reviewer — it does not and must not change the design.\n\n\
         RULE: {rule}\n\
         SEVERITY: {severity:?}\n\
         MESSAGE: {message}\n\
         IMPLICATED ENTITIES: {subjects}\n",
        rule = v.rule,
        severity = v.severity,
        message = v.message,
    )
}

/// Deterministic seed from the grounding text (FNV-1a 64) so a run is reproducible (P4).
fn stable_seed(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use eak_domain::{
        Decision, EntityId, Priority, ProvenanceLink, RelationType, Requirement,
        RequirementCategory, RequirementStatus, ViolationSeverity, ViolationStatus,
    };
    use eak_ports::{
        CandidateExplanation, Event, EventLog, EventRecord, ReasoningResponse, Seq, StoreError,
        Timestamp,
    };
    use eak_reasoning::{Cassette, CassetteEntry, FixtureEngine};
    use eak_runtime::{
        replay, AgentContext, Autonomy, CapabilityRequest, ExecutionEngine, LogicalClock,
        PhaseOutcome, RuntimeCore, SeededIdSource,
    };

    // A fixed rule/subject so the committed violation — and therefore the fixture key — is known
    // before the runtime is built (the FixtureEngine is moved in at construction).
    const RULE: &str = "drc-copper-clearance";
    const MESSAGE: &str = "copper clearance 0.10 mm is below the 0.20 mm process minimum";
    const RID: EntityId = EntityId(0x00A1);
    const VID: EntityId = EntityId(0x00B2);
    const EXPLANATION: &str =
        "Two copper features sit closer than the fab's minimum spacing, so the board may short or \
         fail electrical test after etching.";
    const FIX: &str =
        "Increase the trace-to-trace clearance to at least 0.20 mm, or move one of the nets to a \
         different layer.";

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

    fn known_violation() -> Violation {
        Violation {
            id: VID,
            rule: RULE.into(),
            severity: ViolationSeverity::Error,
            subjects: vec![RID],
            message: MESSAGE.into(),
            status: ViolationStatus::Open,
        }
    }

    /// A FixtureEngine whose cassette answers exactly the request the explainer will raise for the
    /// known violation — proving the offline, keyed cassette path (no live API call).
    fn seeded_fixture() -> FixtureEngine {
        let req = explanation_request(&known_violation());
        let key = FixtureEngine::key(&req);
        FixtureEngine::from_cassette(Cassette {
            entries: vec![CassetteEntry {
                key,
                response: ReasoningResponse {
                    candidates: vec![],
                    explanations: vec![CandidateExplanation {
                        explanation: EXPLANATION.into(),
                        suggested_fix: FIX.into(),
                    }],
                    clarifying_questions: vec![],
                    raw: String::new(),
                },
            }],
            default: None,
        })
    }

    /// Build a runtime with the seeded fixture and commit one requirement + one violation (whose
    /// subject IS that requirement, so it is directly traceable to its originating requirement).
    fn core_with_committed_violation() -> RuntimeCore {
        let mut core = RuntimeCore::new(
            Box::new(MemLog { records: vec![] }),
            Box::new(seeded_fixture()),
            Box::new(SeededIdSource::new(7)),
            Box::new(LogicalClock::new()),
            Autonomy::Autonomous,
        );
        let intent_id = core
            .capture_intent("USB-C IoT sensor node, 2-layer PCB", "engineer")
            .unwrap();
        let did = core.fresh_id();
        core.invoke(CapabilityRequest::CreateRequirement {
            requirement: Requirement {
                id: RID,
                statement: "Copper clearance shall meet the process minimum".into(),
                category: RequirementCategory::Fabrication,
                priority: Priority::High,
                acceptance_criterion: "all clearances >= 0.20 mm".into(),
                status: RequirementStatus::Accepted,
                source: intent_id,
                targets: vec![],
            },
            decision: Decision {
                id: did,
                subject: RID,
                rationale: "fab constraint".into(),
                decider: "test".into(),
                reasoning_call_seq: None,
                evidence: vec![],
                confidence: 1.0,
            },
            evidence: vec![],
            links: vec![],
        })
        .unwrap();

        // Raise the violation, linked to the requirement it implicates (traceability anchor).
        let link_id = core.fresh_id();
        core.invoke(CapabilityRequest::RaiseViolation {
            violation: known_violation(),
            links: vec![ProvenanceLink {
                id: link_id,
                from: VID,
                to: RID,
                relation: RelationType::TracesTo,
            }],
        })
        .unwrap();
        assert_eq!(core.state.violations.len(), 1);
        core
    }

    fn count_explained(log: &dyn EventLog) -> usize {
        log.read_all()
            .unwrap()
            .iter()
            .filter(|r| matches!(r.event, Event::ViolationExplained { .. }))
            .count()
    }

    #[test]
    fn explainer_commits_linked_advisory_explanation_and_replays_deterministically() {
        let mut core = core_with_committed_violation();

        let outcome = ExecutionEngine::new().run(&mut ReviewExplanationMachine::new(), &mut core);
        assert_eq!(outcome, PhaseOutcome::Success);

        // The advisory explanation was committed, carrying the fixture's exact text.
        assert_eq!(core.state.violation_explanations.len(), 1);
        let exp = &core.state.violation_explanations[0];
        assert_eq!(exp.violation, VID);
        assert_eq!(exp.explanation, EXPLANATION);
        assert_eq!(exp.suggested_fix, FIX);
        // …and it is queryable by the violation it explains (the traceability link, by id).
        assert_eq!(core.state.explanations_for(VID).len(), 1);

        // Provenance-by-construction: reasoning_call_seq points at the exact ReasoningCall event.
        let call_seq = exp.reasoning_call_seq;
        let records = core.log().read_all().unwrap();
        assert!(matches!(
            records[call_seq as usize].event,
            Event::ReasoningCall { .. }
        ));

        // Determinism: re-folding the log reconstructs byte-identical state (P4).
        let replayed = replay(core.log()).unwrap();
        assert_eq!(core.state, replayed);
        assert_eq!(core.state.canonical_json(), replayed.canonical_json());
    }

    #[test]
    fn explanation_is_advisory_only_and_idempotent() {
        let mut core = core_with_committed_violation();

        // The violation is blocking BEFORE any explanation exists.
        let before = core.state.violation(VID).unwrap().clone();
        assert!(before.is_blocking());
        assert_eq!(core.state.open_blocking_violations().len(), 1);

        ExecutionEngine::new().run(&mut ReviewExplanationMachine::new(), &mut core);

        // ADVISORY-ONLY: the explanation did NOT touch the violation's severity/status/validity, so
        // the workflow gate is exactly as before — the LLM described, it did not decide.
        let after = core.state.violation(VID).unwrap();
        assert_eq!(before, *after);
        assert!(after.is_blocking());
        assert_eq!(after.severity, ViolationSeverity::Error);
        assert_eq!(after.status, ViolationStatus::Open);
        assert_eq!(core.state.open_blocking_violations().len(), 1);

        // IDEMPOTENT / deterministic re-run: explaining again neither duplicates the record nor
        // re-calls the model for an already-explained violation.
        assert_eq!(core.state.violation_explanations.len(), 1);
        assert_eq!(count_explained(core.log()), 1);
        ExecutionEngine::new().run(&mut ReviewExplanationMachine::new(), &mut core);
        assert_eq!(core.state.violation_explanations.len(), 1);
        assert_eq!(count_explained(core.log()), 1);
        assert_eq!(
            core.state.violation_explanations[0].explanation,
            EXPLANATION
        );
    }
}
