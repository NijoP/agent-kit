# ADR-0025 — `PinCapability` + `PinAssignment`: a pin's datasheet truth, separated from the design's chosen function

**Status:** Accepted (Phase 5 / Band B, increment 4 — "Pin-Function / Mux"). Fourth increment of the
Band B **logical-electrical Maps** layer (`project-plans/02-engineering-world-model.md` §Band B;
Map 22 — Pin-Function / GPIO / Mux). Anchored to `project-plans/00-product-vision.md` (Principle 3 —
the kernel is the moat; Principle 7 — honesty; P9 — first-class physical quantities) and the
master-prompt §31 rule — *keep capability and assignment separate; a conflicting assignment is an
engineering violation, not "just another string."* Delivers Band B **exit criterion 1** (as scoped by
the Band B sibling of `project-plans/12-band-a-implementation-plan.md`): the runtime owns and verifies
a coherent power/clock/return/pin architecture for a real small board.

## Context
The MCU/FPGA pin-planning problem is one of the most painful, error-prone parts of board design and
exactly where "the AI moved a net to a pin that can't carry it" wrecks a week of the schedule. A pin
is not just an electrical endpoint ([`Pin`] carries a `designation` and a coarse `electrical_type`);
a modern IC pin is a **mux**: one physical ball can carry SPI clock *or* a UART TX *or* a GPIO, and
only the datasheet says which. Two things must therefore exist separately:

1. **What a pin *can* do** — the datasheet's mux set. This is imported, factual truth (P7; the model
   never fabricates a pin's abilities).
2. **What the design *assigns* it to do** — an intent decision, one function out of that set.

Conflating them is how pin-planning tools silently accept contradictions: an assignment lands on a
pin that physically cannot carry it (capability violation), or two signals claim the same pin
(mux conflict) and neither is caught because each was "just a string" with no declared set to check
against.

## Decision
Model the pin function architecture as **two first-class, auditable objects** raised through the
capability seam (P2/P3): a `PinCapability` (datasheet truth) and a `PinAssignment` (design truth).
Ship both in the same increment — the Pin-Function/Mux Map is one Map with a co-dependent object
pair (the world-model lists both as its objects): the capability rule verifies an assignment against
its pin's declared functions, and the mux rule needs ≥2 assignments on one pin; neither rule is
meaningful with only one of the objects. This is a **documented exception** to the default
one-object-per-increment discipline, justified by the Map's own object definition.

- **Domain (`eak-domain`).**
  - `struct PinCapability { id, pin: EntityId, functions: Vec<String> }` — the mux functions the pin
    can carry. `validate()` reuses existing `DomainError` variants only: a **non-null** `pin`
    (`Inconsistent`) and a **non-empty** `functions` set (`Inconsistent`) — declaring a capability
    with nothing assignable is inert (mirrors "a power domain must power at least one net").
  - `struct PinAssignment { id, pin: EntityId, function: String }` — the function the design
    assigns. `validate()` reuses existing variants only: a **non-null** `pin` (`Inconsistent`) and a
    **non-empty** `function` (`EmptyField`). `function` is a `String`, deliberately — mux function
    names are datasheet-specific and open-ended; an enum would fabricate a closed world (P7).
  - Both derive `Eq` (no `PhysicalQuantity`).

- **Events (`eak-ports`).** Two new **state-bearing** variants with **explicit fold arms** in
  `EngineeringState::apply` (the one sharp edge; the catch-all would silently diverge replay, P4):
  `PinCapabilityCommitted { capability }` and `PinAssignmentCommitted { assignment }`.

- **State (`eak-runtime`).** Two new stores kept in insertion (event) order:
  `EngineeringState::pin_capabilities` and `EngineeringState::pin_assignments`; accessors
  `pin_capability(id)` / `pin_assignment(id)`.

- **Seam (`eak-runtime`).** `CapabilityRequest::CreatePinCapability { capability, links }` and
  `CapabilityRequest::CreatePinAssignment { assignment, links }` re-validate (non-null pin;
  non-empty functions / non-empty function) **and** enforce referential integrity (P3, P5): the
  `pin` must be a committed pin — a capability or assignment can never attach to a phantom pin.
  **The seam does NOT judge conflicts or capability fit** (see Verification): a well-formed
  assignment names a real pin and a real function, so it must enter state for the rules to *report*
  the conflict (master-prompt §31 — caught as an engineering violation, not silently accepted). A
  rejected proposal commits nothing. Readers `pin_capabilities()` / `pin_assignments()` added to
  `AgentContext` and its impls.

