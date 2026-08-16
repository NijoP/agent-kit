# ADR-0024 — `ReturnPath`: the return half of the signal loop, named and verified

**Status:** Accepted (Phase 5 / Band B, increment 3 — "ReturnPath"). Third increment of the Band B
**logical-electrical Maps** layer of `project-plans/11-build-roadmap.md` (Phase 5, Band B) and
`project-plans/02-engineering-world-model.md` (§Band B — logical-electrical maps: power, clock,
ground/return, pin, signal, interface). Anchored to `project-plans/00-product-vision.md`
(Principle 3 — the kernel is the moat; Principle 7 — honesty; P9 — first-class physical quantities)
and the return-path groundings in `engineering-science/pcb/return-path.md` (L138–L141: the
return-path continuity rule targets *controlled / electrically-long* nets; the runtime must own
which plane each signal references to diagnose a break). Delivers Band B **exit criterion 1** (as
scoped by `project-plans/12-band-a-implementation-plan.md`'s sibling Band B plan): *the runtime owns
and verifies a coherent power/clock/return architecture for a real small board.*

## Context
Band B increments 1–2 (`PowerDomain` ADR-0022, `ClockDomain` ADR-0023) gave the runtime the power
architecture (named rails + a KCL budget) and the clock architecture (named clock regions + typed
frequencies). What the runtime still could not express is the **return architecture**: the return
half of the signal loop. Every signal current must return to its driver; at the speeds digital edges
actually contain it returns on the conductor directly beneath the trace — almost always a reference
plane (`return-path.md`). The runtime routes a [`Net`] as if it were a single forward conductor; the
schematic names the signal but never names its return. An over-long controlled net with a broken or
un-named return is the *silent* failure: connectivity-complete, width-correct, and wrong on the bench
(return-path.md L140 — "width-correct, impedance-wrong").

A return path is a **named, traceable engineering relationship**: the controlled net whose return
current it governs, and the reference net (plane) the current returns on. Naming it closes the loop
that connectivity checks structurally cannot see.

### The honest v0 scope (an explicit adaptation of the ClockDomain ADR)
ADR-0023 implied that ClockDomain's frequency lets the runtime *identify* controlled /
electrically-long nets. The engineering science does not support that shortcut:
`engineering-science/electrical/transmission-lines.md` (L145, L170) is explicit that the
electrically-long boundary must be applied against the **edge rate**, not the clock rate — "using the
clock instead of the edge to gauge 'fast'" is a listed failure mode (a slow clock with fast edges is
still a transmission line). The model does not own rise/edge times, and fabricating a frequency
threshold would be exactly that failure. The model DOES own one honest transmission-line declaration:
`Net::impedance_target` — a typed characteristic-impedance target is a net's own controlled-impedance
declaration (transmission-lines.md L141). **ReturnPath therefore gates on `impedance_target`, not on
a fabricated clock threshold.** This is the smallest justified adaptation per the master-prompt
§27/§30 rule: encode only what the current model can truthfully verify.

## Decision
Model the return architecture as **a first-class, auditable `ReturnPath` entity** raised through the
capability seam (P2/P3): the logical-electrical contract naming the reference net a controlled net's
return current flows on. Give the rule engine a **return-path requirement** check over it: a net that
declares a controlled impedance but has no declared return path is a design finding (its return half
is under-specified before any copper exists). A well-formed path is still **accepted into state** —
the rule engine, not the seam, is the authority on design judgement (an omission is a *violation to
report*, not an *illegal commit*). This mirrors the overload/crossing judgements of ADR-0022/0023.

- **Domain (`eak-domain`).**
  - `struct ReturnPath { id: EntityId, name: String, net: EntityId, reference_plane: EntityId }` — a
    controlled [`Net`]'s return current flows on the `reference_plane` [`Net`]. `validate()` reuses
    existing `DomainError` variants only: a non-empty `name`
    (`DomainError::EmptyField("return path name")`), a **non-null** `net`, a **non-null**
    `reference_plane`, and `net != reference_plane` — a signal cannot be its own return conductor
    (KCL requires a distinct return path; `DomainError::Inconsistent`).
  - `ReturnPath` carries no `PhysicalQuantity` (only ids + a name), so — like `Pin`/`FunctionalBlock`
    — it derives `Eq`. No new `DomainError` variant is invented.

- **Events (`eak-ports`).** One new **state-bearing** variant with an **explicit fold arm** in
  `EngineeringState::apply` (the one sharp edge; the catch-all would silently diverge replay, P4):
  `ReturnPathCommitted { path: ReturnPath }`.

- **State (`eak-runtime`).** A new store `EngineeringState::return_paths: Vec<ReturnPath>` kept in
  insertion (event) order; accessor `return_path(id)`.

- **Seam (`eak-runtime`).** `CapabilityRequest::CreateReturnPath { path, links }` re-validates the
  path (non-empty name, non-null net, non-null reference plane, net != reference plane) **and**
  enforces the referential-integrity invariants the rule engine cannot infer (P3, P5): `net` must be
  a committed net and `reference_plane` must be a committed net — a path can never reference a
  phantom conductor or a phantom plane. A rejected proposal commits **nothing** to the log. Readers
  `return_paths()` / `return_path(id)` added to `AgentContext` and its impls.

