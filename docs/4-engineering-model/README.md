# Engineering Model: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines the canonical engineering data model — the single source of truth for all engineering facts in EAK. Every entity, relationship, and attribute is defined here with its provenance semantics.

---

## Core Principles

1. **Single Source of Truth** — The kernel's folded state is the only authoritative model.
2. **Provenance on Every Fact** — Each attribute knows: source (human, datasheet, AI, derived), timestamp, agent, confidence.
3. **Typed Physical Quantities** — No raw floats; all quantities carry units, tolerances, conditions (P9: `eak-units`).
4. **Imported vs. Synthesized** — Clear lineage distinction (P7).
5. **Replayable** — Event log + deterministic fold = identical model reconstruction (P8).
6. **Opaque Identifiers** — `EntityId` (ULID) for all entities; no natural keys in kernel.

---

## Entity Catalog

### 1. Project
```rust
struct Project {
    id: EntityId,
    name: String,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    revision: Revision,
    settings: ProjectSettings,
}
```
- Root aggregate. Contains all other entities via relationships.
- `Revision` tracks snapshot history.

### 2. Revision
```rust
struct Revision {
    id: EntityId,
    project_id: EntityId,
    parent_revision_id: Option<EntityId>,
    event_seq: u64,           // Event log sequence at snapshot
    label: String,            // Human-readable (e.g., "v1.0-rc1")
    created_at: DateTime<Utc>,
    author: Provenance,
}
```
- Immutable snapshot of the entire project at an event sequence.
- Enables branching, merging, time-travel.

### 3. Requirement
```rust
struct Requirement {
    id: EntityId,
    project_id: EntityId,
    kind: RequirementKind,    // Functional, Performance, Environmental, Regulatory, Derived
    text: String,             // Natural language
    structured: Option<StructuredRequirement>, // Parsed IR
    priority: Priority,
    status: RequirementStatus, // Proposed, Accepted, Verified, Deferred
    traceability: Vec<EntityId>, // Links to components, nets, verification results
    provenance: Provenance,
}
```
- Decomposed from intent by Requirement Agent.
- `StructuredRequirement` is the compiler IR (see `eak-compiler`).

### 4. Component
```rust
struct Component {
    id: EntityId,
    project_id: EntityId,
    mfr_part_number: String,
    supplier_part_numbers: HashMap<Supplier, String>,
    category: ComponentCategory,
    pins: Vec<Pin>,
    electrical: ElectricalSpecs,
    mechanical: MechanicalSpecs,
    thermal: ThermalSpecs,
    packaging: PackagingInfo,
    datasheet_refs: Vec<EvidenceRef>,
    lifecycle: ComponentLifecycle, // Suggested, Selected, Verified, Approved, Procured, Obsolete
    provenance: Provenance,
    confidence: f32,          // 0.0–1.0, aggregated from evidence
}
```
- **Pins** define electrical interface (number, name, type, direction).
- **ElectricalSpecs** use `eak-units` typed quantities (voltage, current, power, frequency, impedance, etc.).
- **EvidenceRef** points to ingested datasheet facts with page/table/figure location.

### 5. Pin
```rust
struct Pin {
    number: String,
    name: String,
    pin_type: PinType,        // Power, Ground, Input, Output, Bidirectional, Analog, RF, NC
    electrical: PinElectrical, // Voltage range, current drive, capacitance, ESD
    provenance: Provenance,
}
```

### 6. Net
```rust
struct Net {
    id: EntityId,
    project_id: EntityId,
    name: String,
    kind: NetKind,            // Signal, Power, Ground, Differential, Analog, RF
    pins: Vec<PinRef>,        // Connected pins (component_id, pin_number)
    properties: NetProperties, // Impedance target, length match group, voltage domain
    provenance: Provenance,
}
```
- Net connectivity is the backbone of schematic and PCB.

### 7. Schematic Representation
```rust
struct Schematic {
    id: EntityId,
    project_id: EntityId,
    sheets: Vec<SchematicSheet>,
    netlist: Netlist,         // Derived from sheets + pins
    erc_results: Option<ERCReport>,
    provenance: Provenance,
}

struct SchematicSheet {
    id: EntityId,
    name: String,
    elements: Vec<SchematicElement>, // ComponentInstance, Wire, Label, Port, Text, Image
    child_sheet_refs: Vec<EntityId>, // Hierarchical
}
```

### 8. SchematicElement
```rust
enum SchematicElement {
    ComponentInstance(ComponentInstance),
    Wire(Wire),
    NetLabel(NetLabel),
    HierarchicalPort(HierarchicalPort),
    Text(Text),
    Image(Image),
}

struct ComponentInstance {
    id: EntityId,
    component_id: EntityId,   // References Component
    position: Point,          // mm, typed
    orientation: Rotation,
    properties: HashMap<String, String>, // Overrides (e.g., value, footprint)
    provenance: Provenance,
}
```

### 9. PCB Representation
```rust
struct Pcb {
    id: EntityId,
    project_id: EntityId,
    board_outline: Polygon,
    stackup: Stackup,
    placement: Vec<PcbPlacement>,
    routing: Vec<PcbRoute>,
    zones: Vec<PcbZone>,
    vias: Vec<Via>,
    drc_results: Option<DRCReport>,
    provenance: Provenance,
}

struct PcbPlacement {
    component_instance_id: EntityId, // References SchematicElement::ComponentInstance
    layer: Layer,
    position: Point3D,               // x, y, rotation
    locked: bool,
    provenance: Provenance,
}

struct PcbRoute {
    net_id: EntityId,
    layer: Layer,
    geometry: RouteGeometry,         // Segments, arcs
    constraints: RouteConstraints,   // Impedance, length, diff pair
    provenance: Provenance,
}
```

