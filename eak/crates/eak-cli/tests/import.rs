//! Epic E5 / increment B1: an imported KiCad design must flow through the SAME capability seam as
//! a generated one — re-validated at the runtime (P3) and recorded in the append-only event log
//! (P2), never through a back door. These tests exercise [`eak_cli::import_design`] and prove:
//!   1. the board is committed through the real seam (a `BoardCommitted` event is in the log),
//!   2. a full design (board + every net + every track) imports end-to-end — each entity funnelled
//!      through `commit`, so the event log carries one `NetCommitted`/`TrackCommitted` per entity —
//!      because imported copper-only nets carry `NetOrigin::Physical`, which the `CreateNet` seam
//!      accepts member-less (while a `Logical` member-less net is still rejected; see the runtime
//!      unit test), and never fabricates a pin, and
//!   3. malformed input is a typed error, never a panic.

use eak_cli::{import_design, CliError};
use eak_domain::NetOrigin;
use eak_ports::Event;

/// The same fixture the `eak-kicad` importer's own tests use: an outline's worth of copper with a
/// dropped net 0, three named nets, three segments, and a skipped via.
const SAMPLE: &str = r#"
    (kicad_pcb
      (net 0 "")
      (net 1 "GND")
      (net 2 "VCC")
      (net 3 "SDA")
      (segment (start 10 10) (end 40 10) (width 0.15) (layer "F.Cu") (net 2))
      (segment (start 10 12) (end 40 12) (width 0.15) (layer "F.Cu") (net 1))
      (segment (start 10 14) (end 40 14) (width 0.25) (layer "B.Cu") (net 3))
      (via (at 20 20) (size 0.6))
    )
"#;

#[test]
fn board_only_import_flows_through_the_real_seam_to_the_event_log() {
    // An outline with no reviewable copper: net 0 is dropped, no segments. The board still flows
    // through the real CreateBoard seam and is committed.
    let core = import_design("(kicad_pcb (net 0 \"\"))").expect("board-only import should succeed");

    // State: exactly one board outline, and no nets/tracks (there was no copper to route).
    assert!(
        core.state.board.is_some(),
        "board outline should be committed"
    );
    assert!(core.state.nets.is_empty());
    assert!(core.state.tracks.is_empty());

    // Event-log proof: the board landed via commit() (a BoardCommitted event), not a back door.
    let records = core.log().read_all().expect("read committed events");
    assert!(
        records
            .iter()
            .any(|r| matches!(r.event, Event::BoardCommitted { .. })),
        "the event log must record the board's commit — proof it went through commit(), not a side channel"
    );
}

#[test]
fn full_import_flows_through_commit_to_state_and_event_log() {
    // The whole design — board + every net + every track — now imports end-to-end. Each entity is
    // committed through the real capability seam (CreateBoard -> CreateNet -> RouteNet), so this is
    // proof both that the copper-only nets pass as `NetOrigin::Physical` and that nothing bypassed
    // commit(). The importer drops net 0 (unconnected) and skips the via, so SAMPLE yields exactly
    // three nets and three copper segments.
    let core = import_design(SAMPLE).expect("full import should succeed once Physical nets pass");

    // State: the board, all three nets, all three tracks were reconstructed from the fold.
    assert!(core.state.board.is_some(), "board outline committed");
    assert_eq!(
        core.state.nets.len(),
        3,
        "all three imported nets committed"
    );
    assert_eq!(
        core.state.tracks.len(),
        3,
        "all three imported tracks committed"
    );

    // Every imported net carries the physical-origin tag — the whole point of the ruling: import
    // never fabricates a member pin, it declares the net's true (copper) origin instead.
    assert!(
        core.state
            .nets
            .iter()
            .all(|n| n.origin == NetOrigin::Physical),
        "imported nets must be tagged Physical"
    );

    // Event-log proof (no back door): the log carries exactly one commit event per imported entity
    // — one BoardCommitted, one NetCommitted per net, one TrackCommitted per track. A CreateNet /
    // RouteNet request that reached commit() is exactly what produces these, so their presence and
    // count prove every entity funnelled through the runtime's single commit path.
    let records = core.log().read_all().expect("read committed events");
    let boards = records
        .iter()
        .filter(|r| matches!(r.event, Event::BoardCommitted { .. }))
        .count();
    let nets = records
        .iter()
        .filter(|r| matches!(r.event, Event::NetCommitted { .. }))
        .count();
    let tracks = records
        .iter()
        .filter(|r| matches!(r.event, Event::TrackCommitted { .. }))
        .count();
    assert_eq!(boards, 1, "one BoardCommitted event");
    assert_eq!(nets, 3, "one NetCommitted event per imported net");
    assert_eq!(tracks, 3, "one TrackCommitted event per imported track");
}

#[test]
fn malformed_input_is_a_typed_error_not_a_panic() {
    assert!(matches!(import_design(""), Err(CliError::Import(_))));
    assert!(matches!(import_design("(foo)"), Err(CliError::Import(_))));
    assert!(matches!(
        import_design("(kicad_pcb"),
        Err(CliError::Import(_))
    ));
}
