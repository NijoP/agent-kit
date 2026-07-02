//! Epic E5 / increment B2: the **verify-only** review closes the "import -> AI-review always works"
//! fallback. [`eak_cli::import_and_verify`] populates a `RuntimeCore` from a `.kicad_pcb` through the
//! real capability seam, then runs ONLY the verification-family machines over that same core — no
//! synthesis, no back door. These tests prove:
//!   1. an imported board whose copper trips a real rule surfaces a first-class, TRACEABLE
//!      `Violation` (it names the subject net, and a provenance link ties the two together), raised
//!      at the runtime's commit seam exactly as a generated design's would be, and
//!   2. a clean board (every declared net realized by copper) yields ZERO violations of that rule —
//!      the review does not false-positive.
//!
//! `drc-unrouted-net` fires on imported copper topology alone: a net declared in the netlist with no
//! segment realizing it is an electrical break the review must catch.
//!
//! Increment E5.1 extends this: [`import_and_verify`] now SEEDS a default fabrication process floor
//! (a `Fabrication` requirement carrying the IPC-2221 Class 2 6 mil / 0.15 mm minimum trace width and
//! copper spacing) through the real `CreateRequirement` seam BEFORE the review runs. A `.kicad_pcb`
//! carries copper geometry but no process class, so without a floor the two geometric rules
//! (`drc-trace-width` = Fabrication length slot 0, `drc-copper-clearance` = slot 2) had nothing to
//! check and stayed silent. With the seeded default they now fire, so the tests below prove an
//! imported too-thin trace raises a traceable `drc-trace-width` violation and a too-close copper pair
//! raises `drc-copper-clearance`, while clean geometry within the floor stays green (no false
//! positive). It is a DEFAULT, overridable by a stated process window.

use eak_cli::{import_and_verify, PhaseOutcome, Relation};
use eak_domain::RequirementCategory;

/// A board with an UNROUTED net: net 3 ("SDA") is declared but no segment references it, so it
/// carries no copper — `drc-unrouted-net` must flag it. Net 2 ("GND") is realized by its segment, so
/// it is clean; net 0 (empty) is dropped by the importer. Net ids 2/3 avoid the board's own
/// `EntityId(1)`, so the flagged subject is unambiguously the SDA net.
const UNROUTED: &str = r#"
    (kicad_pcb
      (net 0 "")
      (net 2 "GND")
      (net 3 "SDA")
      (segment (start 10 10) (end 40 10) (width 0.2) (layer "F.Cu") (net 2))
    )
"#;

/// The same board with SDA also realized: every declared net now carries copper, so the review is
/// clean — the true-negative counterpart to [`UNROUTED`].
const CLEAN: &str = r#"
    (kicad_pcb
      (net 0 "")
      (net 2 "GND")
      (net 3 "SDA")
      (segment (start 10 10) (end 40 10) (width 0.2) (layer "F.Cu") (net 2))
      (segment (start 10 12) (end 40 12) (width 0.2) (layer "F.Cu") (net 3))
    )
"#;

#[test]
fn import_and_verify_flags_the_unrouted_net_traceably() {
    let report = import_and_verify(UNROUTED).expect("import + verify-only review should run");

    // The review reached the DRC gate and it FAILED on the unrouted net. The three gates ahead of it
    // (constraint / ERC / BOM) find nothing on an import-only design, so DRC is where copper is read;
    // the linear plan then stops, so DRC is the last phase in the outcome log.
    let (drc_name, drc_outcome) = report
        .outcomes
        .iter()
        .find(|(name, _)| name == "DrcVerification")
        .expect("DRC Verification must be one of the verify-only phases");
    assert_eq!(drc_name, "DrcVerification");
    assert!(matches!(drc_outcome, PhaseOutcome::Failed(_)));
    assert!(
        report
            .outcomes
            .iter()
            .take_while(|(name, _)| name != "DrcVerification")
            .all(|(_, o)| matches!(o, PhaseOutcome::Success)),
        "constraint / ERC / BOM gates pass on an import-only design"
    );

    // Exactly one violation, and it is the unrouted-net finding.
    assert_eq!(
        report.state.violations.len(),
        1,
        "one unrouted net -> exactly one violation"
    );
    let v = &report.state.violations[0];
    assert_eq!(v.rule, "drc-unrouted-net");
    assert!(v.is_blocking());

    // Traceable: the violation NAMES its subject — the SDA net — and a provenance link ties the
    // violation to that net. Together (Violation -> Net) this is the anchor a reviewer follows back
    // to the offending copper.
    assert_eq!(v.subjects.len(), 1, "the violation implicates one net");
    let net = report
        .state
        .net(v.subjects[0])
        .expect("the implicated subject is a committed net");
    assert_eq!(net.name, "SDA", "the flagged net is the unrouted SDA net");
    assert!(
        report
            .state
            .links
            .iter()
            .any(|l| l.from == v.id && l.to == net.id && l.relation == Relation::TracesTo),
        "a TracesTo link must make the violation traceable to its net"
    );

    // No back door / no synthesis: the review fabricated no DESIGN data. The one requirement in
    // state is the seeded default fabrication floor (increment E5.1) — a Fabrication requirement
    // installed through the real CreateRequirement seam so the geometric DRC rules have a floor; no
    // schematic, BOM, or placement was invented by the verify-only pass.
    assert_eq!(
        report.state.requirements.len(),
        1,
        "only the seeded default fabrication floor is present"
    );
    assert_eq!(
        report.state.requirements[0].category,
        RequirementCategory::Fabrication
    );
    assert!(report.state.functional_blocks.is_empty());
    assert!(report.state.components.is_empty());
    assert!(report.state.bom_line_items.is_empty());
    assert!(report.state.placements.is_empty());
    assert_eq!(report.state.nets.len(), 2, "GND + SDA imported");
    assert_eq!(report.state.tracks.len(), 1, "only GND is routed");
}

