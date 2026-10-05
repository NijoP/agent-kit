# Electronics Agent Kit (EAK)

**EAK — Electronics Agent Kit: An open-source, AI-native electronics engineering runtime.**

**Intent in. Engineering model out. Verified engineering artifacts out.**

EAK owns the canonical engineering truth. The runtime is the source of truth. AI proposes; the EAK kernel validates and commits. UI, KiCad, terminal runtimes, file formats, model providers, and external tools are peripherals/adapters.

---

## What EAK Is

- An **engineering runtime** that maintains a canonical, provenance-rich engineering model
- A **kernel** that executes deterministic engineering rules, validates AI proposals, and commits only verified state
- A **platform** for AI agents to collaborate on electronics engineering under strict validation
- A **system** that produces verified engineering artifacts: schematics, PCBs, BOMs, manufacturing packages

## What EAK Is Not

- A thin wrapper around KiCad or any EDA tool
- A chat interface that writes directly to files
- A replacement for engineering judgment — it amplifies it with verification

---

## Quickstart

```bash
# Prerequisites
# - Rust 1.74+ (via rustup)
# - cargo (comes with Rust)

git clone https://github.com/NijoP/agent-kit.git
cd agent-kit/eak
cargo build --workspace
cargo run --bin eak -- --help
```

### Run a deterministic demo (no API keys, no network)

```bash
cd eak
cargo run --bin eak -- run \
  --intent "USB-C powered I2C temperature sensor, < 1 W" \
  --log /tmp/eak-demo.jsonl \
  --deterministic
```

### Replay the exact same run

```bash
cargo run --bin eak -- replay --log /tmp/eak-demo.jsonl
```

### Trace a requirement's provenance

```bash
cargo run --bin eak -- trace --log /tmp/eak-demo.jsonl <entity-id>
```

---

## Architecture

```
eak/
├── crates/
│   ├── eak-units           # Physical-quantity type system (SI, derived, tolerances)
│   ├── eak-domain          # Domain entities: requirements, components, pins, nets, boards, assumptions, risks
│   ├── eak-ports           # Port traits (EventLog, ReasoningEngine, KiCadSync, TerminalRuntime) + Event definitions
│   ├── eak-runtime         # KERNEL: event log, state fold, FSM framework, execution engine, orchestrator, capability handler, replay
│   ├── eak-engines         # Domain engines: planning, schematic, PCB, verification, manufacturing
│   ├── eak-compiler        # Requirement IR + Engineering IR lowering
│   ├── eak-phases          # Phase agents: Requirement, Component, Schematic, PCB, Verification, Manufacturing + FSMs
│   ├── eak-store           # Adapters: JSON-lines event log, SQLite, PostgreSQL
│   ├── eak-reasoning       # Fixture + live model providers (Anthropic, OpenAI-compat, local)
│   ├── eak-cli             # `eak` binary + composition root
│   ├── eak-assets          # Component assets: symbols, footprints, 3D models, datasheets
│   ├── eak-kicad           # KiCad integration: schematic/PCB import/export, rendering
│   └── eak-observability   # Structured logging, metrics, tracing
└── app/                    # Tauri desktop app (separate workspace; native UI + terminal bridge)
```

**Dependency rule (enforced at compile time):** Dependencies point only inward toward `eak-runtime`. `eak-runtime` depends only on `eak-ports`, `eak-domain`, `eak-units`. No cycles.

---

## Key Capabilities (Implemented & Tested)

| Capability | Status | Evidence |
|------------|--------|----------|
| Event-sourced canonical state | ✅ Implemented | `eak-runtime` commit/fold/replay |
| Byte-identical deterministic replay | ✅ Tested | `hero_flow` integration test |
| 15-phase engineering workflow | ✅ Implemented | `eak-phases` + `default_workflow()` |
| Agent proposal → validation → commit | ✅ Implemented | `CapabilityRequest` seam |
| Requirement → decision → verification traceability | ✅ Implemented | `ProvenanceLink` + `Assumption`/`Risk`/`Tradeoff` |
| KiCad import (PCB) + export | ✅ Partial | `eak-kicad` + integration tests |
| Component datasheet parsing | ✅ Implemented | `eak-assets` parser + cross-check |
| Model provider abstraction | ✅ Implemented | `ReasoningEngine` port (fixture, Anthropic, OpenAI-compat) |
| CLI for agent integration | ✅ Implemented | `eak run/replay/trace/export/asset-*` |
| Assumptions as first-class objects | ✅ Implemented | Manufacturing gate blocks on undischarged critical assumptions |

---

## Desktop App (Tauri)

```bash
cd app
# Requires Tauri v2 prerequisites (webkit2gtk on Linux, Xcode on macOS, VS Build Tools on Windows)
cargo tauri dev
```

The desktop app bridges the Rust kernel to a React/TypeScript UI via Tauri IPC, streaming live kernel events to the frontend.

---

## Documentation

Canonical documentation lives in [`docs/`](docs/):

1. [Product Vision](docs/1-product-vision/)
2. [Product Specification](docs/2-product-spec/)
3. [System Architecture](docs/3-system-architecture/)
4. [Engineering Model](docs/4-engineering-model/)
5. [Component Architecture](docs/5-component-architecture/)
6. [Model/Provider Architecture](docs/6-model-provider-architecture/)
7. [Agent Architecture](docs/7-agent-architecture/)
8. [Terminal/Herdr Architecture](docs/8-terminal-herdr-architecture/)
9. [KiCad Integration Architecture](docs/9-kicad-integration-architecture/)
10. [UI Architecture](docs/10-ui-architecture/)
11. [Verification Architecture](docs/11-verification-architecture/)
12. [Project/Persistence Architecture](docs/12-project-persistence-architecture/)
13. [Development Workflow](docs/13-development-workflow/)
14. [Testing Strategy](docs/14-testing-strategy/)
15. [Release Strategy](docs/15-release-strategy/)
16. [Master Implementation Roadmap](docs/16-master-implementation-roadmap/)

Development roadmap (this repo's source of truth):
[`docs/development/EAK_DEVELOPMENT_ROADMAP.html`](docs/development/EAK_DEVELOPMENT_ROADMAP.html)

---

## License

Licensed under **MIT OR Apache-2.0** at your option.

See [LICENSE](LICENSE) for full text.

---

## Contributing

See [Development Workflow](docs/13-development-workflow/) and [CONTRIBUTING.md](CONTRIBUTING.md) (to be created).

Quality gate: **no-mistakes** — agent change → tests → lint → docs → AI review → no-mistakes gate → clean branch/PR → merge authority.

---

## Related Projects

- **Herdr** — Terminal/workspace runtime peripheral: https://github.com/herdrdev/herdr
- **Firstmate** — Development orchestration control plane: https://github.com/kunchenguid/firstmate
- **No-mistakes** — Quality gate: https://github.com/kunchenguid/no-mistakes
- **KiCad** — EDA implementation peripheral: https://gitlab.com/kicad/code/kicad

---

## Status

**Active development.** See the [Development Roadmap](docs/development/EAK_DEVELOPMENT_ROADMAP.html) for current implementation state, blockers, and launch criteria.

This is an engineering runtime prototype. The deterministic kernel, event-sourced state, and 15-phase workflow are implemented and tested. The desktop UI, KiCad integration, and live model providers are in progress.

**No AI model required for core workflow** — deterministic fixture engine runs the full pipeline offline.