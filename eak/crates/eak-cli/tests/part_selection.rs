//! Increment C2 (epic E6) — the MOAT PROOF for reasoning-driven part selection.
//!
//! BOM Planning is the SECOND phase to reason (after Requirement Planning): the model PROPOSES a
//! manufacturer part number per component class (`part_candidates_v1`) and the kernel RE-VALIDATES
//! every proposal against the authoritative `PartCatalog` before anything is committed. These tests
//! pin the three properties that make that a moat, not a rubber stamp:
//!
//!   1. PROPOSE -> VALIDATE -> ACCEPT: a valid proposal (a catalogued MPN) is accepted and lands in
//!      state through the real capability seam (`Event::PartCommitted`), and the run still releases.
//!   2. PROPOSE -> VALIDATE -> REJECT: a hallucinated MPN the catalog does not carry is REJECTED —
//!      it never enters state; BOM Planning refuses to source the class from a non-catalog part and
//!      surfaces it as a diagnostic phase FAILURE, so the design never releases. "The LLM proposes;
//!      the kernel validates" — a bad proposal is provably refused, not silently corrected.
//!   3. DETERMINISM: the reasoning-driven run replays byte-identically from its event log (P4).

use eak_cli::{replay_cmd, run, run_with, PhaseOutcome, ReasoningChoice, RunConfig};
use eak_domain::{Priority, RequirementCategory};
use eak_ports::{
    CandidatePart, CandidateRequirement, Event, EventLog, ReasoningEngine, ReasoningResponse,
};
use eak_reasoning::FixtureEngine;
use eak_store::FileEventLog;
use std::path::PathBuf;

const HALLUCINATED_MPN: &str = "HALLUCINATED-IC-9000";

/// A NON-default but in-catalog `Ic` part (C2.1): the real I²C temperature sensor. The catalog's
/// first-choice / default `Ic` part is the MCU `STM32L010F4P6`; this MPN is a *different* member of
/// the same class set, so accepting it proves SET-INCLUSION, not equality-to-the-canonical-part.
const TEMP_SENSOR_MPN: &str = "TMP102AIDRLR";

fn temp_log(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("eak-cli-c2-{}-{}.jsonl", tag, std::process::id()));
    p
}

fn cfg(tag: &str, seed: u64) -> (RunConfig, PathBuf) {
    let log = temp_log(tag);
    let _ = std::fs::remove_file(&log);
    (
        RunConfig {
            intent: "USB-C powered IoT sensor node, < 5 W, < 50x50 mm".into(),
            reasoning: ReasoningChoice::Fixture,
            cassette: None,
            log: log.clone(),
            model: "fixture".into(),
            seed,
            deterministic_clock: true,
        },
        log,
    )
}

fn req(statement: &str, category: RequirementCategory) -> CandidateRequirement {
    CandidateRequirement {
        statement: statement.into(),
        category,
        priority: Priority::High,
        acceptance_criterion: "acceptance".into(),
        source_hint: "intent".into(),
        confidence: 0.9,
        rationale: "rationale".into(),
        targets: vec![],
    }
}

/// A reasoning engine whose requirement candidates realize a USB-C power source (a `Connector`) and
/// a logic load (an `Ic`) — so the power rail is driven and ERC is clean and the workflow reaches
/// the BOM layer — and whose PART candidates propose a NON-CATALOG MPN for the `Ic` class. The
/// `Connector` is left unproposed, so it falls back to its catalogued part and is sourced normally;
/// only the `Ic` proposal is bogus, isolating the rejection to a single class.
fn hallucinated_ic_part_engine() -> Box<dyn ReasoningEngine> {
    Box::new(FixtureEngine::single(ReasoningResponse {
        candidates: vec![
            req(
                "Device shall be powered over USB-C",
                RequirementCategory::Functional,
            ),
            req(
                "Logic core shall operate at 3.3 logic level",
                RequirementCategory::Electrical,
            ),
        ],
        part_candidates: vec![CandidatePart {
            component_class: "Ic".into(),
            mpn: HALLUCINATED_MPN.into(),
            rationale: "the model invented a part that is not in the catalog".into(),
            confidence: 0.99,
        }],
        explanations: vec![],
        clarifying_questions: vec![],
        raw: "{}".into(),
    }))
}

