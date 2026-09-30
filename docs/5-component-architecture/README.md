# Component Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines how EAK handles component intelligence: sourcing, evidence ingestion, fact extraction, validation, symbol/footprint/3D model generation and import, and provenance tracking.

---

## Component Lifecycle

```
SOURCE → RESOLVE → INGEST EVIDENCE → EXTRACT FACTS → VALIDATE → GENERATE ASSETS → APPROVE → PROCURE
```

Each stage is a kernel capability with deterministic validation.

---

## 1. Component Sourcing (Resolution)

### Sources
- **Distributor APIs**: DigiKey, Mouser, LCSC, Arrow, Avnet (via `eak-assets` adapters)
- **Manufacturer Sites**: Direct parametric search, datasheet download
- **Local Library**: `data/library.db` (SQLite), user-curated
- **Community Libraries**: KiCad, JLCPCB, EasyEDA, SnapEDA (import adapters)
- **AI Inference**: LLM proposes part numbers from requirements (validated against real data)

### Resolution Process
1. Requirements → parametric constraints (voltage, current, package, temp, etc.)
2. Query sources in priority order (local → distributor → manufacturer → community)
3. Return candidate `Component` entities with `provenance: Import { format, tool }`
4. Kernel validates each candidate against deterministic rules (footprint exists, electrical specs parsable)

### Capability: `ComponentResolution`
```rust
trait ComponentResolution {
    fn resolve(&self, query: ComponentQuery) -> Result<Vec<ComponentCandidate>, ResolutionError>;
}
```

---

## 2. Evidence Ingestion

### Datasheet Acquisition
- **Automated**: Download from manufacturer/distributor (respecting robots.txt, rate limits)
- **Manual**: User uploads PDF
- **Cached**: Local cache keyed by document hash (blake3)

### Document Processing Pipeline
```
PDF → OCR/Parser → Structured Tables/Text → Fact Extraction → Evidence Store
```

### Fact Extraction Methods
| Method | Use Case | Confidence |
|--------|----------|------------|
| **Table Parser** | Parametric tables, absolute max ratings | 0.95 |
| **OCR + LLM** | Graphs, curves, complex layouts | 0.80 |
| **LLM (structured output)** | Unstructured text, notes | 0.75 |
| **Manual** | Critical specs, errata | 1.0 |

Each `ExtractedFact` carries:
- `attribute` (standardized key, e.g., `max_supply_voltage`)
- `value` (typed `Quantity<U>`)
- `conditions` (temperature, voltage, frequency)
- `confidence` (0.0–1.0)
- `location` (page, table, figure)

---

## 3. Fact Validation (Deterministic)

### Validation Rules (in `eak-engines` → `ComponentValidationEngine`)
1. **Unit consistency** — All quantities parse to `eak-units` types.
2. **Range sanity** — Voltage > 0, capacitance > 0, temperature ranges plausible.
3. **Cross-parameter checks** — e.g., `max_power` ≥ `max_voltage` × `max_current`.
4. **Footprint match** — Package code matches footprint dimensions (if footprint exists).
5. **Pin count consistency** — Datasheet pin count matches symbol pin count.
6. **Derating compliance** — Flag if operating point exceeds 80% of absolute max.

Validation runs **before commit**. Failed validation → event rejected, candidate stays `Suggested`.

---

## 4. Asset Generation & Import

### Symbol Generation (`eak-assets` + `eak-kicad`)
- **From pins**: Auto-generate IEEE/ IEC style symbol from `Component.pins`.
- **From template**: Apply package-specific template (e.g., SOIC, QFP, BGA).
- **Import**: Parse existing `.kicad_sym` → EAK symbol model (lossy, flagged `Import`).

### Footprint Generation (`eak-assets` + `eak-kicad`)
- **From IPC-7351**: Compute land pattern from package dimensions (courtyard, pads, silkscreen).
- **From manufacturer data**: Use recommended land pattern from datasheet.
- **Import**: Parse `.kicad_mod` → EAK footprint model.

### 3D Model Resolution
- **Step/WRL**: Search manufacturer, SnapEDA, KiCad 3D models, generate simple extrusion.
- **Mapping**: Link to footprint via `footprint_3d_mapping`.

### Asset Provenance
Every asset (symbol, footprint, 3D model) is an entity with:
```rust
struct Asset {
    id: EntityId,
    component_id: EntityId,
    kind: AssetKind,            // Symbol, Footprint, Model3D
    format: AssetFormat,        // KiCad, EAK-native, STEP, VRML
    source: AssetSource,        // Generated, Imported, Manual
    generator_version: String,  // e.g., "kicad-footprint-gen v0.3"
    hash: Blake3Hash,           // Content-addressed
    provenance: Provenance,
}
```

