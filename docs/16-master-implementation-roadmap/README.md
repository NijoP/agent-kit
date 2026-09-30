# Master Implementation Roadmap: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This is the **single canonical roadmap** for EAK implementation. Stages are **dependency-driven** (not phase-numbered). Each stage has measurable entry/exit criteria (Gates). Stages may overlap but cannot skip dependencies.

---

## Stage Dependency Graph

```
STAGE 0: Repository Reset (THIS TASK)
    │
    ▼
STAGE 1: Foundation Product ──► GATE 1: Developer Preview
    │
    ▼
STAGE 2: Model Intelligence ──► GATE 2: Real AI Engineering Workflow
    │
    ▼
STAGE 3: Component Intelligence ──► GATE 3: Component-Aware Engineering
    │
    ▼
STAGE 4: Schematic ──► GATE 4: Schematic-Capable
    │
    ▼
STAGE 5: PCB ──► GATE 5: PCB-Capable
    │
    ▼
STAGE 6: Engineering Verification ──► GATE 6: Verified Engineering Workflow
    │
    ▼
STAGE 7: Agent Crew ──► GATE 7: Manufacturing Package
    │
    ▼
STAGE 8: Manufacturing ──► GATE 8: Public V1
    │
    ▼
STAGE 9: Product Maturity ──► GATE 9: Mature Engineering Product
```

---

## STAGE 0: Repository Reset (This Task)

**Goal**: Clean repository, canonical documentation, updated branding, master roadmap.

### Deliverables
- [x] Cleaned repository (obsolete material removed/archived)
- [x] Canonical Product Vision (`docs/1-product-vision/`)
- [x] Canonical Product Specification (`docs/2-product-spec/`)
- [x] Canonical Technical Architecture (`docs/3-system-architecture/`)
- [x] Engineering Model (`docs/4-engineering-model/`)
- [x] Component Architecture (`docs/5-component-architecture/`)
- [x] Model/Provider Architecture (`docs/6-model-provider-architecture/`)
- [x] Agent Architecture (`docs/7-agent-architecture/`)
- [x] Terminal/Herdr Architecture (`docs/8-terminal-herdr-architecture/`)
- [x] KiCad Integration Architecture (`docs/9-kicad-integration-architecture/`)
- [x] UI Architecture (`docs/10-ui-architecture/`)
- [x] Verification Architecture (`docs/11-verification-architecture/`)
- [x] Project/Persistence Architecture (`docs/12-project-persistence-architecture/`)
- [x] Development Workflow (`docs/13-development-workflow/`)
- [x] Testing Strategy (`docs/14-testing-strategy/`)
- [x] Release Strategy (`docs/15-release-strategy/`)
- [x] Master Implementation Roadmap (`docs/16-master-implementation-roadmap/`)
- [ ] Updated `eak/README.md` with new vision
- [ ] Updated `app/src-tauri/tauri.conf.json` with EAK branding (or template)
- [ ] No-mistakes validation of all changes

### Gate 0 Criteria (Repository Clean + Docs Canonical)
- [ ] `git status` shows only intended changes (cleanup + new docs + branding)
- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes (existing tests)
- [ ] Documentation builds/links valid
- [ ] No-mistakes gate passes

---

## STAGE 1: Foundation Product

**Goal**: Build system, Tauri, IPC, runtime, event streaming, persistence, provider gateway, basic agent runtime.

### Prerequisites
- Gate 0 passed.

