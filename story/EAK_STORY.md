# EAK — The Complete Story

## Electronics Agent Kit: VS Code for Electronics

---

## Table of Contents

1. [The Origin Story](#the-origin-story)
2. [The Problem: Why Traditional EDA Tools Fail Engineers](#the-problem-why-traditional-eda-tools-fail-engineers)
3. [The New Paradigm: EAK is VS Code for Electronics](#the-new-paradigm-eak-is-vs-code-for-electronics)
4. [The Vision: "Intent In. Manufactured Board Out."](#the-vision-intent-in-manufactured-board-out)
5. [The PRD-First Workflow](#the-prd-first-workflow)
6. [AI Agents: The Claude Code of Electronics](#ai-agents-the-claude-code-of-electronics)
7. [The Custom Model Vision](#the-custom-model-vision)
8. [The Novelty: 7-Touchpoint Verification Seam](#the-novelty-7-touchpoint-verification-seam)
9. [Architecture: Clean Rings, Pure Rust](#architecture-clean-rings-pure-rust)
10. [Tech Stack: Modern, Memory-Safe, Yours](#tech-stack-modern-memory-safe-yours)
11. [The Verification Kernel: The Moat](#the-verification-kernel-the-moat)
12. [Database & Library: The Authoritative Data Layer](#database--library-the-authoritative-data-layer)
13. [Workflow: How Engineers Create Gold-Standard PCBs in EAK](#workflow-how-engineers-create-gold-standard-pcbs-in-eak)
14. [The Editor: Where Engineers Live](#the-editor-where-engineers-live)
15. [Business Model: Solo YC-Style Startup](#business-model-solo-yc-style-startup)
16. [Development Roadmap: From Tool to Platform](#development-roadmap-from-tool-to-platform)
17. [What Makes EAK "Gold" for Electronics Engineers](#what-makes-eak-gold-for-electronics-engineers)
18. [Current Status & Next Steps](#current-status--next-steps)
19. [The Complete Repository Structure](#the-complete-repository-structure)
20. [The One-Liner](#the-one-liner)

---

## The Origin Story

### The Spark

Every electronics engineer knows this feeling:

You send a board to fab. Two weeks later, a purple PCB arrives. You populate it, power it up, and... nothing. Or worse — it *almost* works. The I²C bus is flaky. The 3V3 rail sags under load. The clock tree has a glitch that only shows up on Tuesday afternoons.

You respin. $5K. Three weeks. Again.

**Every electronics engineer has lived this story.**

### The Founding Insight

I'm an electronics engineer. I've spent 10+ years designing boards for aerospace, medical, and consumer electronics. I've watched software engineers transform their craft — and I saw something they have that we don't.

**Software engineers used to write every line of code by hand. Now they write a PRD, and AI agents like Claude Code and ChatGPT write the code for them. The engineer becomes an orchestrator — reviewing, guiding, catching mistakes, steering the project.**

They have:

- A **PRD-first workflow**: plan before you build
- **AI agents** that do the heavy lifting (Claude Code, Cursor, GitHub Copilot)
- **Live development**: watch the code being written in real time
- **AI recommendations**: the tool flags issues and suggests fixes
- **Orchestration**: the engineer steers the whole project like a conductor

**Electronics has none of this.**

We still draw schematics manually. We still check power budgets by hand. We still have "Pray DRC" workflows. We still build boards the way software engineers wrote code in 1995 — before the AI revolution.

**EAK was born from one question:**

> *What if electronics engineers got the same transformation software engineers got? What if a tool gave us PRD-first design, AI agents that build the schematic and PCB, live development, and orchestration — all in one place?*

### The Founding Moment

**EAK is not a traditional EDA tool. It is the clubbing of VS Code + EDA tool.**

It's the first environment where electronics design works exactly like modern software development:

```
SOFTWARE ENGINEERING (Today)          ELECTRONICS (Today)
────────────────────────────────      ──────────────────────────────
1. Write PRD (.md)                    1. Open KiCad/Altium
2. AI agent writes code               2. Manually place components
3. Engineer reviews + orchestrates    3. Manually draw wires
4. Live development + AI feedback     4. Manually check power/clock/bus
5. Ship fast, iterate fast            5. Pray DRC → fab → respin

                     ↓ THE EAK TRANSFORMATION ↓

1. Write PRD (.md) with AI help        ← exactly like software
2. AI agent builds schematic + PCB     ← the Claude Code moment
3. Engineer reviews + orchestrates     ← live development view
4. EAK + AI catch mistakes, suggest    ← AI recommendations
5. Ship a verified board, fast         ← zero-respin, less time
```

---

## The Problem: Why Traditional EDA Tools Fail Engineers

### The State of EDA Today

| Tool | Category | The Gap |
|------|----------|---------|
| **KiCad** | Open Source | Geometry-only DRC, no intent verification, no AI, manual everything |
| **Altium Designer** | Commercial ($10K+/yr) | Geometry DRC + some ERC, but black-box, no traceability, no AI |
| **Cadence Allegro** | Enterprise ($50K+/yr) | Powerful but monolithic, expensive, closed, no AI |
| **Flux.ai** | Cloud/Web | Schematic capture + basic DRC, no deep verification, no orchestration |
| **EasyEDA** | Cloud/Web | Hobbyist-focused, limited verification, no AI |

### The Universal Failure Mode

**Every EDA tool on the market is a *geometry checker*, not an *intent verifier*.** And none of them are *AI agents*.

| What They Check | What They Miss |
|-----------------|----------------|
| Trace width > 6mil? | Does this power domain balance? (KCL) |
| Clearance > 6mil? | Does this clock domain cross safely? (CDC) |
| Nets connected? | Does this signal have a return path? (TL theory) |
| No overlapping copper? | Does this I²C bus have unique addresses? |
| Pad clearance OK? | Does this pin mux assignment conflict? |
| Net connectivity? | Does this signal driver actually drive? |

**The result:** 70% of PCB respins are caused by *schematic-level errors* that DRC never catches. The errors are in **design intent**, not geometry.

### The Cost of Failure

| Failure Type | Typical Cost | Time Lost |
|--------------|--------------|-----------|
| Power budget wrong | $5K + 3 weeks | Respins |
| Clock domain crossing | $10K + 4 weeks | Debug + respin |
| Missing return path | $15K + 6 weeks | EMC failure + respin |
| I²C address conflict | $3K + 2 weeks | Rework + respin |
| Pin mux conflict | $5K + 3 weeks | Respins |

**Industry average: 2.3 respins per board. Average cost: $12K + 4 weeks per board.**

### The Missing Revolution

Software engineering had its revolution: **PRD → AI agent → code → orchestrate**. Electronics is still in the pre-AI era.

**EAK is the revolution.**

---

## The New Paradigm: EAK is VS Code for Electronics

### The Core Insight

> **Software engineers don't build software by hand anymore. They write PRDs, let AI agents build, and orchestrate the result. Electronics engineers should work the same way — and EAK makes that possible.**

### VS Code for Electronics

Just as VS Code became the universal home for software engineers — where they write code, run agents, see live diffs, get AI suggestions, and orchestrate their whole project — **EAK is that same home for electronics engineers**.

| VS Code (Software) | EAK (Electronics) |
|---------------------|---------------------|
| Project folder tree | **PRD folder tree (markdown files)** |
| `.md` PRD files | **`.md` PRD files — same format** |
| Claude Code / Cursor agents | **EAK agents (custom models)** |
| Live diff of code being written | **Live schematic/PCB being drawn** |
| Inline error squiggles | **Inline ERC/DRC squiggles on nets** |
| AI recommendations in terminal | **AI recommendations in verification panel** |
| Engineer orchestrates agent | **Engineer orchestrates schematic + PCB agents** |
| Git history | **Deterministic replay log** |
| CI/CD pipeline | **Verification pipeline (ERC → DRC → DFM → EMC)** |
| Tests in CI | **Physics rules in verification** |

### The Engineer's New Identity

**The electronics engineer becomes an orchestrator — exactly like a software engineer orchestrating Claude Code.**

- You don't draw every wire by hand.
- You don't place every component manually.
- You write the **PRD** — the intent, the requirements, the constraints.
- The **AI agent** builds the schematic and PCB from that PRD.
- You **watch it live**, catch mistakes, point them out.
- The agent **fixes them** and continues.
- EAK's verification kernel runs in the background, flagging physics violations.
- The final board is **verified, mistake-free, in a fraction of the time**.

---

## The Vision: "Intent In. Manufactured Board Out."

### The Vision Statement

> **Intent In. Manufactured Board Out.**
>
> You describe what the circuit *should do* — in plain language and structured PRDs. EAK's AI agents turn that intent into a schematic and PCB. EAK's verification kernel proves it against physics *while it's being built*. The output isn't just a netlist — it's a **verified, manufacturable board**, built by agents, orchestrated by you.

### What "Intent In" Means

You declare **intent** in a PRD — just like a software engineer writes a product requirements document:

```markdown
# PRD: I2C Environmental Sensor Hub

## 1. Objective
Build a sensor hub that reads temperature, humidity, and pressure
over I2C and reports via USB-C. Target: 5 cm × 5 cm board.

## 2. Requirements
### 2.1 Power
- Input: USB-C 5V
- 3V3 rail: LDO, budget 250 mA, tolerance ±5%

### 2.2 Clock
- MCU clock: 64 MHz (internal PLL)
- I2C clock: 400 kHz (fast mode)

### 2.3 Interfaces
- I2C bus 1: MCU as master, 2 sensor slaves
- USB 2.0 device: MCU to host

### 2.4 Sensors
- Temperature: SHT40 (I2C address 0x44)
- Humidity: SHT40 (I2C address 0x44)
- Pressure: BMP388 (I2C address 0x77)

## 3. Constraints
- Board size: 50 × 50 mm
- 2-layer stackup, 1.6 mm FR-4
- Controlled impedance: none required (all < 100 MHz)
- Operating temp: 0 °C to 70 °C
```

**That's it.** That's the entire design input. No dragging, no wiring, no hunting for footprints.

### What "Manufactured Board Out" Means

The output isn't just Gerbers — it's a **verified, manufacturable design package** built by agents and certified by the kernel:

| Output | What It Contains |
|--------|------------------|
| **Schematic** | Auto-generated from PRD, verified |
| **PCB Layout** | Auto-generated, DRC-clean, verified |
| **Gerber/ODB++/IPC-2581** | Manufacturing-ready fabrication data |
| **Verification Report** | Every check passed, with ADR-linked rationale |
| **Traceability Matrix** | Requirement → Domain Object → Rule → Finding → Fix |
| **Replay Log** | Byte-identical deterministic replay of entire design |
| **BOM** | Sourced, with supplier links and stock levels |

---

## The PRD-First Workflow

### The Software Engineering Model, Applied to Electronics

This is the heart of EAK. The workflow mirrors exactly what software engineers do today:

```
SOFTWARE ENGINEER (Claude Code era)          EAK ENGINEER
─────────────────────────────────          ─────────────────────────────
1. Understand client requirement            1. Understand client requirement
2. Write PRD (.md)                          2. Write PRD (.md) — with AI help
3. Hand PRD to Claude Code                  3. Hand PRD to EAK agent
4. Agent writes code                        4. Agent builds schematic + PCB
5. Engineer watches live, reviews           5. Engineer watches live, reviews
6. Engineer points out mistakes             6. Engineer points out mistakes
7. Agent fixes, iterates                    7. Agent fixes, iterates
8. Agent suggests improvements              8. EAK + AI suggest improvements
9. Ship                                      9. Ship verified board
```

### The PRD Folder Tree

Just like VS Code shows a folder tree of markdown PRDs, EAK shows the electronics project as a folder tree of markdown files:

```
my-sensor-hub/
├── PRD.md                    ← the master product requirements document
├── prd/
│   ├── power.md              ← power architecture PRD
│   ├── clock.md              ← clock architecture PRD
│   ├── interfaces.md         ← interface/bus PRD (I2C, SPI, USB)
│   ├── sensors.md            ← sensor selection + integration PRD
│   ├── mechanical.md         ← enclosure, mounting, board size
│   └── compliance.md         ← EMC/CE/FCC targets
├── generated/                ← what the agents built (read-only view)
│   ├── schematic/
│   ├── pcb/
│   └── bom/
└── reports/
    ├── verification.md
    ├── drc.md
    └── manufacturing.md
```

**The engineer sees exactly what a software engineer sees: a folder tree of markdown files.** The schematic and PCB are *generated artifacts* — like compiled code — built by agents from those PRDs.

### The PRD Is the Single Source of Truth

| Traditional EDA | EAK |
|-----------------|-----|
| Schematic is the source of truth | **PRD (markdown) is the source of truth** |
| Board is built by manual drawing | **Board is generated from PRD by agents** |
| Errors found after fab | **Errors caught during generation (kernel)** |
| No rationale for design choices | **PRD documents every choice** |
| Hard to review | **Review the PRD, not the wiring** |
| Respin to fix | **Edit PRD → regenerate → re-verify** |

---

## AI Agents: The Claude Code of Electronics

### The Agent Architecture

EAK embeds AI agents that work exactly like Claude Code — but for electronics:

```
┌─────────────────────────────────────────────────────────┐
│                    EAK AGENT LAYER                        │
│                                                          │
│  ┌──────────────────────────────────────────────────┐    │
│  │            ENGINEER (ORCHESTRATOR)                │    │
│  │  - writes PRD (.md)                              │    │
│  │  - reviews agent output live                     │    │
│  │  - points out mistakes: "this is wrong"          │    │
│  │  - steers the project                            │    │
│  └──────────────────┬───────────────────────────────┘    │
│                     │                                     │
│  ┌──────────────────▼───────────────────────────────┐    │
│  │            EAK AGENT (LLM-powered)                │    │
│  │  - reads PRD markdown files                      │    │
│  │  - plans schematic architecture                   │    │
│  │  - generates schematic netlist                   │    │
│  │  - generates PCB placement + routing             │    │
│  │  - uses EAK library (3,500+ parts)               │    │
│  │  - self-checks against EAK verification kernel   │    │
│  └──────────────────┬───────────────────────────────┘    │
│                     │                                     │
│  ┌──────────────────▼───────────────────────────────┐    │
│  │          EAK VERIFICATION KERNEL                  │    │
│  │  - 9 physics rules (KCL, CDC, TL, Thévenin)      │    │
│  │  - deterministic, ADR-cited                      │    │
│  │  - flags violations as the agent works           │    │
│  └──────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### How the Agent Works

```
ENGINEER: "Build me the sensor hub from PRD.md"

AGENT:
  → Reads PRD.md, prd/*.md
  → Plans: "I need a USB-C connector, LDO, MCU, I2C sensors"
  → Queries library: STM32G0 (MCU), SHT40, BMP388, AMS1117 (LDO)
  → Places components on schematic canvas (LIVE — you watch it)
  → Draws wires: USB, I2C bus, power rails (LIVE)
  → Runs verification: power balance, I2C contract, bus topology
  → Reports: "Power balance 62% — safe. I2C addresses unique — safe."

ENGINEER (pointing at screen): "No wait — the SHT40 humidity sensor
  and SHT40 temp sensor are the same part. There's only one SHT40,
  not two. Use SHT41 for humidity."

AGENT: "You're right. Replacing temp sensor with SHT40, humidity with
  SHT41. Adjusting I2C addresses accordingly."
  → Modifies schematic LIVE
  → Re-runs verification
  → "Now: SHT40 @ 0x44, SHT41 @ 0x44? Conflict — moving SHT41 to 0x45
    using its ADDR pin tied high."

ENGINEER: "Looks good. Route the board now."

AGENT:
  → Places footprints (LIVE)
  → Routes: power first, then I2C, then USB diff pair
  → Runs DRC: clearance, width, via count — all green
  → "Board done. 50×50 mm, 2-layer, DRC clean. Verification: 9/9 pass."

ENGINEER: "Perfect. Export Gerbers and BOM."

AGENT: → Writes Gerbers, BOM, verification report.
       → "Done. Ready for fab."
```

### The Live Development View

**This is the magic moment.** Just as software engineers watch Claude Code write code line by line in real time, EAK engineers watch the agent build the schematic and PCB live:

- **Components appear on the canvas** as the agent places them
- **Wires draw themselves** across the schematic
- **Footprints pop onto the PCB** as the agent lays them out
- **Ratsnest updates** in real time
- **Verification panel updates** live: `✅ Power 62%`, `✅ I2C unique`, `⚠️ Clock jitter marginal`

**The engineer can stop the agent at any moment, point at anything, and say: "No, that's wrong."**

### Pointing Out Mistakes

The engineer is the conductor. When the agent makes a mistake:

```
ENGINEER: "Stop. The I2C pull-ups are 10k — too weak for 400 kHz
  fast mode with 2 sensors. Use 2.2k."

AGENT: "Correct. I2C fast mode with ~50 pF bus capacitance needs
  ~2.2 kΩ pull-ups for the rise time spec (tr ≤ 300 ns).
  Replacing R1, R2 with 2.2 kΩ. Re-running verification."
```

**EAK also catches mistakes itself** — via the verification kernel:

```
AGENT: "Route complete."
EAK KERNEL: ⚠️ Finding (erc-signal-driver-sink): Signal "I2C1_SDA"
  names source pin P3 (an Input) — that pin cannot drive, so the
  bus has no operating point.
  → Suggested fix: swap P3 ↔ P4 (ADRs: 0026, circuit-theory.md L134)

ENGINEER: "OK agent, apply that fix."
AGENT: "Applied. Signal now drives from P4 (open-drain, Output).
  Re-verified: 9/9 pass."
```

---

## The Custom Model Vision

### The Long-Term Vision: EAK's Own Model

**This is the big vision.** Just as Anthropic has Claude, OpenAI has GPT, and Kimi/Moonshot has its own models — **EAK will have its own custom-trained model for electronics.**

### Phase 1: API-Key Models (Launch)

Engineers bring their own keys for the first phase:

| Provider | Model | Use in EAK |
|----------|-------|------------|
| Anthropic | Claude Opus / Sonnet | Architecture, schematic generation |
| OpenAI | GPT-4o / o-series | Schematic generation, routing |
| Kimi (Moonshot) | Kimi models | Cost-effective schematic generation |
| Local | Ollama / llama.cpp | Privacy-focused, offline design |

The EAK agent layer is **model-agnostic** — it talks to a unified agent interface, and any compatible LLM can power it.

### Phase 2: Fine-Tuned Models

As EAK gathers design data (with user consent), it fine-tunes open-weight models (Llama, Qwen, DeepSeek) on:

- **Schematic patterns** — how good engineers structure power, clock, buses
- **Library knowledge** — 3,500 parts, their quirks, best use
- **Verification knowledge** — the 7-touchpoint seam, the physics rules
- **PRD-to-schematic translation** — natural language → verified netlist

### Phase 3: The EAK Model

**The endgame:** a foundation model trained specifically for electronics engineering:

- Trained on millions of verified designs (schematic → netlist → PCB → rules)
- Distilled from the EAK verification kernel — it *knows physics*, not just tokens
- Can take a PRD and produce a **first-pass schematic that is 90% correct**, because it was trained on the kernel's correctness criteria
- **Self-verifying**: it runs its own output through the kernel and fixes itself

```
TODAY:  Engineer writes PRD → Claude/Opus/Sonnet via API → agent builds
TOMORROW: Engineer writes PRD → EAK Model (trained on physics) → builds + self-verifies
```

### Why the EAK Model Wins

| Generic LLM | EAK Model |
|-------------|-----------|
| Trained on all of the internet | Trained on **verified electronics designs** |
| Doesn't know KCL/Thévenin deeply | **Distilled from physics rules** |
| Hallucinates pin numbers | **Grounded in 3,500-part library** |
| Produces plausible-but-wrong | **Self-checks against kernel, fixes itself** |
| Generic coding assistant | **Purpose-built for PRD → PCB** |

---

## The Novelty: 7-Touchpoint Verification Seam

### The Architectural Moat

This is EAK's **core innovation** — the architectural pattern that makes agent-built designs trustworthy.

Every domain object in EAK passes through **7 verification gates** before it becomes part of the design:

```
┌─────────────┐
│  1. DOMAIN  │  ← Structural validation (validate())
│  (Entity)   │     "Does this PowerDomain have a supply net?"
└──────┬──────┘
       ↓
┌─────────────┐
│  2. EVENT   │  ← Immutable log entry
│  (Log)      │     "PowerDomainCommitted { domain }"
└──────┬──────┘
       ↓
┌─────────────┐
│  3. STATE   │  ← Deterministic fold
│  (Store)    │     "EngineeringState.power_domains.push()"
└──────┬──────┘
       ↓
┌─────────────┐
│  4. SEAM    │  ← Re-validate + Referential integrity
│  (API)      │     "CreatePowerDomain: net exists? pin exists?"
└──────┬──────┘
       ↓
┌─────────────┐
│  5. VERIFY  │  ← Rules engine (engineering science)
│  (Rules)    │     "PowerBalanceRule: KCL satisfied?"
└──────┬──────┘
       ↓
┌─────────────┐
│  6. PHASE   │  ← Orchestrated verification
│  (Flow)     │     "ERC → if fail → loop back to agent"
└──────┬──────┘
       ↓
┌─────────────┐
│  7. ADR     │  ← Architectural decision record
│  (Reason)   │     "ADR-0022: PowerDomain balance = KCL"
└─────────────┘
```

### Why This Changes Everything for AI Agents

The 7-touchpoint seam is what makes **AI-built electronics trustworthy**:

| Without the Seam (today's AI tools) | With the Seam (EAK) |
|-------------------------------------|----------------------|
| Agent draws schematic, looks fine | Agent's every object passes 7 gates |
| Agent's output is unverifiable | **Every object validated + logged + verified** |
| Errors hide until fab | **Physics rules flag them live** |
| No traceability | **Requirement → ADR → Rule → Finding → Fix** |
| Agent hallucinates | **Referential integrity: pins, nets, buses must exist** |
| Can't audit the agent | **Deterministic replay: replay every agent action** |

**The kernel is the referee for the agent.** The agent can be creative — but the kernel makes sure it obeys physics.

---

## Architecture: Clean Rings, Pure Rust

### The Clean Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    EAK WORKSPACE                              │
├─────────────────────────────────────────────────────────────┤
│  FRONTEND (VS Code-style IDE)    │  AGENT LAYER               │
├─────────────────────────────────────────────────────────────┤
│  eak-gui (egui)                  │  eak-agent                 │
│  - PRD folder tree (markdown)    │  - Model-agnostic interface│
│  - Markdown PRD editor           │  - Anthropic / OpenAI /    │
│  - Schematic canvas (live)       │    Kimi / Ollama adapters  │
│  - PCB canvas (live)             │  - Agent loop (plan→act→   │
│  - Verification panel            │    verify→fix)             │
│  - Chat/terminal panel           │  - Tool calls: place, wire,│
│  - Live diagnostics squiggles    │    route, verify, library  │
├─────────────────────────────────────────────────────────────┤
│  eak-schematic                   │  eak-core (domain models)  │
│  - Netlister                     │  - EntityId, DomainError   │
│  - Component model               │  - PowerDomain, ClockDomain│
│  - Symbol editor                 │  - ReturnPath, Signal      │
│  - Hierarchy (planned)           │  - PinCapability, PinAssn  │
│                                 │  - Interface, Contract, Bus│
│                                 │  - Subsystem               │
│                                 │  - 7-touchpoint machinery  │
├─────────────────────────────────────────────────────────────┤
│  eak-pcb                         │  eak-engines (verification)│
│  - Manual router                 │  - VerificationEngine      │
│  - Auto-router (agent-driven)    │  - PowerBalanceRule        │
│  - DRC engine                    │  - ClockDomainMembership   │
│  - Gerber export                 │  - ReturnPathRule          │
│  - Footprint editor              │  - PinMuxConflictRule      │
│                                 │  - SignalDriverSinkRule    │
│                                 │  - InterfaceContractRule   │
│                                 │  - BusTopologyRule         │
│                                 │  - SubsystemBoundaryRule   │
├─────────────────────────────────────────────────────────────┤
│  eak-library                     │  eak-runtime (state machine)│
│  - SQLite + FTS5                │  - EngineeringState         │
│  - 3,500 part library           │  - AgentContext (seam)      │
│  - Asset pipeline               │  - CapabilityRequest/Resp   │
│  - Symbol/footprint gen         │  - Deterministic replay     │
├─────────────────────────────────────────────────────────────┤
│  DATA LAYER                     │  eak-compiler (IR stack)    │
│  - PRD markdown files           │  - RequirementIr            │
│  - SQLite + FTS5                │  - EngineeringIr            │
│  - File assets (PDF/STEP)       │  - SchematicIr              │
│  - Design replay log            │  - LogicalElectricalIr      │
│                                 │  - BomIr, PcbIr, MfgIr      │
└─────────────────────────────────────────────────────────────┘
```

### Clean Architecture Principles

| Ring | Responsibility | Crates |
|------|----------------|--------|
| **Entities** | Pure domain models, no deps | `eak-core` |
| **Use Cases** | Business logic, verification rules | `eak-engines` |
| **Interface Adapters** | GUI, CLI, Database, Serialization, Agent adapters | `eak-gui`, `eak-cli`, `eak-library`, `eak-agent` |
| **Frameworks** | GUI framework, DB, OS, LLM providers | `egui`, `rusqlite`, `tokio` |

**Dependency Rule:** Inner rings never depend on outer rings. Dependencies point inward only.

---

## Tech Stack: Modern, Memory-Safe, Yours

### The Stack Decisions

| Layer | Choice | Why |
|-------|--------|-----|
| **Language** | **Rust** | Memory safety, no C++ baggage, WASM for web, fearless concurrency |
| **GUI** | **egui + eframe** | Immediate mode, compiles to WASM, native + web |
| **Database** | **SQLite + FTS5** | Zero-config, 500k parts, full-text search, WAL mode |
| **Serialization** | **RON/JSON** | Human-readable project files, Rust-native |
| **Build** | **Cargo workspace** | 8+ crates, clean boundaries, incremental builds |
| **Python** | **maturin** | Python bindings for data pipelines |
| **3D** | **STEP + glTF** | Industry standard, web-viewable |
| **LLM** | **Provider-agnostic** | Anthropic, OpenAI, Kimi, Ollama — API keys |
| **Markdown** | **`.md` everywhere** | PRDs, reports, ADRs — the engineer's lingua franca |

### Why Not C++/Qt Like Everyone Else?

| Aspect | C++/Qt (KiCad/Altium) | EAK (Rust/egui) |
|--------|----------------------|-----------------|
| **Memory Safety** | Manual, error-prone | **Guaranteed by compiler** |
| **Concurrency** | Manual mutexes, data races | **Fearless concurrency** |
| **Web Deployment** | WebAssembly + Emscripten (painful) | **Native WASM via wasm-bindgen** |
| **Dependency Hell** | CMake, system deps, ABI breaks | **Cargo: semver, lockfiles** |
| **Refactoring** | Terrifying | **Fearless: compiler catches breaks** |
| **AI Integration** | Awkward FFI to Python/C++ SDKs | **Clean async HTTP to LLM APIs** |

### Why Markdown as the PRD Format

| Requirement | Markdown delivers |
|-------------|-------------------|
| Human-readable | **Yes — plain text** |
| Diffable / versionable | **Yes — git-friendly** |
| LLM-friendly | **Yes — token-efficient, structured** |
| Folder-tree navigable | **Yes — just like VS Code** |
| Supports structured data | **Yes — YAML front-matter + tables** |
| Portable | **Yes — every engineer knows it** |

**The PRD folder tree in EAK is indistinguishable from a software project's PRD tree. That's the point.**

---

## The Verification Kernel: The Moat

### What's Built (100% Complete)

| Component | Status | Details |
|-----------|--------|---------|
| **7-Touchpoint Seam** | ✅ 100% | Domain → Event → State → Seam → Verification → Phase → ADR |
| **Domain Models** | ✅ 100% | PowerDomain, ClockDomain, ReturnPath, Signal, PinCapability, PinAssignment, Interface, Contract, Bus, Subsystem |
| **Verification Engine** | ✅ 100% | Generic rule engine, deterministic |
| **ERC Rules (9 Rules)** | ✅ 100% | PowerBalance, ClockDomainMembership, ReturnPath, PinMuxConflict, PinCapability, SignalDriverSink, InterfaceContract, BusTopology, SubsystemBoundary |
| **ADR-Driven Rules** | ✅ 100% | ADR-0022 through ADR-0030, each citing engineering science |
| **Deterministic Replay** | ✅ 100% | Byte-identical state reconstruction |
| **ADR Documents** | ✅ 100% | ADR-0022 through ADR-0030 written |
| **Engineering Science Refs** | ✅ 100% | transmission-lines.md, kirchhoff-laws.md, circuit-theory.md |

### The Rules — Each Grounded in Engineering Science

| Rule | ADR | Science Reference | What It Catches |
|------|-----|-------------------|-----------------|
| `erc-power-balance` | ADR-0022 | kirchhoff-laws.md | Power domain KCL violations |
| `erc-clock-domain-conflict` | ADR-0023 | circuit-theory.md | Clock domain crossings (CDC) |
| `erc-return-path-required` | ADR-0024 | transmission-lines.md L141 | Missing return paths on controlled nets |
| `erc-pin-mux-conflict` | ADR-0025 | §31 | Pin mux conflicts (same pin, different functions) |
| `erc-pin-capability` | ADR-0025 | §31 | Assignment exceeds pin capability |
| `erc-signal-driver-sink` | ADR-0026 | circuit-theory.md L134/L152 | Illegal driver/sink pairings (Thévenin) |
| `erc-interface-contract` | ADR-0027 | Map 14/15/17/18 | Interface contract violations (I²C/SPI/USB) |
| `erc-bus-topology` | ADR-0028 | Map 17 | Bus topology violations (addresses, termination) |
| `erc-subsystem-boundary` | ADR-0029 | Map 14 | Subsystem boundary completeness |

### The Kernel Guards the Agent

The kernel is the **safety net for AI-generated designs**. Every agent action — placing a part, drawing a wire, routing a track — passes through the seam. If the agent proposes something that violates physics, the kernel flags it instantly, with a citation and a fix.

**This is what makes agent-built electronics trustworthy.** No other tool has it.

---

## Database & Library: The Authoritative Data Layer

### The Data Layer: SQLite + FTS5 + Assets

```
data/
├── library.db                    # SQLite (500k rows capable, FTS5)
├── assets/
│   ├── datasheets/               # PDFs
│   ├── symbols/                  # .kicad_sym
│   ├── footprints/               # .kicad_mod
│   └── models3d/                 # .step
```

### Schema (Already Created)

```sql
CREATE TABLE parts (
    lcsc_id       TEXT PRIMARY KEY,
    mpn           TEXT NOT NULL,
    manufacturer  TEXT,
    category      TEXT NOT NULL,
    subcategory   TEXT,
    package       TEXT,
    voltage_v     REAL,
    capacitance_f REAL,
    resistance_ohm REAL,
    inductance_h  REAL,
    tolerance     TEXT,
    temp_coeff    TEXT,
    power_w       REAL,
    stock         INTEGER DEFAULT 0,
    price_cny     REAL,
    moq           INTEGER DEFAULT 1,
    description   TEXT,
    datasheet_url TEXT,
    symbol_url    TEXT,
    footprint_url TEXT,
    model3d_url   TEXT,
    datasheet_path TEXT,
    symbol_path   TEXT,
    footprint_path TEXT,
    model3d_path  TEXT,
    keywords      TEXT,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE VIRTUAL TABLE parts_fts USING fts5(
    lcsc_id, mpn, manufacturer, description, keywords,
    content='parts', content_rowid='rowid'
);
-- + triggers for FTS sync
-- + indexes on category, package, voltage_v, stock, price_cny
```

### The Library Is the Agent's Grounding

The library is what **grounds the AI agent** — no hallucinated pins, no invented parts. When the agent needs a "10 kΩ 0603 resistor," it queries the library and gets a **real part with real specs, real datasheet, real footprint**.

| Asset | Format | Source |
|-------|--------|--------|
| **MPN + Manufacturer** | Text | Manufacturer datasheet |
| **Electrical Specs** | Normalized SI | Manufacturer datasheet |
| **Datasheet** | PDF | Manufacturer website |
| **Symbol** | `.kicad_sym` | Auto-generated from pinout |
| **Footprint** | `.kicad_mod` | IPC-7351 compliant, auto-gen |
| **3D Model** | `.step` | Manufacturer STEP (ICs/connectors) |
| **Keywords** | Text | Auto-generated from attributes |
| **ADR Links** | Text | Linked to relevant ADRs |

### Tools Already Built

| Tool | Status | Command |
|------|--------|---------|
| **Import CLI** | ✅ | `cargo run --bin lib-import --csv file.csv --db data/library.db` |
| **Asset Downloader** | ✅ | `./asset-downloader data/library.db data/assets 8` |
| **Schema** | ✅ | `schema.sql` applied to `data/library.db` |
| **Verification Queries** | ✅ | Defined in verification suite |

---

## Workflow: How Engineers Create Gold-Standard PCBs in EAK

### Today's Workflow (Broken — Pre-AI, Pre-Orchestration)

```
Engineer draws schematic by hand (hours)
    ↓
Engineer manually checks: power, clock, signal, bus
    ↓
Engineer exports netlist
    ↓
PCB layout (manual routing, days)
    ↓
DRC (geometry only)
    ↓
Gerber → Fab
    ↓
🙏 Pray
```

### EAK Workflow (The Vision — PRD → Agent → Orchestrate → Verified)

```
STEP 1 — PLAN (the PRD, the only manual work)
Engineer + AI write PRD.md from client requirements
Engineer sees the PRD folder tree (just like VS Code)
    ↓

STEP 2 — GENERATE (the agent builds)
Engineer: "Build the schematic from PRD.md"
Agent reads PRD → plans → places components → draws wires (LIVE)
Engineer watches the schematic build itself in real time
    ↓

STEP 3 — VERIFY (the kernel referees)
Kernel runs 9 physics rules continuously:
  ✅ Power domains balanced (KCL)
  ✅ Clock domains clean (no CDC)
  ✅ Return paths declared (transmission line theory)
  ✅ Pin muxing valid (capability + assignment)
  ✅ Signal driver/sink legal (Thévenin)
  ✅ Interface contracts satisfied (I²C/SPI/USB)
  ✅ Bus topology valid (addresses, termination)
  ✅ Subsystem boundaries complete
Engineer sees green/amber/red live, with ADR-linked fixes
    ↓

STEP 4 — ORCHESTRATE (the engineer steers)
Engineer points out mistakes: "That's wrong, use 2.2k"
Engineer stops/rewinds the agent at any moment
Agent fixes and re-verifies — live
EAK flags mistakes the engineer might miss, with solutions
    ↓

STEP 5 — ROUTE (the agent lays out the board)
Agent places footprints + routes (LIVE)
Engineer watches the PCB build itself
    ↓

STEP 6 — CERTIFY (zero-respin guarantee)
Full DRC + ERC + DFM
Gerber/ODB++ export
BOM with suppliers
Verification report + replay log
    ↓

✅ Verified board, built by agents, orchestrated by you — in days, not months
```

### The Engineer's Daily Experience

```
Morning: Open EAK → open project → PRD folder tree
    ↓
Read PRD.md, tweak a requirement: "Add a 5V output rail"
    ↓
Tell agent: "Regenerate power section for the new rail"
    ↓
Agent updates schematic LIVE — new LDO, new caps, new net
    ↓
Kernel: ✅ Power balance recomputed — all green
    ↓
Point at a net: "Why is this 2.2k? I2C here is slow mode"
    ↓
Agent: "You're right — slow mode only needs 4.7k. Fixing."
    ↓
Agent routes the board while you review the BOM
    ↓
Export Gerbers → "Ready for fab. 9/9 verification pass."
    ↓
✅ Done in half a day. Zero respins. You orchestrated the whole thing.
```

---

## The Editor: Where Engineers Live

### The Editor Is a VS Code-Style IDE

- **PRD Folder Tree** — the project navigator, exactly like VS Code's explorer
- **Markdown Editor** — write and edit PRDs natively, YAML front-matter for structured intent
- **Schematic Canvas** — live, agent-driven, pan/zoom/selection
- **PCB Canvas** — live, agent-driven, ratsnest, routing
- **Verification Panel** — live ERC/DRC findings with ADR links and fixes
- **Chat/Terminal Panel** — talk to the agent, run `eak build`, `eak verify`
- **Library Panel** — FTS5 search (<50ms) for parts, drag into schematic
- **Live Diagnostics** — red squiggles on nets, just like a linter

### The Editor Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  EAK — my-sensor-hub                                 [Verify] [Build] │
├───────────────┬──────────────────────────────────┬──────────────┤
│  EXPLORER     │   SCHEMATIC CANVAS (LIVE)        │  VERIFICATION│
│  my-sensor-   │                                  │  ✅ Power 62%│
│  hub/         │     [USB-C]──[LDO]──[3V3 rail]   │  ✅ I2C uniq │
│  ├ PRD.md     │              │                   │  ⚠️ SHT41    │
│  ├ prd/       │        [MCU-G0]──[I2C bus]──      │    addr?     │
│  │  power.md  │              │                   │  [Fix] [ADR] │
│  │  clock.md  │        [SHT40] [SHT41] [BMP388]  │              │
│  │  i2c.md    │                                  │  ┌──────────┐│
│  ├ generated/ │  Agent is drawing... [Stop]      │  │ CHAT     ││
│  │  schematic/│                                  │  │ Agent:   ││
│  │  pcb/      │                                  │  │ "SHT41 →  ││
│  └ reports/   │                                  │  │ 0x45 via ││
│               │                                  │  │ ADDR pin"││
├───────────────┴──────────────────────────────────┴──────────────┤
│  STATUS: ● Agent building  ● 0 errors  ● 9/9 rules  ● replay#42 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Business Model: Solo YC-Style Startup

### The Business Thesis

> **EAK is not sold like an EDA tool. It's sold like a developer platform.**
>
> EDA vendors sell seats. EAK sells the future of how hardware gets designed — and monetizes at every layer: software seats, AI compute, the component library, and eventually the model itself. Like GitHub (free for developers, paid for teams), like VS Code (free IDE, paid extensions/AI), like Anthropic (free Claude, paid API and Pro).

### The Funding Reality

| Resource | Amount | Reality |
|----------|--------|---------|
| **KSUM Grant** | ₹3L (~$3,600) | Pending company registration; covers ~2 months runway |
| **Bootstrap** | ₹1-2L | Your own savings, freelance cushion |
| **Runway** | 6 months | Need revenue by Month 3, fundraising by Month 5 |
| **Team** | Solo | You = CEO + CTO + Product + Sales + Support |

### The KSUM Grant (Kerala Startup Mission)

**This is the seed of EAK's business.**

| Detail | Value |
|--------|-------|
| **Amount** | ₹3,00,000 (~$3,600) |
| **Status** | ✅ Sanctioned — **disbursement pending company registration** |
| **Type** | Non-dilutive grant (no equity given up) |
| **Purpose** | Startup support — Kerala Startup Mission backs founders building deep-tech |
| **Key advantage** | Equity-free funding. You keep 100% of the company. |

**How the ₹3L is deployed:**

| Allocation | Amount (₹) | Use |
|------------|-----------|-----|
| **Sustenance** | ~₹60K | Rent/food for ~2 months of full-time building |
| **Company setup** | ~₹15K | Company registration, GST, bank account (required for disbursement) |
| **Tooling & infra** | ~₹15K | VS Code, LLM subscriptions, cloud, hosting |
| **Domain & branding** | ~₹5K | eak.dev, logo, trademark groundwork |
| **Buffer** | ~₹2L | Extended runway while pre-seed conversations happen |

**What KSUM unlocks beyond the money:**

1. **Credibility** — a sanctioned grant is a signal for angel/VC conversations ("KSUM-backed")
2. **Ecosystem access** — KSUM's network of startups, mentors, and investors (Blume, etc.)
3. **Government-backed company** — simplifies banking, invoicing, and future grants
4. **Deep-tech validation** — KSUM specifically funds hardware/AI startups; EAK fits both

**The single action item:** register the company → trigger disbursement → deploy the ₹3L as the bootstrap runway above.

### The Bootstrap Reality

| Resource | Amount | Reality |
|----------|--------|---------|
| **KSUM Grant** | ₹3L | Equity-free; covers initial runway once company is registered |
| **Bootstrap** | ₹1-2L | Your own savings, freelance cushion |
| **Runway** | 6 months | Need revenue by Month 3, fundraising by Month 5 |
| **Team** | Solo | You = CEO + CTO + Product + Sales + Support |

### The Market

| Metric | Value | Basis |
|--------|-------|-------|
| **PCB designers worldwide** | ~500K active | EDA vendor-reported seat counts |
| **Electronics engineers (adjacent)** | ~2M | Many design boards occasionally |
| **Global EDA market** | $2B+ (CAGR ~8%) | Synopsys/Cadence/Siemens EDA revenue |
| **AI-for-EDA market (emerging)** | $1B+ by 2028 (est.) | Analyst projections |
| **Addressable now (SOM)** | ~50K designers | English-speaking, small-firm, indie, startup |
| **India-specific TAM** | ~50K electronics engineers | Growing hardware startup scene |

### Customer Segments

| Segment | Who | Pain | Willingness to Pay | EAK Fit |
|---------|-----|------|--------------------|---------|
| **Indie hardware founders** | Solo/small teams building products | Respin costs kill runway | High (self-funded) | **Best fit** — PRD-first, agent-built, no manual grunt |
| **Hardware startups (India)** | 2-10 engineers | Speed + verification | High (post-funding) | Team workspace + CI/CD |
| **Fablehouses & EMS** | Build-for-hire shops | Fast, correct quotes | High | Batch design, verification reports |
| **Students & makers** | Learning, prototypes | Cheap + guidance | Low | Free tier → funnel |
| **Enterprise (later)** | Aerospace/auto/med | Certification evidence | Very high | Custom rules, audit, DO-254 |

### The Pricing Strategy: Land, Expand, Own

**Land** — free, powerful core. Like VS Code: the tool itself is free and excellent. Engineers bring their own API keys for models.

**Expand** — paid AI usage, teams, CI/CD, advanced verification.

**Own** — enterprise + the EAK Model + the design data flywheel.

### Revenue Streams

| Stream | Model | When | Margin |
|--------|-------|------|--------|
| **Professional subscriptions** | ₹5K/mo per seat (annual ₹50K) | Month 4 | ~90% |
| **Enterprise subscriptions** | ₹50K/mo (negotiated) | Month 12+ | ~90% |
| **Agent compute (credits)** | Pay-per-design or credit packs | Month 4 | 40-70% (pass-through of LLM cost) |
| **Component library** | Free core, premium packs (RF/industrial) | Month 6 | ~100% |
| **Verification reports / certifications** | Per-design fee, BOM-level | Month 8 | ~95% |
| **The EAK Model API** | Token-based, like OpenAI | Year 3 | High once trained |
| **Marketplace (later)** | 30% cut on extensions/footprints/symbols | Year 2 | ~70% |

### Revenue Model (Launch Tiers)

| Tier | Price | Features |
|------|-------|----------|
| **Community** | Free | PRD → schematic → PCB (core), Verification kernel (9 rules), 3,500-part library, Community support. Bring-your-own-API-key. |
| **Professional** | ₹5K/mo | EAK agent credits (bundled), team workspaces, CI/CD + GitHub Action, advanced reports, priority support |
| **Enterprise** | ₹50K/mo | SSO, audit logs, custom rules, on-premise, dedicated support, fine-tuned models, compliance packs |

### Unit Economics (Month 12 target)

| Metric | Value | Notes |
|--------|-------|-------|
| **CAC** | ~₹2K | Content-led, community-led, product-led growth — near-zero paid acquisition |
| **MRR per paying seat** | ₹5K | Professional tier |
| **Gross margin** | ~85% | Infrastructure + LLM pass-through only |
| **Net revenue retention** | 120%+ | Agents consume more credits as designs grow |
| **Payback period** | <1 month | Monthly billing, self-serve onboarding |
| **Churn** | <5%/mo | Design tools are sticky; data lock-in is real but verification reports retain value |

### The Cost Structure (Lean Solo)

| Cost | Monthly (₹) | Notes |
|------|-------------|-------|
| **Infra (SQLite, CI, hosting)** | ~₹2K | Minimal — SQLite, GitHub Actions, static landing |
| **LLM API (shared with users)** | Pass-through | Credits model covers it |
| **Tooling (VS Code, cloud)** | ~₹1K | Your own opencode/Claude subscriptions |
| **Company compliance** | ~₹2K | GST filing, bank, registrations |
| **Total fixed burn** | ~₹5-8K/mo | Grant covers 3+ years of this burn |

### Go-To-Market: Product-Led, Content-Fueled

**You have a weapon competitors don't: you ARE the customer.** You're a solo electronics engineer building for engineers. Go-to-market is community-first:

| Channel | Tactic | Timeline |
|---------|--------|----------|
| **Launch on Hacker News** | "I built VS Code for electronics — it designs your PCB from a markdown PRD" | Month 4 |
| **YouTube demo** | 10-min: PRD → working board, live agent, zero respins | Month 4 |
| **GitHub open-source core** | Stars = credibility + contributors | Month 4 |
| **India hardware community** | KSUM ecosystem, IoT Blr/Hyd/Mumbai meetups, college workshops | Month 4-6 |
| **Reddit / Discord / r/PrintedCircuitBoard** | Honest posts, comparisons, free pilot licenses | Month 4-6 |
| **KiCad plugin + GitHub Action** | Drop into existing workflows — viral surface | Month 3 |
| **Design contests** | "Fastest verified board" — user-generated proof | Month 6 |
| **Referral** | Free credits for every design exported | Month 6 |

### The Growth Flywheel

```
PRD-first design is FAST and VERIFIED
        ↓
Engineers ship boards in days, share on GitHub/YouTube
        ↓
More engineers see "EAK-built" boards → try it
        ↓
More designs flow through the kernel → more verified design data
        ↓
Design data (opt-in) trains the EAK Model
        ↓
The EAK Model makes PRD→board even faster and more correct
        ↓
Faster + more correct → more engineers → (loop)
```

**The flywheel is the moat.** Every design run makes EAK better. Competitors can't copy a data flywheel; they'd have to start from zero verified designs.

### The 6-Month YC-Style Plan

| Month | Milestone | Target |
|-------|-----------|--------|
| **Month 1** | Core Editor + Kernel + PRD-to-Schematic Pipeline | PRD → verified schematic working |
| **Month 2** | Agent Layer + Live Verification | Agent builds; kernel referees live |
| **Month 3** | Launch + KiCad plugin + GitHub Action | 100 downloads, 10 active weekly |
| **Month 4** | Professional Tier + Pilot Users | 50 WAU, ₹50K/mo revenue |
| **Month 5** | Fundraise Prep + India GTM | 50 WAU, 3 paid pilots, metrics deck |
| **Month 6** | **Pre-seed Close** | ₹50L–1Cr @ ₹10–20Cr valuation |

### The YC Pitch

> **Problem:** Electronics engineers still design boards the way software engineers coded in 1995 — manual, error-prone, pray-and-respin. Software got Claude Code; electronics got nothing.
>
> **Solution:** EAK — VS Code for electronics. Write a PRD in markdown, EAK's AI agent builds the schematic and PCB, the engineer orchestrates, and EAK's verification kernel proves it against physics. Zero-respin boards in days, not months.
>
> **Moat:** 7-touchpoint verification kernel + ADR-driven rules + deterministic replay + agent-native library + the design-data flywheel. No competitor has any of this. Long-term: **a custom EAK model** trained on verified electronics designs.
>
> **Traction (Month 4):** 50 WAU, 10 designs/week, 3 paid pilots, GitHub Action + VS Code ext.
>
> **Market:** 500K PCB designers globally. $2B TAM (EDA). Plus AI-for-hardware TAM.
>
> **Ask:** ₹50L for 5% → 18 months runway, hire 2 engineers (router, model trainer).

### Fundraising Strategy

| Stage | Amount | Timing | Use |
|-------|--------|--------|-----|
| **KSUM Grant** | ₹3L | Now (post-registration) | Bootstrap to launch |
| **Pre-seed** | ₹50L–1Cr | Month 6 | 18 months runway, hire 2 engineers |
| **Seed** | ₹3-5Cr | Year 2 | Router engineer, model training, growth |
| **Series A** | $2-5M | Year 3-4 | The EAK Model at scale, enterprise sales |

**Who funds it:** Indian deep-tech/angel networks (Blume, Speciale, Kstart, Titan), hardware-focused angels, and if traction is global — YC/Wayfinder for the hardware stack. Hardware + AI is a hot category; the "VS Code for electronics" narrative is instantly understandable.

### Financial Projection (Conservative)

| Quarter | Paying Seats | MRR | Burn | Cash Position |
|---------|--------------|-----|------|---------------|
| **Q1 (Month 1-3)** | 0 | ₹0 | ₹8K/mo | Grant covers |
| **Q2 (Month 4-6)** | 25 | ₹1.25L/mo | ₹10K/mo | Break-even on burn |
| **Q3 (Month 7-9)** | 100 | ₹5L/mo | ₹30K/mo | Reinvest + pre-seed runway |
| **Q4 (Month 10-12)** | 300 | ₹15L/mo | ₹60K/mo | ARR ~₹1.8Cr (~$210K) |

At ₹1.8Cr ARR with a data flywheel and a defensible kernel — a healthy pre-seed→seed story.

### Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| **"Can't beat KiCad (free)"** | High | EAK isn't competing on drawing — it's the PRD→agent→verify workflow KiCad can't do. Plug-in integration converts KiCad users. |
| **LLM quality on schematics** | Medium | Kernel referees every agent output; deterministic replay; human-orchestrated loop degrades gracefully. |
| **Library curation cost** | Medium | Parametric generation + verify-with-physics; LCSC-scale is a differentiator, not a burden. |
| **Fundraise failure** | High | Bootstrap path: subscription revenue covers burn by Month 4-6; no dependency on external capital. |
| **Big EDA adds AI** | Medium | They're enterprise-moated and slow; EAK moves at startup speed and owns the PRD-first workflow. |

### The Real Endgame

The endgame isn't just selling an EDA tool. It's:

1. **Every electronics engineer designs like a software engineer** — PRD-first, agent-built, orchestrated
2. **The EAK model** becomes the reference "electronics brain" — like Claude for code, EAK for hardware
3. **The design data flywheel** — every verified design trains the model, making it better, making it the standard
4. **The Linux kernel of EDA** — open kernel, commercial frontends, plugin ecosystem, certified for DO-254/ISO 26262
5. **A platform business** — marketplace, model API, certification revenue — not a seat-selling EDA vendor

---

## Development Roadmap: From Tool to Platform

### Year 1: PRD-to-Board Pipeline (Months 1-6)
- ✅ Verification kernel (done — 9 rules, deterministic)
- ✅ ADR-driven rules + science references (done)
- ✅ Database schema + import tools (done)
- 🔄 3,500-part library build (in progress — 7-day sprint)
- 🔄 Agent layer: PRD → schematic (API-key models)
- 🔄 Schematic + PCB canvas (egui)
- 🔄 Agent → kernel integration (live verification of agent output)
- 🔄 Launch + pilot users + first revenue

### Year 2: Professional Editor + Agent Maturity
- Agent-driven auto-routing (push/shove, diff pairs, length tuning)
- Copper pour + planes
- 3D viewer + STEP export
- Fine-tuned open-weight models on EAK design data
- BOM export + supplier links

### Year 3: Physics Simulation + The EAK Model
- SPICE integration (ngspice)
- Signal Integrity (impedance, reflection, crosstalk)
- Power Integrity (PDN impedance, decay caps)
- Thermal + EMC simulation
- **The EAK Model v1** — foundation model trained on verified electronics

### Year 4: AI-Native Design
- The EAK Model at full strength — PRD → 90%-correct first-pass board
- Constraint-driven layout (solver-backed)
- Generative design space exploration
- Counterfactual analysis ("what if I change this cap?")

### Year 5: The Linux Kernel of EDA
- Open kernel (Apache 2.0)
- Commercial frontends (Altium-style, web, VS Code)
- Certified for DO-254/ISO 26262/IEC 61508
- Ecosystem of plugins, solvers, simulators, models
- **Every electronics engineer designs like a software engineer**

---

## What Makes EAK "Gold" for Electronics Engineers

### The Gold Standard Checklist

| Gold Standard | EAK Delivers | How |
|---------------|--------------|-----|
| **Zero-respin designs** | ✅ | Physics-based ERC catches errors before fab |
| **Formal traceability** | ✅ | Requirement → ADR → Rule → Finding → Fix |
| **Deterministic reproducibility** | ✅ | Byte-identical replay |
| **Engineering rationale** | ✅ | Every finding links to ADR + science |
| **Physics-based verification** | ✅ | KCL, Thévenin, Transmission line, CDC |
| **PRD-first design** | ✅ | Plan before you build — like software |
| **AI agents that build** | ✅ | Claude Code for electronics |
| **Live development** | ✅ | Watch the board build itself |
| **Engineer orchestration** | ✅ | You steer, the agent builds |
| **AI recommendations** | ✅ | EAK flags mistakes + suggests fixes |
| **Custom model future** | ✅ | The EAK Model — trained on physics |

### What "Gold" Means for an Engineer

| Engineer Pain | EAK Gold Solution |
|---------------|-------------------|
| "I spent 3 days drawing a schematic" | **PRD → agent builds it while you watch** |
| "Did I connect all power pins?" | **Auto-check: every power pin has a net, every net has a driver** |
| "Is this trace impedance-controlled?" | **Declare `impedance_target: 50Ω` → auto-flag missing return path** |
| "Does this I²C bus have address conflicts?" | **Declare `interface: I2C` → auto-check unique addresses** |
| "Does this clock cross domains?" | **Declare clock domains → auto-flag CDC violations** |
| "Did I forget a decoupling cap?" | **Power domain budget → auto-flag missing caps** |
| "Is this pin muxed correctly?" | **Declare pin capability → auto-flag mux conflicts** |
| "I've been debugging for 2 weeks" | **Replay log + ADR links → find it in minutes** |

**Your product = "The checklist that runs itself, plus an AI agent that does the work."**

---

## Current Status & Next Steps

### What's Done (✅ 100%)

| Component | Status |
|-----------|--------|
| **7-Touchpoint Verification Seam** | ✅ 100% |
| **Domain Models (10 types)** | ✅ 100% |
| **Verification Engine** | ✅ 100% |
| **9 Band B ERC Rules** | ✅ 100% |
| **ADR Documents (0022-0030)** | ✅ 100% |
| **Engineering Science Refs** | ✅ 100% |
| **Deterministic Replay** | ✅ 100% |
| **Database Schema** | ✅ 100% |
| **Import CLI + Asset Downloader** | ✅ 100% |

### In Progress (🔄)

| Component | Status | Target |
|-----------|--------|--------|
| **3,500-Part Library (7-day sprint)** | 🔄 In Progress | Week 1 |
| **PRD Editor + Folder Tree** | ⏳ Queued | Week 1-2 |
| **Schematic + PCB Canvas** | ⏳ Planned | Week 2 |
| **Agent Layer (API-key models)** | ⏳ Planned | Week 2-3 |
| **Agent → Kernel integration** | ⏳ Planned | Week 3 |
| **Launch + Pilot Users** | ⏳ Planned | Week 4 |

### Immediate Next Steps (This Week)

```
THIS WEEK (Day 1-7): 3,500-Part Library Foundation
├── Day 1: 500 Passives (R, C, L)
├── Day 2: 500 Discrete Semiconductors
├── Day 3: 500 Logic/MCU ICs
├── Day 4: 500 Comm/Sensor ICs
├── Day 5: 500 Connectors
├── Day 6: 500 Specialized (RF, Audio, Motor)
└── Day 7: Assets + FTS5 + UI Polish + Export

NEXT WEEK: PRD Editor + Folder Tree + Schematic Canvas (egui)
WEEK 3: Agent Layer (API keys) + Agent→Kernel integration
WEEK 4: LAUNCH
MONTH 6: PRE-SEED CLOSE
```

---

## The Complete Repository Structure

```
eak/
├── Cargo.toml                          # Workspace root
├── README.md
├── LICENSE
├── .gitignore
├── schema.sql                          # SQLite schema
├── story/                              # ← You are here
│   └── EAK_STORY.md
├── docs/
│   ├── adrs/                           # ADR-0022 through ADR-0030
│   ├── engineering-science/
│   │   ├── transmission-lines.md
│   │   ├── kirchhoff-laws.md
│   │   └── circuit-theory.md
│   └── architecture/
│       ├── 7-touchpoint-seam.md
│       └── adr-driven-rules.md
├── crates/
│   ├── eak-core/                       # Domain models, DomainError, EntityId
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── entity.rs
│   │   │   ├── error.rs
│   │   │   ├── power_domain.rs
│   │   │   ├── clock_domain.rs
│   │   │   ├── return_path.rs
│   │   │   ├── signal.rs
│   │   │   ├── pin_capability.rs
│   │   │   ├── pin_assignment.rs
│   │   │   ├── interface.rs
│   │   │   ├── contract.rs
│   │   │   ├── bus.rs
│   │   │   └── subsystem.rs
│   │   └── Cargo.toml
│   │
│   ├── eak-ports/                      # Events, serialization
│   │   ├── src/
│   │   │   ├── lib.rs                  # Event enum, serialization
│   │   │   └── store.rs                # Event store
│   │   └── Cargo.toml
│   │
│   ├── eak-runtime/                    # State machine, seam, protocol
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── state.rs                # EngineeringState
│   │   │   ├── protocol.rs             # CapabilityRequest/Response
│   │   │   ├── runtime_core.rs         # RuntimeCore (AgentContext impl)
│   │   │   ├── orchestrator.rs         # Phase orchestration
│   │   │   └── clock.rs
│   │   └── Cargo.toml
│   │
│   ├── eak-engines/                    # Verification engine + rules
│   │   ├── src/
│   │   │   ├── lib.rs                  # VerificationEngine, Rule trait
│   │   │   ├── rules/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── power_balance.rs
│   │   │   │   ├── clock_domain_membership.rs
│   │   │   │   ├── return_path.rs
│   │   │   │   ├── pin_mux_conflict.rs
│   │   │   │   ├── pin_capability.rs
│   │   │   │   ├── signal_driver_sink.rs
│   │   │   │   ├── interface_contract.rs
│   │   │   │   ├── bus_topology.rs
│   │   │   │   └── subsystem_boundary.rs
│   │   │   └── context.rs              # VerificationContext
│   │   └── Cargo.toml
│   │
│   ├── eak-phases/                     # Phase machines
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schematic_planning.rs
│   │   │   ├── pcb_placement.rs
│   │   │   ├── pcb_routing.rs
│   │   │   ├── erc_verification.rs
│   │   │   ├── drc_verification.rs
│   │   │   ├── constraint_verification.rs
│   │   │   ├── bom_verification.rs
│   │   │   ├── dfm_verification.rs
│   │   │   └── emc_analysis.rs
│   │   └── Cargo.toml
│   │
│   ├── eak-agent/                      # ← The Claude Code of electronics
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── model.rs                # Provider-agnostic model interface
│   │   │   ├── anthropic.rs            # Claude Opus/Sonnet adapter
│   │   │   ├── openai.rs               # GPT-4o adapter
│   │   │   ├── kimi.rs                 # Kimi adapter
│   │   │   ├── ollama.rs               # Local model adapter
│   │   │   ├── agent_loop.rs           # plan→act→verify→fix loop
│   │   │   ├── tools.rs                # place/wire/route/verify/library tools
│   │   │   └── prd_reader.rs           # PRD markdown → structured intent
│   │   └── Cargo.toml
│   │
│   ├── eak-library/                    # Component library
│   │   ├── src/
│   │   │   ├── lib.rs                  # LibraryDb, Part, Asset management
│   │   │   ├── db.rs                   # SQLite operations
│   │   │   ├── assets.rs               # Asset download/management
│   │   │   ├── symbols.rs              # Symbol generation
│   │   │   ├── footprints.rs           # Footprint generation
│   │   │   └── search.rs               # FTS5 search
│   │   └── Cargo.toml
│   │
│   ├── eak-cli/                        # CLI tools
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/
│   │   │   │   ├── lib_import.rs
│   │   │   │   ├── asset_downloader.rs
│   │   │   │   ├── verify.rs
│   │   │   │   └── build.rs
│   │   │   └── bin/
│   │   │       ├── lib-import.rs
│   │   │       ├── asset-downloader.rs
│   │   │       └── generate_dataset.rs
│   │   └── Cargo.toml
│   │
│   ├── eak-gui/                        # egui IDE (VS Code-style)
│   │   ├── src/
│   │   │   ├── main.rs                 # eframe app entry
│   │   │   ├── app.rs                  # Main app state
│   │   │   ├── panels/
│   │   │   │   ├── explorer_panel.rs   # PRD folder tree
│   │   │   │   ├── prd_editor.rs       # Markdown PRD editor
│   │   │   │   ├── library_panel.rs
│   │   │   │   ├── schematic_canvas.rs
│   │   │   │   ├── pcb_canvas.rs
│   │   │   │   ├── properties_panel.rs
│   │   │   │   ├── diagnostics_panel.rs
│   │   │   │   ├── verification_panel.rs
│   │   │   │   └── chat_panel.rs       # Talk to the agent
│   │   │   ├── schematic/
│   │   │   │   ├── editor.rs
│   │   │   │   ├── tools.rs
│   │   │   │   └── netlister.rs
│   │   │   ├── pcb/
│   │   │   │   ├── editor.rs
│   │   │   │   ├── router.rs
│   │   │   │   └── drc.rs
│   │   │   └── verification/
│   │   │       ├── report.rs
│   │   │       └── inline_diagnostics.rs
│   │   └── Cargo.toml
│   │
│   ├── eak-schematic/                  # Schematic model
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── netlist.rs
│   │   │   ├── component.rs
│   │   │   └── wire.rs
│   │   └── Cargo.toml
│   │
│   ├── eak-pcb/                        # PCB model
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── board.rs
│   │   │   ├── layer_stack.rs
│   │   │   ├── track.rs
│   │   │   ├── via.rs
│   │   │   ├── zone.rs
│   │   │   └── gerber.rs
│   │   └── Cargo.toml
│   │
│   ├── eak-compiler/                   # IR compiler stack
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── requirement_ir.rs
│   │   │   ├── engineering_ir.rs
│   │   │   ├── schematic_ir.rs
│   │   │   ├── logical_electrical_ir.rs
│   │   │   ├── bom_ir.rs
│   │   │   ├── pcb_ir.rs
│   │   │   └── manufacturing_ir.rs
│   │   └── Cargo.toml
│   │
│   ├── eak-store/                      # Event store
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── sqlite_store.rs
│   │   └── Cargo.toml
│   │
│   └── eak-units/                      # Physical quantities
│       ├── src/
│       │   ├── lib.rs
│       │   ├── quantity.rs
│       │   ├── dimension.rs
│       │   └── unit.rs
│       └── Cargo.toml
│
├── scripts/
│   ├── day1/
│   │   ├── generate_passives.py
│   │   ├── generate_semiconductors.py
│   │   ├── generate_ics.py
│   │   ├── generate_comm_sensors.py
│   │   ├── generate_connectors.py
│   │   └── generate_specialized.py
│   ├── lib-import/
│   │   └── main.rs
│   └── asset-downloader/
│       └── main.rs
│
├── data/
│   ├── library.db
│   ├── library.db-wal
│   └── assets/
│       ├── datasheets/
│       ├── symbols/
│       ├── footprints/
│       └── models3d/
│
├── examples/
│   ├── blinky/
│   ├── i2c-sensor/
│   └── usb-pd/
│
├── reports/
│   ├── day1-recon.md
│   ├── day1-assets.md
│   ├── day1-validation.md
│   └── day1-final.md
│
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
```

---

## The One-Liner

> **EAK = VS Code for electronics. Write a PRD in markdown, an AI agent builds the board, the engineer orchestrates, and the kernel proves it obeys physics.**

---

## Epilogue: Why This Wins

| Dimension | Competitors (KiCad/Altium/Flux) | EAK |
|-----------|--------------------------------|-----|
| **Workflow** | Manual drawing | **PRD-first, agent-built** |
| **AI** | None | **Claude Code for electronics, custom model future** |
| **Verification** | Geometry-only DRC | **Physics-based ERC (KCL, transmission lines, Thévenin)** |
| **Traceability** | None | **Requirement → ADR → Rule → Finding → Fix** |
| **Replay** | None | **Byte-identical deterministic replay** |
| **Rules** | Hardcoded, black-box | **ADR-driven, science-cited, engineer-readable** |
| **Kernel** | Monolithic C++ | **Rust, WASM-ready, memory-safe** |
| **Architecture** | Monolith | **7-touchpoint seam, clean rings** |
| **Library** | CSV/DB dumps | **SQLite + FTS5 + Assets + ADR-linked** |
| **Engineer's role** | Manual everything | **Orchestrator — like a software PM** |
| **Extensibility** | Plugin APIs | **7-touchpoint seam = infinite extensibility** |

**Your moat:** The *only* EDA where design is **PRD-first, agent-built, and physics-verified** — and where the long-term vision is a **custom electronics model** that becomes the reference "brain" for hardware design.

**Software got Claude Code. Electronics gets EAK.**

---

## The Final Word

**EAK is not "another EDA tool."**

It's a **new category**: *Agent-orchestrated, intent-verified EDA*.

Every other tool asks: *"Can you draw the geometry?"*

EAK asks: *"What do you want to build — and let an agent build it, and prove it works."*

That difference — **manual drawing vs. PRD-orchestrated agent building** — is the difference between "it took three months and two respins" and "it took three days and it worked first time."

**That's the gold standard.**

---

*EAK — Intent In. Manufactured Board Out.*

*VS Code for electronics. Claude Code for hardware.*

*Write the PRD. Watch the agent build. Orchestrate the result. Let physics verify.*

*Built by an electronics engineer, for electronics engineers.*

*Written in Rust. Powered by agents. Verified by physics. Shipped with proof.*