# Electronics Agent Kit — The Engineering World Model & Map Atlas (CANONICAL)

> **Priority 3 in the repository**, beneath `00-product-vision.md` (what) and
> `01-engineering-philosophy.md` (why). This document is the **blueprint of what the runtime must
> eventually own** — the concrete operationalization of the philosophy's Part VII. It defines the
> *Maps* that compose the Engineering World Model, their dependency graph, the gap between them and
> today's runtime, the object meta-model (Engineering Knowledge Graph), the Engineering Memory, and
> a six-stage roadmap.
>
> **This is an architecture and systems-thinking artifact. It contains no code and mandates no code
> change.** It challenges assumptions and reasons from first principles. Where a lower document
> describes *how to build a slice*, it remains valid; where it contradicts the ownership model
> defined here, this document wins. Written 2026-07-10, grounded in a fresh inventory of the current
> `eak/` runtime.

---

## Part 0 · What a "Map" is (and what it is not)

A **Map** is a coherent, owned sub-model of the Engineering World Model: a set of engineering
*objects* + their *relationships* + their *state* + the *constraints* over one engineering concern.
"Power," "Thermal," "Traceability," "Intent" are Maps.

Three first-principles claims that shape everything below — and each **challenges an assumption**:

1. **A Map is a lens, not a silo.** *Challenged assumption:* "each map is its own database/tool."
   All Maps are views over **one** owned truth-graph. A `Net` appears in the Connectivity Map, the
   Power Map, the Signal-Integrity Map, and the Thermal Map — it is the *same object*, seen through
   different projections. Maps never own private copies; they own *perspectives* and the objects
   unique to their concern. This is the difference between an integrated world model and a bag of
   disconnected EDA tools.

2. **The World Model is a graph, not a pipeline.** *Challenged assumption:* "engineering is the
   linear 15-phase flow Intent→…→Release." That flow is a *traversal order* the orchestrator happens
   to run; it is **not** the model. The real structure is a directed graph with dense feedback: a
   thermal finding mutates placement, a supply-chain event mutates part selection, a cost ceiling
   mutates architecture. The pipeline is one path through the graph; the graph is the truth.

3. **The board is not the terminal object.** *Challenged assumption:* "this is about producing a
   PCB." The PCB is one *realization* Map. The World Model spans intent, behavior, cost, risk,
   supply, compliance, and — ultimately — beyond the board toward system/firmware/mechanical
   co-design. The runtime owns the *engineering*; the board is an output it emits.

**The per-Map schema.** Every Map below is specified against the twelve attributes the exercise
requires, rendered compactly in four lines:
- **Purpose & why it exists**
- **Objects it introduces · Owner · Inputs → Outputs**
- **Relationships · Runtime representation**
- **AI interaction · Verification strategy · How it evolves / future scalability**

**Ownership legend (who owns it).** Unless stated, *every* Map's truth is owned by the **deterministic
runtime**; the split within a Map is always the same philosophy invariant — **the runtime owns the
objects and their validation; AI may only propose into them through the seam; deterministic
algorithms compute anything with a right answer; humans own the goals.** Where a Map has a notable
AI or human role, the fourth line says so.

**Status legend:** ✅ owned today · ◐ partially owned · ○ not yet owned (missing).

---

## Part I · The Map Atlas

A master table first, then compact per-Map specifications by tier.

| # | Map | Tier | Status | One-line purpose |
|---|-----|------|--------|------------------|
| 1 | Engineering Truth | 0 meta | ◐ | The totality of owned, justified, provenance-bearing truth. |
| 2 | Runtime State | 0 meta | ✅ | The current state as the fold of the fact log. |
| 3 | Traceability / Provenance | 0 meta | ✅ | The lineage graph binding every object to intent. |
| 4 | Change / Revision | 0 meta | ◐ | Versions, branches, diffs — "git for hardware." |
| 5 | Authority / Autonomy | 0 meta | ◐ | Who/what may act, and at what autonomy level. |
| 6 | Fidelity | 0 meta | ○ | How well the World Model models reality, per concern. |
| 7 | Knowledge / Memory | 0 meta | ○ | Compounding cross-project engineering knowledge. |
| 8 | Intent | 1 purpose | ✅ | Human goals that seed everything. |
| 9 | Requirement | 1 purpose | ✅ | Testable, dimensioned obligations from intent. |
| 10 | Assumption | 1 purpose | ○ | Unverified premises; dischargeable; the dangerous ones. |
| 11 | Objective / Tradeoff | 1 purpose | ○ | Competing objectives and the weighed-and-rejected space. |
| 12 | Constraint | 1 purpose | ◐ | Enforceable bounds (physical, regulatory, purposive). |
| 13 | Architecture / Block | 2 logic | ◐ | Functional decomposition into blocks. |
| 14 | Subsystem | 2 logic | ○ | Hierarchical grouping + subsystem boundaries. |
| 15 | Interface / Contract | 2 logic | ○ | Ports, protocols, handshakes between blocks. |
| 16 | Signal Flow | 2 logic | ○ | Directional logical signal flow (distinct from copper). |
| 17 | Bus / Protocol | 2 logic | ○ | Bus topologies (I²C/SPI/USB/…) and their rules. |
| 18 | Net / Connectivity | 3 electrical | ✅ | Electrical connectivity graph. |
| 19 | Power Domain | 3 electrical | ○ | Rails, domains, budgets, sequencing. |
| 20 | Ground / Return | 3 electrical | ○ | Return paths and reference structure. |
| 21 | Clock Domain | 3 electrical | ○ | Clocks, domains, crossings, timing. |
| 22 | Pin-Function / GPIO / Mux | 3 electrical | ○ | Pin capability + function assignment / muxing. |
| 23 | Component | 4 realization | ✅ | Realized components. |
| 24 | Part | 4 realization | ✅ | Chosen parts (MPN, lifecycle) + BOM. |
| 25 | Package / Footprint | 4 realization | ◐ | Physical package + land pattern. |
| 26 | Placement | 4 realization | ✅ | Component positions on the board. |
| 27 | Stackup | 4 realization | ✅ | Layer build-up (copper/dielectric). |
| 28 | Copper / Routing | 4 realization | ◐ | Tracks (+ vias, zones — vias/zones missing). |
| 29 | Mechanical / Enclosure | 4 realization | ○ | Outline, mounting, 3D/enclosure fit. |
| 30 | Thermal | 5 physics | ◐ | Heat sources, dissipation, junction temps. |
| 31 | Signal Integrity | 5 physics | ◐ | Impedance, reflection, crosstalk, timing margins. |
| 32 | Power Integrity | 5 physics | ○ | PDN, decoupling, rail sag, transient response. |
| 33 | EMC / EMI | 5 physics | ◐ | Emissions, susceptibility, radiating structures. |
| 34 | Reliability / Failure | 5 physics | ○ | Failure modes (FMEA), derating, MTBF. |
| 35 | Behavior | 5 physics | ○ | Component/subsystem behavioral models over conditions. |
| 36 | Verification | 6 verify | ✅ | Rules, checks, violations, waivers. |
| 37 | Simulation / Analysis | 6 verify | ○ | Simulation runs as owned, evidence-producing objects. |
| 38 | Test / Bring-up | 6 verify | ○ | DFT, test coverage, bring-up procedure. |
| 39 | Manufacturing / DFM | 7 production | ◐ | Fab process capability + DFM constraints. |
| 40 | Assembly / DFA | 7 production | ○ | Assembly process + DFA constraints. |
| 41 | Supply Chain | 7 production | ○ | Sourcing, lead time, alternates, lifecycle risk. |
| 42 | Cost | 7 production | ○ | BOM cost, NRE, cost drivers. |
| 43 | Compliance / Regulatory | 7 production | ○ | Standards, certifications, evidence of compliance. |
| 44 | Decision | 8 reasoning | ✅ | Choices with rationale, alternatives, confidence. |
| 45 | Evidence | 8 reasoning | ✅ | The justification substrate. |
| 46 | Risk | 8 reasoning | ○ | Tracked liabilities, mitigations, residuals. |