### Work Items
| ID | Work Item | Crate(s) | Dependency |
|----|-----------|----------|------------|
| 1.1 | Cargo workspace setup, crate topology, dependency guard test | `eak/` (all) | — |
| 1.2 | `eak-units`: Physical quantity type system (SI, derived, tolerances) | `eak-units` | 1.1 |
| 1.3 | `eak-domain`: EntityId, core entities (Project, Revision, Requirement, Component, Net), invariants | `eak-domain` | 1.2 |
| 1.4 | `eak-ports`: Port traits (EventLog, ReasoningEngine, KicadSync, TerminalRuntime), Event definitions, Capability schemas | `eak-ports` | 1.3 |
| 1.5 | `eak-runtime`: Event log (JSONL), state fold, FSM framework, execution engine, orchestrator, capability handler, replay | `eak-runtime` | 1.4 |
| 1.6 | `eak-store`: `JsonLinesEventLog`, `SqliteEventLog`, snapshot (msgpack+zstd) | `eak-store` | 1.5 |
| 1.7 | `eak-engines`: Planning Engine (trivial sequencer), stub Verification Engine | `eak-engines` | 1.5 |
| 1.8 | `eak-compiler`: Requirement IR parser, stub Engineering IR lowering | `eak-compiler` | 1.5 |
| 1.9 | `eak-phases`: Requirement Agent (two-part split) + FSM, Phase trait | `eak-phases` | 1.5, 1.7 |
| 1.10 | `eak-reasoning`: `FixtureReasoningEngine` (deterministic) | `eak-reasoning` | 1.4 |
| 1.11 | `eak-cli`: `eak` binary, composition root, `run`, `replay`, `trace` commands | `eak-cli` | 1.5, 1.6, 1.9, 1.10 |
| 1.12 | Tauri app: `start_run` IPC, event streaming to frontend, basic UI shell | `app/` | 1.11 |
| 1.13 | Herdr integration: `TerminalRuntime` port, `HerdrAdapter` stub | `herdr` (new crate) | 1.4 |
| 1.14 | Observability: `eak-observability` (tracing), `eak-dashboard` (metrics) | `eak-observability`, `eak-dashboard` | 1.5 |
| 1.15 | CI: `cargo check/test/clippy/fmt`, doc tests, no-mistakes integration | `.github/` | 1.1 |

### Gate 1 Criteria (Developer Preview)
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes (unit + integration)
- [ ] `eak run --deterministic` executes Requirement phase, writes event log
- [ ] `eak replay` reconstructs identical state
- [ ] `eak trace` shows provenance chain
- [ ] Tauri app launches, shows live event stream from `start_run`
- [ ] Herdr session created, PTY accessible
- [ ] No-mistakes gate passes on `main`

---

## STAGE 2: Model Intelligence

**Goal**: Provider abstraction, NVIDIA/OpenAI-compatible endpoint, streaming, tool calls, model capabilities, credential handling, structured responses, agent proposals.

### Prerequisites
- Gate 1 passed.

### Work Items
| ID | Work Item | Crate(s) | Dependency |
|----|-----------|----------|------------|
| 2.1 | `ReasoningEngine` port: capability negotiation, structured output, tool calls, streaming | `eak-ports` | 1.4 |
| 2.2 | `NvidiaNimAdapter`: NIM API, auth, models, structured output | `eak-reasoning` | 2.1 |
| 2.3 | `OpenAICompatibleAdapter`: Generic OpenAI-compatible endpoint (Ollama, vLLM, TGI, OpenAI) | `eak-reasoning` | 2.1 |
| 2.4 | `OllamaAdapter`: Local Ollama, grammar-based structured output | `eak-reasoning` | 2.1 |
| 2.5 | Credential management: env vars, keyring, rotation, audit log | `eak-reasoning` | 2.1 |
| 2.6 | Kernel-defined tool schemas (JSON Schema) for all capability seams | `eak-ports/schemas/` | 1.4 |
| 2.7 | Agent proposal flow: model → tool_call → capability validation → commit | `eak-phases`, `eak-runtime` | 2.1, 1.5 |
| 2.8 | Bounded retries, circuit breaker, timeout, escalation | `eak-phases` | 2.7 |
| 2.9 | Model capabilities registry, routing by agent role | `eak-reasoning` | 2.1 |
| 2.10 | Observability: token usage, latency, cost, error rates | `eak-observability` | 2.1 |

### Gate 2 Criteria (Real AI Engineering Workflow)
- [ ] Live NVIDIA NIM provider works (with API key)
- [ ] Live OpenAI-compatible provider works (Ollama local)
- [ ] Structured output enforced (JSON Schema validation)
- [ ] Tool calls map to capability seams correctly
- [ ] Agent proposes Requirement → kernel validates → commits
- [ ] Retries work, circuit breaker triggers
- [ ] Streaming tokens displayed in Tauri UI
- [ ] No-mistakes gate passes

---

## STAGE 3: Component Intelligence

**Goal**: Component source resolution, evidence ingestion, datasheet extraction, fact model, validation, symbol generation/import, footprint generation/import, 3D resolution, provenance.

### Prerequisites
- Gate 2 passed.

