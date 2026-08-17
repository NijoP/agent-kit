# ADR-0027 — `Contract` + `Interface`: a protocol rule-set and its governed signal collection

**Status:** Accepted (Phase 5 / Band B, increment 6 — "Interface / Contract"). Sixth increment of the
Band B **logical-electrical Maps** layer (`project-plans/02-engineering-world-model.md` §Band B;
sits between Signal Flow and Bus/Protocol; `02` line 340: `Subsystem ─► Interface/Contract ─►
Signal Flow ─► Bus/Protocol`). Anchored to `project-plans/00-product-vision.md` (Principle 3 — the
kernel is the moat; Principle 7 — honesty; P9 — first-class physical quantities) and the master-
prompt §40 increment list: `Interface` as a Band B object. Delivers Band B **exit criterion 1** (as
scoped by the Band B sibling of `project-plans/12-band-a-implementation-plan.md`): the runtime owns
and verifies a coherent power/clock/return/pin/signal/interface architecture for a real small board.

## Context
A [`Signal`] (ADR-0026) is a single named directional flow (source → sinks). Real subsystems connect
through **interfaces**: named collections of signals that together implement a protocol (I²C, SPI,
USB, Ethernet, etc.). An interface without a protocol is just a bag of signals; a protocol without
an interface has nowhere to apply. They are **co-dependent** — the Interface/Contract Map is one
Map with an object pair (the world-model §Band B lists `Interface` as a Band B object; the
dependency diagram `02` line 340 shows `Subsystem ─► Interface/Contract ─► Signal Flow`). Shipping
both in one increment is a documented exception to the default one-object-per-increment discipline,
justified by the Map's own object definition (mirrors PinCapability/PinAssignment in ADR-0025).

## Decision
Model the interface architecture as **two first-class, auditable objects** raised through the
capability seam (P2/P3): a `Contract` (protocol rule-set) and an `Interface` (named signal
collection governed by a contract).

- **Domain (`eak-domain`).**
  - `struct Contract { id, protocol: String, name: String, constraints: Vec<String> }` — the
    protocol rule-set (e.g. "I²C", "SPI", "USB2"). `validate()` reuses existing `DomainError`
    variants only: non-empty `protocol` (`EmptyField("contract protocol")`) and non-empty `name`
    (`EmptyField("contract name")`). `protocol` is a `String` because protocol names are open-
    ended (I²C, SPI, USB, CAN, Ethernet, DDR, PCIe, custom); an enum would fabricate a closed
    world (P7). `constraints` is free-form text for protocol-specific rules (structured rules live
    in the rule engine, not here).
  - `struct Interface { id, name: String, signals: Vec<EntityId>, contract: EntityId }` — a named
    collection of signals governed by a contract. `validate()` reuses existing variants only: non-
    empty `name` (`EmptyField("interface name")`), non-empty `signals` (`Inconsistent`), non-null
    `contract` (`Inconsistent`).
  - Both derive `Eq` (no `PhysicalQuantity`).

- **Events (`eak-ports`).** Two new **state-bearing** variants with **explicit fold arms** in
  `EngineeringState::apply` (the one sharp edge; the catch-all would silently diverge replay, P4):
  `ContractCommitted { contract }` and `InterfaceCommitted { interface }`.

- **State (`eak-runtime`).** Two new stores kept in insertion (event) order:
  `EngineeringState::contracts` and `EngineeringState::interfaces`; accessors `contract(id)` /
  `interface(id)`.

- **Seam (`eak-runtime`).** `CapabilityRequest::CreateContract { contract, links }` and
  `CapabilityRequest::CreateInterface { interface, links }`. The contract seam re-validates (non-
  empty protocol/name). The interface seam re-validates (non-empty name, ≥1 signal, non-null
  contract) **and** enforces referential integrity (P3, P5): `contract` and every `signal` must
  resolve to committed objects — an interface can never reference a phantom contract or signal.
  **The seam does NOT judge contract satisfaction** (see Verification): a well-formed interface
  names real objects, so it must enter state for the rule to report a contract violation (e.g. an
  I²C interface with 3 signals instead of 2). A rejected proposal commits nothing. Readers
  `contracts()` / `interfaces()` added to `AgentContext` and its impls.