**Coverage summary:** of 46 derived Maps, **~13 are owned, ~9 partial, ~24 missing.** The runtime
today owns the *skeleton* (intent → requirements → blocks → components → nets → board → verify →
manufacturing IR) and almost none of the *behavioral, lifecycle, and governance* flesh. That gap is
the product roadmap.

### Tier 0 — Meta / substrate Maps

**1. Engineering Truth Map** ◐
- *Purpose & why:* the union of all owned justified truth; the thing the product *is*. Exists because truth scattered is truth destroyed (philosophy Part II).
- *Objects · owner · in→out:* no objects of its own — it is the closure over every other Map's objects. Owner: runtime. In: every committed fact. Out: the queryable whole.
- *Relationships · runtime rep:* the transitive graph of all objects + provenance edges. Today: the `EngineeringState` fold + event log.
- *AI · verify · evolves:* AI reads it, never owns it. Verified by construction (nothing enters unvalidated). Scales as every other Map matures.

**2. Runtime State Map** ✅
- *Purpose & why:* the authoritative *present*, derived from the fact log. Exists so the present is an entailment, never a free-floating assertion.
- *Objects · owner · in→out:* `EngineeringState`. Owner: runtime (sole mutator). In: `EventRecord`s. Out: current object collections.
- *Relationships · runtime rep:* fold of the append-only log; single commit path (stamp→append→fold).
- *AI · verify · evolves:* AI never writes it directly. Determinism/replay verified by byte-identical re-fold. Scales via richer events as Maps are added.

**3. Traceability / Provenance Map** ✅
- *Purpose & why:* the lineage binding every object to its evidence and to intent. Exists because "no fact without provenance" (Principle 3); the answer to "why is this here?"
- *Objects · owner · in→out:* `ProvenanceLink{from,to,relation}`, `RelationType`. Owner: runtime. In: every commit's links. Out: forward/backward trace, impact set.
- *Relationships · runtime rep:* a directed edge overlay across all objects; today `ProvenanceLink` + `trace_cmd`.
- *AI · verify · evolves:* AI consumes traces to explain; never fabricates them. Verified by referential integrity at the seam. Scales to full impact-analysis ("what breaks if this spec changes?").

**4. Change / Revision Map** ◐
- *Purpose & why:* versions, branches, diffs, merges of the design — "git for hardware" (ADR-0008). Exists so engineering is explorable and comparable across alternatives, not a single mutable present.
- *Objects · owner · in→out:* (missing) `Revision`, `Branch`, `Diff`, `MergeDecision`. Owner: runtime. In: the event log ranges. Out: semantic diffs, branch state.
- *Relationships · runtime rep:* today the log gives linear history + replay; branching/semantic-diff objects are not built.
- *AI · verify · evolves:* AI proposes and explains diffs; the runtime computes them deterministically. Verified by replay equivalence. Scales to design-space exploration (many branches compared by objective).

**5. Authority / Autonomy Map** ◐
- *Purpose & why:* who/what may act on which objects, at what autonomy level; the locus of "humans own goals." Exists to make human-in-the-loop a first-class, tunable property (ADR-0010).
- *Objects · owner · in→out:* `Autonomy` (mode today); (missing) per-object authority/approval objects. Owner: runtime + human. In: policy + human dispositions. Out: gate decisions.
- *Relationships · runtime rep:* today a global `Autonomy::{Autonomous,Supervised}` flag on the core.
- *AI · verify · evolves:* AI acts only within granted authority; humans grant/deny. Verified by seam checks on autonomy. Scales to graduated, per-concern autonomy dials (ADR-0015 deferred).

**6. Fidelity Map** ○ *(new; philosophy §12/Part VII)*
- *Purpose & why:* how faithfully each behavioral Map models reality, made explicit. Exists so the World Model can say "I do not model EMI yet" — honesty about its own reach (Principle 12).
- *Objects · owner · in→out:* (missing) `ModelFidelity{concern, method, confidence, scope}`. Owner: runtime. In: which solver/approximation produced a result. Out: trust-weighting on every derived fact.
- *Relationships · runtime rep:* attaches to every behavioral result (thermal, SI, PI…) and to `Evidence.reliability`.
- *AI · verify · evolves:* AI must respect fidelity (never over-trust a stub). Verified by requiring a fidelity tag on derived facts. Scales as solvers deepen — each increment raises a declared fidelity.

**7. Knowledge / Memory Map** ○ *(the Engineering Memory — see Part VI)*
- *Purpose & why:* compounding, cross-project engineering knowledge — the moat that grows (philosophy Part III §Knowledge).
- *Objects · owner · in→out:* (missing) reusable `Pattern`, `Lesson`, `ComponentBehaviorRecord`, `FailureRecord`, `Heuristic`. Owner: runtime (knowledge tier). In: every completed, verified design. Out: priors, warnings, suggested defaults.
- *Relationships · runtime rep:* built atop provenance + evidence; not yet realized.
- *AI · verify · evolves:* AI queries memory to reason; the runtime curates what is admitted (validated, not gossip). Verified by tying each memory item to the evidence that earned it. Scales into the central long-term asset.

### Tier 1 — Intent & Purpose Maps

**8. Intent Map** ✅
- *Purpose & why:* capture human purpose as the root of all provenance. Exists because intent is the "why" that outlives the "what."
- *Objects · owner · in→out:* `DesignIntent`. Owner: human (source) + runtime (custody). In: natural-language goal. Out: the provenance root.
- *Relationships · runtime rep:* head of every trace; `IntentCaptured` event.
- *AI · verify · evolves:* AI helps *structure* intent (proposes), never invents it (honesty). Verified as the mandatory root of requirements. Scales to multi-stakeholder, prioritized, conflicting intents.

**9. Requirement Map** ✅
- *Purpose & why:* testable, dimensioned obligations derived from intent. Exists to make purpose measurable.
- *Objects · owner · in→out:* `Requirement` (+category, status, priority, `targets: PhysicalQuantity`). Owner: runtime. In: intent. Out: constraints + acceptance criteria.
- *Relationships · runtime rep:* `DerivedFrom` intent; justified by evidence; realized by blocks. `RequirementCommitted`.
- *AI · verify · evolves:* AI proposes candidate requirements (the live reasoning phase); kernel validates. Verified: every Accepted requirement testable (P13). Scales to requirement conflict detection, negotiation.

