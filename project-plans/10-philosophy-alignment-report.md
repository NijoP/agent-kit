# Engineering-Philosophy Alignment Report

> Companion to **[`01-engineering-philosophy.md`](01-engineering-philosophy.md)** (canonical,
> priority 2) and **[`00-product-vision.md`](00-product-vision.md)** (canonical, priority 1).
> Product of Swarms 8 (architecture validation) and 9 (repository alignment) in the philosophy run.
>
> **This is a report only. No source files were modified.** It validates the existing architecture
> and documentation *against* the newly-frozen philosophy and stages recommendations for a reviewed
> pass. Written 2026-07-10.

---

## Executive summary

**The architecture is a faithful expression of the philosophy. There are zero high-severity
contradictions.** Nothing lets a projection become the source of truth; nothing lets AI write state
without the validating seam; determinism and replay are enforced in code and tested. Of the 12
immutable principles (philosophy Part IX), the architecture **upholds 11 outright**; only #12 (the
generative world model) is *partial and trending up*, exactly as the philosophy itself predicts is
honest for its stage.

The **documentation corpus is broadly aligned in substance but not yet in vocabulary.** Every
core / engineering / compiler / engineering-science doc already *enforces* the constitutional
invariants (sovereign single-writer, propose-then-dispose seam, provenance, determinism, typed
quantities, projection-not-truth). But the corpus reasons in the older **"runtime owns *knowledge*"**
register and is missing the philosophy's four load-bearing frames: **Engineering Truth** as an
epistemic object, **Objectification-by-consequence**, the **layered-and-never-laundered Division of
Authority**, and the **World Model**. This is a targeted tightening, not a rewrite.

**Two things genuinely need correcting** (neither is an architecture flaw):
1. **One philosophy inversion** — `project-plans/00-overview.md` seats the product's "soul" and
   "moat" in the *AI harness*, where the philosophy seats them in the *runtime's ownership of truth*.
2. **A stale, misleading knowledge doc** — `engineering-science/compliance/compliance-report.md` now
   **understates** the shipped code (see Part F); it should not be cited as the current verdict.

---

## Part A · Architecture coherence scorecard (Part IX + Part VI)

| # | Immutable principle | Verdict | Evidence |
|---|---|---|---|
| 1 | Owned executable model, never dead artifact | **Upholds** | ADR-0004 (event log = system of record; state is its fold); ADR-0005 (IRs/artifacts are projections, never rival definitions); vision §10 quarantines vendor formats. |
| 2 | Runtime sovereign; one authority, one write path | **Upholds** | `RuntimeCore::commit` sole mutator; dependency-guard test `kernel_has_no_outward_dependencies` (`eak-runtime/src/lib.rs:32`). |
| 3 | No truth without provenance | **Upholds** | P5 + ADR-0004; every commit carries `ProvenanceLink`s + `Decision{reasoning_call_seq}`; provenance written *with* the change. |
| 4 | Correctness enforced by construction | **Upholds** | Capability seam re-validates; rejected proposal writes nothing; manufacturing gate blocks release on open blocking violations. |
| 5 | Reasoning proposes; deterministic core disposes | **Upholds** | ADR-0002/0006 two-part agent; ADR-0006 explicitly *rejects* letting the reasoning adapter commit. |
| 6 | Determinism and replay inviolable | **Upholds** | ADR-0009; `replay` re-folds without model/clock; byte-identical assertion tests present. |
| 7 | Authority layered, never laundered | **Upholds (structure) / Partial (depth)** | Vision §9 + crosswalk encode the descent; engines deterministic, reasoning walled. Caveat: eng-science thresholds are floored *constants*, not yet typed revisable `Standard` objects. |
| 8 | Physical quantities typed | **Upholds** | ADR-0007; `PhysicalQuantity` = magnitude+unit+tolerance, dimensional analysis at the seam. |
| 9 | Honesty over fabrication | **Upholds (exemplary)** | ADR-0016/0017 *reject* fabricating a provenance spine on import; imported artifacts terminate honestly at "no upstream intent." Strongest coherence signal in the repo. |
| 10 | Objecthood by consequence | **Upholds** | Requirement/Constraint/Net/Decision/Evidence are first-class owned entities. |
| 11 | Humans own goals | **Upholds (model) / gap (dial)** | ADR-0010 "AI proposes, engineer disposes," reversibility precondition; graduated-autonomy dial not yet built (ADR-0015 defers HITL — honestly recorded). |
| 12 | World model = honest deepening approximation | **Partial / trending up** | Physical model has materially advanced (typed `LayerStack`, microstrip widths, clearance + union-find connectivity rules) but is still a *checker* world, not a *generative* one. Honest incompleteness. |

**Division of Authority (Part VI):** upheld in structure (engines never call the model; single reasoning seam), thin in depth (practice enters as constants that "know themselves as practice" only by comment, not by a typed revisable representation).

