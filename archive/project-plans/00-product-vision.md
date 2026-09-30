# Electronics Agent Kit — Product Vision (CANONICAL)

> **This is the highest-priority document in the repository.**
> Every architectural, product, engineering, terminology, and positioning decision must be
> validated against this document. **If any other document contradicts this one, THIS ONE WINS.**
> That explicitly includes `00-overview.md`, `02-technical-architecture.md`, the `docs/` tree, the
> `engineering-science/` layer, and every ADR. Where those describe *how* to build a fundable slice,
> they remain valid. Where their **framing** contradicts the category defined here, they are
> subordinate and must be reframed (see `project-plans/09-legacy-framing-audit.md`).
>
> Written 2026-07-10. **The "Cursor for hardware" tagline is retired** (founder ruling, 2026-07-10):
> it structurally invites the "so it's just a copilot?" confusion this product is defined against.
> Lead exclusively with **AI-Native Engineering Operating System**.

---

## 0. The one-sentence definition

**Electronics Agent Kit is an AI-Native Engineering Operating System for electronics: a
deterministic runtime that *owns* engineering intent, knowledge, state, constraints, and
correctness as first-class living objects — and on top of which engineers (and AI acting under a
strict verification boundary) perform the complete engineering lifecycle.**

The runtime is the product. Everything visual, every file format, every editor, every renderer,
every vendor integration is a **peripheral** — an adapter at the edge — never the center, never the
source of truth, never the identity.

---

## 1. The inversion (why this is a new category, from first principles)

Ask the question the way an inventor would, ignoring every existing tool:

> *If you had to invent electronics engineering software from scratch today — with modern AI,
> compiler theory, event sourcing, knowledge graphs, constraint solving, and codified engineering
> science — what would you build?*

You would **not** build a drawing program.

Every incumbent (KiCad, EasyEDA, Altium, Cadence, OrCAD, Mentor) is, at its core, a **graphics
editor that emits artifacts**. Its primary object is a *picture* — a schematic sheet, a copper
polygon, a Gerber. The actual engineering — *why* this part, *which requirement* this net
satisfies, *what constraint* this trace width honors, *what was decided and rejected and why* —
lives nowhere in the tool. It lives in the engineer's head, in a spec Word doc, in a review PDF, in
a spreadsheet of constraints, in Slack threads. The file knows **where** the copper is. It does not
know **why**. When the engineer leaves, the engineering leaves with them. The artifact is dead.

**EAK inverts this.** The primary object is not a drawing — it is a **live, versioned, executable
model of the engineering itself**: intent → requirements → constraints → decisions → parts → nets →
placements → routes → verifications → provenance, every one a first-class, typed, traceable object
owned by a deterministic runtime. Drawings — schematic, PCB, BOM — become **projections** of that
model, computed on demand, re-derivable, and verified by construction. The model is the truth; the
picture is a view.

This is the same category leap as:

| From (artifact tool) | To (semantic runtime) | The shift |
|---|---|---|
| Text editor | Compiler + language server | text → an owned semantic model (AST, types) |
| Files on disk | Database | records → an owned system of record with enforced invariants |
| Manual bookkeeping | Operating system | ad-hoc access → an owned substrate that mediates all state |
| Drawing a schematic | **Electronics Agent Kit** | **a picture → an owned model of the engineering** |

Incumbents digitized the **drafting table**. EAK builds the **engineering runtime** the drafting
table never had. That is the category: not a better editor — the first system whose object *is the
engineering*.

---

## 2. What category of software this is

**An Engineering Operating System** (equivalently: an *AI-native engineering runtime*, a *system of
record and system of reasoning for electronics engineering*).

It sits in the lineage of substrates that own truth and mediate work, not tools that produce files:

- Like an **OS**, it owns the resources (here: engineering state, intent, constraints, memory) and
  mediates every access to them through a controlled seam.
- Like a **compiler/LLVM**, it lowers a high-level intent through a series of canonical intermediate
  representations, running verification passes at every boundary.
- Like a **database (RDBMS)**, it is the single source of truth and enforces invariants by
  construction; nothing mutates state except through the transaction path.
- Like **Git**, it versions and makes the entire history replayable — but semantically (intent and
  engineering objects), not textually (lines in a file). "Git for hardware," realized as
  event-sourced engineering state, not diffed files.
- Like a **language server / type system**, it *understands meaning* and rejects incorrectness by
  construction rather than flagging it after the fact.

