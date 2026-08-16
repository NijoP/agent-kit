//! Engineering domain model — the Phase-1 entity subset (Entities ring).
//!
//! Phase 1 (Requirement Planning) needs exactly five entities plus one first-class
//! relationship: [`DesignIntent`], [`Requirement`], [`Decision`], [`Evidence`], and
//! [`ProvenanceLink`]. Downstream entities (Component, Net, Constraint, ...) are NOT
//! modelled in Phase 1. See `docs/foundation/engineering-domain-model.md`.

use eak_units::{Dimension, PhysicalQuantity, Unit};
use serde::{Deserialize, Serialize};

/// Opaque, immutable identity (domain-model modelling principle 1). Carries no meaning;
/// referenced by value, never by name or position. `EntityId(0)` is reserved as the null
/// sentinel and is never minted by the runtime's id source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u128);

impl EntityId {
    pub const NULL: EntityId = EntityId(0);

    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    pub fn to_hex(self) -> String {
        format!("{:032x}", self.0)
    }

    /// Short 8-hex-digit form for human-facing traces.
    pub fn short(self) -> String {
        format!("{:08x}", self.0 as u32)
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequirementCategory {
    Functional,
    Electrical,
    Mechanical,
    Thermal,
    Regulatory,
    /// Fabrication/assembly *process* limits — what the chosen fab and assembly flow can build
    /// (minimum trace width, drill sizes, layer count, panelization). Distinct from `Regulatory`
    /// (external standards/compliance): a process floor is a property of the manufacturer, not a
    /// regulation. The trace-width DRC floor and future fab/process rules read from this category.
    Fabrication,
    Cost,
    Schedule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequirementStatus {
    Proposed,
    Accepted,
    Satisfied,
    Violated,
    Waived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    DerivedFrom,
    JustifiedBy,
    BasedOnReasoning,
    Supports,
    TracesTo,
    Supersedes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceKind {
    DesignIntentSource,
    StandardClause,
    PriorDesign,
    DatasheetParameter,
    ReviewNote,
}

/// The originating goal, preserved verbatim and as a structured summary. Never deleted,
/// only refined (domain-model entity lifecycle).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignIntent {
    pub id: EntityId,
    pub statement: String,
    pub structured_summary: String,
    pub source: String,
}

/// A single testable statement the design must satisfy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Requirement {
    pub id: EntityId,
    pub statement: String,
    pub category: RequirementCategory,
    pub priority: Priority,
    pub acceptance_criterion: String,
    pub status: RequirementStatus,
    /// The DesignIntent (or external standard entity) this requirement is rooted in.
    pub source: EntityId,
    /// Typed physical targets within the requirement (P9).
    pub targets: Vec<PhysicalQuantity>,
}

/// The justification for a design-significant change (P5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub id: EntityId,
    pub subject: EntityId,
    pub rationale: String,
    pub decider: String,
    /// Sequence number of the recorded reasoning call this decision relied on, if any.
    pub reasoning_call_seq: Option<u64>,
    pub evidence: Vec<EntityId>,
    pub confidence: f64,
}

/// A fact supporting a [`Decision`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub id: EntityId,
    pub kind: EvidenceKind,
    pub content_reference: String,
    pub source: String,
    pub reliability: f64,
}

/// A first-class, addressed relationship ("X relation Y") — the edges of the
/// provenance graph (shared-state-model.md identity rule 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceLink {
    pub id: EntityId,
    pub from: EntityId,
    pub to: EntityId,
    pub relation: RelationType,
}

// ===================== Phase 2: verification entities =====================
//
// Phase 2 adds the machine-checkable layer on top of the Phase-1 intent layer: a
// [`Constraint`] is a typed bound derived from a [`Requirement`]'s physical target; a
// [`Violation`] is a first-class, addressed breach of a verification rule; a [`Waiver`]
// is the recorded decision to accept a violation. See
// `docs/engineering/constraint-engine.md` and `docs/engineering/verification-engine.md`.

/// Comparison sense of a [`Constraint`]'s bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConstraintKind {
    /// The subject value must not exceed the bound (e.g. "power <= 5 W").
    Max,
    /// The subject value must be at least the bound (e.g. "power >= 8 W").
    Min,
    /// The subject value must equal the bound.
    Equal,
}

/// Lifecycle of a [`Constraint`]. Constraints are never deleted, only superseded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConstraintStatus {
    Active,
    Superseded,
}

/// A machine-checkable bound on a [`Requirement`]'s physical target (P9). Derived from a
/// requirement, never authored directly; carries the unit so verification is dimensionally
/// unambiguous.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub id: EntityId,
    pub statement: String,
    /// The requirement this constraint bounds.
    pub subject_requirement: EntityId,
    pub kind: ConstraintKind,
    pub bound: PhysicalQuantity,
    /// The entity (usually the subject requirement) this constraint derives from.
    pub source: EntityId,
    pub status: ConstraintStatus,
}

impl Constraint {
    pub fn is_active(&self) -> bool {
        self.status == ConstraintStatus::Active
    }

    /// Domain invariant: a constraint carries a non-empty statement (reuses
    /// [`DomainError::EmptyStatement`]). The null-subject check lives in the capability
    /// handler, so no new error variant is needed.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.statement.trim().is_empty() {
            return Err(DomainError::EmptyStatement);
        }
        Ok(())
    }
}

/// How serious a [`Violation`] is. Only [`ViolationSeverity::Error`] can block a workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

/// Lifecycle of a [`Violation`]. An [`Open`](ViolationStatus::Open) error blocks; a
/// [`Waived`](ViolationStatus::Waived) or [`Resolved`](ViolationStatus::Resolved) one does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationStatus {
    Open,
    Waived,
    Resolved,
}

/// A detected breach of a verification rule, made first-class so it is addressed and fully
/// traceable to its cause via [`Violation::subjects`] + provenance links.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Violation {
    pub id: EntityId,
    /// Stable identifier of the rule that raised it (e.g. `"constraint-consistency"`).
    pub rule: String,
    pub severity: ViolationSeverity,
    /// The entities implicated (constraints, requirements, ...). The traceability anchor.
    pub subjects: Vec<EntityId>,
    pub message: String,
    pub status: ViolationStatus,
}

impl Violation {
    /// A violation blocks a workflow iff it is an unaddressed error (P13: failures are
    /// explicit, never silently dropped).
    pub fn is_blocking(&self) -> bool {
        self.severity == ViolationSeverity::Error && self.status == ViolationStatus::Open
    }
}

/// A recorded decision to accept a [`Violation`] rather than fix it (P5: every
/// design-significant change is justified; P10: the human/agent who decided is named).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Waiver {
    pub id: EntityId,
    pub violation: EntityId,
    pub justification: String,
    pub decided_by: String,
}

// ===================== Phase 3: schematic entities =====================
//
// Phase 3 adds the realization layer beneath the Phase-2 verification layer: a
// [`FunctionalBlock`] groups the requirements it realizes; a [`Component`] is a concrete
// part minted from a block; a [`Pin`] is an addressed terminal of a component; a [`Net`]
// is a first-class electrical connection between pins. Each entity carries the upstream
// trace it derives from (P3), so the schematic stays explainable back to intent. See
// `docs/engineering/schematic-model.md`.

/// A unit of design intent realized as part of the architecture: it names a function and
/// the requirements (P3) it is responsible for satisfying. Components are minted from a block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionalBlock {
    pub id: EntityId,
    pub name: String,
    pub function: String,
    /// The requirements this block is responsible for realizing.
    pub requirements: Vec<EntityId>,
}

impl FunctionalBlock {
    /// Domain invariant: a block carries a non-empty name. Requirement-link integrity
    /// (each id exists) is re-checked at the capability seam (P3).
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::EmptyField("functional block name"));
        }
        Ok(())
    }
}

/// The coarse kind of a [`Component`]. Drives ERC expectations (e.g. a regulator is a
/// power source, a connector may be a sink).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentClass {
    Connector,
    Regulator,
    Ic,
    Resistor,
    Capacitor,
}

/// The electrical role of a [`Pin`]. Drives ERC drive/sink analysis (P9): a power net must
/// have a driver ([`PowerOut`](PinElectricalType::PowerOut)); two outputs on one net contend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PinElectricalType {
    PowerIn,
    PowerOut,
    Input,
    Output,
    Bidirectional,
    Passive,
    Ground,
    NoConnect,
}

/// The electrical class of a [`Net`]. Drives ERC: power and ground nets demand a driver,
/// signal nets are checked for contention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetClass {
    Power,
    Ground,
    Signal,
}

/// Where a [`Component`] came from — the discriminator that lets the capability seam apply the right
/// traceability invariant to each, mirroring [`NetOrigin`]. A `Synthesized` component is *minted*
/// from a committed [`FunctionalBlock`] (Schematic Planning realizes it from intent), so it MUST
/// carry a non-null [`from_block`](Component::from_block) referencing a committed block, or it is a
/// synthesis defect. An `Imported` component is parsed from a finished board's `(footprint …)`, which
/// declares no functional decomposition, so it MAY carry a null `from_block` — the seam must never
/// fabricate a block to satisfy the synthesis rule (that would invent design intent the board never
/// stated, poisoning traceability). Old event logs predate this field, so [`Component::origin`] is
/// `#[serde(default)]` and defaults to `Synthesized` (their historical meaning). A static, fieldless
/// enum: it folds identically on replay (P4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ComponentOrigin {
    /// A component minted from a committed [`FunctionalBlock`] (the schematic/synthesis path).
    /// Requires a non-null `from_block` referencing a committed block. The default, so pre-existing
    /// event logs (which never carried an origin) deserialize here.
    #[default]
    Synthesized,
    /// A component parsed from a finished board's footprint, which declares no functional block. May
    /// carry a null `from_block`; any block it *does* reference must still be committed (no phantom).
    Imported,
}

