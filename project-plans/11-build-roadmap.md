# Electronics Agent Kit — Build-From-Scratch Roadmap (construction order)

> The **ground-up construction sequence**: if you built the Engineering Operating System deliberately
> from the first commit, in what order would you build it, and why. Anchored to `00-product-vision.md`
> (what), `01-engineering-philosophy.md` (why), `02-engineering-world-model.md` (the Maps to own),
> `03-roadmap.md` (the stage/Map-arrival + fundraising plan).
>
> **This document answers a different question than `03`.** `03` sequences *which Map comes online
> and when* against a timeline and a raise. **This one sequences *how you construct the machine that
> owns Maps*, foundation-first, by technical dependency** — the substrate `03` treats as "Stage 0,
> already built." It doubles as a **position marker**: each phase is tagged with where the current
> `eak/` codebase actually stands. Written 2026-07-10. No code; a construction plan.

---

## 0. Build principles (the invariants of the construction *order*)

The order is not arbitrary. Eight principles fix it, and violating any of them makes the thing you
build *not* an Engineering OS:

1. **Substrate before content.** Build the machine that can own *any* engineering truth correctly
   before you teach it *which* truth. The runtime is the product; Maps pour into it (`01 §Prologue`).
2. **Determinism from the first commit.** The clock, the id source, and the append-only log exist
   before anything they stamp. You cannot retrofit replay onto a nondeterministic core.
3. **Own before surface.** Build ownership (objects + seam + provenance + verification) before any
   interface. An interface is a projection of owned truth; there is nothing to project until the
   truth is owned (`03 §0`).
4. **Rings from day one.** Dependencies point only inward. Enforce it with a guard test on the first
   day, not after the fifth crate tangles.
5. **The reasoning seam exists before real reasoning.** Put the propose-then-dispose boundary in
   place with a *stub* reasoner first; wire a real model into the same port later. The boundary is
   the architecture; the model is a plug-in (`01 §Part III`).
6. **Build the object meta-model once.** The twelve facets every object carries (`02 §Part V`) are
   built as a pattern before the first object, so every Map inherits identity, provenance, lifecycle,
   verification, and history for free.
7. **Vertical slice before horizontal breadth.** Drive *one* Map (Intent) through the *entire*
   machine — capture → commit → fold → verify → project → replay → trace — before adding a second
   Map. Prove the whole spine on one object type; then breadth is repetition, not risk.
8. **Every layer is replayable and tested before the next is built.** The exit criterion of each
   phase is "it replays byte-identically and the green-gate is green." No layer is built on an
   unverified layer.

---

## 1. The construction stack (what rests on what)

```
        ┌──────────────────────────────────────────────────────────────┐
   L8   │  ENGINEERING OS — drive end-to-end · deep Memory · cross-domain│
        ├──────────────────────────────────────────────────────────────┤
   L7   │  Band D — lifecycle Maps + eak-memory (the compounding moat)   │
   L6   │  Band C — behavior/world-model Maps + eak-solvers              │
   L5   │  Band B — logical-electrical Maps + Logical-Electrical IR      │
        ├──────────────────────────────────────────────────────────────┤
   L4   │  SURFACING — event contract · shell · panels · rendered view   │  (interface = projection)
        ├──────────────────────────────────────────────────────────────┤
   L3   │  Band A — epistemic Maps (Assumption·Risk·Tradeoff·Fidelity)   │
   L2   │  SKELETON Maps + IR projections + Verification + Orchestrator   │
   L1   │  Object meta-model + first vertical slice (Intent) + provenance │
        ├──────────────────────────────────────────────────────────────┤
   L0   │  DETERMINISTIC SUBSTRATE — units · id · clock · event log ·     │  ← everything rests here
        │  fold · capability seam · reasoning port · replay · ring guard │
        └──────────────────────────────────────────────────────────────┘
```

Build strictly bottom-up. L0→L2 is "construct the runtime + skeleton"; L3 makes reasoning honest; L4
makes it visible; L5→L8 is "own the rest of engineering." **You never build a layer before the layer
beneath it replays and passes the gate.**

---

## 2. Phase-by-phase build order

Each phase states: **Build** · **Why here (dependency)** · **Introduces** · **Exit criterion** ·
**Invariant it upholds** · **Current status in `eak/`.**

