# ADR-0017 — `ComponentOrigin`: distinguishing synthesized from imported components

**Status:** Accepted (Phase 3 / epic E5, increment F2). The eak-architect ruled on and signed off
this fix, rejecting the fabricated-spine alternative.

## Context
The KiCad importer (`eak-kicad`, epic E5) parses a finished board's `(footprint …)` nodes into
`eak_domain` [`Component`]s + [`Placement`]s (each pad becomes a [`Pin`]) so an imported board carries
*parts* for the canvas + review, not just copper (increment F2). Those parts must enter the runtime
through the same `RealizeComponent` capability seam a generated part uses — re-validated (P3),
recorded in the append-only log (P2), no back door.

But `RealizeComponent` (`handle_realize_component`) required every component to be *minted from a
committed [`FunctionalBlock`]* — a synthesis invariant: a generated component with a null or unknown
`from_block` is untraceable to intent (P3) and is rejected. An imported board declares **no functional
decomposition** — no design intent, no requirements, no blocks — so an imported footprint has no
block to be minted from.

The first F2 implementation satisfied the seam by **fabricating a provenance spine**: on any import
carrying parts it committed a synthetic `IntentCaptured` ("Imported from .kicad_pcb"), one
reverse-engineered `Requirement`, and one `FunctionalBlock`, then stitched every imported component's
`from_block` to it. The architect **rejected that**: it invents design intent the board never stated
and poisons traceability — a trace query on an imported part would terminate at a fabricated
requirement/intent that no engineer ever authored, exactly the kind of confident fiction the moat
exists to prevent. The fix must not fabricate intent, and must not weaken the synthesis invariant (a
*synthesised* component with a null/unknown block must still be rejected).

## Decision
Make the block-origin rule **origin-dependent** via a first-class tag rather than fabricating a spine
or relaxing the check — mirroring ADR-0016's [`NetOrigin`] exactly:

- Add `enum ComponentOrigin { Synthesized, Imported }` to `eak-domain` (`Default = Synthesized`), and
  a `#[serde(default)] pub origin: ComponentOrigin` field on [`Component`]. `Component::validate()` is
  **unchanged** (it never checked the block — the block rule lives at the seam, not in the entity).
- In `handle_realize_component` (P3), branch on `component.origin`: `Synthesized` keeps the existing
  behavior **verbatim** (`from_block` non-null AND a committed block); `Imported` **may** carry a null
  `from_block` (an imported board declares no block), but any block it *does* reference must still be a
  committed one — Imported relaxes existence only, never the no-phantom-block rule. The `≥1`-pin
  rejection stays for **both** origins (footprint pads are real pins). The `RealizeComponent` protocol
  variant is unchanged (the branch lives inside the handler).
- "No parent block" is represented by the existing null sentinel `EntityId::NULL` on the plain
  `EntityId` `from_block` field (the same sentinel the synthesis-defect check already used) — the
  smallest honest change; no `Option<EntityId>` migration was needed.
- The importer tags every component it builds `ComponentOrigin::Imported` with `from_block =
  EntityId::NULL`; every synthesis/existing path is `ComponentOrigin::Synthesized`. `eak-cli`'s
  `import_design` now realizes imported parts directly through `RuntimeCore::invoke` → `commit`, with
  **no** synthesized intent/requirement/block spine. (The default fabrication-floor seeding, E5.1,
  stays — that is a legitimate standard default, not a fabricated provenance chain.)

## Consequences
- **Invariant preserved:** a `Synthesized` component still needs a committed originating block (the
  synthesis defect is still caught); an `Imported` component never fabricates intent (its trace
  terminates honestly at "imported artifact, no upstream block"); a listed block on an import is still
  re-validated against committed blocks (no phantom); the `≥1`-pin rule holds for both. The moat's P3
  seam re-validation is intact — the model/importer is still never trusted.
- **Traceability stays honest:** the integration guard — `SchematicIr::project` in `eak-compiler`,
  the one consumer that walks `Component -> from_block` for an invariant — now tolerates an `Imported`
  component's null block (it flags only a *synthesised* orphan), so a projected trace terminates
  honestly instead of raising a false `OrphanComponent` or unwrapping. A footprint-only import
  commits **zero** `IntentCaptured` / `RequirementCommitted` / `FunctionalBlockCommitted` events.
- **Determinism/replay unaffected (P4):** `ComponentOrigin` is a static, fieldless enum folded
  identically on replay; `#[serde(default)]` lets pre-existing event logs — which never carried an
  origin — deserialize as `Synthesized`, their original meaning, so historical logs replay
  byte-identically. No `#[serde(deny_unknown_fields)]` sits on `Component` or any envelope that
  carries it, so the serde default is free to fill in `origin` for old logs.
- **Schema version:** the field is additive and serde-defaulted, so it is non-breaking. As with
  ADR-0016, the primary carrier of `Component` in the persisted log — `Event::ComponentCommitted` —
  has no schema-version envelope, and the IR schema constants are monolithic `u32`s with no
  MAJOR.MINOR decomposition; there is no MINOR component to bump, so this is noted and skipped rather
  than incrementing a monolithic counter.

## Alternatives considered
- **Fabricate a synthetic intent/requirement/block spine per import (the first F2 implementation)** —
  rejected by the architect: it invents design intent the board never declared and poisons
  traceability, so a trace on an imported part terminates at fiction no engineer authored (breaks the
  moat's "never confidently fabricate" discipline, P13).
- **Weaken the seam to allow any null-block component** — rejected: it would silently accept a
  *synthesised* component with no block, a real traceability defect, erasing a P3 guarantee.
- **A separate `ImportComponent` capability variant** — rejected as the primary path (heavier seam
  surface for a one-field distinction); the origin tag mirrors the already-accepted ADR-0016
  precedent, keeping the seam surface and mental model uniform.
