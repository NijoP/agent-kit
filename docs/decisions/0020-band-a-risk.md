# ADR-0020 — `Risk`: an auditable risk posture the human owns (tracked truth, not a gate)

**Status:** Accepted (Phase 3 / Band A, increment 3 — "Risk"). Part of the Band A epistemic layer of
`project-plans/12-band-a-implementation-plan.md` (increment 3). Anchored to
`project-plans/00-product-vision.md` (Principle 11 — humans own goals/acceptance; Principle 10 — the
decider is named; Principle 7 — honesty about certainty; Principle 3 — the kernel is the moat, the
model never usurps an object's authority) and Map 46 of `project-plans/02-engineering-world-model.md`.

## Context
After increments 1–2, the runtime models load-bearing presumptions (`Assumption`, which *blocks*
release on an undischarged critical one) and the confidence behind derived facts (`ModelFidelity`, an
advisory tag). What it still could not express is the **risk posture** of a design: a named hazard, how
likely and how severe it is, the mitigation in place, and — critically — the **residual** severity that
remains after mitigation and *who is accountable for accepting it*. Increment 1 already referenced the
concept: `DischargeResolution::AcceptedRisk` names accepting a (tracked) risk as one way to discharge an
assumption, but there was no `Risk` object to point at.

Risk differs from an `Assumption` in one decisive way: it is **not** the runtime's job to block on it.
Whether a residual risk is tolerable is an engineering-*goal* judgement, and **humans own goals**
(Principle 11). The honest shape is therefore *tracked truth a named human accepts*, not a gate the
kernel enforces. The construction risk is content, not structure: `Risk` is a repetition of the proven
Phase-2 object pattern (`Assumption`/`Constraint`/`Waiver`) through the same seam.

## Decision
Model risk as a **first-class, auditable entity with a lifecycle**, raised and accepted through the
capability seam (P2/P3), whose acceptance is a **human-authority act** (Principle 11). It is **tracked
truth**: raising or holding an open risk **does not block release in v0**.

- **Domain (`eak-domain`).** `struct Risk { id: EntityId, statement: String, likelihood:
  RiskLikelihood, severity: RiskSeverity, mitigation: String, residual: RiskSeverity, owner: String,
  status: RiskStatus }` with `enum RiskLikelihood { Low, Medium, High }`, `enum RiskSeverity { Low,
  Medium, High, Critical }` (reused for both raw and `residual` severity), and `enum RiskStatus { Open,
  Mitigated, Accepted }`. `Risk` carries **no `f64`/`PhysicalQuantity`** field, so it derives `Eq`
  legally (like `Assumption`, unlike `Component`/`Net`). `validate()` enforces two intrinsic invariants
  and **reuses existing `DomainError` variants only**: a non-empty `statement`
  (`DomainError::EmptyStatement`) and a non-empty `owner` (`DomainError::EmptyField("risk owner")`) — a
  risk with no accountable owner cannot be owned or accepted, so it is rejected. `validate()` is
  deliberately **status-agnostic**: the `Open -> Accepted` transition and the human-acceptance authority
  live at the seam/fold, exactly as `Assumption`'s discharge lifecycle does.

- **Events (`eak-ports`).** Two new **state-bearing** variants, both given **explicit fold arms** in
  `EngineeringState::apply` (the one sharp edge; the `_ => {}` catch-all would otherwise silently
  diverge replay, P4): `RiskRaised { risk: Risk }` (fold pushes into `risks`) and `RiskAccepted { risk:
  EntityId, accepted_by: String }` (fold finds by id and flips `status -> Accepted`). `accepted_by`
  names the decider (Principle 10, mirroring `Waiver::decided_by`).

- **State (`eak-runtime`).** A store `EngineeringState::risks: Vec<Risk>`; accessor `risk(id) ->
  Option<&Risk>`; and a **read** query `unaccepted_critical_risks() -> Vec<&Risk>` filtering `residual
  == Critical && status != Accepted`. That query is a **lens, not a lock**: it surfaces the posture for
  the human; it is never consulted by any gate in v0.

- **Seam (`eak-runtime`).** `CapabilityRequest::RaiseRisk { risk, links }` (following the
  `{payload, links}` shape of `CreateConstraint`) re-validates the risk (non-empty statement + owner)
  before committing (P3); `CapabilityRequest::AcceptRisk { risk, accepted_by }` (id-targeting, no links,
  mirroring `GrantWaiver`) checks the target risk exists and folds its status to `Accepted`. A rejected
  proposal commits **nothing** to the log. Reader `risks()` added to `AgentContext` (and every impl —
  `RuntimeCore` and the orchestrator's `NoopCtx` test double).

- **No gate.** Risk does **not** block release in v0. This is the deliberate contrast with
  `Assumption`'s honesty gate: an undischarged *critical* assumption blocks; an open *critical-residual*
  risk is surfaced but the human owns whether to accept it (Principle 11).

## Consequences
- **The risk posture is expressible and owned.** Every design can now record its hazards with a named
  accountable `owner`, a mitigation, and a residual, and a named human can `AcceptRisk` — the acceptance
  is a recorded, attributable fact (Principles 10/11), not an implicit assumption baked into the model.
- **The kernel never overreaches (Principle 11).** By design, no gate blocks on risk; the runtime tracks
  truth and surfaces it, leaving the goal-judgement of tolerability to the human. The QA suite pins this
  boundary: `unaccepted_critical_risks` is a read that surfaces exactly the critical-residual,
  non-accepted risks and is asserted to be a query, never a release gate.
- **Seam re-validation holds (P3).** A malformed proposal (empty statement or empty owner) is rejected
  at the seam and nothing enters the log; accepting a risk that was never raised is rejected as an
  unknown target. The QA suite asserts both, and that the log stays empty on rejection.
- **Determinism/replay intact (P4).** `RiskRaised`/`RiskAccepted` are folded by exactly two explicit
  arms; the fold is the only mutation path. The QA suite asserts **byte-identical** replay
  (`canonical_json` equality across a re-fold) across raise, accept, and multiple risks in the log.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the entity +
  enums, `eak-ports` the two events, `eak-runtime` the store/fold/accessor/query/seam. No adapter
  dependency introduced.
- **Schema version.** Additive — two new `Event` variants plus a new domain entity and its enums; no
  existing carrier was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Alternatives considered
- **Gate release on unaccepted critical residual risk** — rejected for v0: whether a residual risk is
  acceptable is an engineering *goal* judgement, and humans own goals (Principle 11). Auto-blocking on it
  would usurp the human's authority and block legitimate risk-accepting work. `unaccepted_critical_risks`
  is therefore a **read** the human consults, not a lock. (A later deepen may *optionally* gate once Risk
  can be auto-aggregated from open Assumptions/Violations — explicitly out of v0 scope, `12 §7.4`.)
- **Auto-derive `Risk` from open `Assumption`s/`Violation`s** — rejected and explicitly out of Band A
  scope (`12 §4`): aggregation is a later deepen. In v0 a risk is *entered and owned*, keeping the
  kernel small and the provenance honest (no synthesized risk with no accountable author).
- **Model risk as advisory metadata like `ModelFidelity`** — rejected: unlike a fidelity tag, a risk
  genuinely **has a lifecycle** (`Open -> Mitigated -> Accepted`), a stable **identity** (it is
  referenced, e.g. as an assumption's discharge target), and a named **owner** who acts on it. Those are
  exactly the properties that make an object first-class (contrast ADR-0019, where the tag has no
  lifecycle and no id). So `Risk` is a full entity through the capability seam, not audit metadata.
- **Tighten `Assumption` discharge to require `AcceptedRisk.target` to resolve to a `Risk`** — deferred
  (`12 §7.4`): increment 1 left `Discharge.target` a free `EntityId` that must merely resolve to *some*
  committed entity. Now that `Risk` exists, that coupling *could* be tightened, but doing so would edit
  a frozen increment-1 seam check; it is left as a separate, later increment rather than folded in here.