### Work Items
| ID | Work Item | Crate(s) | Dependency |
|----|-----------|----------|------------|
| 3.1 | `eak-assets`: Component database (SQLite), search index (FTS5) | `eak-assets` | 1.3 |
| 3.2 | Supplier adapters: DigiKey, Mouser, LCSC (API clients, rate limiting) | `eak-assets` | 3.1 |
| 3.3 | Datasheet download/caching (respect robots.txt, hash-based dedup) | `eak-assets` | 3.2 |
| 3.4 | PDF parsing: table extraction (pdfextract), OCR (tesseract), LLM fallback | `eak-assets` | 3.3 |
| 3.5 | Fact extraction: standardized attribute ontology, typed quantities | `eak-assets`, `eak-units` | 3.4 |
| 3.6 | Deterministic fact validation (unit consistency, range sanity, cross-param) | `eak-engines` | 3.5 |
| 3.7 | Symbol generation: from pins → IEEE/IEC symbol, KiCad `.kicad_sym` | `eak-assets`, `eak-kicad` | 3.1 |
| 3.8 | Footprint generation: IPC-7351 from package dims, KiCad `.kicad_mod` | `eak-assets`, `eak-kicad` | 3.1 |
| 3.9 | 3D model resolution: manufacturer, SnapEDA, KiCad, simple extrusion | `eak-assets` | 3.1 |
| 3.10 | Asset provenance: generated vs imported, generator version, content hash | `eak-assets` | 3.7, 3.8, 3.9 |
| 3.11 | Component lifecycle: Suggested → Selected → Verified → Approved → Procured | `eak-phases` (Component Agent) | 3.6 |
| 3.12 | KiCad library sync: approved components → project `lib/` | `eak-kicad` | 3.10 |

### Gate 3 Criteria (Component-Aware Engineering)
- [ ] Component search returns evidence-backed parts from multiple suppliers
- [ ] Datasheet PDF → extracted facts with provenance, confidence
- [ ] Fact validation catches errors (unit mismatch, range violation)
- [ ] Symbol/footprint generated for new component, KiCad-compatible
- [ ] 3D model resolved or generated
- [ ] Component approval gate enforced (validation + supplier stock)
- [ ] KiCad library sync works bidirectionally
- [ ] No-mistakes gate passes

---

## STAGE 4: Schematic

**Goal**: Canonical schematic model, rendering, placement, wiring, net labels, properties, ERC, synchronization.

### Prerequisites
- Gate 3 passed.

### Work Items
| ID | Work Item | Crate(s) | Dependency |
|----|-----------|----------|------------|
| 4.1 | Schematic entities: Sheet, ComponentInstance, Wire, NetLabel, HierarchicalPort, Text, Image | `eak-domain` | 1.3 |
| 4.2 | Netlist derivation from schematic (pins + wires + labels) | `eak-engines` | 4.1 |
| 4.3 | ERC rules: unconnected pins, power/ground, pin conflicts, shorts | `eak-engines` | 4.2 |
| 4.4 | SchematicEdit capability: add/move/delete components, wires, labels | `eak-ports`, `eak-phases` | 4.1 |
| 4.5 | Schematic Agent FSM: place components, wire nets, run ERC, iterate | `eak-phases` | 4.4 |
| 4.6 | KiCad schematic import/export (S-expression parser/generator) | `eak-kicad` | 4.1 |
| 4.7 | KiCad schematic round-trip fidelity (golden tests) | `eak-kicad` | 4.6 |
| 4.8 | Schematic rendering: React Flow graph in Tauri UI (read-only) | `app/ui` | 4.1 |
| 4.9 | Live KiCad sync: watch `.kicad_sch` → propose SchematicEdit | `eak-kicad` | 4.6 |
| 4.10 | Hierarchical design: multi-sheet, hierarchical ports, sheet instances | `eak-domain`, `eak-engines` | 4.1 |

### Gate 4 Criteria (Schematic-Capable)
- [ ] Schematic model created from requirements + components
- [ ] ERC passes on clean design, catches errors on dirty design
- [ ] KiCad round-trip: EAK → KiCad → EAK semantically equivalent
- [ ] Tauri UI shows schematic graph (read-only)
- [ ] Engineer edits in KiCad → sync → kernel validates → commits
- [ ] Hierarchical sheets supported
- [ ] No-mistakes gate passes

---

## STAGE 5: PCB

**Goal**: Board model, placement, routing, vias, layers, constraints, DRC, KiCad integration.

### Prerequisites
- Gate 4 passed.