**10. Assumption Map** ○ *(new; philosophy Part II — "the most dangerous objects")*
- *Purpose & why:* make unverified premises first-class and **dischargeable**. Exists because most failures are unexamined assumptions true in the lab, false in the field.
- *Objects · owner · in→out:* (missing) `Assumption{statement, rests-on, status}`, dischargeable → grounded fact / enforced constraint / accepted risk. Owner: runtime. In: any reasoning step that presumes. Out: a tracked debt with a due date.
- *Relationships · runtime rep:* attaches to Decisions and Requirements; blocks release while undischarged-and-load-bearing.
- *AI · verify · evolves:* AI must *declare* its assumptions as objects (not bury them). Verified by a gate: no undischarged critical assumption at release. Scales to automated assumption surfacing across the design.

**11. Objective / Tradeoff Map** ○ *(new)*
- *Purpose & why:* represent competing objectives (cost↔performance↔size↔reliability) and the weighed-and-rejected space. Exists because engineering is tradeoffs, and the road not taken must be preserved.
- *Objects · owner · in→out:* (missing) `Objective`, `Tradeoff{alternatives, criteria, chosen, rejected, rationale}`. Owner: human (weights) + runtime (record). In: intent priorities. Out: a Pareto/decision frontier feeding Decisions.
- *Relationships · runtime rep:* links Objectives to Decisions and Requirements.
- *AI · verify · evolves:* AI proposes and scores alternatives; humans own the value weights (Principle 11). Verified by tying each Decision to the Tradeoff it resolved. Scales to design-space exploration + multi-objective optimization.

**12. Constraint Map** ◐
- *Purpose & why:* the enforceable bounds reality and intent impose. Exists so bounds are law, not advice.
- *Objects · owner · in→out:* `Constraint{kind, bound: PhysicalQuantity, subject}` (+ Fabrication floor). Owner: runtime. In: requirements, standards, physics. Out: the predicate set verification ranges over.
- *Relationships · runtime rep:* `Constraint` over subjects; consumed by rules. Today limited kinds (Max/Min/Equal).
- *AI · verify · evolves:* AI proposes constraints; kernel enforces. Verified by `constraint-consistency`. Scales to a real **constraint calculus** (propagation to fixpoint; typed, revisable `Standard` objects rather than hard-coded floors).

### Tier 2 — Architecture & Logic Maps

**13. Architecture / Block Map** ◐
- *Purpose & why:* functional decomposition of the design into blocks realizing requirements.
- *Objects · owner · in→out:* `FunctionalBlock{name, function, requirements}`. Owner: runtime. In: requirements. Out: the frame components are minted into.
- *Relationships · runtime rep:* block `Supports` requirements; components `RealizeComponent` from blocks. Thin today (no ports/interfaces/hierarchy).
- *AI · verify · evolves:* AI proposes decomposition; kernel validates ≥1 requirement per block. Verified by block→requirement coverage. Scales to hierarchical architecture with interfaces (Maps 14–15).

**14. Subsystem Map** ◐
- *Purpose & why:* hierarchy and boundaries above the flat block list; the unit of reuse and reasoning at scale.
- *Objects · owner · in→out:* `Subsystem{name, blocks, interfaces, boundary}` (implemented Band B inc 8). Owner: runtime. In: blocks. Out: composable, reusable units.
- *Relationships · runtime rep:* subsystems contain blocks and expose interfaces; boundary completeness checked at ERC.
- *AI · verify · evolves:* AI proposes groupings; runtime validates boundaries. Verified by interface completeness (`erc-subsystem-boundary` now; full cross-boundary pin check when net→pins lands). Scales to cross-project subsystem reuse (feeds Memory).

**15. Interface / Contract Map** ○ *(new)*
- *Purpose & why:* the ports, protocols, and handshakes between blocks/subsystems — where integration fails.
- *Objects · owner · in→out:* (missing) `Interface{ports, protocol, direction, electrical/logical contract}`. Owner: runtime. In: block boundaries. Out: connectivity + protocol constraints.
- *Relationships · runtime rep:* interfaces bind block ports to nets/buses; enforce protocol rules.
- *AI · verify · evolves:* AI proposes interface matches; runtime checks contract compatibility. Verified by interface-contract rules (voltage-level, protocol, direction). Scales to automatic subsystem interconnect + ERC-by-contract.

**16. Signal Flow Map** ◐
- *Purpose & why:* directional *logical* signal flow (source→sink), distinct from undirected copper. Exists because a Net says "connected"; a Signal says "flows from here to there, and means this."
- *Objects · owner · in→out:* `Signal{name, source, sinks, semantics}` (implemented Band B inc 5; `direction` is encoded by source→sinks topology). Owner: runtime. In: interfaces + pins. Out: the logical layer nets realize.
- *Relationships · runtime rep:* `Signal` realized-by `Net`; carried-by pins.
- *AI · verify · evolves:* AI infers flow; runtime validates against pin electrical types. Verified by driver/sink consistency (extends ERC, `erc-signal-driver-sink` now; `signal→Net` realization check later). Scales to full behavioral signal-flow simulation.

**17. Bus / Protocol Map** ◐
- *Purpose & why:* bus topologies (I²C/SPI/USB/CAN…) and their structural rules (addressing, termination, fan-out).
- *Objects · owner · in→out:* `Bus{name, contract, members, topology}` (implemented Band B inc 7). Owner: runtime. In: interfaces/signals. Out: bus-level constraints (pull-ups, unique addresses, stubs).
- *Relationships · runtime rep:* a `Bus` groups interfaces under a protocol contract with a declared topology.
- *AI · verify · evolves:* AI recognizes/proposes bus structure; runtime enforces protocol rules. Verified by protocol-specific checks (e.g., I²C address collision via `erc-bus-topology` now; full protocol knowledge library later). Scales to a protocol knowledge library (Memory).

### Tier 3 — Electrical-domain Maps

**18. Net / Connectivity Map** ✅
- *Purpose & why:* the electrical connectivity graph — the bridge from logical intent to physical copper.
- *Objects · owner · in→out:* `Net{members, class, current?, impedance_target?, origin}`, `Pin`. Owner: runtime. In: pins + signals. Out: the graph routing realizes and DRC checks.
- *Relationships · runtime rep:* nets contain pins; realized by tracks; `NetCommitted`/`PinCommitted`.
- *AI · verify · evolves:* AI proposes connectivity; kernel validates membership. Verified by `drc-unrouted-net`/`drc-net-open`. Scales as nets gain domain roles (power/clock/bus).

**19. Power Domain Map** ○ *(new; power is implicit in `NetClass::Power` today)*
- *Purpose & why:* rails, power domains, budgets, and sequencing — the backbone every board lives or dies on.
- *Objects · owner · in→out:* (missing) `PowerDomain{rail voltage, tolerance, budget, sources, loads, sequence}`. Owner: runtime. In: requirements (power budget), parts (consumption). Out: PDN + budget constraints.
- *Relationships · runtime rep:* domains span nets/pins/components; drive PI and thermal.
- *AI · verify · evolves:* AI proposes domain structure; runtime computes budgets deterministically. Verified by power-balance + sequencing rules. Scales to full PDN + sequencing simulation (Map 32).