- **Verification (`eak-engines` / `eak-phases`).** `VerificationContext` gains
  `pin_capabilities: &[PinCapability]` and `pin_assignments: &[PinAssignment]` (all construction
  sites updated: every phase machine — ERC, DRC, BOM, DFM, EMC, constraint — the `eak-kicad`
  fixture, and the engines' own test fixtures). Two new rules, both registered on the ERC engine:
  - `PinMuxConflictRule` (`erc-pin-mux-conflict`): a pin claimed for **≥2 different** functions is a
    real hardware clash — one Error finding per conflicting pin, deterministically grouped by pin in
    slice order (P4), subjects = the pin + every assignment claiming it, message lists the clashing
    functions. Two assignments of the *same* function on one pin are a benign re-assertion, silent.
  - `PinCapabilityRule` (`erc-pin-capability`): an assignment whose function is **not** in the pin's
    declared capability is an Error (the pin is asked to do what its silicon cannot); an assignment
    whose pin has **no capability at all** is also an Error (unverifiable — the datasheet truth was
    never imported; surfacing that is honesty, not an invented requirement).

## Consequences
- **The pin-function architecture is committed, traceable objects (exit criterion 1).** Capabilities
  and assignments have stable identities and resolve to committed pins (→ components → blocks →
  intent, P3). The kernel owns the pin architecture; the model can *propose* but never bypass the
  seam.
- **Conflicting assignments are engineering violations, reported.** The mux rule raises a blocking
  Error the moment two functions claim one pin, and the capability rule raises one when an assignment
  exceeds its pin's datasheet. This is the §31 directive made concrete: the conflict is a first-class
  finding (traceable, waivable) rather than a silently-accepted string collision.
- **Capability and assignment stay separate.** The datasheet truth (what a pin can do) is never
  conflated with the design decision (what it is assigned); the system can later reason about pin-mux
  re-planning ("move this function to another pin that also carries it") as a search over the
  capability graph, exactly as the Map's *AI · verify · evolves* column intends.
- **Seam re-validation holds (P3).** Six malformed proposals — a capability with a null pin, an
  empty capability, a capability on a phantom pin, an assignment with a null pin, a blank-function
  assignment, and an assignment on a phantom pin — are each rejected at the seam and nothing enters
  the log. The QA suite asserts all six plus the empty-log-on-rejection property, and the accept-path
  asserts on `pin_capability(id)` / `pin_assignment(id)`.
- **Determinism/replay intact (P4).** Both events are folded by exactly one explicit arm each; the
  fold is the only mutation path. The QA suite asserts byte-identical replay across a re-fold with
  both objects in the log.
- **Verification context is uniform.** Both fields are threaded through every phase machine so the
  engine sees one context shape everywhere; the Band B rules run where the pin layer is populated.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the entities,
  `eak-ports` the events, `eak-runtime` the stores/folds/accessors/seam, `eak-engines` the rules,
  `eak-phases` the registration. No adapter dependency introduced.
- **Schema version.** Additive — two new `Event` variants plus two domain entities; no existing
  carrier was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Limitations (explicit, per the honesty principle)
This v0 owns the **declared** pin-function truth only. It does not yet infer capability from a part
datasheet (a future importer feeds `PinCapability` from real device data), nor does it reason about
*electrical* pin type conflicts (e.g. assigning an output function to a `PowerIn` pin) — those need
the electrical-type matrix, a natural later increment. The rules are deterministic over the declared
sets; if the datasheet truth was never imported, the capability rule honestly reports the assignment
as unverifiable rather than assuming the best.

## Alternatives considered
- **A single `PinFunction { pin, function }` object** — rejected: that is the conflation §31
  forbids. Without a declared capability set there is nothing to check an assignment against, and a
  mux conflict would only be visible by string comparison with no datasheet ground truth.
- **`PinCapability` as a field on [`Pin`] (e.g. `Pin.functions: Vec<String>`) — rejected: the
  capability is a separate, auditable engineering object with its own identity, lifecycle, and rule
  surface; nesting it into `Pin` would collapse the Map into a tag (against the master-prompt §29
  rule) and forfeit the seam's referential integrity for the pin.
- **An enum for `function`** — rejected: mux names are datasheet-specific and open-ended; an enum
  would fabricate a closed world (P7). The capability rule validates the string against the declared
  set, which is the real ground truth.
- **Reject a conflicting assignment at the seam** — rejected: the *shape* of an assignment (pin +
  function) is a fact the runtime records; whether it *conflicts* is a **judgement** the verification
  layer owns, where violations are first-class, explainable, and can be waived. Rejecting at the seam
  would conflate "malformed commit" with "this pin-out is incomplete" (mirrors ADR-0022/0023/0024).
- **Warn (not Error) on a mux conflict / capability violation** — rejected: claiming one physical pin
  for two functions or asking a pin to do what its silicon cannot are the exact silent failures the
  Map exists to prevent. They block release, matching the Phase-3 ERC severity discipline.
- **One increment per object** — rejected (documented exception): the two rules are meaningless with
  only one object present, and the world-model defines the Map's objects as the pair
  `PinCapability`/`PinAssignment`. Shipping both is the unit of *verifiable* value; splitting would
  leave a half-Map with no rule surface in between commits.