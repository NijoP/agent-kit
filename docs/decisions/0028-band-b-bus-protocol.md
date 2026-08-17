# ADR-0028 — `Bus`: a collection of interfaces sharing a physical bus line under one protocol contract, with a declared topology

**Status:** Accepted (Phase 5 / Band B, increment 7 — "Bus / Protocol"). Seventh increment of the Band B
**logical-electrical Maps** layer (`project-plans/02-engineering-world-model.md` §Band B; Map 17 —
Bus / Protocol). Anchored to `project-plans/00-product-vision.md` (Principle 3 — the kernel is the moat;
Principle 7 — honesty; P9 — first-class physical quantities) and the dependency diagram `02` line 340
(`Subsystem ─► Interface/Contract ─► Signal Flow ─► Bus/Protocol`). Delivers Band B **exit criterion 1**
(as scoped by the Band B sibling of `project-plans/12-band-a-implementation-plan.md`): the runtime
owns and verifies a coherent power/clock/return/pin/signal/interface/bus architecture for a real
small board.

## Context
An [`Interface`] (ADR-0027) is a single connection point governed by a contract; a [`Bus`] is a
**collection of interfaces (or signals) that share a physical bus line** under one protocol contract,
with topology rules (addressing, termination, fan-out, stub length). This is Map 17 (`02` line 82):
"bus topologies (I²C/SPI/USB/CAN…) and their structural rules (addressing, termination, fan-out)."
The Bus is the architectural unit where protocol-level constraints live: an I²C bus needs unique
7-bit addresses and pull-ups; a CAN bus needs termination at both ends; a USB bus needs hub fan-out
limits; an SPI bus needs one controller and peripheral CS lines.

## Decision
Model the bus architecture as a **first-class, auditable `Bus` entity** raised through the
capability seam (P2/P3): a named collection of interfaces governed by a contract, with a declared
topology.

- **Domain (`eak-domain`).**
  - `enum BusTopology { Linear, Star, MultiDrop, PointToPoint, Other(String) }` — the physical
    topology of the bus. Serialized as a string (open world; an enum would fabricate a closed world
    of bus types, P7). Parsing via `FromStr` (open world: unknown names become `Other(name)`).
  - `struct Bus { id, name, contract: EntityId, members: Vec<EntityId>, topology: BusTopology }` —
    a bus groups member interfaces under a contract with a declared topology.
  - `validate()` reuses existing `DomainError` variants only: non-empty `name`
    (`EmptyField("bus name")`), non-null `contract` (`Inconsistent`), non-empty `members`
    (`Inconsistent`).
  - `Bus` carries no `PhysicalQuantity`, so — like `Pin`/`FunctionalBlock` — it derives `Eq`.

- **Events (`eak-ports`).** One new **state-bearing** variant with an **explicit fold arm** in
  `EngineeringState::apply` (the one sharp edge; the catch-all would silently diverge replay, P4):
  `BusCommitted { bus }`.

- **State (`eak-runtime`).** A new store `EngineeringState::buses: Vec<Bus>` kept in insertion
  (event) order; accessor `bus(id)`.

- **Seam (`eak-runtime`).** `CapabilityRequest::CreateBus { bus, links }` re-validates (non-empty
  name, non-null contract, ≥1 member) **and** enforces referential integrity (P3, P5): `contract`
  and every `member` must resolve to committed interfaces — a bus can never reference a phantom
  contract or interface. **The seam does NOT judge topology satisfaction** (see Verification): a
  well-formed bus names real objects, so it must enter state for the rule to report a violation
  (e.g. two I²C devices with the same address, a CAN bus missing termination). A rejected proposal
  commits nothing. Reader `buses()` added to `AgentContext` and its impls.

