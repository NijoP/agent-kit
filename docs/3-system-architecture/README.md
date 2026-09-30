# System Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document describes the high-level system decomposition, runtime boundaries, data flows, and crate topology of EAK.

---

## Architectural Style

**Event-sourced, capability-based, kernel-centric.**

- **Event sourcing**: All state changes are appended as events to an immutable log. State is derived by deterministic fold.
- **Capability-based security**: Agents interact with the kernel only through validated capability seams. No direct state mutation.
- **Kernel-centric**: The kernel (`eak-runtime`) is the single authority for engineering state. All other crates are adapters, engines, or drivers.

---

## Crate Topology (Workspace: `eak/`)

```
eak-units          # Physical-quantity type system (P9)
eak-domain         # Domain entities, opaque EntityId, invariants
eak-ports          # Port traits (EventLog, ReasoningEngine, KiCadSync, etc.) + Event definitions
eak-runtime        # KERNEL: state/fold, FSM framework, execution engine, orchestrator, capability handler, replay
eak-engines        # Domain engines: Planning, Schematic, PCB, Verification, Manufacturing
eak-compiler       # Requirement IR + Engineering IR lowering, codegen
eak-phases         # Phase instances: Requirement Agent, Schematic Agent, PCB Agent, etc. + FSMs
eak-store          # Adapters: append-only JSON-lines event log, SQLite, PostgreSQL
eak-reasoning      # Adapters: Fixture + live model provider adapters (NVIDIA, OpenAI, local)
eak-cli            # Drivers: `eak` binary + composition root
eak-assets         # Component asset management: symbols, footprints, 3D models, datasheets
eak-dashboard      # Observability: metrics, tracing, health
eak-kicad          # KiCad integration adapter: schematic/PCB import/export, rendering
eak-observability  # Structured logging, metrics, distributed tracing
eak-telemetry      # Telemetry collection (opt-in)
```

**Dependency Rule (enforced at compile time):**
- Dependencies point **only inward** toward `eak-runtime`.
- `eak-runtime` depends only on `eak-ports`, `eak-domain`, `eak-units`.
- No cycles. Verified by `cargo-depgraph` in CI.

---

