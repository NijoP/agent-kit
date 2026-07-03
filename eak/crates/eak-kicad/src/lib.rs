//! KiCad `.kicad_pcb` import — the fuel for the hero demo's bulletproof "import → AI-review" path.
//!
//! This is an **adapter** (outermost ring): it parses a real KiCad board's copper into the plain
//! [`eak_domain`] entities the verification engine already understands ([`Net`]s, [`Track`]s, a
//! [`Board`]), so an existing design can be reviewed by the same deterministic rule set that a
//! generated one is (`drc-trace-width`, `drc-copper-clearance`, `drc-ampacity-width`, …). It maps
//! nothing it does not understand and invents nothing it cannot see — an unstated field simply is
//! not carried (P4/P9 discipline), so the review is honest about what the board actually declares.
//!
//! Scope (MVP): the copper-realization subset — `(net …)` and `(segment …)` nodes plus a derived
//! outline. Footprints, vias, zones, and arcs are recognised-but-skipped; richer coverage is a
//! later increment. See `project-plans/03-roadmap.md` (Hero Flow) and `07-engineering-backlog.md`.

use eak_domain::{Board, BoardSide, EntityId, LayerStack, Net, NetClass, NetOrigin, Track};
use eak_units::{PhysicalQuantity, Unit};
use std::collections::HashSet;

mod export;
pub use export::export;

/// A design imported from a `.kicad_pcb` file: the outline plus the copper the review rules read,
/// alongside any non-fatal [`ImportWarning`]s recorded while skipping content that could not be
/// resolved (e.g. a segment on an undeclared net). A well-formed board imports with `warnings` empty.
#[derive(Debug, Clone)]
pub struct ImportedDesign {
    pub board: Board,
    pub nets: Vec<Net>,
    pub tracks: Vec<Track>,
    /// Non-fatal problems recorded during import: content that could not be carried was skipped, but
    /// the skip is surfaced here rather than hidden (honesty), so the caller can report e.g.
    /// "N segments skipped: unknown net" without aborting the whole import.
    pub warnings: Vec<ImportWarning>,
}

/// A non-fatal problem recorded during import: the affected content is skipped, but the skip is
/// surfaced rather than silently dropped, so the review stays honest about what it did not carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportWarning {
    /// A `(segment …)` referenced a net id with no reviewable `(net …)` declaration — an undeclared
    /// index, or net 0 / an empty-name net (which the importer drops). The track could not be
    /// resolved to a committed net, so it was skipped rather than emitting a phantom-net track that
    /// `RouteNet` would reject downstream, aborting the entire import.
    SegmentUnknownNet { net_id: u128 },
}

impl std::fmt::Display for ImportWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportWarning::SegmentUnknownNet { net_id } => {
                write!(f, "segment skipped: references unknown net {net_id}")
            }
        }
    }
}

/// Why an import could not produce a well-formed design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// The input was empty or contained no s-expression.
    Empty,
    /// Parentheses or a quoted string were left unbalanced.
    Unbalanced,
    /// The root node is not a `(kicad_pcb …)` list.
    NotKicadPcb,
    /// A recognised node was missing a required field or carried a non-numeric one.
    Malformed(&'static str),
    /// A produced entity failed a domain invariant (carries the domain message).
    Invalid(String),
    /// The s-expression nested past the safe recursion depth — corrupt or adversarial input that
    /// would otherwise overflow the parser's stack; refused cleanly instead.
    TooDeep,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::Empty => write!(f, "empty input: no s-expression to parse"),
            ImportError::Unbalanced => write!(f, "unbalanced parentheses or string in .kicad_pcb"),
            ImportError::NotKicadPcb => write!(f, "root node is not (kicad_pcb ...)"),
            ImportError::Malformed(what) => write!(f, "malformed {what}"),
            ImportError::Invalid(msg) => write!(f, "imported entity is invalid: {msg}"),
            ImportError::TooDeep => {
                write!(
                    f,
                    "s-expression nested too deeply (malformed or adversarial input)"
                )
            }
        }
    }
}
impl std::error::Error for ImportError {}

// ------------------------------- minimal s-expression parse -------------------------------

/// A parsed s-expression node: either a leaf token or a parenthesised list.
enum Sexp {
    Atom(String),
    List(Vec<Sexp>),
}

/// Maximum s-expression nesting depth. Real boards nest only a handful of levels (`kicad_pcb` →
/// `segment` → `start` is three); this bound turns adversarial deeply-nested input into a clean
/// [`ImportError::TooDeep`] instead of a stack overflow, without affecting any well-formed board.
const MAX_PARSE_DEPTH: usize = 256;

