# ADR-0019 — `ModelFidelity`: a trust-tag on derived facts (advisory, not an entity)

**Status:** Accepted (Phase 3 / Band A, increment 2 — "ModelFidelity"). Delivers Band A exit
criterion 2 of `project-plans/12-band-a-implementation-plan.md` ("every predicted/derived fact
carries a fidelity tag"). Anchored to `project-plans/00-product-vision.md` (Principle 7 — honesty
about certainty; Principle 3 — the kernel is the moat, the model never usurps an object's authority)
and Map 6 of `project-plans/02-engineering-world-model.md`.

## Context
After increment 1, the runtime models load-bearing presumptions (`Assumption`) and blocks release on
an undischarged critical one. What it still could not express is *how well any given derived fact is
known*: a "worst-case rail droop" number that is a bare guess, a conservative IPC first-order floor, a
hand calculation, a simulation, or a measurement all looked identical in the model. That flattening is
dishonest — the marquee "AI you can trust" story requires the runtime to make the **provenance of
confidence** visible on every predicted fact, so a reviewer can see at a glance which numbers rest on
air and which are grounded (Principle 7 — honest about certainty).

Fidelity is a **tag**, not an authority: it must never be able to change a decision, gate a phase, or
alter the fact it describes. It attaches to *many* future facts (especially Band C's computed
predictions), so building the tag now — before there are many facts to tag — is the cheapest,
highest-leverage brick. The construction risk is content, not structure: `ModelFidelity` is a
repetition of the proven advisory-metadata pattern already used by `ViolationExplanation` (E6 C1).

## Decision
Model fidelity as **advisory attached metadata**, modeled EXACTLY like `ViolationExplanation`: it
carries **no `EntityId` of its own**, references a `target`, folds into its **own** store, and never
mutates the target. It is committed through the audit `emit` seam (not a `CapabilityRequest`), so a
tag can never reach an engineering decision (P3).

- **Domain (`eak-domain`).** `struct ModelFidelity { concern: String, method: FidelityMethod,
  confidence: f64, scope: String }` with `enum FidelityMethod { Assumed, FirstOrderFloor, Calculated,
  Simulated, Measured }` (weakest → strongest). Because it carries an `f64`, it derives `PartialEq`
  but **not** `Eq` — consistent with `Component`/`Net`. Its single domain invariant is the numeric
  boundary `confidence ∈ [0, 1]`, checked in `validate()` and reusing the **existing**
  `DomainError::Inconsistent` variant (no new variant). The check is written
  `!(0.0..=1.0).contains(&c)` precisely so that `NaN` — which is neither `>= 0` nor `<= 1` — is
  **rejected** rather than silently admitted by a naive range comparison.

- **Event (`eak-ports`).** One new **state-bearing but advisory** variant: `FidelityTagged { target:
  EntityId, fidelity: ModelFidelity, reasoning_call_seq: Option<Seq> }`. It is committed exactly like
  `ViolationExplained` — through the audit seam, as metadata. It gets an **explicit fold arm** in
  `EngineeringState::apply` (the one sharp edge; the `_ => {}` catch-all would otherwise silently
  diverge replay, P4). `reasoning_call_seq` is `Some` when the reasoning boundary produced the tag
  (provenance-by-construction) and `None` when the runtime itself tags a fact (e.g. a first-order
  floor).

- **State (`eak-runtime`).** A **separate** store `EngineeringState::fidelity_tags: Vec<FidelityTag>`
  in insertion (event) order, where `FidelityTag { target, fidelity, reasoning_call_seq }` mirrors
  `ViolationExplanation`. The fold pushes here and **never** touches the tagged entity — the
  advisory-only invariant is **structural**, not merely a convention. Accessor `fidelity_for(target)
  -> Vec<&ModelFidelity>` reads all tags for a target back, in order.

- **No gate.** Fidelity does **not** block release in v0. An optional `Warning`-severity rule
  ("derived fact carries no fidelity tag") may be added by the Verification Engineer to make coverage
  *visible*, but it is **non-blocking** by design — fidelity is a lens, not a lock.

## Consequences
- **Coverage is expressible and provable (exit criterion 2).** Every predicted/derived fact the
  pipeline emits can now carry a `ModelFidelity` tag keyed by target, and `fidelity_for` reads them
  back for the review/traceability surface. The tag records the *method* behind the number, so the
  reviewer sees `Assumed` vs `Measured` at a glance (Principle 7).
- **The tag can never usurp authority (P3).** Because it has no id, folds into its own store, and is
  committed as audit metadata rather than through a validating capability, no fidelity tag can ever
  change a `Requirement`/`Constraint`/`Violation` or gate a phase. The QA suite asserts structurally
  that tagging a requirement does not mutate it.
- **Determinism/replay intact (P4).** `FidelityTagged` is folded by exactly one explicit arm; the fold
  is the only mutation path. The QA suite asserts **byte-identical** replay (`canonical_json` equality
  across a re-fold) with fidelity tags in the log, both for a single tag and for multiple tags on the
  same and different targets.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the value type,
  `eak-ports` the event, `eak-runtime` the store/fold/accessor. No adapter dependency introduced.
- **Schema version.** Additive — one new `Event` variant plus a new domain value type; no existing
  carrier was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Alternatives considered
- **A first-class `Fidelity` entity with its own `EntityId` and lifecycle** — rejected for v0: a
  fidelity has **no lifecycle** (it does not transition, it is not discharged, it does not gate) and no
  identity of its own; it only *describes a target*. Giving it an id and a capability seam would be
  heavier and would invite the model to treat it as an authority. Advisory metadata is the honest
  shape (contrast `Assumption`, increment 1, which genuinely has a lifecycle and must gate, and so
  *is* a first-class entity). Reversing this later is a bigger change, hence it is fixed here by ADR.
- **A computed/derived confidence number** — rejected and explicitly out of Band A scope: fidelity is
  a *declared tag*, not a solver output. Computing confidence is Band C's `eak-solvers` boundary; doing
  it here would boil the ocean (`00 §12.3`).
- **Gate release on missing/low fidelity** — rejected for v0: fidelity is *tracked truth a reviewer
  reads*, not a lock. Over-gating on it would block honest low-confidence work. The optional
  `Warning`-severity rule surfaces coverage without blocking.
- **Store tags inline on the tagged entity** (e.g. a `fidelity: Vec<ModelFidelity>` field on
  `Requirement`) — rejected: it would couple the tag to the entity's schema and let a fold that writes
  a tag also touch the entity, breaking the advisory-only structural guarantee. A separate store keeps
  the seam clean, exactly as `ViolationExplanation` does.