### 10. Verification Result
```rust
struct VerificationResult {
    id: EntityId,
    project_id: EntityId,
    kind: VerificationKind,         // ERC, DRC, Power, Clock, SI, EMC, DFM
    status: VerificationStatus,     // Pass, Fail, Warning, Info
    checks: Vec<CheckResult>,
    confidence: f32,                // Fidelity of the analysis
    fidelity: FidelityLevel,        // Analytical, Simulation, Measured
    provenance: Provenance,
}

struct CheckResult {
    rule_id: String,
    description: String,
    severity: Severity,
    locations: Vec<EntityId>,       // Entities involved
    expected: Option<Quantity>,
    actual: Option<Quantity>,
    message: String,
}
```

### 11. Manufacturing Outputs
```rust
struct Bom {
    id: EntityId,
    project_id: EntityId,
    lines: Vec<BomLine>,
    provenance: Provenance,
}

struct BomLine {
    component_id: EntityId,
    quantity: u32,
    reference_designators: Vec<String>,
    supplier_options: Vec<SupplierOption>,
    lifecycle: ComponentLifecycle,
    provenance: Provenance,
}

struct ManufacturingPackage {
    id: EntityId,
    project_id: EntityId,
    gerber_files: Vec<GerberFile>,
    drill_files: Vec<DrillFile>,
    pick_and_place: PickAndPlaceFile,
    assembly_drawings: Vec<AssemblyDrawing>,
    provenance: Provenance,
}
```

### 12. Evidence (Datasheet Facts)
```rust
struct Evidence {
    id: EntityId,
    project_id: EntityId,
    source: EvidenceSource,         // Datasheet, AppNote, TestReport, Standard, Web
    document_ref: DocumentRef,      // URL, local path, hash
    location: DocumentLocation,     // Page, table, figure, paragraph
    facts: Vec<ExtractedFact>,
    extraction_method: ExtractionMethod, // Manual, OCR, LLM, Parser
    provenance: Provenance,
}

struct ExtractedFact {
    attribute: String,              // e.g., "max_supply_voltage"
    value: Quantity,                // Typed with units
    conditions: Vec<Condition>,     // Temp, voltage, frequency
    confidence: f32,
}
```

---

## Provenance Model

```rust
struct Provenance {
    source: ProvenanceSource,       // Human, Agent, Import, Derived, Fixture
    agent_id: Option<String>,       // If Agent: which agent (role + instance)
    timestamp: DateTime<Utc>,
    event_seq: u64,                 // Kernel event sequence
    confidence: f32,                // 0.0–1.0
    derivation_chain: Vec<EntityId>, // For Derived facts: chain of sources
}

enum ProvenanceSource {
    Human,
    Agent { role: AgentRole, instance: String },
    Import { format: String, tool: String },
    Derived { rule: String },
    Fixture,
}
```

**Imported vs. Synthesized** — `Import` source means the fact came from an external file (datasheet, KiCad, CSV). `Derived` means it was computed by a deterministic kernel rule. `Agent` means an AI proposed it (and kernel validated).

---

## Relationships (Graph View)

```
Project
  ├─ Revision* (history)
  ├─ Requirement* (requirements)
  ├─ Component* (library)
  ├─ Net* (connectivity)
  ├─ Schematic* (one per project, hierarchical)
  ├─ Pcb* (one per project)
  ├─ VerificationResult* (per kind)
  ├─ Bom* (one per revision)
  ├─ ManufacturingPackage* (per release)
  └─ Evidence* (per component/requirement)
```

All relationships are **explicit edges** with provenance. No implicit foreign keys.

---

## Quantities and Units (`eak-units`)

All physical quantities use the `Quantity<U>` type where `U` is a unit marker (e.g., `Volt`, `Ampere`, `Ohm`, `Farad`, `Meter`, `Second`, `Hertz`, `Celsius`, `Watt`).

```rust
// Examples
let voltage = Quantity::new(3.3, Volt);           // 3.3 V
let current = Quantity::new(500, Milli<Ampere>);  // 500 mA
let impedance = Quantity::new(50, Ohm);           // 50 Ω
let capacitance = Quantity::new(100, Nano<Farad>); // 100 nF
let length = Quantity::new(1.6, Milli<Meter>);    // 1.6 mm
```

**Tolerances and conditions** are first-class:
```rust
struct SpecWithTolerance {
    nominal: Quantity<U>,
    tolerance: Tolerance,        // ±5%, ±0.1V, etc.
    conditions: Vec<Condition>,  // e.g., Temp(-40..125°C), Vcc(3.0..3.6V)
}
```

---

## Model Evolution (Schema Versioning)

- Each entity struct has a `schema_version: u32`.
- Kernel migrations are deterministic functions `migrate_vN_to_vN+1: State -> State`.
- Migrations are part of the fold; replay applies them in order.
- New fields are optional with sensible defaults.

---

## Next Document

[Component Architecture →](../5-component-architecture/README.md)