Editors, viewers, autorouters, simulators, part catalogs, and vendor-format bridges are
**applications and adapters that run on this substrate** — the way apps run on an OS. They are
replaceable. The substrate is not.

---

## 3. The problems this solves that existing software does not

1. **Knowledge evaporation.** In every incumbent, engineering rationale is never captured as data.
   EAK makes intent, decisions, evidence, and provenance first-class and permanent. The "why"
   survives the engineer.
2. **No source of truth for *engineering* — only for *artifacts*.** Incumbents own geometry and
   connectivity. Requirements, constraints, and trade-offs live in disconnected documents. EAK
   unifies the entire engineering model in one owned, queryable runtime.
3. **Correctness is bolted on and end-stage.** DRC runs at the end, on a finished drawing, as a
   human-invoked check. EAK enforces correctness **continuously and by construction** — an invalid
   design physically cannot be committed past the seam.
4. **No traceability.** You cannot ask an incumbent "which requirement justifies this net?" or "what
   breaks downstream if this spec changes?" EAK answers both — every fact carries its provenance to
   originating intent.
5. **Non-reproducibility.** A `.kicad_pcb` is a dead endpoint; you cannot replay how it was derived,
   or re-derive it under a changed assumption. EAK's process is deterministically replayable and
   re-derivable — an auditable, executable history, not a final artifact.
6. **AI cannot be trusted to *do* the work.** Bolting a chatbot onto a drawing tool yields
   suggestions a human must police. EAK makes AI **drivable** because the deterministic runtime
   verifies everything the AI proposes before it becomes real.
7. **Fragmentation of the lifecycle.** Spec here, constraints there, schematic in tool A, BOM in
   tool B, review in PDFs, knowledge in nobody's tool. EAK is the single environment where the whole
   lifecycle happens on one coherent model.

---

## 4. What makes EAK fundamentally different

**vs KiCad / EasyEDA / Altium / Cadence / OrCAD (ECAD editors):**
They own **geometry and connectivity**; EAK owns **engineering**. They are drawing tools whose
artifact is a picture; EAK is a reasoning-and-verification substrate whose artifact is a live model.
In EAK a schematic or PCB is a *derivable, re-derivable, verified-by-construction projection* — not
the source of truth. EAK is not a competitor to these tools at the level of *drawing*; it operates
one level beneath them, owning the engineering they never modeled. (It may *render through* a KiCad
projection for human eyes — but that is a display peripheral, not the backend and not the identity.)

**vs Cursor / GitHub Copilot (AI coding assistants):**
The "AI copilot for hardware" analogy is superficially tempting and precisely the framing to resist —
which is why the tagline built on it is retired. Cursor and
Copilot are **AI assistants operating inside a text buffer** — the source of truth is still text
files, the AI suggests text, and a human or compiler catches mistakes. The AI does not *own* a
semantic model of the program's correctness. EAK inverts the trust model: the **runtime owns truth
and correctness**, and AI is a bounded reasoning engine that can only *propose* through a capability
seam that re-validates every proposal. Moreover, code already had a compiler that defines
correctness; electronics engineering never had one — **EAK is that compiler + runtime**, not a
copilot bolted onto a drawing tool. Cursor makes a human faster at editing text; EAK makes the
engineering itself an owned, correct, living object.

**The moat, stated plainly:** it is not "we have AI too." It is the **deterministic runtime that
owns engineering truth** — intent, state, constraints, verification, provenance, replay — the
substrate that makes AI trustworthy and that is architecturally impossible to bolt onto an
editor-first product after the fact.

---

## 5. Core responsibilities of the Engineering Runtime (the kernel)

The runtime is the sole owner and sole mutator of engineering truth. Its responsibilities:

1. **Own Engineering State** — the single, authoritative, mutable model of the design. Exactly one
   write path (`commit`: stamp → append → fold). Nothing else may mutate it.
2. **Own Engineering Intent** — capture and permanently preserve the human's goal and the "why," as
   first-class objects, rooted at the head of every provenance chain.
3. **Own Engineering Constraints** — typed, dimensioned, first-class, enforced — not comments, not
   spreadsheet cells.
4. **Own Verification** — correctness by construction, continuous, at every phase boundary and at
   the seam; never an optional end-stage pass.
5. **Own Provenance & Traceability** — every fact links to its justification, its evidence, and its
   originating intent. No fact without lineage.