### Phase 0 — The deterministic substrate *(the kernel that owns nothing yet, but can own anything correctly)*
- **Build:** the physical-quantity type system (dimensions, units, tolerance); the identity primitive
  (opaque, never-null id); the logical clock and seeded id source; the append-only event log; the
  state fold; the single commit path (stamp → append → fold → observe); the capability seam (the sole
  write path); the reasoning-engine port (with a stub reasoner); replay; the ring/dependency guard test.
- **Why here:** nothing can be owned correctly until there is a deterministic, replayable, validated
  way to change state. This is the whole architecture in miniature, owning zero engineering objects.
- **Introduces:** `EntityId`, `PhysicalQuantity`/`Unit`/`Dimension`, `Event`/`EventRecord`, `EventLog`,
  `EngineeringState`, `RuntimeCore::commit`, `CapabilityRequest`/seam, `ReasoningEngine` port, `replay`.
- **Exit criterion:** a trivial event commits, folds, and **replays byte-identically**; the ring guard
  test fails the build on an outward dependency; the seam rejects a malformed proposal (nothing enters
  the log).
- **Invariant:** Principles 2, 4, 5 (`01 §VIII` architectural invariants).
- **Status in `eak/`:** ✅ **built** (`eak-units`, `eak-domain::EntityId`, `eak-ports`, `eak-store`,
  `eak-runtime`; `kernel_has_no_outward_dependencies`; `replay`).

### Phase 1 — The object meta-model + first vertical slice + provenance spine
- **Build:** the twelve-facet object contract as a reusable pattern; the *first* engineering object —
  the **Intent Map** (`DesignIntent`) — driven through the entire machine; the provenance/traceability
  spine (`ProvenanceLink` + relation taxonomy) and a `trace` walk.
- **Why here:** prove the substrate on one real object end-to-end before multiplying objects (Principle
  7). Provenance is cross-cutting — every later Map needs it, so build it now, not later.
- **Introduces:** `DesignIntent`, `ProvenanceLink`/`RelationType`, the trace command, the object-facet
  pattern (identity·relationships·lifecycle·…·history) every future object reuses.
- **Exit criterion:** capture an intent → commit → fold → trace back to it → replay identically. One
  Map, whole spine.
- **Invariant:** Principles 3, 5, 6, 10 (traceability by construction; objecthood).
- **Status in `eak/`:** ✅ **built** (Intent, provenance, `trace_cmd`).

### Phase 2 — The reasoning loop + the skeleton Maps + IR + verification + orchestrator
- **Build, in dependency order:** the two-part agent (propose→validate→commit) on the reasoning seam;
  then the skeleton Maps — **Requirement → Constraint → Architecture/Block → Component/Pin → Net →
  Part/BOM → Board/Stackup → Placement → Routing**; the IR projection machinery (canonical
  phase-boundary projections); the verification engine + its first rules; the orchestrator (phase FSM +
  bounded loop-backs + termination guarantee); the terminal manufacturing gate.
- **Why here:** with the substrate proven and provenance in place, this is *repetition of the Phase-1
  pattern* across the design skeleton — each Map is a new object type through the same seam. The
  orchestrator and gate come last because they *sequence and close* what the Maps produce.
- **Introduces:** `RequirementAgent`, `Constraint`/`Violation`/`Waiver`, `FunctionalBlock`,
  `Component`/`Pin`/`Net`, `Part`/`BomLineItem`/catalog, `Board`/`LayerStack`/`Placement`/`Track`, the
  six IRs, the rule engine + ~17 rules, the orchestrator + loop-backs, the release gate.
- **Exit criterion:** one intent flows the full pipeline to a released `ManufacturingIr` iff no open
  blocking violation, and the whole run **replays byte-identically**; a hallucinated part is rejected
  at the seam.
- **Invariant:** Principles 1, 4, 5 (correctness by construction; propose-then-dispose).
- **Status in `eak/`:** ✅ **built** (this is today's MVP skeleton — 236 tests, the full 15-phase
  pipeline, the verification engine, the gate).

> **You-are-here line:** the current codebase has completed Phases 0–2. Everything below is the build
> that remains. Phase 3 (Band A) is the next construction, and it is deliberately the cheapest.

### Phase 3 — Band A: the epistemic Maps *(make the reasoning honest)*
- **Build:** the judgment/epistemic objects — **Assumption** (dischargeable), **Risk**,
  **Tradeoff/Objective**, and **Fidelity** (a tag on every derived fact); deepen **Constraint** toward
  a real calculus; add the **Change/Revision** spine (git-for-hardware v0).
