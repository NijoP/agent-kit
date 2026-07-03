//! Increment D1 (epic E7) — the HERO-FLOW SMOKE TEST: a CI guard so the demo can't silently rot.
//!
//! Drives the overview's hero demo intent — "USB-C powered I²C temperature sensor, < 1 W" — through
//! the full deterministic `default_workflow()` on the offline `FixtureEngine` cassette (no network,
//! fixed responses) and pins the two properties the live demo depends on:
//!
//!   1. RELEASE (P6): all fifteen phases run clean and the terminal Manufacturing Generation phase
//!      RELEASES a `ManufacturingIr` — the global gate found no open blocking violation, the design
//!      lowered to the terminal IR, and its release milestone was recorded with a shape consistent
//!      with the committed board / parts / tracks. We assert the released IR itself, not merely that
//!      "the phase said OK".
//!   2. REPLAY (P4): folding the event log rebuilds byte-identical state, so a run and its replay
//!      can never diverge — the demo is reproducible.
//!
//! This is a distinct guard from `integration.rs::run_replays_byte_identical_and_runs_fifteen_phases`
//! (which drives the generic "IoT sensor node, < 5 W, < 50x50 mm" intent and asserts phase Success +
//! state shape): this test pins the specific hero intent AND cross-checks the released IR's recorded
//! `Event::ManufacturingGenerated` counts against committed state.
//!
//! CURATED HERO CASSETTE (E7, DONE): the hero run is now driven by `fixtures/hero_cassette.json`
//! — an AUTHORED, offline stand-in for the reasoning model (labelled as such in that file's `_note`)
//! whose `requirement_candidates_v1` response encodes requirements SPECIFIC to the hero design
//! (a USB-C power input, an I²C temperature-sensor IC, a < 1 W power budget, a compact board),
//! and whose `part_candidates_v1` response names catalog-carried MPNs for the two classes those
//! requirements realize (Connector + Ic). So the released IR is genuinely the hero board — an
//! on-topic USB-C / I²C / temperature / < 1 W design — not the generic one, while still reaching a
//! clean release and replaying byte-identically. The cassette is authored to fit the existing
//! deterministic phases (no Regulator, whose catalog part is EOL and would block release; no
//! Fabrication/frequency/thermal/current targets, so those gates stay silent), NOT the other way
//! round — the phase/classify logic is untouched.

use eak_cli::{replay_cmd, run, PhaseOutcome, ReasoningChoice, Relation, RunConfig};
use eak_domain::RequirementCategory;
use eak_ports::{Event, EventLog};
use eak_store::FileEventLog;
use std::path::{Path, PathBuf};

/// The overview's hero demo intent (epic E7): the < 1 W USB-C powered I²C temperature sensor.
const HERO_INTENT: &str = "USB-C powered I²C temperature sensor, < 1 W";

/// The curated hero cassette (E7): authored requirements + part candidates specific to the hero
/// design. Resolved relative to this crate so the test is CWD-independent.
fn hero_cassette() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/hero_cassette.json")
}

fn hero_log(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("eak-cli-hero-{}-{}.jsonl", tag, std::process::id()));
    p
}

/// The hero run configuration: the frozen hero intent on the curated hero cassette, with a pinned
/// seed + logical clock so the run is fully reproducible (P4). Shared by both hero guards.
fn hero_config(log: &Path) -> RunConfig {
    RunConfig {
        intent: HERO_INTENT.into(),
        reasoning: ReasoningChoice::Fixture,
        // The CURATED hero cassette (E7): offline, fixed, no API key (network-free CI), but its
        // recorded requirements + part candidates are authored for THIS design, so the released IR
        // is the hero board rather than the generic one. Selecting it explicitly (vs the intent-
        // generic default cassette) is how the hero run is keyed to hero-specific judgement.
        cassette: Some(hero_cassette()),
        log: log.to_path_buf(),
        model: "fixture".into(),
        seed: 1,
        deterministic_clock: true,
    }
}