/// A concrete part realizing some of a [`FunctionalBlock`]'s function. A `Synthesized` component is
/// minted from a block, so it is traceable back to the intent it serves (P3); an `Imported` one is
/// parsed from a finished board and declares no upstream block — its [`origin`](Component::origin)
/// and null [`from_block`](Component::from_block) say so honestly rather than fabricating intent. The
/// optional `value` is a typed physical quantity (e.g. a resistor's 10 kΩ), hence `Component` is not
/// `Eq`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub id: EntityId,
    pub refdes: String,
    pub class: ComponentClass,
    pub value: Option<PhysicalQuantity>,
    /// The functional block this component was realized from — [`EntityId::NULL`] for an
    /// [`Imported`](ComponentOrigin::Imported) component, which declares no upstream block.
    pub from_block: EntityId,
    /// Whether this component was synthesised (`Synthesized`, minted from a committed block) or
    /// imported from a finished board's footprint (`Imported`, may carry a null `from_block`). Drives
    /// the traceability invariant the capability seam applies (see [`ComponentOrigin`]).
    /// `#[serde(default)]` so pre-existing logs — which never carried an origin — deserialize as
    /// `Synthesized`, their original meaning (P4 replay-stability).
    #[serde(default)]
    pub origin: ComponentOrigin,
}

impl Component {
    /// Domain invariant: a component carries a non-empty reference designator. The
    /// block-link existence check lives at the capability seam (P3).
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.refdes.trim().is_empty() {
            return Err(DomainError::EmptyField("component reference designator"));
        }
        Ok(())
    }
}

/// An addressed terminal of a [`Component`]. Referenced by id from [`Net::members`], never
/// by position, so connectivity survives renumbering (domain-model identity rule).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pin {
    pub id: EntityId,
    pub component: EntityId,
    pub designation: String,
    pub electrical_type: PinElectricalType,
}

/// Where a [`Net`] came from — the discriminator that lets the capability seam apply the right
/// connectivity invariant to each. A `Logical` net is *synthesised* (Schematic Planning joins
/// pins), so it must join ≥1 pin or it is a synthesis defect. A `Physical` net is *imported* from a
/// finished board's copper, which carries no parsed pins yet, so it may legitimately be member-less
/// — the seam must never fabricate a pin to satisfy the logical rule. Old event logs predate this
/// field, so [`Net::origin`] is `#[serde(default)]` and defaults to `Logical` (their historical
/// meaning). A static, fieldless enum: it folds identically on replay (P4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NetOrigin {
    /// Synthesised connectivity that joins pins (the schematic/synthesis path). Requires ≥1 member.
    /// The default, so pre-existing event logs (which never carried an origin) deserialize here.
    #[default]
    Logical,
    /// Imported copper from a finished board, which carries no parsed pins. May be member-less.
    Physical,
}

/// A first-class electrical connection joining a set of [`Pin`]s. Made addressable so ERC
/// findings can name the offending net and trace it (P3, P13). The optional `current` is the
/// net's worst-case DC load — physically the KCL sum of its consumers (see
/// `engineering-science/electrical/kirchhoff-laws.md`) — a typed Current [`PhysicalQuantity`]
/// (P9). It is the per-net input to the ampacity trace-width floor (`drc-ampacity-width`): a net
/// that states no current gets no DC sizing floor, so that check stays silent rather than
/// inventing one. Carrying an optional `PhysicalQuantity` makes `Net` not `Eq` (exactly like
/// [`Component`]/[`Board`]/[`Track`]); nets are addressed by [`EntityId`], never by equality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Net {
    pub id: EntityId,
    pub name: String,
    pub class: NetClass,
    /// The pin ids joined by this net.
    pub members: Vec<EntityId>,
    /// The net's worst-case DC current, if known (a typed Current quantity). `None` = unstated,
    /// in which case the ampacity width floor is not computed for this net.
    pub current: Option<PhysicalQuantity>,
    /// The net's controlled characteristic-impedance target, if any — a typed Resistance/Ω
    /// quantity (e.g. 50 Ω single-ended). `None` = uncontrolled, so Routing keeps the per-class
    /// default width and the impedance-match check stays silent. When `Some`, Routing sizes the
    /// trace to the stack-up-derived microstrip width and `drc-impedance-match` confirms the
    /// realized Z₀ is within tolerance (see `engineering-science/electrical/transmission-lines.md`).
    pub impedance_target: Option<PhysicalQuantity>,
    /// Whether this net was synthesised (`Logical`, joins pins) or imported from a physical board's
    /// copper (`Physical`, may be member-less). Drives the connectivity invariant the capability
    /// seam applies (see [`NetOrigin`]). `#[serde(default)]` so pre-existing logs — which never
    /// carried an origin — deserialize as `Logical`, their original meaning (P4 replay-stability).
    #[serde(default)]
    pub origin: NetOrigin,
}

impl Net {
    /// Domain invariant: a net carries a non-empty name; if a `current` is stated it is a finite,
    /// positive Current quantity (a net cannot carry a negative or dimensionally-wrong load); and
    /// if an `impedance_target` is stated it is a finite, positive Resistance (Ω) quantity (P9).
    /// Member-pin integrity (each id exists) is re-checked at the capability seam (P3).
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::EmptyField("net name"));
        }
        if let Some(current) = &self.current {
            if current.dimension() != Dimension::Current
                || !current.si_magnitude().is_finite()
                || current.si_magnitude() <= 0.0
            {
                return Err(DomainError::Inconsistent(
                    "net current must be a finite, positive Current quantity",
                ));
            }
        }
        if let Some(z0) = &self.impedance_target {
            if z0.dimension() != Dimension::Resistance
                || !z0.si_magnitude().is_finite()
                || z0.si_magnitude() <= 0.0
            {
                return Err(DomainError::Inconsistent(
                    "net impedance target must be a finite, positive Resistance quantity",
                ));
            }
        }
        Ok(())
    }
}

// ===================== Phase 3 (increment 2): BOM entities =====================
//
// The bill-of-materials layer binds the abstract [`Component`]s of the schematic to
// concrete, orderable [`Part`]s. A [`Part`] is a manufacturer part number with its
// lifecycle state; a [`BomLineItem`] is the first-class binding of one part to the set
// of components it realizes, with a build quantity. Lifecycle is carried so the BOM gate
// can flag end-of-life parts (P13: procurement risk is surfaced, never silently shipped).
// See `docs/engineering/bom-model.md`.

/// Procurement lifecycle of a [`Part`]. An [`Eol`](PartLifecycle::Eol) part can no longer be
/// sourced and must block; an [`Nrnd`](PartLifecycle::Nrnd) (not-recommended-for-new-designs)
/// part is a warning the designer should heed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartLifecycle {
    Active,
    Nrnd,
    Eol,
}

/// A concrete, orderable part identified by its manufacturer part number. Bound to the
/// abstract [`Component`]s it realizes through a [`BomLineItem`], so the BOM stays traceable
/// back to the schematic (P3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    pub id: EntityId,
    pub mpn: String,
    pub manufacturer: String,
    pub lifecycle: PartLifecycle,
    pub datasheet: String,
}

impl Part {
    /// Domain invariant: a part carries a non-empty manufacturer part number — without it
    /// the part cannot be ordered (P13). Manufacturer/datasheet completeness is a softer
    /// concern checked downstream, not a hard domain invariant.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.mpn.trim().is_empty() {
            return Err(DomainError::EmptyField("manufacturer part number"));
        }
        Ok(())
    }
}

/// A first-class binding of one [`Part`] to the [`Component`]s it realizes, with a build
/// quantity. Made addressable so BOM findings can name the offending line and trace it back
/// to both the part and its components (P3, P13). Quantity/component-link integrity is
/// re-checked at the capability seam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BomLineItem {
    pub id: EntityId,
    /// The part this line orders.
    pub part: EntityId,
    /// The component ids this line covers.
    pub components: Vec<EntityId>,
    pub quantity: u32,
}

impl BomLineItem {
    /// Domain invariants: a line covers at least one component, its quantity equals the
    /// number of components it covers, and it lists no component twice. Part/component
    /// existence and cross-line single-sourcing are re-checked at the capability seam (P3).
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.components.is_empty() {
            return Err(DomainError::EmptyField("BOM line item components"));
        }
        if self.quantity as usize != self.components.len() {
            return Err(DomainError::Inconsistent(
                "BOM line item quantity must equal the number of components it covers",
            ));
        }
        for (i, a) in self.components.iter().enumerate() {
            if self.components[i + 1..].contains(a) {
                return Err(DomainError::Inconsistent(
                    "BOM line item lists a component more than once",
                ));
            }
        }
        Ok(())
    }
}

// ===================== Phase 3 (increment 3): PCB entities =====================
//
// The PCB layer places the abstract schematic onto a physical substrate: a [`Board`] is
// the rectangular outline (with a layer count) the design must fit within; a [`Placement`]
// binds one [`Component`] to a position, courtyard extent, and [`BoardSide`] on that board.
// Physical values are typed [`PhysicalQuantity`]s (P9), compared via `si_magnitude()` so
// DRC checks (out-of-bounds, courtyard overlap) are dimensionally unambiguous. Referential
// integrity (component exists, board precedes placement) is re-checked at the capability
// seam (P3). See `docs/engineering/pcb-model.md`.

/// Which copper side of the [`Board`] a [`Placement`] sits on. Two courtyards only collide
/// when they share a side, so this drives the courtyard-overlap DRC rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoardSide {
    Top,
    Bottom,
}

/// The role a copper [`Layer`] plays in the stack: a `Signal` layer carries routed tracks; a
/// `Plane` layer is a solid copper pour (a power/ground reference). The role drives impedance
/// and return-path reasoning (see `engineering-science/pcb/ground-plane.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerRole {
    Signal,
    Plane,
}

