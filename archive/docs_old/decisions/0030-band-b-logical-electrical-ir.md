# ADR-0030 — `LogicalElectricalIr`: the logical electrical architecture as a first-class IR between Engineering IR and Schematic IR

**Status:** Accepted (Phase 5 / Band B, increment 9 — "Logical-Electrical IR"). Ninth increment of the
Band B **logical-electrical Maps** layer (`project-plans/02-engineering-world-model.md` §Band B;
the IR band between Engineering IR and Schematic IR called for by `02` line 408). Anchored to
`project-plans/00-product-vision.md` (Principle 3 — the kernel is the moat; P6 — IR is derived,
never a rival source of truth; P9 — first-class physical quantities) and the master-prompt §40
increment list. Delivers Band B **exit criterion 1** (as scoped by the Band B sibling of
`project-plans/12-band-a-implementation-plan.md`): the runtime owns and verifies a coherent
power/clock/return/pin/signal/interface/bus/subsystem architecture, and the compiler can lower
this logical architecture to a schematic.

## Context
The compiler has two IRs on the engineering-to-schematic path:
- [`EngineeringIr`] (increment 1): blocks, constraints, requirements — the *behavioral* architecture.
- [`SchematicIr`] (increment 3): components, pins, nets — the *connectivity* architecture.

What was missing is the **logical electrical architecture** in between: the Band B domain objects
(`PowerDomain`, `ClockDomain`, `ReturnPath`, `PinCapability`/`PinAssignment`, `Signal`,
`Interface`/`Contract`, `Bus`, `Subsystem`) as a first-class IR. Without this, the compiler had to
lower the Engineering IR directly to the Schematic IR, losing the logical-electrical layer where
real engineers specify designs (power trees, clock trees, signal flow, bus topologies, subsystem
boundaries). The Schematic IR can only express *copper* (components, pins, nets); it cannot express
"this net is a 50 Ω transmission line with a declared return path" or "this interface is an I²C bus
with address uniqueness requirements."

The world-model §Band B (`02` line 408) explicitly calls for this: "new IR stage ('Logical Electrical
IR'); domain objects committed through the seam; rules (power balance, mux conflict, CDC, protocol)
added to the verification engine."

## Decision
Model the **Logical-Electrical IR** as a first-class compiler IR (`eak-compiler`) projected from
the [`EngineeringIr`] by *enriching* it with the full Band B domain object set:

- `PowerDomain` (Map 38)
- `ClockDomain` (Map 21)
- `ReturnPath` (Map 20)
- `PinCapability` / `PinAssignment` (Map 22)
- `Signal` (Map 16)
- `Interface` / `Contract` (Map 14/15/17/18)
- `Bus` (Map 17)
- `Subsystem` (Map 14)

The IR is **projected** from the `EngineeringIr` by *enrichment* (transformation P1): the
engineering architecture (blocks + constraints + requirements) is taken as-is, and the Band B
objects are added. The projection enforces traceability (P3): every domain object must be
well-formed (`validate()`) and every cross-reference must resolve (contracts for interfaces,
contracts for buses, interfaces for bus members, interfaces for subsystems). Net/pin existence
checks are deferred to the Schematic IR projection (the Schematic IR already validates that every
net member is a real pin).

The projection is **deterministic** (P4): given the same Engineering IR and Band B state, the
Logical-Electrical IR is always byte-identical. The IR carries its schema version
(`LOGICAL_ELECTRICAL_IR_SCHEMA_VERSION`) and the Engineering IR schema version it was projected
from, enabling forward/backward compatibility checks.

The IR is **never a rival source of truth** (P6): it is a *projection* of canonical state (the
kernel's canonical state + Band B state). It is never authored directly; it is always derived.

## Consequences
- **The logical electrical architecture is a first-class IR (exit criterion 1).** The compiler now
  owns the full logical-electrical architecture as a first-class IR between Engineering and
  Schematic. The schematic can now be *lowered* from a logical architecture rather than guessed
  from a flat netlist.
- **Cross-IR traceability is enforced.** The LogicalElectricalIr carries the Engineering IR it was
  projected from (by clone, so it's self-contained). The Schematic IR projection will (in a future
  increment) take the LogicalElectricalIr as input, ensuring the schematic *realizes* the declared
  logical architecture.
- **Verification context is unified.** The verification engine already runs rules over the
  `VerificationContext` (which has all Band B objects). The LogicalElectricalIr now provides a
  single, serializable IR that captures the entire logical-electrical architecture for offline
  analysis, diffing, and exchange.
- **Determinism/replay intact (P4).** The projection is a pure function of its inputs. The test
  suite will assert byte-identical replay across a re-projection.
- **Schema versioning.** The IR carries `LOGICAL_ELECTRICAL_IR_SCHEMA_VERSION` and the
  `engineering_ir_schema_version` it was projected from, enabling compatibility checks.
- **Clean-architecture rings unchanged.** Edges point inward only: `eak-domain` provides the
  domain objects; `eak-compiler` gains the IR; `eak-runtime`/`eak-engines` are unchanged.
- **No schema bump for existing IRs.** The new IR is additive; existing IR schema constants
  (`ENGINEERING_IR_SCHEMA_VERSION`, `SCHEMATIC_IR_SCHEMA_VERSION`) are unchanged.

## Limitations (explicit, per the honesty principle)
This v0 owns the **logical-electrical IR projection only**:
- The Schematic IR projection still takes `components`, `pins`, `nets` as separate inputs. A future
  increment will make `SchematicIr::project` take a `LogicalElectricalIr` directly, ensuring the
  schematic *realizes* the declared logical architecture (e.g., every `Signal` in the LE IR has a
  corresponding `Net` in the schematic, every `Interface` has a corresponding set of `Net`s).
- Deep cross-referencing (e.g., verifying that a `Signal`'s source/sink pins exist in the
  schematic, that a `PowerDomain`'s supply rail net exists) is deferred to the Schematic IR
  projection. The LE IR only validates that the objects themselves are well-formed and
  cross-referenced internally (contracts exist for interfaces, buses reference real contracts and
  interfaces, etc.).
- `Subsystem` boundary pin-completeness check is not yet implemented (same limitation as the
  `SubsystemBoundaryRule` in the runtime).

## Alternatives considered
- **No separate LE IR; lower Engineering IR directly to Schematic IR** — rejected: that is the
  current (pre-increment) state, which loses the logical electrical architecture where engineers
  actually specify designs. The Schematic IR can only express copper; it cannot express "this is a
  differential pair" or "this is an I²C bus."
- **Embed the Band B objects directly into EngineeringIr** — rejected: the Engineering IR is the
  *behavioral* architecture (blocks + constraints + requirements). Mixing logical electrical
  objects into it conflates layers (violates clean architecture). The LE IR is a *separate* IR
  layer with its own schema version.
- **Make LogicalElectricalIr the new SchematicIr (replace SchematicIr)** — rejected: the
  Schematic IR is the *copper* IR (components, pins, nets). The LE IR is the *logical* IR. They
  serve different purposes and have different schema evolution rates.
- **Schema version bump for EngineeringIr** — rejected: the Engineering IR is unchanged; the new
  IR is a separate layer with its own schema version. The LE IR records the Engineering IR
  schema version it was projected from, enabling compatibility checks.