**20. Ground / Return Map** ◐
- *Purpose & why:* the reference/return structure — the single most under-modeled cause of SI/EMC failure ("current returns; where?").
- *Objects · owner · in→out:* `ReturnPath` (implemented Band B inc 3; `ReferencePlane` still missing). Owner: runtime. In: stackup + nets. Out: return-path constraints for SI/EMC.
- *Relationships · runtime rep:* `ReturnPath` couples a controlled net to the reference net its return current flows on; full cross-stackup adjacency still needs the PCB-IR reference-adjacency model.
- *AI · verify · evolves:* AI flags reference discontinuities; runtime computes return geometry. Verified by return-path continuity rules (`erc-return-path-required` now; reference-continuity geometry when the adjacency model lands). Scales to field-solved return analysis.

**21. Clock Domain Map** ◐
- *Purpose & why:* clocks, their domains, crossings, and timing budgets.
- *Objects · owner · in→out:* `ClockDomain{frequency, source, members}` (implemented Band B inc 2; crossings flag CDC concerns). Owner: runtime. In: components/signals. Out: timing + SI constraints (length matching, skew).
- *Relationships · runtime rep:* domains span signals/nets; `erc-clock-domain-conflict` flags a net in ≥2 domains as a crossing.
- *AI · verify · evolves:* AI proposes domain assignment; runtime checks crossings/timing. Verified by skew/length-match rules. Scales to static timing over the board.

**22. Pin-Function / GPIO / Mux Map** ◐
- *Purpose & why:* a pin's *capabilities* and its *assigned function* — the MCU/FPGA pin-planning problem, a top real-world pain point.
- *Objects · owner · in→out:* `PinCapability` (implemented Band B inc 4), `PinAssignment` (implemented Band B inc 4). Owner: runtime. In: part datasheets (capabilities) + intent (needed functions). Out: a validated pin-out.
- *Relationships · runtime rep:* assignments bind functions to `Pin`s under mux constraints.
- *AI · verify · evolves:* AI proposes pin assignments (a genuinely hard search); runtime validates against capability + mutual-exclusion rules (`erc-pin-mux-conflict` + `erc-pin-capability` now; electrical-type matrix later). Scales to constraint-solved auto pin-assignment.

### Tier 4 — Realization Maps

**23. Component Map** ✅ — *Purpose:* realized components. *Objects:* `Component{refdes,class,value?,origin}` (runtime). *Rel/rep:* from blocks; bears pins; `ComponentCommitted`. *AI/verify/scale:* AI proposes; kernel validates refdes/pins/origin; scales to behavioral models (Map 35).

**24. Part Map (+ BOM)** ✅ — *Purpose:* the specific parts chosen + bill of materials. *Objects:* `Part{mpn,manufacturer,lifecycle,datasheet}`, `BomLineItem`, `PartCatalog` (runtime; AI proposes MPN, catalog validates). *Rel/rep:* parts source components; `PartCommitted`. *AI/verify/scale:* hallucinated MPN rejected; scales to supply/cost Maps (41–42) + datasheet knowledge (Memory).

**25. Package / Footprint Map** ◐ — *Purpose:* physical package + land pattern. *Objects:* footprint geometry (imported; no first-class `Package` beyond placement extent). *Rel/rep:* binds part→pads→pins. *AI/verify/scale:* AI proposes footprints; runtime validates pad↔pin; scales to a verified footprint/land-pattern library (Memory).

**26. Placement Map** ✅ — *Purpose:* component positions. *Objects:* `Placement{x,y,w,h,side}` (runtime). *Rel/rep:* component on board; `PlacementCommitted`. *AI/verify/scale:* AI/curated placement proposes; DRC validates (`drc-out-of-bounds`, `drc-courtyard-overlap`); scales to thermal/SI-aware placement.

**27. Stackup Map** ✅ — *Purpose:* the vertical build-up. *Objects:* `LayerStack`, `Layer{role,copper_thickness,dielectric_height,er,loss_tangent}` (runtime). *Rel/rep:* layers ordered; referenced by SI/PI/thermal. *AI/verify/scale:* proposed then validated (ε_r≥1 etc.); scales to impedance-controlled stackup synthesis.

**28. Copper / Routing Map** ◐ — *Purpose:* the realized copper. *Objects:* `Track` (runtime); **missing** `Via`, `Zone/Pour`, `Teardrop`. *Rel/rep:* tracks realize nets across layers; `TrackCommitted`. *AI/verify/scale:* curated/assisted routing proposes; DRC validates (width/clearance/ampacity/impedance); scales to vias/zones + full assisted routing.

**29. Mechanical / Enclosure Map** ○ *(new)* — *Purpose:* outline, mounting, height, 3D/enclosure fit — where electrical meets the physical product. *Objects:* (missing) `Outline`, `MountingHole`, `KeepoutVolume`, `EnclosureFit`. *Owner:* runtime + human/mechanical. *Rel/rep:* constrains placement/routing in 3D. *AI/verify/scale:* AI proposes; runtime checks fit/collision; scales to full ECAD-MCAD co-design.

### Tier 5 — Physics & Behavior Maps

**30. Thermal Map** ◐ — *Purpose:* heat sources, dissipation paths, junction temperatures. *Objects:* `thermal-tj` rule + `ThermalResistance` unit; **missing** owned `HeatSource`, `ThermalPath`, `ThermalNode`. *Rel/rep:* couples power (dissipation) to placement/stackup (spreading). *AI/verify/scale:* AI flags hotspots; runtime computes T_j deterministically; scales from 1-node θ_JA to a thermal network / field solve (fidelity-tagged).

**31. Signal Integrity Map** ◐ — *Purpose:* impedance, reflection, crosstalk, timing margin. *Objects:* `impedance_target` on nets + `drc-impedance-match` (microstrip); **missing** owned `TransmissionLineModel`, `CrosstalkPair`. *Rel/rep:* couples net geometry + stackup + return path. *AI/verify/scale:* AI flags risks; runtime computes impedance; scales to field-solved SI (fidelity-tagged).

**32. Power Integrity Map** ○ *(new)* — *Purpose:* PDN impedance, decoupling, rail sag, transient response. *Objects:* (missing) `PdnModel`, `DecouplingNetwork`, `TransientEvent`. *Owner:* runtime. *Rel/rep:* couples power domains + stackup + placement. *AI/verify/scale:* AI proposes decoupling; runtime computes PDN impedance; scales to frequency-domain PDN analysis.

**33. EMC / EMI Map** ◐ — *Purpose:* emissions, susceptibility, radiating structures. *Objects:* `emc-antenna-length` rule; **missing** owned `RadiatingStructure`, `CoupledPath`. *Rel/rep:* couples nets/return/clock domains. *AI/verify/scale:* AI flags antennas/loops; runtime computes λ/10 etc.; scales to coupling models + pre-compliance estimation.

**34. Reliability / Failure Map** ○ *(new)* — *Purpose:* failure modes, derating, MTBF (FMEA as a living object). *Objects:* (missing) `FailureMode`, `Derating`, `ReliabilityBudget`. *Owner:* runtime. *Rel/rep:* attaches to components/parts/domains; feeds Risk. *AI/verify/scale:* AI proposes failure modes from Memory; runtime enforces derating rules; scales to full FMEA + field-return learning.