#[test]
fn hero_flow_releases_manufacturing_ir_and_replays_byte_identically() {
    let log = hero_log("release");
    let _ = std::fs::remove_file(&log);
    let config = hero_config(&log);

    let report = run(&config).expect("hero run succeeds");

    // This really is the hero flow: the captured design intent is the demo's, verbatim.
    let intent = report.state.intent.as_ref().expect("intent captured");
    assert_eq!(intent.statement, HERO_INTENT);

    // (1) RELEASE — all fifteen phases ran clean and the terminal Manufacturing Generation phase
    // RELEASED: the global gate found no open blocking violation, so the design lowered to a
    // Manufacturing IR.
    assert_eq!(report.outcomes.len(), 15);
    assert!(report
        .outcomes
        .iter()
        .all(|(_, o)| matches!(o, PhaseOutcome::Success)));
    assert_eq!(report.outcomes.last().unwrap().0, "ManufacturingGeneration");
    assert!(
        report.state.open_blocking_violations().is_empty(),
        "a released design has no open blocking violation"
    );

    // The released design carries the expected board / parts / tracks shape: a board outline, at
    // least one placement, sourced parts + BOM line items, and copper realizing the nets.
    assert!(report.state.board.is_some(), "released design has a board");
    assert!(
        !report.state.placements.is_empty(),
        "components are placed on the board"
    );
    assert!(!report.state.parts.is_empty(), "components are sourced");
    assert!(
        !report.state.bom_line_items.is_empty(),
        "the BOM binds parts to components"
    );
    assert!(
        !report.state.tracks.is_empty(),
        "nets are realized in copper"
    );

    // (1b) HERO-SPECIFIC — the released design is genuinely the demo's USB-C / I²C / temperature /
    // < 1 W sensor, not the generic board. These pin the CURATED cassette's on-topic requirements
    // and the parts they source. We assert on domain STRUCTURE (categories, targets, component
    // classes, catalog MPNs) rather than brittle prose, so wording can be re-authored without
    // breaking the guard — but we do check the hero keywords survive into the committed requirements.
    let reqs = &report.state.requirements;
    assert_eq!(
        reqs.len(),
        4,
        "the curated hero cassette commits its four authored requirements"
    );

    // The requirement statements, lowercased, so keyword checks are case-insensitive.
    let statements: Vec<String> = reqs.iter().map(|r| r.statement.to_lowercase()).collect();
    let mentions = |needle: &str| statements.iter().any(|s| s.contains(needle));

    // The hero domain is present in the committed requirements: USB-C power entry, an I²C
    // temperature sensor, and an explicit power budget.
    assert!(
        mentions("usb-c"),
        "a requirement names the USB-C power input"
    );
    assert!(
        mentions("i²c"),
        "a requirement names the I²C sensor interface"
    );
    assert!(
        mentions("temperature"),
        "a requirement names the temperature-sensor function"
    );

    // The < 1 W power budget is a real, machine-checkable constraint: an Electrical requirement
    // carrying a 1 W (or tighter) power ceiling, lowered into a constraint the gates verified clean.
    let power_budget = reqs.iter().find(|r| {
        r.category == RequirementCategory::Electrical
            && r.statement.to_lowercase().contains("power")
            && r.targets
                .iter()
                .any(|t| t.dimension() == eak_units::Dimension::Power && t.magnitude <= 1.0)
    });
    assert!(
        power_budget.is_some(),
        "the < 1 W power budget is a committed Electrical requirement with a <= 1 W power target"
    );
    assert!(
        !report.state.constraints.is_empty(),
        "the hero requirements lowered to machine-checkable constraints"
    );

    // The design synthesizes the right hardware for the hero intent: a USB-C power source
    // (Connector) driving the I²C sensor host (Ic) — and, per the moat, both are sourced to the
    // catalog parts the cassette PROPOSED and the kernel VALIDATED (the model never wrote state).
    use eak_domain::ComponentClass;
    assert!(
        report
            .state
            .components
            .iter()
            .any(|c| c.class == ComponentClass::Connector),
        "a USB-C connector realizes the power input"
    );
    assert!(
        report
            .state
            .components
            .iter()
            .any(|c| c.class == ComponentClass::Ic),
        "an IC realizes the I²C temperature-sensor host"
    );
    // No Regulator was synthesized (the hero wording avoids the EOL catalog regulator that would
    // block release) — the curated design is catalog-clean by construction.
    assert!(
        report
            .state
            .components
            .iter()
            .all(|c| c.class != ComponentClass::Regulator),
        "the curated hero design carries no (EOL) regulator, so it releases clean"
    );

    // The BOM sources hero-appropriate, catalog-valid parts: the USB-C receptacle and — the C2.1
    // honesty fix — the I²C temperature sensor itself (a NON-default, in-catalog Ic member), NOT the
    // MCU. So a temperature-sensor design now sources a temperature-sensor MPN, validated by catalog
    // set-inclusion (the model proposed the sensor; the kernel confirmed it is in the Ic set).
    let mpns: Vec<&str> = report.state.parts.iter().map(|p| p.mpn.as_str()).collect();
    assert!(
        mpns.contains(&"USB4110-GF-A"),
        "the BOM sources the USB-C receptacle part"
    );
    assert!(
        mpns.contains(&"TMP102AIDRLR"),
        "the BOM sources the I²C temperature-sensor part (honest hero BOM), not the MCU"
    );
    assert!(
        !mpns.contains(&"STM32L010F4P6"),
        "the hero sensor Ic no longer sources the MCU MPN (E7 honesty fix)"
    );

    // The RELEASE MILESTONE is real, not merely a green phase: the terminal Manufacturing IR was
    // projected and its `Event::ManufacturingGenerated` audit event recorded — exactly once — with a
    // shape consistent with the committed design. The projection makes one pick-and-place assignment
    // per placement, one copper directive per routed track, and carries the committed BOM line items,
    // so these three counts must equal the state we just released. This pins the released IR itself.
    let records = FileEventLog::open(&log)
        .expect("open event log")
        .read_all()
        .expect("read event log");
    let releases: Vec<(u32, usize, usize, usize)> = records
        .iter()
        .filter_map(|r| match &r.event {
            Event::ManufacturingGenerated {
                schema_version,
                place_count,
                copper_count,
                line_item_count,
            } => Some((
                *schema_version,
                *place_count,
                *copper_count,
                *line_item_count,
            )),
            _ => None,
        })
        .collect();
    assert_eq!(
        releases.len(),
        1,
        "the hero design is released exactly once"
    );
    let (schema_version, place_count, copper_count, line_item_count) = releases[0];
    assert!(
        schema_version >= 1,
        "the released Manufacturing IR carries a schema version"
    );
    assert_eq!(
        place_count,
        report.state.placements.len(),
        "one pick-and-place assignment per placement"
    );
    assert_eq!(
        copper_count,
        report.state.tracks.len(),
        "one copper directive per routed track"
    );
    assert_eq!(
        line_item_count,
        report.state.bom_line_items.len(),
        "the assembly BOM equals the committed line items"
    );
    assert!(
        place_count > 0 && copper_count > 0 && line_item_count > 0,
        "the released Manufacturing IR is non-empty"
    );

    // (2) REPLAY — folding the event log rebuilds byte-identical state (P4): the demo is
    // deterministic and reproducible, so it cannot silently rot between the live run and its replay.
    let replayed = replay_cmd(&log).expect("replay succeeds");
    assert_eq!(report.state, replayed);
    assert_eq!(report.state.canonical_json(), replayed.canonical_json());

    let _ = std::fs::remove_file(&log);
}