### Work Items
| ID | Work Item | Crate(s) | Dependency |
|----|-----------|----------|------------|
| 5.1 | PCB entities: BoardOutline, Stackup, Placement, Route, Zone, Via, Layer | `eak-domain` | 1.3 |
| 5.2 | Constraint system: impedance, length match, diff pair, clearance, min trace | `eak-engines` | 5.1 |
| 5.3 | DRC rules: clearance, min trace/space/drill, annular ring, mask sliver, acid trap | `eak-engines` | 5.1 |
| 5.4 | Placement engine: auto-place (constraint-driven), manual override | `eak-engines` | 5.1 |
| 5.5 | Routing engine: topological → geometric, constraint-aware, via optimization | `eak-engines` | 5.1, 5.2 |
| 5.6 | PcbEdit capability: placement, routing, stackup, zones, vias | `eak-ports`, `eak-phases` | 5.1 |
| 5.7 | PCB Agent FSM: place → route → DRC → iterate | `eak-phases` | 5.6 |
| 5.8 | KiCad PCB import/export (S-expression parser/generator) | `eak-kicad` | 5.1 |
| 5.9 | KiCad PCB round-trip fidelity (golden tests) | `eak-kicad` | 5.8 |
| 5.10 | PCB rendering: React Flow placement/routing graph (read-only) | `app/ui` | 5.1 |
| 5.11 | Live KiCad PCB sync: watch `.kicad_pcb` → propose PcbEdit | `eak-kicad` | 5.8 |
| 5.12 | Stackup editor: layer materials, thickness, impedance calculation | `eak-engines` | 5.2 |

### Gate 5 Criteria (PCB-Capable)
- [ ] PCB model created from schematic netlist
- [ ] Auto-placement respects constraints
- [ ] Auto-routing completes simple boards, DRC clean
- [ ] KiCad PCB round-trip: EAK → KiCad → EAK semantically equivalent
- [ ] Tauri UI shows PCB graph (read-only)
- [ ] Engineer edits in KiCad → sync → kernel validates → commits
- [ ] Stackup editor with impedance calculation
- [ ] No-mistakes gate passes

---

## STAGE 6: Engineering Verification

**Goal**: Electrical rules, power, clocks, signal integrity, return paths, DFM, EMC, confidence/fidelity, human review.

### Prerequisites
- Gate 5 passed.

### Work Items
| ID | Work Item | Crate(s) | Dependency |
|----|-----------|----------|------------|
| 6.1 | Verification framework: rule trait, engine trait, confidence/fidelity model | `eak-engines` | 1.7 |
| 6.2 | Power Engine: rail analysis, IR drop, decap, current loops, thermal | `eak-engines` | 5.1 |
| 6.3 | Clock Engine: tree distribution, skew, jitter, termination | `eak-engines` | 5.1 |
| 6.4 | SI Engine: impedance, reflections, crosstalk, eye diagrams (post-layout) | `eak-engines` | 5.1, 5.5 |
| 6.5 | Return Path Engine: ground continuity, stitching vias, layer transitions | `eak-engines` | 5.1 |
| 6.6 | EMC Engine: loop/dipole antenna, edge rate, shielding | `eak-engines` | 5.1 |
| 6.7 | DFM Engine: fabricator rules (min trace, space, drill, annular, mask, copper balance) | `eak-engines` | 5.3 |
| 6.8 | VerificationRequest capability: run specific kind, scope, fidelity | `eak-ports`, `eak-phases` | 6.1 |
| 6.9 | Verification Agent FSM: run → analyze → report → gate | `eak-phases` | 6.8 |
| 6.10 | Confidence calibration: analytical=1.0, simulation=0.8, measured=1.0 | `eak-engines` | 6.1 |
| 6.11 | Human approval gates for Error severity, confidence > 0.9 | `eak-phases` | 6.9 |
| 6.12 | Waiver system: human waives with reason, recorded in provenance | `eak-runtime` | 6.11 |
| 6.13 | Verification Dashboard UI: summary, table, provenance graph, trends | `app/ui` | 6.1 |

### Gate 6 Criteria (Verified Engineering Workflow)
- [ ] Power verification runs, produces confidence/fidelity
- [ ] Clock verification runs, produces confidence/fidelity
- [ ] SI verification runs (post-layout), produces confidence/fidelity
- [ ] Return path analysis runs
- [ ] EMC risk assessment runs
- [ ] DFM runs against fabricator rules
- [ ] Human approval gates block release on critical failures
- [ ] Waiver system works with provenance
- [ ] Dashboard shows all verification results with drill-down
- [ ] No-mistakes gate passes

---

## STAGE 7: Agent Crew

**Goal**: Lead, Research, Component, Schematic, PCB, Verification, Manufacturing, Handoff agents; bounded retries, approval gates, durable state.

### Prerequisites
- Gate 6 passed.