fn parse_node(
    it: &mut std::iter::Peekable<std::str::Chars>,
    depth: usize,
) -> Result<Sexp, ImportError> {
    if depth > MAX_PARSE_DEPTH {
        return Err(ImportError::TooDeep);
    }
    skip_ws(it);
    match it.peek() {
        Some('(') => {
            it.next();
            let mut list = Vec::new();
            loop {
                skip_ws(it);
                match it.peek() {
                    Some(')') => {
                        it.next();
                        return Ok(Sexp::List(list));
                    }
                    None => return Err(ImportError::Unbalanced),
                    _ => list.push(parse_node(it, depth + 1)?),
                }
            }
        }
        Some('"') => {
            it.next();
            let mut s = String::new();
            loop {
                match it.next() {
                    Some('\\') => {
                        if let Some(c) = it.next() {
                            s.push(c);
                        }
                    }
                    Some('"') => return Ok(Sexp::Atom(s)),
                    Some(c) => s.push(c),
                    None => return Err(ImportError::Unbalanced),
                }
            }
        }
        Some(_) => {
            let mut s = String::new();
            while let Some(&c) = it.peek() {
                if c.is_whitespace() || c == '(' || c == ')' {
                    break;
                }
                s.push(c);
                it.next();
            }
            Ok(Sexp::Atom(s))
        }
        None => Err(ImportError::Empty),
    }
}

fn skip_ws(it: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = it.peek() {
        if c.is_whitespace() {
            it.next();
        } else {
            break;
        }
    }
}

// ------------------------------- node helpers -------------------------------

/// The head token of a list, e.g. `"segment"` for `(segment …)`.
fn head(list: &[Sexp]) -> Option<&str> {
    match list.first() {
        Some(Sexp::Atom(a)) => Some(a.as_str()),
        _ => None,
    }
}

/// The args of the first child sub-list whose head is `key` — e.g. `sub(seg, "start")` yields the
/// `[x, y]` atoms of `(start x y)`.
fn sub<'a>(list: &'a [Sexp], key: &str) -> Option<&'a [Sexp]> {
    list.iter().find_map(|n| match n {
        Sexp::List(inner) if head(inner) == Some(key) => Some(&inner[1..]),
        _ => None,
    })
}

/// The `n`-th atom of a list as an f64.
fn atom_f64(list: &[Sexp], i: usize) -> Option<f64> {
    match list.get(i) {
        Some(Sexp::Atom(a)) => a.parse::<f64>().ok(),
        _ => None,
    }
}

/// The `n`-th atom of a list as a string slice.
fn atom_str(list: &[Sexp], i: usize) -> Option<&str> {
    match list.get(i) {
        Some(Sexp::Atom(a)) => Some(a.as_str()),
        _ => None,
    }
}

/// Classify a net by its name — the same coarse power/ground/signal split the schematic uses. A
/// heuristic on the imported name, not an electrical claim (a real class comes from the schematic).
fn classify(name: &str) -> NetClass {
    let u = name.to_uppercase();
    if u == "GND" || u == "GROUND" || u.starts_with("GND") || u.starts_with("AGND") {
        NetClass::Ground
    } else if u.starts_with('+')
        || [
            "VCC", "VDD", "VBUS", "VIN", "VOUT", "PWR", "3V3", "5V", "3.3V",
        ]
        .iter()
        .any(|p| u.contains(p))
    {
        NetClass::Power
    } else {
        NetClass::Signal
    }
}

fn mm(v: f64) -> PhysicalQuantity {
    PhysicalQuantity::new(v, Unit::Millimetre)
}

// ------------------------------- the importer -------------------------------

