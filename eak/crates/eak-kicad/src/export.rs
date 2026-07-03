//! `ImportedDesign` → `.kicad_pcb` export — the mirror of [`crate::import_kicad_pcb`].
//!
//! This emits the copper-realization subset the importer round-trips: a `(net …)` table, one
//! `(segment …)` per [`Track`], plus a header (version/generator/layer table) and an `Edge.Cuts`
//! rectangle so the reused KiCanvas renderer can draw the board and its copper. Since increment F3 it
//! also emits one `(footprint …)` per parsed [`crate::ImportedComponent`] (F2), completing the canvas
//! round-trip for *parts*: the refdes as `(fp_text reference …)`, the side `(layer …)`, the placement
//! centre `(at …)`, a courtyard `(fp_rect …)` sized to the placement so its width/height re-derive, and
//! one `(pad …)` per [`Pin`](eak_domain::Pin) carrying that pin's designation and — since increment G1
//! — its `(net IDX "name")` when the pad was captured on a net ([`crate::ImportedComponent::pin_nets`]),
//! so pad→net membership survives import→export→import too. It emits **only the subset the importer
//! parses back** (P4/P9): the footprint library-id is a self-labelled render scaffold and pad geometry
//! is filler — because [`Pin`](eak_domain::Pin) itself carries no position, none is invented; an
//! unconnected pad (no captured net) emits no `(net …)` node, so nothing is invented there either. A
//! copper-only design (no `components`) emits no footprints, exactly as before F3.
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

use crate::{ImportedComponent, ImportedDesign};
use eak_domain::{BoardSide, Track};
use eak_units::PhysicalQuantity;
use std::collections::BTreeMap;

/// Serialize an [`ImportedDesign`] into a valid `(kicad_pcb …)` document.
///
/// The `(net …)`, `(segment …)`, and `(footprint …)` nodes are exactly the shapes
/// [`crate::import_kicad_pcb`] reads, so `import → export → import` is a fixed point over `board`,
/// `nets`, `tracks`, and (since F3) `components`/placements/pins. Deterministic: same input, same
/// bytes.
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

    // --- parts (F3): one footprint per imported component, in Vec order so the importer re-mints the
    //     same component/pin/placement ids (20_000 + i / 30_000 + running / 40_000 + i) on re-import.
    //     A net index -> name lookup (G1) lets each connected pad re-emit its `(net IDX "name")`.
    let net_names: BTreeMap<u128, &str> = design
        .nets
        .iter()
        .map(|n| (n.id.0, n.name.as_str()))
        .collect();
    for ic in &design.components {
        out.push_str(&footprint(ic, &net_names));
    }

    out.push_str(")\n");
    out
}

