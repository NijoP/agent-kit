# Electronics Agent Kit (EAK)

**EAK — Electronics Agent Kit: An open-source, AI-native electronics engineering system / engineering runtime.**

**Intent in. Engineering model out. Verified engineering artifacts out.**

EAK owns the canonical engineering truth. The runtime is the source of truth. AI proposes. The EAK kernel validates and commits. UI, KiCad, terminal runtimes, file formats, model providers and external tools are peripherals/adapters.

---

## Vision

EAK is an **engineering runtime** that maintains a canonical, provenance-rich engineering model. It executes deterministic engineering rules, validates AI proposals, and commits only verified state. It produces verified engineering artifacts: schematics, PCBs, BOMs, manufacturing packages.

**Canonical model contains:** intent, requirements, constraints, decisions, assumptions, evidence, components, pins, nets, topology, schematic representation, PCB representation, verification, BOM, manufacturing outputs, provenance, revisions, project history.

---

## Non-Negotiable Architecture Principles

1. **EAK runtime is the source of truth** — No external system holds authoritative engineering state.
2. **External file formats are never the source of truth** — `.kicad_pcb`, `.json`, `.yaml` are import/export formats only.
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

## Product Direction

| Role | System | Relationship to EAK |
|------|--------|---------------------|
| **Engineering System** | EAK | Canonical kernel, owns truth |
| **EDA Implementation Peripheral** | KiCad | Edge integration for schematic/PCB editing, viewing, import/export, rendering |
| **Terminal/Workspace Runtime Peripheral** | Herdr | Persistent sessions, PTY, tabs, panes, agent detection, attach/detach |
| **Development/Orchestration Control Plane** | Firstmate | Agent orchestration, CI, quality gates, PR management |
| **Reasoning Providers** | LLM providers (NVIDIA, OpenAI, local) | Behind model boundary, provide structured proposals |

**For V1:** Use KiCad wherever it substantially accelerates delivery of real schematic/PCB editing, viewing, import/export, or rendering. **But:** EAK state remains canonical. Do not copy KiCad's data model into the EAK kernel. Do not make `.kicad_pcb` the canonical project state. Do not make EAK a thin KiCad wrapper. Use KiCad as an edge implementation where practical.

---

## Workspace Layout (Crate Topology)

```
eak/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── eak-units              # Physical-quantity type system (P9)
│   ├── eak-domain             # Domain entities, opaque EntityId, invariants
│   ├── eak-ports              # Port traits (EventLog, ReasoningEngine, KicadSync, TerminalRuntime) + Event
│   ├── eak-runtime            # KERNEL: state/fold, FSM framework, execution engine, orchestrator, capability handler, replay
│   ├── eak-engines            # Domain engines: Planning, Schematic, PCB, Verification, Manufacturing
│   ├── eak-compiler           # Requirement IR + Engineering IR lowering, codegen
│   ├── eak-phases             # Phase instances: Requirement, Component, Schematic, PCB, Verification, Manufacturing, Handoff agents + FSMs
│   ├── eak-store              # Adapters: append-only JSON-lines event log, SQLite, PostgreSQL
│   ├── eak-reasoning          # Adapters: Fixture + live model provider adapters (NVIDIA, OpenAI, local)
│   ├── eak-cli                # Drivers: `eak` binary + composition root
│   ├── eak-assets             # Component asset management: symbols, footprints, 3D models, datasheets
│   ├── eak-dashboard          # Observability: metrics, tracing, health
│   ├── eak-kicad              # KiCad integration adapter: schematic/PCB import/export, rendering
│   ├── eak-observability      # Structured logging, metrics, distributed tracing
│   └── eak-telemetry          # Telemetry collection (opt-in)
```

**Dependency Rule (enforced at compile time):** Dependencies point only inward toward `eak-runtime`. `eak-runtime` depends only on `eak-ports`, `eak-domain`, `eak-units`. No cycles.

---

## Build & Verify

```sh
# From eak/ directory
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                       # unit + integration + contract + golden
cargo fmt --workspace --check
```