/// One copper layer plus the dielectric beneath it in a [`LayerStack`]. `copper_thickness`
/// and `dielectric_height` are Length [`PhysicalQuantity`]s (P9, e.g. 35µm = 1oz copper on a
/// 1.6mm FR-4 core); `dielectric_er` (ε_r, ≥ 1.0) and `loss_tangent` (tan δ, ≥ 0) are
/// dimensionless ratios — modelled as plain f64 like the existing confidence/reliability
/// fields, not a new [`eak_units::Dimension`]. Carries `PhysicalQuantity` + f64, so `Layer`
/// is not `Eq` (exactly like [`Board`]/[`Track`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub role: LayerRole,
    pub copper_thickness: PhysicalQuantity,
    pub dielectric_height: PhysicalQuantity,
    pub dielectric_er: f64,
    pub loss_tangent: f64,
}

/// The board's copper/dielectric build-up, ordered top→bottom. Replaces a bare layer count so
/// there is a single source of truth (no count/stack drift) and impedance/return-path
/// reasoning has real material data to work from. See `engineering-science/pcb/stackup.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerStack {
    pub layers: Vec<Layer>,
}

impl LayerStack {
    /// The single canonical default: a 2-layer 1.6mm FR-4 board — a top `Signal` layer and a
    /// bottom `Plane` (ground reference), both 35µm (1oz) copper on the same FR-4 dielectric
    /// (ε_r 4.5, tan δ 0.02). A deterministic constant (P4): it is never sized from
    /// requirements or reasoning. Replaces every former `layers: 2`.
    pub fn standard_two_layer() -> Self {
        let copper_thickness = PhysicalQuantity::new(0.035, Unit::Millimetre); // 35µm = 1oz
        let dielectric_height = PhysicalQuantity::new(1.6, Unit::Millimetre); // FR-4 core
        let dielectric_er = 4.5;
        let loss_tangent = 0.02;
        Self {
            layers: vec![
                Layer {
                    role: LayerRole::Signal,
                    copper_thickness,
                    dielectric_height,
                    dielectric_er,
                    loss_tangent,
                },
                Layer {
                    role: LayerRole::Plane,
                    copper_thickness,
                    dielectric_height,
                    dielectric_er,
                    loss_tangent,
                },
            ],
        }
    }

    /// Domain invariants: the stack has at least one layer; every layer has positive, finite
    /// copper thickness and dielectric height (compared via `si_magnitude()`, P9); a finite
    /// ε_r ≥ 1.0; and a finite tan δ ≥ 0.0.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.layers.is_empty() {
            return Err(DomainError::Inconsistent(
                "board layer stack must have at least one layer",
            ));
        }
        for layer in &self.layers {
            if !layer.copper_thickness.si_magnitude().is_finite()
                || layer.copper_thickness.si_magnitude() <= 0.0
                || !layer.dielectric_height.si_magnitude().is_finite()
                || layer.dielectric_height.si_magnitude() <= 0.0
            {
                return Err(DomainError::Inconsistent(
                    "layer copper thickness and dielectric height must be positive and finite",
                ));
            }
            if !layer.dielectric_er.is_finite() || layer.dielectric_er < 1.0 {
                return Err(DomainError::Inconsistent(
                    "layer dielectric relative permittivity must be finite and at least 1.0",
                ));
            }
            if !layer.loss_tangent.is_finite() || layer.loss_tangent < 0.0 {
                return Err(DomainError::Inconsistent(
                    "layer loss tangent must be finite and non-negative",
                ));
            }
        }
        Ok(())
    }
}

/// The physical board outline the design must fit within: a rectangle of `width` x `height`
/// with a typed [`LayerStack`] build-up. Dimensions are typed [`PhysicalQuantity`]s (P9), so
/// placement DRC stays dimensionally unambiguous; hence `Board` is not `Eq`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Board {
    pub id: EntityId,
    pub width: PhysicalQuantity,
    pub height: PhysicalQuantity,
    pub stack: LayerStack,
}

impl Board {
    /// The number of copper layers in the stack — a convenience accessor for any count reader
    /// (the [`LayerStack`] is the single source of truth).
    pub fn layers(&self) -> u32 {
        self.stack.layers.len() as u32
    }

    /// Domain invariants: the outline has positive dimensions and a well-formed layer stack.
    /// Dimensions are compared via `si_magnitude()` so the check is unit-independent (P9).
    pub fn validate(&self) -> Result<(), DomainError> {
        if !self.width.si_magnitude().is_finite()
            || !self.height.si_magnitude().is_finite()
            || self.width.si_magnitude() <= 0.0
            || self.height.si_magnitude() <= 0.0
        {
            return Err(DomainError::Inconsistent(
                "board dimensions must be positive and finite",
            ));
        }
        self.stack.validate()?;
        Ok(())
    }
}

/// The placement of one [`Component`] on a [`Board`]: an origin (`x`, `y`), a courtyard
/// extent (`width` x `height`), and the [`BoardSide`] it occupies. Positions and extents are
/// typed [`PhysicalQuantity`]s (P9), so `Placement` is not `Eq`. Component-link and
/// board-precedence integrity are re-checked at the capability seam (P3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub id: EntityId,
    pub component: EntityId,
    pub x: PhysicalQuantity,
    pub y: PhysicalQuantity,
    pub width: PhysicalQuantity,
    pub height: PhysicalQuantity,
    pub side: BoardSide,
}

impl Placement {
    /// Domain invariant: the courtyard has positive extent — a zero-area footprint cannot be
    /// checked for overlap or fit. Extents are compared via `si_magnitude()` (P9).
    pub fn validate(&self) -> Result<(), DomainError> {
        if !self.width.si_magnitude().is_finite()
            || !self.height.si_magnitude().is_finite()
            || self.width.si_magnitude() <= 0.0
            || self.height.si_magnitude() <= 0.0
        {
            return Err(DomainError::Inconsistent(
                "placement courtyard must be positive and finite",
            ));
        }
        Ok(())
    }
}

// =================== Phase 3 (increment 4): routing entities ===================
//
// The routing layer realizes the abstract schematic [`Net`]s physically as copper: a
// [`Track`] binds one [`Net`] to a copper segment of a given `width` on one [`BoardSide`]
// layer, running from (`x1`,`y1`) to (`x2`,`y2`). One track realizes one net
// (net-realization completeness — the routing invariant), so a track is always traceable
// back through its net to the schematic and on to intent (P3). Physical values are typed
// [`PhysicalQuantity`]s (P9), compared via `si_magnitude()` so trace-width DRC stays
// dimensionally unambiguous. Net-link and net-existence integrity are re-checked at the
// capability seam (P3). See `docs/state-machines/routing-planning.md`.

/// A copper realization of one [`Net`]: a trace of `width` on one [`BoardSide`] layer,
/// running from (`x1`,`y1`) to (`x2`,`y2`). Positions and width are typed
/// [`PhysicalQuantity`]s (P9), so `Track` is not `Eq`. Net-link integrity is re-checked at
/// the capability seam (P3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: EntityId,
    /// The net this track realizes.
    pub net: EntityId,
    pub layer: BoardSide,
    /// The copper width of the trace (P9).
    pub width: PhysicalQuantity,
    pub x1: PhysicalQuantity,
    pub y1: PhysicalQuantity,
    pub x2: PhysicalQuantity,
    pub y2: PhysicalQuantity,
}

impl Track {
    /// Domain invariants: a trace has a positive, finite copper width — a zero/negative-width
    /// trace carries no copper and cannot be DRC-checked — and finite endpoints. Width is
    /// compared via `si_magnitude()` so the check is unit-independent (P9). Net-link existence
    /// is re-checked at the capability seam (P3).
    pub fn validate(&self) -> Result<(), DomainError> {
        if !self.width.si_magnitude().is_finite() || self.width.si_magnitude() <= 0.0 {
            return Err(DomainError::Inconsistent(
                "track width must be positive and finite",
            ));
        }
        if !self.x1.si_magnitude().is_finite()
            || !self.y1.si_magnitude().is_finite()
            || !self.x2.si_magnitude().is_finite()
            || !self.y2.si_magnitude().is_finite()
        {
            return Err(DomainError::Inconsistent("track endpoints must be finite"));
        }
        Ok(())
    }
}

// =================== Band A (Phase 3, increment 1): the honesty object ===================
//
// An [`Assumption`] makes the reasoning's presumptions first-class and auditable (Map 10;
// `00` Principle 7 — honesty). It states what was presumed, `rests_on` a committed entity
// (so it is always traceable), carries a [`AssumptionCriticality`], and moves through a
// lifecycle (Open -> Discharged | Invalidated). A Critical + Open assumption BLOCKS release
// (mirrors [`Violation::is_blocking`]) — the honesty gate. Discharging it resolves it to a
// grounded fact, an enforced constraint, or an accepted risk. `validate()` reuses existing
// [`DomainError`] variants only. `Assumption` has no `f64`/`PhysicalQuantity` field, so `Eq`
// is legal (unlike `Component`/`Net`).

/// How load-bearing an [`Assumption`] is. A `Critical` open assumption blocks release; a
/// `Normal` one is surfaced but never blocking. Every assumption states its criticality
/// explicitly (no `Default`), like [`ViolationSeverity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssumptionCriticality {
    Critical,
    Normal,
}

/// Lifecycle of an [`Assumption`], mirroring [`ViolationStatus`]. An `Open` assumption is
/// still relied upon (and, if `Critical`, blocks); a `Discharged` one has been grounded; an
/// `Invalidated` one is no longer relied upon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssumptionStatus {
    Open,
    Discharged,
    Invalidated,
}

/// How an [`Assumption`] was discharged: grounded in a fact, enforced by a constraint, or
/// accepted as a (tracked) risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DischargeResolution {
    GroundedFact,
    EnforcedConstraint,
    AcceptedRisk,
}

