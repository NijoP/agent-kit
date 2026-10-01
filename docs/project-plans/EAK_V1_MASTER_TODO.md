# EAK V1 MASTER EXECUTION TODO

## EAK V1 Vision: "Intent In. Manufactured Board Out."

### CURRENT EXECUTION QUEUE (ACTIVE)

- [x] **ACTIVE** Finish authoritative 3,500-component taxonomy (`docs/engineering/library-taxonomy.md`)
- [x] **ACTIVE** Validate exact 3,500 allocation (3000 additional + 500 current = 3500)
- [x] **ACTIVE** Define family-specific metadata architecture (COMMON CORE + FAMILY-SPECIFIC)
- [x] **ACTIVE** Create golden-board component capability matrix (`docs/engineering/eak-v1-demo-component-matrix.md`)
- [x] **ACTIVE** Add deterministic taxonomy validation (allocation total == 3500, no duplicates, etc.)
- [x] **ACTIVE** Generate Day 2 report (`reports/day2-taxonomy-60min.md`)

---

### PHASE 0 — Repository and Architecture Stabilization
- [ ] Verify current repository state
- [ ] Audit existing architecture and documentation
- [ ] Verify 500-component Day 1 foundation integrity
- [ ] Confirm existing tests pass
- [ ] **DONE** - Initial audit complete

### PHASE 1 — 3,500-Component Taxonomy
- [x] **COMPLETE** Authoritative 3,500-component taxonomy (`docs/engineering/library-taxonomy.md`)
- [x] **COMPLETE** Validate exact 3,500 allocation (3000 additional + 500 current = 3500)
- [x] **COMPLETE** Define family-specific metadata requirements
- [x] **COMPLETE** Create golden-board component capability matrix
- [x] **COMPLETE** Add deterministic taxonomy validation
- [x] **COMPLETE** Generate Day 2 report

### PHASE 2 — Component Data Model
- [ ] Design common core + family-specific metadata architecture
- [ ] Extend ComponentClass enum for all required families
- [ ] Design family-specific metadata structures
- [ ] Update PartCatalog / PartCatalog trait
- [ ] Update database schema
- [ ] Update validation scripts

### PHASE 3 — Real Component Acquisition/Import Pipeline
- [ ] Design honest asset state machine (SOURCE_DECLARED → VERIFIED / BLOCKED / NOT_FOUND)
- [ ] Implement asset state machine in code
- [ ] Implement honest downloader with Cloudflare/403/404 detection
- [ ] Implement SHA-256 deduplication
- [ ] Implement honest downloader with Cloudflare/403/404 detection
- [ ] Implement SHA-256 deduplication
- [ ] Implement idempotent import pipeline
- [ ] Add semantic validation for family-specific fields

### PHASE 4 — Symbol / Footprint / 3D Asset Pipeline
- [ ] Implement generic symbol generator (per ComponentClass)
- [ ] Implement footprint generator (IPC-7351 compliant)
- [ ] Implement STEP model acquisition (for power inductors)
- [ ] Implement asset deduplication (SHA-256)
- [ ] Implement asset verification pipeline

### PHASE 5 — Component Intelligence and Search
- [ ] Implement deterministic library search (FTS5 + filters)
- [ ] Implement deterministic component ranking
- [ ] Implement asset availability awareness in search
- [ ] Implement readiness-aware filtering

### PHASE 5 — User Model/Provider API Integration
- [ ] Design model provider adapter trait
- [ ] Implement Anthropic adapter
- [ ] Implement OpenAI adapter
- [ ] Implement Ollama/local adapter
- [ ] Implement model-agnostic agent interface

### PHASE 6 — Agent Orchestration Framework
- [ ] Define agent orchestration trait
- [ ] Implement Requirement Planning agent
- [ ] Implement Schematic Planning agent
- [ ] Implement Component Selection agent
- [ ] Implement BOM Planning agent

### PHASE 8 — PRD / Intent Interpretation
- [ ] Implement PRD parser (markdown + YAML front-matter)
- [ ] Implement structured intent extraction
- [ ] Implement requirement decomposition

### PHASE 9 — Component Selection Agent
- [ ] Implement component search/ranking
- [ ] Implement family-aware component selection
- [ ] Implement asset-aware selection (prefer parts with verified assets)
- [ ] Implement readiness-aware selection

### PHASE 10 — Schematic Generation Engine
- [ ] Implement schematic IR
- [ ] Implement netlister
- [ ] Implement schematic generator from PRD
- [ ] Implement component placement on schematic
- [ ] Implement net wiring

### PHASE 11 — Electrical Engineering Verification
- [ ] Implement power balance rule (KCL)
- [ ] Implement clock domain crossing detection
- [ ] Implement return path verification
- [ ] Implement pin mux conflict detection
- [ ] Implement pin capability checking
- [ ] Implement signal driver/sink validation
- [ ] Implement interface contract validation
- [ ] Implement bus topology verification
- [ ] Implement subsystem boundary verification