/// One `(footprint …)` node in the exact subset the importer parses back (F2 → F3 round-trip): the
/// refdes as an `(fp_text reference …)`, the side `(layer …)`, the placement centre `(at …)`, a
/// courtyard `(fp_rect …)` centred on the origin whose extent equals the placement's width × height
/// (so `footprint_courtyard` re-derives that exact size instead of the class default), and one
/// `(pad …)` per pin whose second atom is the pin's designation and which — since G1 — carries a
/// `(net IDX "name")` when that pin was captured on a net (both the fields the importer reads).
///
/// Fields the importer does not read are handled honestly (P4/P9): the library-id is a self-labelled
/// `"eak:<refdes>"` render scaffold — the importer identifies the part by its `(fp_text reference …)`,
/// not this name — and pad geometry is minimal filler, because a [`Pin`](eak_domain::Pin) itself
/// carries no position; all pads therefore sit at the footprint origin. A pin captured on no net
/// (`pin_nets` entry `None`) emits no `(net …)` node — an unconnected pad, invented nowhere. None of
/// the filler is asserted by the round-trip, so nothing invented is silently dropped.
fn footprint(ic: &ImportedComponent, net_names: &BTreeMap<u128, &str>) -> String {
    let c = &ic.component;
    let p = &ic.placement;
    let (cu, crtyd, silks) = match p.side {
        BoardSide::Top => ("F.Cu", "F.CrtYd", "F.SilkS"),
        BoardSide::Bottom => ("B.Cu", "B.CrtYd", "B.SilkS"),
    };
    let (x, y) = (mm_str(&p.x), mm_str(&p.y));
    // Courtyard half-extents in mm, snapped to the KiCad grid; extent (end - start) == placement size.
    let w = p.width.si_magnitude() * 1_000.0;
    let h = p.height.si_magnitude() * 1_000.0;
    let (hw, hh) = (fmt_mm(w / 2.0), fmt_mm(h / 2.0));
    let (nhw, nhh) = (fmt_mm(-w / 2.0), fmt_mm(-h / 2.0));

    let mut s = String::new();
    s.push_str(&format!(
        "  (footprint {} (layer \"{cu}\") (at {x} {y})\n",
        quote(&format!("eak:{}", c.refdes))
    ));
    s.push_str(&format!(
        "    (fp_text reference {} (at 0 0) (layer \"{silks}\"))\n",
        quote(&c.refdes)
    ));
    s.push_str(&format!(
        "    (fp_rect (start {nhw} {nhh}) (end {hw} {hh}) (layer \"{crtyd}\") (width 0.05))\n"
    ));
    for (pin, net) in ic.pins.iter().zip(ic.pin_nets.iter()) {
        // Emit the pad's `(net IDX "name")` when it was captured on a net (G1); an unconnected pad
        // (`None`) gets no net node — nothing invented. Names are quoted so any character survives.
        let net_node = match net {
            Some(idx) => format!(
                " (net {idx} {})",
                quote(net_names.get(idx).copied().unwrap_or(""))
            ),
            None => String::new(),
        };
        s.push_str(&format!(
            "    (pad {} smd rect (at 0 0) (size 1 1) (layers \"{cu}\"){net_node})\n",
            quote(&pin.designation)
        ));
    }
    s.push_str("  )\n");
    s
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

    // ------------------------------- F3: footprints -> (footprint …) -------------------------------

    /// A board carrying parts: a top 0805 resistor with an explicit courtyard rect (2.0 x 1.4 mm) and
    /// two pads, and a bottom SOIC-8 IC with no courtyard (class default 6 x 6 mm) and three pads.
    /// Exercises both courtyard paths, both sides, and multi-pad→pin fan-out across the F3 round-trip.
    const WITH_FOOTPRINTS: &str = r#"
        (kicad_pcb
          (net 0 "")
          (net 1 "GND")
          (net 2 "VCC")
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
        )
    "#;

    #[test]
    fn footprints_export_with_a_node_per_component_and_a_pad_per_pin() {
        // String-shape assertion: the emitted document carries the parts, not just copper.
        let d = import_kicad_pcb(WITH_FOOTPRINTS).unwrap();
        assert_eq!(d.components.len(), 2);
        let s = export(&d);
        assert_eq!(s.matches("(footprint").count(), d.components.len());
        let total_pins: usize = d.components.iter().map(|c| c.pins.len()).sum();
        assert_eq!(total_pins, 5); // R1 (2) + U1 (3)
        assert_eq!(s.matches("(pad").count(), total_pins);
        // the refdes rides an (fp_text reference …) — the only shape footprint_refdes reads.
        assert!(s.contains("(fp_text reference \"R1\""));
        assert!(s.contains("(fp_text reference \"U1\""));
    }

    #[test]
    fn footprints_round_trip_as_a_fixed_point() {
        // The point of F3: import a board WITH parts, export it, re-import — and the components,
        // placements, and pins land identically, ALONGSIDE the existing board/nets fixed point.
        let d1 = import_kicad_pcb(WITH_FOOTPRINTS).unwrap();
        let d2 = import_kicad_pcb(&export(&d1)).unwrap();

        assert_eq!(d1.components.len(), d2.components.len());
        for (a, b) in d1.components.iter().zip(&d2.components) {
            // refdes survives; class is re-inferred from it, so it round-trips iff the refdes does.
            assert_eq!(a.component.refdes, b.component.refdes);
            assert_eq!(a.component.class, b.component.class);
            // placement: side + centre position + courtyard size (explicit rect AND class default).
            assert_eq!(a.placement.side, b.placement.side);
            assert!(q_eq(&a.placement.x, &b.placement.x));
            assert!(q_eq(&a.placement.y, &b.placement.y));
            assert!(q_eq(&a.placement.width, &b.placement.width));
            assert!(q_eq(&a.placement.height, &b.placement.height));
            // pins: count + ordered designations + component binding all survive.
            assert_eq!(a.pins.len(), b.pins.len());
            let da: Vec<&str> = a.pins.iter().map(|p| p.designation.as_str()).collect();
            let db: Vec<&str> = b.pins.iter().map(|p| p.designation.as_str()).collect();
            assert_eq!(da, db);
            assert!(b.pins.iter().all(|p| p.component == b.component.id));
        }

        // ...and the copper/outline fixed point still holds with parts present (no F2/F3 regression).
        assert!(q_eq(&d1.board.width, &d2.board.width));
        assert!(q_eq(&d1.board.height, &d2.board.height));
        assert_eq!(d1.nets.len(), d2.nets.len());
        assert_eq!(d1.tracks.len(), d2.tracks.len());
    }

    /// A board whose two pads share net 1 (GND) plus a pad on net 2 (VCC) and an unconnected pad —
    /// so the export round-trip exercises captured membership AND the no-net (unconnected) pad.
    const WITH_PAD_NETS: &str = r#"
        (kicad_pcb
          (net 0 "")
          (net 1 "GND")
          (net 2 "VCC")
          (footprint "Resistor_SMD:R_0805" (layer "F.Cu") (at 20 30)
            (fp_text reference "R1" (at 0 -2) (layer "F.SilkS"))
            (pad "1" smd rect (at -0.9 0) (size 1 1) (layers "F.Cu") (net 1 "GND"))
            (pad "2" smd rect (at 0.9 0) (size 1 1) (layers "F.Cu") (net 2 "VCC"))
          )
          (footprint "Connector:Conn_01x02" (layer "F.Cu") (at 5 5)
            (fp_text reference "J1" (at 0 -2) (layer "F.SilkS"))
            (pad "1" thru_hole circle (at 0 0) (size 1.7 1.7) (layers "*.Cu") (net 1 "GND"))
            (pad "2" thru_hole circle (at 2.54 0) (size 1.7 1.7) (layers "*.Cu") (net 0 ""))
          )
        )
    "#;

    #[test]
    fn pad_nets_survive_the_export_round_trip() {
        // G1: the pad's captured `(net IDX …)` must re-emit and re-import, so pad→net membership is a
        // fixed point — not just the pins themselves.
        let d1 = import_kicad_pcb(WITH_PAD_NETS).unwrap();
        let s = export(&d1);
        // Connected pads carry a (net …) node; the unconnected J1.2 does not.
        assert!(s.contains("(net 1 \"GND\")"));
        assert!(s.contains("(net 2 \"VCC\")"));
        let d2 = import_kicad_pcb(&s).unwrap();

        assert_eq!(d1.components.len(), d2.components.len());
        for (a, b) in d1.components.iter().zip(&d2.components) {
            assert_eq!(a.pin_nets, b.pin_nets, "pad→net membership must round-trip");
        }
        // Concretely: R1 both pins on their nets, J1.1 on GND, J1.2 unconnected.
        let r1 = d2
            .components
            .iter()
            .find(|c| c.component.refdes == "R1")
            .unwrap();
        assert_eq!(r1.pin_nets, vec![Some(1), Some(2)]);
        let j1 = d2
            .components
            .iter()
            .find(|c| c.component.refdes == "J1")
            .unwrap();
        assert_eq!(j1.pin_nets, vec![Some(1), None]);
    }

    #[test]
    fn copper_only_design_emits_no_footprints() {
        // Regression guard: a parts-free board exports exactly as before F3 — no (footprint …) nodes.
        let d = import_kicad_pcb(SAMPLE).unwrap();
        assert!(d.components.is_empty());
        let s = export(&d);
        assert!(!s.contains("(footprint"));
        assert!(!s.contains("(pad"));
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