## Runtime Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                        EAK RUNTIME (Kernel)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │ Event Log   │  │ State Fold  │  │ Capability  │             │
│  │ (append-only)│  │ (deterministic)│ │ Handler     │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│         ▲               ▲               ▲                       │
│         │               │               │                       │
│  ┌──────┴──────┐ ┌──────┴──────┐ ┌──────┴──────┐               │
│  │  Engines    │ │  Phases     │ │  Adapters   │               │
│  │ (planning,  │ │ (agents,    │ │ (store,     │               │
│  │  schematic, │ │  FSMs)      │ │  reasoning, │               │
│  │  pcb, etc.) │ │             │ │  kicad,     │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
└─────────────────────────────────────────────────────────────────┘
         ▲               ▲               ▲               ▲
         │               │               │               │
    ┌────┴────┐    ┌─────┴─────┐   ┌─────┴─────┐   ┌────┴────┐
    │  CLI    │    │  Tauri    │   │  Herdr    │   │  KiCad  │
    │ Driver  │    │  App      │   │  Adapter  │   │ Adapter │
    └─────────┘    └───────────┘   └───────────┘   └─────────┘
```

---

## Data Flow: Intent to Artifact

```
1. USER/AGENT
      │
      ▼
2. INTENT (natural language) ──► REQUIREMENTS ENGINE (eak-engines)
      │                                 │
      │  Structured Requirements         │
      ▼                                 ▼
3. COMPONENT ENGINE ◄─── EVIDENCE INGESTION (eak-assets + eak-reasoning)
      │
      ▼
4. SCHEMATIC ENGINE ──► SCHEMATIC MODEL (canonical)
      │                     │
      │           ┌─────────┴─────────┐
      ▼           ▼                   ▼
   ERC          KiCad Sync           Export
      │           │                   │
      ▼           ▼                   ▼
5. PCB ENGINE ──► PCB MODEL (canonical) ◄─── KiCad Sync
      │                     │
      ▼                     ▼
   DRC                  Verification Engines
      │                     │
      ▼                     ▼
6. MANUFACTURING ENGINE ──► BOM, Gerber, Drill, PnP
```

All transitions are **events** appended to the log. The kernel folds events into the canonical model.

---

## Kernel Responsibilities (`eak-runtime`)

1. **Event Log** — Append-only, immutable, content-addressed (blake3). Supports multiple backends (file, SQLite, PostgreSQL).
2. **State Fold** — Pure function `fold: (State, Event) -> State`. Deterministic, no side effects.
3. **FSM Framework** — Generic finite-state machine executor for agent phases.
4. **Execution Engine** — Schedules phases, handles retries, timeouts, compensation.
5. **Orchestrator** — Coordinates multi-agent workflows (lead agent delegates to specialists).
6. **Capability Handler** — Validates and executes capability invocations (the only way to mutate state).
7. **Replay** — Reconstructs state from log; supports time-travel and branching.

---

## Capability Seams (Validated Entry Points)

Every AI proposal must cross a capability seam. The kernel defines capabilities as Rust traits in `eak-ports`:

| Capability | Purpose | Validation |
|------------|---------|------------|
| `RequirementProposal` | Propose new/modified requirements | Schema validation, traceability |
| `ComponentSelection` | Select/suggest components | Evidence check, rule validation |
| `SchematicEdit` | Add/modify/delete schematic elements | ERC, connectivity, netlist integrity |
| `PcbEdit` | Placement, routing, stackup changes | DRC, constraint checking |
| `VerificationRequest` | Run verification analysis | Deterministic rule execution |
| `ManufacturingOutput` | Generate export artifacts | Format validation, completeness |
| `KicadSync` | Import/export KiCad projects | Round-trip fidelity |

Each capability has a **deterministic validation function** that runs before commit. If validation fails, the event is rejected (not appended).

---

## Event Schema

```rust
struct EventRecord {
    seq: u64,                    // Monotonic sequence number
    timestamp: DateTime<Utc>,    // Wall-clock (for display only)
    event: Event,                // Enum of all domain events
    provenance: Provenance,      // Source: Human, Agent, Import, Derived
    correlation_id: Uuid,        // Links related events (e.g., one agent turn)
}

enum Event {
    RequirementAdded(Requirement),
    ComponentSelected(ComponentRef),
    SchematicElementAdded(SchematicElement),
    PcbElementPlaced(PcbElement),
    VerificationRun(VerificationResult),
    // ... 50+ variants
}
```

Events are **serialized as JSON Lines** (one JSON object per line) for storage and streaming.

---

## Persistence Adapters (`eak-store`)

| Backend | Use Case |
|---------|----------|
| JSON Lines (file) | Local development, replay, air-gapped |
| SQLite | Embedded, single-user, ACID |
| PostgreSQL | Multi-user, remote, Herdr session backend |

All backends implement `EventLog` port trait. Kernel is backend-agnostic.

---

## Model Provider Adapters (`eak-reasoning`)

| Provider | Integration |
|----------|-------------|
| Fixture (deterministic) | Testing, replay, CI |
| NVIDIA NIM | Production, structured output, streaming |
| OpenAI-compatible | Generic endpoint (Ollama, vLLM, etc.) |
| Local (llama.cpp) | Air-gapped, privacy |

All implement `ReasoningEngine` port trait. Kernel sees only the trait.

---

## KiCad Integration Adapter (`eak-kicad`)

- **Import**: Parse `.kicad_sch`, `.kicad_pcb` → EAK canonical model (lossy, flagged as imported).
- **Export**: Serialize EAK canonical model → `.kicad_sch`, `.kicad_pcb` (round-trip for supported subset).
- **Rendering**: Headless KiCad CLI for PNG/PDF/3D views.
- **Editing**: Launch KiCad GUI with project file; on save, detect changes and sync back via `KicadSync` capability.

**Critical**: KiCad files are **never** the source of truth. They are edge representations.

---

## Tauri Desktop App (`app/`)

- **Backend**: `eak-cli` linked as library, runs kernel in-process.
- **Frontend**: React/TypeScript (in `app/ui/`), receives live `EventRecord` stream via Tauri events.
- **IPC**: `start_run`, `get_state`, `propose_capability`, `sync_kicad`.
- **Not in `eak/` workspace** — Tauri system deps (webkit2gtk) must not gate kernel build.

---

## Herdr Integration (`herdr` crate, planned)

- Embed Herdr PTY/session manager as a library.
- Expose Herdr sessions as EAK workspaces.
- Agent detection: Herdr knows which pane runs which agent.
- Attach/detach: Engineers can attach to running agent sessions.
- Session restore: Herdr restores PTY state; EAK restores kernel state from log.

---

## Security Boundaries

- **Kernel**: Runs with user privileges, no sandbox (local-first).
- **Model Providers**: Network calls only from `eak-reasoning` adapters. Kernel never makes network calls.
- **KiCad**: Launched as separate process, communicates via file system + CLI.
- **Herdr**: PTY management requires appropriate permissions.

---

## Observability (`eak-observability`, `eak-dashboard`)

- Structured logging (tracing) with correlation IDs.
- Metrics: event throughput, validation latency, verification duration.
- Distributed tracing across kernel, agents, adapters.
- Dashboard: live engineering state view, verification results, provenance graph.

---

## Next Document

[Engineering Model →](../4-engineering-model/README.md)