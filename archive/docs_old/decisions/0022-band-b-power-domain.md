# ADR-0022 — `PowerDomain`: the power architecture as a verifiable, traceable rail

**Status:** Accepted (Phase 5 / Band B, increment 1 — "PowerDomain"). First increment of the
Band B **logical-electrical Maps** layer of `project-plans/11-build-roadmap.md` (Phase 5, Band B)
and `project-plans/02-engineering-world-model.md` (§Band B — logical-electrical maps: power,
clock, pin, signal, interface). Anchored to `project-plans/00-product-vision.md` (Principle 3 —
the kernel is the moat, the model never usurps an object's authority; Principle 7 — honesty; P9 —
first-class physical quantities) and the power-integrity groundings in
`engineering-science/electrical/kirchhoff-laws.md` and `power-integrity.md`. Delivers Band B
**exit criterion 1** (as scoped by `project-plans/12-band-a-implementation-plan.md`'s sibling Band B
plan): *the runtime owns a committed power architecture whose rails are verified by construction.*

## Context
Phase 3 closed the loop from intent to a routable board: requirements → blocks → components →
nets → board → placements → tracks, each object committed through the capability seam and verified
by a rule engine. Phase 3's electrical checks reason per-net (undriven power net, contended
drivers). What the runtime still could not express is the **power architecture itself**: the rail a
set of nets must hold at a voltage, the component that supplies it, and the **budget** that supplier
must satisfy. Without a first-class `PowerDomain`, the load on a rail is invisible to the runtime —
KCL is an engineering-science principle the kernel has no object to apply it to.

A power rail is not an incidental grouping: it is a **named, traceable engineering object** with
stable identity, a supplying component (the traceability anchor back to intent, P3), and an
intrinsic budget invariant — *the sum of what the rail must deliver cannot exceed what its source
can deliver.* A design whose rails overload silently is a design that will fail at bring-up; the
runtime should flag it **at construction**, before the schematic is ever routed.

## Decision
Model the power architecture as **a first-class, auditable `PowerDomain` entity** raised through the
capability seam (P2/P3), carrying its supply-side budget as a first-class physical quantity, and give
the rule engine a KCL **power-balance** check over it. A well-formed-but-overloaded domain is still
**accepted into state** — the rule engine, not the seam, is the authority on design judgement (a
rail that is heavier than its budget is a *violation to report*, not an *illegal commit*).

- **Domain (`eak-domain`).**
  - `struct PowerDomain { id: EntityId, name: String, voltage: PhysicalQuantity,
    source_component: EntityId, max_current: PhysicalQuantity, nets: Vec<EntityId> }` — a rail that
    must hold `voltage`, supplied by `source_component`, with the supplier's deliverable `max_current`
    and the member `nets` it powers. `validate()` reuses existing `DomainError` variants only:
    a non-empty `name` (`DomainError::EmptyField("power domain name")`), a **finite, positive**
    `voltage`, a **finite, positive** `max_current`, and **≥ 1 net** (a rail powering nothing is a
    silent defect — mirroring `component has no pins`). Values are compared via `si_magnitude()` so
    the checks are unit-independent (P9).
  - **Overload is not a validation error.** `validate()` enforces only *well-formedness*. A domain
    whose loads exceed its budget is well-formed but *unsafe* — a design-flaw to be reported by the
    rule engine, never an illegal commit. This split keeps the seam mechanical (shape + referential
    integrity) and the design judgement in the verification layer where violations are first-class
    and explainable.
  - `PowerDomain` carries an `f64` (`voltage`/`max_current` are `PhysicalQuantity`), so it derives
    `PartialEq` but **not** `Eq` (mirrors `Decision`/`Component`/`Objective`/`Tradeoff`). No new
    `DomainError` variant is invented.

- **Events (`eak-ports`).** One new **state-bearing** variant with an **explicit fold arm** in
  `EngineeringState::apply` (the one sharp edge; the `_ => {}` catch-all would otherwise silently
  diverge replay, P4): `PowerDomainCommitted { domain: PowerDomain }`.

- **State (`eak-runtime`).** A new store `EngineeringState::power_domains: Vec<PowerDomain>` kept in
  insertion (event) order; accessor `power_domain(id)`.