- **Verification (`eak-engines` / `eak-phases`).** `VerificationContext` gains `contracts: &[Contract]`
  and `interfaces: &[Interface]` (all construction sites updated: every phase machine — ERC, DRC,
  BOM, DFM, EMC, constraint — the `eak-kicad` fixture, and the engines' own test fixtures). New
  rule `InterfaceContractRule` (`erc-interface-contract`), registered on the ERC engine: an
  interface must satisfy its contract's structural requirements. This v0 encodes a **minimal set of
  well-known protocol checks directly** (I²C = exactly 2 signals; SPI ≥ 4 signals; USB2 ≥ 2
  signals) because the runtime does not yet own a full protocol knowledge library (that's a Memory-
  layer concern). Deterministic (P4): interfaces scanned in slice order; one Error finding per
  violated requirement. A missing contract or unknown signals are also flagged (honesty).

## Consequences
- **The interface architecture is a committed, traceable object (exit criterion 1).** Contracts and
  interfaces have stable identities and resolve to committed signals/contracts (→ pins → components
  → blocks → intent, P3). The kernel owns the interface layer; the model can *propose* but never
  bypass the seam.
- **Contract violations are engineering findings, reported.** The rule raises a blocking Error the
  moment an I²C interface has ≠2 signals, SPI has <4, USB <2. These are first-class findings
  (traceable, waivable) rather than silently accepted malformed interfaces.
- **Protocol names are open-ended strings, not a closed enum.** The rule handles known protocols
  (I²C, SPI, USB2) and silently passes unknown ones — the Memory layer (future) will supply the
  full protocol knowledge library.
- **Seam re-validation holds (P3).** Six malformed proposals — blank contract protocol, blank
  contract name, blank interface name, no signals, null contract, dangling contract, dangling
  signal — are each rejected at the seam and nothing enters the log. The QA suite asserts all six
  plus the empty-log-on-rejection property (for interfaces; the contract from setup remains), and
  the accept-paths assert on `contract(id)` / `interface(id)`.
- **Determinism/replay intact (P4).** Both events are folded by exactly one explicit arm each; the
  fold is the only mutation path. The QA suite asserts byte-identical replay across a re-fold with
  both objects in the log.
- **Verification context is uniform.** Both fields are threaded through every phase machine so the
  engine sees one context shape everywhere; the Band B rule runs where the interface layer is
  populated.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the entities,
  `eak-ports` the events, `eak-runtime` the stores/folds/accessors/seam, `eak-engines` the rule,
  `eak-phases` the registration. No adapter dependency introduced.
- **Schema version.** Additive — two new `Event` variants plus two domain entities; no existing
  carrier was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Limitations (explicit, per the honesty principle)
This v0 owns the **structural contract check only** (signal count for well-known protocols). It does
not yet verify:
- Signal *direction* matching the protocol (e.g. I²C SDA bidirectional, SCL output from master).
- Signal *electrical type* matching the protocol (e.g. I²C needs open-drain + pull-ups).
- *Timing* requirements (setup/hold, clock stretching, bus frequency).
- *Address* uniqueness (I²C address collision).
- *Termination* requirements (USB differential pair impedance, CAN termination).

These need the protocol knowledge library (Memory layer), the pin electrical-type matrix, and the
SI layer — all natural future increments. The v0 rule honestly passes unknown protocols rather than
inventing requirements.

## Alternatives considered
- **A single `Interface { protocol: String, signals: ... }`** — rejected: that conflates the
  protocol rule-set (reusable across interfaces) with a specific interface instance. An I²C
  contract is defined once and applied to many I²C interfaces; duplicating the protocol name in
  each interface loses that reusability and makes protocol updates error-prone.
- **`Contract` as a field on `Interface`** — rejected: the contract is a separate, auditable
  engineering object with its own identity and lifecycle; nesting it would collapse the Map into a
  tag (against master-prompt §29) and forfeit seam referential integrity for the contract.
- **An enum for `protocol`** — rejected: protocol names are open-ended (I²C, SPI, USB, CAN,
  Ethernet, DDR, PCIe, MIPI, SDIO, custom); an enum would fabricate a closed world (P7).
- **Reject a contract-violating interface at the seam** — rejected: the *shape* of an interface
  (name, signals, contract) is a fact the runtime records; whether it *satisfies* the contract is a
  **judgement** the verification layer owns (mirrors ADR-0022–0026). Rejecting at the seam would
  conflate "malformed commit" with "this interface violates its protocol", and would make an
  illegal-but-honestly-declared interface unrepresentable.
- **Warn (not Error) on a contract violation** — rejected: an I²C interface with 1 signal or an SPI
  interface missing CS is a structural protocol violation that will fail on the bench. It blocks
  release, matching the Phase-3 ERC severity discipline.
- **One increment per object** — rejected (documented exception): the two rules are meaningless
  with only one object present, and the dependency diagram (`02` line 340) shows
  `Interface/Contract` as a single Map step. Shipping both is the unit of *verifiable* value.