6. **Own Engineering Knowledge & Memory** — accumulate reusable, queryable engineering knowledge
   across the lifecycle and (eventually) across projects; knowledge is a first-class asset, not a
   side effect.
7. **Own Determinism & Replay** — the entire engineering process is reproducible and auditable;
   recorded reasoning + seeded ids + logical clock ⇒ byte-identical re-derivation offline.
8. **Own the Reasoning Boundary** — a single controlled port through which AI proposes and nothing
   else. The kernel disposes.
9. **Own Planning & Orchestration** — the engineering lifecycle as an executable, bounded,
   self-correcting process (state machines + loop-backs with guaranteed termination).
10. **Produce Projections** — IRs, views, and exports as *outputs* of the model. A projection may
    never become the source of truth.

---

## 6. Immutable principles

These are constitutional. They may be *extended* but never *violated*. A change that weakens any of
them requires an ADR that explicitly argues why the principle no longer holds — and the default
answer is no.

1. **The runtime is the source of truth. Artifacts are projections.** No file format, drawing, or
   external tool is ever the truth.
2. **Correctness is enforced, not suggested.** Invalid state cannot be committed. The safety model
   is verification-by-construction, never "AI suggests / human catches."
3. **The LLM proposes; the kernel disposes.** AI reaches state only through the capability seam,
   which re-validates. Untrusted model output never becomes truth directly.
4. **Determinism and replay are sacred.** Every design is re-derivable and auditable. Nothing may
   introduce hidden nondeterminism into the committed history.
5. **Traceability by construction.** Every committed fact carries provenance to its originating
   intent. Nothing enters state untraceable.
6. **Typed physical quantities everywhere.** Dimensional correctness is enforced by the type system;
   cross-dimension errors are impossible, not merely caught.
7. **Honesty over fabrication.** The system never invents intent, knowledge, or provenance it does
   not have. It distinguishes what it *knows* from what it *infers* (the origin-tagging discipline of
   ADR-0016/0017). It would rather say "imported artifact, no upstream intent" than fake a rationale.
8. **Clean-architecture rings; dependencies point only inward.** The kernel never depends on an
   adapter, a UI, or a vendor format. Vendor formats live *only* in the outermost ring.
9. **Local-first ownership.** The engineer owns their truth. The only edge that leaves the machine is
   the explicit reasoning call. Truth is never taken hostage by a cloud dependency.
10. **Engineering knowledge is first-class and permanent** — captured as data, queryable, and
    compounding, never a disposable side effect of drawing.

---

## 7. What must NEVER become part of this product

- **A drawing tool whose core object is a graphic.** The moment the picture becomes the source of
  truth, the category is lost.
- **A wrapper, plugin, or skin around KiCad / EasyEDA / Altium / any ECAD.** EAK is not an
  integration layer for existing EDA software and never uses one as its backend.
- **A vendor file format (`.kicad_pcb`, `.sch`, Gerber, ODB++) as the internal model or source of
  truth.** These may be read/written *only* as edge adapters, translated to and from the owned model.
