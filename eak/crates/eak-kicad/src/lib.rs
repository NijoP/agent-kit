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
//! outline — AND, since increment F2, `(footprint …)` nodes as domain [`Component`]s + [`Placement`]s
//! (each footprint's `(pad …)`s become [`Pin`]s so the part is realizable at the seam), so an
//! imported real board carries *parts* for the canvas + review, not just copper. Vias, zones, and
//! arcs remain recognised-but-skipped. Since increment G1 each `(pad …)`'s `(net IDX …)` is captured
//! alongside the [`Pin`] it mints (see [`ImportedComponent::pin_nets`]), so an imported net can carry
//! its real pad members once the pins are committed — the seam wiring lives in `eak-cli`'s
//! `import_design`, which joins those committed pins to the net; the importer only surfaces the
//! pad→net association it can see (P4/P9 honesty). See `project-plans/03-roadmap.md` (Hero Flow) and
//! `07-engineering-backlog.md`.

use eak_domain::{
    Board, BoardSide, Component, ComponentClass, ComponentOrigin, EntityId, LayerStack, Net,
    NetClass, NetOrigin, Pin, PinElectricalType, Placement, Track,
};
use eak_units::{PhysicalQuantity, Unit};
use std::collections::HashSet;

mod export;
pub use export::export;

/// A design imported from a `.kicad_pcb` file: the outline, the copper the review rules read, and
/// (since F2) the [`components`](ImportedDesign::components) each `(footprint …)` lands as, alongside
/// any non-fatal [`ImportWarning`]s recorded while skipping content that could not be resolved (e.g.
/// a segment on an undeclared net). A well-formed board imports with `warnings` empty.
#[derive(Debug, Clone)]
pub struct ImportedDesign {
    pub board: Board,
    pub nets: Vec<Net>,
    pub tracks: Vec<Track>,
    /// The parts parsed from the board's `(footprint …)` nodes (F2), each ready to be realized +
    /// placed through the capability seam. Empty for a copper-only board — exactly as before F2.
    pub components: Vec<ImportedComponent>,
    /// Non-fatal problems recorded during import: content that could not be carried was skipped, but
    /// the skip is surfaced here rather than hidden (honesty), so the caller can report e.g.
    /// "N segments skipped: unknown net" without aborting the whole import.
    pub warnings: Vec<ImportWarning>,
}

/// One parsed `(footprint …)`: the [`Component`] it realizes, the [`Pin`]s minted from its pads, and
/// the [`Placement`] positioning it on the board. The component is tagged
/// [`ComponentOrigin::Imported`] with a null [`from_block`](Component::from_block) — an imported board
/// declares no functional decomposition, so the importer states that honestly (ADR-0017) rather than
/// fabricating a synthetic block/requirement/intent spine to satisfy the seam. The `RealizeComponent`
/// seam accepts an `Imported` component with a null block (it never fabricates intent) while still
/// requiring ≥1 pin — the pads are real, so that holds.
#[derive(Debug, Clone)]
pub struct ImportedComponent {
    pub component: Component,
    pub pins: Vec<Pin>,
    pub placement: Placement,
    /// Per-pin net membership (G1), positionally parallel to [`pins`](Self::pins): entry `i` is the
    /// KiCad net index the `i`-th pad declared via `(net IDX …)`, or `None` when the pad is
    /// unconnected (`(net 0 …)` / no `(net …)` node) or referenced a net the board never declared
    /// (skipped-and-warned, see [`ImportWarning::PadUnknownNet`]). Only *kept* net indices appear
    /// here, so a consumer wiring pin→net membership at the seam never fabricates a phantom net —
    /// the importer states only the pad→net association it can actually resolve (P4/P9). The importer
    /// itself does not mutate [`Net::members`]; that join is performed downstream (`eak-cli`
    /// `import_design`) once the pins are committed and their [`EntityId`]s are real.
    pub pin_nets: Vec<Option<u128>>,
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
    /// A `(footprint …)` carried no reference designator (`fp_text reference` / `property "Reference"`)
    /// — a [`Component`] needs a non-empty refdes ([`Component::validate`]), and a nameless part is
    /// untraceable, so it was skipped rather than committed with an invented name.
    FootprintNoReference,
    /// A `(footprint …)` carried no `(pad …)` (a fiducial, logo, or mounting mark). `RealizeComponent`
    /// rejects a pin-less component (it could never join a net and would pass ERC vacuously), and the
    /// importer will not fabricate a phantom pin, so the part was skipped and the skip surfaced here.
    FootprintNoPads { refdes: String },
    /// A `(pad …)` referenced a net index with no reviewable `(net …)` declaration — a non-zero,
    /// non-empty-name index the board never declared (G1). The pin is still minted (the pad is real
    /// copper), but it is joined to NO net rather than to a phantom one, so `CreateNet` never receives
    /// a member for a net it cannot resolve. Net 0 / a pad with no `(net …)` node is genuinely
    /// *unconnected* and is dropped silently, not surfaced here — only a dangling reference is.
    PadUnknownNet {
        refdes: String,
        pad: String,
        net_id: u128,
    },
}

