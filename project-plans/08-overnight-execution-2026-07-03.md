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

(Eng Lead updates this section as increments land.)

- [ ] A1 — exporter
- [ ] B1 — import → capability seam
- [ ] B2 — verify-only workflow
- [ ] C1 — AI-review explainer
- [ ] D1 — hero-flow smoke test
- [ ] C2 — 2nd reasoning phase (stretch)