---

## Part B · Contradictions & tensions (Swarm 8)

**No High-severity contradiction found.** The following are Med/Low and are documentation- or depth-issues, not wrong load-bearing decisions.

- **T-1 (Med) — the compliance report is stale and *undersells* principle #12.** See Part F. It describes the board cross-section as "a bare `layers: u32`" and calls stackup the top gap, but `eak-domain` now ships a typed `Layer{role, copper_thickness, dielectric_height, dielectric_er, loss_tangent}` in an ordered `LayerStack`, and routing computes `microstrip_width(z0, er, h, t)`. **Re-audit needed.**
- **T-2 (Med) — `02-technical-architecture.md` framing lags two canonical rulings.** It still presents interop (KiCanvas/KiCad) as a standing MVP capability (retired by vision §10 → scaffold) and still embeds the ADR-0017-*rejected* "synthesize a minimal intent spine" for import. Framing/factual, not a truth-ownership breach (kernel stays vendor-clean). Already partly covered by `09-legacy-framing-audit.md`.
- **T-3 (Low) — runtime crosswalk overstates coverage.** It labels test-fixture reasoner names as "net classes" and hangs rows on a non-existent high-speed `NetClass` (real enum: {Power, Ground, Signal}). A self-described "binding" honesty doc overstating coverage is a mild principle-#9 risk; its own footnote already flags it.
- **T-4 (Low) — practice-as-constants vs Part VI.** Eng-science floors are hard-coded, not a first-class revisable `Standard`/`Threshold` object with provenance to its clause. No laundering (physics never overridden), but the layer boundary is enforced by convention. Roadmap item, aligns with Part V's "build the internal language."

---

## Part C · Documentation alignment matrix (Swarm 9)

| Doc-area | Verdict | Evidence |
|---|---|---|
| `docs/core/` (runtime, state, reasoning, provenance, determinism) | **ALIGNED** | Sovereignty, single write path, propose/dispose seam, "no fact without provenance," replay — all faithful. |
| `docs/engineering/` (constraint/verification/learning/units) | **ALIGNED** | Deterministic-owns-consequence, typed quantities, continuous checking. |
| `docs/compiler/` | **ALIGNED** | "IR is a projection, never authoritative" = Principle 1 exactly. |
| `engineering-science/` + `runtime-mapping/` | **ALIGNED** | The layer *is* Part VI (physics=laws, math=method, eng-science=practice→rules); honesty contract mirrors Principle 9. |
| `team/` | **ALIGNED** (non-conflict) | Process docs; founder-owns-intent respects Part VI (humans own goals). |
| `docs/foundation/engineering-domain-model.md` | **PARTIAL** | Omits **Assumption, Tradeoff, Risk, Behavior** (Parts II & IV name these load-bearing). |
| `docs/foundation/principles.md` (P1–P13) | **PARTIAL** | Sound subset of Part IX, but frames "owns **knowledge**" (P2) not "owns **truth**"; no objectification / world-model / layered-warrant principle. |
| `docs/foundation/system-overview.md`, `quality-attributes.md` | **PARTIAL** | "mutate/owns **knowledge**" register; correctness-by-construction not tied to "the compiler engineering never had." |
| `docs/foundation/vision.md` | **PARTIAL** | Thesis aligned; softens sovereignty into "AI-native Engineering **IDE**" / "Like Cursor." |
| `project-plans/00-overview.md` (+ 03, 07 derive) | **MISALIGNED** | "soul is a superior AI **harness**"; moat = "kernel + traceability + replay" — inverts the seat of value (see Rec 1–2). |
| `project-plans/02-technical-architecture.md` | **PARTIAL** | Kernel-as-core aligned; "our IP goes into the **harness**" repeats the inversion. |
| `docs/GLOSSARY.md` | **PARTIAL** | No entries for Truth, Sovereignty, Objectification, World Model, Warrant/Authority-layer, Assumption. |

**Corpus scan:** the philosophy's signature vocabulary is essentially absent from the pre-existing corpus (objectification / world-model / division-of-authority / epistemic-truth ≈ 0 files). The corpus is *compatible with but shallower than* the philosophy — it enforces the invariants without naming the frames that justify them.

---

## Part D · Prioritized recommendations (staged — not applied)

**Tier 1 — the one genuine philosophy inversion:**
1. `project-plans/00-overview.md §1` — "soul is a superior AI harness" → the soul is the **sovereign runtime that owns engineering truth**; the AI harness is a bounded reasoning shell admitted only through the seam. (Part III; Vision §9.2)
2. `project-plans/00-overview.md §2` — moat = "kernel + traceability + replay" (mechanism) → moat = **ownership of engineering truth** (intent, constraints, decisions, provenance, memory), which makes AI leverage not liability. (Part III §Knowledge; Part X)
3. `project-plans/02-technical-architecture.md §0` — "IP goes into the harness" → the durable IP is the **owned model + validating seam**; the harness is a replaceable application on the substrate. (Vision §2)