/// E7 — the CURATED HERO CASSETTE actually drove the run, and every committed requirement is
/// provenance-traceable to the hero intent. This is the moat made testable for the hero demo: the
/// reasoning boundary recorded the AUTHORED hero judgement verbatim (an `Event::ReasoningCall`
/// under the `requirement_candidates_v1` schema), and the deterministic half then committed only
/// validated requirements, each rooted in the captured design intent. Complementary to the
/// release+replay guard above: that one pins the *output* IR; this one pins that the on-topic
/// requirements came through the reasoning seam (not a hardcoded phase) and are fully traceable.
#[test]
fn hero_run_is_cassette_driven_and_every_requirement_traces_to_intent() {
    let log = hero_log("provenance");
    let _ = std::fs::remove_file(&log);
    let report = run(&hero_config(&log)).expect("hero run succeeds");

    let intent = report.state.intent.as_ref().expect("intent captured");
    assert_eq!(intent.statement, HERO_INTENT);

    // The reasoning boundary recorded the curated hero judgement: at least one `ReasoningCall`
    // under the requirement schema whose RESPONSE candidates carry the hero domain and the < 1 W
    // power budget. Because replay re-folds this recorded response WITHOUT calling the model, the
    // hero requirements are reproducible offline (P4) — the cassette, not a phase, produced them.
    let records = FileEventLog::open(&log)
        .expect("open event log")
        .read_all()
        .expect("read event log");
    let requirement_calls: Vec<&eak_ports::ReasoningResponse> = records
        .iter()
        .filter_map(|r| match &r.event {
            Event::ReasoningCall { request, response }
                if request.schema_name == "requirement_candidates_v1" =>
            {
                Some(response)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        requirement_calls.len(),
        1,
        "the Requirement agent made exactly one recorded reasoning call"
    );
    let proposed = requirement_calls[0];
    let proposed_lc: Vec<String> = proposed
        .candidates
        .iter()
        .map(|c| c.statement.to_lowercase())
        .collect();
    assert!(
        proposed_lc.iter().any(|s| s.contains("usb-c")),
        "the recorded reasoning proposed a USB-C requirement"
    );
    assert!(
        proposed_lc.iter().any(|s| s.contains("i²c")),
        "the recorded reasoning proposed an I²C requirement"
    );
    assert!(
        proposed.candidates.iter().any(|c| c
            .targets
            .iter()
            .any(|t| t.dimension() == eak_units::Dimension::Power && t.magnitude <= 1.0)),
        "the recorded reasoning proposed the < 1 W power budget"
    );

    // Provenance-by-construction: every committed requirement is rooted in the hero intent (its
    // source is the intent AND a DerivedFrom link backs the hop), and is justified by a decision
    // that points back at the reasoning call. Unvalidated model text never becomes a requirement.
    assert!(!report.state.requirements.is_empty());
    for r in &report.state.requirements {
        assert_eq!(
            r.source, intent.id,
            "requirement is rooted in the hero intent"
        );
        assert!(
            report.state.links.iter().any(|l| l.from == r.id
                && l.to == intent.id
                && l.relation == Relation::DerivedFrom),
            "a DerivedFrom link backs requirement -> intent"
        );
        assert!(
            report
                .state
                .links
                .iter()
                .any(|l| l.from == r.id && l.relation == Relation::JustifiedBy),
            "the requirement is justified by a decision"
        );
    }

    let _ = std::fs::remove_file(&log);
}
