//! `ImportedDesign` → `.kicad_pcb` export — the mirror of [`crate::import_kicad_pcb`].
//!
//! This emits the copper-realization subset the importer round-trips: a `(net …)` table, one
//! `(segment …)` per [`Track`], plus a header (version/generator/layer table) and an `Edge.Cuts`
//! rectangle so the reused KiCanvas renderer can draw the board and its copper. It fabricates no
//! per-entity KiCad fields it cannot see (P4/P9): while an [`ImportedDesign`] may now carry parsed
//! `components` (F2), **no `(footprint …)` nodes are emitted yet** — footprint *export* (round-tripping
//! parts back to KiCad) is a deliberate follow-up, so parts are simply omitted from this IR here,
//! never half-guessed. The copper subset (nets + segments + outline) round-trips exactly as before.
//!
//! Determinism (E4/testability): the same [`ImportedDesign`] always yields the byte-identical
//! string. Nothing here reads a clock, a UUID source, or hash-ordered map — every list is walked in
//! its `Vec` order and every number is formatted to KiCad's native 1 nm (6-decimal-mm) grid.
//!
//! Fidelity boundary (honesty): the importer does **not** parse an outline — it re-derives the
//! [`Board`](eak_domain::Board) from the copper bounding box. So the `Edge.Cuts` rectangle we emit
//! is a *render aid*, and an exact board round-trip holds only when the board already equals that
//! bounding-box value — which is precisely the board the importer itself produces. Net *class* and
//! *members*, and the optional `current` / `impedance_target`, are likewise not KiCad concepts: on
//! re-import class is re-inferred from the net name and the rest come back empty/`None`, exactly as
//! the importer defines them.

use crate::ImportedDesign;
use eak_domain::{BoardSide, Track};
use eak_units::PhysicalQuantity;

/// Serialize an [`ImportedDesign`] into a valid `(kicad_pcb …)` document.
///
/// The `(net …)` and `(segment …)` nodes are exactly the shapes [`crate::import_kicad_pcb`] reads,
/// so `import → export → import` is a fixed point over `board`, `nets`, and `tracks`. Deterministic:
/// same input, same bytes.
pub fn export(design: &ImportedDesign) -> String {
    let mut out = String::new();

    // --- header: the minimum KiCanvas needs to map copper + edge layers (importer ignores it) ---
    out.push_str("(kicad_pcb (version 20221018) (generator \"eak\")\n");
    out.push_str("  (layers\n");
    out.push_str("    (0 \"F.Cu\" signal)\n");
    out.push_str("    (31 \"B.Cu\" signal)\n");
    out.push_str("    (44 \"Edge.Cuts\" user)\n");
    out.push_str("  )\n");

    // --- net table: net 0 is the format-mandated unconnected net (dropped on re-import), then the
    //     design's own nets by id. Names are quoted + escaped so any character survives the parser.
    out.push_str("  (net 0 \"\")\n");
    for net in &design.nets {
        out.push_str(&format!("  (net {} {})\n", net.id.0, quote(&net.name)));
    }

    // --- board outline: an Edge.Cuts rectangle at the board's real width x height. The importer
    //     re-derives the board from the copper bbox and ignores this; it exists so KiCanvas draws
    //     the edge. Origin is (0,0), matching the importer's bbox-from-origin convention.
    let w = mm_str(&design.board.width);
    let h = mm_str(&design.board.height);
    let zero = "0";
    for (sx, sy, ex, ey) in [
        (zero, zero, w.as_str(), zero),
        (w.as_str(), zero, w.as_str(), h.as_str()),
        (w.as_str(), h.as_str(), zero, h.as_str()),
        (zero, h.as_str(), zero, zero),
    ] {
        out.push_str(&format!(
            "  (gr_line (start {sx} {sy}) (end {ex} {ey}) (layer \"Edge.Cuts\") (width 0.1))\n"
        ));
    }

    // --- copper: one segment per track, in Vec order so re-minted track ids (10_000 + i) line up.
    for track in &design.tracks {
        out.push_str(&segment(track));
    }

    out.push_str(")\n");
    out
}

