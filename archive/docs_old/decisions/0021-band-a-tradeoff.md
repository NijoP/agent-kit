# ADR-0021 — `Objective` / `Tradeoff`: the weighed-and-rejected space, preserved

**Status:** Accepted (Phase 3 / Band A, increment 4 — "Objective / Tradeoff"). Part of the Band A
epistemic layer of `project-plans/12-band-a-implementation-plan.md` (increment 4). Anchored to
`project-plans/00-product-vision.md` (Principle 7 — honesty about what the design did *not* choose and
why; Principle 10 — the decider is named; Principle 3 — the kernel is the moat, the model never usurps
an object's authority) and Map 11 of `project-plans/02-engineering-world-model.md`. Delivers Band A
**exit criterion 3**: *a rejected alternative is preserved as a Tradeoff*.

## Context
After increments 1–3, the runtime models load-bearing presumptions (`Assumption`, which *blocks*
release on an undischarged critical one), the confidence behind derived facts (`ModelFidelity`, an
advisory tag), and the risk posture (`Risk`, tracked truth the human owns). What it still could not
express is **the reasoning about the choice itself**: the goals the design was optimizing for, the
alternatives that were weighed against them, and — decisively — **the space the design considered and
then rejected**.

A design that records only its final choice is dishonest by omission: it cannot answer *"what else did
you consider, and why did the chosen option win?"* (Principle 7). The engineering value of a tradeoff is
almost entirely in the **rejected** alternatives it preserves — that is the institutional memory that
stops a later engineer re-litigating a settled question, or silently regressing to an option that was
already ruled out. The construction risk is content, not structure: `Objective`/`Tradeoff` are a
repetition of the proven Phase-2 object pattern (`Constraint`/`Waiver`/`Risk`) through the same seam.

## Decision
Model the weighed-and-rejected space as **two first-class, auditable entities** raised through the
capability seam (P2/P3), whose defining invariant is that the **rejected space is preserved and
re-checked by the runtime** — the model is never trusted to have kept it.

- **Domain (`eak-domain`).**
  - `struct Objective { id: EntityId, statement: String, weight: f64, source: EntityId }` — a weighted
    design goal rooted in the committed entity (`source`) it derives from. `validate()` reuses existing
    `DomainError` variants only: a non-empty `statement` (`DomainError::EmptyStatement`).
  - `struct Alternative { label: String, description: String, scores: Vec<f64>, rejected: bool }` — one
    weighed option; `scores` are positional against the tradeoff's `criteria`; `rejected` records whether
    the design set it aside.
  - `struct Tradeoff { id: EntityId, question: String, alternatives: Vec<Alternative>, criteria:
    Vec<String>, chosen: usize, rationale: String, decided_by: String }` — `decided_by` names the
    decider (Principle 10, mirroring `Waiver::decided_by`). `validate()` enforces four intrinsic
    invariants, **reusing existing `DomainError` variants only**: (1) **≥ 2 alternatives** (a single
    option weighed nothing — `Inconsistent`); (2) `chosen` is a **valid index** (`Inconsistent`); (3) the
    chosen alternative is **not itself marked `rejected`** (a self-contradiction — `Inconsistent`); and
    (4) **≥ 1 rejected alternative is preserved** (`Inconsistent`) — the whole point of the object is to
    remember the space it did not choose (exit criterion 3).
  - Both `Objective` and `Tradeoff` carry an `f64` (`weight` / `scores`), so — unlike `Assumption`/`Risk`
    — they derive `PartialEq` but **not `Eq`** (mirrors `Decision`/`Component`). No new `DomainError`
    variant is invented.

- **Events (`eak-ports`).** Two new **state-bearing** variants, both given **explicit fold arms** in
  `EngineeringState::apply` (the one sharp edge; the `_ => {}` catch-all would otherwise silently diverge
  replay, P4): `ObjectiveRecorded { objective: Objective }` and `TradeoffRecorded { tradeoff: Tradeoff }`
  (each fold pushes into its own store, so the tradeoff's preserved rejected alternatives are
  reconstructed byte-for-byte on replay).

- **State (`eak-runtime`).** Two stores `EngineeringState::objectives: Vec<Objective>` and `tradeoffs:
  Vec<Tradeoff>`, kept in insertion (event) order; accessors `objective(id)` / `tradeoff(id)`.

- **Seam (`eak-runtime`).** `CapabilityRequest::RecordObjective { objective, links }` re-validates the
  objective (non-empty statement) **and** checks its `source` resolves to a committed entity (P3, P5) —
  a weighted goal can never dangle. `CapabilityRequest::RecordTradeoff { tradeoff, links }` re-validates
  the full tradeoff invariant set (≥ 2 alternatives, `chosen` in range and not rejected, ≥ 1 rejected
  preserved). Both follow the `{payload, links}` shape of `CreateConstraint`. A rejected proposal commits
  **nothing** to the log. Readers `objectives()` / `tradeoffs()` added to `AgentContext` (and every impl
  — `RuntimeCore` and the orchestrator's `NoopCtx` test double).

- **Link to `Decision`.** A `Decision` may cite the `Tradeoff` it resolved via a `ProvenanceLink`
  (`RelationType::JustifiedBy`), tying the chosen option to the rejected space it beat (Map 11). This
  needs no new machinery: `ProvenanceLink` already carries arbitrary `from`/`to` `EntityId`s, so a
  committed `Decision → Tradeoff` edge resolves against both stores.

## Consequences
- **The rejected space is preserved and honest (Principle 7, exit criterion 3).** Every design can now
  record its objectives and the tradeoffs it made, and the runtime *guarantees* at the seam that a
  recorded tradeoff both weighed a real choice (≥ 2 options) and kept at least one rejected alternative.
  A design can always answer "what else was considered, and why did the winner win?".
- **Seam re-validation holds (P3).** Four malformed proposals — a single-alternative "tradeoff", an
  out-of-range `chosen`, a `chosen` that points at a rejected alternative, and a tradeoff that threw its
  rejected space away — are each rejected at the seam and nothing enters the log. The QA suite asserts
  all four and that the log stays empty on rejection. An objective rooted in an empty statement (or an
  unknown/`null` source) is likewise rejected.
- **Provenance ties the choice to what it beat.** The QA suite records a `Tradeoff`, then commits a
  `Decision` that cites it via a `ProvenanceLink`, and asserts the edge resolves against both the
  committed decision and the committed tradeoff — the choice is never orphaned from its rejected space.
- **Determinism/replay intact (P4).** `ObjectiveRecorded`/`TradeoffRecorded` are folded by exactly two
  explicit arms; the fold is the only mutation path. The QA suite asserts **byte-identical** replay
  (`canonical_json` equality across a re-fold) with objectives, tradeoffs, and the decision→tradeoff link
  in the log — so the preserved rejected space survives serialization intact.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the two entities +
  `Alternative`, `eak-ports` the two events, `eak-runtime` the stores/folds/accessors/seam. No adapter
  dependency introduced.
- **Schema version.** Additive — two new `Event` variants plus new domain entities; no existing carrier
  was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Alternatives considered
- **Fold the rejected space into a single field on `Decision`** (e.g. `Decision.rejected: Vec<String>`) —
  rejected: it would make the rejected space second-class, un-addressable, and unvalidated. A `Tradeoff`
  is genuinely first-class — it has a stable **identity** (a `Decision` references it), its own
  **invariant** (≥ 2 alternatives, a preserved rejected set), and a named **decider**. Those are exactly
  the properties that warrant a full entity through the capability seam, not a bag of strings.
- **Enforce the objective's `weight` to a `[0,1]` range** — rejected for v0: weights are *relative* among
  competing objectives, and a normalization scheme is a modeling decision the runtime should not
  prejudge. `validate()` therefore constrains only the non-empty statement; the `source` referential
  check at the seam is what keeps an objective honest and traceable. (A later increment may add a
  normalization invariant once objectives are compared numerically.)
- **Cross-check `scores.len() == criteria.len()` in `validate()`** — deferred: it is a plausible tighten,
  but v0's exit criterion is about *preserving the rejected space*, not about score/criteria alignment,
  and over-constraining the object now would reject legitimate partial-scoring records. Left as a
  candidate for a later deepen rather than folded into the frozen increment-4 seam.
- **Auto-derive a `Tradeoff` from competing `Decision`s** — rejected and out of Band A scope (`12 §4`):
  a tradeoff is *entered and owned*, keeping the kernel small and the provenance honest (no synthesized
  tradeoff with no named decider and no author-preserved rejected space).
