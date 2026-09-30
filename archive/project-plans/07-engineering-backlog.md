# Electronics Agent Kit — Engineering Backlog (07)

> Executable backlog for the 3-month fundable-MVP effort. Anchored to
> `00-overview.md` (source of truth): a **local Tauri AI-native EDA IDE** whose native
> core is the **existing Rust kernel** (`eak/crates/*`), frontend is a web-UI shell,
> canvas is **reused** (KiCanvas / KiCad formats), AI is wired through the kernel's
> reasoning boundary, and the frontend↔backend contract is the **kernel event stream**.
> Hero flow: intent → generate → AI-review, traceable, on curated examples.
>
> **Tag legend:** `[BUILD]` net-new · `[REUSE]` embed/integrate existing · `[INT]`
> integrate our kernel with new shell. Effort: **S** ≤1 day · **M** 1–3 days · **L** 3–5 days.

---

## Kernel ground truth (what already exists — do not rebuild)

Verified by reading `eak/crates/`. Reference these real names in every kernel-touching task.

- **`eak-runtime`** — the kernel. `RuntimeCore` is the *sole mutator* (single `commit()`
  path: stamp→append→fold), implements `AgentContext`, holds `EngineeringState`.
  `Orchestrator::run(&mut WorkflowPlan, &mut ctx)` sequences phases with `LoopBack` edges;
  `replay()` rebuilds state from the log; FSM = `Machine`/`ExecutionEngine`/`PhaseOutcome`.
- **`eak-ports`** — the contracts. **`Event` enum = the event stream** (`EventRecord{seq,
  timestamp,event}`); `EventLog` trait; `ReasoningEngine` trait + `ReasoningRequest`/
  `ReasoningResponse`/`CandidateRequirement` = **the reasoning boundary**.
- **`eak-reasoning`** — `FixtureEngine` (deterministic cassette) + `AnthropicEngine`
  (`from_env`, feature `live`, Anthropic Messages API tool-mode) behind the boundary.
- **`eak-phases`** — the 15 state machines (`RequirementPlanningMachine` …
  `ManufacturingGenerationMachine`); only `RequirementPlanning` reasons today (`agent.rs`
  builds the one `ReasoningRequest`, schema `requirement_candidates_v1`).
- **`eak-engines`** — `VerificationEngine` (a `Rule` registry) + the DRC/ERC/DFM/BOM/EMC/
  ampacity/impedance/thermal rules; `ConstraintEngine`; `PartCatalog`.
- **`eak-compiler`** — IR projections `RequirementIr`→`EngineeringIr`→`SchematicIr`→
  `BomIr`→`PcbIr`→`ManufacturingIr` (the last carries `Board`+`Placement`+`Track`+refdes→MPN).
- **`eak-store`** — `FileEventLog` (append-only JSONL, one `EventRecord`/line).
- **`eak-cli`** — composition root: `run_cli`, `run`/`run_with(RunConfig)`, `default_workflow()`,
  `replay_cmd`, `trace_cmd`, binary `eak` (`Run`/`Replay`/`Trace`). `build_reasoning()` picks
  fixture vs `live`.
- **`eak-domain` / `eak-units`** — typed entities; `Board`/`Placement`/`Track` carry x/y/width
  as `PhysicalQuantity` (mm) + `BoardSide` → enough geometry to emit `.kicad_pcb`.

**Two gaps that shape the backlog:** (1) there is **no Tauri app, frontend, or `package.json`**
anywhere — the shell is greenfield; (2) `RuntimeCore::commit` folds synchronously and
`Orchestrator::run` only returns *after* the whole run — there is **no mid-run event
observer**, so streaming to a UI is a real BUILD task (E2), not a wire-up.

---

## Epics overview

| # | Epic | BUILD/REUSE | Blocks |
|---|------|-------------|--------|
| E1 | Tauri shell + kernel-as-backend spine | BUILD/INT | everything |
| E2 | Event contract + streaming to UI | BUILD | E3, E4, E6 |
| E3 | IDE frontend shell / panels | BUILD | hero flow |
| E4 | Reused canvas / renderer integration | REUSE | AI-review visual |
| E5 | KiCad import (bulletproof fallback) | BUILD/REUSE | review flow |
| E6 | AI harness / agent loop + LLM boundary | BUILD/INT | generate flow |
| E7 | Hero-flow curation + demo | INT | the raise |
| E8 | Packaging / telemetry / waitlist | BUILD | distribution |

---

## E1 — Tauri shell + kernel-as-backend spine

**Goal:** a native window that boots, embeds the web UI, and calls the real Rust kernel
in-process. **Why:** this is the seam that makes "our kernel *is* the app's native core"
real; it unblocks all other work.

