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
use eak_domain::{BoardSide, ComponentClass, ComponentOrigin, EntityId, NetOrigin};
use eak_ports::Event;

/// A board carrying real footprints (F2): a top 0805 resistor with an explicit courtyard, a
/// bottom-side SOIC-8, and a thru-hole connector — plus one net so the copper path still runs.
const WITH_FOOTPRINTS: &str = r#"
    (kicad_pcb
      (net 0 "")
      (net 1 "GND")
      (footprint "Resistor_SMD:R_0805" (layer "F.Cu") (at 20 30 90)
        (fp_text reference "R1" (at 0 -2) (layer "F.SilkS"))
        (fp_rect (start -1 -0.7) (end 1 0.7) (layer "F.CrtYd") (width 0.05))
        (pad "1" smd roundrect (at -0.9 0) (size 1 1.2) (layers "F.Cu"))
        (pad "2" smd roundrect (at 0.9 0) (size 1 1.2) (layers "F.Cu"))
      )
      (footprint "Package_SO:SOIC-8" (layer "B.Cu") (at 50 40)
        (fp_text reference "U1" (at 0 -3) (layer "B.SilkS"))
        (pad "1" smd rect (at -2 -1.9) (size 0.6 1.5) (layers "B.Cu"))
        (pad "2" smd rect (at -2 -0.6) (size 0.6 1.5) (layers "B.Cu"))
        (pad "3" smd rect (at -2 0.6) (size 0.6 1.5) (layers "B.Cu"))
      )
      (footprint "Connector:Conn_01x02" (layer "F.Cu") (at 5 5)
        (fp_text reference "J1" (at 0 -2) (layer "F.SilkS"))
        (pad "1" thru_hole circle (at 0 0) (size 1.7 1.7) (layers "*.Cu"))
        (pad "2" thru_hole circle (at 2.54 0) (size 1.7 1.7) (layers "*.Cu"))
      )
    )
"#;

fn near(q: &eak_units::PhysicalQuantity, mm: f64) -> bool {
    (q.si_magnitude() - mm * 1e-3).abs() < 1e-9
}

#[test]
fn imported_footprints_land_as_components_and_placements_through_the_real_seam() {
    // F2 (ADR-0017): an imported board carries PARTS, not just copper. Each footprint flows through
    // the real RealizeComponent + PlaceComponent seam (≥1 pin + a board; for an Imported component NO
    // block is required), so state and the event log carry every part — proof the import went through
    // commit(), no back door. Crucially, NO synthetic intent/requirement/block spine is fabricated:
    // the components are tagged `Imported` with a null `from_block`, so traceability is honest.
    let core = import_design(WITH_FOOTPRINTS).expect("footprint import should succeed");

    // State: three components, three placements, seven pins (2 + 3 + 2). Every component is tagged
    // `Imported` with a null `from_block` — no fabricated block, no fabricated intent.
    assert_eq!(core.state.components.len(), 3, "three footprints realized");
    assert_eq!(core.state.placements.len(), 3, "three placements committed");
    assert_eq!(core.state.pins.len(), 7, "every pad became a pin");
    assert!(
        core.state
            .components
            .iter()
            .all(|c| c.origin == ComponentOrigin::Imported && c.from_block.is_null()),
        "every imported component is tagged Imported with a null block — no fabricated intent"
    );
    // The fabricated spine is GONE: no synthetic block, no reverse-engineered requirement, no intent.
    assert!(
        core.state.functional_blocks.is_empty(),
        "an import fabricates no functional block (ADR-0017)"
    );
    assert!(
        core.state.requirements.is_empty(),
        "an import fabricates no requirement (ADR-0017)"
    );
    assert!(
        core.state.intent.is_none(),
        "an import captures no design intent (ADR-0017)"
    );

    // Placement geometry matches the parsed (at ..) + layer: R1 top @ (20,30) with the 2.0 x 1.4
    // courtyard rect; U1 bottom @ (50,40).
    let comp_id = |refdes: &str| -> EntityId {
        core.state
            .components
            .iter()
            .find(|c| c.refdes == refdes)
            .unwrap()
            .id
    };
    let placement = |refdes: &str| {
        let id = comp_id(refdes);
        core.state
            .placements
            .iter()
            .find(|p| p.component == id)
            .unwrap()
    };
    let r1 = placement("R1");
    assert_eq!(r1.side, BoardSide::Top);
    assert!(near(&r1.x, 20.0) && near(&r1.y, 30.0));
    assert!(
        near(&r1.width, 2.0) && near(&r1.height, 1.4),
        "explicit courtyard honoured"
    );
    let u1 = placement("U1");
    assert_eq!(u1.side, BoardSide::Bottom);
    assert!(near(&u1.x, 50.0) && near(&u1.y, 40.0));
    assert_eq!(
        core.state
            .components
            .iter()
            .find(|c| c.refdes == "J1")
            .unwrap()
            .class,
        ComponentClass::Connector
    );

    // Event-log proof (no back door): one ComponentCommitted + one PlacementCommitted per part and
    // one PinCommitted per pad — the shapes RealizeComponent / PlaceComponent produce only by reaching
    // commit(). And, per ADR-0017, the log carries NO fabricated-spine events: zero IntentCaptured,
    // zero RequirementCommitted, zero FunctionalBlockCommitted for a footprint-only import.
    let records = core.log().read_all().expect("read committed events");
    let count = |pred: fn(&Event) -> bool| records.iter().filter(|r| pred(&r.event)).count();
    assert_eq!(count(|e| matches!(e, Event::ComponentCommitted { .. })), 3);
    assert_eq!(count(|e| matches!(e, Event::PlacementCommitted { .. })), 3);
    assert_eq!(count(|e| matches!(e, Event::PinCommitted { .. })), 7);
    assert_eq!(
        count(|e| matches!(e, Event::FunctionalBlockCommitted { .. })),
        0,
        "no fabricated functional block (ADR-0017)"
    );
    assert_eq!(
        count(|e| matches!(e, Event::IntentCaptured { .. })),
        0,
        "no fabricated design intent (ADR-0017)"
    );
    assert_eq!(
        count(|e| matches!(e, Event::RequirementCommitted { .. })),
        0,
        "no fabricated requirement (ADR-0017)"
    );
    // The copper path still ran alongside the parts (GND net + no segments here).
    assert_eq!(count(|e| matches!(e, Event::NetCommitted { .. })), 1);
}

#[test]
fn copper_only_import_commits_no_parts_no_spine() {
    // No-regression: a footprint-free board synthesizes NO import spine and NO parts — byte-for-byte
    // the pre-F2 behaviour (board + nets + tracks only), so the verify-only fixtures stay unperturbed.
    let core = import_design(SAMPLE).expect("copper-only import should succeed");
    assert!(core.state.components.is_empty());
    assert!(core.state.placements.is_empty());
    assert!(core.state.pins.is_empty());
    assert!(core.state.functional_blocks.is_empty());
    assert!(core.state.requirements.is_empty());
    assert!(core.state.intent.is_none());
    let records = core.log().read_all().expect("read committed events");
    assert!(records
        .iter()
        .all(|r| !matches!(r.event, Event::ComponentCommitted { .. })));
}

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