**35. Behavior Map** ○ *(new; the heart of the "world model vs knowledge graph" leap)* — *Purpose:* how components/subsystems *behave* over conditions/time (not just what they are). *Objects:* (missing) `BehaviorModel{function-over-conditions}`. *Owner:* runtime (curated) + AI (proposed from datasheets). *Rel/rep:* attaches to components; consumed by all physics Maps and simulation. *AI/verify/scale:* AI extracts behavior from datasheets (proposes); runtime validates against measured/simulated evidence; scales to the generative core that lets the runtime *predict* (philosophy Part VII).

### Tier 6 — Verification & Analysis Maps

**36. Verification Map** ✅ — *Purpose:* rules, checks, results. *Objects:* `Violation`, `Waiver`, 17 `Rule`s (runtime; deterministic). *Rel/rep:* rules range over objects; violations link subjects→requirements; `ViolationRaised`. *AI/verify/scale:* AI *explains* violations (advisory); rules are deterministic; scales as every physics Map adds rules; the manufacturing gate blocks release on open blocking violations.

**37. Simulation / Analysis Map** ○ *(new)* — *Purpose:* simulation runs as owned, evidence-producing objects (SPICE/SI/PI/thermal/EMC). *Objects:* (missing) `SimulationRun{model, inputs, results, fidelity}`. *Owner:* runtime (owns the run + result as evidence); external solvers behind a port. *Rel/rep:* results become `Evidence` with a fidelity tag. *AI/verify/scale:* AI sets up/interprets; solver computes; runtime owns the result. Scales to a solver ecosystem (Part VII world model).

**38. Test / Bring-up Map** ○ *(new)* — *Purpose:* DFT, test coverage, bring-up procedure — the bridge from design to a working physical unit. *Objects:* (missing) `TestPoint`, `TestCoverage`, `BringUpStep`. *Owner:* runtime. *Rel/rep:* attaches to nets/components. *AI/verify/scale:* AI proposes test plans; runtime checks coverage; scales to closed-loop "measured vs predicted" (feeds Fidelity + Memory).

### Tier 7 — Production & Lifecycle Maps

**39. Manufacturing / DFM Map** ◐ — *Purpose:* fab process capability + DFM constraints. *Objects:* Fabrication floor (via requirement) + DFM rules; **missing** first-class `ProcessCapability{min width/space/drill, layers, classes}`. *Owner:* runtime (fab-sourced). *Rel/rep:* the floor DRC/DFM range over. *AI/verify/scale:* AI maps a chosen fab→capability; runtime enforces; scales to per-fab capability library + `ManufacturingIr` outputs.

**40. Assembly / DFA Map** ○ *(new)* — *Purpose:* assembly process + DFA constraints (courtyard, orientation, thermal relief, panelization). *Objects:* (missing) `AssemblyProcess`, `DfaConstraint`. *Owner:* runtime. *Rel/rep:* constrains placement/footprint. *AI/verify/scale:* AI flags DFA issues; runtime enforces; scales to assembly-house capability library.

**41. Supply Chain Map** ○ *(new)* — *Purpose:* sourcing, lead time, alternates, lifecycle risk — the thing that kills real programs. *Objects:* (missing) `SourcingRecord{stock, lead-time, alternates, risk}`. *Owner:* runtime; data from external APIs (untrusted→validated). *Rel/rep:* attaches to parts; feeds Risk + Cost. *AI/verify/scale:* AI proposes alternates; runtime validates equivalence; scales to live supply-risk monitoring.

**42. Cost Map** ○ *(new)* — *Purpose:* BOM cost, NRE, cost drivers as owned truth. *Objects:* (missing) `CostModel{unit, qty-breaks, NRE}`. *Owner:* runtime. *Rel/rep:* aggregates over parts/assembly/fab. *AI/verify/scale:* AI proposes cost-downs; runtime computes rollups deterministically; feeds Tradeoff/Objective. Scales to cost-as-a-constraint optimization.

**43. Compliance / Regulatory Map** ○ *(new; `RequirementCategory::Regulatory` exists but no compliance object)* — *Purpose:* standards, certifications, and the *evidence* of compliance. *Objects:* (missing) `ComplianceTarget{standard, clauses}`, `ComplianceEvidence`. *Owner:* runtime. *Rel/rep:* clauses become constraints; evidence links to verification/test. *AI/verify/scale:* AI maps standards→constraints; runtime enforces + assembles the compliance dossier. Scales to a certifiable, auditable evidence package.

### Tier 8 — Reasoning & Governance Maps

**44. Decision Map** ✅ — *Purpose:* choices with rationale, alternatives, confidence, author. *Objects:* `Decision{subject, rationale, decider, reasoning_call_seq, evidence, confidence}` (runtime). *Rel/rep:* links reasoning→commit; `DecisionCreated`. *AI/verify/scale:* AI reasoning recorded as the call behind the decision; scales to full decision graphs + Tradeoff linkage.

**45. Evidence Map** ✅ — *Purpose:* the justification substrate grounding facts/decisions. *Objects:* `Evidence{kind, content_reference, source, reliability}` (runtime). *Rel/rep:* supports decisions/requirements; `EvidenceReferenced`. *AI/verify/scale:* AI cites; runtime scores reliability; scales to auto-invalidation when a source is superseded + fidelity linkage.

**46. Risk Map** ○ *(new)* — *Purpose:* tracked liabilities, mitigations, residuals — engineering judgment made durable. *Objects:* (missing) `Risk{likelihood, severity, mitigation, residual, owner}`. *Owner:* runtime + human (accepts residual). *Rel/rep:* aggregates from assumptions, failures, supply, compliance. *AI/verify/scale:* AI proposes risks from Memory; humans own acceptance (Principle 11); scales to a portfolio risk view + field-learning.

---

## Part II · The Map Dependency Graph

The real structure is a **graph with feedback**, not a line. Solid arrows are primary derivation
(upstream defines downstream); dashed arrows are **feedback edges** (downstream truth mutates
upstream) — these are the self-correction seam the philosophy's planning discussion names.

```
                         ┌──────────────── META / SUBSTRATE (spans everything) ───────────────┐
                         │  Engineering Truth · Runtime State · Traceability · Change/Revision │
                         │  Authority/Autonomy · Fidelity · Knowledge/Memory                   │
                         └────────────────────────────────────────────────────────────────────┘
   Intent
     │
     ▼
   Requirement ──► Constraint ──────────────────────────────────────────────┐
     │  │             ▲                                                       │
     │  └──► Assumption│        ┌──────────── Objective/Tradeoff ◄───────┐    │ (constraints
     ▼                │        │                                        │    │  range over
   Architecture/Block ┤        │                                        │    │  everything
     │                │        ▼                                        │    │  below)
     ▼                │     Decision ◄──► Evidence      Risk ◄───────────┤    │
   Subsystem ─► Interface/Contract ─► Signal Flow ─► Bus/Protocol        │    │
     │                                   │                               │    │
     ▼                                   ▼                               │    │
   Component ─► Part(+BOM) ─► Package/Footprint      Net/Connectivity ◄──┘    │
     │            │                                    │  │  │  │             │
     │            ▼                                    ▼  ▼  ▼  ▼             │
     │        Supply · Cost                     Power · Ground · Clock · Pin-Fn│
     ▼                                                 │                       │
   Placement ─► Stackup ─► Copper/Routing (Track·Via·Zone) ─► Mechanical       │
     │                         │                                              │
     ▼                         ▼                                              │
   ┌─ Behavior ─► Thermal · Signal-Integrity · Power-Integrity · EMC · Reliability ─┐
   │        (physics Maps read geometry + power + behavior, emit predicted facts)   │
   └───────────────────────────────┬────────────────────────────────────────────────┘
                                    ▼
                        Verification ◄──► Simulation/Analysis ◄──► Test/Bring-up
                                    │
        (open blocking violation)   │  dashed feedback ▲ to Placement/Routing/Part/Architecture
                                    ▼
                 Manufacturing/DFM · Assembly/DFA · Compliance
                                    │
                                    ▼
                                 Release ──► (measured field data) ──► Knowledge/Memory ──► priors for the next design
```

