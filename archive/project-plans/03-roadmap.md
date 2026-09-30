# Electronics Agent Kit — Roadmap (which Map comes online next)

> Anchored to the canonical stack: `00-product-vision.md` (what), `01-engineering-philosophy.md`
> (why), `02-engineering-world-model.md` (the 46-Map atlas + dependency graph + 4 Bands). **The unit
> of this roadmap is a *Map coming online* — not a week and not a feature.** Growth = transferring
> more of engineering into owned, verified truth. Re-expressed 2026-07-10 from the earlier
> week/feature roadmap (its execution detail is preserved below, recast in Map terms).

**One sentence:** first make the **already-owned skeleton Maps** visible and drivable end-to-end on
one curated example and bring the cheap **Band-A epistemic Maps** online so the reasoning is provably
honest (that is the pre-seed raise); then, funded, bring the runtime's missing Maps online Band by
Band — logical-electrical (V1), behavioral/world-model (V2), lifecycle + Memory (V3) — until the
runtime is the engineering brain a hardware program runs on.

---

## 0. What "a Map comes online" means

Every Map has **two** independent milestones, and the roadmap tracks both:

- **OWNED** — the runtime owns the Map's engineering objects: seam-validated, provenance-bearing,
  verified, replayable. *This is the product growing.* (The philosophy's substrate.)
- **SURFACED** — the Map is visible and drivable through an interface (a panel, the agent chat, a
  rendered view). *This is the demo / UX.* Surfacing is **an interface to the runtime, never the
  runtime.** A Map can be OWNED without being SURFACED (most of today's skeleton), and must never be
  SURFACED without being OWNED (that would be a picture pretending to be truth).

The ordering rule: **OWN first, surface second.** The pre-seed is unusual only because most of its
Maps are *already* owned — so its work is mostly *surfacing the skeleton* plus *owning Band A*. Every
later stage is *owning new Maps*, with surfacing following for whatever that stage must demonstrate.

Reference for every Map name, its objects, and its dependencies: `02-engineering-world-model.md`
(the atlas, the dependency graph, and the four Bands — A epistemic · B logical-electrical · C
physics/behavior · D lifecycle).

---

## 1. The Map arrival sequence (the spine)

| Stage | Horizon | Maps that come **ONLINE (owned)** | Maps **SURFACED** | Stage gate |
|---|---|---|---|---|
| **0 · MVP** | *done* | Intent, Requirement, Constraint(◐), Block(thin), Component, Part/BOM, Net, Placement, Stackup, Routing(tracks), Verification, Decision, Evidence, Traceability, Runtime-State | *(headless)* | Skeleton owned, replayable, seam-validated ✅ |
| **1 · Pre-seed** | ~13 wks | **Band A:** Assumption, Risk, Tradeoff/Objective, Fidelity · deepen Constraint · Change/Revision(v0) | The whole **owned skeleton**, end-to-end on one curated example + Band A visible | Skeleton surfaced end-to-end once; Band-A honesty visible; raise-ready |
| **2 · V1** | funded | **Band B:** Power Domain, Ground/Return, Clock Domain, Pin-Function/Mux, Signal Flow, Interface/Contract, Bus/Protocol, Subsystem (+ Logical-Electrical IR) | Band B on real (small) partner boards | Runtime owns & verifies a power/pin/interface architecture |
| **3 · V2** | funded | **Band C:** Behavior, Power-Integrity, deepen Thermal/SI/EMC, Reliability, Simulation (+ `eak-solvers`) | Predictions/counterfactuals surfaced, fidelity-tagged | Runtime can predict, contradict, counterfactual, explain |
| **4 · V3** | funded | **Band D:** Supply, Cost, Compliance, DFA, Test/Bring-up, Mechanical, graduated Autonomy (+ **`eak-memory`**) | Program-of-record surfaced; Memory begins compounding | Cost/supply/compliance owned truth; Memory accruing |
| **5 · Engineering OS** | long-term | Fidelity(deep), Knowledge/Memory(deep), cross-domain (firmware/mechanical) | Runtime drives end-to-end under supervision | The World Model is deep enough to *drive* |

**Why this order (the dependency logic):**
- **Band A first** because it is *cheap* (domain objects + gates, no heavy numerics) and *trust-defining* — it is what makes the AI demonstrably honest on camera, which is the pre-seed's whole job.
- **Band B next** because it is where the runtime stops "checking a board" and starts "architecting the electronics" — the first *irreplaceable* value, and the layer real engineers actually specify in.
- **Band C** is the world-model leap (checker → generative); it is the deepest moat and the hardest, and it needs the new `eak-solvers` boundary, so it is a funded, deliberate bet.
- **Band D** is the lifecycle and the compounding **Memory** moat; it turns the tool into the program of record.

The Maps come online along the dependency graph in `02-engineering-world-model.md §Part II` — upstream
Maps before the downstream Maps that read them, with the feedback edges (verification→placement,
thermal→placement, supply→part) live from the start.

---

## 2. Stage 1 — Pre-seed (~13 weeks): surface the skeleton, own Band A

Two jobs, run in parallel: **(J1) SURFACE** the already-owned skeleton Maps end-to-end through a
native interface on one curated example; **(J2) OWN** the Band-A epistemic Maps in the kernel so the
demo can *show* honest reasoning (the AI declares and discharges assumptions; every finding carries a
fidelity tag; risks and rejected tradeoffs are recorded, not lost). J1 is the "whoa"; J2 is the moat
made visible.

**Honesty legend** — **REAL** = computed live · **CURATED** = real code, tuned inputs for one example
· **CASSETTE** = recorded LLM output replayed deterministically · **SCAFFOLD** = throwaway demo
scaffold with an expiry (interop/render; per vision §10, never a standing capability) · these labels
appear on every demo claim so live-vs-curated is never blurred.

| Wk | Map milestone | Concrete deliverable | Owned / Surfaced | Honesty |
|---|---|---|---|---|
| 1 | *(substrate)* Runtime-State surfaced | Kernel linked as the native core of a desktop shell; one command round-trips a real kernel result to the view; CI builds the shell. | Surface substrate | REAL plumbing |
| 2 | **Freeze the surfacing seam [GATE]** | Version the event/query contract (the projection of the owned truth to any interface); capture one fixture run's stream; mock player replays it; tag `v1`. | Surfacing contract | REAL contract |
| 3 | Skeleton Maps surfaced (live stream) [GATE] | Stream the 15-phase run (fixture reasoning) through the contract; a live engineering-state / traceability panel folds it. | Surface skeleton | REAL run, fixture reasoning |
| 4 | *Interop scaffold* — import→review parachute | Import a real `.kicad_pcb` → run the owned Verification Map's rules → show findings. **Scaffold, expiry-tagged**: it exists only so a working review exists independent of generation. | Surface (scaffold) | REAL checks · SCAFFOLD render |
| 5 | Reasoning boundary live [GATE] | `live` reasoning adapter behind the port; record cassettes; verify byte-identical replay; timeout/fallback-to-cassette. | (substrate) | REAL live + CASSETTE |
| 6 | **Intent → Requirement → Block** surfaced | Intent chat → requirement + architecture agents, each kernel-validated; Traceability Map fills (sentence→requirement→block). | Surface skeleton | REAL (curated prompt) |
| 7 | **Part/BOM → Net** surfaced | Part selection (catalog-validated) + BOM; netlist; ERC/BOM rules run; traceability links on hover. | Surface skeleton | CURATED gen |
| 8 | **Placement → Routing → Verification** = hero end-to-end [GATE] | Curated placement + constrained route rendered; full review pass with AI explanation + suggested fix, each finding tracing to intent. | Surface skeleton | CURATED board · REAL checks |
| 6–9 | **Band A comes ONLINE** | Kernel gains `Assumption` (dischargeable), `Risk`, `Tradeoff`, and a `Fidelity` tag on derived facts; a release gate blocks on undischarged critical assumptions; the demo surfaces "watch the AI declare & discharge its assumptions." | **Own Band A** + surface | REAL kernel objects |
| 9 | Harden the curated run | Pin seeds/params; byte-reproducible golden run; safe-mode serves the cassette if live wobbles. | — | CURATED + CASSETTE |
| 10 | Package + interface polish | Signed local build; offline path; performance + visual polish; keyboard demo script. | Surface | REAL app |
| 11 | Demo-proof + first signal | 10/10 dry-runs; backup video; landing + waitlist; open design-partner threads. | — | REAL demo |
| 12 | Deck + narrative | Deck (problem · **moat = the runtime owns engineering truth** · demo · market · ask); one-pager; light data room; investor list. | — | — |
| 13 | Outreach + buffer | Send outreach; book meetings; run partner calls; buffer absorbs slippage. | — | — |

**Gates (Map-completion, not calendar):** *W2* the surfacing seam is frozen; *W3* the owned skeleton
streams live; *W5* the reasoning boundary is live + replayable; *W8* the skeleton is surfaced
end-to-end once; **Band A is online by W9.** If a gate is at risk, pull *polish*, never the gate.

**What changed vs the old plan, and why:** the old roadmap *only* surfaced the skeleton. Band A
(owning Assumption/Risk/Tradeoff/Fidelity) is now folded into the pre-seed because it is cheap and it
is the single strongest way to *demonstrate the moat* — "AI you can trust" stops being a claim and
becomes a visible behavior: the runtime refuses to release with an undischarged critical assumption,
and every predicted fact wears its fidelity.

---

## 3. Stage 2 — V1 (funded): the runtime becomes an ARCHITECT (Band B online)

**Maps online:** Power Domain, Ground/Return, Clock Domain, Pin-Function/Mux, Signal Flow,
Interface/Contract, Bus/Protocol, Subsystem — plus a new **Logical-Electrical IR** band between
Engineering IR and Schematic IR (`02 §III Band B`).

**Why now:** this is the layer engineers actually design in, and where the AI delivers its first
*irreplaceable* value — power planning, pin-out/mux solving, interface matching, bus rules. The
product crosses from "checks a board" to "helps architect the electronics." **Stage gate:** the
runtime can own and verify a power/pin/interface architecture for a real (small) partner board, with
every assignment traced to intent and checked by construction (power balance, mux conflict, CDC,
protocol, return-path continuity).

**Also:** onboard the first design partners on real small boards; the interop *scaffold* from W4 is
retired or replaced by owned import-as-translation only if partners need it — never re-elevated to a
standing capability.

---

## 4. Stage 3 — V2 (funded): the runtime becomes a WORLD MODEL (Band C online)

**Maps online:** Behavior (component/subsystem behavior over conditions), Power-Integrity, deepened
Thermal / Signal-Integrity / EMC, Reliability/Failure, Simulation-as-owned-evidence — via the new
outer-ring **`eak-solvers`** crate (deterministic analyses + external-solver ports producing
fidelity-tagged Evidence; heavy numerics stay out of the kernel).

**Why now:** this is the leap from *checker* to *generative* — the runtime can now **predict**,
answer **counterfactuals**, and **explain**, which is the precondition for trustworthy autonomy
(`01 §Part VII`, `02 §Band C`). AI now reasons *against a faithful model it cannot fool.* **Stage
gate:** the runtime answers "what would happen if…" for thermal / SI / PI / EMC on a real board, with
every prediction carrying a declared fidelity, and contradictions detected by construction.

---

## 5. Stage 4 — V3 (funded): the runtime owns the PROGRAM (Band D online)

**Maps online:** Supply Chain, Cost, Compliance/Regulatory, Assembly/DFA, Test/Bring-up,
Mechanical/Enclosure, graduated Authority/Autonomy — and stand up **`eak-memory`**, the knowledge
tier: the compounding cross-project Engineering Memory begins accruing (`01 §Part III`, `02 §Part VI`).

**Why now:** real engineering is a lifecycle; the value (cost/supply/compliance) and the *moat that
compounds* (Memory) live here. **Stage gate:** cost, supply, and compliance are owned truth (not
spreadsheets); Change/Revision gives git-for-hardware; Memory is measurably making the next design
start smarter than the last. Design partners move to paid; onboarding no longer requires the founder
in the room.

**The Memory tension to design here, not default:** private / organizational / universal tiers, with
every cross-tier crossing opt-in and privacy-preserving (`02 §Part VI`). This is both the biggest
asset and the biggest trust risk — build it deliberately.

---

## 6. Stage 5 — Long-term: the Engineering Operating System

The World Model is deep enough that the runtime can **drive** end-to-end under supervision — propose
intent decompositions, plan power/pins/interfaces, place and route with behavioral awareness, predict
and verify against a high-fidelity model, and justify every step back to intent. **Memory** becomes an
industry-scale, tiered, provenance-linked corpus. The model generalizes **beyond the board** toward
system co-design (firmware, mechanical, thermal). At this point EAK is not a PCB tool with AI — it is
the **owned engineering brain** every hardware program runs on, and the board is one of its outputs.

---

## 7. Critical path & gates (as Map gates)

The load-bearing gates, each a Map milestone rather than a date:

1. **Surfacing seam frozen (pre-seed W2).** Every interface is a projection of owned truth; freeze
   and version this once. Re-opening it is the most expensive mistake available.
2. **Owned skeleton streams live (pre-seed W3).** Proves the runtime can drive any interface from its
   event stream. Until this is real, surfacing is theory.
3. **Reasoning boundary live + replayable (pre-seed W5).** The seam that makes "AI you can trust"
   literal (propose-then-dispose, recorded, byte-identical replay).
4. **Skeleton surfaced end-to-end + Band A online (pre-seed W8–W9).** The raise rests on the "whoa"
   *and* on the visible honesty Band A provides.
5. **Each later Band's stage gate (V1→V3)** as in §§3–5: architect (B) → world model (C) → program +
   Memory (D). A Band is not "done" until its Maps are OWNED (seam-validated, provenance-bearing,
   verified), not merely surfaced.

**Rule:** a Map counts as online only when OWNED. A surfaced-but-not-owned Map is a picture
pretending to be truth — forbidden by Principle 1.

---

## 8. Parallelization (who owns *owning* vs *surfacing*)

The frozen surfacing seam (§7.1) converts a solo founder into two tracks:

| Track | Owner | Builds | The Map work it does |
|---|---|---|---|
| **The runtime (owning)** | **Founder** (deep, correctness-critical) | Kernel objects, capability seam, IR bands, reasoning boundary, Band-A objects, later `eak-solvers` / `eak-memory`. | Brings Maps **ONLINE** — the product. |
| **The interface (surfacing)** | **AI agents**, founder-reviewed | Panels, agent chat, traceability viz, review panel, the rendered view — all against the mock stream. | **SURFACES** owned Maps — the demo. |
| **Interop scaffold** | Agents (integration only) | Import→translate + a rendered view, **expiry-tagged**. | Surfacing scaffold only; never a Map the product owns. |
| **Fundraise** | Founder (+ agents for copy/design) | Deck, one-pager, landing, waitlist, video. | Narrates the owned Maps. |

**Why it works:** the founder's scarce time goes to *owning* (the moat); the mockable *surfacing*
surface is delegated. The seam is what makes surfacing a merge, not a rewrite.

---

## 9. Contingency (the demo ships even if generation disappoints)

The demo is non-negotiable; generation fidelity is the adjustable variable. Descend only as far as
needed, keeping the layer below live.

| If behind on… | Substitute | Result |
|---|---|---|
| Live generation flaky (W6–W8) | Replay a **cassette** of a good run. | Real reasoning + real checks, not sampled live. |
| Assisted placement/route won't converge (W8) | **Pre-baked** curated board rendered. | Board still appears, checked + explained + traced. |
| Generation not demo-safe (by W9) | Lead with the **interop-scaffold import → review** parachute; present generation as "streaming preview." | The owned Verification Map carries the demo. |
| Band A slips | Ship Assumption + Fidelity only (drop Tradeoff/Risk surfacing to V1). | The core honesty beat still lands. |
| Packaging slips (W10) | Demo from the dev machine; ship the **video** as the primary artifact. | No dependence on a distributable build. |
| Time collapses | Demo = **owned skeleton surfaced on the curated example + Band-A honesty**, plus the Map roadmap as the vision. | Still fundable: trustworthy owned truth today; the Map arc as the plan. |

**Non-negotiables (never cut):** the runtime doing **real** checks over **owned** truth; **traceability**
to intent; a demo that runs the same way every time; an honest line between REAL / CURATED / CASSETTE
/ SCAFFOLD (investors forgive curation, not deception).

---

## 10. Operating rhythm & how to read this

- **The unit of progress is a Map coming online** (OWNED > SURFACED). Each week, name which Map you are
  owning and which you are surfacing.
- **Monday:** pick the current stage's next Map (§1 spine); split founder=owning / agents=surfacing (§8).
- **Friday:** gate check — is the Map OWNED (seam-validated, provenance-bearing, verified) or only
  surfaced? Record a short clip of whatever Map newly lit up (proof + deck fuel).
- **Guardrails:** freeze the surfacing seam once (W2); keep the golden run replayable from W5; never
  surface a Map that isn't owned; keep the interop parachute *labeled as scaffold* and building until
  generation is demo-safe, then retire it.

> The ticket-level breakdown lives in `07-engineering-backlog.md` — which should likewise be
> re-expressed as "which Map's objects/rules/IR this ticket brings online." This roadmap is the
> Map-level intent; the backlog is the object-level work.

### Reading it Monday morning
1. §1 tells you which Map is next to come online, and whether the job is *own* or *surface*.
2. §7 tells you if it's on the critical path (protect the Map gate above all).
3. §8 tells you what to hand to agents (surfacing) vs keep yourself (owning).
4. If behind, §9 tells you what to cut — **top of the ladder first, the demo always ships, and a Map
   is never surfaced before it is owned.**
