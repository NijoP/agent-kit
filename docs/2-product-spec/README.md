# Product Specification: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines the user-facing capabilities, interfaces, and acceptance criteria for EAK V1. It translates the product vision into testable requirements.

---

## User Personas

1. **Hardware Engineer** — Designs schematics and PCBs, needs verified, manufacturable outputs.
2. **AI Agent (Lead/Research/Component/Schematic/PCB/Verification/Manufacturing/Handoff)** — Operates within kernel capability seams.
3. **Developer/Integrator** — Extends EAK with new agents, providers, verification rules, exporters.

---

## Core User Flows

### 1. Intent to Verified Design
```
Natural language intent → Requirements → Component selection → Schematic → PCB → Verification → Manufacturing package
```
Each step is an engineering phase executed by agents under kernel validation.

### 2. Interactive Refinement
Engineer opens design in KiCad (via EAK integration), makes changes, syncs back to EAK canonical model. Kernel validates.

### 3. Evidence-Backed Component Selection
Engineer/Agent queries component database → EAK returns components with datasheet evidence, provenance, confidence scores.

### 4. Verification Review
Engineer reviews verification results (ERC, DRC, power, SI, EMC, DFM) with confidence/fidelity metrics. Human approval gates.

---

## Functional Requirements

### FR1: Requirements Engineering
- **FR1.1** Capture intent in natural language; decompose into structured requirements (functional, performance, environmental, regulatory).
- **FR1.2** Requirements are versioned, traceable, and linked to evidence.
- **FR1.3** Requirements can be edited by human or proposed by AI (validated by kernel).

### FR2: Component Intelligence
- **FR2.1** Resolve components from multiple sources (DigiKey, Mouser, LCSC, manufacturer sites, local library).
- **FR2.2** Ingest datasheets, extract facts (electrical, mechanical, thermal) with provenance.
- **FR2.3** Generate/import KiCad symbols, footprints, 3D models with provenance.
- **FR2.4** Validate component facts against engineering rules (voltage, current, temperature, package).
- **FR2.5** Track component lifecycle: suggested → selected → verified → approved → procured.

### FR3: Schematic Engineering
- **FR3.1** Canonical schematic model: components, pins, nets, hierarchical sheets, properties.
- **FR3.2** Placement and wiring with deterministic rules (grid, clearance, net naming).
- **FR3.3** Electrical Rule Check (ERC) as deterministic kernel function.
- **FR3.4** Bidirectional sync with KiCad schematic editor (import/export).

### FR4: PCB Engineering
- **FR4.1** Canonical PCB model: board outline, layers, placement, routing, vias, zones, constraints.
- **FR4.2** Placement and routing with constraint-driven engine (impedance, length matching, differential pairs).
- **FR4.3** Design Rule Check (DRC) as deterministic kernel function.
- **FR4.4** Bidirectional sync with KiCad PCB editor (import/export).

### FR5: Engineering Verification
- **FR5.1** Power integrity: rail analysis, decoupling, current loops.
- **FR5.2** Clock/tree distribution: skew, jitter, termination.
- **FR5.3** Signal integrity: impedance, reflections, crosstalk, eye diagrams (post-layout).
- **FR5.4** Return path analysis: ground continuity, stitching vias.
- **FR5.5** EMC: emission/susceptibility risk assessment.
- **FR5.6** DFM: manufacturability rules (min trace, space, drill, annular ring, solder mask).
- **FR5.7** Confidence/fidelity scoring for each verification result.

### FR6: Manufacturing Outputs
- **FR6.1** BOM with sourcing options, lifecycle status, alternates.
- **FR6.2** Gerber (RS-274X), Drill (Excellon), Pick-and-Place (CSV), Assembly drawings (PDF).
- **FR6.3** Release package: signed, versioned, reproducible.

### FR7: Project Lifecycle
- **FR7.1** Project creation, revision, branching, merging.
- **FR7.2** Snapshots and time-travel (replay to any event).
- **FR7.3** Crash recovery: kernel state reconstructed from event log.
- **FR7.4** Diff between revisions (requirements, schematic, PCB, BOM).

