# Phase 3 — Band A execution ledger (2026-07-17 swarm)

> The record of the multi-agent swarm run that built **Band A — the epistemic Maps** (the next phase
> in `11-build-roadmap.md`). Plan: `12-band-a-implementation-plan.md`. Anchored to
> `00-product-vision.md` (Principle 7 — honesty over fabrication). All four increments are committed
> green on branch **`phase-3-band-a`** (not pushed — canonical-remote rule + founder review first).

---

## 1. Outcome in one line

**All four Band A objects built, green, replay-verified — the three Band A exit criteria are met.**
Tests **235 → 282** (+47). ADRs **0018–0021**. Kernel green-gate clean (build · clippy `-D warnings` ·
fmt · test).

## 2. What shipped (one commit per increment — "one object per increment" discipline)

| # | Increment | Object(s) | Commit | Exit criterion |
|---|---|---|---|---|
| 1 | **Assumption** + honesty gate | `Assumption` (dischargeable) + `AssumptionRaised`/`AssumptionDischarged`; the global manufacturing gate now **blocks release on undischarged *critical* assumptions** | `4356f17` | **#1 met** — a run refuses to release with an undischarged critical assumption |
| 2 | **ModelFidelity** | `ModelFidelity{concern,method,confidence,scope}` as an **advisory trust-tag** on derived facts (modeled like `ViolationExplanation` — its own store, never mutates the target) + `FidelityTagged` | `a5317bc` | **#2 met** — every predicted/derived fact can carry a fidelity tag |
| 3 | **Risk** | `Risk{likelihood,severity,mitigation,residual,owner,status}` + `RiskRaised`/`RiskAccepted`; **the human owns residual acceptance** (Principle 11); non-blocking in v0 | `d9f353d` | (auditable risk posture) |
| 4 | **Objective / Tradeoff** | `Objective` + `Tradeoff{alternatives,criteria,chosen,rejected,rationale}` + `ObjectiveRecorded`/`TradeoffRecorded`; a `Decision` may cite the `Tradeoff` it resolved | `b533b49` | **#3 met** — a rejected alternative is preserved as a Tradeoff |

Each increment traversed the proven 7-touchpoint seam pattern (domain type + `validate()` → `Event`
variant → `EngineeringState` fold arm + accessor → `CapabilityRequest` + `AgentContext` reader →
`handle_*` seam re-validation → gate/rule → tests incl. **byte-identical replay**).

## 3. The swarm

- **16 agents · ~1.21M subagent tokens · 576 tool uses · ~2.1 h wall-clock.**
- **Architecture phase (parallel):** `eak-architect` froze the four contracts against live code
  (`design_frozen = true`); an adversarial vision-alignment critic (applying *objectification-by-
  consequence*, *layered-authority*, *runtime-owns-knowledge*) returned **`approved_with_changes`** and
  confirmed the four open decisions: assumptions raised via fixture in-kernel for inc. 1 (live agent =
  fast-follow); **Fidelity is a tag, not an entity**; the gate blocks on **Critical only**;
  `Discharge.target` stays a free `EntityId` until Risk lands.
- **Build phase (sequential — the increments share the kernel enums/traits, so serialized to avoid
  clobbering):** per increment, `eak-qa-test-engineer` wrote failing tests first (TDD), then
  `eak-kernel-engineer` implemented to green + wrote the ADR + a replay test + committed.
- **Verify phase (parallel per increment):** `eak-rust-reviewer`, plus `eak-security-reviewer` and
  `eak-verification-engineer` on the honesty gate, re-confirmed green and audited the seam.

## 4. Honest caveats & the network-failure recovery

- **The swarm's last two agents died on a transient API socket error** (`FailedToOpenSocket`), not a
  code fault: `build:tradeoff` had already written green tradeoff code (domain types + ADR-0021) but
  crashed **before verifying and committing**; `tpm:ledger` never ran. Recovery (this session): the
  uncommitted tradeoff work was independently green-gated (**282 tests, clippy/fmt clean**), its
  seam-validation + replay tests confirmed (`tradeoff_rejects_fewer_than_two_alternatives`,
  `..._chosen_index_out_of_range`, `..._when_no_rejected_alternative_is_preserved`, byte-identical
  replay), and committed as `b533b49`; this ledger + the roadmap status were then written.
- **Deferred (in scope for Band A, not built):** the **Change/Revision spine** (`Revision`/`Diff`,
  git-for-hardware v0 — optional inc. 5) and the deepening of `Constraint` toward a real calculus.
- **Fast-follow (harness):** wire the reasoning agents to *raise* assumptions during a live run (+ a
  cassette) so the demo shows the AI declaring its own assumptions — the honest form of exit criterion 1.

## 5. What remains on the roadmap (explicitly out of scope for this run)

Bands B/C/D → the Engineering OS are **not** completable in a kernel-only run: **Band B**
(logical-electrical: power/clock/interface + a new IR band) is buildable-here next and can run in
parallel with the deferred Revision spine; **Band C** (behavior + solvers) needs external solver ports;
**Band D** (lifecycle + cross-project memory) needs a memory engine; **Phase 4 surfacing** is the
interface layer. This run advanced exactly one layer: **the runtime now owns the reasoning about the
design, not just the design** — Principle 7 made operational.

---

*State of truth: branch `phase-3-band-a`, 4 commits `4356f17 → b533b49`, **not pushed**. Green-gate:
`cargo build/clippy -D warnings/fmt/test` all clean at 282 tests.*