- **Why here:** these are domain objects + gates, *no heavy numerics* — the cheapest high-value layer,
  and the one that makes AI *demonstrably* trustworthy (it must declare and discharge its assumptions;
  release blocks on undischarged critical ones). It slots directly onto the Phase-2 seam.
- **Introduces:** `Assumption`, `Risk`, `Objective`/`Tradeoff`, `ModelFidelity`; a release gate on
  undischarged critical assumptions; `Revision`/`Diff` objects.
- **Exit criterion:** a run records its assumptions and refuses to release with an undischarged
  critical one; every predicted fact carries a fidelity tag; a rejected alternative is preserved as a
  Tradeoff.
- **Invariant:** Principle 9 (honesty over fabrication) made operational.
- **Status in `eak/`:** ✅ **built** (2026-07-17, swarm) — `Assumption` + release-blocking honesty
  gate, `ModelFidelity` (advisory trust-tag), `Risk` (human owns acceptance), `Objective`/`Tradeoff`
  (rejected space preserved); ADR-0018–0021; **all three exit criteria met**; tests 235→282. On branch
  `phase-3-band-a` (not pushed). `Revision`/`Diff` spine deferred (optional inc. 5). See
  `13-phase-3-band-a-execution.md`.

### Phase 4 — The surfacing layer *(make it visible — the interface)*
- **Build:** freeze the event/query contract (the projection of owned truth); the desktop shell with
  the runtime as its native core; the panels (agent chat, engineering-state, traceability, review);
  the rendered view; the interop **scaffold** (import→translate + render, expiry-tagged — never a
  standing capability, `00 §10`).
- **Why here:** *after* there is owned truth worth projecting (Principle 3). Building the UI earlier
  means surfacing objects that don't yet exist. The frozen contract converts a solo builder into two
  tracks (owning vs surfacing, `03 §8`).
- **Introduces:** the versioned contract, the shell, the panels, the render host — all *projections*,
  none authoritative.
- **Exit criterion:** the owned skeleton + Band A stream into a native window and a person can drive
  one curated example end-to-end; nothing surfaced is un-owned.
