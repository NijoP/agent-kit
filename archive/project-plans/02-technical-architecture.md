# Technical Architecture — Tauri Shell over the EAK Rust Kernel

> Anchored to `00-overview.md` (the source of truth). Where this doc adds detail it never
> contradicts the overview: **Shell = Tauri**, **backend = the existing `eak/` Rust kernel**,
> **frontend = a web UI**, **canvas = reused**, **contract = the kernel's event stream**,
> **LLM = Claude via the reasoning boundary (`live` feature)**. Written 2026-07-02. Every
> kernel claim below cites a real crate/type read from `eak/crates/`.

---

## 0. Key decisions (TL;DR)

1. **The kernel becomes the app's native core, not a sidecar.** Tauri's backend is Rust, so
   the existing `eak-runtime::RuntimeCore` (the "sole mutator of Engineering State") is compiled
   *into* the desktop binary. No IPC to a separate server, no re-implementation.
2. **The seam is the already-serializable `eak_ports::Event` enum.** It is `#[derive(Serialize,
   Deserialize)]` today and is written as JSON-lines by `eak_store::FileEventLog`. The UI is a
   *projection of that same event stream* — exactly as `eak-runtime`'s `EngineeringState` is.
   Freezing this contract is what lets frontend and backend proceed in parallel.
3. **Reuse KiCanvas for the canvas**; our IP goes into the harness (`eak-phases` agents on top of
   `eak-runtime`) and the kernel, never the renderer. A `PcbIr → .kicad_pcb` exporter and a
   `.kicad_pcb →` capability-seam importer let the reused renderer and the trusted kernel meet at
   KiCad's formats.

---

## 1. System diagram

```
┌───────────────────────────────────────────────────────────────────────────────┐
│  TAURI DESKTOP APP  (single native binary, local-first, offline-capable)        │
│                                                                                 │
│  ┌──────────────────────────── WebView (system WebKit/WebView2) ─────────────┐  │
│  │  WEB FRONTEND  (React + TS + Vite)          "the IDE shell / chrome"       │  │
│  │                                                                           │  │
│  │   Intent/Agent chat │ Live engineering-state │ DRC/ERC/DFM findings       │  │
│  │   Traceability graph │ Reasoning stream       │ ┌───────────────────────┐ │  │
│  │                                                │  EMBEDDED RENDERER      │ │  │
│  │   FE event-fold store  (mirrors EngineeringState in TS)                  │ │  │
│  │        ▲  subscribe (Tauri `event`)      ▼  actions (Tauri `command`)   │ │  │
│  └────────┼─────────────────────────────────┼──────────────────────────────┘  │
│           │  EventRecord{seq,timestamp,event} JSON        │ RunConfig / intent  │
│  ┌────────┼─────────────────────────────────┼──────────────────────────────┐  │
│  │  eak-app  (NEW outer-ring crate = composition root + Tauri glue)         │  │
│  │    • #[tauri::command] fns  • EventSink → window.emit()  • KiCad I/O      │  │
│  └────────┼───────────────────────────────────────────────────────────────┘  │
│           │                                                                     │
│  ┌────────▼──────── RUST KERNEL  (the existing eak/ workspace) ─────────────┐  │
│  │                                                                           │  │
│  │  eak-cli / eak-app ─ Orchestrator.run(WorkflowPlan, &mut RuntimeCore)     │  │
│  │  eak-phases   ── 15 phase Machines + RequirementAgent (the harness)       │  │
│  │  eak-runtime  ── RuntimeCore (commit path) · AgentContext · replay        │  │
│  │  eak-engines  ── VerificationEngine + 17 Rules (DRC/ERC/DFM/EMC/thermal)  │  │
│  │  eak-compiler ── RequirementIr → … → ManufacturingIr projections          │  │
│  │  eak-domain / eak-ports / eak-units ── entities · contracts · quantities  │  │
│  │                                                                           │  │
│  │        ▲ ReasoningEngine port                 ▲ EventLog port             │  │
│  │  ┌─────┴───────────────┐               ┌──────┴──────────────┐            │  │
│  │  │ eak-reasoning       │               │ eak-store           │            │  │
│  │  │  FixtureEngine      │               │  FileEventLog       │            │  │
│  │  │  AnthropicEngine ───┼──► Claude API │  (append-only JSONL)│            │  │
│  │  │  (feature `live`)   │   (only edge  └─────────────────────┘            │  │
│  │  └─────────────────────┘    that leaves the machine)                      │  │
│  └───────────────────────────────────────────────────────────────────────────┘
```

