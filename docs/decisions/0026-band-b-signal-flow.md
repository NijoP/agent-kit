# ADR-0026 — `Signal`: the logical electrical meaning above raw connectivity

**Status:** Accepted (Phase 5 / Band B, increment 5 — "Signal Flow"). Fifth increment of the Band B
**logical-electrical Maps** layer (`project-plans/02-engineering-world-model.md` §Band B; Map 16 —
Signal Flow). Anchored to `project-plans/00-product-vision.md` (Principle 3 — the kernel is the moat;
Principle 7 — honesty; P9 — first-class physical quantities) and the master-prompt §32 rule — *Signal
is the logical electrical meaning above raw connectivity, NOT a Net rename; only fields the
architecture can justify.* Delivers Band B **exit criterion 1** (as scoped by the Band B sibling of
`project-plans/12-band-a-implementation-plan.md`): the runtime owns and verifies a coherent
power/clock/return/pin/signal architecture for a real small board.

## Context
A [`Net`] says "these pins are connected" — it is undirected copper. It does NOT say "current flows
from here to there, and means this." The schematic's logical layer is *directional signal flow*:
source → sink, with meaning. This is Map 16 (`02` line 215): "directional *logical* signal flow
(source→sink), distinct from undirected copper. Exists because a Net says 'connected'; a Signal says
'flows from here to there, and means this.'"

The kernel has the power architecture (ADR-0022), clock (ADR-0023), return (ADR-0024), and pin
function (ADR-0025). What it still lacks is the **signal flow architecture** — the logical
electrical meaning that tells an AI *and* a human what a net *does*, not just what it connects.

## Decision
Model the signal flow architecture as a **first-class, auditable `Signal` entity** raised through
the capability seam (P2/P3): a named, directional logical flow (source pin → sink pins) with a
semantic meaning.

- **Domain (`eak-domain`).**
  - `struct Signal { id, name, source: EntityId, sinks: Vec<EntityId>, semantics: String }` — a
    named flow from one driving pin to one or more receiving pins, carrying a meaning (e.g.
    "system clock", "SPI clock", "active-low reset"). The world-model Map 16 lists
    `{source, sinks, direction, semantics}`; `direction` is **encoded implicitly** by the
    `source`→`sinks` pair — a separate `direction` enum would be redundant (a signal with one source
    and N sinks has one direction by construction) and would over-encode (master-prompt §32: only
    fields the architecture can justify). `semantics` is a `String` because logical meaning is
    open-ended; an enum would fabricate a closed world (P7).
  - `validate()` reuses existing `DomainError` variants only: non-empty `name`
    (`EmptyField("signal name")`), non-empty `semantics` (`EmptyField("signal semantics")`), a
    non-null `source` (`Inconsistent`), ≥1 `sink` (`Inconsistent`), and `source ∉ sinks`
    (`Inconsistent` — a self-driving loop is KCL nonsense).
  - `Signal` carries no `PhysicalQuantity`, so — like `Pin`/`FunctionalBlock` — it derives `Eq`.

- **Events (`eak-ports`).** One new **state-bearing** variant with an **explicit fold arm** in
  `EngineeringState::apply` (the one sharp edge; the catch-all would silently diverge replay, P4):
  `SignalCommitted { signal }`.

- **State (`eak-runtime`).** A new store `EngineeringState::signals: Vec<Signal>` kept in insertion
  (event) order; accessor `signal(id)`.

- **Seam (`eak-runtime`).** `CapabilityRequest::CreateSignal { signal, links }` re-validates
  (non-empty name/semantics, non-null source, ≥1 sink, source ∉ sinks) **and** enforces referential
  integrity (P3, P5): `source` and every `sink` must be committed pins — a signal can never flow
  from/to a phantom pin. **The seam does NOT judge driver/sink legality** (see Verification): a
  well-formed signal names real pins, so it must enter state for the rule to report an illegal
  driver/sink pairing (an Input source, an Output sink). A rejected proposal commits nothing.
  Reader `signals()` added to `AgentContext` and its impls.