- **Invariant:** Principle 1 (artifacts are projections; never the truth).
- **Status in `eak/`:** ◐ **stubbed** (`app/` spine exists — event-sink bridge + a vanilla feed;
  needs the founder's machine + build-out).

### Phase 5 — Band B: the logical-electrical Maps *(the runtime becomes an architect)*
- **Build:** **Power Domain, Ground/Return, Clock Domain, Pin-Function/Mux, Signal Flow,
  Interface/Contract, Bus/Protocol, Subsystem** — plus a new **Logical-Electrical IR** band between
  Engineering IR and Schematic IR, and their rules (power balance, mux conflict, CDC, protocol,
  return-path continuity).
- **Why here:** this is the layer real engineers design in; it sits *above* the skeleton's blocks/nets
  and *feeds* physical realization. It needs the Phase-2 seam and IR machinery, and it is the first
  *irreplaceable* value.
- **Introduces:** `PowerDomain`, `ReturnPath`, `ClockDomain`, `PinCapability`/`PinAssignment`,
  `Signal`, `Interface`, `Bus`, `Subsystem`; the Logical-Electrical IR; ~a dozen new rules.
- **Exit criterion:** the runtime owns and verifies a power/pin/interface architecture for a real small
  board, every assignment traced to intent and checked by construction.
- **Invariant:** Principles 1, 4, 7 (owned truth; correctness by construction; layered authority).
- **Status in `eak/`:** ◐ **inc 1–6 built** — `PowerDomain` (seam + `erc-power-balance` rule,
  [ADR-0022](../docs/decisions/0022-band-b-power-domain.md)), `ClockDomain` (seam +
  `erc-clock-domain-conflict` rule, [ADR-0023](../docs/decisions/0023-band-b-clock-domain.md)),
  `ReturnPath` (seam + `erc-return-path-required` rule, gated on the design's own
  `impedance_target` declaration per `engineering-science/electrical/transmission-lines.md`
  L145/L170; [ADR-0024](../docs/decisions/0024-band-b-return-path.md)),
  `PinCapability`+`PinAssignment` (seam + `erc-pin-mux-conflict`/`erc-pin-capability` rules,
  [ADR-0025](../docs/decisions/0025-band-b-pin-function-mux.md)), `Signal` (seam +
  `erc-signal-driver-sink` rule — the logical electrical meaning above raw connectivity per §32;
  [ADR-0026](../docs/decisions/0026-band-b-signal-flow.md)), and `Contract`+`Interface` (seam +
  `erc-interface-contract` rule — protocol rule-set and its governed signal collection,
  [ADR-0027](../docs/decisions/0027-band-b-interface-contract.md)); remaining objects one per
  increment.

### Phase 6 — Band C: the behavior/world-model Maps *(checker → generative)*
- **Build:** the **Behavior** Map (component/subsystem behavior over conditions); **Power-Integrity**;
  deepened **Thermal / Signal-Integrity / EMC**; **Reliability**; **Simulation-as-owned-evidence** —
  via a new outer-ring **`eak-solvers`** crate (deterministic analyses + external-solver ports emitting
  fidelity-tagged Evidence; heavy numerics stay out of the kernel).
- **Why here:** this is the leap from *recording* to *understanding* (`01 §Part VII`). It requires the
  Band-B logical-electrical model to have something to reason over, and the Phase-3 Fidelity tag to be
  honest about approximation. It is the deepest moat and the hardest — a deliberate funded bet.
- **Introduces:** `BehaviorModel`, `SimulationRun`, `HeatSource`/thermal network, `PdnModel`,
  `RadiatingStructure`, `FailureMode`; the `eak-solvers` boundary + solver ports.
- **Exit criterion:** the runtime answers "what would happen if…" for thermal/SI/PI/EMC on a real
  board, each prediction fidelity-tagged, contradictions detected by construction.
- **Invariant:** Principles 7, 12 (unlaundered authority; fidelity as an honest, deepening approximation).
- **Status in `eak/`:** ◐ **first-order proxies only** (impedance/thermal/ampacity rules exist as
  floors; no owned behavior model, no solver boundary).

### Phase 7 — Band D: the lifecycle Maps + Engineering Memory *(own the program)*
- **Build:** **Supply, Cost, Compliance, Assembly/DFA, Test/Bring-up, Mechanical**, graduated
  **Authority/Autonomy** — and stand up **`eak-memory`**, the knowledge tier that accrues validated,
  evidence-backed, tiered (private/org/universal) knowledge across projects.
- **Why here:** the lifecycle wraps the design, and **Memory** can only compound *after* there are
  completed, verified, released designs to learn from (it feeds on the output of Phases 2–6). It is the
  moat that grows.
- **Introduces:** `SourcingRecord`, `CostModel`, `ComplianceTarget`, `DfaConstraint`, `TestPoint`,
  mechanical objects, per-concern autonomy; the `eak-memory` crate + the tiered-sharing model.
- **Exit criterion:** cost/supply/compliance are owned truth; a completed design deposits validated
  lessons that measurably improve the next design's proposals and checks.
- **Invariant:** Principles 3, 9, 11 (provenance; honesty; humans own goals — esp. what Memory shares).
- **Status in `eak/`:** ○ **not built** (learning engine specced only).

### Phase 8 — The Engineering Operating System
- **Build:** deepen the World Model until the runtime can **drive** end-to-end under supervision;
  Memory becomes an industry-scale corpus; generalize **beyond the board** toward system co-design
  (firmware, mechanical, thermal).
- **Why here:** driving requires a faithful world model (Phase 6) + accumulated Memory (Phase 7) +
  graduated trust — it is the top of the stack by construction.
- **Exit criterion:** the runtime proposes, plans, realizes, predicts, verifies, and justifies a
  design end-to-end under supervision, on real programs, with the board as one of its outputs.
- **Invariant:** all twelve principles hold; autonomy rises only as the substrate earns the trust.
- **Status in `eak/`:** ○ **the destination.**

---

## 3. The build dependency graph (why the order cannot be permuted)

```
 Units ─┐
 Id ────┼─► Event log ─► Fold ─► Commit path ─► Capability seam ─► Replay ──┐   (L0 substrate)
 Clock ─┘                                     │                            │
                                    Reasoning port (stub) ─────────────────┤
                                                                           ▼
                            Object meta-model ─► Intent slice ─► Provenance (L1)
                                                                           │
                                       ┌───────────────────────────────────┘
                                       ▼
        Two-part agent ─► Skeleton Maps ─► IR projections ─► Verification ─► Orchestrator ─► Gate (L2)
                                       │
                                       ├─► Band A (epistemic)  ──────────────── (L3, cheap, next)
                                       │
                                       ├─► Surfacing / interface ─────────────── (L4, needs owned truth)
                                       │
                                       └─► Band B (logical-electrical) ─► Band C (behavior + solvers)
                                                                              └─► Band D (lifecycle + memory) ─► OS
```

**Hard orderings (cannot be permuted):** substrate before objects; provenance before breadth; skeleton
before Band A; **owned before surfaced** (L2/L3 before L4); logical-electrical (B) before behavior (C,
which reasons over it); everything before Memory (D, which learns from completed designs). **Soft
parallelism:** within a Band, individual Maps can be built concurrently; surfacing (L4) can proceed in
parallel with Band A once the contract is frozen.

---

## 4. Where the current repository sits (the position marker)

| Phase | Layer | Status |
|---|---|---|
| 0 Deterministic substrate | L0 | ✅ **built** |
| 1 Meta-model + Intent + provenance | L1 | ✅ **built** |
| 2 Skeleton Maps + IR + verify + orchestrator + gate | L2 | ✅ **built** (236 tests) |
| 3 Band A — epistemic Maps | L3 | ✅ **built** (282 tests; ADR-0018–0021; branch `phase-3-band-a`) |
| 4 Surfacing / interface | L4 | ◐ **stubbed** (`app/` spine; needs founder's machine) |
| 5 Band B — logical-electrical | L5 | ◐ **inc 1–6 built** (`PowerDomain` + `ClockDomain` + `ReturnPath` + `PinCapability`/`PinAssignment` + `Signal` + `Contract`/`Interface` + `erc-power-balance`/`erc-clock-domain-conflict`/`erc-return-path-required`/`erc-pin-mux-conflict`/`erc-pin-capability`/`erc-signal-driver-sink`/`erc-interface-contract`; 375 tests; ADR-0022–0027) |
| 6 Band C — behavior/world-model + solvers | L6 | ◐ first-order proxies only |
| 7 Band D — lifecycle + memory | L7 | ○ not built |
| 8 Engineering OS | L8 | ○ destination |

**Read:** the hard part — the substrate and a correct, replayable skeleton — **exists**. From-scratch,
Phases 0–2 are typically the majority of the *risk* and a large share of the *effort*, and they are
done. The remaining build is *pouring Maps into a proven substrate* (`02 §Part III`: "the gap is
content, not structure"). The immediate next brick is **Phase 3 / Band A** — cheap, and the one that
makes the moat visible.

---

## 5. Cross-cutting build disciplines (hold on every phase)

- **Green-gate every increment** — build + lint + format + tests, count ratchets up never down; commit
  only green (the repo's existing protocol).
- **Tests/fixtures first** — a failing test or cassette before the object it verifies.
- **Replay-as-CI** — every phase's exit gate includes a byte-identical replay assertion; determinism is
  a test, not a hope.
- **Canonical-first** — freeze a contract/IR/seam before parallel work forks off it; changing a frozen
  seam is an ADR + version bump, never an edit.
- **One object per increment** — a Map is added object-by-object through the seam, each with its twelve
  facets, each independently revertable.

---

## 6. If truly starting from zero — the first ten bricks (Phase 0→1)

A concrete commit sequence to bootstrap the substrate, in order:

1. Workspace + ring skeleton + the dependency-guard test (fail the build on an outward edge).
2. The physical-quantity type system (dimensions, units, tolerance, dimensional-safety).
3. `EntityId` (opaque, never-null) + the logical clock + the seeded id source.
4. The `Event`/`EventRecord` types + the append-only event log (JSON-lines) + `read_all`.
5. `EngineeringState` + the fold (`apply`), ignoring nothing it should keep.
6. `RuntimeCore::commit` — stamp → append → fold — as the *only* mutator.
7. `replay` — re-fold the log without clock/model; assert byte-identical state.
8. The `CapabilityRequest` seam + one handler that validates and rejects a malformed proposal.
9. The `ReasoningEngine` port + a stub (fixture) reasoner; record every call as an event.
10. The first object — `DesignIntent` — captured → committed → folded → traced → replayed. **The whole
    spine, proven on one Map.**

After brick 10, you have an Engineering OS that owns exactly one thing correctly — and every remaining
Map in `02-engineering-world-model.md` is a repetition of that same, proven pattern.

---

*This is the construction order. `03-roadmap.md` sequences the same build against a timeline and a
raise (OWNED vs SURFACED, stages, contingency); `02-engineering-world-model.md` names every Map this
order pours in. No code was written or modified in producing this document.*