**Feedback edges (the dashed truth):** Verification→Routing/Placement/Part/Architecture (DRC/DFM/EMC
failures loop back); Thermal/SI/PI→Placement/Stackup; Supply/Cost→Part→BOM; Risk→Assumption→Decision;
Test/Field→Fidelity + Memory. The graph is **cyclic by design**; the orchestrator's job is to run it
to a fixpoint with a termination guarantee (bounded loop-backs), not to march it once.

**The compounding loop (the strategic核心):** every Release emits measured reality that updates
**Fidelity** and deposits into **Memory**, which becomes *priors* that make the next design's
proposals better and its verification sharper. This loop — not any single Map — is the moat.

---

## Part III · Does the runtime support these Maps today? (gap analysis)

**Owned skeleton (✅, ~13 Maps):** Intent, Requirement, Constraint (partial), Architecture/Block
(thin), Component, Part/BOM, Net/Connectivity, Placement, Stackup, Copper/Routing (tracks only),
Verification, Decision, Evidence, Traceability, Runtime State. This is a genuine, replayable,
provenance-bearing spine — the hard part (the substrate) is real.

**The missing flesh (○/◐, ~33 Maps)** falls into four bands. For each, the exercise's required
lens — *why missing · why it should exist · where it belongs · integration · new objects · unlocked
capabilities*:

**Band A — the epistemic gaps (highest philosophical priority): Assumption, Risk, Tradeoff/Objective,
Fidelity.**
- *Why missing:* the runtime modeled the *design* but not the *reasoning about the design*. These are
  the judgment objects the philosophy (Parts II/IV) calls load-bearing.
- *Why it should exist:* untracked assumptions and un-owned risk are the #1 real cause of failure and
  the #1 knowledge that evaporates.
- *Where it belongs:* the domain-entity ring (`eak-domain`), alongside `Decision`/`Evidence`;
  enforced at the capability seam like every other object.
- *Integration:* new events (`AssumptionRaised`, `AssumptionDischarged`, `RiskRaised`,
  `TradeoffRecorded`); a release gate that blocks on undischarged critical assumptions; Fidelity as a
  tag on every derived fact.
- *New objects:* `Assumption`, `Risk`, `Objective`, `Tradeoff`, `ModelFidelity`.
- *Unlocks:* honest AI (it must declare assumptions, not bury them); design-space exploration;
  auditable risk posture; trust-weighted reasoning.

**Band B — the electrical-domain gaps: Power Domain, Ground/Return, Clock Domain, Pin-Function/Mux,
 Signal Flow, Interface/Contract, Bus/Protocol, Subsystem.**
- *Why missing:* the runtime jumped from flat blocks to nets, skipping the *logical electrical
  architecture* real engineers reason in.
- *Why it should exist:* power/clock/pin-planning/interfaces are where designs are actually specified
  and where most integration errors live.
- *Where it belongs:* `eak-domain` (objects) + `eak-engines` (their rules) + `eak-compiler` (they
  need their own IR band between Engineering IR and Schematic IR).
- *Integration:* new IR stage ("Logical Electrical IR"); domain objects committed through the seam;
  rules (power balance, mux conflict, CDC, protocol) added to the verification engine.
- *New objects:* `PowerDomain`, `ReturnPath`, `ClockDomain`, `PinAssignment`/`PinCapability`,
  `Signal`, `Interface`, `Contract`, `Bus`, `Subsystem`.
- *Status:* increment 1 — `PowerDomain` implemented (domain `validate()`, seam `CreatePowerDomain`,
  `erc-power-balance` rule; [ADR-0022](../docs/decisions/0022-band-b-power-domain.md)); increment 2 —
  `ClockDomain` implemented (domain `validate()`, seam `CreateClockDomain`, `erc-clock-domain-conflict`
  rule — the seed of CDC reasoning; [ADR-0023](../docs/decisions/0023-band-b-clock-domain.md)); increment 3 —
  `ReturnPath` implemented (domain `validate()`, seam `CreateReturnPath`, `erc-return-path-required`
  rule — the return half of the signal loop, gated on the design's own `impedance_target`
  declaration rather than a fabricated clock threshold per `transmission-lines.md` L145/L170;
  [ADR-0024](../docs/decisions/0024-band-b-return-path.md)); increment 4 — `PinCapability` +
  `PinAssignment` implemented (domain `validate()`, seam `CreatePinCapability`/`CreatePinAssignment`,
  `erc-pin-mux-conflict` + `erc-pin-capability` rules — capability and assignment kept separate per
  the master-prompt §31 rule, so a mux conflict is an engineering violation, not a silent string
  collision; [ADR-0025](../docs/decisions/0025-band-b-pin-function-mux.md)); increment 5 — `Signal`
  implemented (domain `validate()`, seam `CreateSignal`, `erc-signal-driver-sink` rule — the
  logical electrical meaning above raw connectivity, NOT a Net rename, only fields the architecture
  can justify per §32; direction encoded by source→sinks; [ADR-0026](../docs/decisions/0026-band-b-signal-flow.md));
  increment 6 — `Contract` + `Interface` implemented (domain `validate()`, seam `CreateContract`/
  `CreateInterface`, `erc-interface-contract` rule — protocol rule-set and its governed signal
  collection, co-dependent objects per the Map; minimal v0 structural checks for I²C/SPI/USB;
  [ADR-0027](../docs/decisions/0027-band-b-interface-contract.md)); increment 7 — `Bus` implemented
  (domain `validate()`, seam `CreateBus`, `erc-bus-topology` rule — a collection of interfaces
  sharing a physical bus line under one protocol contract with a declared topology; minimal v0
  structural checks for I²C/CAN/USB per protocol/topology; [ADR-0028](../docs/decisions/0028-band-b-bus-protocol.md)); increment 8 — `Subsystem` implemented (domain `validate()`, seam `CreateSubsystem`, `erc-subsystem-boundary` rule — the unit of reuse and reasoning at scale, hierarchical grouping of blocks exposing interfaces; [ADR-0029](../docs/decisions/0029-band-b-subsystem.md)).
  Remaining objects follow one per increment through the same seam. NOTE: ClockDomain precedes ReturnPath
  because return-path continuity targets controlled/electrically-long nets; the truthful v0 gate is the
  net's own controlled-impedance declaration (`Net::impedance_target`), NOT clock frequency — the
  electrically-long boundary is applied against the edge rate, which the model does not yet own
  (a documented correction to the increment-order rationale; `transmission-lines.md` L145/L170).
- *Unlocks:* AI that plans power/pin-out/interfaces (huge real value); ERC-by-contract; correct
  return paths (prevents most SI/EMC failure by construction).