/// One `(segment …)` node in the exact field order/shape the importer's `sub()` lookups expect.
fn segment(t: &Track) -> String {
    format!(
        "  (segment (start {} {}) (end {} {}) (width {}) (layer \"{}\") (net {}))\n",
        mm_str(&t.x1),
        mm_str(&t.y1),
        mm_str(&t.x2),
        mm_str(&t.y2),
        mm_str(&t.width),
        layer_token(t.layer),
        t.net.0,
    )
}

/// The KiCad copper-layer name for a board side — the inverse of the importer's `"B."`-prefix test.
fn layer_token(side: BoardSide) -> &'static str {
    match side {
        BoardSide::Top => "F.Cu",
        BoardSide::Bottom => "B.Cu",
    }
}

/// A length quantity as a millimetre string. KiCad geometry is millimetres, so we convert from the
/// IR's SI (metre) magnitude regardless of the source unit, then snap to KiCad's 1 nm grid.
fn mm_str(q: &PhysicalQuantity) -> String {
    fmt_mm(q.si_magnitude() * 1_000.0)
}

/// Format a millimetre value to KiCad's native 1 nm (6-decimal) resolution, trimming trailing
/// zeros so the output is clean and deterministic (e.g. `0.150000` → `0.15`, `10.000000` → `10`).
fn fmt_mm(v: f64) -> String {
    let s = format!("{v:.6}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-0" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Quote a string as a KiCad atom, escaping `"` and `\` so the importer's string reader recovers it.
fn quote(s: &str) -> String {
    let mut q = String::with_capacity(s.len() + 2);
    q.push('"');
    for c in s.chars() {
        if c == '"' || c == '\\' {
            q.push('\\');
        }
        q.push(c);
    }
    q.push('"');
    q
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import_kicad_pcb;
    use eak_domain::{Board, EntityId, LayerStack, Net, NetClass, NetOrigin};
    use eak_units::Unit;

    /// A real-shaped board with power/ground/signal nets on both copper sides, plus a via the
    /// importer skips — enough to exercise every export branch.
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

    fn q_eq(a: &PhysicalQuantity, b: &PhysicalQuantity) -> bool {
        a.same_value(b).unwrap_or(false)
    }

    fn mm(v: f64) -> PhysicalQuantity {
        PhysicalQuantity::new(v, Unit::Millimetre)
    }

    #[test]
    fn output_has_the_expected_shape() {
        let d = import_kicad_pcb(SAMPLE).unwrap();
        let s = export(&d);

        assert!(s.starts_with("(kicad_pcb"));
        // exactly one segment per track (the token is unique to copper segments).
        assert_eq!(s.matches("(segment").count(), d.tracks.len());
        // every declared net appears with its id + quoted name.
        for n in &d.nets {
            assert!(
                s.contains(&format!("(net {} \"{}\")", n.id.0, n.name)),
                "missing net {}",
                n.name
            );
        }
        // header + outline so KiCanvas can map copper and draw the edge.
        assert!(s.contains("\"F.Cu\"") && s.contains("\"B.Cu\""));
        assert!(s.contains("Edge.Cuts"));
        // no placements in the IR -> no footprints fabricated (P4/P9).
        assert!(!s.contains("(footprint"));
    }

    #[test]
    fn export_is_deterministic() {
        let d = import_kicad_pcb(SAMPLE).unwrap();
        assert_eq!(
            export(&d),
            export(&d),
            "same IR must yield byte-identical output"
        );
    }

    #[test]
    fn import_export_import_is_a_fixed_point() {
        let d1 = import_kicad_pcb(SAMPLE).unwrap();
        let d2 = import_kicad_pcb(&export(&d1)).unwrap();

        // board: both are re-derived from the identical copper bbox, so they match.
        assert!(q_eq(&d1.board.width, &d2.board.width));
        assert!(q_eq(&d1.board.height, &d2.board.height));
        assert_eq!(d1.board.stack.layers.len(), d2.board.stack.layers.len());

        // nets: id/name/class and the carried-empty fields all survive.
        assert_eq!(d1.nets.len(), d2.nets.len());
        for (a, b) in d1.nets.iter().zip(&d2.nets) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.name, b.name);
            assert_eq!(a.class, b.class);
            assert_eq!(a.members, b.members);
            assert!(a.current.is_none() && b.current.is_none());
            assert!(a.impedance_target.is_none() && b.impedance_target.is_none());
        }

        // tracks: id/net/layer/width/endpoints all survive.
        assert_eq!(d1.tracks.len(), d2.tracks.len());
        for (a, b) in d1.tracks.iter().zip(&d2.tracks) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.net, b.net);
            assert_eq!(a.layer, b.layer);
            assert!(q_eq(&a.width, &b.width));
            assert!(q_eq(&a.x1, &b.x1));
            assert!(q_eq(&a.y1, &b.y1));
            assert!(q_eq(&a.x2, &b.x2));
            assert!(q_eq(&a.y2, &b.y2));
        }
    }

    #[test]
    fn hand_built_design_round_trips() {
        // Build an ImportedDesign directly (not via the parser) to prove export handles arbitrary
        // IR, not just re-emitted input. Board dims are set to the importer's bbox formula
        // (max_endpoint + 5, floored at 10) so the derived board round-trips exactly.
        let nets = vec![
            Net {
                id: EntityId(1),
                name: "GND".into(),
                class: NetClass::Ground,
                members: vec![],
                current: None,
                impedance_target: None,
                origin: NetOrigin::Logical,
            },
            Net {
                id: EntityId(2),
                name: "+3V3".into(),
                class: NetClass::Power,
                members: vec![],
                current: None,
                impedance_target: None,
                origin: NetOrigin::Logical,
            },
        ];
        let tracks = vec![
            Track {
                id: EntityId(10_000),
                net: EntityId(2),
                layer: BoardSide::Top,
                width: mm(0.3),
                x1: mm(5.0),
                y1: mm(5.0),
                x2: mm(25.0),
                y2: mm(5.0),
            },
            Track {
                id: EntityId(10_001),
                net: EntityId(1),
                layer: BoardSide::Bottom,
                width: mm(0.2),
                x1: mm(5.0),
                y1: mm(8.0),
                x2: mm(25.0),
                y2: mm(8.0),
            },
        ];
        // bbox: max_x = 25 -> width = 30; max_y = 8 -> height = 13.
        let board = Board {
            id: EntityId(1),
            width: mm(30.0),
            height: mm(13.0),
            stack: LayerStack::standard_two_layer(),
        };
        let d1 = ImportedDesign {
            board,
            nets,
            tracks,
            components: vec![],
            warnings: vec![],
        };

        let d2 = import_kicad_pcb(&export(&d1)).unwrap();

        assert!(q_eq(&d1.board.width, &d2.board.width));
        assert!(q_eq(&d1.board.height, &d2.board.height));
        assert_eq!(d1.nets.len(), d2.nets.len());
        for (a, b) in d1.nets.iter().zip(&d2.nets) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.name, b.name);
            assert_eq!(a.class, b.class); // Power/Ground re-inferred from the name
        }
        assert_eq!(d1.tracks.len(), d2.tracks.len());
        for (a, b) in d1.tracks.iter().zip(&d2.tracks) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.net, b.net);
            assert_eq!(a.layer, b.layer);
            assert!(q_eq(&a.width, &b.width));
            assert!(q_eq(&a.x1, &b.x1));
            assert!(q_eq(&a.y2, &b.y2));
        }
    }

    #[test]
    fn numbers_snap_to_kicad_grid_cleanly() {
        assert_eq!(fmt_mm(0.15), "0.15");
        assert_eq!(fmt_mm(10.0), "10");
        assert_eq!(fmt_mm(0.0), "0");
        assert_eq!(fmt_mm(-0.0), "0");
        assert_eq!(fmt_mm(40.5), "40.5");
    }

    #[test]
    fn quote_escapes_specials() {
        assert_eq!(quote("GND"), "\"GND\"");
        assert_eq!(quote("A\"B"), "\"A\\\"B\"");
        assert_eq!(quote("C\\D"), "\"C\\\\D\"");
    }
}