/// A reasoning engine whose requirement candidates realize a USB-C power source (a `Connector`) and
/// a logic load (an `Ic`) — identical to [`hallucinated_ic_part_engine`] so ERC is clean and the
/// workflow reaches the BOM layer — but whose `Ic` PART proposal is the NON-DEFAULT, in-catalog
/// temperature sensor (`TMP102AIDRLR`), not the class's canonical/default MCU. The `Connector` is
/// left unproposed, so it falls back to its catalogued part. This isolates the set-inclusion accept
/// path: the model chose a member of the `Ic` set that is NOT the default, and it must be accepted.
fn temp_sensor_ic_part_engine() -> Box<dyn ReasoningEngine> {
    Box::new(FixtureEngine::single(ReasoningResponse {
        candidates: vec![
            req(
                "Device shall be powered over USB-C",
                RequirementCategory::Functional,
            ),
            req(
                "Logic core shall operate at 3.3 logic level",
                RequirementCategory::Electrical,
            ),
        ],
        part_candidates: vec![CandidatePart {
            component_class: "Ic".into(),
            mpn: TEMP_SENSOR_MPN.into(),
            rationale: "a NON-default but in-catalog Ic part — the I²C temperature sensor".into(),
            confidence: 0.95,
        }],
        explanations: vec![],
        clarifying_questions: vec![],
        raw: "{}".into(),
    }))
}

/// A `part_candidates_v1` reasoning call was recorded in the log — proof the BOM phase actually
/// reasoned (P4: the call is recorded so replay needs no model).
fn made_part_reasoning_call(records: &[eak_ports::EventRecord]) -> bool {
    records.iter().any(|r| {
        matches!(&r.event, Event::ReasoningCall { request, .. }
            if request.schema_name == "part_candidates_v1")
    })
}

