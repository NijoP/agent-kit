# KiCad Integration Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines KiCad as an **EDA Implementation Peripheral** — an edge integration for schematic/PCB editing, viewing, import/export, and rendering. EAK state remains canonical. KiCad files are never the source of truth.

---

## Design Principles

1. **EAK Canonical Model is Source of Truth** — KiCad files are import/export formats only.
2. **No KiCad Data Model in Kernel** — Do not copy KiCad's data structures into `eak-domain` or `eak-runtime`.
3. **Bidirectional Sync with Fidelity Tracking** — Round-trip for supported subset; loss flagged as `Import` provenance.
4. **KiCad as Editing Frontend** — Engineer edits in KiCad GUI; changes sync back via `KicadSync` capability.
5. **Headless Rendering** — KiCad CLI for automated PNG/PDF/3D/STEP export.
6. **License Compliance** — KiCad is GPLv3. EAK integrates via CLI/subprocess (not linking). No GPL code in EAK kernel.

---

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        EAK KERNEL                               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    KicadSync Capability                   │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │   │
│  │  │  Import     │  │  Export     │  │  Render     │       │   │
│  │  │  (.kicad_*) │  │  (.kicad_*) │  │  (CLI)      │       │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      KICAD ADAPTER (eak-kicad)                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Parser     │  │  Generator  │  │  CLI Driver │             │
│  │  (sexp)     │  │  (sexp)     │  │  (subprocess)           │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
       ┌────────────┐  ┌────────────┐  ┌────────────┐
       │ .kicad_sch │  │ .kicad_pcb │  │  Assets    │
       │  (sch)     │  │  (pcb)     │  │  (sym, fp, │
       └────────────┘  └────────────┘  │  3d)       │
                                       └────────────┘
```

---

## Port Trait (`eak-ports`)

```rust
#[async_trait]
pub trait KicadSync: Send + Sync {
    /// Import KiCad project into EAK canonical model
    async fn import(&self, project_path: &Path) -> Result<ImportResult, KicadError>;

    /// Export EAK canonical model to KiCad project
    async fn export(&self, model: &EngineeringModel, project_path: &Path) -> Result<ExportResult, KicadError>;

    /// Render schematic/PCB to PNG/PDF/STEP via KiCad CLI
    async fn render(&self, project_path: &Path, spec: RenderSpec) -> Result<RenderOutput, KicadError>;

