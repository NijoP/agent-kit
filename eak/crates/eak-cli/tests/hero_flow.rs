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
//! HONESTY NOTE (E7 open item — no green-but-hollow test): the default cassette is intent-independent
//! — it carries a `default` response and no keyed `entries`, so ANY intent (the hero one included)
//! resolves to the SAME canned requirements the other end-to-end tests use, and therefore reaches
//! release on the generic board. Authoring a *curated* hero cassette whose recorded requirements are
//! specific to the I²C-temperature-sensor / < 1 W design (so the released IR is genuinely the hero
//! board rather than the generic one) is the remaining E7 item. This smoke test pins the hero flow's
//! release + replay contract regardless of that follow-up.

use eak_cli::{replay_cmd, run, PhaseOutcome, ReasoningChoice, RunConfig};
use eak_ports::{Event, EventLog};
use eak_store::FileEventLog;
use std::path::PathBuf;

/// The overview's hero demo intent (epic E7): the < 1 W USB-C powered I²C temperature sensor.
const HERO_INTENT: &str = "USB-C powered I²C temperature sensor, < 1 W";

fn hero_log() -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("eak-cli-hero-{}.jsonl", std::process::id()));
    p
}

#[test]
fn hero_flow_releases_manufacturing_ir_and_replays_byte_identically() {
    let log = hero_log();
    let _ = std::fs::remove_file(&log);
    let config = RunConfig {
        intent: HERO_INTENT.into(),
        reasoning: ReasoningChoice::Fixture,
        // The default deterministic cassette: offline, fixed responses, no API key (network-free CI).
        cassette: None,
        log: log.clone(),
        model: "fixture".into(),
        // Pinned seed + logical clock -> the run is fully reproducible (P4).
        seed: 1,
        deterministic_clock: true,
    };

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
