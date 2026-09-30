# ADR-0023 — `ClockDomain`: the clock architecture as a verifiable, traceable clock region

**Status:** Accepted (Phase 5 / Band B, increment 2 — "ClockDomain"). Second increment of the Band B
**logical-electrical Maps** layer of `project-plans/11-build-roadmap.md` (Phase 5, Band B) and
`project-plans/02-engineering-world-model.md` (§Band B — logical-electrical maps: power, clock, pin,
signal, interface). Anchored to `project-plans/00-product-vision.md` (Principle 3 — the kernel is the
moat; Principle 7 — honesty; P9 — first-class physical quantities) and the clock/return-path groundings
in `engineering-science/pcb/return-path.md` (L138/L141: the return-path continuity rule targets
*controlled / electrically-long* nets, a set the runtime can only identify once it owns clock
frequencies). Delivers Band B **exit criterion 1** (as scoped by
`project-plans/12-band-a-implementation-plan.md`'s sibling Band B plan): *the runtime owns a committed
power/clock architecture whose nets are classified and verified by construction.*

## Context
Band B increment 1 (`PowerDomain`, ADR-0022) gave the runtime the power architecture: named rails,
their supplying components, and a KCL budget checked at ERC time. What the runtime still could not
express is the **clock architecture**: the named clock regions that make a board actually *operate*.
A board without a first-class `ClockDomain` cannot express which nets are synchronous to which
frequency — so the engineering-science rules that depend on that fact (return-path continuity for
electrically-long nets, CDC reasoning at clock-domain crossings, EMI aggressor classification) have
no object to apply to. Just as KCL needed a rail to sum currents over, return-path reasoning needs a
clock region to classify nets over.

A clock region is a **named, traceable engineering object**: a sourcing component (oscillator /
crystal / PLL / clock generator — the traceability anchor back to intent, P3), a frequency (P9, typed
Hertz), and the set of nets synchronous to that clock. A net that is a member of two distinct clock
domains is a **clock-domain crossing** — an unsynchronized net claiming to be synchronous to two
independent clocks. That is a design *finding* (a CDC hazard) for the rule engine to report, not a
malformed domain for the seam to reject.

## Decision
Model the clock architecture as **a first-class, auditable `ClockDomain` entity** raised through the
capability seam (P2/P3), carrying its frequency as a first-class physical quantity, and give the rule
engine a **clock-domain membership** check over it. A well-formed domain whose membership crosses
another domain's is still **accepted into state** — the rule engine, not the seam, is the authority on
design judgement (a crossing is a *violation to report*, not an *illegal commit*). This mirrors the
overload judgement of ADR-0022 exactly.

- **Domain (`eak-domain`).**
  - `struct ClockDomain { id: EntityId, name: String, frequency: PhysicalQuantity,
    source_component: EntityId, members: Vec<EntityId> }` — a clock region at `frequency`, sourced by
    `source_component`, driving the member `members` (the synchronous nets). `validate()` reuses
    existing `DomainError` variants only: a non-empty `name`
    (`DomainError::EmptyField("clock domain name")`), a **finite, positive** `frequency`
    (`DomainError::Inconsistent`, compared via `si_magnitude()` so the check is unit-independent —
    `Megahertz` and `Kilohertz` both work, P9), and **≥ 1 member net** (a clock that drives nothing is
    a silent defect, mirroring "a rail powering nothing").
  - **A crossing is not a validation error.** `validate()` enforces only *well-formedness*. A domain
    whose membership collides with another domain is well-formed but *unsafe* — a design-flaw to be
    reported by the rule engine, never an illegal commit. The seam must accept it so the rule can
    reason over the crossing.
  - `ClockDomain` carries an `f64` (`frequency` is a `PhysicalQuantity`), so it derives `PartialEq`
    but **not** `Eq` (mirrors `Decision`/`Component`/`Objective`/`Tradeoff`/`PowerDomain`). No new
    `DomainError` variant is invented.

- **Events (`eak-ports`).** One new **state-bearing** variant with an **explicit fold arm** in
  `EngineeringState::apply` (the one sharp edge; the `_ => {}` catch-all would otherwise silently
  diverge replay, P4): `ClockDomainCommitted { domain: ClockDomain }`.

- **State (`eak-runtime`).** A new store `EngineeringState::clock_domains: Vec<ClockDomain>` kept in
  insertion (event) order; accessor `clock_domain(id)`.

- **Seam (`eak-runtime`).** `CapabilityRequest::CreateClockDomain { domain, links }` re-validates the
  domain (non-empty name, positive finite frequency, ≥ 1 member net) **and** enforces the
  referential-integrity invariants the rule engine cannot infer (P3, P5): a **null** source component
  is rejected, an **unknown** source component is rejected, and **every** member net must be a
  committed net — a domain can never reference a phantom clock. A rejected proposal commits
  **nothing** to the log. Readers `clock_domains()` / `clock_domain(id)` added to `AgentContext` and
  its impls.

- **Verification (`eak-engines` / `eak-phases`).** `VerificationContext` gains
  `clock_domains: &[ClockDomain]` (all construction sites updated: every phase machine — ERC, DRC,
  BOM, DFM, EMC, constraint — plus the `eak-kicad` fixture and the engines' own test fixtures). New
  rule `ClockDomainMembershipRule` (`erc-clock-domain-conflict`), registered on the ERC engine: for
  each net, collect the set of domains that list it as a member; a net that is a member of **two or
  more** distinct domains is a **blocking `Error`** finding naming the net and both domains (a
  clock-domain crossing needing a synchronizer or a membership correction). Deterministic by
  construction: domains in slice order, nets in domain order, one finding per crossing net (P4). A
  member net missing from the context contributes nothing — the per-net "no input → silent"
  discipline (integrity is the commit seam's job, P3).

## Consequences
- **The clock architecture is a committed, traceable object (exit criterion 1).** Every clock region
  has a stable identity, a named sourcing component that resolves to a committed component (→ block →
  requirement → intent, P3), and a typed frequency (P9). The kernel owns the architecture; the model
  can *propose* regions but never bypass the seam.
- **The return-path continuity rule now has its prerequisite.** Per `engineering-science/pcb/return-path.md`
  (L138/L141), that rule targets controlled / electrically-long nets — a set the runtime identifies
  via clock frequency. This increment is the foundation the ReturnPath increment (next in Band B)
  builds on. That is why ClockDomain precedes ReturnPath in the increment order (a documented
  adaptation of the world-model Map order, §Band B).
- **CDC reasoning has a seed.** `erc-clock-domain-conflict` flags a net claimed by two domains as a
  blocking error before the board is routed. The QA suite asserts: a net in two domains is flagged
  (Error, naming both domains and "crossing"), a net in one domain passes, a net shared by three
  domains yields exactly one finding, an empty architecture yields nothing, and a member net missing
  from the context stays silent.
- **Seam re-validation holds (P3).** Six malformed proposals — a blank domain name, a non-positive
  frequency, a null source, a dangling source, a dangling member net, and an empty member list — are
  each rejected at the seam and **nothing** enters the log. The QA suite asserts all six plus the
  empty-log-on-rejection property, and the accept-path asserts on `clock_domain(id)` and
  `frequency` round-tripping.
- **Determinism/replay intact (P4).** `ClockDomainCommitted` is folded by exactly one explicit arm;
  the fold is the only mutation path. The QA suite asserts **byte-identical** replay
  (`canonical_json` equality across a re-fold) with the clock domain in the log.
- **Verification context is uniform.** The `clock_domains` field is threaded through every phase
  machine so the engine sees one context shape everywhere; the Band B rule simply runs where the
  clock layer is populated.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the entity,
  `eak-ports` the event, `eak-runtime` the store/fold/accessor/seam, `eak-engines` the rule,
  `eak-phases` the registration. No adapter dependency introduced.
- **Schema version.** Additive — one new `Event` variant plus a new domain entity; no existing carrier
  was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Alternatives considered
- **Reject a crossing at the seam** — rejected: a clock region's *shape* (name, frequency, source,
  members) is a fact the runtime records; whether its membership collides with another domain is a
  **judgement** the verification layer owns, where violations are first-class, explainable, and can be
  waived. Rejecting at the seam would conflate "this is a malformed commit" with "this is a design
  that will not work", and would make an honest-but-crossing architecture unrepresentable (mirrors
  the ADR-0022 overload decision).
- **Infer a `ClockDomain` from per-net attributes** (e.g. every net's frequency tag) — rejected: a
  clock region is a deliberate, owned object with a named source and an explicit frequency; inferring
  it would fabricate a source component and a clock rate the design never declared (P3, P7 honesty).
  The domain is entered and owned, like every Band A/B object.
- **Fold the frequency into a `Net` field** (e.g. `Net.clock_rate`) — rejected: clock membership is a
  region-level fact (a whole *set* of nets is synchronous to one source), not a per-net attribute, and
  the source component must be named for traceability (P3). `ClockDomain` is the correct unit; a
  per-net tag would scatter the region and lose the sourcing anchor.
- **Warn (not Error) on crossing** — rejected: an unsynchronized net claiming two clocks is a physical
  hazard, not a style preference. It blocks release, matching the severity discipline of the Phase-3
  ERC rules and `erc-power-balance`.
- **Skip `ClockDomain` and implement the return-path rule directly on `Net` frequency tags** —
  rejected: `engineering-science/pcb/return-path.md` itself requires the runtime to identify
  controlled/electrically-long nets *by clock domain*; a direct-to-net rule would fabricate the region
  the science depends on. The dependency is foundational, so it ships first.