- **Verification (`eak-engines` / `eak-phases`).** `VerificationContext` gains `signals: &[Signal]`
  (all construction sites updated: every phase machine — ERC, DRC, BOM, DFM, EMC, constraint — the
  `eak-kicad` fixture, and the engines' own test fixtures). New rule `SignalDriverSinkRule`
  (`erc-signal-driver-sink`), registered on the ERC engine: a signal's `source` pin must be
  **output-capable** (`Output` or `Bidirectional`) and every `sink` pin must be
  **input-capable** (`Input` or `Bidirectional`). This is source/load legality as a
  Thévenin/KCL question (`engineering-science/electrical/circuit-theory.md` L134/L152): two
  outputs on one net, a floating input, or an input driving cannot satisfy KCL with a consistent
  voltage — the flow has no physical operating point. Deterministic (P4): signals scanned in slice
  order; one Error finding per illegal pin (source or each offending sink), so a signal with an
  illegal source AND an illegal sink produces two findings. A signal referencing a pin the runtime
  has never seen is ALSO flagged: the flow is unverifiable (honesty, not an invented requirement).

## Consequences
- **The signal flow architecture is a committed, traceable object (exit criterion 1).** Every
  logical signal has a stable identity, resolves to committed pins (→ components → blocks → intent,
  P3), and carries its meaning. The kernel owns the signal layer; the model can *propose* signals
  but never bypass the seam.
- **Illegal driver/sink pairings are caught at ERC.** The rule raises a blocking Error the moment
  an Input drives or an Output receives, exactly the silent failures ERC exists to prevent
  (circuit-theory.md L166: "two outputs on a net, a floating input, or a power pin tied to a signal
  output cannot satisfy KCL with a consistent Thévenin voltage"). These are first-class findings
  (traceable, waivable), not silently accepted.
- **Direction is implicit, not a redundant field.** The `source`→`sinks` pair encodes the direction;
  a separate `direction` enum would over-encode. A bidirectional signal (I²C SDA) is represented by
  *two* `Signal` objects (one in each direction) or by a `Bidirectional` pin electrical type on
  both ends — the rule's `Bidirectional` handling supports this. This is an honest v0 adaptation:
  the Map's `direction` field is realized by the source/sink topology rather than a redundant enum.
- **Seam re-validation holds (P3).** Seven malformed proposals — blank name, blank semantics, null
  source, no sinks, self-drive, dangling source, dangling sink — are each rejected at the seam and
  nothing enters the log. The QA suite asserts all seven plus the empty-log-on-rejection property,
  and the accept-path asserts on `signal(id)`.
- **Determinism/replay intact (P4).** `SignalCommitted` is folded by exactly one explicit arm; the
  fold is the only mutation path. The QA suite asserts byte-identical replay across a re-fold with
  the signal in the log.
- **Verification context is uniform.** The `signals` field is threaded through every phase machine
  so the engine sees one context shape everywhere; the Band B rule runs where the signal layer is
  populated.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` gains the entity,
  `eak-ports` the event, `eak-runtime` the store/fold/accessor/seam, `eak-engines` the rule,
  `eak-phases` the registration. No adapter dependency introduced.
- **Schema version.** Additive — one new `Event` variant plus one domain entity; no existing
  carrier was re-shaped and no IR schema constant changed, so there is no `u32` to bump.

## Limitations (explicit, per the honesty principle)
This v0 owns the **logical-electrical contract** only:
- The rule checks pin electrical-type legality (Output/Bidirectional driving → Input/Bidirectional
  receiving). It does **not** yet verify that source and sinks share a *realized Net* (copper
  realization) — that would require looking up `Net` membership from pins, a natural future
  refinement when the Connectivity Engine matures.
- It does not yet verify timing (setup/hold, clock-to-output) or electrical margins (reflection,
  crosstalk) — those need the SI layer (`TransmissionLineModel`, `CrosstalkPair` per Map 31) and
  the return-path geometry (Map 20).
- A `Signal` names the *logical* flow; the *physical* realization is the `Net` that carries it. The
  two-layer separation (Map 16 vs copper Maps 28/31) is the architecture's intent — this increment
  owns the logical layer only.

## Alternatives considered
- **`Signal` as a field on [`Net`] (e.g. `Net.signal: Option<Signal>`)** — rejected: that is the
  conflation the master-prompt §32 forbids ("NOT a Net rename"). A net can carry multiple logical
  signals (time-multiplexed), and a logical signal may span multiple nets (through bridges). The
  objects are distinct.
- **An enum for `semantics`** — rejected: logical meaning is open-ended ("SPI clock", "active-low
  reset", "enable line", "interrupt", "DDR DQS", etc.); an enum would fabricate a closed world (P7).
- **A separate `direction` enum** — rejected: the source→sinks topology *is* the direction; a
  redundant enum adds no information and would over-encode (§32). A bidirectional physical line is
  modeled by `Bidirectional` pins or paired signals.
- **Reject an illegal driver/sink at the seam** — rejected: the *shape* of a signal (name, source,
  sinks, semantics) is a fact the runtime records; whether the pairing is *legal* is a **judgement**
  the verification layer owns (mirrors ADR-0022/0023/0024/0025). Rejecting at the seam would
  conflate "malformed commit" with "this flow has no operating point", and would make an
  illegal-but-honestly-declared signal unrepresentable.
- **Warn (not Error) on an illegal driver/sink** — rejected: an Input driving or an Output
  receiving is a circuit with no physical solution (circuit-theory.md L166). It blocks release,
  matching Phase-3 ERC severity discipline.
- **No `name` field** — rejected: the Map 16 object lists `semantics` but not `name`. A signal
  without a name ("SYS_CLK", "SPI_MOSI") is not a first-class engineering object; every real signal
  has a name in the datasheet and schematic. The exemplar objects (PowerDomain, ClockDomain,
  ReturnPath) all carry names. The `name` is the identity; `semantics` is the meaning.