    /// Watch for file changes (for live sync when engineer edits in KiCad)
    async fn watch(&self, project_path: &Path) -> Result<ChangeStream, KicadError>;
}
```

---

## Import: KiCad → EAK Canonical Model

### Supported Subset (V1)
| KiCad Element | EAK Mapping | Fidelity |
|---------------|-------------|----------|
| Schematic sheets (hierarchical) | `Schematic.sheets` | High |
| Components (symbol instances) | `ComponentInstance` + `Component` reference | High |
| Wires, buses, junctions | `Wire` + `Net` connectivity | High |
| Net labels (local, global, hierarchical) | `NetLabel`, `HierarchicalPort` | High |
| Text, graphics | `Text`, `Image` | Medium |
| Component fields (value, footprint, datasheet) | `ComponentInstance.properties` | High |
| PCB board outline | `Pcb.board_outline` | High |
| PCB layers, stackup | `Pcb.stackup` | High |
| Footprints (placement, orientation) | `PcbPlacement` | High |
| Tracks, vias, zones | `PcbRoute`, `Via`, `PcbZone` | High |
| Footprint libraries (local) | `Asset(Footprint)` | High |
| Symbol libraries (local) | `Asset(Symbol)` | High |

### Unsupported / Lossy (Flagged)
- KiCad-specific UUIDs → mapped to EAK `EntityId` (new IDs generated).
- KiCad-only properties (e.g., `uuid`, `timestamp`) → dropped, recorded in `ImportProvenance`.
- Custom schematic graphics → imported as `Image` with low fidelity.
- 3D model references without local file → `Asset(Model3D)` with `Missing` status.

### Import Provenance
Every imported entity gets `ProvenanceSource::Import { format: "kicad", tool: "kicad-v8.0" }`.
Lossy conversions generate `ImportWarning` events (non-blocking).

### Import Algorithm
1. Parse `.kicad_pro` → get project name, schematic/pcb file list.
2. Parse `.kicad_sch` (S-expression) → build schematic model.
3. Parse `.kicad_pcb` (S-expression) → build PCB model.
4. Cross-reference: match schematic components to PCB footprints via reference designator.
5. For each component, look up/create `Component` in EAK library (by MPN).
6. Emit `ImportResult` with canonical model + warnings.

---

## Export: EAK Canonical Model → KiCad

### Export Algorithm
1. Create project directory with `.kicad_pro`, `.kicad_sch`, `.kicad_pcb`.
2. Serialize schematic sheets → S-expression (KiCad 8 format).
3. Serialize PCB → S-expression (board outline, stackup, placements, routes, zones, vias).
4. Write symbol library (`.kicad_sym`) for all referenced components.
5. Write footprint library (`.kicad_mod`) for all referenced footprints.
6. Copy 3D models to `models3d/` directory.
7. Write `.kicad_pro` with library references.

### Fidelity Guarantees
- **Round-trip**: `EAK → KiCad → EAK` produces semantically equivalent model (modulo UUIDs).
- **Unsupported EAK features** (e.g., advanced verification results) → exported as KiCad comments or custom properties.

---

## Render: Headless KiCad CLI

### Render Spec
```rust
struct RenderSpec {
    kind: RenderKind,                // SchematicPng, SchematicPdf, PcbPng, PcbPdf, Pcb3dStep, Pcb3dVrml
    layers: Vec<Layer>,              // For PCB: F.Cu, B.Cu, F.SilkS, etc.
    dpi: u32,                        // 300 default
    output_path: PathBuf,
    theme: Option<RenderTheme>,      // KiCad color theme
}
```

### Use Cases
- **UI Thumbnails**: Tauri app requests PNG for schematic/PCB tabs.
- **Documentation**: Auto-generate PDF for design reviews.
- **Manufacturing**: STEP export for mechanical integration.
- **CI**: Visual regression testing (golden images).

---

## Live Sync: Engineer Edits in KiCad → EAK

### Flow
1. Engineer opens KiCad project (launched by Tauri UI via `KicadSync` capability).
2. EAK starts `watch()` on project directory (using `notify` crate).
3. Engineer makes changes, saves.
4. `watch()` detects file modification → triggers incremental re-import.
5. Incremental import computes diff → proposes `SchematicEdit`/`PcbEdit` capability invocations.
6. Kernel validates → commits if valid, or returns validation errors.
7. Engineer sees validation feedback in Tauri UI (or KiCad plugin future).

### Conflict Resolution
- If engineer edits conflict with kernel state (e.g., another agent modified same net), kernel rejects with `ConflictError`.
- UI shows diff; engineer chooses: **Keep Kernel**, **Keep KiCad**, **Merge**.

---

## Asset Libraries (Symbols, Footprints, 3D)

### EAK-Managed Libraries
EAK maintains KiCad-compatible library directories:
```
<project>/
├── lib/
│   ├── symbols/          # .kicad_sym files (one per component)
│   ├── footprints/       # .kicad_mod files (one per footprint)
│   └── models3d/         # .step/.wrl files
├── <project>.kicad_pro
├── <project>.kicad_sch
└── <project>.kicad_pcb
```

### Library Sync
- On `ComponentApprove` capability commit → symbol/footprint/3D written to `lib/`.
- On `Asset` import from KiCad → added to `lib/` with `Import` provenance.
- Tauri UI provides "Open in KiCad" button that sets `KICAD_SYMBOL_DIR` and `KICAD_FOOTPRINT_DIR` to project `lib/`.

---

## Capability: `KicadSync`

### Validation Function (`validate_kicad_sync`)
1. **Import**: Validates that parsed model passes ERC/DRC (optional, configurable).
2. **Export**: Validates that EAK model is exportable (no missing footprints, valid stackup).
3. **Render**: Validates that KiCad CLI is available and project path exists.
4. **Watch**: Validates that project directory exists and is writable.

### Proposal Schema
```json
{
  "type": "object",
  "properties": {
    "action": {"enum": ["import", "export", "render", "watch"]},
    "project_path": {"type": "string"},
    "options": {"type": "object"}
  },
  "required": ["action", "project_path"]
}
```

---

## KiCad Version Compatibility

| KiCad Version | Support | Notes |
|---------------|---------|-------|
| 7.0 | Deprecated | Import only, no live sync |
| 8.0 | Full | Primary target |
| 9.0 (future) | Planned | Track upstream |

EAK pins to KiCad 8 S-expression format. Parser/generator are versioned.

---

## Licensing Compliance

- **KiCad**: GPLv3+
- **EAK Kernel**: MIT/Apache-2.0 (permissive)
- **Integration Method**: Subprocess/CLI only. No linking to KiCad libraries.
- **Adapter Crate (`eak-kicad`)**: Also MIT/Apache-2.0. It shells out to `kicad-cli`.
- **No GPL Code in EAK Workspace** — Verified by `cargo-deny` in CI.

---

## Testing

- **Golden Files**: Sample KiCad projects → import → export → compare S-expressions (normalized).
- **Round-trip**: EAK model → KiCad → EAK → assert semantic equality.
- **Render**: Compare generated PNG/PDF against golden images (pixel diff threshold).
- **Live Sync**: Integration test with simulated file edits.

---

## Next Document

[UI Architecture →](../10-ui-architecture/README.md)