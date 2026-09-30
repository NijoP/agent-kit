# Verification Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines the engineering verification framework: electrical rules, power integrity, clock distribution, signal integrity, return paths, DFM, EMC, confidence/fidelity scoring, and human review gates.

---

## Design Principles

1. **Deterministic Rules** — All verification rules are pure functions in `eak-engines`. Same input → same output.
2. **Confidence & Fidelity** — Every result carries `confidence` (0.0–1.0) and `fidelity` (Analytical, Simulation, Measured).
3. **Provenance** — Verification results are events with full provenance (rule version, inputs, model).
4. **Human-in-the-Loop** — Critical failures gate on human approval; warnings are advisory.
5. **Incremental** — Verification runs on relevant subset after each change; full run on release.
6. **Extensible** — New rules added via `VerificationRule` trait; no kernel changes.

---

## Verification Kinds (V1)

| Kind | Description | Engine | Fidelity |
|------|-------------|--------|----------|
| **ERC** | Electrical Rule Check (schematic) | `SchematicEngine` | Analytical |
| **DRC** | Design Rule Check (PCB) | `PcbEngine` | Analytical |
| **Power** | Rail analysis, decoupling, current loops | `PowerEngine` | Analytical → Simulation |
| **Clock** | Tree distribution, skew, jitter, termination | `ClockEngine` | Analytical → Simulation |
| **SI** | Impedance, reflections, crosstalk, eye diagrams | `SiEngine` | Simulation |
| **Return Path** | Ground continuity, stitching vias, layer transitions | `ReturnPathEngine` | Analytical |
| **EMC** | Emission/susceptibility risk assessment | `EmcEngine` | Analytical |
| **DFM** | Manufacturability (min trace, space, drill, annular, mask) | `DfmEngine` | Analytical |

---

## Verification Engine Framework (`eak-engines`)

### Rule Trait
```rust
pub trait VerificationRule: Send + Sync {
    fn rule_id(&self) -> RuleId;
    fn kind(&self) -> VerificationKind;
    fn severity(&self) -> Severity;        // Error, Warning, Info
    fn fidelity(&self) -> FidelityLevel;   // Analytical, Simulation, Measured
    fn evaluate(&self, model: &EngineeringModel) -> Result<Vec<CheckResult>, RuleError>;
}
```

### Engine Trait
```rust
pub trait VerificationEngine: Send + Sync {
    fn kind(&self) -> VerificationKind;
    fn rules(&self) -> Vec<&dyn VerificationRule>;
    fn run(&self, model: &EngineeringModel) -> Result<VerificationResult, EngineError>;
}
```

### Verification Result
```rust
struct VerificationResult {
    kind: VerificationKind,
    status: VerificationStatus,           // Pass, Fail, Warning, Info
    checks: Vec<CheckResult>,
    confidence: f32,                      // Aggregated confidence
    fidelity: FidelityLevel,              // Highest fidelity among checks
    runtime_ms: u64,
    provenance: Provenance,
}

struct CheckResult {
    rule_id: RuleId,
    description: String,
    severity: Severity,
    locations: Vec<EntityId>,             // Entities involved
    expected: Option<Quantity>,           // Typed with units
    actual: Option<Quantity>,
    message: String,
    confidence: f32,
    fidelity: FidelityLevel,
}
```

---

## Confidence & Fidelity Model

### Confidence (0.0–1.0)
- **1.0** — Deterministic analytical rule, exact match (e.g., DRC clearance).
- **0.9** — Analytical with conservative margins.
- **0.8** — Simulation with calibrated models.
- **0.7** — Simulation with estimated models.
- **0.6** — Heuristic/empirical rule.
- **0.5** — AI-assisted estimation (flagged).

### Fidelity Levels
| Level | Description | Example |
|-------|-------------|---------|
| **Analytical** | Closed-form equations, geometric rules | DRC, ERC, basic power |
| **Simulation** | SPICE, field solver, IBIS-AMI | SI eye diagram, power integrity |
| **Measured** | Lab measurement on prototype | Post-silicon validation |

**Rule**: Kernel only runs Analytical fidelity by default. Simulation/Measured require explicit `VerificationRequest` with `fidelity_requirement`.

---

## Verification Capability Seam

```rust
struct VerificationRequest {
    kind: VerificationKind,
    scope: VerificationScope,          // Full, Incremental(entity_ids), Differential(rev_a, rev_b)
    fidelity_requirement: FidelityLevel,
    target_confidence: f32,            // Minimum confidence for pass
}

struct VerificationResponse {
    result: VerificationResult,
    approval_gate: Option<ApprovalGate>, // If status == Fail && severity == Error
}
```

### Validation Function (`validate_verification_request`)
1. Checks that `kind` is supported.
2. Checks that model has required data (e.g., PCB routes for SI).
3. Checks that `fidelity_requirement` is available (simulation engines installed).
4. Returns `ValidationError` if not ready.

---

## Incremental Verification

### Trigger
- After each `SchematicEdit`/`PcbEdit` capability commit.
- Kernel computes affected entities (dirty tracking).
- Runs only rules touching dirty entities.

### Dirty Tracking
```rust
struct DirtyTracker {
    dirty_entities: HashSet<EntityId>,
    dirty_rules: HashSet<RuleId>,
}

impl VerificationEngine {
    fn incremental(&self, model: &EngineeringModel, dirty: &DirtyTracker) -> VerificationResult;
}
```

### Differential Verification
- Between two revisions: `VerificationScope::Differential(rev_a, rev_b)`.
- Runs only rules where inputs changed.
- Produces `VerificationDiff` (new passes, new failures, changed confidence).

