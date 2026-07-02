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
//! WHY `drc-unrouted-net` is the rule under test: on an import-only design there are no
//! requirements, so the fabrication-floor rules (`drc-trace-width` = Fabrication length slot 0,
//! `drc-copper-clearance` = slot 2) have no stated floor and correctly stay SILENT — a `.kicad_pcb`
//! carries copper geometry, not a process class. `drc-unrouted-net` is the one DRC rule that fires
//! on imported copper topology alone: a net declared in the netlist with no segment realizing it is
//! an electrical break the review must catch.

use eak_cli::{import_and_verify, PhaseOutcome, Relation};

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

    // No back door / no synthesis: the review fabricated nothing. The only entities in state are the
    // ones the import committed (a board, two nets, one track); no requirement, schematic, BOM, or
    // placement was invented by the verify-only pass.
    assert!(report.state.requirements.is_empty());
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