**Tier 2 — elevate four missing frames into the foundation docs:**
4. `docs/foundation/principles.md` P2 — "owns **Knowledge**" → "owns **Truth**"; distinguish truth (owned epistemic object) from knowledge (justified rules atop it). (Part II)
5. Add a principle: **"Authority is layered and never laundered"** (nature → formal method → practice → execution → judgment → purpose). (Part VI; Principle 7)
6. Add the discipline **"objecthood is conferred by consequence, not category"** (the guardrail that keeps the kernel small). (Part IV; Principle 10)
7. Add a framing note that the entity model is the **static projection of an aspirational Engineering World Model** (objects bear behavior; laws derive consequences; fidelity is owned and honest). (Part VII; Principle 12)

**Tier 3 — first-class the missing epistemic objects (domain-model doc, not code yet):**
8. Add **Assumption** as first-class + **dischargeable** (→grounded fact / →enforced constraint / →accepted residual risk) — "the most dangerous objects." (Part II)
9. Add **Tradeoff** (records what was weighed *and rejected*) and **Risk** as Judgment-objects. (Part IV)

**Tier 4 — vocabulary sweep + glossary:**
10. `docs/core/*` + `GLOSSARY.md` — sweep "owns/mutate **knowledge**" → "owns **engineering truth**" (with knowledge as the compounding corpus atop it); add GLOSSARY entries: Engineering Truth, Sovereignty, Objectification, World Model, Division of Authority, Assumption.
11. `docs/foundation/vision.md` — recast the "Like Cursor / Engineering IDE" line to the inverted trust model (runtime owns truth). (Vision §4; overlaps `09` legacy-framing audit.)
12. `docs/foundation/quality-attributes.md`, `system-overview.md` — tie correctness-by-construction to "the compiler engineering never had." (Part VIII)
13. `engineering-science/` READMEs — one line each: this layer holds *contingent, revisable* warrant and may never override physics. (Part VI)

---

## Part E · Not-yet-realized gaps (honest roadmap — NOT contradictions)

These are principles the philosophy asserts that the architecture aspires to but has not yet built. A gap is honest incompleteness; a contradiction is a wrong decision. **All of these are gaps.**

- **G-1 — Generative world model** (Part VII, #12): still a *checker* world; SI/PI/EMC are first-order proxies; no "fidelity-as-first-class-object" yet.
- **G-2 — Engineering memory across projects** (#10; Part III "the moat compounds"): learning engine specced; cross-project accrual is Year 1–3+; local-first vs shared-knowledge tension explicitly unresolved (vision §12.4).
- **G-3 — The engineering language layers** (Part V): internal representation is linguistic in spirit; the eight named facets and a real constraint *calculus* (propagation-to-fixpoint) are deferred — correctly, per Part V.
- **G-4 — Assumption / Tradeoff / Risk as first-class objects** (Parts II & IV): recommended above (Recs 8–9); not yet modeled.
- **G-5 — Graduated autonomy / HITL dial** (#11): model upholds it; the dial isn't built (ADR-0015).
- **G-6 — Determinism under real concurrency / streaming** (#6): holds today (single-threaded write path, one reasoning phase); a watch-item when streaming/multi-window is added.

---

## Part F · Flagged separately — the stale compliance report

Independent of the philosophy, Swarm 8 surfaced a **factual staleness bug** worth its own note:
`engineering-science/compliance/compliance-report.md` describes the physical model as coarse
("bare `layers: u32`", stackup missing, clearance/connectivity DRC absent) — but the shipped code has
**moved past it**: a typed `Layer`/`LayerStack` (`eak-domain/src/lib.rs:~585`), impedance-derived
`microstrip_width()` in routing planning (`~:55`), and `DrcCopperClearanceRule` + a union-find
connectivity/short rule in `eak-engines`. The report now **under-credits** the architecture on
principle #12 and would mislead anyone citing it as the current verdict.
**Recommendation:** re-run/replace the compliance audit against current `eak/` before it is cited
again. (This is a knowledge-doc drift, not an architecture problem — arguably the healthiest kind of
staleness: the code got better than the docs claim.)

---

## Bottom line

- **Architecture:** faithful to the philosophy; 11/12 principles upheld; 0 high-severity
  contradictions; the honesty ADRs are a model of the philosophy in action.
- **Documentation:** aligned in substance, shallow in vocabulary; one real inversion in
  `00-overview.md`; four conceptual frames to elevate; a handful of epistemic objects to add.
- **Nothing here requires re-deciding the architecture.** The change pass is vocabulary/framing +
  small doc additions + a compliance re-audit — staged, not applied, awaiting direction.

*No files were modified in producing this report.*
