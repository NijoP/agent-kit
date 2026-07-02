# Overnight Execution Plan — 2026-07-03 (5-hour autonomous team run)

> Founder went to sleep and asked the autonomous team to continue MVP development for ~5 hours.
> This doc is the plan the Eng Lead (main loop) executes and the ledger you read on wake.
> Source of truth: [`07-engineering-backlog.md`](07-engineering-backlog.md) (epics E1–E8).
> Protocols: [`../team/operating-protocols.md`](../team/operating-protocols.md) (green-gate §3,
> increment cadence §9, work loop §10).

## Constraint that shapes the whole run

The Tauri desktop app (`../app/`) is **deliberately outside the `eak/` Cargo workspace** (needs
webkit2gtk/node the sandbox can't build). The green-gate can therefore only verify **kernel Rust**
(`eak/crates/*`). So this overnight run attacks the **buildable + testable** MVP-critical items —
the kernel work that makes the hero flow (intent → generate → AI-review, traceable) real and
demoable — and leaves pure-frontend / Tauri-wiring for a founder-at-keyboard session.

Every increment ships behind the full green-gate (`cargo build` + `cargo clippy -D warnings` +
`cargo fmt --check` + `cargo test`, count ratchets up never down) and is committed + pushed
independently, mirroring the existing `Phase N (increment M): <capability>` history.

## Cadence (team operating-protocol §10)

`parallel read-only design swarm → sequential implement → green-gate → domain review → commit → push`

Proven in this repo (increments 15–17). Design agents are read-only ⇒ zero file conflicts ⇒ safe
full parallelism. Implement/gate/commit is serial (one green-gate, one commit at a time).

## Increment queue (priority order, dependency-aware)

| # | Item | Epic | Size | Owner agent | Why it's top-value & verifiable |
|---|------|------|------|-------------|----------------------------------|
| A1 | `ManufacturingIr`/`Board`+`Placement`+`Track`+`PartAssignment` → `.kicad_pcb` **exporter** in `eak-kicad` | E4 | L | eak-canvas-integration-engineer | Unblocks rendering **generated** boards on the reused canvas. **Self-verifying**: export → re-parse with the existing importer → assert entities round-trip. |
| B1 | **Import path through the real capability seam** — feed `eak-kicad::ImportedDesign` through `CreateBoard`/`CreateNet`/`RouteNet` so imported boards get the same validation + event log | E5 | M | eak-kernel-engineer | Completes "import → AI-review always works" with **no back door**. Testable end-to-end in the kernel. |
| B2 | **verify-only workflow** (subset of `default_workflow`: DRC/DFM/EMC/BOM machines) over imported state | E5 | S | eak-kernel-engineer | Produces traceable violations on a real imported board. Depends on B1. |
| C1 | **AI-review explainer** — new reasoning schema `violation_explanation_v1`; each `Violation` → plain-English explanation + suggested fix, linked to subjects + originating requirement; fixture-backed | E6 | M | eak-harness-engineer | The demo "whoa" beat, deterministic via cassette ⇒ testable. LLM only proposes; kernel data grounds it. |
| D1 | **Hero-flow smoke test** — asserts the hero run releases a `ManufacturingIr` and **replays byte-identically** | E7 | S | eak-qa-test-engineer | CI guard against demo rot; hardens the run that *is* the raise. |
| C2 | **Extend reasoning to a 2nd phase** — one more phase gets a real `AgentContext::reason` behind a new schema, mirroring `agent.rs` | E6 | L | eak-harness-engineer | Deepest moat work. Done only if time/context allow after A1–D1. |

## Gate & review matrix (protocol §6)

- All Rust → `eak-rust-reviewer` (build/clippy/fmt/idioms).
- Kernel rules / physics / import fidelity → `eak-verification-engineer` (+ `eak-eda-domain-scientist` if a formula/threshold).
- Tests / fixtures / cassettes → `eak-qa-test-engineer`.
- LLM boundary / new schema → `eak-security-reviewer` (injection, untrusted model output).

## Moat guardrails (non-negotiable, protocol §7)

Determinism, the LLM boundary (LLM proposes, kernel validates), traceability, and byte-identical
replay must not weaken. Any change that would touch them escalates instead of proceeding.

## Live status ledger

(Eng Lead updates this section as increments land. Test ratchet: baseline 195 → **201**.)

- [x] A1 — `.kicad_pcb` exporter (`68b1f32`) — export→import round-trip fixed point; footprints honestly deferred.
- [x] B1 — import → capability seam (`beb35fc`) — needed an architect ruling (ADR-0016 `NetOrigin{Logical,Physical}`); full board+nets+tracks now flow through `commit`, event-log-proven.
- [x] B2 — verify-only workflow (`55f34e8`) — `import_and_verify` trips a traceable `drc-unrouted-net` violation over imported copper, no synthesis, no back door.
- [x] C1 — AI-review explainer (`f4fcd52`) — `ViolationExplained` advisory event; separate store, never mutates the violation; fixture-deterministic; rust+security approved.
- [x] D1 — hero-flow smoke test (`c3c7b56`) — hero intent → released `ManufacturingIr` (checked from raw log) + byte-identical replay.
- [x] C2 — 2nd reasoning phase (`47c6334`) — BOM part selection reasons; LLM proposes, kernel catalog validates; hallucinated MPN provably rejected; rust+verification+security approved.
- [x] E5.1 — default Fabrication process floor on import (`fe9a9f0`) — IPC Class-2 (0.15 mm/6 mil) floor seeded via the real `CreateRequirement` seam; imported too-thin trace now raises a traceable `drc-trace-width` violation. rust + eda-domain-scientist approved (slot contract verified).

## Final tally (2026-07-03 overnight run)

**7 increments, all green + reviewed + pushed. Test ratchet 195 → 209 (+14), zero red at any point.**
`68b1f32` A1 · `beb35fc` B1 (+ADR-0016) · `55f34e8` B2 · `f4fcd52` C1 · `c3c7b56` D1 · `47c6334` C2 · `fe9a9f0` E5.1.

What now demonstrably works, kernel-verified:
- **Generate → canvas**: `ManufacturingIr`/copper → `.kicad_pcb` (round-trip fixed point).
- **Import → AI-review (bulletproof)**: real KiCad board → capability seam → verify-only workflow →
  traceable connectivity **and geometric** DRC violations (trace-width/clearance now fire).
- **The soul (E6)**: every violation gets an advisory AI explanation (deterministic, boundary-safe);
  a 2nd phase (BOM part selection) now reasons with the LLM while the kernel catalog rejects
  hallucinated parts — "LLM proposes, kernel validates" proven by test.
- **Anti-rot**: hero-flow smoke test pins release + byte-identical replay.

Remaining top follow-ups (see Findings): curated hero cassette (E7); import skip-and-warn (#3);
parser depth cap (#4); multi-part catalog set-inclusion (#5); the E5.1 override site. And the
**non-buildable-here** frontend/Tauri wiring (E1 `start_run`, E2 TS event store, E3 panels, E4
KiCanvas embed) — needs a founder-at-keyboard session with the Tauri toolchain.

### C2 follow-up (captured)
5. **Catalog membership is 1:1 today.** `PartSelectionAgent` accepts a proposal only if it equals
   the single `part_for(class)` entry (fails *closed* — safe). When the catalog grows multiple parts
   per class, membership must become **set-inclusion**, else a valid alternative MPN is wrongly
   rejected. Invariant future writers must keep: **never construct a `Part` from model text** —
   always from the trusted `CatalogPart`.

## Findings & follow-ups surfaced by the swarm (for the founder)

1. **ADR-0016 shipped** — `NetOrigin::{Logical,Physical}` distinguishes schematic-synthesized nets
   (must join ≥1 pin) from imported copper nets (pin membership not parsed). Architect-approved,
   determinism-preserving. This is now a load-bearing domain concept.
2. **Import-review only checks connectivity today.** The geometric DRC rules (`drc-trace-width`,
   `drc-copper-clearance`) stay silent on a pure `.kicad_pcb` import because they require a
   **Fabrication process floor** the file doesn't carry. To make "import → AI-review" catch real
   geometry problems, a follow-up must give imported boards a **default or declared process floor /
   constraint set**. High-value next backlog item for the hero fallback.
3. **Import is all-or-nothing.** A segment citing an undeclared / net-0 fails the whole import
   (RouteNet rejects the phantom net). Real boards may hit this — a follow-up should skip-and-warn
   on unresolvable copper rather than abort. (Robustness for the "bulletproof" promise.)
4. **Pre-existing:** the `.kicad_pcb` S-expr parser recurses per nesting depth (stack-overflow risk
   on adversarial input; landed in `fa117a5`, not this run). Consider an iterative parse or depth cap.
