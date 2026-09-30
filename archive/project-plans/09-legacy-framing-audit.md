# Legacy-Framing Audit — reconciling the repo with the canonical vision

> Companion to **[`00-product-vision.md`](00-product-vision.md)** (canonical). This report finds
> every place the repository still frames Electronics Agent Kit as an **ECAD / PCB editor / KiCad
> or EasyEDA replacement / CAD software / AI-copilot-on-an-editor**, and recommends a reframing to
> the **AI-Native Engineering Operating System** category the vision defines.
>
> **This is a report only. No source files were modified.** Recommendations are staged for a
> deliberate, reviewed pass. Written 2026-07-10.

---

## Executive summary

**The good news the audit surfaced is as important as the drift.** Two of the three layers of the
repo are *already aligned* with the new vision:

1. **The kernel is vendor-format-clean — a core vision invariant already holds in code.** No KiCad /
   `.kicad_pcb` / vendor-format concept appears anywhere in `eak-domain`, `eak-runtime`, `eak-ports`,
   or `eak-units`. Vendor formats are confined to the outermost ring (`eak-kicad`, `eak-cli`),
   exactly as Principle 8 and §10 of the vision require. The dependency-rule guard test
   (`eak-runtime/src/lib.rs`) enforces this at compile time. **The runtime already owns truth;
   KiCad is already peripheral in the code.**
2. **The `docs/` design tree already rejects the legacy framing.** `docs/README.md:5`,
   `docs/foundation/vision.md:17`, and `docs/presentation/frontend.md:109` explicitly say *"not a PCB
   editor... it is an Engineering Runtime."* The Phase-0 architecture docs are on-message.
3. **The drift is a documentation lag, concentrated in the MVP-pivot layer.** The
   `project-plans/` docs (written 2026-07-02, eight days before the vision), `team/` +
   `team/agents/` + `.claude/agents/`, and `app/README.md` lean on the **"AI-native EDA IDE — Cursor
   for hardware"** tagline and the **"reuse the canvas"** MVP pragmatics. That framing was correct
   for a fundable-demo memo; it is subordinate to the vision now.

**Nothing in the audit contradicts the vision's feasibility.** The reset is overwhelmingly a
*terminology and positioning* correction, not an architecture change. The two genuine product
questions it raised — how hard-line on interop, and whether to keep the tagline — are now **resolved
by founder ruling** (see below).

