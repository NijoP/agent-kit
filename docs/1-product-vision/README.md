# Product Vision: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Vision Statement

**EAK — Electronics Agent Kit: An open-source, AI-native electronics engineering system / engineering runtime.**

**Intent in. Engineering model out. Verified engineering artifacts out.**

EAK owns the canonical engineering truth. The runtime is the source of truth. AI proposes; the EAK kernel validates and commits. UI, KiCad, terminal runtimes, file formats, model providers, and external tools are peripherals/adapters.

---

## What EAK Is

- An **engineering runtime** that maintains a canonical, provenance-rich engineering model.
- A **kernel** that executes deterministic engineering rules, validates AI proposals, and commits only verified state.
- A **platform** for AI agents to collaborate on electronics engineering under strict validation.
- A **system** that produces verified engineering artifacts: schematics, PCBs, BOMs, manufacturing packages.

## What EAK Is Not

- A thin wrapper around KiCad or any EDA tool.
- A chat interface that writes directly to files.
- A replacement for engineering judgment — it amplifies it with verification.

---

## Canonical Engineering Model

The canonical model contains:

- **Intent, requirements, constraints, decisions, assumptions**
- **Evidence** (datasheets, application notes, test reports) with provenance
- **Components** (pins, electrical characteristics, packaging, provenance)
- **Nets, topology, schematic representation, PCB representation**
- **Verification results** (ERC, DRC, power, signal integrity, EMC, DFM)
- **BOM, manufacturing outputs**
- **Provenance, revisions, project history**

Every fact in the model carries provenance: imported vs. synthesized, source, timestamp, agent, confidence.

---

## Non-Negotiable Architecture Principles

1. **EAK runtime is the source of truth** — No external system holds authoritative engineering state.
2. **External file formats are never the source of truth** — .kicad_pcb, .json, .yaml are import/export formats only.
3. **AI never writes directly into authoritative state** — All mutations go through the kernel's validated capability seam.
4. **All AI proposals cross a validated capability seam** — The kernel checks deterministic rules before commit.
5. **Deterministic engineering rules stay deterministic** — Physics, geometry, connectivity rules are pure functions.
6. **Provenance is mandatory** — Every fact knows its origin.
7. **Imported facts are distinguishable from synthesized facts** — Clear lineage: human, datasheet, AI inference, derivation.
8. **Engineering state is replayable** — Event log + deterministic fold = identical state reconstruction.
9. **Physical quantities remain typed** — Units, tolerances, temperature ranges are first-class.
10. **UI is a projection/editor surface, not authority** — UI reads from kernel, proposes via kernel.
11. **Provider/model implementation stays behind a model boundary** — Swap NVIDIA, OpenAI, local models without kernel changes.
12. **Product architecture must remain replaceable and modular** — Crate boundaries enforce dependency direction.

---

## Product Direction (Critical)

| Role | System | Relationship to EAK |
|------|--------|---------------------|
| **Engineering System** | EAK | Canonical kernel, owns truth |
| **EDA Implementation Peripheral** | KiCad | Edge integration for schematic/PCB editing, viewing, import/export, rendering |
| **Terminal/Workspace Runtime Peripheral** | Herdr | Persistent sessions, PTY, tabs, panes, agent detection, attach/detach |
| **Development/Orchestration Control Plane** | Firstmate | Agent orchestration, CI, quality gates, PR management |
| **Reasoning Providers** | LLM providers (NVIDIA, OpenAI, local) | Behind model boundary, provide structured proposals |

**For V1:** Use KiCad wherever it substantially accelerates delivery of real schematic/PCB editing, viewing, import/export, or rendering. **But:** EAK state remains canonical. Do not copy KiCad's data model into the EAK kernel. Do not make `.kicad_pcb` the canonical project state. Do not make EAK a thin KiCad wrapper. Use KiCad as an edge implementation where practical.

Before copying any third-party source code, perform license review and preserve required notices. Prefer integration or adapters over copying large third-party codebases.

---

## Success Criteria (V1)

- An engineer can express intent in natural language and receive a verified, manufacturable PCB design.
- Every engineering decision is traceable to requirements, evidence, and verification.
- The system runs locally, air-gapped capable, with no mandatory cloud dependencies.
- AI agents operate under kernel validation; no hallucinated nets, footprints, or BOM entries reach manufacturing.
- KiCad opens the design for manual refinement; changes sync back to EAK canonical model.

---

## Scope Boundaries

**In scope for V1:**
- Requirements capture, component selection, schematic, PCB, verification, manufacturing outputs.
- Local-first operation with optional cloud model providers.
- KiCad integration for editing/viewing.
- Herdr integration for terminal workspace.
- Firstmate for development orchestration.

**Out of scope for V1:**
- Mechanical/CAD integration (enclosures, thermal).
- Firmware/software co-design.
- Supply chain automation beyond BOM export.
- Multi-user real-time collaboration (single-user local first).

---

## Relationship to Other Systems

- **Herdr** (https://github.com/herdrdev/herdr): EAK integrates Herdr concepts for persistent sessions, workspaces, tabs, panes, PTY terminals, agent detection, attach/detach, session restore, agent-aware state, CLI/socket control, remote operation. EAK derives the necessary runtime boundary from Herdr. Herdr does not own engineering state.
- **Firstmate** (https://github.com/kunchenguid/firstmate): Firstmate is the development control plane for this repository. EAK does not embed Firstmate; Firstmate orchestrates EAK development.
- **Backpass** (https://github.com/kunchenguid/backpass): Used for evidence-backed memory/instruction improvements. Model speculation never rewrites architectural truth.
- **No-mistakes** (https://github.com/kunchenguid/no-mistakes): The quality gate for all EAK development. Agent change → tests → lint → docs → AI review → no-mistakes gate → clean branch/PR → merge authority.

---

## Next Document

[Product Specification →](../2-product-spec/README.md)