### Work Items
| ID | Work Item | Crate(s) | Dependency |
|----|-----------|----------|------------|
| 7.1 | Lead Agent: orchestrates workflow, decomposes intent, manages gates | `eak-phases` | 6.9 |
| 7.2 | Research Agent: gathers datasheets, app notes, standards | `eak-phases`, `eak-assets` | 3.3 |
| 7.3 | Component Agent: selects, validates, generates assets, approves | `eak-phases`, `eak-assets` | 3.11 |
| 7.4 | Schematic Agent: places, wires, ERC, hierarchical | `eak-phases` | 4.5 |
| 7.5 | PCB Agent: places, routes, DRC, stackup | `eak-phases` | 5.7 |
| 7.6 | Verification Agent: runs all verification kinds, reports | `eak-phases` | 6.9 |
| 7.7 | Manufacturing Agent: BOM, Gerber, Drill, PnP, assembly drawings | `eak-phases` | 8.1 |
| 7.8 | Handoff Agent: release package, documentation, sign-off | `eak-phases` | 8.1 |
| 7.9 | Agent communication: via kernel events only (no direct) | `eak-runtime` | 1.5 |
| 7.10 | Durable agent state: phase FSM persisted in event log | `eak-runtime`, `eak-phases` | 1.5 |
| 7.11 | Bounded retries per capability, exponential backoff | `eak-phases` | 2.8 |
| 7.12 | Approval gates integrated in Lead Agent workflow | `eak-phases` | 6.11 |
| 7.13 | Agent observability: status, proposals, retries, approvals in UI | `app/ui`, `eak-dashboard` | 7.1 |

### Gate 7 Criteria (Manufacturing Package)
- [ ] Full agent crew runs end-to-end on sample design
- [ ] Lead Agent delegates, manages gates, produces release
- [ ] All agents persist state, recover on restart
- [ ] Retries and escalation work
- [ ] Human approval gates at each phase transition
- [ ] No-mistakes gate passes

---

## STAGE 8: Manufacturing

**Goal**: BOM, sourcing, Gerber, drill, pick-and-place, assembly output, release package.

### Prerequisites
- Gate 7 passed.

### Work Items
| ID | Work Item | Crate(s) | Dependency |
|----|-----------|----------|------------|
| 8.1 | BOM generation: lines, quantities, refdes, supplier options, lifecycle | `eak-engines` | 3.11 |
| 8.2 | Gerber export (RS-274X): all layers, apertures, format validation | `eak-engines`, `eak-kicad` | 5.1 |
| 8.3 | Drill export (Excellon): plated/non-plated, tool list | `eak-engines`, `eak-kicad` | 5.1 |
| 8.4 | Pick-and-place export (CSV): refdes, package, x, y, rotation, side | `eak-engines` | 5.1 |
| 8.5 | Assembly drawings (PDF): top/bottom placement, polarity, fiducials | `eak-engines`, `eak-kicad` | 5.1 |
| 8.6 | ManufacturingOutput capability: generate all artifacts | `eak-ports`, `eak-phases` | 8.1 |
| 8.7 | Fabricator DFM check: integrate with OSH Park, JLCPCB, PCBWay rules | `eak-engines` | 6.7 |
| 8.8 | Release package: signed, versioned, reproducible, includes SBOM | `eak-cli`, `app/` | 8.6 |
| 8.9 | Sourcing integration: DigiKey/Mouser/LCSC cart export, API ordering | `eak-assets` | 3.2 |

### Gate 8 Criteria (Public V1)
- [ ] BOM generated with supplier options, lifecycle status
- [ ] Gerber/Drill/PnP pass fabricator DFM checks (OSH Park, JLCPCB)
- [ ] Assembly drawings complete
- [ ] Release package signed, versioned, reproducible
- [ ] Sourcing integration works for major distributors
- [ ] No-mistakes gate passes

---

## STAGE 9: Product Maturity

**Goal**: Project lifecycle, revision, diff, snapshots, release management, crash recovery, observability, CI, packaging, signed builds, installers, docs, examples, onboarding.

### Prerequisites
- Gate 8 passed.