## CLI (CLI)

The `eak` binary provides the following subcommands:

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
- `eak export` — Export manufacturing artifacts (KiCad, Gerber, BOM). *(Planned)*

### Examples

```sh
# Offline, deterministic (built-in fixture): run pipeline, write event log
cargo run --bin eak -- run \
  --intent "USB-C powered IoT sensor node, < 5 W, < 50x50 mm" \
  --log /tmp/eak.jsonl --deterministic

# Replay recorded history into reconstructed state (no model, no clock)
cargo run --bin eak -- replay --log /tmp/eak.jsonl

# Show provenance chain for an entity (short id from run output)
cargo run --bin eak -- trace --log /tmp/eak.jsonl <entity-id>

# Export manufacturing artifacts (planned)
# cargo run --bin eak -- export --format kicad --log /tmp/eak.jsonl --output ./kicad-project
```

### Live Reasoning (Real Model, Recorded Then Replayable)

```sh
export NVIDIA_API_KEY=...   # or OPENAI_API_KEY, or run local Ollama
cargo run --bin eak -- run \
  --intent "..." --reasoning live --model nvidia/nemotron-3-ultra --log /tmp/eak-live.jsonl
```

---

## Desktop App (Tauri)

The Tauri desktop app (`../app/`) is a **projection/editor surface**. It links the kernel as a library, streams live events via Tauri IPC, and provides:
- Project explorer, schematic/PCB/verification dashboards
- Integrated terminal (via Herdr) for agent logs
- KiCad launch/sync buttons
- Manufacturing export UI

```sh
# From app/ directory (requires Tauri v2 prerequisites)
cargo tauri dev
```

---

## Documentation

Canonical documentation lives in [`../docs/`](../docs/):

1. [Product Vision](../docs/1-product-vision/)
2. [Product Specification](../docs/2-product-spec/)
3. [System Architecture](../docs/3-system-architecture/)
4. [Engineering Model](../docs/4-engineering-model/)
5. [Component Architecture](../docs/5-component-architecture/)
6. [Model/Provider Architecture](../docs/6-model-provider-architecture/)
7. [Agent Architecture](../docs/7-agent-architecture/)
8. [Terminal/Herdr Architecture](../docs/8-terminal-herdr-architecture/)
9. [KiCad Integration Architecture](../docs/9-kicad-integration-architecture/)
10. [UI Architecture](../docs/10-ui-architecture/)
11. [Verification Architecture](../docs/11-verification-architecture/)
12. [Project/Persistence Architecture](../docs/12-project-persistence-architecture/)
13. [Development Workflow](../docs/13-development-workflow/)
14. [Testing Strategy](../docs/14-testing-strategy/)
15. [Release Strategy](../docs/15-release-strategy/)
16. [Master Implementation Roadmap](../docs/16-master-implementation-roadmap/)

---

## Development Workflow

**Agent change → tests → lint → docs → AI review → no-mistakes gate → clean branch/PR → merge authority.**

See [Development Workflow](../docs/13-development-workflow/) for details.

### Quality Gate: no-mistakes

```sh
no-mistakes axi run --intent "Execute the 24-hour EAK product maturity reset..."
```

### Evidence-Backed Improvements: backpass

```sh
# After collecting evidence from test runs, reviews
backpass improve --evidence ./evidence --target .claude/agents/
```

---

## Integrations

- **Herdr** (https://github.com/herdrdev/herdr): Terminal/workspace runtime peripheral.
- **Firstmate** (https://github.com/kunchenguid/firstmate): Development control plane.
- **Backpass** (https://github.com/kunchenguid/backpass): Evidence-backed instruction improvements.
- **No-mistakes** (https://github.com/kunchenguid/no-mistakes): Quality gate.

---

## License

MIT OR Apache-2.0 (per crate, see `Cargo.toml`).

---

## Contributing

See [Development Workflow](../docs/13-development-workflow/) and [CONTRIBUTING.md](../CONTRIBUTING.md) (to be created).