- **Verification (`eak-engines` / `eak-phases`).** `VerificationContext` gains
  `return_paths: &[ReturnPath]` (all construction sites updated: every phase machine — ERC, DRC, BOM,
  DFM, EMC, constraint — plus the `eak-kicad` fixture and the engines' own test fixtures). New rule
  `ReturnPathRule` (`erc-return-path-required`), registered on the ERC engine: for each net that
  declares `impedance_target`, if no committed return path names it, that net is a **blocking `Error`**
  finding ("declares an impedance target but has no declared return path — the return half of the
  signal loop is under-specified"). A net with no impedance declaration is **silent** (ordinary
  routing; no invented requirement — surfacing under-specification is the `ModelFidelity` tag's
  concern, exactly like the power-balance discipline). Deterministic by construction: nets in slice
  order (P4).

## Consequences
- **The return architecture is a committed, traceable object (exit criterion 1).** Every controlled
  net's return relationship has a stable identity and resolves to committed nets (→ pins → components
  → blocks → intent, P3). The kernel owns the return architecture; the model can *propose* paths but
  never bypass the seam.
- **The silent SI failure is caught at construction.** `erc-return-path-required` flags a
  controlled net with no declared return as a blocking error before routing — the "width-correct,
  impedance-wrong" net is under-specified by the time copper exists. The QA suite asserts: a
  controlled net with no return is flagged (Error, naming the net), a controlled net with a declared
  return passes, an uncontrolled net is silent, an uncontrolled net with an over-specified path stays
  silent, and an empty architecture yields nothing.
- **Honesty: no fabricated threshold.** The rule keys on the design's own `impedance_target`
  declaration, not a made-up clock-frequency cutoff — honoring the science's "edge rate, not clock
  rate" directive (transmission-lines.md L145/L170) and the no-fabrication principle. This is a
  documented correction to ADR-0023's implied frequency-based classification.
- **Seam re-validation holds (P3).** Six malformed proposals — a blank path name, a null net, a null
  reference plane, a self-return (net == plane), a dangling net, and a dangling reference plane —
  are each rejected at the seam and **nothing** enters the log. The QA suite asserts all six plus the
  empty-log-on-rejection property, and the accept-path asserts on `return_path(id)`.
- **Determinism/replay intact (P4).** `ReturnPathCommitted` is folded by exactly one explicit arm;
  the fold is the only mutation path. The QA suite asserts **byte-identical** replay
  (`canonical_json` equality across a re-fold) with the return path in the log.
- **Verification context is uniform.** The `return_paths` field is threaded through every phase
  machine so the engine sees one context shape everywhere; the Band B rule simply runs where the
  return layer is populated.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the entity,
  `eak-ports` the event, `eak-runtime` the store/fold/accessor/seam, `eak-engines` the rule,
  `eak-phases` the registration. No adapter dependency introduced.
- **Schema version.** Additive — one new `Event` variant plus a new domain entity; no existing
  carrier was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Limitations (explicit, per the honesty principle)
This v0 owns the **logical-electrical contract** only. The full reference-continuity *geometry* check
— a plane split/void under a trace, board-edge plane truncation, stitching-via placement — needs the
PCB-IR reference-adjacency model (which layer references which plane; plane voids/splits; layer
heights; `ε_r`), which the runtime does not yet own (return-path.md L141). `ReturnPath` names *which*
net is the reference; it does not yet verify *that the copper is continuous under the trace*. That
geometry rule is a natural later Band-B/DRC increment, and `ReturnPath` is its prerequisite.

## Alternatives considered
- **Reject a controlled net with no return path at the seam** — rejected: the *shape* of a return
  path (name, net, reference plane) is a fact the runtime records; whether the controlled net is
  under-specified is a **judgement** the verification layer owns, where violations are first-class,
  explainable, and can be waived. Rejecting at the seam would conflate "malformed commit" with "this
  architecture is incomplete", and would make an honest-but-omitted return unrepresentable (mirrors
  the ADR-0022/0023 decisions).
- **Derive the return path automatically from net class (e.g. "every Signal net returns on Ground")**
  — rejected: that fabricates a reference-plane choice the design never declared (P3, P7 honesty). A
  real board has many planes (analog/digital GND, power islands); only the design can name the true
  return. The path is entered and owned, like every Band A/B object.
- **Gate the rule on a clock-frequency threshold** — rejected: the science (transmission-lines.md
  L145/L170) directs the electrically-long boundary be applied against the **edge rate**, which the
  model does not own; inventing a frequency cutoff would under-classify fast-edge slow-clock nets
  exactly as the science warns. The design's own `impedance_target` is the only honest trigger the
  current model can verify.
- **Warn (not Error) on a missing return** — rejected: an un-named return on a controlled net is the
  silent SI/EMC failure the return-path science exists to prevent. It blocks release, matching the
  severity discipline of the Phase-3 ERC rules and `erc-power-balance`.
- **Fold `reference_plane` into the `Net` type (e.g. `Net.reference`) — rejected: the return
  relationship is a distinct, auditable engineering object with its own identity, lifecycle, and
  rule surface; nesting it into `Net` would collapse the Map into a tag (against the master-prompt
  §29 "do not collapse concepts into strings/tags" rule) and forfeit the seam's referential
  integrity for the plane.