impl std::fmt::Display for ImportWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportWarning::SegmentUnknownNet { net_id } => {
                write!(f, "segment skipped: references unknown net {net_id}")
            }
            ImportWarning::FootprintNoReference => {
                write!(f, "footprint skipped: no reference designator")
            }
            ImportWarning::FootprintNoPads { refdes } => {
                write!(
                    f,
                    "footprint {refdes} skipped: no pads (cannot realize a pin-less component)"
                )
            }
            ImportWarning::PadUnknownNet {
                refdes,
                pad,
                net_id,
            } => {
                write!(
                    f,
                    "pad {refdes}.{pad} references unknown net {net_id}: pin joined to no net"
                )
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

/// Every direct child sub-list whose head is `key` (e.g. all `(pad …)` of a footprint) — the
/// many-match sibling of [`sub`], which returns only the first. Non-recursive: it walks the list's
/// own children, not their descendants, so a footprint's own `(layer …)`/`(pad …)` are found while
/// the nested `(layer …)` inside each pad are not. Returns each match's full list (head included).
fn children<'a>(list: &'a [Sexp], key: &str) -> Vec<&'a [Sexp]> {
    list.iter()
        .filter_map(|n| match n {
            Sexp::List(inner) if head(inner) == Some(key) => Some(inner.as_slice()),
            _ => None,
        })
        .collect()
}

/// The reference designator of a `(footprint …)`, from either the classic `(fp_text reference "R1" …)`
/// or the newer `(property "Reference" "R1" …)` form. `None` when neither is present or the name is
/// blank — such a footprint is skipped ([`ImportWarning::FootprintNoReference`]) rather than named.
fn footprint_refdes(fp: &[Sexp]) -> Option<String> {
    let non_blank = |s: &str| (!s.trim().is_empty()).then(|| s.to_string());
    for t in children(fp, "fp_text") {
        if atom_str(t, 1) == Some("reference") {
            if let Some(r) = atom_str(t, 2).and_then(non_blank) {
                return Some(r);
            }
        }
    }
    for p in children(fp, "property") {
        if atom_str(p, 1) == Some("Reference") {
            if let Some(r) = atom_str(p, 2).and_then(non_blank) {
                return Some(r);
            }
        }
    }
    None
}

/// Infer a [`ComponentClass`] from a refdes prefix — the coarse convention every schematic follows
/// (R=resistor, C=capacitor, U/IC=integrated circuit, J/P/CN=connector, VR/REG=regulator). A
/// HEURISTIC on the imported name, not an electrical claim: the domain enum has no generic/unknown
/// class, so an unrecognised prefix (L, D, Q, Y, …) falls back to the neutral active class,
/// [`ComponentClass::Ic`]. The class only drives the default courtyard size and ERC expectations.
fn classify_refdes(refdes: &str) -> ComponentClass {
    let prefix: String = refdes
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_uppercase();
    match prefix.as_str() {
        "R" | "RN" => ComponentClass::Resistor,
        "C" => ComponentClass::Capacitor,
        "J" | "P" | "CN" | "CON" => ComponentClass::Connector,
        "VR" | "REG" => ComponentClass::Regulator,
        "U" | "IC" => ComponentClass::Ic,
        _ => ComponentClass::Ic,
    }
}

/// The default square courtyard edge (mm) for a class — mirrors `eak-phases`'
/// `component_placement::courtyard_mm` so an imported part occupies the same footprint a synthesised
/// one of the same class would. Used only when the footprint declares no explicit courtyard rect.
fn courtyard_default_mm(class: ComponentClass) -> f64 {
    match class {
        ComponentClass::Connector => 9.0,
        ComponentClass::Regulator | ComponentClass::Ic => 6.0,
        ComponentClass::Resistor | ComponentClass::Capacitor => 3.0,
    }
}