### FR8: AI Agent Orchestration
- **FR8.1** Agent roles: Lead, Research, Component, Schematic, PCB, Verification, Manufacturing, Handoff.
- **FR8.2** Bounded retries with exponential backoff and human approval gates.
- **FR8.3** Durable agent state persisted in kernel event log.

### FR9: Integrations
- **FR9.1** KiCad: schematic/PCB editing, viewing, import/export, rendering.
- **FR9.2** Herdr: persistent sessions, PTY, tabs, panes, agent detection.
- **FR9.3** Model providers: NVIDIA NIM, OpenAI-compatible, local (llama.cpp, Ollama).
- **FR9.4** Firstmate: development orchestration (external).

---

## Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR1 | Local-first, air-gapped capable | No mandatory network calls |
| NFR2 | Deterministic replay | Byte-identical state from event log |
| NFR3 | Provenance granularity | Per-fact, per-event |
| NFR4 | Kernel latency (commit) | < 10ms for typical events |
| NFR5 | Schematic/PCB sync round-trip | < 2s for 500-component design |
| NFR6 | Verification throughput | 10k rules/sec on modern CPU |
| NFR7 | Model provider swap | Zero kernel changes |
| NFR8 | Crash recovery time | < 5s for 10k-event project |
| NFR9 | Binary size (kernel) | < 50MB |
| NFR10 | Cross-platform | Linux, macOS, Windows (via Tauri) |

---

## Interfaces

### Kernel API (Rust)
- `RuntimeCore::commit(event)` — single commit path, returns `Result<StateDelta, KernelError>`
- `RuntimeCore::replay(log)` — reconstructs state
- `EventSink` trait — live event streaming to UI/integrations
- `Capability` traits — validated entry points for AI proposals

### CLI (`eak` binary)
- `eak run` — Run Requirement Planning (+ Engineering Analysis stub) on a design intent.
- `eak replay` — Replay an event log and print the reconstructed state.
- `eak trace` — Print the provenance chain for a requirement (by short or full id).
- `eak list-providers` — List all configured providers.
- `eak list-models` — List available models for a provider.
- `eak test-connection` — Test connectivity to a provider.
- `eak configure-provider` — Add or update a provider configuration.
- `eak asset-parse` — Parse a datasheet PDF and extract structured facts.
- `eak asset-facts` — List extracted facts for a datasheet asset.
- `eak asset-compare` — Cross-check datasheet facts against component metadata.
- `eak asset-verify-identity` — Verify datasheet identity matches component record.

### IPC (Tauri)
- `start_run(intent, config)` — returns stream of `EventRecord`
- `get_state(snapshot_id)` — returns serialized engineering model
- `propose_capability(capability, payload)` — returns `ProposalResult`
- `sync_kicad(project_path)` — bidirectional sync

### Model Provider Protocol
- OpenAI-compatible `/v1/chat/completions` with structured output (JSON Schema)
- Streaming via SSE
- Tool calls with kernel-defined function schemas
- Capability negotiation at session start

---

## Acceptance Criteria (V1 Gate)

| Gate | Criteria |
|------|----------|
| **GATE 1: Developer Preview** | Kernel builds, `eak run --deterministic` works, event log replays, CLI trace works. |
| **GATE 2: Real AI Engineering Workflow** | Live model provider works, agent proposes requirement → kernel validates → commits. |
| **GATE 3: Component-Aware Engineering** | Component search returns evidence-backed parts, symbols/footprints generated, provenance tracked. |
| **GATE 4: Schematic-Capable** | Schematic model created, ERC passes, KiCad round-trip works. |
| **GATE 5: PCB-Capable** | PCB model created, DRC passes, KiCad round-trip works. |
| **GATE 6: Verified Engineering Workflow** | Power/SI/EMC/DFM verification runs, confidence scores, human review gate. |
| **GATE 7: Manufacturing Package** | BOM, Gerber, Drill, PnP exported and pass fabricator checks. |
| **GATE 8: Public V1** | Signed installers, docs, examples, onboarding, CI green. |

---

## Next Document

[System Architecture →](../3-system-architecture/README.md)