- **Seam (`eak-runtime`).** `CapabilityRequest::CreatePowerDomain { domain, links }` re-validates the
  domain (non-empty name, positive voltage, positive max current, ≥ 1 net) **and** enforces the
  referential-integrity invariants the rule engine cannot infer (P3, P5): a **null** source component
  is rejected, an **unknown** source component is rejected, and **every** rail net must be a committed
  net — a domain can never reference a phantom rail. A rejected proposal commits **nothing** to the
  log. Readers `power_domains()` / `power_domain(id)` added to `AgentContext` and its impls.

- **Verification (`eak-engines` / `eak-phases`).** `VerificationContext` gains
  `power_domains: &[PowerDomain]` (all construction sites updated). New rule
  `PowerBalanceRule` (`erc-power-balance`), registered on the ERC engine alongside the Phase-3
  rules: for each domain, sum the **declared** currents of its member nets and compare against the
  domain's `max_current` (relative-epsilon compare, the same idiom as the geometry rules). An
  overloaded rail is a **blocking `Error`** finding naming the domain. A net that states **no** current
  contributes nothing — the per-net "no input → silent" discipline, exactly like the ampacity rule:
  the rule never invents a load it was not given (surfacing under-specification is the `ModelFidelity`
  tag's concern, not this rule's). Deterministic by construction: domains in slice order, nets in
  domain order (P4).

## Consequences
- **The power architecture is a committed, traceable object (exit criterion 1).** Every rail has a
  stable identity, a named supplying component that resolves to a committed component (→ block →
  requirement → intent, P3), and an auditable budget. The kernel owns the architecture; the model
  can *propose* rails but never bypass the seam.
- **KCL is enforced at construction.** `erc-power-balance` flags an overloaded rail as a blocking
  error before the schematic is routed. The QA suite asserts: a rail within capacity passes, an
  overloaded rail is flagged (Error, naming the domain), a rail at exactly its capacity passes
  (floating-point tolerance), undeclared loads stay silent, and an empty architecture yields nothing.
- **Seam re-validation holds (P3).** Four malformed proposals — a blank rail name, a null source, a
  dangling source, and a dangling rail net — are each rejected at the seam and **nothing** enters the
  log. The QA suite asserts all four plus the empty-log-on-rejection property, and the
  accept-path assert on `power_domain(id)`.
- **Determinism/replay intact (P4).** `PowerDomainCommitted` is folded by exactly one explicit arm;
  the fold is the only mutation path. The QA suite asserts **byte-identical** replay
  (`canonical_json` equality across a re-fold) with the power domain in the log.
- **Verification context is uniform.** The `power_domains` field is threaded through every phase
  machine (DRC, ERC, BOM, DFM, EMC, constraint) so the engine sees one context shape everywhere; the
  Band B rule simply runs where the power layer is populated.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the entity,
  `eak-ports` the event, `eak-runtime` the store/fold/accessor/seam, `eak-engines` the rule,
  `eak-phases` the registration. No adapter dependency introduced.
- **Schema version.** Additive — one new `Event` variant plus a new domain entity; no existing
  carrier was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Alternatives considered
- **Reject an overloaded rail at the seam** — rejected: a rail's *shape* (name, voltage, budget, nets)
  is a fact the runtime records; whether its loads fit the budget is a **judgement** the verification
  layer owns, where violations are first-class, explainable, and can be waived. Rejecting at the seam
  would conflate "this is a malformed commit" with "this is a design that will not work", and would
  make an overloaded-but-honest architecture unrepresentable.
- **Infer a `PowerDomain` from per-net attributes** (e.g. every net's voltage) — rejected: a rail is a
  deliberate, owned object with a named supplier and an explicit budget; inferring it would fabricate
  a source component and a capacity the design never declared (P3, P7 honesty). The domain is entered
  and owned, like every Band A object.
- **Fold the load budget into a `Component` field** (e.g. `Component.max_current`) — rejected: the
  budget is a property of the **rail**, not the component (one regulator may feed several rails, and
  one rail may be fed by several parallel sources in later increments). `PowerDomain` is the correct
  unit of KCL bookkeeping; the component is just its traceability anchor.
- **Warn (not Error) on overload** — rejected: an overloaded rail is a physical infeasibility, not a
  style preference. It blocks release, matching the severity discipline of the Phase-3 ERC rules.
- **Skip the "declared-only load" discipline and treat unstated currents as zero for a Warning** —
  rejected for v0: the per-net "no input → silent" rule is a settled seam-wide discipline (P13).
  Folding a warning about unstated loads into this rule would blur its single job; under-specification
  is the `ModelFidelity` tag's concern and stays out of the balance rule.