/// The record of discharging an [`Assumption`]: how it was resolved, the `target` entity it
/// resolved to, and the human/agent who decided (`decided_by` mirrors [`Waiver::decided_by`],
/// `00` Principle 10 — the decider is named).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discharge {
    pub resolution: DischargeResolution,
    pub target: EntityId,
    pub decided_by: String,
}

/// A first-class, auditable presumption the reasoning made. See the module comment above.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assumption {
    pub id: EntityId,
    pub statement: String,
    /// The committed entity (requirement / decision / functional block) this rests on — the
    /// traceability anchor. Referential integrity is re-checked at the capability seam (P3).
    pub rests_on: EntityId,
    pub criticality: AssumptionCriticality,
    pub status: AssumptionStatus,
    /// Set exactly when `status == Discharged`.
    pub discharge: Option<Discharge>,
}

impl Assumption {
    /// Still relied upon.
    pub fn is_open(&self) -> bool {
        self.status == AssumptionStatus::Open
    }

    /// An assumption blocks release iff it is Critical AND still Open (mirrors
    /// [`Violation::is_blocking`] exactly — the honesty gate reads this).
    pub fn is_blocking(&self) -> bool {
        self.criticality == AssumptionCriticality::Critical && self.status == AssumptionStatus::Open
    }

    /// Domain invariants (reuses existing [`DomainError`] variants only): a non-empty
    /// statement; and a discharge record present iff (and only iff) `status == Discharged`.
    /// The `rests_on`/discharge-target referential checks live at the capability seam (P3).
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.statement.trim().is_empty() {
            return Err(DomainError::EmptyStatement);
        }
        match (self.status, self.discharge.is_some()) {
            (AssumptionStatus::Discharged, false) => Err(DomainError::Inconsistent(
                "discharged assumption must record a discharge",
            )),
            (status, true) if status != AssumptionStatus::Discharged => Err(
                DomainError::Inconsistent("only a discharged assumption may carry a discharge"),
            ),
            _ => Ok(()),
        }
    }
}

// ================= Band A (increment 2): ModelFidelity =================
//
// A trust-tag on a derived/predicted fact (Map 6). It is ADVISORY attached metadata, modeled
// EXACTLY like `ViolationExplanation`: it carries NO `EntityId` of its own, references a target,
// folds into its own store, and never mutates the target — so it can never usurp an object's
// authority (P3). Its one domain invariant is a numeric boundary: `confidence ∈ [0,1]`.
// `ModelFidelity` carries an `f64`, so — like `Component`/`Net` — it derives `PartialEq` but NOT
// `Eq`. `validate()` reuses `Inconsistent` (no new `DomainError` variant).

/// How a fact was established, from weakest to strongest. A tag declares which method backs the
/// number so the honesty is visible (`00` Principle 7 — honest about certainty).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FidelityMethod {
    /// A bare presumption, no calculation behind it.
    Assumed,
    /// A conservative first-order floor (e.g. an IPC default), not a real calculation.
    FirstOrderFloor,
    /// Hand/closed-form calculation.
    Calculated,
    /// Established by simulation.
    Simulated,
    /// Established by physical measurement — the strongest.
    Measured,
}

/// An advisory trust-tag on a derived fact: which `concern` it speaks to, by what `method`, with
/// what `confidence ∈ [0,1]`, over what `scope`. It has no id and no lifecycle; it only describes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelFidelity {
    pub concern: String,
    pub method: FidelityMethod,
    pub confidence: f64,
    pub scope: String,
}

impl ModelFidelity {
    /// The single domain invariant: `confidence` is a real number in the CLOSED interval
    /// `[0, 1]`. Written as `!(0.0..=1.0).contains(&c)` so that NaN — which is neither `>= 0`
    /// nor `<= 1` — is rejected rather than silently admitted. Reuses `Inconsistent`.
    pub fn validate(&self) -> Result<(), DomainError> {
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(DomainError::Inconsistent(
                "fidelity confidence must be within [0, 1]",
            ));
        }
        Ok(())
    }
}

// ===================== Band A (increment 3): Risk =====================
//
// An auditable risk-posture object (Map 46; `00` Principle 11 — humans own goals/acceptance).
// A `Risk` states a hazard, its `likelihood`/`severity`, a `mitigation`, the `residual`
// severity that remains after mitigation, the `owner` who is accountable, and a lifecycle
// (Open -> Mitigated -> Accepted). The HUMAN owns acceptance of residual risk: the
// `AcceptRisk` seam/fold (not `validate()`) enforces that authority. Risk is TRACKED TRUTH —
// it does NOT block release in v0. `validate()` reuses existing `DomainError` variants only
// (non-empty statement + owner). `Risk` has no `f64`/`PhysicalQuantity` field, so `Eq` is
// legal (like `Assumption`, unlike `Component`/`Net`).

/// How likely the hazard is to occur. Explicit on every risk (no `Default`), like
/// [`AssumptionCriticality`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLikelihood {
    Low,
    Medium,
    High,
}

/// How damaging the hazard is if it occurs. Reused for both the raw `severity` and the
/// `residual` severity that remains after mitigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Lifecycle of a [`Risk`]: `Open` (raised, unmitigated), `Mitigated` (a mitigation is in
/// place but residual remains), `Accepted` (a named human owner has accepted the residual).
/// The `Open -> Accepted` transition is a human-authority act enforced at the seam/fold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskStatus {
    Open,
    Mitigated,
    Accepted,
}

/// A first-class, auditable risk. See the module comment above.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Risk {
    pub id: EntityId,
    pub statement: String,
    pub likelihood: RiskLikelihood,
    pub severity: RiskSeverity,
    pub mitigation: String,
    /// The severity that remains AFTER the mitigation is applied.
    pub residual: RiskSeverity,
    /// The named human accountable for this risk (Principle 11). Required non-empty.
    pub owner: String,
    pub status: RiskStatus,
}

impl Risk {
    /// Domain invariants (reuses existing [`DomainError`] variants only): a non-empty
    /// `statement` and a non-empty `owner` (a risk with no accountable owner cannot be owned
    /// or accepted). `validate()` is status-agnostic — the `Open -> Accepted` transition and
    /// the human-acceptance authority live at the seam/fold, like [`Assumption`]'s discharge.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.statement.trim().is_empty() {
            return Err(DomainError::EmptyStatement);
        }
        if self.owner.trim().is_empty() {
            return Err(DomainError::EmptyField("risk owner"));
        }
        Ok(())
    }
}

// ===================== Band A (increment 4): Objective / Tradeoff =====================
//
// The weighed-and-rejected space, preserved (Map 11; `00` Principle 7 — the design remembers
// what it did NOT choose and why). An [`Objective`] is a weighted goal rooted in a committed
// `source`. A [`Tradeoff`] records the [`Alternative`]s that were considered, the `criteria` they
// were scored against, the `chosen` index, the `rationale`, and — critically — PRESERVES the
// rejected space: at least one rejected alternative must survive, and the chosen one may not itself
// be marked rejected. `validate()` reuses existing `DomainError` variants only (`EmptyStatement` /
// `Inconsistent`). Both carry an `f64` (weight / scores), so — unlike [`Assumption`]/[`Risk`] —
// they derive `PartialEq` but NOT `Eq` (mirrors [`Decision`]/[`Component`]).

/// A single option that was weighed in a [`Tradeoff`]. `scores` are its raw scores against the
/// tradeoff's `criteria` (positional). `rejected` records whether the design set it aside — the
/// rejected space is what a tradeoff exists to preserve (`00` Principle 7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alternative {
    pub label: String,
    pub description: String,
    pub scores: Vec<f64>,
    pub rejected: bool,
}

/// A weighted design goal, rooted in the entity it derives from (`source`). `weight` expresses its
/// relative importance among competing objectives. Map 11.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Objective {
    pub id: EntityId,
    pub statement: String,
    pub weight: f64,
    /// The entity (e.g. a [`Requirement`] or [`DesignIntent`]) this objective is rooted in.
    pub source: EntityId,
}

impl Objective {
    /// Domain invariant (reuses existing [`DomainError`] variants only): a non-empty `statement`.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.statement.trim().is_empty() {
            return Err(DomainError::EmptyStatement);
        }
        Ok(())
    }
}

/// A recorded decision among weighed [`Alternative`]s, preserving the rejected space (Map 11). The
/// `chosen` index selects the winning alternative; `criteria`/`rationale`/`decided_by` record how
/// and by whom (Principle 10, named authority). A [`Decision`] may later cite the `Tradeoff` it
/// resolved via a [`ProvenanceLink`], tying the choice to what it beat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tradeoff {
    pub id: EntityId,
    pub question: String,
    pub alternatives: Vec<Alternative>,
    pub criteria: Vec<String>,
    pub chosen: usize,
    pub rationale: String,
    /// The named human/agent who decided (Principle 10), like [`Waiver::decided_by`].
    pub decided_by: String,
}

impl Tradeoff {
    /// Domain invariants (reuses existing [`DomainError`] variants only): (1) at least two
    /// alternatives were weighed (a single option is no tradeoff); (2) `chosen` is a valid index;
    /// (3) the chosen alternative is not itself marked rejected (self-contradiction); (4) at least
    /// one rejected alternative is PRESERVED — the whole point of the object is to remember the
    /// space it did not choose (`00` Principle 7, exit criterion 3).
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.alternatives.len() < 2 {
            return Err(DomainError::Inconsistent(
                "a tradeoff must weigh at least two alternatives",
            ));
        }
        let chosen = self
            .alternatives
            .get(self.chosen)
            .ok_or(DomainError::Inconsistent(
                "tradeoff chosen index is out of range",
            ))?;
        if chosen.rejected {
            return Err(DomainError::Inconsistent(
                "the chosen alternative may not be marked rejected",
            ));
        }
        if !self.alternatives.iter().any(|a| a.rejected) {
            return Err(DomainError::Inconsistent(
                "a tradeoff must preserve at least one rejected alternative",
            ));
        }
        Ok(())
    }
}

