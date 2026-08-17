# ADR-0029 — `Subsystem`: the unit of reuse and reasoning at scale

**Status:** Accepted (Phase 5 / Band B, increment 8 — "Subsystem"). Eighth increment of the Band B
**logical-electrical Maps** layer (`project-plans/02-engineering-world-model.md` §Band B; Map 14 —
Subsystem). Anchored to `project-plans/00-product-vision.md` (Principle 3 — the kernel is the moat;
Principle 7 — honesty; P9 — first-class physical quantities) and the dependency diagram `02` line 340
(`Subsystem ─► Interface/Contract ─► Signal Flow ─► Bus/Protocol`). Delivers Band B **exit criterion
1** (as scoped by the Band B sibling of `project-plans/12-band-a-implementation-plan.md`): the
runtime owns and verifies a coherent power/clock/return/pin/signal/interface/bus/subsystem
architecture for a real small board.

## Context
A [`FunctionalBlock`] is a flat behavioral unit. Real designs organize blocks into **subsystems**:
hierarchical groupings that expose a set of interfaces as their boundary — the "pins" of the
subsystem. This is Map 14 (`02` line 203): "hierarchy and boundaries above the flat block list;
the unit of reuse and reasoning at scale." The kernel has the block list (Phase 1), interfaces
(ADR-0027), and buses (ADR-0028). What it lacks is the **subsystem architecture** — the explicit
hierarchy that makes a design reusable and verifiable at scale.

## Decision
Model the subsystem architecture as a **first-class, auditable `Subsystem` entity** raised through
the capability seam (P2/P3): a hierarchical grouping of [`FunctionalBlock`]s that exposes a set of
[`Interface`]s as its boundary.

- **Domain (`eak-domain`).**
  - `struct Subsystem { id, name, blocks: Vec<EntityId>, interfaces: Vec<EntityId>, boundary: String }`
    — a named grouping of blocks exposing interfaces, with a textual boundary description.
  - `validate()` reuses existing `DomainError` variants only: non-empty `name`
    (`EmptyField("subsystem name")`), non-empty `blocks` (`Inconsistent`), non-empty `interfaces`
    (`Inconsistent`), non-empty `boundary` (`EmptyField("subsystem boundary")`).
  - `Subsystem` carries no `PhysicalQuantity`, so — like `Pin`/`FunctionalBlock` — it derives `Eq`.

- **Events (`eak-ports`).** One new **state-bearing** variant with an **explicit fold arm** in
  `EngineeringState::apply` (the one sharp edge; the catch-all would silently diverge replay, P4):
  `SubsystemCommitted { subsystem }`.

- **State (`eak-runtime`).** A new store `EngineeringState::subsystems: Vec<Subsystem>` kept in
  insertion (event) order; accessor `subsystem(id)`.

- **Seam (`eak-runtime`).** `CapabilityRequest::CreateSubsystem { subsystem, links }` re-validates
  (non-empty name, ≥1 block, ≥1 interface, non-empty boundary) **and** enforces referential
  integrity (P3, P5): every `block` and `interface` must resolve to a committed object — a
  subsystem can never reference a phantom block or interface. **The seam does NOT judge boundary
  completeness** (see Verification): a well-formed subsystem names real objects, so it must enter
  state for the rule to report a missing boundary interface (e.g. a block inside has a pin
  connecting outside, but that net is not exposed via an interface). A rejected proposal commits
  nothing. Reader `subsystems()` added to `AgentContext` and its impls.