- **A path for AI to write directly to state** without passing the kernel's capability seam.
- **Probabilistic / bolt-on correctness** as the safety model ("the AI is usually right and the human
  will catch the rest").
- **A cloud-first SaaS that takes ownership of the engineer's truth** or that makes the runtime
  depend on the network to function.
- **Scope creep of the kernel with UI, rendering, or vendor concerns.** Those belong in outer rings
  as applications/adapters.
- **"Just an autorouter," "just a simulator," "just a schematic capture."** Those are *applications*
  on the substrate, never the product's identity.
- **Nondeterminism, silent truncation, or untraceable state** introduced for demo speed or
  convenience.

---

## 8. Architectural decisions that must never be violated

(These are the load-bearing invariants already encoded in the code and ADRs; the vision elevates
them to constitutional status.)

1. **The Dependency Rule** — compile-time enforced; the kernel has no outward dependencies.
2. **Single commit path / sole mutator** — `RuntimeCore::commit`, stamp → append → fold → observe.
3. **Event sourcing as the system of record** — state is the fold of an append-only event log.
4. **IRs as canonical phase-boundary projections** — versioned, one-way lowerings, never the truth.
5. **Reasoning behind one port** — a single `ReasoningEngine` boundary; provider/model isolated in one
   adapter; the API key never leaves the Rust core.
6. **Deterministic core + recorded reasoning + replay** — reasoning calls are recorded; replay never
   calls the model or the clock.
7. **Capability-seam re-validation** — every proposal is re-checked at the seam before commit; a
   rejected proposal writes nothing.
8. **Origin-tagging honesty** — synthesized vs imported is distinguished; provenance is never
   fabricated.
9. **Vendor formats confined to the outermost ring** — never a kernel or use-case concern.

---

## 9. The ownership model — who owns what, and why

The system is a layered division of authority. Each layer owns exactly what it is uniquely fit to
own, and nothing it isn't. This is the heart of the design.

### 9.1 The Engineering Kernel owns **truth and its lawful change**
State, intent, constraints, provenance, the event log, verification orchestration, IR projection,
replay, the capability seam, autonomy gating. The kernel answers: *what is true, and how is it
allowed to change?* It is deterministic and it is sovereign.

### 9.2 AI owns **judgement under ambiguity**
Translating fuzzy human intent into candidate structured requirements; proposing functional
decompositions; proposing parts; explaining violations in natural language; suggesting fixes;
prioritizing under uncertainty. AI produces **proposals and explanations**. It never owns truth,
never commits, never defines correctness. It is the reasoning shell around a deterministic core.
Its output is always untrusted until the kernel validates it.

### 9.3 Deterministic algorithms own **the consequence — anything with a right answer**
Graph connectivity, constraint propagation and solving, bounded placement/routing search, geometric
DRC checks, IR lowering, the state fold, replay, id/clock generation, orchestration. If a question
has a computable correct answer, an algorithm answers it — never the LLM.

### 9.4 Mathematics owns **the formal method — how the algorithms are correct**
Graph theory (netlists, connectivity, the provenance graph); constraint satisfaction and optimization
(placement, routing, spec solving); computational geometry (clearance, courtyards, keepouts); linear
algebra and numerical methods (field solving, impedance, tolerance analysis); probability and
statistics (yield, tolerance stack-up, confidence); decision and control theory (the bounded
self-correction loop). Mathematics defines *why* a deterministic result is provably right.

### 9.5 Physics owns **ground truth — the laws of reality**
Maxwell/electromagnetics (impedance, crosstalk, EMC); thermal (junction temperature, dissipation);
semiconductor device physics; materials (dielectric, copper weight, loss tangent); RF. Physics sets
the values and laws that constraints and verification must respect. It is non-negotiable reality.

### 9.6 Engineering Science owns **codified practice — turning laws into rules**
IPC standards, DFM principles, stackup / return-path / SI-PI methodology, placement and routing
philosophy, and the mapping from physics/math to actionable thresholds and enforceable rules. It is
the bridge from *what nature permits* to *what good engineering requires*, and it owns the numeric
floors and rule definitions the kernel enforces.

**The clean seam:** physics = laws of nature · mathematics = formal methods to compute over them ·
engineering science = codified standards that turn laws into rules · deterministic algorithms =
execute the rules and compute consequences · AI = judgement where no algorithm suffices · kernel =
owns truth and enforces all of the above. Each layer is replaceable and testable in isolation; none
may usurp another's authority.

---

## 10. Interop (KiCad, KiCanvas, Gerber, part APIs) is demo scaffolding — not the product

**Founder ruling (2026-07-10): interop is scaffolding, not a standing capability.** The KiCad
importer/exporter and any reused renderer exist *only* to make the fundraise demo bulletproof. They
are a temporary crutch, explicitly slated for removal, and they define **nothing** about the product.
EAK is **not** a bridge, a converter, or an interoperability layer for existing EDA tools.

The rules:

- **The product does not import from or export to existing EDA tools as a feature.** Any
  vendor-format code (`.kicad_pcb` import/export, Gerber, ODB++) is throwaway demo scaffold —
  quarantined in the outermost ring, clearly labelled as such, and carrying an expiry. It is never a
  standing product capability and never appears in the product narrative as one.
- **Nothing in the runtime may ever depend on it.** The kernel is already vendor-format-clean; that
  is non-negotiable and is precisely what makes the scaffold safe to keep for the demo and safe to
  delete after it.
- **A renderer, if used, is a disposable display crutch** for the demo — never a standing component,
  never the source of truth, never the identity. When the runtime can project its own views, the
  reused renderer goes.
- **Part data enters as untrusted external input** validated into the owned model — never trusted
  directly, never a dependency.

**Litmus test (sharpened):** if every interop and rendering component were deleted tomorrow, the
engineering runtime — the product — would still exist and still be correct. Because that is true
*today*, the scaffold is safe to keep for the demo and safe to delete after. The day anything about
*truth* depends on a vendor format or a renderer, the architecture has failed.

---

## 11. Ten-year evolution

- **Year 0–1 (now): prove the substrate.** Kernel owns intent → verified release on a single curated
  vertical, local-first, deterministic and replayable, knowledge captured per project. A fundable
  demo of the AI-native loop. *(This is the current MVP slice — a thin vertical proof of the vision,
  not the vision itself.)*
- **Year 1–3: deepen the model.** Represent far more physics as owned state (real stackup, copper
  weight, dielectric, SI/PI/thermal *solved* rather than floored); richer verification; graduated
  human-in-the-loop autonomy; real parts and knowledge ingestion; editors and viewers as first-class
  *applications* on the runtime. Multi-project engineering memory begins.
- **Year 3–5: become a platform.** A stable capability API on which third-party engineering
  applications and verification adapters (SPICE, field solvers, thermal solvers) run as ports; a
  knowledge graph that learns across designs; team collaboration as multiple humans and agents on one
  event-sourced truth — "Git for hardware" fully realized as semantic version control.
- **Year 5–10: the Engineering OS.** The default environment where electronics engineering happens.
  The owned model generalizes beyond the board toward co-design (mechanical, thermal, firmware). An
  accumulated corpus of provenance-linked engineering knowledge becomes a compounding, defensible
  moat. AI autonomy rises *because the deterministic substrate earned the trust* — the system can
  propose, verify, and justify engineering end-to-end under supervision. The economy of dead artifact
  files is replaced by living, queryable, re-derivable engineering models.

The through-line: every year, **more of engineering becomes owned, typed, verified truth**, and less
of it lives in human heads and dead files. That monotonic transfer *is* the strategy.

---

## 12. Tensions and honest caveats (challenging our own assumptions)

A canonical vision that only flatters itself is a liability. The real risks:

1. **"New category" is cheap; substance is the runtime.** The claim is only credible because the
   substrate exists and enforces its invariants today. It must never become a slogan that outruns the
   code. The discipline: every grand claim must point at an enforced invariant.
2. **The owned model is currently shallow.** The kernel's physical representation is a seed —
   stackup is coarse, several physics quantities are floored rather than solved, whole engineering
   domains are unmodeled. The ten-year vision is a promise about *what becomes owned*, and it is not
   yet kept. Say so.
3. **"Engineering OS" invites boil-the-ocean.** The failure mode is an everything-tool that owns
   nothing well. The ring discipline and the "applications on a substrate" model are the guardrail:
   the kernel stays small and sovereign; breadth lives in outer-ring apps.
4. **Local-first vs. compounding cross-project knowledge is a genuine tension.** A knowledge graph
   that learns across designs wants shared data; local-first ownership resists it. This is unresolved
   and must be designed deliberately (federation, opt-in, privacy-preserving) — not defaulted into a
   cloud that violates Principle 9.
5. **Rejecting interop-as-a-feature is commercially bold.** The market still lives in vendor formats,
   and treating interop as disposable demo scaffold (not a bridge) means EAK asks engineers to work
   *inside* the runtime rather than meeting their existing files halfway. That is the correct
   category-defining stance *and* a real go-to-market risk: the migration story for a new user's
   existing designs is a deliberately deferred, unsolved problem. It must be an eyes-open bet, not an
   accident.

---

## 13. The litmus test for every future decision

Before adding anything, ask:

1. **Does it strengthen the runtime's ownership of engineering truth** (intent, state, constraints,
   verification, provenance, knowledge)? If yes, it is core.
2. **Or is it an application/adapter on the substrate** (an editor, a viewer, an importer, a solver)?
   If so, it lives in an outer ring and must never leak into the kernel.
3. **Does it make a picture, a file, or a vendor tool the source of truth?** If yes — **reject it.**
4. **Does it let anything untrusted mutate state without the seam, or introduce nondeterminism, or
   fabricate provenance?** If yes — **reject it.**

If a proposed feature is neither strengthening the owned model nor a clean edge adapter for it, it
does not belong in Electronics Agent Kit.

---

*End of canonical vision. All other documents anchor to this one.*