| Component | What it is | Concrete crate/type |
|---|---|---|
| Tauri shell | Native window + WebView + Rust host | new `eak-app` crate (sibling of `eak-cli`) |
| Rust kernel core | Deterministic engineering runtime | `eak-runtime::RuntimeCore`, `EngineeringState` |
| Event bus / stream | Ordered, serialized fact log = the seam | `eak_ports::Event`, `EventRecord`, `EventLog` |
| LLM reasoning adapter | The single stochastic boundary | `eak_ports::ReasoningEngine`; `eak_reasoning::{FixtureEngine, AnthropicEngine}` |
| Web frontend | IDE chrome + panels | React/TS in the WebView |
| Embedded renderer | Reused WebGL KiCad viewer | KiCanvas web component |
| KiCad import | Real board → domain/IR | `eak-app` parser → `CapabilityRequest`s |

The rings are enforced at compile time: the kernel test `dependency_rule::kernel_has_no_outward_dependencies`
fails the build if `eak-runtime` ever gains a dependency on an adapter (`eak-store`, `eak-reasoning`,
`eak-phases`, `eak-engines`, `eak-compiler`, `eak-cli`). `eak-app` sits in the **same outermost ring
as `eak-cli`** — it is allowed to depend inward on everything; nothing depends on it.

---

## 2. Why Tauri, not Electron

The decisive reason is not bundle size — it is that **our backend is already Rust**. With Tauri the
kernel *is* the native host process; with Electron we would run a Node main process and then have to
reach the Rust kernel over a subprocess/FFI boundary, re-introducing exactly the serialization seam
Tauri gives us for free.

| Axis | Tauri (chosen) | Electron | Why it matters here |
|---|---|---|---|
| Backend language | **Rust** — kernel compiles in-process | Node/JS main; Rust becomes a sidecar | `RuntimeCore` is the host, no cross-process RPC |
| Bundle size | ~3–10 MB (system WebView) | ~120–200 MB (bundled Chromium) | pre-seed demo, easy download |
| Memory | Low (shared OS WebView) | High (per-app Chromium) | runs on a laptop during a live demo |
| Local-first / offline | Native; only the `live` LLM call leaves the box | Same, but heavier | matches "local, native, AI-native" thesis |
| Security | Rust core; explicit `#[tauri::command]` allowlist; API key stays in Rust | Wide Node surface in renderer reach | API key never enters JS; kernel gates all writes |
| LLM key handling | Held in `AnthropicEngine` (Rust), OS keychain | Tempting to put in JS env | keeps the secret off the web tier |

**Honest tradeoffs.** Tauri renders on the *system* WebView (WebKit on macOS/Linux, WebView2 on
Windows), so we test on three engines instead of one bundled Chromium (mitigated: our UI is panels +
one embedded WebGL canvas, not exotic CSS), and the Tauri/JS ecosystem is smaller than Electron's (we
lean on standard Vite/React inside the WebView). Net: Tauri's "Rust host" property is worth far more
to this product than Electron's uniform-Chromium convenience.

---

## 3. The event contract — the seam (the linchpin)

The whole parallel-build plan rests on one fact: **the kernel already speaks a versioned, serialized
event language, and state is nothing but its fold.** `eak-runtime`'s single commit path is
`RuntimeCore::commit`: *stamp (clock) → append (`EventLog`) → fold (`EngineeringState::apply`)*.
Both the live run and `eak_runtime::replay` call the *same* `apply`, guaranteeing byte-identical
reconstruction. The UI becomes a **third folder of the same stream**.

### 3.1 What crosses the seam

The unit is `eak_ports::EventRecord { seq: u64, timestamp: Timestamp, event: Event }`. The `Event`
enum (already `Serialize`/`Deserialize`) is the contract. Its variants map cleanly onto the four
things the UI must render:

| UI panel | Event variants it consumes (real names) |
|---|---|
| **Engineering-state updates** | `IntentCaptured`, `RequirementCommitted`, `FunctionalBlockCommitted`, `ComponentCommitted`, `PinCommitted`, `NetCommitted`, `PartCommitted`, `BomLineItemCommitted`, `BoardCommitted`, `PlacementCommitted`, `TrackCommitted` |
| **Check results (DRC/ERC/DFM/EMC)** | `ConstraintCommitted`, `ViolationRaised`, `WaiverGranted`, `VerificationCompleted { rule_count, open_violations }` |
| **Reasoning stream** | `ReasoningCall { request, response }` (carries `ReasoningRequest`/`ReasoningResponse`) |
| **Traceability edges** | `ProvenanceLinked { link: ProvenanceLink { from, to, relation } }` |
| **Pipeline / phase status** | `PhaseEntered`, `PhaseStateChanged`, `PhaseCompleted`, `PhaseFailed`, and IR milestones `RequirementIrProduced … ManufacturingGenerated` |

Every record carries a monotonic `seq`, so the UI store is idempotent and self-ordering: it can
drop duplicates, resume, and time-travel by folding a prefix. The traceability graph the demo needs
is *already in the log* — nodes are the entity-bearing deltas, edges are `ProvenanceLinked` records
whose `relation` is a `RelationType` (`DerivedFrom`, `JustifiedBy`, `Supports`, `TracesTo`, …).

### 3.2 How it is surfaced to the WebView

Two Tauri channels, one direction each:

- **Backend → UI (stream):** Tauri **events** (`window.emit("eak://event", record)`). `eak-app` wires
  a small `EventSink` observer onto the commit path (§9.1) so each committed `EventRecord` is pushed
  to the WebView as it is appended — ordered and low-latency.
- **UI → backend (actions):** Tauri **commands** (`#[tauri::command]`) for the verbs the user drives:

```rust
#[tauri::command] async fn capture_intent(text: String) -> Result<Seq, String>;     // → IntentCaptured
#[tauri::command] async fn run_pipeline(cfg: RunConfigDto) -> Result<(), String>;   // Orchestrator.run(...)
#[tauri::command] async fn import_kicad(path: String) -> Result<(), String>;        // §6
#[tauri::command] async fn trace(requirement: String) -> Result<TraceDto, String>;  // wraps eak_cli::trace_cmd
#[tauri::command] async fn replay(log: String) -> Result<StateDto, String>;         // wraps eak_runtime::replay
```

These are thin wrappers over functions that already exist in `eak-cli` (`run_with`, `replay_cmd`,
`trace_cmd`) — the CLI *is* the reference composition root; `eak-app` is the same wiring behind Tauri.