### PHASE 12 — Autonomous Schematic Correction Loop
- [ ] Implement agent proposal → kernel verification loop
- [ ] Implement finding → agent fix → re-verify cycle
- [ ] Implement maximum iteration limits
- [ ] Implement convergence detection

### PHASE 13 — PCB Architecture
- [ ] Design PCB IR (PcbIr)
- [ ] Implement board outline / stackup definition
- [ ] Implement design rules (clearance, width, via)

### PHASE 14 — Component Placement Engine
- [ ] Implement placement algorithm
- [ ] Implement courtyard collision detection
- [ ] Implement keep-out zone handling
- [ ] Implement thermal-aware placement

### PHASE 15 — Trace Width / Current Engineering
- [ ] Implement ampacity calculations
- [ ] Implement trace width calculator
- [ ] Implement thermal analysis
- [ ] Implement via current capacity

### PHASE 16 — Ground and Return-Path Engineering
- [ ] Implement return path assignment
- [ ] Implement via stitching
- [ ] Implement stitching via placement
- [ ] Implement ground pour

### PHASE 17 — PCB Routing Engine
- [ ] Implement trace routing algorithm
- [ ] Implement differential pair routing
- [ ] Implement length matching
- [ ] Implement via minimization

### PHASE 18 — DRC/ERC/Engineering Verification
- [ ] Implement clearance checking
- [ ] Implement trace width checking
- [ ] Implement annular ring checking
- [ ] Implement annular ring checking
- [ ] Implement DFM checks
- [ ] Implement manufacturing constraint checks

### PHASE 19 — BOM Generation
- [ ] Implement BOM IR
- [ ] Implement supplier lookup
- [ ] Implement lifecycle checking (EOL/NRND)
- [ ] Implement alternate part suggestions

### PHASE 19 — Manufacturing Outputs
- [ ] Implement Gerber export
- [ ] Implement ODB++ export
- [ ] Implement IPC-2581 export
- [ ] Implement drill files
- [ ] Implement pick-and-place files
- [ ] Implement assembly drawings

### PHASE 21 — EAK V1 Frontend / Workspace
- [ ] Implement PRD folder tree view
- [ ] Implement markdown PRD editor
- [ ] Implement live schematic canvas
- [ ] Implement live PCB canvas
- [ ] Implement verification panel
- [ ] Implement chat/terminal panel
- [ ] Implement library panel with FTS5 search

### PHASE 22 — Live Agent Orchestration Visualization
- [ ] Implement agent action streaming
- [ ] Implement live verification panel
- [ ] Implement agent reasoning visualization
- [ ] Implement finding display with fixes

### PHASE 23 — End-to-End Golden Board Demo
- [ ] Define golden board requirements
- [ ] Implement golden board test case
- [ ] Execute full PRD → PCB flow
- [ ] Validate manufacturing outputs
- [ ] Physical PCB fabrication
- [ ] Physical board testing

### PHASE 24 — Physical PCB Fabrication
- [ ] Export manufacturing package
- [ ] Submit to fab
- [ ] Receive boards
- [ ] Assemble
- [ ] Test

### PHASE 25 — Investor Demonstration Preparation
- [ ] Prepare demo script
- [ ] Prepare live demo environment
- [ ] Prepare backup scenarios
- [ ] Create demo script

## CURRENT EXECUTION QUEUE

**ACTIVE:**
- [x] Initial repository audit and audit
- [x] Create EAK_V1_MASTER_TODO.md
- [x] **COMPLETE**: Finish authoritative 3,500-component taxonomy (`docs/engineering/library-taxonomy.md`)
- [x] Validate exact 3,500 allocation (3000 additional + 500 current = 3500)
- [x] Define family-specific metadata architecture
- [x] Create golden-board component capability matrix
- [x] Add deterministic taxonomy validation
- [x] Generate Day 2 report

**NEXT PHASE:**
- [ ] **IN PROGRESS**: Phase 2 — Component Data Model (extend ComponentClass enum, define Rust metadata structs, update PartCatalog/database)

---

## EXECUTION LOG

| Timestamp | Task | Files Changed | Tests Executed | Result | Evidence | Next Task |
|-----------|------|---------------|----------------|--------|----------|-----------|
| 2025-08-20 23:45 | Initial repository audit | N/A | N/A | COMPLETE | git status, git log, cargo test all pass | Create EAK_V1_MASTER_TODO.md |
| 2025-08-20 | Phase 1 Complete (6 tasks) | 3 files created | 14 validation checks | COMPLETE | 3,500 taxonomy validated; golden-board matrix; validation script CI-ready; Day 2 report | Phase 2: Data Model |

---

## BLOCKERS

None currently.

---

## NEXT IMMEDIATE ACTION

**Phase 2: Component Data Model** — Extend ComponentClass enum for 16 families, define family-specific Rust metadata structs, update PartCatalog trait & database schema (Terminal 2 coordination required)