// ===================== Band B (Phase 5, increment 1): Power Domain =====================
//
// The first logical-electrical Map (Map 38; `02 §Band B`). A [`PowerDomain`] is a named power
// rail: the set of [`Net`]s that must all be held at a nominal `voltage`, supplied by a single
// [`Component`] (a regulator/connector) with a finite `max_current` the source can deliver.
// Real power engineering (KCL / PI, `engineering-science/electrical/*`) reduces to a *budget*:
// the sum of the currents the domain's nets must carry must not exceed what its source can
// supply. That check is [`PowerBalanceRule`] (in `eak-engines`), not `validate()` — a domain is
// self-consistent even when overloaded; it is the *design* that is wrong, and the rule says so.
//
// `PowerDomain` carries `PhysicalQuantity` fields, so — like [`Component`]/[`Net`] — it derives
// `PartialEq` but NOT `Eq`. `validate()` reuses existing [`DomainError`] variants only.

/// A named power rail: one [`Net`] group held at `voltage`, supplied by one source component.
/// See the module comment above.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerDomain {
    pub id: EntityId,
    /// The rail name (e.g. `"3V3"`, `"VBUS"`, `"VDD_CORE"`). The canonical handle engineers use.
    pub name: String,
    /// The nominal voltage the rail must hold (P9, e.g. 3.3 V).
    pub voltage: PhysicalQuantity,
    /// The [`Component`] (regulator / connector) that supplies this rail. The traceability anchor
    /// back to intent (P3). Referential integrity re-checked at the capability seam.
    pub source_component: EntityId,
    /// The maximum current the source can deliver (P9, e.g. 3 A) — the supply side of the budget.
    pub max_current: PhysicalQuantity,
    /// The [`Net`]s that belong to this rail. At least one (a rail powering nothing is inert,
    /// mirroring `component has no pins`). Each must be a committed net (re-checked at the seam).
    pub nets: Vec<EntityId>,
}

impl PowerDomain {
    /// Domain invariants (reuses existing [`DomainError`] variants only): a non-empty `name`;
    /// a finite, positive `voltage`; a finite, positive `max_current`; and at least one net —
    /// a rail that powers nothing is a silent defect (P13), exactly like a component with no pins.
    /// Values are compared via `si_magnitude()` so the checks are unit-independent (P9). Net-link
    /// integrity is re-checked at the capability seam (P3).
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::EmptyField("power domain name"));
        }
        if !self.voltage.si_magnitude().is_finite() || self.voltage.si_magnitude() <= 0.0 {
            return Err(DomainError::Inconsistent(
                "power domain voltage must be positive and finite",
            ));
        }
        if !self.max_current.si_magnitude().is_finite() || self.max_current.si_magnitude() <= 0.0 {
            return Err(DomainError::Inconsistent(
                "power domain max current must be positive and finite",
            ));
        }
        if self.nets.is_empty() {
            return Err(DomainError::Inconsistent(
                "power domain must power at least one net",
            ));
        }
        Ok(())
    }
}

// ===================== Band B (Phase 5, increment 2): Clock Domain =====================
//
// The second logical-electrical Map (Map 21; `02 §Band B`). A [`ClockDomain`] is a named clock
// region: a single [`Component`] (oscillator / crystal / PLL / clock generator) that sources a
// `frequency` (P9, typed Hertz) to a set of [`Net`]s that are synchronous to that clock. This is
// the abstraction the return-path continuity rule (`engineering-science/pcb/return-path.md` L138)
// targets — that rule applies to *controlled / electrically-long* nets, a set the runtime can only
// identify once it owns clock frequencies — and the seed of CDC (clock-domain-crossing) reasoning.
// The seam keeps referential integrity (source component + member nets must be committed); a net
// that belongs to more than one clock domain is a *design* finding (a clock-domain conflict,
// [`ClockDomainMembershipRule`] in `eak-engines`), NOT a validation error — the domain itself is
// well-formed the moment it names a real source, a positive frequency, and at least one net.
//
// `ClockDomain` carries a `PhysicalQuantity` field, so — like [`Component`]/[`Net`]/[`PowerDomain`]
// — it derives `PartialEq` but NOT `Eq`. `validate()` reuses existing [`DomainError`] variants only.

/// A named clock region: one source [`Component`] driving `frequency` onto a set of synchronous
/// [`Net`]s. See the module comment above.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClockDomain {
    pub id: EntityId,
    /// The domain name (e.g. `"SYS"`, `"48M"`, `"I2S_MCLK"`). The canonical handle engineers use.
    pub name: String,
    /// The domain's clock frequency (P9, e.g. 48 MHz). The net class that makes a net
    /// "electrically-long" for return-path and SI purposes.
    pub frequency: PhysicalQuantity,
    /// The [`Component`] (oscillator / crystal / PLL / clock generator) that sources this clock.
    /// The traceability anchor back to intent (P3). Referential integrity re-checked at the seam.
    pub source_component: EntityId,
    /// The [`Net`]s synchronous to this clock. At least one (a clock that drives nothing is inert,
    /// mirroring `power domain must power at least one net`). Each must be a committed net
    /// (re-checked at the seam).
    pub members: Vec<EntityId>,
}

impl ClockDomain {
    /// Domain invariants (reuses existing [`DomainError`] variants only): a non-empty `name`; a
    /// finite, positive `frequency`; and at least one member net — a clock driving nothing is a
    /// silent defect (P13). Values are compared via `si_magnitude()` so the checks are
    /// unit-independent (P9). Member-link integrity is re-checked at the capability seam (P3).
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::EmptyField("clock domain name"));
        }
        if !self.frequency.si_magnitude().is_finite() || self.frequency.si_magnitude() <= 0.0 {
            return Err(DomainError::Inconsistent(
                "clock domain frequency must be positive and finite",
            ));
        }
        if self.members.is_empty() {
            return Err(DomainError::Inconsistent(
                "clock domain must drive at least one net",
            ));
        }
        Ok(())
    }
}

/// A violated domain invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyStatement,
    /// A named, required text field was blank (carries the field's human label, so the
    /// rejection message is accurate for whichever entity raised it).
    EmptyField(&'static str),
    /// An entity's fields are internally inconsistent (carries a human explanation).
    Inconsistent(&'static str),
    AcceptedRequirementNeedsCriterion,
    AcceptedRequirementNeedsSource,
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::EmptyStatement => write!(f, "requirement statement is empty"),
            DomainError::EmptyField(field) => write!(f, "{field} must not be empty"),
            DomainError::Inconsistent(msg) => write!(f, "{msg}"),
            DomainError::AcceptedRequirementNeedsCriterion => {
                write!(f, "accepted requirement lacks an acceptance criterion")
            }
            DomainError::AcceptedRequirementNeedsSource => {
                write!(f, "accepted requirement lacks a source")
            }
        }
    }
}
impl std::error::Error for DomainError {}

impl Requirement {
    pub fn is_testable(&self) -> bool {
        !self.acceptance_criterion.trim().is_empty()
    }