**Band C — the physics/behavior gaps: Behavior, Thermal (deepen), SI (deepen), Power Integrity, EMC
(deepen), Reliability, Simulation.**
- *Why missing:* the runtime checks geometry against *floors*; it does not yet *model behavior* —
  it's a checker world, not a generative one (philosophy Part VII, the world-model gap).
- *Why it should exist:* this is the leap from "records the board" to "understands the board"; without
  behavior the runtime cannot predict, and AI cannot be checked against reality.
- *Where it belongs:* `eak-domain` (`BehaviorModel`, `SimulationRun`), `eak-engines`/a new
  `eak-solvers` boundary (deterministic analysis + external solver ports as Evidence sources).
- *Integration:* behavior attaches to components (AI-proposed from datasheets, validated by
  measurement/sim); solvers behind a port produce fidelity-tagged Evidence; physics Maps consume
  behavior + geometry + power.
- *New objects:* `BehaviorModel`, `SimulationRun`, `HeatSource`/`ThermalNode`, `PdnModel`,
  `RadiatingStructure`, `FailureMode`.
- *Unlocks:* prediction, counterfactuals, explanation — the generative world model; trustworthy
  autonomous design.

**Band D — the lifecycle/governance gaps: Supply, Cost, Compliance, Assembly/DFA, Test/Bring-up,
Change/Revision, Authority/Autonomy (deepen), Knowledge/Memory, Mechanical.**
- *Why missing:* the runtime stops at a manufacturable IR; it does not own the *program* around the
  board (buy, cost, certify, revise, remember).
- *Why it should exist:* real engineering is a lifecycle; the moat (Memory) and much of the value
  (cost/supply/compliance) live here.
- *Where it belongs:* `eak-domain` + adapter ring (external data validated in) + a new **knowledge
  tier** for Memory (Part VI).
- *Integration:* external data enters as untrusted→validated Evidence; Change/Revision extends the
  event log with branch/diff objects; Memory is a cross-project store fed by Release.
- *New objects:* `SourcingRecord`, `CostModel`, `ComplianceTarget`, `Revision`/`Branch`,
  `Pattern`/`Lesson`/`FailureRecord`.
- *Unlocks:* cost/supply/compliance as owned truth; git-for-hardware; the compounding knowledge moat.

**Verdict:** the architecture *supports* these Maps in the sense that its ownership discipline
(objects + seam + provenance + verification + IR projection) is the *right and sufficient pattern*
for every one of them — no new architecture is needed to add a Map, only new objects, events, rules,
and IR bands. **The gap is content, not structure.** That is the healthiest possible finding: the
runtime was built as a substrate, and Maps are what you pour into a substrate.

---

## Part IV · Does the repository organization reflect the World Model?

**Today** the code is organized by **clean-architecture ring** (`eak-units`, `eak-domain`,
`eak-ports`, `eak-runtime`, `eak-engines`, `eak-compiler`, `eak-phases`, `eak-store`,
`eak-reasoning`, `eak-cli`). That is correct and must be kept — the ring/dependency rule is a
load-bearing invariant. But it means **all engineering concerns are flattened into one giant
`eak-domain` and one giant `eak-engines`**: every object in one file, every rule in one file. The
repository reflects the *architecture*, not the *World Model*.

**Assessment:** the ring structure is right; the *intra-ring* organization does not yet express the
Map taxonomy, and it will not scale to ~46 Maps and hundreds of objects/rules in single files.

**Recommendation (no code change now — a target shape):** keep the rings; organize *within* them by
**Map/domain module**, so the code's shape mirrors the World Model:

```
eak-domain/   → submodules per Map cluster: intent/ requirements/ constraints/ architecture/
                electrical/ realization/ physics/ verification/ production/ reasoning/ meta/
eak-engines/  → rules grouped by Map: electrical/ physics/ dfm/ dfa/ compliance/ …
eak-compiler/ → IR bands per Map layer (adds Logical-Electrical IR, Physics/Analysis IR)
eak-solvers/  → NEW adapter-ring crate: deterministic analyses + external-solver ports (SI/PI/thermal/EMC/SPICE) producing fidelity-tagged Evidence
eak-memory/   → NEW crate (knowledge tier): the cross-project Engineering Memory (Part VI)
eak-knowledge/→ (optional) standards/process-capability/component-behavior libraries as owned, versioned data
```

**Principle for the reorg:** *the module tree should be a readable index of the Engineering World
Model.* A new engineer should be able to open `eak-domain/electrical/power_domain` and find the
Power Map, its objects, and its invariants in one place. Two genuinely new crates are justified by
the World Model and do not exist today: **`eak-solvers`** (the boundary that turns the checker world
into a generative one, keeping heavy numerics out of the kernel) and **`eak-memory`** (the
compounding knowledge tier). Both are outer-ring; neither weakens the dependency rule.

---

## Part V · The Engineering Knowledge Graph — the universal object meta-model

Every engineering object in every Map is a node in **one** knowledge graph, and every node carries
the **same twelve facets**. This uniform contract is what makes the World Model queryable, verifiable,
and trustworthy — and it is the concrete meaning of "objectification" (philosophy Part IV).

| Facet | What it is | Owner | Today |
|---|---|---|---|
| **Identity** | Opaque, stable, never-null id; survives rename/edit. | runtime | ✅ `EntityId(u128)` |
| **Relationships** | Typed edges to other nodes (contains, realizes, derives-from, constrains, references…). | runtime | ◐ `ProvenanceLink` + membership; needs a richer edge taxonomy |
| **Lifecycle** | The states an object moves through (proposed→committed→superseded; open→discharged…). | runtime | ◐ per-object status enums; not uniform |
| **Constraints** | The bounds that must hold over it. | runtime | ◐ `Constraint` for some |
| **Verification** | The rules that range over it + their results. | runtime (deterministic) | ✅ for realized objects |
| **Traceability** | Forward/backward lineage to intent and to dependents. | runtime | ✅ provenance graph |
| **Ownership** | Which layer/authority may assert/mutate it (Part VI). | runtime | ◐ autonomy flag; needs per-object |
| **Versioning** | Its identity across revisions/branches. | runtime | ○ (Change Map missing) |
| **History** | The ordered facts that produced it. | runtime | ✅ event log / replay |
| **Evidence** | What justifies it + reliability. | runtime | ✅ `Evidence` for decisions |
| **Reasoning** | The decision/call that created or changed it. | runtime (records AI proposal) | ✅ `Decision.reasoning_call_seq` |
| **Manufacturing Impact** | How it affects buildability/cost/yield. | runtime | ○ (needs DFM/DFA/Cost Maps) |

**The edge taxonomy (needs to grow from `RelationType`):** structural (`contains`, `part-of`),
derivational (`derived-from`, `realizes`, `refines`), constraint (`constrains`, `bounds`), evidential
(`justified-by`, `supersedes`), behavioral (`couples-to`, `drives`, `returns-through`), and lifecycle
(`discharges`, `mitigates`, `waives`). Today's `RelationType` covers the derivational/evidential
subset; the behavioral and lifecycle families are the growth area that Bands B–C require.

