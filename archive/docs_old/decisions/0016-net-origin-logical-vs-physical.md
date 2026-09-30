# ADR-0016 — `NetOrigin`: distinguishing logical (synthesised) from physical (imported) nets

**Status:** Accepted (Phase 3 / epic E5, increment B1). The eak-architect ruled on and signed off
this fix.

## Context
The KiCad importer (`eak-kicad`, epic E5) parses a finished board's **copper** — `(net …)` and
`(segment …)` nodes — into `eak_domain` entities so an existing design can be reviewed by the same
deterministic rule set a generated one is. Copper carries no parsed **pins**, so an imported
[`Net`] joins no members.

The `CreateNet` capability seam (`handle_create_net`) enforced a `≥1`-member connectivity invariant
(P13): a net that joins no pins carries no connectivity and is rejected. That invariant is correct
for a **synthesised** net (Schematic Planning joins pins, so a member-less one is a synthesis
defect), but it wrongly blocked a **legitimately member-less imported** net — halting the whole
import→review flow at the first net.

The fix must not weaken the seam (a member-less *synthesised* net must still be rejected) and must
not fabricate synthetic pins (that would invent connectivity the board never declared, breaking the
"invents nothing it cannot see" honesty of the importer).

## Decision
Make the connectivity rule **origin-dependent** via a first-class tag rather than relaxing the
check or minting phantom pins:

- Add `enum NetOrigin { Logical, Physical }` to `eak-domain` (`Default = Logical`), and a
  `#[serde(default)] pub origin: NetOrigin` field on `Net`. `Net::validate()` is unchanged (it
  never checked members — the `≥1`-pin rule lives at the seam, not in the entity).
- In `handle_create_net` (P3), branch on `net.origin`: `Logical` keeps the existing behavior
  **verbatim** (`≥1` member, and every member a committed pin); `Physical` **may** be member-less,
  but any member it *does* list must still be a committed pin — Physical relaxes the count, never
  the no-phantom-terminal rule. The `CreateNet` protocol variant is unchanged (the branch lives
  inside the handler).
- The importer tags every net it builds `NetOrigin::Physical`; every synthesis/existing path is
  `NetOrigin::Logical`. `eak-cli::import_design` now completes the loop — board → each net → each
  track, all through `RuntimeCore::invoke` → `commit`, with no back door.

## Consequences
- **Invariant preserved:** a `Logical` net still needs `≥1` pin (the synthesis defect is still
  caught); a `Physical` net never fabricates a pin (a listed member is still re-validated against
  committed pins). The moat's P3 seam re-validation is intact — the model/importer is still never
  trusted.
- **Determinism/replay unaffected (P4):** `NetOrigin` is a static, fieldless enum folded identically
  on replay; `#[serde(default)]` lets pre-existing event logs — which never carried an origin —
  deserialize as `Logical`, their original meaning, so historical logs replay byte-identically.
- **Schema version:** the field is additive and serde-defaulted, so it is non-breaking. The IR
  schema constants (`SCHEMATIC_IR_SCHEMA_VERSION` et al.) are monolithic `u32`s with no MAJOR.MINOR
  decomposition, and the primary carrier of `Net` in the persisted log — `Event::NetCommitted` — has
  no schema-version envelope at all. There is therefore no MINOR component to bump; per the ruling's
  guidance this is noted and skipped rather than incrementing a monolithic counter (which would
  falsely signal a breaking schema generation). No `#[serde(deny_unknown_fields)]` sits on `Net` or
  any envelope that carries it, so the serde default is free to fill in `origin` for old logs.

## Alternatives considered
- **Weaken the seam to allow all member-less nets** — rejected: it would silently accept a
  member-less *synthesised* net, a real connectivity defect, erasing a P13 guarantee.
- **Fabricate synthetic pins for imported copper** — rejected: it invents connectivity the board
  never declared, breaking importer honesty (P4/P9) and polluting downstream ERC/DRC with phantom
  terminals.
- **A separate `CreatePhysicalNet` capability variant** — rejected as the primary path (heavier
  seam surface for a one-field distinction); kept only as the fallback had `deny_unknown_fields`
  blocked the serde default (it does not).