- [ ] `[BUILD] S` Add a `tauri` app crate (`eak/crates/eak-app` or `src-tauri/`) to the
      workspace; depend on `eak-cli`, `eak-runtime`, `eak-phases`, `eak-reasoning`,
      `eak-store`. Keep the ring rule (adapter, points inward).
- [ ] `[BUILD] S` Scaffold the frontend toolchain (Vite + TS) that Tauri serves; blank window boots.
- [ ] `[INT] M` Expose a `run_pipeline(intent, seed)` **Tauri command** that calls
      `eak_cli::run(&RunConfig{..})` on a background thread and returns the final `RunReport`
      (outcomes + counts). REUSE `default_workflow()` and `run_with`.
- [ ] `[INT] S` Expose `replay(log_path)` and `trace(log_path, req)` Tauri commands over
      `eak_cli::replay_cmd` / `trace_cmd`.
- [ ] `[BUILD] S` App state: hold an open project (event-log path, last `EngineeringState`)
      in Tauri managed state so commands share one project.
- [ ] `[BUILD] S` Decide the log location per project (app data dir) and thread it through
      `RunConfig.log`; stop clobbering a shared default path.

**DoD:** double-clicking the built app opens a native window that runs the full 15-phase
pipeline on a typed intent via the real kernel and shows the phase outcomes.

---

## E2 — Event contract + streaming to UI

**Goal:** stream `eak_ports::Event`s to the frontend *as they commit*, live. **Why:** this
is the frontend↔backend contract from the overview; the "traceability graph filling in as
it goes" demo beat is impossible without it. This is the highest-risk kernel change.

- [ ] `[BUILD] M` Add an **event observer hook** to `RuntimeCore`: an optional
      `Box<dyn Fn(&EventRecord)>` / `std::sync::mpsc::Sender<EventRecord>` invoked inside
      `commit()` *after* append+fold. Determinism unaffected (observation only, never re-read).
- [ ] `[BUILD] S` Thread the observer through `RuntimeCore::new` and add a
      `run_with_observer` variant in `eak-cli` so the app can subscribe.
- [ ] `[INT] M` In the Tauri command, forward each `EventRecord` to the webview via
      `tauri::ipc::Channel` (or `Window::emit`) as JSON — `Event` already derives `Serialize`.
- [ ] `[BUILD] S` Define a stable **frontend event schema doc** (TS types mirroring the `Event`
      enum variants used by the UI: `IntentCaptured`, `RequirementCommitted`, `ReasoningCall`,
      `ViolationRaised`, `PlacementCommitted`, `TrackCommitted`, `*IrProduced`, `PhaseCompleted`).
- [ ] `[BUILD] S` Frontend event bus/store (Zustand/signals) that folds the stream into a
      live view-model — the UI's mirror of `EngineeringState`.
- [ ] `[BUILD] S` Backpressure/ordering guard: events arrive in `seq` order; UI tolerates a
      fast run without dropping frames (buffer + rAF flush).

**DoD:** one real pipeline run streams its ordered `EventRecord`s into the window and the
UI updates panels live, in `seq` order, with no post-hoc reload.

---

## E3 — IDE frontend shell / panels

**Goal:** the Cursor-for-hardware chrome: intent/chat, live engineering-state, DRC/reasoning,
traceability. **Why:** it's our surface; it must feel like an IDE, not a form.

- [ ] `[BUILD] M` App layout (VSCode-like): left agent/chat pane, center canvas, right
      inspector, bottom problems/DRC panel, collapsible.
- [ ] `[BUILD] M` **Intent/agent chat panel**: text intent in, streamed reasoning + phase
      progress out (consumes E2 stream).
- [ ] `[BUILD] M` **Engineering-state panel**: live counts + lists of requirements, blocks,
      components, nets, parts, placements, tracks (folded from the stream).
- [ ] `[BUILD] M` **Problems/DRC panel**: renders `ViolationRaised` events with severity,
      subjects, and (from E6) the AI explanation + suggested fix.
- [ ] `[BUILD] M` **Traceability view**: render provenance (`ProvenanceLink` / `trace_cmd`
      output) as a graph; clicking a violation walks back to the originating requirement and
      the English intent sentence.
- [ ] `[BUILD] S` Phase timeline/stepper mirroring `default_workflow()`'s 15 phases with
      per-phase status + loop-back indicators.
- [ ] `[BUILD] S` Design-taste pass (use the frontend design skill): dark IDE theme, typography,
      motion — must not look templated for the demo.

**DoD:** all panels populate live from one streamed run; a violation is clickable and traces
back to its intent sentence on screen.

---

## E4 — Reused canvas / renderer integration

**Goal:** render the generated/imported board on a **reused** canvas, never a hand-built one.
**Why:** the overview mandates reuse; building a PCB renderer would burn the whole budget.

- [ ] `[REUSE] M` Embed **KiCanvas** (web component) in the center pane; render a static
      `.kicad_pcb` to prove the pipeline.