**Why uniform facets matter:** a rule, an AI agent, or a human can ask the *same questions* of any
object ("what justifies you? what depends on you? what's your manufacturing impact? are you
verified?") — which is exactly what makes the graph a *world model* the runtime can reason over,
rather than a heterogeneous pile of records.

---

## Part VI · Engineering Memory — the compounding knowledge substrate

Engineering Memory is **not** conversation memory. It remembers *engineering*, and it is the asset
that turns a tool into an institution. It is a distinct **knowledge tier** (proposed crate
`eak-memory`) built atop — never bypassing — the owned truth and provenance.

**What it remembers (each item is a first-class, evidence-backed object, never gossip):**
design decisions and their outcomes · **rejected solutions and why** (the road not taken) · trade-offs
and the weights that resolved them · reusable constraints and standards · verified calculations
(cached with their inputs + fidelity) · datasheet-extracted **component behavior** · **failure
history** (what broke, in the lab and the field) · manufacturing lessons · DFM/DFA capability records ·
EMI, thermal, power-integrity, and signal-integrity knowledge · reusable PCB design patterns
(decoupling schemes, return strategies, layout idioms).

**How it is owned and stays trustworthy (the discipline that separates Memory from a vector-DB of
vibes):**
1. **Every memory item is earned.** It enters only when a design is *verified and released* — tied to
   the evidence and outcome that justify it. No memory without provenance (Principle 3, applied to
   knowledge).
2. **AI proposes, the runtime curates.** AI may *suggest* a pattern or a lesson; the runtime admits it
   only against the evidence, and tags its confidence/fidelity. Memory is validated, not asserted.
3. **It is versioned and revisable.** A lesson can be superseded by better evidence (a failure record
   overturns a "safe" pattern); supersession is recorded, never erased.
4. **It compounds by feedback.** The Release→field-data→Fidelity→Memory→priors loop (Part II) is what
   makes each design start smarter than the last. This is the moat that *grows* while competitors'
   exhaust is discarded.

**The honest, unresolved tension (challenge the assumption of "just build a big shared brain"):**
local-first ownership (Principle 9) says the engineer owns their truth; a compounding cross-project
knowledge graph *wants* shared data. These conflict. The resolution must be *designed*, not defaulted:
tiered memory — **private** (this design), **organizational** (this company's compounding corpus,
owned locally/federated), and **universal** (vendor-neutral physics/standards/component behavior,
shareable). What crosses a tier boundary is an explicit, opt-in, privacy-preserving decision — never a
silent upload. Getting this right is both the biggest asset and the biggest trust risk in the whole
system.

**How Memory interacts with the layers (Part VI of the philosophy):** universal memory is mostly
*engineering-science/physics* (near-permanent); organizational memory is mostly *codified practice*
(revisable); private memory is *this design's* truth. AI is the retrieval-and-proposal engine over
all three; the runtime is the curator; humans own what is shared.

---

## Part VII · The roadmap — from AI-assisted PCB app to Engineering Operating System

Each stage is defined by **which Maps come online and become owned truth**, not by features. The
through-line: *every stage transfers more of engineering out of human heads and dead files into owned,
verified truth.*

**Stage 0 — Current MVP (owned skeleton).** Maps: Intent, Requirement, Constraint(◐), Block(thin),
Component, Part/BOM, Net, Placement, Stackup, Routing(tracks), Verification, Decision, Evidence,
Traceability. The substrate is proven: event-sourced, replayable, provenance-bearing, seam-validated,
17 rules. *Milestone already met.*

**Stage 1 — Pre-seed (make the reasoning honest + demoable).** Add the **epistemic Band A**:
Assumption, Risk, Tradeoff/Objective, Fidelity. Deepen Constraint toward a real calculus; add the
Change/Revision spine (git-for-hardware v0). *Why first:* it is cheap (domain objects + gates, no
heavy numerics), it directly strengthens the moat and the demo ("watch the AI declare and discharge
its assumptions; watch every finding trace to intent"), and it makes AI *trustworthy on camera*.

**Stage 2 — Version 1 (own the logical electrical architecture — Band B).** Add Power Domain,
Ground/Return, Clock Domain, Pin-Function/Mux, Signal Flow, Interface/Contract, Bus/Protocol,
Subsystem, plus a Logical-Electrical IR band. *Why:* this is where real engineers specify designs and
where the AI delivers its first *irreplaceable* value (power planning, pin-out, interface matching).
The product stops being "checks a board" and becomes "helps architect the electronics."

**Stage 3 — Version 2 (become a world model — Band C).** Introduce **Behavior** + the `eak-solvers`
boundary: thermal networks, SI/PI/EMC models, reliability, simulation-as-owned-evidence, all
fidelity-tagged. *Why:* this is the leap from checker to generative — the runtime can now *predict*,
answer *counterfactuals*, and *explain*, which is the precondition for trustworthy autonomy (philosophy
Part VII). AI now reasons *against a faithful model it cannot fool.*

**Stage 4 — Version 3 (own the lifecycle + program — Band D).** Add Supply, Cost, Compliance,
Assembly/DFA, Test/Bring-up, Mechanical, and graduated Authority/Autonomy. Stand up **`eak-memory`**:
the compounding knowledge tier begins accruing across projects. *Why:* the product becomes the whole
engineering program of record, and the moat starts compounding measurably.

**Stage 5 — Long-term Vision (the Engineering Operating System / the engineering brain).** The World
Model is deep enough that the runtime can *drive* end-to-end under supervision: propose intent
decompositions, plan power/pins/interfaces, place/route with behavioral awareness, predict and verify
against a high-fidelity model, and justify every step to intent. Memory becomes an industry-scale,
tiered corpus of provenance-linked engineering knowledge. The model generalizes **beyond the board**
toward system co-design (firmware, mechanical, thermal). At this point EAK is not a PCB tool with AI —
it is the **owned engineering brain** every hardware program runs on, and the drawing is merely one
of its outputs.

**The invariant across all five stages:** no stage may violate the twelve immutable principles
(philosophy Part IX). Every new Map is owned truth, seam-validated, provenance-bearing, and — where it
touches physics — fidelity-honest. Growth is monotonic transfer of engineering into ownership; it is
never a shortcut around the substrate.

---

## Coda · Assumptions challenged

1. **"The pipeline is the model."** No — the pipeline is one traversal of a cyclic graph; the Map
   graph (Part II) is the truth, and feedback is first-class.
2. **"Maps are separate tools/databases."** No — Maps are lenses over one owned truth-graph; a `Net`
   is the same object in five Maps.
3. **"The PCB is the product / the terminal artifact."** No — the owned engineering truth is the
   product; the board is one Realization Map's output; the World Model reaches past it.
4. **"We're most missing more rules / more canvas."** No — the runtime is missing *engineering
   objects* (Assumption, Power Domain, Behavior, Risk, Memory…), not more of what it already has.
5. **"Add features."** No — add **Maps** (owned truth). Features are interfaces to the runtime; Maps
   are the runtime.
6. **"Memory = remember the chat."** No — Memory remembers *engineering*, is evidence-earned, curated,
   versioned, and tiered; it is the compounding moat, and its sharing model is an unsolved trust
   problem to design deliberately.

*This document defines what the runtime must eventually own. It grounds `03-roadmap.md` and the
engineering backlog, which should be re-expressed as "which Map comes online next." No code was
written or modified in producing it.*