### Work Items
| ID | Work Item | Crate(s) | Dependency |
|----|-----------|----------|------------|
| 9.1 | Project lifecycle: create, open, close, archive | `eak-cli`, `app/` | 12.1 |
| 9.2 | Revision management: create, list, diff, branch, tag | `eak-cli`, `app/` | 12.2 |
| 9.3 | Diff engine: structural diff between revisions (all entity types) | `eak-runtime` | 12.2 |
| 9.4 | Snapshot management: automatic, manual, retention, cleanup | `eak-store` | 12.3 |
| 9.5 | Crash recovery: automated, reported, bounded data loss | `eak-runtime`, `eak-store` | 12.4 |
| 9.6 | Observability: structured logging, metrics, tracing, dashboard | `eak-observability`, `eak-dashboard` | 1.14 |
| 9.7 | CI/CD: signed builds, installers, SBOM, SLSA provenance | `.github/` | 15.1 |
| 9.8 | Packaging: crates.io, GitHub Releases, Homebrew, Scoop, AUR | `.github/` | 9.7 |
| 9.9 | Documentation: complete, examples, tutorials, API reference | `docs/`, `examples/` | 0 |
| 9.10 | Onboarding: `eak init`, guided first project, sample designs | `eak-cli`, `app/` | 9.1 |
| 9.11 | Plugin/extension system (post-V1): custom verification rules, exporters | `eak-ports` | 9.9 |

### Gate 9 Criteria (Mature Engineering Product)
- [ ] Full project lifecycle managed in UI and CLI
- [ ] Revision diff shows engineering changes clearly
- [ ] Crash recovery tested and documented
- [ ] Observability dashboard shows system health
- [ ] Signed installers for all platforms, auto-update
- [ ] Comprehensive documentation, examples, tutorials
- [ ] New user can complete first design in < 30 min
- [ ] No-mistakes gate passes

---

## Resource Estimates (Rough)

| Stage | Person-Weeks | Key Risks |
|-------|--------------|-----------|
| 0 | 1 (this task) | — |
| 1 | 4-6 | Rust learning curve, Tauri integration |
| 2 | 3-4 | Provider API changes, structured output reliability |
| 3 | 6-8 | Datasheet extraction accuracy, supplier API stability |
| 4 | 4-6 | KiCad S-expression complexity, round-trip fidelity |
| 5 | 6-10 | Routing engine complexity, DRC completeness |
| 6 | 8-12 | Simulation integration, confidence calibration |
| 7 | 4-6 | Agent coordination, deadlock avoidance |
| 8 | 3-4 | Fabricator rule variations, export format edge cases |
| 9 | 4-6 | Cross-platform packaging, documentation scale |

**Total**: ~46-62 person-weeks to Gate 9 (V1 Mature).

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| KiCad S-expression changes | Pin to KiCad 8, adapter versioned, golden tests |
| Model provider API drift | Adapter pattern, capability negotiation, fixture fallback |
| Routing engine complexity | Start with topological, integrate open-source router (e.g., `kicad-pcb-router` via CLI) |
| Simulation integration | Optional fidelity; analytical first, simulation via pluggable adapters |
| Agent deadlock | Timeout supervision, Lead Agent circuit breaker |
| Data migration | Event schema versioning, deterministic migrations, tested in CI |

---

## Next Steps

1. **Complete Stage 0** — Merge this branch via no-mistakes.
2. **Begin Stage 1** — Firstmate spawns `fm/EAK-Stage1-Foundation` tasks.
3. **Parallelize** — Stage 1 crates can be developed in parallel (units, domain, ports, runtime, store).
4. **Weekly Sync** — Firstmate reviews progress against Gates.

---

## Appendix: Gate Checklist Summary

| Gate | Name | Key Criteria |
|------|------|--------------|
| **GATE 0** | Repository Clean + Docs Canonical | Cleanup done, docs written, branding updated, tests pass |
| **GATE 1** | Developer Preview | Kernel runs, replay works, Tauri streams events, Herdr session |
| **GATE 2** | Real AI Engineering Workflow | Live providers, structured output, tool calls, agent proposals validated |
| **GATE 3** | Component-Aware Engineering | Evidence-backed components, assets generated, approval gate |
| **GATE 4** | Schematic-Capable | Schematic model, ERC, KiCad round-trip, UI graph |
| **GATE 5** | PCB-Capable | PCB model, DRC, routing, KiCad round-trip, UI graph |
| **GATE 6** | Verified Engineering Workflow | Power/Clock/SI/EMC/DFM, confidence/fidelity, human gates |
| **GATE 7** | Manufacturing Package | Agent crew end-to-end, durable state, approvals |
| **GATE 8** | Public V1 | BOM/Gerber/Drill/PnP, fabricator pass, signed installers |
| **GATE 9** | Mature Engineering Product | Project lifecycle, diff, recovery, observability, packaging, docs |

---

*End of Master Implementation Roadmap*