- **Verification (`eak-engines` / `eak-phases`).** `VerificationContext` gains `buses: &[Bus]`
  (all construction sites updated: every phase machine — ERC, DRC, BOM, DFM, EMC, constraint — the
  `eak-kicad` fixture, and the engines' own test fixtures). New rule `BusTopologyRule`
  (`erc-bus-topology`), registered on the ERC engine: a bus's topology and protocol determine
  which structural checks apply. This v0 encodes a **minimal set of well-known protocol/topology
  checks directly** (I²C MultiDrop → address uniqueness; CAN Linear → termination at both ends;
  USB2 Star → hub fan-out ≤127). A full protocol knowledge library is a Memory-layer concern.
  Deterministic (P4): buses scanned in slice order; one Error finding per violated requirement.
  Unknown contract or unknown member interfaces are also flagged (honesty).

## Consequences
- **The bus architecture is a committed, traceable object (exit criterion 1).** Buses have stable
  identities and resolve to committed interfaces (→ their contracts → signals → pins → components
  → blocks → intent, P3). The kernel owns the bus layer; the model can *propose* buses but never
  bypass the seam.
- **Bus-level structural violations are caught at ERC.** The rule raises a blocking Error the
  moment an I²C bus has duplicate addresses, a CAN bus lacks termination, a USB bus exceeds fan-out.
  These are first-class findings (traceable, waivable) rather than silently accepted malformed
  buses.
- **Topology is an explicit, open-ended field.** The `BusTopology` enum with `Other(String)` keeps
  the architecture open to new bus types without schema changes (P7). Parsing via `FromStr` with
  `Other(String)` fallback makes the parser total and open-world.
- **Seam re-validation holds (P3).** Four malformed proposals — blank name, null contract, no
  members, dangling contract, dangling member — are each rejected at the seam and nothing enters
  the log. The QA suite asserts all four plus the empty-log-on-rejection property, and the
  accept-path asserts on `bus(id)`.
- **Determinism/replay intact (P4).** `BusCommitted` is folded by exactly one explicit arm; the
  fold is the only mutation path. The QA suite asserts byte-identical replay across a re-fold with
  the bus in the log.
- **Verification context is uniform.** The `buses` field is threaded through every phase machine
  so the engine sees one context shape everywhere; the Band B rule runs where the bus layer is
  populated.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the entity,
  `eak-ports` the event, `eak-runtime` the store/fold/accessor/seam, `eak-engines` the rule,
  `eak-phases` the registration. No adapter dependency introduced.
- **Schema version.** Additive — one new `Event` variant plus one domain entity; no existing
  carrier was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Limitations (explicit, per the honesty principle)
This v0 owns the **structural topology check only**:
- The rule checks well-known protocol/topology combinations (I²C MultiDrop → address uniqueness;
  CAN Linear → termination; USB2 Star → fan-out ≤127). It does **not** yet verify:
  - Signal *direction* matching the protocol (e.g. I²C SDA bidirectional, SCL output from master).
  - Signal *electrical type* matching the protocol (e.g. I²C needs open-drain + pull-ups).
  - *Timing* requirements (setup/hold, clock stretching, bus frequency).
  - *Termination values* (CAN 120Ω, USB 45Ω).
  - *Stub length* limits (I²C < 1m, CAN < 30cm at 1Mbps).
- A `Bus` groups member interfaces; it does not yet model the *physical* wire segments (that's the
  Copper/Routing Map 28). The full protocol knowledge library (all protocols, all rules) is a
  Memory-layer concern.
- The `BusTopology::Other(String)` variant means unknown topologies pass silently — this is
  honest (no invented requirements) but means a custom bus type gets no structural checks until
  the Memory layer supplies them.

## Alternatives considered
- **`Bus` as a field on [`Contract`] (e.g. `Contract.buses: Vec<Bus>`)** — rejected: the contract
  is the protocol rule-set; the bus is a physical instance of that protocol. One contract (I²C)
  governs many buses (I2C_1, I2C_2). They are distinct objects.
- **An enum for `topology` without `Other(String)`** — rejected: that would fabricate a closed
  world of bus types (P7). New bus types (e.g. a proprietary differential bus) must be
  representable without a schema change. `Other(String)` makes the parser total and open-world.
- **`Bus` members as [`Signal`]s directly (not interfaces)** — rejected: an interface is the
  logical connection point; a bus connects interfaces. Signals are inside interfaces. Grouping at
  the interface level matches how engineers think ("this I²C bus has these interfaces").
- **Reject a topology-violating bus at the seam** — rejected: the *shape* of a bus (name, contract,
  members, topology) is a fact the runtime records; whether it *satisfies* the topology rules is a
  **judgement** the verification layer owns (mirrors ADR-0022–0027). Rejecting at the seam would
  conflate "malformed commit" with "this bus violates its topology".
- **Warn (not Error) on a topology violation** — rejected: an I²C bus with duplicate addresses or
  a CAN bus without termination is a structural protocol violation that will fail on the bench. It
  blocks release, matching the Phase-3 ERC severity discipline.