---

## Human Review Gates

### Gate Trigger
- Any `CheckResult` with `severity == Error` AND `confidence > 0.9`.
- Any `VerificationResult` with `status == Fail` for critical kinds (Power, Clock, DFM).

### Gate Representation
```rust
struct ApprovalGate {
    id: EntityId,
    verification_result_id: EntityId,
    required_approvers: Vec<Approver>,   // Human, LeadAgent
    status: ApprovalStatus,              // Pending, Approved, Rejected, Waived
    waiver_reason: Option<String>,       // If Waived
    expires_at: Option<DateTime<Utc>>,   // Auto-escalate
}
```

### Waiver Process
1. Engineer reviews failure in Verification Dashboard.
2. If acceptable (e.g., known false positive), creates waiver with reason.
3. Waiver recorded as event with `ProvenanceSource::Human`.
4. Kernel treats waived check as `Pass` for release gating.

---

## Rule Catalog (Selected V1 Rules)

### ERC (Schematic)
| Rule ID | Description | Severity |
|---------|-------------|----------|
| `erc.unconnected_pin` | Input/power pin not connected | Error |
| `erc.power_pin_not_driven` | Power net has no source | Error |
| `erc.short_power_ground` | Power net shorted to ground | Error |
| `erc.pin_type_conflict` | Output connected to output | Error |
| `erc.unannotated` | Component missing reference designator | Warning |

### DRC (PCB)
| Rule ID | Description | Severity |
|---------|-------------|----------|
| `drc.clearance` | Trace/trace, trace/pad, pad/pad < min | Error |
| `drc.min_trace_width` | Trace width < min for current | Error |
| `drc.min_drill` | Via/hole drill < fabricator min | Error |
| `drc.annular_ring` | Annular ring < min | Error |
| `drc.solder_mask_sliver` | Mask sliver < min | Warning |
| `drc.acid_trap` | Acute angle trap | Warning |

### Power
| Rule ID | Description | Severity |
|---------|-------------|----------|
| `pwr.rail_voltage_drop` | IR drop > 5% at max current | Error |
| `pwr.decoupling_capacitance` | Total decap < target per rail | Warning |
| `pwr.current_loop_area` | High-current loop area > threshold | Warning |
| `pwr.thermal_hotspot` | Component junction temp > max | Error |

### Clock
| Rule ID | Description | Severity |
|---------|-------------|----------|
| `clk.skew` | Clock skew > spec | Error |
| `clk.jitter` | Total jitter > UI | Error |
| `clk.termination` | Missing/incorrect termination | Warning |
| `clk.crosstalk` | Clock coupling to sensitive nets | Warning |

### SI (Post-Layout)
| Rule ID | Description | Severity |
|---------|-------------|----------|
| `si.impedance` | Single-ended/diff impedance out of tolerance | Error |
| `si.reflection` | Reflection coefficient > threshold | Warning |
| `si.crosstalk` | Near-end/far-end crosstalk > threshold | Warning |
| `si.eye_diagram` | Eye height/width < spec | Error |

### Return Path
| Rule ID | Description | Severity |
|---------|-------------|----------|
| `ret.ground_cut` | Signal crosses ground plane cut | Error |
| `ret.stitching_via` | Layer transition missing stitching via | Warning |
| `ret.return_path_discontinuity` | Impedance discontinuity at transition | Warning |

### EMC
| Rule ID | Description | Severity |
|---------|-------------|----------|
| `emc.loop_antenna` | Large current loop area | Warning |
| `emc.dipole_antenna` | Untermminated stub > λ/20 | Warning |
| `emc.edge_rate` | Fast edges on unshielded connectors | Warning |

### DFM
| Rule ID | Description | Severity |
|---------|-------------|----------|
| `dfm.min_trace` | Trace width < fabricator min | Error |
| `dfm.min_space` | Trace spacing < fabricator min | Error |
| `dfm.min_drill` | Drill size < fabricator min | Error |
| `dfm.annular_ring` | Annular ring < fabricator min | Error |
| `dfm.solder_mask` | Mask clearance < fabricator min | Error |
| `dfm.copper_balance` | Layer copper balance < 30% | Warning |

---

## Simulation Integration (Future)

### SPICE Interface
```rust
trait SpiceEngine {
    fn simulate(&self, netlist: SpiceNetlist, analysis: SpiceAnalysis) -> Result<SpiceResult, SpiceError>;
}
```

### Field Solver Interface
```rust
trait FieldSolver {
    fn impedance(&self, geometry: CrossSection) -> Result<ImpedanceResult, SolverError>;
    fn s_parameters(&self, geometry: ViaTransition, freq_range: Range<f64>) -> Result<SParamResult, SolverError>;
}
```

### Adapters (Pluggable)
- **ngspice** (open source) — via subprocess.
- **LTspice** — via CLI (license review needed).
- **OpenEMS** — open source EM solver.
- **Commercial** (Ansys HFSS, Keysight ADS) — via API (enterprise).

---

## Verification Dashboard (UI)

### Views
1. **Summary Cards** — Pass/Fail/Warning counts per kind, overall confidence.
2. **Check Table** — Sortable, filterable by kind, severity, confidence, entity.
3. **Provenance Graph** — Click check → see rule, inputs, model version.
4. **Trend** — Verification status over revisions (sparkline).
5. **Waiver Log** — All waivers with reasons, approvers, timestamps.

---

## Next Document

[Project/Persistence Architecture →](../12-project-persistence-architecture/README.md)