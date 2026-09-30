# ADR-0018 — `Assumption`: the honesty object and the release-blocking honesty gate

**Status:** Accepted (Phase 3 / Band A, increment 1 — "Assumption + honesty gate"). Delivers Band A
exit criterion 1 of `project-plans/12-band-a-implementation-plan.md`. Anchored to
`project-plans/00-product-vision.md` (Principle 7 — honesty; Principle 10 — the decider is named;
Principle 13 — failures are explicit) and Map 10 of `project-plans/02-engineering-world-model.md`.

## Context
Phases 0–2 gave the runtime a deterministic, event-sourced substrate that models the *design*
(requirements → constraints → schematic → BOM → PCB → routing → manufacturing IR) and a **global
manufacturing gate** (`eak-phases/src/manufacturing_generation.rs`, `CheckingGate`) that refuses to
release while any open, blocking (error-severity) [`Violation`] remains anywhere in the design.

What the runtime did **not** model is *the reasoning about the design* — specifically the load-bearing
presumptions the AI (or a human) makes along the way ("the USB-C source can deliver 5 V @ 3 A", "the
ambient stays below 60 °C"). Those presumptions were invisible: nothing recorded them, nothing forced
them to be grounded, and a design could be "released" while resting on unverified guesses. That is
exactly the confident-fiction failure the moat exists to prevent, and it undermines the marquee "AI
you can trust" story: an honest agent must *declare* its assumptions and must not be allowed to ship a
design that still rests on an undischarged **critical** one.

The construction risk here is content, not structure: an `Assumption` is a repetition of the proven
Phase-2 object pattern (`Constraint`/`Violation`/`Waiver`) through the same capability seam, fold, and
gate machinery.

## Decision
Add a first-class, auditable **`Assumption`** engineering object with a lifecycle, route every
mutation through the existing capability seam with P3 re-validation, fold it deterministically, and
**extend the global manufacturing gate** to also refuse release while an undischarged *critical*
assumption remains.

- **Domain (`eak-domain`).** `struct Assumption { id, statement, rests_on: EntityId, criticality,
  status, discharge: Option<Discharge> }`, with `enum AssumptionCriticality { Critical, Normal }`,
  `enum AssumptionStatus { Open, Discharged, Invalidated }`, `enum DischargeResolution { GroundedFact,
  EnforcedConstraint, AcceptedRisk }`, and `struct Discharge { resolution, target: EntityId,
  decided_by: String }`. `Assumption` has no `f64`/`PhysicalQuantity` field, so `Eq` is legal (unlike
  `Component`/`Net`). No `Default` on the enums: every assumption states its criticality explicitly,
  like `ViolationSeverity`. `Discharge.decided_by` names the decider (Principle 10), mirroring
  `Waiver.decided_by`.
  - `Assumption::is_blocking()` mirrors `Violation::is_blocking()` **exactly**: `Critical AND Open`
    blocks; discharged/invalidated/normal does not. The honesty gate reads this predicate.
  - `Assumption::validate()` reuses **existing** `DomainError` variants only (no new variant):
    non-empty statement (`EmptyStatement`); and a discharge record present **iff** `status ==
    Discharged` (`Inconsistent`, both directions). The `rests_on`/discharge-target referential checks
    live at the capability seam, not in the entity.

- **Events (`eak-ports`).** Two new **state-delta** variants: `AssumptionRaised { assumption }` (fold
  pushes) and `AssumptionDischarged { assumption: EntityId, discharge }` (fold finds by id, flips
  `status → Discharged`, records the `Discharge` on it). Both get an **explicit fold arm** in
  `EngineeringState::apply` — the one sharp edge; the `_ => {}` catch-all would otherwise silently
  diverge replay (P4).

- **Seam (`eak-runtime`).** New `CapabilityRequest::RaiseAssumption { assumption, links }` (the
  `{payload, links}` shape of `CreateConstraint`) and `DischargeAssumption { assumption, discharge }`
  (the id-targeting shape of `GrantWaiver`), wired into the exhaustive `invoke` dispatch. The handlers
  re-validate at the seam (P3 — the model is never trusted):
  - `handle_raise_assumption`: `validate()`; reject a null `rests_on` (untraceable); reject a
    `rests_on` that does not resolve to a committed requirement / decision / functional block; reject
    an assumption not born `Open` (never born discharged).
  - `handle_discharge_assumption`: the target assumption must exist and still be `Open` (no
    re-discharge, no discharging an `Invalidated` one); the `Discharge.target` must be non-null and
    resolve to some committed entity (requirement / decision / functional block / constraint). A
    rejection commits nothing.
  - Two new `AgentContext` readers (`assumptions()`, `undischarged_critical_assumptions()`), added to
    **both** impls (`RuntimeCore` and the `NoopCtx` test double), returning owned `Vec<Assumption>`
    like every other reader.

- **Gate (`eak-phases`, the honesty gate).** `CheckingGate` now blocks release when
  `open_blocking_violations > 0` **OR** `undischarged_critical_assumptions > 0`, with a **distinct**
  message ("the design rests on unverified presumptions"). A merely-**Normal** open assumption is
  surfaced but does **not** block — matching the roadmap's "undischarged *critical*". This is the
  runtime enforcing Principle 7: the AI cannot ship a design resting on a presumption it has not
  grounded.

## Consequences
- **Honesty is now enforced, not hoped (exit criterion 1).** A run that raises a critical assumption
  and leaves it open is `Blocked` at release with a distinct reason; discharging it (grounding it
  against a committed entity) unblocks release. An open Normal assumption never blocks. This is the
  "declare and discharge your assumptions" guarantee made mechanical.
- **Determinism/replay intact (P4).** Both new events are state deltas with explicit fold arms; the
  fold is the only mutation path. The QA suite asserts **byte-identical** replay (`canonical_json`
  equality across a re-fold) after raise, after discharge, and across a full blocked-then-released
  pipeline run. The gate reads state only (no clock/model), so replay is unaffected.
- **P3 seam re-validation intact.** Malformed (empty statement, born-discharged), dangling (null /
  unknown `rests_on`), and illegitimate discharge (unknown target, non-open, dangling discharge
  target) proposals are all rejected before anything enters the log — the model/agent is never
  trusted.
- **Clean-architecture rings unchanged.** All edges point inward: `eak-domain` gains the object;
  `eak-ports` the events; `eak-runtime` the seam/fold/readers; `eak-phases` reads the gate query.
  No adapter dependency was introduced.
- **`AcceptedRisk` coupling deferred (v0).** `DischargeResolution::AcceptedRisk` names the concept,
  but `Risk` is a later Band-A increment. Per the plan §7 item 4, v0 leaves `Discharge.target` a free
  `EntityId` that must only resolve to *some* committed entity; the `AcceptedRisk → Risk` seam
  tightening lands with the `Risk` object. This is a documented, intentional deferral, not a relaxed
  invariant.
- **Schema version.** The additions are new `Event` and `CapabilityRequest` variants plus new domain
  types — additive, not a breaking re-shape of an existing carrier. `Event::AssumptionRaised` /
  `AssumptionDischarged` carry no schema-version envelope (consistent with every other state-delta
  event), and no IR schema constant changed, so there is no monolithic `u32` to bump.

## Alternatives considered
- **Model an assumption as advisory metadata (like `ViolationExplanation`) rather than a first-class
  entity** — rejected: an assumption has a *lifecycle* (Open → Discharged | Invalidated) and must
  *gate* release, so it needs its own id, `validate()`, and seam. Advisory metadata can never gate,
  by construction. (Fidelity, increment 2, *is* the tag case and stays advisory.)
- **Block release on ANY open assumption, not just Critical** — rejected: it over-blocks and erases
  the criticality signal; the roadmap and Principle-7 honesty target load-bearing presumptions.
  Normal open assumptions are surfaced, not blocking.
- **Let an assumption be born `Discharged` (seed a grounded fact directly)** — rejected: it hides the
  presumption's moment of being made and muddies the audit trail; a discharge is a distinct, named
  decision that must follow an open assumption.
- **Reuse `GrantWaiver` semantics to discharge** — rejected: a waiver *accepts a defect*; a discharge
  *grounds a presumption* (to a fact, a constraint, or a risk). They are different judgements and
  deserve different objects and messages.