- [ ] `[BUILD] L` **`ManufacturingIr` → `.kicad_pcb` exporter** (new module in `eak-compiler`
      or a sibling `eak-export` crate): map `Board`(width/height/`LayerStack`),
      `Placement`(x/y/side/courtyard), `Track`(net/layer/width/x1..y2), `PartAssignment`
      (refdes/MPN) → KiCad S-expr. All geometry is already typed `PhysicalQuantity` (mm).
- [ ] `[INT] S` Tauri command `export_pcb(log_path) -> kicad_pcb string` projecting current
      state via `PcbIr::project` + `ManufacturingIr::project`, feeding KiCanvas.
- [ ] `[BUILD] S` Live canvas refresh on `PlacementCommitted` / `TrackCommitted` stream events
      (re-export + reload, MVP-grade).
- [ ] `[BUILD] S` Canvas highlight hook: selecting a violation/net in the DRC panel highlights
      the implicated placement/track on the canvas.

**DoD:** a generated board and an imported board both render on the reused canvas; DRC
findings visually highlight on it.

---

## E5 — KiCad import (bulletproof fallback)

**Goal:** parse a real `.kicad_pcb`/`.kicad_sch` into kernel state so **import → AI-review
always works**, independent of generation. **Why:** the overview's demo insurance policy.

- [ ] `[BUILD] L` `.kicad_pcb` parser → domain entities: board outline→`Board`,
      footprints→`Component`+`Placement`, tracks→`Track`, nets→`Net`. New `eak-import` crate
      (outer ring, depends on `eak-domain`/`eak-units`).
- [ ] `[BUILD] M` **Import path into the kernel**: feed parsed entities through the real
      capability seam (`CapabilityRequest::CreateBoard`/`PlaceComponent`/`RouteNet`/`CreateNet`)
      so imported designs get the same validation + event log as generated ones — no back door.
- [ ] `[BUILD] S` A "verify-only" workflow (subset of `default_workflow()`: DRC/DFM/EMC/BOM
      machines, skip the synthesis phases) that runs the existing `VerificationEngine` rules
      over imported state.
- [ ] `[INT] S` Tauri command `import_kicad(path)` + a frontend "Open board" affordance.
- [ ] `[BUILD] S` Curate 2–3 known-good real KiCad boards as import fixtures for the fallback demo.

**DoD:** opening a real KiCad board renders it and runs the full DRC/DFM/EMC rule set,
producing traceable violations — with generation entirely bypassed.

---

## E6 — AI harness / agent loop + LLM boundary

**Goal:** a real, streamed, multi-step agent grounded by the kernel — the *soul*. **Why:**
this is the moat: AI you can trust to drive because the kernel verifies every action.

- [ ] `[INT] S` Wire the **live boundary end-to-end**: build the app with `--features live`,
      route `ReasoningChoice::Live` → `AnthropicEngine::from_env(model)`; surface `ANTHROPIC_API_KEY`
      config in-app. (Read `claude-api` skill before touching model ids/params.)
- [ ] `[BUILD] M` **Stream reasoning to the UI**: `ReasoningCall` events already commit through
      `RuntimeCore`; render request/response (candidates, rationale, confidence) live in the chat
      pane via the E2 stream.
- [ ] `[BUILD] L` **Extend reasoning beyond Requirement Planning**: give ≥1 more phase a real
      `AgentContext::reason` call behind a new `schema_name` (e.g. architecture/block selection or
      part choice), mirroring `agent.rs`'s `build_prompt` + candidate-mapping pattern. Kernel
      re-validates at the seam — the LLM only *proposes*.
- [ ] `[BUILD] M` **AI-review explainer**: for each `Violation`, call the boundary to produce a
      plain-English explanation + suggested fix, linked to the violation's subjects and its
      originating requirement (feeds the E3 DRC panel). New reasoning schema + fixture responses.
- [ ] `[BUILD] M` **Cassette capture/replay tooling**: record live `AnthropicEngine` responses
      into a `Cassette` (via `FixtureEngine::key`) so any curated demo replays offline and
      byte-identically — the demo never depends on a live API round-trip.
- [ ] `[BUILD] S` Budget/guardrails surfacing: show `Budget.max_reasoning_calls`, retries, and
      loop-back events so the "verified, bounded agent" story is visible.
- [ ] `[BUILD] S` Graceful degradation: if the live call errors (`ReasoningError`), fall back to
      the matching cassette entry so the demo can't hard-fail on network.

**DoD:** an intent typed in the chat drives a live (or cassette-backed) multi-phase run whose
reasoning streams to the UI, every proposal is kernel-validated, and each DRC finding gets an
AI explanation traceable to the original sentence.

---

## E7 — Hero-flow curation + demo