    /// Domain invariants (engineering-domain-model Requirement invariant; requirement-ir
    /// invariant 2): an *accepted* Requirement is testable and rooted in a source.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.statement.trim().is_empty() {
            return Err(DomainError::EmptyStatement);
        }
        if self.status == RequirementStatus::Accepted {
            if !self.is_testable() {
                return Err(DomainError::AcceptedRequirementNeedsCriterion);
            }
            if self.source.is_null() {
                return Err(DomainError::AcceptedRequirementNeedsSource);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(status: RequirementStatus, crit: &str, source: EntityId) -> Requirement {
        Requirement {
            id: EntityId(1),
            statement: "Operating power shall not exceed 5 W".into(),
            category: RequirementCategory::Electrical,
            priority: Priority::High,
            acceptance_criterion: crit.into(),
            status,
            source,
            targets: vec![],
        }
    }

    #[test]
    fn accepted_requirement_needs_criterion() {
        let r = req(RequirementStatus::Accepted, "", EntityId(2));
        assert_eq!(
            r.validate(),
            Err(DomainError::AcceptedRequirementNeedsCriterion)
        );
    }

    #[test]
    fn accepted_requirement_needs_source() {
        let r = req(
            RequirementStatus::Accepted,
            "measured power < 5 W",
            EntityId::NULL,
        );
        assert_eq!(
            r.validate(),
            Err(DomainError::AcceptedRequirementNeedsSource)
        );
    }

    #[test]
    fn well_formed_accepted_requirement_validates() {
        let r = req(
            RequirementStatus::Accepted,
            "measured power < 5 W",
            EntityId(2),
        );
        assert!(r.validate().is_ok());
        assert!(r.is_testable());
    }

    #[test]
    fn proposed_requirement_may_lack_criterion() {
        let r = req(RequirementStatus::Proposed, "", EntityId(2));
        assert!(r.validate().is_ok());
    }

    #[test]
    fn entity_id_null_is_reserved() {
        assert!(EntityId::NULL.is_null());
        assert!(!EntityId(1).is_null());
    }

    #[test]
    fn constraint_rejects_empty_statement() {
        let c = Constraint {
            id: EntityId(1),
            statement: "   ".into(),
            subject_requirement: EntityId(2),
            kind: ConstraintKind::Max,
            bound: PhysicalQuantity::new(5.0, eak_units::Unit::Watt),
            source: EntityId(2),
            status: ConstraintStatus::Active,
        };
        assert_eq!(c.validate(), Err(DomainError::EmptyStatement));
    }

    #[test]
    fn well_formed_constraint_validates_and_is_active() {
        let c = Constraint {
            id: EntityId(1),
            statement: "power <= 5 W".into(),
            subject_requirement: EntityId(2),
            kind: ConstraintKind::Max,
            bound: PhysicalQuantity::new(5.0, eak_units::Unit::Watt),
            source: EntityId(2),
            status: ConstraintStatus::Active,
        };
        assert!(c.validate().is_ok());
        assert!(c.is_active());
    }

    #[test]
    fn only_open_error_violations_block() {
        let mut v = Violation {
            id: EntityId(1),
            rule: "constraint-consistency".into(),
            severity: ViolationSeverity::Error,
            subjects: vec![EntityId(2), EntityId(3)],
            message: "contradictory bounds".into(),
            status: ViolationStatus::Open,
        };
        assert!(v.is_blocking());
        v.status = ViolationStatus::Waived;
        assert!(!v.is_blocking());
        v.status = ViolationStatus::Open;
        v.severity = ViolationSeverity::Warning;
        assert!(!v.is_blocking());
    }

    #[test]
    fn functional_block_rejects_blank_name() {
        let b = FunctionalBlock {
            id: EntityId(1),
            name: "   ".into(),
            function: "5 V rail".into(),
            requirements: vec![EntityId(2)],
        };
        assert_eq!(
            b.validate(),
            Err(DomainError::EmptyField("functional block name"))
        );
    }

    #[test]
    fn well_formed_functional_block_validates() {
        let b = FunctionalBlock {
            id: EntityId(1),
            name: "Power Supply".into(),
            function: "step 12 V down to 5 V".into(),
            requirements: vec![EntityId(2)],
        };
        assert!(b.validate().is_ok());
    }

    #[test]
    fn component_rejects_blank_refdes() {
        let c = Component {
            id: EntityId(1),
            refdes: "  ".into(),
            class: ComponentClass::Regulator,
            value: None,
            from_block: EntityId(2),
            origin: ComponentOrigin::Synthesized,
        };
        assert_eq!(
            c.validate(),
            Err(DomainError::EmptyField("component reference designator"))
        );
    }

    #[test]
    fn well_formed_component_validates() {
        let c = Component {
            id: EntityId(1),
            refdes: "U1".into(),
            class: ComponentClass::Resistor,
            value: Some(PhysicalQuantity::new(10_000.0, eak_units::Unit::Ohm)),
            from_block: EntityId(2),
            origin: ComponentOrigin::Synthesized,
        };
        assert!(c.validate().is_ok());
    }

    #[test]
    fn component_origin_defaults_to_synthesized_so_old_logs_replay_unchanged() {
        // The default (which `#[serde(default)]` backfills onto pre-origin event logs, so historical
        // logs deserialize as Synthesized — their original meaning) is Synthesized (P4). Serde-level
        // backfill is proven end-to-end in the runtime replay tests, where serde_json is already a
        // dependency; the entities ring keeps its dependency surface minimal.
        assert_eq!(ComponentOrigin::default(), ComponentOrigin::Synthesized);
        let imported = Component {
            id: EntityId(1),
            refdes: "R1".into(),
            class: ComponentClass::Resistor,
            value: None,
            from_block: EntityId::NULL,
            origin: ComponentOrigin::Imported,
        };
        // An imported component validates with a null block — validate() is origin-agnostic (the
        // origin-dependent block rule lives at the capability seam, P3), exactly like Net.
        assert!(imported.validate().is_ok());
    }

    #[test]
    fn net_rejects_blank_name() {
        let n = Net {
            id: EntityId(1),
            name: "".into(),
            class: NetClass::Power,
            members: vec![EntityId(2), EntityId(3)],
            current: None,
            impedance_target: None,
            origin: NetOrigin::Logical,
        };
        assert_eq!(n.validate(), Err(DomainError::EmptyField("net name")));
    }

    #[test]
    fn well_formed_net_validates() {
        let n = Net {
            id: EntityId(1),
            name: "+5V".into(),
            class: NetClass::Power,
            members: vec![EntityId(2), EntityId(3)],
            current: None,
            impedance_target: None,
            origin: NetOrigin::Logical,
        };
        assert!(n.validate().is_ok());
    }

    #[test]
    fn net_accepts_a_finite_positive_current() {
        let n = Net {
            id: EntityId(1),
            name: "+5V".into(),
            class: NetClass::Power,
            members: vec![EntityId(2), EntityId(3)],
            current: Some(PhysicalQuantity::new(2.0, Unit::Ampere)),
            impedance_target: None,
            origin: NetOrigin::Logical,
        };
        assert!(n.validate().is_ok());
    }

    #[test]
    fn net_rejects_a_dimensionally_wrong_current() {
        // A length where a current belongs is a dimensional error (P9).
        let n = Net {
            id: EntityId(1),
            name: "+5V".into(),
            class: NetClass::Power,
            members: vec![EntityId(2)],
            current: Some(PhysicalQuantity::new(2.0, Unit::Millimetre)),
            impedance_target: None,
            origin: NetOrigin::Logical,
        };
        assert!(matches!(n.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn net_rejects_a_non_positive_current() {
        let n = Net {
            id: EntityId(1),
            name: "GND".into(),
            class: NetClass::Ground,
            members: vec![EntityId(2)],
            current: Some(PhysicalQuantity::new(0.0, Unit::Ampere)),
            impedance_target: None,
            origin: NetOrigin::Logical,
        };
        assert!(matches!(n.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn net_accepts_a_finite_positive_impedance_target() {
        let n = Net {
            id: EntityId(1),
            name: "USB_DP".into(),
            class: NetClass::Signal,
            members: vec![EntityId(2)],
            current: None,
            impedance_target: Some(PhysicalQuantity::new(50.0, Unit::Ohm)),
            origin: NetOrigin::Logical,
        };
        assert!(n.validate().is_ok());
    }

    #[test]
    fn net_rejects_a_dimensionally_wrong_impedance_target() {
        // A length where an impedance belongs is a dimensional error (P9).
        let n = Net {
            id: EntityId(1),
            name: "USB_DP".into(),
            class: NetClass::Signal,
            members: vec![EntityId(2)],
            current: None,
            impedance_target: Some(PhysicalQuantity::new(50.0, Unit::Millimetre)),
            origin: NetOrigin::Logical,
        };
        assert!(matches!(n.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn part_rejects_blank_mpn() {
        let p = Part {
            id: EntityId(1),
            mpn: "   ".into(),
            manufacturer: "Texas Instruments".into(),
            lifecycle: PartLifecycle::Active,
            datasheet: "https://ti.com/lm1117".into(),
        };
        assert_eq!(
            p.validate(),
            Err(DomainError::EmptyField("manufacturer part number"))
        );
    }

    #[test]
    fn well_formed_part_validates() {
        let p = Part {
            id: EntityId(1),
            mpn: "LM1117-3.3".into(),
            manufacturer: "Texas Instruments".into(),
            lifecycle: PartLifecycle::Eol,
            datasheet: "https://ti.com/lm1117".into(),
        };
        assert!(p.validate().is_ok());
    }

    fn mm(v: f64) -> PhysicalQuantity {
        PhysicalQuantity::new(v, eak_units::Unit::Millimetre)
    }

    #[test]
    fn board_rejects_non_positive_dimensions() {
        let zero_width = Board {
            id: EntityId(1),
            width: mm(0.0),
            height: mm(50.0),
            stack: LayerStack::standard_two_layer(),
        };
        assert_eq!(
            zero_width.validate(),
            Err(DomainError::Inconsistent(
                "board dimensions must be positive and finite"
            ))
        );
        let negative_height = Board {
            id: EntityId(1),
            width: mm(50.0),
            height: mm(-1.0),
            stack: LayerStack::standard_two_layer(),
        };
        assert_eq!(
            negative_height.validate(),
            Err(DomainError::Inconsistent(
                "board dimensions must be positive and finite"
            ))
        );
    }

    #[test]
    fn board_rejects_empty_stack() {
        let b = Board {
            id: EntityId(1),
            width: mm(50.0),
            height: mm(50.0),
            stack: LayerStack { layers: vec![] },
        };
        assert_eq!(
            b.validate(),
            Err(DomainError::Inconsistent(
                "board layer stack must have at least one layer"
            ))
        );
    }

    #[test]
    fn well_formed_board_validates() {
        let b = Board {
            id: EntityId(1),
            width: mm(50.0),
            height: mm(40.0),
            stack: LayerStack::standard_two_layer(),
        };
        assert!(b.validate().is_ok());
        // The convenience accessor reflects the stack's layer count.
        assert_eq!(b.layers(), 2);
    }

    #[test]
    fn standard_two_layer_stack_validates() {
        let stack = LayerStack::standard_two_layer();
        assert!(stack.validate().is_ok());
        assert_eq!(stack.layers.len(), 2);
        assert_eq!(stack.layers[0].role, LayerRole::Signal);
        assert_eq!(stack.layers[1].role, LayerRole::Plane);
    }

    #[test]
    fn layer_stack_rejects_empty() {
        let stack = LayerStack { layers: vec![] };
        assert_eq!(
            stack.validate(),
            Err(DomainError::Inconsistent(
                "board layer stack must have at least one layer"
            ))
        );
    }

    fn good_layer() -> Layer {
        Layer {
            role: LayerRole::Signal,
            copper_thickness: mm(0.035),
            dielectric_height: mm(1.6),
            dielectric_er: 4.5,
            loss_tangent: 0.02,
        }
    }

    #[test]
    fn layer_stack_rejects_non_positive_copper_thickness() {
        let stack = LayerStack {
            layers: vec![Layer {
                copper_thickness: mm(0.0),
                ..good_layer()
            }],
        };
        assert_eq!(
            stack.validate(),
            Err(DomainError::Inconsistent(
                "layer copper thickness and dielectric height must be positive and finite"
            ))
        );
    }

    #[test]
    fn layer_stack_rejects_non_positive_dielectric_height() {
        let stack = LayerStack {
            layers: vec![Layer {
                dielectric_height: mm(-1.0),
                ..good_layer()
            }],
        };
        assert_eq!(
            stack.validate(),
            Err(DomainError::Inconsistent(
                "layer copper thickness and dielectric height must be positive and finite"
            ))
        );
    }

    #[test]
    fn layer_stack_rejects_permittivity_below_one() {
        let stack = LayerStack {
            layers: vec![Layer {
                dielectric_er: 0.5,
                ..good_layer()
            }],
        };
        assert_eq!(
            stack.validate(),
            Err(DomainError::Inconsistent(
                "layer dielectric relative permittivity must be finite and at least 1.0"
            ))
        );
    }

    #[test]
    fn layer_stack_rejects_negative_loss_tangent() {
        let stack = LayerStack {
            layers: vec![Layer {
                loss_tangent: -0.01,
                ..good_layer()
            }],
        };
        assert_eq!(
            stack.validate(),
            Err(DomainError::Inconsistent(
                "layer loss tangent must be finite and non-negative"
            ))
        );
    }

    #[test]
    fn placement_rejects_non_positive_courtyard() {
        let zero_width = Placement {
            id: EntityId(1),
            component: EntityId(2),
            x: mm(1.0),
            y: mm(1.0),
            width: mm(0.0),
            height: mm(5.0),
            side: BoardSide::Top,
        };
        assert_eq!(
            zero_width.validate(),
            Err(DomainError::Inconsistent(
                "placement courtyard must be positive and finite"
            ))
        );
        let negative_height = Placement {
            id: EntityId(1),
            component: EntityId(2),
            x: mm(1.0),
            y: mm(1.0),
            width: mm(5.0),
            height: mm(-2.0),
            side: BoardSide::Bottom,
        };
        assert_eq!(
            negative_height.validate(),
            Err(DomainError::Inconsistent(
                "placement courtyard must be positive and finite"
            ))
        );
    }

    #[test]
    fn well_formed_placement_validates() {
        let p = Placement {
            id: EntityId(1),
            component: EntityId(2),
            x: mm(10.0),
            y: mm(10.0),
            width: mm(5.0),
            height: mm(5.0),
            side: BoardSide::Top,
        };
        assert!(p.validate().is_ok());
    }

    fn track(width: f64) -> Track {
        Track {
            id: EntityId(1),
            net: EntityId(2),
            layer: BoardSide::Top,
            width: mm(width),
            x1: mm(1.0),
            y1: mm(1.0),
            x2: mm(9.0),
            y2: mm(1.0),
        }
    }

    #[test]
    fn track_rejects_non_positive_width() {
        assert_eq!(
            track(0.0).validate(),
            Err(DomainError::Inconsistent(
                "track width must be positive and finite"
            ))
        );
        assert_eq!(
            track(-0.2).validate(),
            Err(DomainError::Inconsistent(
                "track width must be positive and finite"
            ))
        );
    }

    #[test]
    fn track_rejects_non_finite_endpoint() {
        let mut t = track(0.25);
        t.x2 = mm(f64::INFINITY);
        assert_eq!(
            t.validate(),
            Err(DomainError::Inconsistent("track endpoints must be finite"))
        );
    }

    #[test]
    fn well_formed_track_validates() {
        assert!(track(0.25).validate().is_ok());
    }

    // ===================== Band A (increment 1): Assumption =====================
    //
    // TDD (written before the object): the honesty object. An [`Assumption`] states what
    // the reasoning presumed, rests on a committed entity, carries a criticality, and moves
    // through a lifecycle (Open -> Discharged | Invalidated). A Critical + Open assumption
    // BLOCKS release (mirrors `Violation::is_blocking`). `validate()` reuses existing
    // `DomainError` variants only (no new variant).

    fn open_assumption(statement: &str, crit: AssumptionCriticality) -> Assumption {
        Assumption {
            id: EntityId(1),
            statement: statement.into(),
            rests_on: EntityId(2),
            criticality: crit,
            status: AssumptionStatus::Open,
            discharge: None,
        }
    }

    #[test]
    fn assumption_rejects_empty_statement() {
        let a = open_assumption("   ", AssumptionCriticality::Critical);
        assert_eq!(a.validate(), Err(DomainError::EmptyStatement));
    }

    #[test]
    fn well_formed_open_assumption_validates() {
        let a = open_assumption(
            "the load draws at most 500 mA",
            AssumptionCriticality::Normal,
        );
        assert!(a.validate().is_ok());
    }

    #[test]
    fn discharged_assumption_without_a_discharge_is_inconsistent() {
        // Rule (2): status == Discharged but discharge is None -> Inconsistent.
        let mut a = open_assumption("the rail is 3.3 V", AssumptionCriticality::Critical);
        a.status = AssumptionStatus::Discharged;
        a.discharge = None;
        assert_eq!(
            a.validate(),
            Err(DomainError::Inconsistent(
                "discharged assumption must record a discharge"
            ))
        );
    }

    #[test]
    fn non_discharged_assumption_carrying_a_discharge_is_inconsistent() {
        // Rule (3): status != Discharged but discharge is Some -> Inconsistent. An Open (or
        // Invalidated) assumption may not already carry a discharge record.
        let mut a = open_assumption("the rail is 3.3 V", AssumptionCriticality::Critical);
        a.status = AssumptionStatus::Open;
        a.discharge = Some(Discharge {
            resolution: DischargeResolution::GroundedFact,
            target: EntityId(3),
            decided_by: "engineer".into(),
        });
        assert_eq!(
            a.validate(),
            Err(DomainError::Inconsistent(
                "only a discharged assumption may carry a discharge"
            ))
        );
    }

    #[test]
    fn well_formed_discharged_assumption_validates() {
        let a = Assumption {
            id: EntityId(1),
            statement: "the rail is 3.3 V".into(),
            rests_on: EntityId(2),
            criticality: AssumptionCriticality::Critical,
            status: AssumptionStatus::Discharged,
            discharge: Some(Discharge {
                resolution: DischargeResolution::EnforcedConstraint,
                target: EntityId(3),
                decided_by: "engineer".into(),
            }),
        };
        assert!(a.validate().is_ok());
    }

    #[test]
    fn only_open_critical_assumptions_block() {
        // is_blocking mirrors Violation::is_blocking exactly: Critical AND Open blocks; a
        // discharged/invalidated Critical does not, and an open Normal does not.
        let mut a = open_assumption("critical presumption", AssumptionCriticality::Critical);
        assert!(a.is_open());
        assert!(a.is_blocking());

        // Discharged Critical: no longer blocks (it is grounded).
        a.status = AssumptionStatus::Discharged;
        a.discharge = Some(Discharge {
            resolution: DischargeResolution::AcceptedRisk,
            target: EntityId(3),
            decided_by: "engineer".into(),
        });
        assert!(!a.is_open());
        assert!(!a.is_blocking());

        // Invalidated Critical: no longer blocks either (it is no longer relied upon).
        a.status = AssumptionStatus::Invalidated;
        a.discharge = None;
        assert!(!a.is_blocking());

        // Open but only Normal: surfaced, never blocking.
        let normal = open_assumption("normal presumption", AssumptionCriticality::Normal);
        assert!(normal.is_open());
        assert!(!normal.is_blocking());
    }

    // ================= Band A (increment 2): ModelFidelity =================
    //
    // TDD (written before the object): the trust-tag on a derived fact (Map 6). A
    // `ModelFidelity{concern, method, confidence, scope}` is ADVISORY attached metadata,
    // modeled EXACTLY like `ViolationExplanation` (no `EntityId` of its own, folds into its
    // own store, references a target, never mutates it). Its one domain invariant is a
    // numeric boundary: `confidence ∈ [0,1]`. `validate()` reuses `Inconsistent` (no new
    // `DomainError` variant). `ModelFidelity` carries an `f64`, so — like `Component`/`Net` —
    // it derives `PartialEq` but NOT `Eq`.
    //
    // Delivers exit criterion 2 (every predicted/derived fact carries a fidelity tag).

    fn fidelity(concern: &str, method: FidelityMethod, confidence: f64) -> ModelFidelity {
        ModelFidelity {
            concern: concern.into(),
            method,
            confidence,
            scope: "the 3.3 V rail".into(),
        }
    }

    #[test]
    fn well_formed_fidelity_tag_validates() {
        let f = fidelity(
            "worst-case junction temperature",
            FidelityMethod::FirstOrderFloor,
            0.6,
        );
        assert!(f.validate().is_ok());
    }

    #[test]
    fn fidelity_confidence_zero_and_one_are_valid_boundaries() {
        // The interval is CLOSED: both endpoints are legal (a measured fact may be 1.0; a
        // pure guess may be 0.0). This is the epsilon/boundary case for the numeric invariant.
        let floor = fidelity("assumed", FidelityMethod::Assumed, 0.0);
        assert!(floor.validate().is_ok(), "confidence == 0.0 is valid");
        let ceil = fidelity("measured", FidelityMethod::Measured, 1.0);
        assert!(ceil.validate().is_ok(), "confidence == 1.0 is valid");
    }

    #[test]
    fn fidelity_confidence_above_one_is_inconsistent() {
        // Just past the boundary: 1.0 + epsilon must reject.
        let f = fidelity("over-confident", FidelityMethod::Calculated, 1.000_000_1);
        assert!(matches!(f.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn fidelity_confidence_below_zero_is_inconsistent() {
        let f = fidelity("negative", FidelityMethod::Simulated, -0.000_000_1);
        assert!(matches!(f.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn fidelity_confidence_nan_is_inconsistent() {
        // NaN is neither ≥0 nor ≤1, so it must be rejected (a fidelity tag with an undefined
        // confidence is not honest metadata). Guards the naive `0.0..=1.0` range check that
        // would silently admit NaN.
        let f = fidelity("undefined", FidelityMethod::Calculated, f64::NAN);
        assert!(matches!(f.validate(), Err(DomainError::Inconsistent(_))));
    }

    // ===================== Band A (increment 3): Risk =====================
    //
    // TDD (written before the object): the auditable risk-posture object (Map 46). A `Risk`
    // states a hazard, its `likelihood`/`severity`, a `mitigation`, the `residual` severity
    // that remains after mitigation, the `owner` who is accountable, and a lifecycle
    // (Open -> Mitigated -> Accepted). The HUMAN owns acceptance of residual risk (`00`
    // Principle 11 — humans own goals), which the `AcceptRisk` seam/fold enforces (inc. 3).
    // `validate()` reuses existing `DomainError` variants only (non-empty statement + owner).
    // A `Risk` has no `f64`/`PhysicalQuantity` field, so `Eq` is legal (like `Assumption`).

    fn open_risk(statement: &str, owner: &str, residual: RiskSeverity) -> Risk {
        Risk {
            id: EntityId(1),
            statement: statement.into(),
            likelihood: RiskLikelihood::Medium,
            severity: RiskSeverity::High,
            mitigation: "add a TVS diode on the USB-C VBUS".into(),
            residual,
            owner: owner.into(),
            status: RiskStatus::Open,
        }
    }

    #[test]
    fn risk_rejects_empty_statement() {
        let r = open_risk("   ", "hardware lead", RiskSeverity::Low);
        assert_eq!(r.validate(), Err(DomainError::EmptyStatement));
    }

    #[test]
    fn risk_rejects_empty_owner() {
        // A risk with no accountable owner cannot be owned or accepted (Principle 11) — reject it.
        let r = open_risk("ESD on the exposed connector", "  ", RiskSeverity::Medium);
        assert_eq!(r.validate(), Err(DomainError::EmptyField("risk owner")));
    }

    #[test]
    fn well_formed_open_risk_validates() {
        let r = open_risk(
            "an ESD strike on the USB-C connector could latch up the MCU",
            "hardware lead",
            RiskSeverity::Low,
        );
        assert!(r.validate().is_ok());
    }

    #[test]
    fn risk_validate_is_status_agnostic() {
        // validate() checks only the intrinsic invariants (non-empty statement + owner); the
        // Open->Accepted transition and the human-acceptance authority live at the seam/fold,
        // exactly as Assumption's discharge lifecycle does.
        let mut r = open_risk(
            "thermal runaway under sustained load",
            "thermal owner",
            RiskSeverity::Critical,
        );
        r.status = RiskStatus::Mitigated;
        assert!(r.validate().is_ok());
        r.status = RiskStatus::Accepted;
        assert!(r.validate().is_ok());
    }

    // ============ Band A (increment 4): Objective / Tradeoff domain invariants ============
    //
    // TDD (RED before the objects exist): the weighed-and-rejected space, preserved (Map 11,
    // exit criterion 3). An `Objective` is a weighted goal rooted in a source; a `Tradeoff`
    // records the alternatives considered, the criteria they were scored on, the chosen index,
    // and — critically — PRESERVES the rejected space (`00` Principle 7 honesty: the design
    // remembers what it did NOT choose and why). `validate()` reuses only existing DomainError
    // variants (`EmptyStatement`/`EmptyField`/`Inconsistent`) — no new variant is invented.

    /// A well-formed 3-alternative tradeoff: `chosen == 0` is not rejected; two rejected
    /// alternatives are preserved. Helper so each rejection test mutates one facet at a time.
    fn well_formed_tradeoff() -> Tradeoff {
        Tradeoff {
            id: EntityId(20),
            question: "Which regulator topology for the 3.3 V rail?".into(),
            alternatives: vec![
                Alternative {
                    label: "buck".into(),
                    description: "switching buck converter".into(),
                    scores: vec![0.9, 0.6],
                    rejected: false,
                },
                Alternative {
                    label: "ldo".into(),
                    description: "linear low-dropout regulator".into(),
                    scores: vec![0.4, 0.9],
                    rejected: true,
                },
                Alternative {
                    label: "charge-pump".into(),
                    description: "switched-capacitor charge pump".into(),
                    scores: vec![0.3, 0.5],
                    rejected: true,
                },
            ],
            criteria: vec!["efficiency".into(), "simplicity".into()],
            chosen: 0,
            rationale: "efficiency dominates at this load; the LDO's droop is unacceptable".into(),
            decided_by: "hardware lead".into(),
        }
    }

    #[test]
    fn well_formed_objective_validates() {
        let o = Objective {
            id: EntityId(19),
            statement: "minimize board area".into(),
            weight: 0.7,
            source: EntityId(3),
        };
        assert!(o.validate().is_ok());
    }

    #[test]
    fn objective_rejects_empty_statement() {
        let o = Objective {
            id: EntityId(19),
            statement: "   ".into(),
            weight: 0.7,
            source: EntityId(3),
        };
        assert_eq!(o.validate(), Err(DomainError::EmptyStatement));
    }

    #[test]
    fn well_formed_tradeoff_validates() {
        assert!(well_formed_tradeoff().validate().is_ok());
    }

    #[test]
    fn tradeoff_rejects_fewer_than_two_alternatives() {
        // A "tradeoff" with a single option weighed nothing — it is not a tradeoff (P7).
        let mut t = well_formed_tradeoff();
        t.alternatives = vec![Alternative {
            label: "buck".into(),
            description: "the only option".into(),
            scores: vec![0.9],
            rejected: false,
        }];
        t.chosen = 0;
        assert!(matches!(t.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn tradeoff_rejects_chosen_index_out_of_range() {
        let mut t = well_formed_tradeoff();
        t.chosen = 3; // len is 3, so index 3 is out of range.
        assert!(matches!(t.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn tradeoff_rejects_a_chosen_alternative_that_is_marked_rejected() {
        // The design cannot both choose an alternative and mark it rejected (self-contradiction).
        let mut t = well_formed_tradeoff();
        t.chosen = 1; // index 1 is the LDO, which is rejected == true.
        assert!(matches!(t.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn tradeoff_rejects_when_no_rejected_alternative_is_preserved() {
        // The whole point of the object is to PRESERVE the rejected space (exit criterion 3). A
        // tradeoff where every alternative is un-rejected has thrown that space away.
        let mut t = well_formed_tradeoff();
        for alt in &mut t.alternatives {
            alt.rejected = false;
        }
        t.chosen = 0;
        assert!(matches!(t.validate(), Err(DomainError::Inconsistent(_))));
    }

    // ===================== Band B (increment 1): PowerDomain =====================
    //
    // TDD: a power rail is self-consistent even when overloaded — `validate()` guards the
    // rail's own invariants (name/voltage/current/nets), and the power-balance RULE (in
    // eak-engines) judges whether the design is right. So validate() rejects a blank name,
    // a non-positive voltage, a non-positive max current, and a rail powering nothing —
    // but accepts a well-formed rail whose load may still exceed its source (that is a
    // design error the rule reports, not a malformed object).

    fn well_formed_power_domain() -> PowerDomain {
        PowerDomain {
            id: EntityId(30),
            name: "3V3".into(),
            voltage: PhysicalQuantity::new(3.3, Unit::Volt),
            source_component: EntityId(4),
            max_current: PhysicalQuantity::new(1.0, Unit::Ampere),
            nets: vec![EntityId(31), EntityId(32)],
        }
    }

    #[test]
    fn power_domain_rejects_blank_name() {
        let mut d = well_formed_power_domain();
        d.name = "  ".into();
        assert_eq!(
            d.validate(),
            Err(DomainError::EmptyField("power domain name"))
        );
    }

    #[test]
    fn power_domain_rejects_non_positive_voltage() {
        let mut d = well_formed_power_domain();
        d.voltage = PhysicalQuantity::new(0.0, Unit::Volt);
        assert!(matches!(d.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn power_domain_rejects_non_positive_max_current() {
        let mut d = well_formed_power_domain();
        d.max_current = PhysicalQuantity::new(-0.5, Unit::Ampere);
        assert!(matches!(d.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn power_domain_rejects_empty_nets() {
        let mut d = well_formed_power_domain();
        d.nets = vec![];
        assert!(matches!(d.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn well_formed_power_domain_validates_even_if_overloaded() {
        // The rail is well-formed; whether it is overloaded is the rule's judgement, not
        // validate()'s (a power-balance violation is a design finding, not a malformed object).
        assert!(well_formed_power_domain().validate().is_ok());
    }

    // ----------------- Band B (increment 2): ClockDomain -----------------
    // The clock-domain invariants mirror the power domain's: a non-empty name, a finite positive
    // frequency, at least one member net — and a well-formed domain whose members conflict with
    // another domain's membership is the RULE's judgement (clock-domain conflict), not validate()'s.

    fn well_formed_clock_domain() -> ClockDomain {
        ClockDomain {
            id: EntityId(40),
            name: "SYS".into(),
            frequency: PhysicalQuantity::new(48.0, Unit::Megahertz),
            source_component: EntityId(4),
            members: vec![EntityId(41), EntityId(42)],
        }
    }

    #[test]
    fn clock_domain_rejects_blank_name() {
        let mut d = well_formed_clock_domain();
        d.name = "  ".into();
        assert_eq!(
            d.validate(),
            Err(DomainError::EmptyField("clock domain name"))
        );
    }

    #[test]
    fn clock_domain_rejects_non_positive_frequency() {
        let mut d = well_formed_clock_domain();
        d.frequency = PhysicalQuantity::new(0.0, Unit::Megahertz);
        assert!(matches!(d.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn clock_domain_rejects_non_finite_frequency() {
        let mut d = well_formed_clock_domain();
        d.frequency = PhysicalQuantity::new(f64::NAN, Unit::Megahertz);
        assert!(matches!(d.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn clock_domain_rejects_empty_members() {
        let mut d = well_formed_clock_domain();
        d.members = vec![];
        assert!(matches!(d.validate(), Err(DomainError::Inconsistent(_))));
    }

    #[test]
    fn well_formed_clock_domain_validates() {
        assert!(well_formed_clock_domain().validate().is_ok());
    }
}