#[test]
fn import_and_verify_clean_board_finds_no_violation() {
    let report = import_and_verify(CLEAN).expect("import + verify-only review should run");

    // Every declared net carries copper, so the unrouted-net rule (and every other) is silent — the
    // review runs all six verification phases and each passes.
    assert!(
        report
            .state
            .violations
            .iter()
            .all(|v| v.rule != "drc-unrouted-net"),
        "no unrouted-net violation on a fully realized board"
    );
    assert!(
        report.state.violations.is_empty(),
        "a clean imported board yields zero violations"
    );
    assert_eq!(
        report.outcomes.len(),
        6,
        "no gate fails, so all six verify-only phases run"
    );
    assert!(
        report
            .outcomes
            .iter()
            .all(|(_, o)| matches!(o, PhaseOutcome::Success)),
        "every verify-only phase passes on a clean board"
    );
    // Both nets are realized by their own copper.
    assert_eq!(report.state.nets.len(), 2);
    assert_eq!(report.state.tracks.len(), 2);
}

// ----------------------- Increment E5.1: geometric DRC fires on import -----------------------

/// A board with a routed net whose copper is FINER than the seeded 0.15 mm process floor: the GND
/// segment is 0.10 mm wide (below 6 mil / 0.15 mm), so `drc-trace-width` must flag it. The net is
/// realized (it carries the segment), so `drc-unrouted-net` stays silent — the flag is unambiguously
/// the too-thin trace. Net id 2 avoids the board's own `EntityId(1)`.
const TOO_THIN: &str = r#"
    (kicad_pcb
      (net 0 "")
      (net 2 "GND")
      (segment (start 10 10) (end 40 10) (width 0.10) (layer "F.Cu") (net 2))
    )
"#;

/// A board with two same-layer segments on DIFFERENT nets whose centre-lines are 0.30 mm apart; at
/// 0.20 mm wide each the edge-to-edge gap is 0.10 mm, below the seeded 0.15 mm copper-clearance
/// floor, so `drc-copper-clearance` must flag the pair. Both segments are 0.20 mm (> 0.15 mm) and
/// both nets are realized, so neither `drc-trace-width` nor `drc-unrouted-net` fires.
const TOO_CLOSE: &str = r#"
    (kicad_pcb
      (net 0 "")
      (net 2 "GND")
      (net 3 "SDA")
      (segment (start 10 10) (end 40 10) (width 0.20) (layer "F.Cu") (net 2))
      (segment (start 10 10.3) (end 40 10.3) (width 0.20) (layer "F.Cu") (net 3))
    )
"#;

/// A board whose only net is realized by copper WITHIN the default floor: a single 0.20 mm segment
/// (wider than the 0.15 mm minimum) on a net that carries it. The seeded floor is present, yet the
/// geometric rules find nothing — the true-negative that guards against a false positive.
const WITHIN_FLOOR: &str = r#"
    (kicad_pcb
      (net 0 "")
      (net 2 "GND")
      (segment (start 10 10) (end 40 10) (width 0.20) (layer "F.Cu") (net 2))
    )