**Counts:** ~10 hard identity statements ("EDA IDE" as the product's category noun) · ~14
"Cursor for hardware" tagline occurrences · ~30 "reuse the canvas / KiCad" interop mentions (mostly
acceptable) · engineering-science domain references to "CAD tool" (all fine — they describe the
industry, not EAK).

---

## Classification scheme

| Cat | Meaning | Action |
|---|---|---|
| **A — Identity drift** | Frames EAK *itself* as an EDA IDE / editor / CAD tool / KiCad replacement, or frames "we're not an editor" as a *time/scope* constraint rather than a deliberate architectural boundary. | **Must reframe.** |
| **B — Interop** | Import/export of vendor formats, reusing a renderer, parts APIs. | **Per Ruling 1 (hard-line): demote/remove from the product narrative; any retained mention must be labelled throwaway demo scaffold with an expiry. The runtime must never depend on it.** |
| **C — Positioning** | Tagline ("Cursor for hardware") and competitive-comparison language. | **Per Ruling 2: remove the "Cursor for hardware" tagline everywhere; lead with the Engineering-OS category.** |
| **D — Domain description** | Vendor-neutral prose about how *the industry / fabs / other CAD tools* work. Describes the domain, not EAK. | **No change.** |

**A precision that prevents over-correcting:** *"IDE"* as a **UI-shell metaphor** for the frontend
(docking, command palette, panels — "command a system, not operate a drawing tool") is **fine** and
well-argued in the docs. *"EDA IDE"* as the **product's category noun** ("the product is an EDA
IDE") is the drift. Reframe the noun; keep the shell metaphor.

---

## Founder rulings (2026-07-10) — RESOLVED

**Ruling 1 — Interop is scaffold (the hard-line option).** KiCad import/export and any reused
renderer are throwaway demo scaffolding, not a standing capability. The vision/roadmap narrative must
stop presenting interop as a peripheral *feature* and instead mark it as a temporary demo crutch with
an expiry; the runtime must never depend on it (already true — the kernel is vendor-format-clean, so
the scaffold is safely removable). This **upgrades** the ~30 Category-B "reuse the canvas / KiCad"
lines from *"acceptable, keep"* to *"demote or remove from the product narrative"* — they survive only
as descriptions of clearly-labelled throwaway demo scaffold.

**Ruling 2 — Retire "Cursor for hardware" entirely.** Drop the tagline everywhere; lead exclusively
with *AI-Native Engineering Operating System*. This **upgrades** every Category-C tagline occurrence
from *"keep as qualified hook"* to *"remove."*

`00-product-vision.md` has been updated to encode both rulings (§10 rewritten to the scaffold stance;
the tagline retired in the header and §4). The recommendations and tiers below reflect them. The
per-file tables still tag each line A/B/C for provenance; **apply the ruling-adjusted action, not the
original table verb, where they differ.**

---

## Findings by file

### `project-plans/00-overview.md` — the current "source of truth" (now subordinate to the vision)

| Line | Quote (short) | Cat | Recommendation |
|---|---|---|---|
| 12 | "AI-native **EDA IDE for PCB/electronics design** — Cursor for hardware" | **A** | Lead identity → "**AI-Native Engineering Operating System for electronics**"; keep tagline as a parenthetical demo hook. |
| 21 | "Cursor for hardware — an AI harness you can actually trust…" | C | Keep as hook; immediately qualify with "on a deterministic runtime that owns engineering truth." |
| 41 | "Not beating KiCad on **editing features**" | C | "Not competing as an editor at all" (we have no editing features by design). |
| 82 | "cannot build a full **schematic-capture + PCB-layout editor** … in 3 months" | **A** | Reframe as architectural boundary, not a time constraint: "We are not building an editor; drawing is a peripheral projection." |
| 98 | "a full **schematic/layout editor to rival KiCad**" | **A** | "We do not build an editor to rival KiCad — different category." |
| 18, 75, 89, 93 | "visual editor/canvas is **reused**"; "Canvas/renderer = REUSED"; "Reuse — KiCad libs" | B | Fine per §10. Optionally tag the renderer "peripheral display adapter (interchangeable; kernel can emit SVG/PNG)." |
| 26, 32–37 | "Competitors are **editor-first tools with an AI copilot**"; "AI that suggests vs AI you can trust to drive" | C→D | Strong, on-message *contrast*. Keep; it defines the category by negation. |

> Add a header banner: *"Subordinate to `00-product-vision.md`. Where framing here conflicts with the
> vision, the vision wins."*

### `project-plans/01-product-spec.md`

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| 11 | "The MVP is: a local, native **AI-native EDA IDE** — Cursor for hardware" | **A** | "The MVP is a fundable *slice* of the Engineering Operating System, surfaced through a local desktop shell." |
| 203 | "Cursor for hardware — an AI harness you can actually trust…" | C | Qualify with the runtime/moat clause. |
| 29 | "**Not a KiCad replacement** — we don't **out-edit** KiCad; we reuse its canvas" | **A** | The reason is wrong: not "we edit worse," but "we are not an editor." → "Not an editor at all; the runtime owns engineering, the canvas is a peripheral view." |
| 13, 67, 85, 141 | "canvas is reused (KiCanvas)"; "reused renderer"; "Building our own editor" (as a non-goal) | B | Fine per §10. `:141` correctly lists "building our own editor" as out-of-scope — keep. |

### `project-plans/02-technical-architecture.md`

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| 5, 20, 22, 74 | "canvas = reused"; "Reuse KiCanvas for the canvas"; "Reused WebGL KiCad viewer" | B | Fine. Add one line: "the renderer is a peripheral display adapter; if KiCanvas fails, the kernel projects SVG/PNG." |
| 230 | "## 5. **Reuse strategy for the canvas/editor** (buy vs build)" | B/C | Retitle "Rendering strategy (peripheral display adapter)"; drop "editor." |
| §6 (import) | "the importer **synthesizes a minimal spine**" | **stale** | **Factually outdated** (separate from framing): ADR-0017 *replaced* this with `ComponentOrigin::Imported`. Correct the mechanism regardless of the reset. |

### `project-plans/03-roadmap.md`

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| 51, 58, 162 | "REUSED = third-party (KiCanvas/KiCad)"; "embed KiCanvas and render it" | B | Fine per §10 (peripheral render). |
| 196 | "M7–M9: **real editing** … a genuine **schematic/layout editing surface** so users can drive" | B/C | Acceptable *if* framed as an application/view onto the runtime whose edits round-trip through the seam (the line already says "back through the kernel with full traceability"). Tighten: an editing *surface* is an app on the substrate, never a second source of truth. |

### `project-plans/04-competitive-positioning.md`

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| 157, 161, 166, 168, 192 | tagline analysis; "**Category:** Cursor for hardware — on a deterministic correctness runtime" | C | Already self-aware about the copilot risk. Update the **Category** row to name the true category — *AI-Native Engineering Operating System* — with "Cursor for hardware" as the hook beneath it. |
| 58, 136 | "Pre-product: kernel only"; "reuse the canvas (KiCanvas), compete on *trust*, not pixels" | B/C | On-message. Keep. |

### `project-plans/05-fundraise-plan.md`

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| 79, 178 | deck one-liner; "Show HN: Cursor for hardware…" | C | Keep as hook; ensure the sentence *after* it states the category (Engineering OS / runtime-owns-truth). |

### `project-plans/06-risks-and-buy-vs-build.md`

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| 27 | "KiCad import edge cases … curated import files we ship and control" | B | Fine — this is honest interop risk management. |
| 112–114 | "Canvas/renderer: REUSE — never build (KiCanvas), fall back to kernel SVG/PNG"; "parsing: REUSE"; "parts: REUSE + cache" | B | Fine per §10. The SVG/PNG fallback already proves the renderer is peripheral. |

### `project-plans/07-engineering-backlog.md`

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| 4 | "a **local Tauri AI-native EDA IDE**" | **A** | → "a local desktop shell over the Engineering Operating System kernel." |
| 6, 56, 141–159 (E4) | "canvas is reused"; "**E4 — Reused canvas / renderer integration**"; "never build a PCB renderer" | B | Fine. Optionally rename E4 "Peripheral renderer integration." |

### `docs/` — already aligned; two positioning tweaks only

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| `docs/README.md:5` | "AI-native **Engineering IDE for PCB and electronics design**. It is **not** a PCB editor … an **Engineering Runtime**." | C | Body is correct. Lead noun → "Engineering Operating System"; let "PCB" be *a domain it serves*, not the identity. |
| `docs/foundation/vision.md:7` | "AI-native **Engineering IDE for PCB and electronics design**." | C | Same tweak. `:17` ("Not another PCB editor") is exemplary — keep. |
| `docs/presentation/frontend.md:109` | "Why an **IDE shell**, not a PCB editor and not a chatbot." | **D/keep** | Model example of correct framing. No change. |
| `docs/integration/simulation-interface.md:69` | SPICE/SI/PI solvers "behind one port … provider-independent" | B/keep | Correct outer-ring interop. No change. |

### `app/`

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| `app/README.md:3` | "the shell for the MVP (**Cursor for hardware**)" | C | "the desktop shell over the Engineering Operating System kernel." |
| `app/src-tauri/*`, `app/ui/index.html` | (event-feed spine) | — | Code framing is clean; no drift. |

### `team/` and `team/agents/` (+ `.claude/agents/` mirrors)

| Line | Quote | Cat | Recommendation |
|---|---|---|---|
| `team/README.md:4` | "AI-native **EDA IDE** (Cursor for hardware)" | **A** | → "AI-Native Engineering Operating System." |
| `team/00-plan-and-requirements.md:12` | "a local, AI-native **EDA IDE**" | **A** | Same. |
| `team/org-structure.md:78` | "reuse the canvas, don't rebuild it" | B | Fine. |
| `team/agents/eak-eng-lead.md:7` · `.claude/agents/eak-eng-lead.md:7` | "a local, AI-native **EDA IDE**" | **A** | Same. **Edit both mirrors.** |
| `team/agents/eak-product-manager.md:10` · `.claude/agents/eak-product-manager.md:10` | "The product is a local, native, AI-native **EDA IDE**" | **A** | Same. **Edit both mirrors.** |
| `team/agents/eak-market-analyst.md:16,37` · `.claude/` mirror | "Own the one-liner: Cursor for hardware…" | C | Keep as hook; add "the category is Engineering OS; the analogy is the hook." |
| `team/agents/eak-canvas-integration-engineer.md:13` · `.claude/` mirror | "never build a canvas — reuse KiCanvas" | B | Fine. Consider role rename emphasis "peripheral renderer + KiCad *bridge* engineer." |
| `team/agents/eak-design-lead.md:11` · `.claude/` mirror | "canvas is reused (KiCanvas), not a bespoke editor" | B | Fine. |

### `engineering-science/` — domain descriptions, no change

| Example | Cat |
|---|---|
| `industry/manufacturing-methodology.md:41,138` — "independent of which **CAD tool** drew it"; "the fab cannot ask your CAD tool" | **D** — describes fabs/industry, not EAK. Keep. |
| `industry/routing-philosophy.md:3` — "vendor-neutral; not any one **CAD tool**" | **D** — keep. |
| `industry/constraint-systems.md:112` — "re-entered by hand in the **PCB editor**" | **D** — describes legacy flows. Keep. |
| `mathematics/control-theory.md:141` — "Auto-routers have used negotiated-congestion…" | **D** — domain theory. Keep. |

---

## Prioritized change list (when you approve a pass)

**Tier 1 — hard identity statements (Category A, ~10 edits, highest leverage):**
1. `project-plans/00-overview.md:12` — the single most-cited identity line.
2. `project-plans/01-product-spec.md:11`
3. `project-plans/README.md:4`
4. `project-plans/07-engineering-backlog.md:4`
5. `team/README.md:4`, `team/00-plan-and-requirements.md:12`
6. `team/agents/eak-eng-lead.md:7` **+** `.claude/agents/eak-eng-lead.md:7`
7. `team/agents/eak-product-manager.md:10` **+** `.claude/agents/eak-product-manager.md:10`
8. `project-plans/00-overview.md:82, 98` — reframe "not an editor" as boundary, not time.
9. `project-plans/01-product-spec.md:29` — fix the "out-edit" reasoning.

**Tier 2 — retire the tagline + fix positioning (Category C, Ruling 2):** **remove** every "Cursor
for hardware" occurrence (project-plans, `team/`, `app/README.md`, agent defs **+ their `.claude/`
mirrors**); update `docs/README.md:5` + `docs/foundation/vision.md:7` lead noun to "Engineering
Operating System"; update the `04-competitive-positioning.md` **Category** row to name the true
category.