**Goal:** the one flawless intent→generate→review→explain run from the overview, hardened.
**Why:** this run *is* the raise.

- [ ] `[INT] M` Pick + freeze the hero intent (overview's "USB-C powered I²C temperature
      sensor, < 1 W"); author the full cassette so `run` produces a released `ManufacturingIr`
      with a clean board every time.
- [ ] `[INT] M` Author a **curated/assisted placement + route** for the hero board (NOT a general
      autorouter) so the canvas result is reliable — hand-tuned `Placement`/`Track` fixtures the
      Placement/Routing machines emit deterministically.
- [ ] `[BUILD] M` Seed one **intentional, curated DRC finding** that the AI explains and traces
      back — the "whoa" beat — then a clean re-run.
- [ ] `[BUILD] S` "Demo mode": scripted, one-click launch of the hero flow with pacing, plus the
      E5 import→review fallback bound to a hotkey.
- [ ] `[BUILD] S` End-to-end smoke test (extend `eak-cli/tests/integration.rs` style) asserting
      the hero run releases and replays byte-identically — CI guard against demo rot.
- [ ] `[BUILD] S` Record a backup screen capture of the full flow (the "it always works" reel).

**DoD:** the hero flow runs flawlessly from a cold start in the packaged app, and the
import→review fallback is one hotkey away.

---

## E8 — Packaging / telemetry / waitlist

**Goal:** a distributable build + the fundraise signal instruments. **Why:** investors need to
run it (or see it run) and see early demand.

- [ ] `[BUILD] M` `tauri build` producing signed-ish installers for macOS (primary demo) +
      Linux; verify cold-boot on a clean machine.
- [ ] `[BUILD] S` Bundle curated fixtures/cassettes so the offline demo works with no API key.
- [ ] `[BUILD] S` Minimal, privacy-respecting local telemetry (run count, phase timings) — off by
      default, for the founder's own iteration.
- [ ] `[BUILD] S` In-app **waitlist / design-partner** capture (email → a simple endpoint).
- [ ] `[BUILD] S` First-run onboarding: the hero intent pre-filled, one "Run" button.
- [ ] `[BUILD] S` Crash/error surface so a failed live call or bad import degrades visibly, not silently.

**DoD:** a founder-signed build runs the hero demo offline on a fresh machine, and the app can
capture a waitlist email.

---

## Dependencies (what blocks what)

- **E1 (spine) blocks everything** — no window/command surface, nothing to hang UI on.
- **E2 (streaming) blocks E3, E4-live-refresh, E6-streamed-reasoning** — the contract that
  lets frontend + backend proceed in parallel once it exists.
- **E4 export (`ManufacturingIr`→kicad) blocks canvas rendering of *generated* boards;**
  **E5 import blocks the AI-review fallback** (review needs state in the kernel).
- **E6 depends on E2** (reasoning is streamed) and feeds **E3's DRC panel** (explanations).
- **E7 depends on E4 + E5 + E6** (needs canvas, fallback, and explainer) — it's integration.
- **E8 depends on E7** (you package the hardened flow).
- Kernel-internal enabler on the critical path: **E2's `RuntimeCore` observer hook** — start it
  early; it is the one change that touches the sole-mutator `commit()` path.

---

## Start here Monday (the first week — build the streaming spine)

The goal for week one: **one real 15-phase pipeline run streams live into a native window.**

1. `[S]` Add the Tauri app crate to the `eak` workspace, depending on `eak-cli` +
   `eak-runtime`; get a blank native window to boot. **(E1)**
2. `[S]` Scaffold the Vite/TS frontend Tauri serves; render "hello kernel". **(E1)**
3. `[M]` Add the **event observer hook** to `RuntimeCore::commit` (mpsc `Sender<EventRecord>`),
   threaded through `RuntimeCore::new`. This is the load-bearing kernel change. **(E2)**
4. `[S]` Add `run_with_observer` to `eak-cli` alongside `run_with`, reusing `default_workflow()`.
   **(E2)**
5. `[M]` Expose the `run_pipeline(intent, seed)` Tauri command that runs on a background thread
   and forwards each `EventRecord` to the webview via `tauri::ipc::Channel`. **(E1/E2)**
6. `[S]` Frontend: subscribe to the channel, fold events into a store, print the ordered
   `EventRecord` log + live phase outcomes in a scrolling panel. **(E2/E3)**
7. `[S]` Verify byte-identity still holds: run the existing `eak-cli/tests/integration.rs`
   + a `replay` of the streamed run — the observer must not perturb determinism. **(E2)**
8. `[S]` Wire the first real panel: live requirement/violation counts from the stream, proving
   the contract end-to-end. **(E3)**

Deliverable: type `"USB-C powered I²C temperature sensor, < 1 W"` in the window, hit Run, and
watch the real kernel's events stream in live — the spine the whole product hangs on.