"#;

#[test]
fn import_and_verify_flags_a_too_thin_trace_via_the_seeded_floor() {
    let report = import_and_verify(TOO_THIN).expect("import + verify-only review should run");

    // The seeded default floor is what makes this fire: a Fabrication requirement carrying the
    // 0.15 mm minimum trace width (slot 0), installed through the real CreateRequirement seam.
    assert_eq!(report.state.requirements.len(), 1);
    assert_eq!(
        report.state.requirements[0].category,
        RequirementCategory::Fabrication
    );

    // The DRC gate FAILED on the trace-width rule (the plan is linear, so it stops there).
    let (_, drc_outcome) = report
        .outcomes
        .iter()
        .find(|(name, _)| name == "DrcVerification")
        .expect("DRC Verification must run");
    assert!(matches!(drc_outcome, PhaseOutcome::Failed(_)));

    // Exactly one violation, and it is the trace-width finding — no unrouted-net (the net is
    // routed) and no clearance (a single track has no pair).
    assert_eq!(report.state.violations.len(), 1);
    let v = &report.state.violations[0];
    assert_eq!(v.rule, "drc-trace-width");
    assert!(v.is_blocking());

    // Traceable: the violation NAMES the offending track, and a TracesTo link ties them together —
    // the anchor a reviewer follows back to the copper (Violation -> Track -> Net -> ...).
    assert_eq!(v.subjects.len(), 1, "one too-thin track");
    let track = report
        .state
        .tracks
        .iter()
        .find(|t| t.id == v.subjects[0])
        .expect("the implicated subject is a committed track");
    assert!(
        report
            .state
            .links
            .iter()
            .any(|l| l.from == v.id && l.to == track.id && l.relation == Relation::TracesTo),
        "a TracesTo link must make the trace-width violation traceable to its track"
    );
}

#[test]
fn import_and_verify_flags_too_close_copper_via_the_seeded_floor() {
    let report = import_and_verify(TOO_CLOSE).expect("import + verify-only review should run");

    // The seeded floor supplies slot 2 (0.15 mm copper-to-copper clearance).
    assert_eq!(report.state.requirements.len(), 1);

    let (_, drc_outcome) = report
        .outcomes
        .iter()
        .find(|(name, _)| name == "DrcVerification")
        .expect("DRC Verification must run");
    assert!(matches!(drc_outcome, PhaseOutcome::Failed(_)));

    // Exactly one violation, and it is the copper-clearance finding on the two-track pair — no
    // trace-width (both 0.20 mm > 0.15 mm) and no unrouted-net (both nets realized).
    assert_eq!(report.state.violations.len(), 1);
    let v = &report.state.violations[0];
    assert_eq!(v.rule, "drc-copper-clearance");
    assert!(v.is_blocking());

    // Traceable to BOTH offending tracks, each with a TracesTo link.
    assert_eq!(
        v.subjects.len(),
        2,
        "a clearance breach implicates both tracks"
    );
    for subj in &v.subjects {
        assert!(
            report.state.tracks.iter().any(|t| t.id == *subj),
            "each subject is a committed track"
        );
        assert!(
            report
                .state
                .links
                .iter()
                .any(|l| l.from == v.id && l.to == *subj && l.relation == Relation::TracesTo),
            "a TracesTo link must tie the clearance violation to each track"
        );
    }
}

#[test]
fn import_and_verify_seeds_floor_yet_clean_geometry_passes() {
    let report = import_and_verify(WITHIN_FLOOR).expect("import + verify-only review should run");

    // The default floor IS seeded (proof it is installed, not merely absent)...
    assert_eq!(report.state.requirements.len(), 1);
    assert_eq!(
        report.state.requirements[0].category,
        RequirementCategory::Fabrication
    );
    assert_eq!(
        report.state.requirements[0].targets.len(),
        3,
        "the three positional Fabrication slots (width, edge keep-out, clearance)"
    );

    // ...yet copper within the floor trips NEITHER geometric rule — no false positive.
    assert!(
        report
            .state
            .violations
            .iter()
            .all(|v| v.rule != "drc-trace-width" && v.rule != "drc-copper-clearance"),
        "geometry within the default floor raises no trace-width/clearance violation"
    );
    assert!(
        report.state.violations.is_empty(),
        "a clean board within the floor yields zero violations"
    );
    assert_eq!(
        report.outcomes.len(),
        6,
        "no gate fails, so all six verify-only phases run"
    );
}