**Tier 3 — demote interop to scaffold (Category B, Ruling 1):** relabel every "reuse the canvas /
KiCad import-export" mention as *throwaway demo scaffold with an expiry*; remove framing that presents
interop as a standing product capability; retitle `02-technical-architecture.md §5` and backlog E4.
The runtime stays vendor-format-clean, so this is a **narrative-only** pass — no kernel code changes.

**Tier 4 — factual (independent of the reset):** correct `02-technical-architecture.md §6`
"synthesizes a minimal spine" → `ComponentOrigin::Imported` (ADR-0017). This is a staleness bug, not
framing.

**Cross-cutting mechanic:** add a one-line "**Subordinate to `00-product-vision.md`**" banner to the
top of `00-overview.md`, `project-plans/README.md`, and `team/README.md`, so the hierarchy is
explicit and future edits inherit the correct frame.

---

## What is explicitly NOT drift (do not touch)

- The vendor-format-clean kernel (`eak-domain/-runtime/-ports/-units`) — already correct.
- `eak-kicad` / `eak-cli` doc-comments — already frame KiCad as "an adapter (outermost ring)."
- The `docs/` tree's "not a PCB editor / Engineering Runtime" statements — already on-message.
- All `engineering-science/` "CAD tool / PCB editor" references — domain description (Category D).
- "IDE" used as a *UI-shell metaphor* — a defended, correct choice; only the *product-category* use
  of "EDA IDE" is drift.

---

*No files were modified in producing this report. Await founder direction on Open Decisions 1–2,
then execute the change pass tier by tier behind the green-gate.*