#[test]
fn valid_part_proposal_is_accepted_and_lands_via_the_seam_and_releases() {
    // The default cassette proposes the correct catalogued MPN for every class, so BOM Planning
    // reasons, the kernel accepts each proposal, and the parts are committed through the seam.
    let (config, log) = cfg("accept", 1);
    let report = run(&config).expect("run succeeds");

    // The whole 15-phase pipeline still RELEASES: reasoning-driven part selection did not regress
    // the deterministic release path (the accepted proposals are exactly the catalogued parts).
    assert_eq!(report.outcomes.len(), 15);
    assert!(report
        .outcomes
        .iter()
        .all(|(_, o)| matches!(o, PhaseOutcome::Success)));
    assert!(
        report.state.open_blocking_violations().is_empty(),
        "an accepted, catalogued BOM releases cleanly"
    );

    // Parts + line items landed, and the BOM covers every component.
    assert!(!report.state.parts.is_empty(), "components are sourced");
    assert!(
        !report.state.bom_line_items.is_empty(),
        "the BOM binds parts"
    );

    let records = FileEventLog::open(&log)
        .expect("open log")
        .read_all()
        .expect("read log");

    // (a) the BOM phase REASONED: a part_candidates_v1 call is recorded.
    assert!(
        made_part_reasoning_call(&records),
        "BOM Planning made a part_candidates_v1 reasoning call"
    );
    // (b) every sourced part reached state through the REAL seam (a PartCommitted event exists),
    // never a side channel — the accepted proposal was committed via ctx.invoke -> commit.
    let committed_mpns: Vec<&str> = records
        .iter()
        .filter_map(|r| match &r.event {
            Event::PartCommitted { part } => Some(part.mpn.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        !committed_mpns.is_empty(),
        "parts were committed via the capability seam"
    );
    // The committed parts are catalogued MPNs the model proposed (e.g. the USB-C receptacle / MCU).
    for p in &report.state.parts {
        assert!(
            committed_mpns.contains(&p.mpn.as_str()),
            "part {} in state came from a PartCommitted seam event",
            p.mpn
        );
    }

    // Replay identity holds for the reasoning-driven accept run (P4).
    let replayed = replay_cmd(&log).expect("replay succeeds");
    assert_eq!(report.state, replayed);
    assert_eq!(report.state.canonical_json(), replayed.canonical_json());

    let _ = std::fs::remove_file(&log);
}

#[test]
fn non_default_but_in_catalog_part_is_accepted_via_set_inclusion() {
    // C2.1 SET-INCLUSION (positive): the model proposes a NON-default Ic part — the temperature
    // sensor — that is nonetheless a member of the Ic catalog set. The kernel must ACCEPT it and
    // commit that EXACT catalog entry (proving membership, not equality-to-the-canonical MCU).
    let (config, log) = cfg("setincl", 11);
    let report = run_with(temp_sensor_ic_part_engine(), &config).expect("run completes");

    let records = FileEventLog::open(&log)
        .expect("open log")
        .read_all()
        .expect("read log");

    // The BOM phase REASONED (a part_candidates_v1 call is recorded) and the non-default proposal
    // was ACCEPTED — BOM Planning succeeded rather than failing the way a rejection would.
    assert!(
        made_part_reasoning_call(&records),
        "BOM Planning made a part_candidates_v1 reasoning call"
    );
    let bom_outcome = report
        .outcomes
        .iter()
        .find(|(n, _)| n == "BomPlanning")
        .map(|(_, o)| o)
        .expect("BOM Planning ran");
    assert!(
        matches!(bom_outcome, PhaseOutcome::Success),
        "a NON-default but in-catalog part is accepted, so BOM Planning succeeds; got: {bom_outcome:?}"
    );

    // The EXACT proposed catalog part (the temperature sensor) reached state through the REAL seam
    // (a PartCommitted event carried it) — set-inclusion committed the matched member, not the
    // canonical default.
    assert!(
        records.iter().any(|r| matches!(&r.event,
            Event::PartCommitted { part } if part.mpn == TEMP_SENSOR_MPN)),
        "the non-default temp-sensor part was committed via the capability seam"
    );
    let sensor = report
        .state
        .parts
        .iter()
        .find(|p| p.mpn == TEMP_SENSOR_MPN)
        .expect("the temp sensor is in engineering state");
    // The committed part is the TRUSTED CatalogPart (manufacturer from the catalog record, NOT model
    // free text — the proposal carried no manufacturer at all). The moat holds on the accept path.
    assert_eq!(
        sensor.manufacturer, "Texas Instruments",
        "the committed part is built from the trusted catalog record, not the proposal text"
    );
    // Because the model chose the sensor, the class DEFAULT (the MCU) was NOT sourced — this is a
    // genuine set-member choice, not a fallback to the canonical part.
    assert!(
        !report.state.parts.iter().any(|p| p.mpn == "STM32L010F4P6"),
        "the non-default choice replaced the canonical/default Ic part"
    );
    // The un-proposed Connector class fell back to its catalog part and was still sourced.
    assert!(
        report.state.parts.iter().any(|p| p.mpn == "USB4110-GF-A"),
        "the un-proposed Connector class fell back to its catalog part"
    );

    // Replay identity holds for the set-inclusion accept run (P4).
    let replayed = replay_cmd(&log).expect("replay succeeds");
    assert_eq!(report.state, replayed);
    assert_eq!(report.state.canonical_json(), replayed.canonical_json());

    let _ = std::fs::remove_file(&log);
}

#[test]
fn hallucinated_part_proposal_is_rejected_and_never_enters_state() {
    // THE MOAT: the model proposes an MPN the catalog does not carry for the Ic class.
    let (config, log) = cfg("reject", 7);
    let report =
        run_with(hallucinated_ic_part_engine(), &config).expect("run completes with a failure");

    // The BOM phase REASONED — the proposal was made and recorded (so this is a genuine
    // propose->validate->reject, not a phase that never asked).
    let records = FileEventLog::open(&log)
        .expect("open log")
        .read_all()
        .expect("read log");
    assert!(
        made_part_reasoning_call(&records),
        "BOM Planning made a part_candidates_v1 reasoning call"
    );

    // (1) THE HALLUCINATED PART NEVER ENTERED STATE — not as a committed part event, and not in the
    // reconstructed state. The kernel refused it at validation; nothing was appended for it.
    assert!(
        !records.iter().any(|r| matches!(&r.event,
            Event::PartCommitted { part } if part.mpn == HALLUCINATED_MPN)),
        "no PartCommitted event ever carried the hallucinated MPN"
    );
    assert!(
        !report.state.parts.iter().any(|p| p.mpn == HALLUCINATED_MPN),
        "the hallucinated MPN is absent from engineering state"
    );

    // (2) THE REJECTION SURFACED as a diagnostic phase FAILURE: BOM Planning refused to source the
    // Ic class from a non-catalog part and reported it, rather than forcing the bad part in OR
    // silently substituting the catalogued one (either of which would hide the moat). Because the
    // class is genuinely unsourceable, the phase fails and the design NEVER releases — no
    // Manufacturing Generation phase is ever reached.
    let bom_outcome = report
        .outcomes
        .iter()
        .find(|(n, _)| n == "BomPlanning")
        .map(|(_, o)| o)
        .expect("BOM Planning ran");
    match bom_outcome {
        PhaseOutcome::Failed(reason) => assert!(
            reason.contains("non-catalog") || reason.to_lowercase().contains("catalog"),
            "the failure names the rejected non-catalog proposal, got: {reason}"
        ),
        PhaseOutcome::Success => panic!("BOM Planning must fail when a part proposal is rejected"),
    }
    assert!(
        !report
            .outcomes
            .iter()
            .any(|(n, _)| n == "ManufacturingGeneration"),
        "the workflow never released past the failed BOM gate"
    );
    assert!(
        matches!(report.outcomes.last(), Some((_, PhaseOutcome::Failed(_)))),
        "the run ends on the failed BOM gate, not a release"
    );

    // (3) The Connector class, which the model did NOT mispropose, fell back to its catalog part and
    // WAS sourced — so the rejection is scoped to the bad proposal, not a blanket failure.
    assert!(
        report.state.parts.iter().any(|p| p.mpn == "USB4110-GF-A"),
        "the un-misproposed Connector class still sourced its catalog part"
    );

    // Replay identity holds even for the rejected, looped-back run (P4).
    let replayed = replay_cmd(&log).expect("replay succeeds");
    assert_eq!(report.state, replayed);
    assert_eq!(report.state.canonical_json(), replayed.canonical_json());

    let _ = std::fs::remove_file(&log);
}