---

## 5. Component Approval Gate

Before a component reaches `Approved` lifecycle:
1. All critical facts validated (voltage, current, temp, package).
2. Symbol + footprint present and DRC/ERC clean in test schematic.
3. At least one supplier with stock > 0 (or `Procurement` phase handles).
4. Human or lead-agent sign-off (recorded as `ProvenanceSource::Human` or `Agent{role: Lead}`).

---

## 6. Component Database (`eak-assets`)

### Local Cache (`data/assets/`)
```
data/assets/
├── components/          # Component JSON (canonical model subset)
├── symbols/             # .kicad_sym files
├── footprints/          # .kicad_mod files
├── models3d/            # .step, .wrl files
├── datasheets/          # PDFs (cached by hash)
└── index.sqlite         # SQLite FTS5 for search
```

### Search Index
- Full-text search on part number, manufacturer, description, keywords.
- Parametric filters: voltage range, current, package, pins, temperature, price.
- Provenance filter: show only `Approved`, or include `Suggested`.

---

## 7. Integration with KiCad

### Symbol Library
- EAK maintains a **KiCad symbol library** (`.kicad_sym` directory) synced from approved components.
- On `ComponentSelection` capability commit, symbol is written to library.
- KiCad schematic editor uses this library.

### Footprint Library
- EAK maintains a **KiCad footprint library** (`.kicad_mod` directory).
- Same sync mechanism.

### 3D Model Library
- EAK maintains a **KiCad 3D model directory** (`.step`/`.wrl`).
- Footprint-to-3D mapping maintained.

### Round-Trip
- Engineer edits symbol/footprint in KiCad → saves → EAK watches directory → imports changes → creates `Asset` with `Import` provenance → kernel validates → if valid, updates component.

---

## 8. Standardized Attribute Keys (Ontology)

To enable cross-component queries and validation, all facts use standardized keys:

| Domain | Keys (examples) |
|--------|-----------------|
| **Electrical** | `max_supply_voltage`, `min_supply_voltage`, `quiescent_current`, `max_output_current`, `input_voltage_range`, `output_voltage_accuracy`, `switching_frequency`, `bandwidth`, `gain_bandwidth_product`, `input_offset_voltage`, `cmtwr`, `psrr`, `esd_hbm`, `esd_cdm` |
| **Timing** | `propagation_delay`, `rise_time`, `fall_time`, `setup_time`, `hold_time`, `clock_frequency_max` |
| **Thermal** | `operating_temp_min`, `operating_temp_max`, `junction_to_ambient_thermal_resistance`, `junction_to_case_thermal_resistance`, `max_power_dissipation` |
| **Mechanical** | `package_code`, `pin_count`, `pitch`, `body_width`, `body_length`, `body_height`, `lead_width`, `lead_length`, `standoff` |
| **Reliability** | `moisture_sensitivity_level`, `reflow_profile`, `qualification_standard` (AEC-Q100, etc.) |

Keys are defined in `eak-domain` as `ComponentAttribute` enum. Extensible via `Custom(String)`.

---

## 9. Supplier Integration

### Supplier Adapter Trait
```rust
trait SupplierAdapter {
    fn search(&self, query: ComponentQuery) -> Result<Vec<SupplierPart>, SupplierError>;
    fn get_datasheet(&self, part_number: &str) -> Result<DocumentRef, SupplierError>;
    fn check_stock(&self, part_number: &str) -> Result<StockInfo, SupplierError>;
    fn get_pricing(&self, part_number: &str, quantity: u32) -> Result<Pricing, SupplierError>;
}
```

### Supported Suppliers (V1)
- DigiKey (official API)
- Mouser (official API)
- LCSC (public API)
- Generic Octopart-compatible

---

## 10. Capability Seams for Components

| Capability | Input | Validation | Output |
|------------|-------|------------|--------|
| `ComponentResolution` | `ComponentQuery` | Parametric constraints valid | `Vec<ComponentCandidate>` |
| `ComponentIngestEvidence` | `DocumentRef` | PDF parsable, facts extractable | `Vec<Evidence>` |
| `ComponentValidate` | `Component` | All deterministic rules pass | `ValidationReport` |
| `ComponentGenerateSymbol` | `Component` | Pin count matches, valid KiCad symbol | `Asset(Symbol)` |
| `ComponentGenerateFootprint` | `Component` | IPC-7351 compliant, 3D mapping | `Asset(Footprint)` |
| `ComponentApprove` | `ComponentId` | All above gates passed | `ComponentLifecycle::Approved` |

---

## Next Document

[Model/Provider Architecture →](../6-model-provider-architecture/README.md)