/// The `(width, height)` mm courtyard of a footprint: the bounding box of its `(fp_rect …)` on a
/// courtyard layer (`*.CrtYd`) when one is declared, else the class-sized default square
/// ([`courtyard_default_mm`]). A real courtyard is honoured; otherwise a sane, documented default is
/// used rather than inventing a precise extent the file never stated.
fn footprint_courtyard(fp: &[Sexp], class: ComponentClass) -> (f64, f64) {
    for r in children(fp, "fp_rect") {
        let on_courtyard = sub(r, "layer")
            .and_then(|ly| atom_str(ly, 0))
            .is_some_and(|l| l.contains("CrtYd"));
        if !on_courtyard {
            continue;
        }
        if let (Some(s), Some(e)) = (sub(r, "start"), sub(r, "end")) {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                atom_f64(s, 0),
                atom_f64(s, 1),
                atom_f64(e, 0),
                atom_f64(e, 1),
            ) {
                let (w, h) = ((x2 - x1).abs(), (y2 - y1).abs());
                if w > 0.0 && h > 0.0 {
                    return (w, h);
                }
            }
        }
    }
    let edge = courtyard_default_mm(class);
    (edge, edge)
}

fn mm(v: f64) -> PhysicalQuantity {
    PhysicalQuantity::new(v, Unit::Millimetre)
}

// ------------------------------- the importer -------------------------------