/// Parse a `.kicad_pcb` document into an [`ImportedDesign`]. Nets keep their KiCad id; the empty
/// net 0 (unconnected) is dropped. Each `(segment …)` becomes a [`Track`] on `F.Cu`→`Top` /
/// `B.Cu`→`Bottom`; the outline is a standard 2-layer FR-4 stack sized to the copper's bounding box
/// (a placeholder until `Edge.Cuts` parsing lands). Produced entities are domain-validated, so a
/// malformed board is a typed error, never a silently bad design.
pub fn import_kicad_pcb(src: &str) -> Result<ImportedDesign, ImportError> {
    let root = parse_node(&mut src.chars().peekable(), 0)?;
    let items = match root {
        Sexp::List(items) if head(&items) == Some("kicad_pcb") => items,
        _ => return Err(ImportError::NotKicadPcb),
    };

    let mut nets = Vec::new();
    // The ids of the nets we actually keep, so a segment can be resolved to a *committed* net. Net 0
    // / empty-name nets are dropped and thus absent here, exactly as they are absent from `nets`.
    let mut kept_net_ids: HashSet<u128> = HashSet::new();
    let mut tracks = Vec::new();
    let mut warnings = Vec::new();
    let (mut max_x, mut max_y) = (0.0_f64, 0.0_f64);

    // First pass — nets. Collected before any segment so a segment can be resolved regardless of the
    // order the two appear in the file (real boards declare nets first, but we do not depend on it).
    for item in &items {
        let Sexp::List(l) = item else { continue };
        if head(l) != Some("net") {
            continue; // footprints/vias/zones/arcs and segments are handled elsewhere.
        }
        // (net <id> "<name>")
        let id = atom_f64(l, 1).ok_or(ImportError::Malformed("net id"))? as u128;
        let name = atom_str(l, 2).unwrap_or("").to_string();
        if name.trim().is_empty() {
            continue; // net 0 / unconnected: no reviewable net
        }
        let class = classify(&name);
        let net = Net {
            id: EntityId(id),
            name,
            class,
            // Imported copper carries no parsed pins yet, so the net is member-less; it is
            // tagged `Physical` so the CreateNet seam applies the physical-origin invariant
            // (member-less is allowed) rather than rejecting it as a logical-synthesis defect.
            members: vec![],
            current: None,
            impedance_target: None,
            origin: NetOrigin::Physical,
        };
        net.validate()
            .map_err(|e| ImportError::Invalid(e.to_string()))?;
        kept_net_ids.insert(id);
        nets.push(net);
    }

    // Second pass — segments. A segment whose net id has no reviewable `(net …)` declaration cannot
    // be resolved to a committed net; emitting it would produce a phantom-net track that `RouteNet`
    // rejects downstream, aborting the ENTIRE import. Instead we skip it and record a warning, so a
    // single bad segment never sinks an otherwise reviewable board (honesty: recorded, not dropped).
    for item in &items {
        let Sexp::List(l) = item else { continue };
        if head(l) != Some("segment") {
            continue;
        }
        let start = sub(l, "start").ok_or(ImportError::Malformed("segment start"))?;
        let end = sub(l, "end").ok_or(ImportError::Malformed("segment end"))?;
        let width = sub(l, "width").and_then(|w| atom_f64(w, 0));
        let layer = sub(l, "layer")
            .and_then(|ly| atom_str(ly, 0))
            .unwrap_or("F.Cu");
        let net_id = sub(l, "net").and_then(|n| atom_f64(n, 0));

        let (x1, y1) = (
            atom_f64(start, 0).ok_or(ImportError::Malformed("segment start x"))?,
            atom_f64(start, 1).ok_or(ImportError::Malformed("segment start y"))?,
        );
        let (x2, y2) = (
            atom_f64(end, 0).ok_or(ImportError::Malformed("segment end x"))?,
            atom_f64(end, 1).ok_or(ImportError::Malformed("segment end y"))?,
        );
        let width = width.ok_or(ImportError::Malformed("segment width"))?;
        let net_id = net_id.ok_or(ImportError::Malformed("segment net"))? as u128;

        // Unresolvable copper: reference to net 0 / an undeclared / an empty-name net. Skip + record.
        if !kept_net_ids.contains(&net_id) {
            warnings.push(ImportWarning::SegmentUnknownNet { net_id });
            continue;
        }

        let side = if layer.starts_with("B.") {
            BoardSide::Bottom
        } else {
            BoardSide::Top
        };

        let track = Track {
            // Track ids are offset well clear of the small KiCad net ids they reference.
            id: EntityId(10_000 + tracks.len() as u128),
            net: EntityId(net_id),
            layer: side,
            width: mm(width),
            x1: mm(x1),
            y1: mm(y1),
            x2: mm(x2),
            y2: mm(y2),
        };
        track
            .validate()
            .map_err(|e| ImportError::Invalid(e.to_string()))?;
        tracks.push(track);
        max_x = max_x.max(x1).max(x2);
        max_y = max_y.max(y1).max(y2);
    }

    // Outline placeholder: a standard 2-layer FR-4 stack sized to the copper bounding box + margin
    // (never zero, so Board::validate's positive-dimension invariant always holds).
    let board = Board {
        id: EntityId(1),
        width: mm((max_x + 5.0).max(10.0)),
        height: mm((max_y + 5.0).max(10.0)),
        stack: LayerStack::standard_two_layer(),
    };
    board
        .validate()
        .map_err(|e| ImportError::Invalid(e.to_string()))?;

    Ok(ImportedDesign {
        board,
        nets,
        tracks,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn imports_nets_and_tracks_with_inferred_classes() {
        let d = import_kicad_pcb(SAMPLE).unwrap();
        // net 0 (empty) is dropped; GND/VCC/SDA remain.
        assert_eq!(d.nets.len(), 3);
        let by_name = |n: &str| d.nets.iter().find(|x| x.name == n).unwrap().class;
        assert_eq!(by_name("GND"), NetClass::Ground);
        assert_eq!(by_name("VCC"), NetClass::Power);
        assert_eq!(by_name("SDA"), NetClass::Signal);
        // three segments -> three tracks; the via is skipped.
        assert_eq!(d.tracks.len(), 3);
        let sda = d.tracks.iter().find(|t| t.net == EntityId(3)).unwrap();
        assert_eq!(sda.layer, BoardSide::Bottom); // B.Cu -> Bottom
        assert!((sda.width.si_magnitude() - 0.25e-3).abs() < 1e-12); // 0.25 mm
                                                                     // board sized to the copper bounding box (x up to 40) + margin.
        assert!(d.board.width.si_magnitude() > 40e-3);
    }

    #[test]
    fn well_formed_board_imports_with_zero_warnings() {
        // Regression guard: the ordinary path must stay warning-free (F1 must not perturb it).
        let d = import_kicad_pcb(SAMPLE).unwrap();
        assert!(d.warnings.is_empty(), "clean board must have no warnings");
        assert_eq!(d.tracks.len(), 3);
    }

    #[test]
    fn segment_on_unknown_net_is_skipped_and_warned_not_aborted() {
        // Finding #3: one segment references net 7, which is never declared, and one references net 0
        // (dropped). Both are unresolvable copper. The import must SUCCEED — skipping those two and
        // recording a warning each — while the valid GND track (net 1) still lands.
        let src = r#"
            (kicad_pcb
              (net 0 "")
              (net 1 "GND")
              (segment (start 0 0) (end 10 0) (width 0.2) (layer "F.Cu") (net 1))
              (segment (start 0 2) (end 10 2) (width 0.2) (layer "F.Cu") (net 7))
              (segment (start 0 4) (end 10 4) (width 0.2) (layer "F.Cu") (net 0))
            )
        "#;
        let d = import_kicad_pcb(src).unwrap();

        // Only the resolvable GND track survives.
        assert_eq!(d.tracks.len(), 1);
        assert_eq!(d.tracks[0].net, EntityId(1));

        // Both skips are recorded (honesty), naming the unresolved net ids — not silently dropped.
        assert_eq!(d.warnings.len(), 2);
        assert!(d
            .warnings
            .contains(&ImportWarning::SegmentUnknownNet { net_id: 7 }));
        assert!(d
            .warnings
            .contains(&ImportWarning::SegmentUnknownNet { net_id: 0 }));
    }

    #[test]
    fn pathologically_deep_input_is_a_clean_error_not_a_stack_overflow() {
        // Finding #4: adversarial deeply-nested input must fail with a typed error, never overflow.
        let deep = "(".repeat(100_000);
        assert!(matches!(import_kicad_pcb(&deep), Err(ImportError::TooDeep)));
    }

    #[test]
    fn malformed_input_is_a_typed_error_not_a_panic() {
        assert!(matches!(import_kicad_pcb(""), Err(ImportError::Empty)));
        assert!(matches!(
            import_kicad_pcb("(foo)"),
            Err(ImportError::NotKicadPcb)
        ));
        assert!(matches!(
            import_kicad_pcb("(kicad_pcb"),
            Err(ImportError::Unbalanced)
        ));
    }

    #[test]
    fn imported_board_feeds_the_verification_engine() {
        // The whole point of the import path: an existing KiCad board flows straight into the same
        // deterministic review the kernel runs on generated designs. Here the fab process floor is
        // 0.20 mm; the two 0.15 mm F.Cu traces are finer than it, so drc-trace-width flags them.
        use eak_domain::{Priority, Requirement, RequirementCategory, RequirementStatus};
        use eak_engines::{DrcTraceWidthRule, Rule, VerificationContext};

        let d = import_kicad_pcb(SAMPLE).unwrap();
        let fab = vec![Requirement {
            id: EntityId(900),
            statement: "min trace width".into(),
            category: RequirementCategory::Fabrication,
            priority: Priority::High,
            acceptance_criterion: "process floor".into(),
            status: RequirementStatus::Accepted,
            source: EntityId(1),
            targets: vec![mm(0.20)], // slot 0 = minimum trace width
        }];
        let ctx = VerificationContext {
            requirements: &fab,
            constraints: &[],
            components: &[],
            pins: &[],
            nets: &d.nets,
            parts: &[],
            bom_line_items: &[],
            board: Some(&d.board),
            placements: &[],
            tracks: &d.tracks,
        };
        let findings = DrcTraceWidthRule::new().evaluate(&ctx);
        // The two 0.15 mm traces are under the 0.20 mm floor; the 0.25 mm one clears it.
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|f| f.rule == DrcTraceWidthRule::ID));
    }
}