We choose Tauri commands+events over an embedded WS/SSE server for the MVP: no port to bind, no CORS,
same `Event` JSON payloads, fully offline. (An embedded local WS is the drop-in escalation for
multi-window or an external inspector — the envelope doesn't change.)

### 3.3 Contract stability

Because `Event` is a Rust enum with `serde`, we generate the TypeScript mirror from it (e.g. `ts-rs`
or `schemars` → JSON Schema → TS) so the FE `Event` type **cannot drift** from the kernel's. The
IR-boundary events already carry a `schema_version` (`REQUIREMENT_IR_SCHEMA_VERSION` … 
`MANUFACTURING_IR_SCHEMA_VERSION` in `eak-compiler`), giving us an explicit versioning story for the
richer payloads the UI reads at phase boundaries.

---

## 4. The AI harness (the agent loop, on top of the kernel)

The overview's moat is "AI you can trust to drive." The kernel already implements the mechanism that
makes that literally true: **the LLM only ever produces judgement; the kernel commits.** This is the
two-part agent split (documented in `eak-phases/src/agent.rs`, the `RequirementAgent`):

```
        ┌─────────────── RequirementAgent::activate(ctx, activation) ───────────────┐
        │                                                                            │
  LLM   │  ctx.reason(ReasoningRequest)  ──►  ReasoningEngine  ──►  Claude / fixture │  stochastic half
        │        returns (Seq, ReasoningResponse{ candidates })                      │  (judgement only)
        │              │  ← recorded as a ReasoningCall event (replayable, P4)        │
        │              ▼                                                              │
 KERNEL │  for each candidate:  validate() → attach Decision + Evidence + links      │  deterministic half
        │        ctx.invoke(CapabilityRequest::CreateRequirement{..})                │  (the seam, P3)
        │              │                                                             │
        │              ▼  RuntimeCore re-validates at the seam, then commit()        │
        │        stamp → append(EventLog) → fold(EngineeringState) → EventRecords    │
        └────────────────────────────────────┬───────────────────────────────────────┘
                                              ▼   emitted to the UI (§3.2)
```

**Why this makes the harness trustworthy — three kernel-enforced guarantees:**

1. **Can't commit invalid state.** The *only* write path an agent has is
   `AgentContext::invoke(CapabilityRequest)`. `CapabilityRequest` has 12 variants
   (`CreateRequirement`, `CreateFunctionalBlock`, `RealizeComponent`, `CreateNet`, `CreatePart`,
   `CreateBomLineItem`, `CreateBoard`, `PlaceComponent`, `RouteNet`, …). Each is re-validated by
   `RuntimeCore` *at the seam* before anything is written — e.g. `handle_create_functional_block`
   rejects a block that realizes no existing requirement; `handle_route_net` rejects a track on a
   phantom net or before a `Board` exists; `handle_create_bom_line_item` rejects double-sourcing.
   A rejected proposal returns `CapabilityError::Rejected` and **nothing enters the log**. The LLM
   physically cannot push an untraceable or malformed design past the kernel.
2. **Replay / determinism.** `ctx.reason` records every model call as a `ReasoningCall` event; ids
   come from `SeededIdSource`, clocks from `LogicalClock`. `eak_runtime::replay` re-folds the log
   *without calling the model or reading the clock*, reproducing byte-identical state (asserted by
   `canonical_json()` equality in the kernel tests). Any AI-generated board is re-derivable and
   auditable offline — "traceable + reproducible" is a test, not a slogan.
3. **Traceability by construction.** Every commit carries its `ProvenanceLink`s and a `Decision`
   with `reasoning_call_seq` pointing back at the exact model call. `eak_cli::trace_cmd` already
   walks requirement → decision → evidence → intent. The UI's traceability graph is this data
   rendered live.

**The loop.** `Orchestrator::run(&mut WorkflowPlan, ctx)` sequences the 15 phase `Machine`s of
`default_workflow()` (Requirement Planning → Engineering Analysis → Constraint Extraction/Verify →
Schematic Planning → ERC → BOM Planning/Verify → PCB Floor Planning → Component Placement → Routing
Planning → DRC → DFM → EMC → Manufacturing Generation). Six `LoopBack` edges are the **self-correction
seam**: a failed DRC routes back to Routing Planning, a failed DFM to Component Placement, etc., each
capped at `max_retries: 2`, with a global step cap so the loop always terminates (no silent infinite
loop). Manufacturing Generation is the terminal **global gate**: it releases the `ManufacturingIr`
iff no open blocking `Violation` remains. Only Requirement Planning currently reasons; every other
phase is deterministic, which is why a whole run replays bit-identically.

**Live vs fixture.** `ReasoningEngine` is the single boundary. `FixtureEngine` replays a recorded
`Cassette` (deterministic, offline, no key) — powering reproducible demos and tests. `AnthropicEngine`
(behind Cargo feature `live`) calls the Claude Messages API in **tool/JSON mode** constrained to the
requirement schema, reporting `model_id()` = `"anthropic:claude-opus-4-8"`. The composition root picks
one (`ReasoningChoice::{Fixture, Live}`); the hero demo runs `live`, the fallback and CI run `fixture`.

---

## 5. Reuse strategy for the canvas/editor (buy vs build)

The overview is emphatic: **never build the canvas.** We evaluate three reuse paths against the MVP
need, which is *render + review* (not interactive schematic/layout editing — that's out of scope
year-1).

| Option | What you get | Fit for MVP | Cost / risk |
|---|---|---|---|
| **KiCanvas** (WebGL KiCad renderer, web component, MIT) | Drop-in `<kicanvas-embed>` that renders `.kicad_pcb` / `.kicad_sch` in the WebView | **Strong** — it's a web component, lives in our React tree, renders exactly the formats we already plan to import/export | Read-only (no editing); young project; we own the `PcbIr → .kicad_pcb` exporter |
| Embed the **KiCad engine** (C++ / libkicad) | Full render + edit fidelity | Weak — C++, not a web target; huge to embed under Tauri's WebView | High integration cost, GPL, defeats "small chrome" |
| **LibrePCB / Horizon EDA** | Full open-source EDA editors | Weak — they are whole applications to fork, not embeddable renderers; heavy GPL surface | We'd inherit an editor we don't want to maintain |

**Recommendation: KiCanvas for the MVP.** Rationale: (1) it is a *web component*, so it composes
into the React IDE shell with zero process boundary; (2) it renders the KiCad file formats, which we
already treat as the interchange for import (§6) and for the assisted-route output — so the reused
renderer and the trusted kernel meet at KiCad's `.kicad_pcb`; (3) read-only is exactly right for a
*review* flow — the AI drives, the kernel commits, the canvas *shows*. The only build we own is a
small, deterministic **`PcbIr`/`ManufacturingIr` → `.kicad_pcb` exporter** in `eak-app` (board
outline + `Placement`s + `Track`s + `LayerStack` → KiCad s-expressions). If KiCanvas proves limiting
for a specific overlay (e.g. highlighting a violating `Track`), we layer a thin HTML/SVG overlay
keyed by `EntityId` on top — still not "building a canvas."

---

## 6. KiCad import (the bulletproof review fallback)

Goal (overview §4): *import a real `.kicad_pcb` / netlist → AI review always works.* The importer
lives in `eak-app` and lands imported geometry through the **same capability seam** as generated
designs, so imported boards get the same validation, IR projection, and replayable event log.

**Mapping (KiCad → `eak-domain`):**

| KiCad artifact | Domain entity | Capability |
|---|---|---|
| board outline + stackup | `Board { width, height, stack: LayerStack }` | `CreateBoard` |
| footprint | `Component` (+ `Pin`s from pads) | `RealizeComponent` |
| pad | `Pin { designation, electrical_type }` | (part of `RealizeComponent`) |
| net / netlist | `Net { members: [pin ids], class }` | `CreateNet` |
| footprint position/side | `Placement { x, y, width, height, side }` | `PlaceComponent` |
| track segment | `Track { net, layer, width, x1..y2 }` | `RouteNet` |

**The provenance-spine subtlety (honest).** The seam enforces upstream traceability: `RealizeComponent`
requires an existing `FunctionalBlock`, and a block requires an existing `Requirement` rooted in a
`DesignIntent`. An imported board has none of these. So the importer **synthesizes a minimal spine**:
one `IntentCaptured` (`"Imported from <file>"`), one reverse-engineered `Requirement`
(status `Accepted`, acceptance = "reproduces the imported board"), and one `FunctionalBlock` that
every imported `Component` is minted from. This keeps referential integrity intact so the same seam
accepts the import, and it keeps the whole board traceable and replayable. The importer then runs the
**verification-only sub-workflow** (ERC → DRC → DFM → EMC over `eak-engines`' 17 rules) rather than the
generative phases — the review runs, findings stream to the UI as `ViolationRaised` events, and the
demo never depends on generation succeeding.

---

## 7. Tech stack

| Layer | Choice | Rationale |
|---|---|---|
| Desktop shell | **Tauri 2** | Rust host = kernel is the native core (§2) |
| Kernel (unchanged) | the `eak/` workspace: `eak-units, -domain, -ports, -runtime, -engines, -compiler, -phases, -store, -reasoning, -cli` | already built, 186 tests, clippy/fmt clean |
| New backend crate | **`eak-app`** (outer ring, sibling of `eak-cli`) | Tauri commands/events, `EventSink`, KiCad I/O, composition root |
| Frontend framework | **React + TypeScript + Vite** | largest ecosystem, easy hiring, Vite is Tauri's default dev server; (SolidJS/Svelte are lighter fallbacks) |
| FE state | small **event-fold store** (Zustand/reducer) mirroring `EngineeringState` in TS | the FE is a projection of the `Event` stream — same discipline as the kernel |
| Contract types | **`ts-rs`/`schemars`** generating TS from the Rust `Event`/IR types | contract cannot drift from the kernel |
| Styling / chrome | **Tailwind + Radix/shadcn** | fast, unopinionated IDE chrome; our differentiation is the flow, not bespoke widgets |
| Canvas | **KiCanvas** web component (§5) | reuse, never build |
| LLM | **Claude** via `eak-reasoning::AnthropicEngine` (feature `live`), `claude-opus-4-8` | single reasoning boundary; key stays in Rust |
| Packaging | Tauri bundler → `.dmg` / `.AppImage` / `.msi`; API key in OS keychain | one-file local install for the demo |
| Persistence | `eak-store::FileEventLog` (append-only JSONL) per project | already the source of truth; enables replay/trace |

**Stays in Rust (not the web tier):** the entire kernel, all validation, all verification rules, IR
projection, replay, the LLM call and API key, and KiCad import/export. The WebView holds *only*
presentation and an event-fold projection — it has no authority to mutate design state, which is the
architectural expression of the moat.

---

## 8. Parallel-build plan (frontend + backend behind the fixed contract)

The contract that unblocks parallelism is **the `Event` JSON stream + the command signatures (§3)**.
Freeze those first; then both tracks run independently.

```
              ┌──────────── FROZEN CONTRACT ────────────┐
              │  eak_ports::Event (serde JSON)           │
              │  EventRecord{seq,timestamp,event}        │
              │  #[tauri::command] signatures (§3.2)     │
              └───────────────┬──────────────────────────┘
   BACKEND track              │              FRONTEND track
   ─────────────              │              ──────────────
   • add EventSink hook       │    • generate TS types from Event (ts-rs)
     on commit path (§9.1)    │    • build panels against a RECORDED
   • wrap run_with/replay/    │      event log ("UI cassette") replayed
     trace_cmd as commands    │      from disk at 1×/step-through
   • PcbIr→.kicad_pcb export  │    • KiCanvas integration on sample
   • KiCad importer (§6)      │      .kicad_pcb files
   • (opt) streaming reason   │    • traceability graph from ProvenanceLinked
              │               │               │
              └──────► integration: swap the recorded stream for the live
                       Tauri `event` channel — no UI code changes, because the
                       envelope is identical.
```

**The "UI cassette" trick.** Run `eak run --reasoning fixture --deterministic` to produce a canonical
`eak-events.jsonl` for the hero-demo intent. That file *is* the frontend's mock backend: FE devs fold
it exactly as the live app will, and can scrub through it step-by-step. This mirrors the kernel's own
reasoning `Cassette` pattern, one layer up. Because the recorded and the live stream are both
`EventRecord` JSON, the day we connect the real Tauri channel the UI needs zero changes — the seam did
its job.

---

## 9. Required kernel additions (small, honest list)

The kernel is contract-ready but was built headless. Three additive changes enable the UI; none
touch the ring rules or the commit invariants.

1. **An `EventSink`/observer on the commit path.** Today `RuntimeCore::commit` appends to `EventLog`
   and folds, but nothing can *subscribe*. Add an inward-facing port (like `EventLog`) that `eak-app`
   implements to forward each `EventRecord` to `window.emit` — an outer-ring adapter, so it respects
   `kernel_has_no_outward_dependencies`. (Zero-core-change week-1 stopgap: tail the `FileEventLog`.)
2. **A streaming reasoning variant (optional, for the "reasoning tokens" panel).** `ReasoningEngine`
   is synchronous today (`request_judgement` returns a full `ReasoningResponse`; the crate doc notes
   "stream/cancel deferred"). Token-by-token display needs an added streaming call (or a callback) on
   the boundary. The demo can ship without it (stream the fixture at a scripted cadence); wire real
   streaming when polishing the `live` path.
3. **The KiCad import/export module in `eak-app`** (§6) and the `PcbIr → .kicad_pcb` exporter (§5) —
   new code in the outermost ring only.

Everything else the UI needs already exists: serialized events with `seq` ordering, replay,
traceability links, the 17-rule verification engine, and the 15-phase orchestrated pipeline.