/// Parse a `.kicad_pcb` document into an [`ImportedDesign`]. Nets keep their KiCad id; the empty
/// net 0 (unconnected) is dropped. Each `(segment …)` becomes a [`Track`] on `F.Cu`→`Top` /
/// `B.Cu`→`Bottom`; each `(footprint …)` becomes an [`ImportedComponent`] (refdes → [`Component`],
/// pads → [`Pin`]s, `(at …)`+layer → [`Placement`]); the outline is a standard 2-layer FR-4 stack
/// sized to the copper + footprint bounding box (a placeholder until `Edge.Cuts` parsing lands).
/// Produced entities are domain-validated, so a malformed board is a typed error, never a silently
/// bad design.
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

    // Third pass — footprints (F2). Each `(footprint …)` lands as a Component, its pads as Pins (so
    // the part is realizable at the seam), and a Placement at its `(at …)` on its `(layer …)` side.
    // A footprint with no reference designator or no pads cannot become a valid, realizable component,
    // so it is skipped and the skip surfaced (honesty), never committed with invented data. Ids are
    // deterministic offsets clear of the board (1), the small net ids, and the tracks (10_000+i):
    // components at 20_000+i, pins at 30_000+running, placements at 40_000+i.
    let mut components: Vec<ImportedComponent> = Vec::new();
    let mut pin_counter: u128 = 0;
    for item in &items {
        let Sexp::List(l) = item else { continue };
        if head(l) != Some("footprint") && head(l) != Some("module") {
            continue;
        }
        // Side from the footprint's own top-level layer (F.*→Top, B.*→Bottom), mirroring segments.
        let fp_layer = sub(l, "layer")
            .and_then(|ly| atom_str(ly, 0))
            .unwrap_or("F.Cu");
        let side = if fp_layer.starts_with("B.") {
            BoardSide::Bottom
        } else {
            BoardSide::Top
        };
        // Placement centre — `(at x y [rot])`; rotation is not modelled by `Placement`, so ignored.
        let at = sub(l, "at").ok_or(ImportError::Malformed("footprint at"))?;
        let x = atom_f64(at, 0).ok_or(ImportError::Malformed("footprint at x"))?;
        let y = atom_f64(at, 1).ok_or(ImportError::Malformed("footprint at y"))?;

        // A component needs a non-empty refdes and >=1 pin to be realizable; skip + warn otherwise.
        let Some(refdes) = footprint_refdes(l) else {
            warnings.push(ImportWarning::FootprintNoReference);
            continue;
        };
        let class = classify_refdes(&refdes);
        let pads = children(l, "pad");
        if pads.is_empty() {
            warnings.push(ImportWarning::FootprintNoPads { refdes });
            continue;
        }

        let component_id = EntityId(20_000 + components.len() as u128);
        // Pads → pins. A Pin carries no geometry, only an addressed terminal; the pad name is its
        // designation (falling back to its 1-based index when a pad is unnamed). Electrical type is
        // Passive — a PCB pad states no schematic role, so we do not over-claim one. Each pad's
        // `(net IDX …)` is captured into `pin_nets` (G1) so the pin can join that net's membership
        // downstream at the seam; the importer only records the association it can resolve.
        let mut pins: Vec<Pin> = Vec::with_capacity(pads.len());
        let mut pin_nets: Vec<Option<u128>> = Vec::with_capacity(pads.len());
        for (pi, pad) in pads.iter().enumerate() {
            let designation = atom_str(pad, 1)
                .map(str::to_string)
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| (pi + 1).to_string());
            // The pad's net: `(net IDX "name")`. Only a *kept* (declared, non-empty-name) index is
            // carried as a join target — so downstream membership never references a phantom net. A
            // pad on net 0 / with no `(net …)` node is unconnected (join to nothing, silent); a pad
            // on a non-zero index the board never declared is a dangling reference (join to nothing,
            // surfaced as a warning), never a fabricated net.
            let pad_net = sub(pad, "net")
                .and_then(|n| atom_f64(n, 0))
                .map(|v| v as u128);
            let joined = match pad_net {
                Some(idx) if kept_net_ids.contains(&idx) => Some(idx),
                Some(idx) if idx != 0 => {
                    warnings.push(ImportWarning::PadUnknownNet {
                        refdes: refdes.clone(),
                        pad: designation.clone(),
                        net_id: idx,
                    });
                    None
                }
                _ => None,
            };
            pins.push(Pin {
                id: EntityId(30_000 + pin_counter),
                component: component_id,
                designation,
                electrical_type: PinElectricalType::Passive,
            });
            pin_nets.push(joined);
            pin_counter += 1;
        }

        let (cw, ch) = footprint_courtyard(l, class);
        // Grow the outline to enclose the placed courtyard (centre ± half-extent) so a placed part
        // never falls outside the bbox-derived board and trips placement DRC spuriously.
        max_x = max_x.max(x + cw / 2.0);
        max_y = max_y.max(y + ch / 2.0);

        let component = Component {
            id: component_id,
            refdes,
            class,
            value: None,
            // An imported board declares no functional block, so `from_block` is honestly NULL and the
            // component is tagged `Imported`; the RealizeComponent seam accepts that without
            // fabricating an intent spine (ADR-0017).
            from_block: EntityId::NULL,
            origin: ComponentOrigin::Imported,
        };
        component
            .validate()
            .map_err(|e| ImportError::Invalid(e.to_string()))?;
        let placement = Placement {
            id: EntityId(40_000 + components.len() as u128),
            component: component_id,
            x: mm(x),
            y: mm(y),
            width: mm(cw),
            height: mm(ch),
            side,
        };
        placement
            .validate()
            .map_err(|e| ImportError::Invalid(e.to_string()))?;
        components.push(ImportedComponent {
            component,
            pins,
            placement,
            pin_nets,
        });
    }

    // Outline placeholder: a standard 2-layer FR-4 stack sized to the copper + footprint bounding box
    // + margin (never zero, so Board::validate's positive-dimension invariant always holds).
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
        components,
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
            power_domains: &[],
            clock_domains: &[],
        };
        let findings = DrcTraceWidthRule::new().evaluate(&ctx);
        // The two 0.15 mm traces are under the 0.20 mm floor; the 0.25 mm one clears it.
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|f| f.rule == DrcTraceWidthRule::ID));
    }

    // ------------------------------- F2: footprints -> components -------------------------------

    /// A board with three real footprints (an 0805 resistor on top with an explicit courtyard rect,
    /// a SOIC-8 IC on the bottom, a thru-hole connector named via `property "Reference"`) plus a
    /// pad-less fiducial the importer must skip. Exercises class inference, side, position, both
    /// courtyard paths (explicit + class default), pads→pins, and the no-pads skip.
    const WITH_FOOTPRINTS: &str = r#"
        (kicad_pcb
          (net 0 "")
          (net 1 "GND")
          (net 2 "VCC")
          (footprint "Resistor_SMD:R_0805" (layer "F.Cu") (at 20 30 90)
            (fp_text reference "R1" (at 0 -2) (layer "F.SilkS"))
            (fp_text value "10k" (at 0 2) (layer "F.Fab"))
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
            (property "Reference" "J1" (at 0 -2) (layer "F.SilkS"))
            (pad "1" thru_hole circle (at 0 0) (size 1.7 1.7) (layers "*.Cu"))
            (pad "2" thru_hole circle (at 2.54 0) (size 1.7 1.7) (layers "*.Cu"))
          )
          (footprint "Fiducial" (layer "F.Cu") (at 1 1)
            (fp_text reference "FID1" (at 0 -1) (layer "F.SilkS"))
          )
        )
    "#;

    fn comp<'a>(d: &'a ImportedDesign, refdes: &str) -> &'a ImportedComponent {
        d.components
            .iter()
            .find(|c| c.component.refdes == refdes)
            .unwrap_or_else(|| panic!("no imported component {refdes}"))
    }

    fn near(q: &PhysicalQuantity, mm_val: f64) -> bool {
        (q.si_magnitude() - mm_val * 1e-3).abs() < 1e-9
    }

    #[test]
    fn footprints_import_as_components_with_inferred_class_side_and_position() {
        let d = import_kicad_pcb(WITH_FOOTPRINTS).unwrap();
        // Three realizable footprints; the pad-less fiducial is skipped.
        assert_eq!(d.components.len(), 3);

        let r1 = comp(&d, "R1");
        assert_eq!(r1.component.class, ComponentClass::Resistor); // R prefix
        assert_eq!(r1.placement.side, BoardSide::Top); // F.Cu
        assert!(near(&r1.placement.x, 20.0) && near(&r1.placement.y, 30.0));

        let u1 = comp(&d, "U1");
        assert_eq!(u1.component.class, ComponentClass::Ic); // U prefix
        assert_eq!(u1.placement.side, BoardSide::Bottom); // B.Cu
        assert!(near(&u1.placement.x, 50.0) && near(&u1.placement.y, 40.0));

        let j1 = comp(&d, "J1");
        assert_eq!(j1.component.class, ComponentClass::Connector); // J prefix, via property
        assert_eq!(j1.placement.side, BoardSide::Top);
    }

    #[test]
    fn footprint_courtyard_uses_the_rect_when_present_else_the_class_default() {
        let d = import_kicad_pcb(WITH_FOOTPRINTS).unwrap();
        // R1 declares an explicit F.CrtYd rect: (-1,-0.7)..(1,0.7) -> 2.0 x 1.4 mm.
        let r1 = comp(&d, "R1");
        assert!(near(&r1.placement.width, 2.0) && near(&r1.placement.height, 1.4));
        // U1 (Ic) has no courtyard rect -> class default 6 x 6 mm square.
        let u1 = comp(&d, "U1");
        assert!(near(&u1.placement.width, 6.0) && near(&u1.placement.height, 6.0));
        // J1 (Connector) default 9 x 9 mm square.
        let j1 = comp(&d, "J1");
        assert!(near(&j1.placement.width, 9.0) && near(&j1.placement.height, 9.0));
    }

    #[test]
    fn footprint_pads_become_pins_bound_to_their_component() {
        let d = import_kicad_pcb(WITH_FOOTPRINTS).unwrap();
        let r1 = comp(&d, "R1");
        assert_eq!(r1.pins.len(), 2);
        assert!(r1.pins.iter().all(|p| p.component == r1.component.id));
        assert!(r1
            .pins
            .iter()
            .all(|p| p.electrical_type == PinElectricalType::Passive));
        let names: Vec<&str> = r1.pins.iter().map(|p| p.designation.as_str()).collect();
        assert_eq!(names, vec!["1", "2"]); // pad numbers become pin designations
                                           // U1 has three pads -> three pins.
        assert_eq!(comp(&d, "U1").pins.len(), 3);
        // Pin ids are unique across the whole import (offset scheme has no collisions).
        let mut ids: Vec<_> = d
            .components
            .iter()
            .flat_map(|c| c.pins.iter().map(|p| p.id))
            .collect();
        let n = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), n, "all pin ids distinct");
    }

    #[test]
    fn padless_footprint_is_skipped_and_warned_not_committed() {
        let d = import_kicad_pcb(WITH_FOOTPRINTS).unwrap();
        // FID1 has no pads -> not a component, and the skip is surfaced (honesty).
        assert!(d.components.iter().all(|c| c.component.refdes != "FID1"));
        assert!(d.warnings.contains(&ImportWarning::FootprintNoPads {
            refdes: "FID1".into()
        }));
    }

    #[test]
    fn copper_only_board_has_no_components() {
        // Regression guard: a board with no footprints imports parts-free, exactly as before F2.
        let d = import_kicad_pcb(SAMPLE).unwrap();
        assert!(d.components.is_empty());
        assert_eq!(d.tracks.len(), 3);
        assert!(d.warnings.is_empty());
    }

    // ------------------------------- G1: pad -> net capture -------------------------------

    /// A board where R1's two pads share net 1 (GND) and R2's two pads sit on net 2 (VCC) and an
    /// undeclared net 9, plus an unconnected pad (net 0). Exercises shared membership, the unknown-net
    /// warning, and the silent unconnected case.
    const WITH_PAD_NETS: &str = r#"
        (kicad_pcb
          (net 0 "")
          (net 1 "GND")
          (net 2 "VCC")
          (footprint "Resistor_SMD:R_0805" (layer "F.Cu") (at 20 30)
            (fp_text reference "R1" (at 0 -2) (layer "F.SilkS"))
            (pad "1" smd roundrect (at -0.9 0) (size 1 1.2) (layers "F.Cu") (net 1 "GND"))
            (pad "2" smd roundrect (at 0.9 0) (size 1 1.2) (layers "F.Cu") (net 1 "GND"))
          )
          (footprint "Resistor_SMD:R_0805" (layer "F.Cu") (at 30 30)
            (fp_text reference "R2" (at 0 -2) (layer "F.SilkS"))
            (pad "1" smd roundrect (at -0.9 0) (size 1 1.2) (layers "F.Cu") (net 2 "VCC"))
            (pad "2" smd roundrect (at 0.9 0) (size 1 1.2) (layers "F.Cu") (net 9 "MYSTERY"))
          )
          (footprint "Connector:Conn_01x01" (layer "F.Cu") (at 5 5)
            (fp_text reference "J1" (at 0 -2) (layer "F.SilkS"))
            (pad "1" thru_hole circle (at 0 0) (size 1.7 1.7) (layers "*.Cu") (net 0 ""))
          )
        )
    "#;

    #[test]
    fn pad_net_index_is_captured_per_pin_for_kept_nets() {
        let d = import_kicad_pcb(WITH_PAD_NETS).unwrap();
        // R1: both pads on net 1 -> both pin_nets = Some(1), parallel to pins.
        let r1 = comp(&d, "R1");
        assert_eq!(r1.pin_nets.len(), r1.pins.len());
        assert_eq!(r1.pin_nets, vec![Some(1), Some(1)]);
        // R2: pad 1 on kept net 2, pad 2 on undeclared net 9 -> Some(2), None.
        let r2 = comp(&d, "R2");
        assert_eq!(r2.pin_nets, vec![Some(2), None]);
    }

    #[test]
    fn pad_on_undeclared_net_is_warned_and_joined_to_nothing() {
        let d = import_kicad_pcb(WITH_PAD_NETS).unwrap();
        // The dangling net-9 reference is surfaced (honesty) but the pin is still minted, joined to no
        // net — no phantom net is fabricated.
        assert!(d.warnings.contains(&ImportWarning::PadUnknownNet {
            refdes: "R2".into(),
            pad: "2".into(),
            net_id: 9,
        }));
        // The unconnected pad (net 0) is silent — genuinely unconnected, not a dangling reference.
        assert!(!d
            .warnings
            .iter()
            .any(|w| matches!(w, ImportWarning::PadUnknownNet { net_id: 0, .. })));
        let j1 = comp(&d, "J1");
        assert_eq!(j1.pin_nets, vec![None]);
    }

    #[test]
    fn pads_without_net_nodes_capture_no_membership() {
        // Regression: the F2 fixture declares no pad nets, so every pin_nets entry is None — an
        // imported net stays member-less exactly as before G1 (no fabricated membership).
        let d = import_kicad_pcb(WITH_FOOTPRINTS).unwrap();
        assert!(d
            .components
            .iter()
            .all(|c| c.pin_nets.iter().all(Option::is_none)));
        assert!(d
            .warnings
            .iter()
            .all(|w| !matches!(w, ImportWarning::PadUnknownNet { .. })));
    }
}