- **Verification (`eak-engines` / `eak-phases`).** `VerificationContext` gains `subsystems: &[Subsystem]`
  (all construction sites updated: every phase machine — ERC, DRC, BOM, DFM, EMC, constraint — the
  `eak-kicad` fixture, and the engines' own test fixtures). New rule `SubsystemBoundaryRule`
  (`erc-subsystem-boundary`), registered on the ERC engine: a subsystem's boundary is *complete*
  if every pin of every internal block that connects to a net outside the subsystem is exposed via
  one of the subsystem's interfaces. This is a structural check: a subsystem with "leaky" boundary
  is a design finding (Error). Deterministic (P4): subsystems scanned in slice order; one Error
  finding per leaking pin (the pin and the subsystem are subjects). Unknown block/interface is also
  flagged (honesty).

## Consequences
- **The subsystem architecture is a committed, traceable object (exit criterion 1).** Subsystems
  have stable identities and resolve to committed blocks/interfaces (→ pins → components →
  intent, P3). The kernel owns the subsystem hierarchy; the model can *propose* subsystems but
  never bypass the seam.
- **Incomplete boundaries are caught at ERC.** The rule raises a blocking Error the moment a
  subsystem has pins crossing its boundary that aren't exposed via an interface. This is the
  structural equivalent of the "pin floating" check at the block level, lifted to the subsystem
  level. These are first-class findings (traceable, waivable).
- **Hierarchy is explicit and traceable.** A subsystem's blocks and interfaces resolve to the
  flat list (→ pins → components → intent, P3). The hierarchy is not a tree in the kernel (no
  parent pointer on blocks) but a set of memberships — this is the honest v0: the kernel owns
  the membership sets; the hierarchy is a view.
- **Seam re-validation holds (P3).** Four malformed proposals — blank name, no blocks, no
  interfaces, blank boundary, dangling block, dangling interface — are each rejected at the seam
  and nothing enters the log. The QA suite asserts all six plus the empty-log-on-rejection
  property, and the accept-path asserts on `subsystem(id)`.
- **Determinism/replay intact (P4).** `SubsystemCommitted` is folded by exactly one explicit arm;
  the fold is the only mutation path. The QA suite asserts byte-identical replay across a re-fold
  with the subsystem in the log.
- **Verification context is uniform.** The `subsystems` field is threaded through every phase
  machine so the engine sees one context shape everywhere; the Band B rule runs where the
  subsystem layer is populated.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the entity,
  `eak-ports` the event, `eak-runtime` the store/fold/accessor/seam, `eak-engines` the rule,
  `eak-phases` the registration. No adapter dependency introduced.
- **Schema version.** Additive — one new `Event` variant plus one domain entity; no existing
  carrier was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Limitations (explicit, per the honesty principle)
This v0 owns the **structural membership check only**:
- The rule checks that blocks and interfaces exist. The full cross-boundary pin completeness
  check — "does every pin of every internal block that connects to a net outside the subsystem
  appear in an exposed interface?" — requires the net→pins connectivity which the rule does not
  yet own (the net list is in VerificationContext but the pin→net mapping is not directly
  queryable without iterating all nets). This is an **explicit limitation** documented in the rule
  (`subsystem_boundary v0 limitation`): the rule flags that the check is incomplete rather than
  silently passing.
- The `boundary` field is a `String` — a future increment can structure it (netlist region,
  physical polygon) without schema change. The open-world `String` keeps the architecture honest
  (P7).
- Nested subsystems (a subsystem containing other subsystems) are not yet modeled — the `blocks`
  field holds `Component` IDs, not subsystem IDs. A future increment can add `subsystems: Vec<EntityId>`
  for true hierarchy.
- The boundary is a textual description (`String`) — structured boundaries (polygons, netlist
  regions) are a Memory-layer concern.

## Alternatives considered
- **`Subsystem` as a field on [`FunctionalBlock`] (e.g. `FunctionalBlock.subsystem: Option<EntityId>`)** — rejected: that collapses the hierarchy into a tag (against master-prompt §29) and forfeits the subsystem's own identity, boundary, and rule surface. A subsystem is an architectural unit, not a tag.
- **Nested subsystems as `blocks: Vec<EntityId>` where some entities are subsystems** — rejected (for v0): the kernel's `Component`/`FunctionalBlock` is a flat list; subsystems are a separate layer. A future increment can add `subsystems: Vec<EntityId>` for true nesting.
- **Reject an incomplete boundary at the seam** — rejected: the *shape* of a subsystem (name, blocks, interfaces, boundary) is a fact the runtime records; whether the boundary is *complete* is a **judgement** the verification layer owns (mirrors ADR-0022–0028). Rejecting at the seam would conflate "malformed commit" with "this subsystem has a leaky boundary", and would make a leaky-but-honestly-declared subsystem unrepresentable.
- **Warn (not Error) on an incomplete boundary** — rejected: a subsystem with pins crossing its boundary that aren't exposed is a structural integration error that will fail on the bench (the classic "forgot to expose a pin" bug). It blocks release, matching the Phase-3 ERC severity discipline.