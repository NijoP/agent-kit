# Electronics Agent Kit — Engineering Philosophy (CANONICAL)

> **Priority 2 in the repository**, beneath only [`00-product-vision.md`](00-product-vision.md).
> The vision defines *what* Electronics Agent Kit is. This document defines *why* it exists and
> *why every architectural decision exists.* Where a lower document contradicts this one on the
> **reasoning** behind a decision, this one wins; where it contradicts the vision on the **identity**
> of the product, the vision wins.
>
> This is **not** a technical document and **not** a marketing document. It is the intellectual
> foundation — the philosophy of engineering that the architecture is a faithful expression of.
> It is written to be **timeless**: no tool, language, model, vendor, or implementation detail
> appears here that would date it. If it is doing its job, it is still true in twenty years, when
> every technology named in the rest of the repository has been replaced. Written 2026-07-10,
> synthesized from first principles.

---

## Prologue — the one idea

Engineering has always been recorded in **dead media** — drawings, documents, spreadsheets, a
drafting table digitized into a graphics editor. A drawing knows *where* the copper is. It cannot
know *why*. It cannot check itself against the laws it must obey. It cannot tell you what breaks
downstream when an assumption changes. It cannot be re-derived under a new constraint. It cannot
carry its own reasoning. The artifact is *a corpse of a thought* — the engineering that produced it
is not in it.

Everything in this repository follows from one conviction: **engineering must stop being a dead
artifact and become a living, owned, executable model of itself.** That is the leap software already
made when text stopped being marks a human interprets and became a model a machine owns, checks, and
runs. This document explains what that means, and why it is not a feature but a change of category.

---

## Part I · What Engineering Is

**Engineering is the disciplined transformation of intent and constraint into a physically
realizable artifact that behaves correctly under the laws of reality.** It is a directed act, not a
discovery. Three primitives are always present and always coupled: **intent** (a human purpose that
does not yet exist in matter), **constraint** (the finite, non-negotiable envelope reality imposes),
and **realization** (the commitment of the design to physical form, where it becomes irreversible and
consequential). The word *correct* is load-bearing: not merely functional, not merely elegant, but
**provably adequate to its purpose within every governing constraint simultaneously.**

It is neither science nor craft. **Science** expands what is known to be true; it is descriptive and
reversible and rewards the single decisive experiment. Engineering *consumes* scientific truth as a
fixed input and produces a particular committed object. **Craft** transforms material through
embodied, tacit skill, justified by result and tradition — and it *dies with the craftsman.*
Engineering demands justification that is inspectable and transferable: the "why" of every choice,
defensible before the artifact exists, independent of the individual who made it. Craft is correct
because the master's hands were correct; engineering must be correct because the *reasoning* was
correct, and the reasoning can be shown.

**Engineering thinking** is a distinct cognitive posture, not general problem-solving pointed at
physical things. It reasons *under constraint, not toward an optimum* — seeking an answer that
satisfies all binding constraints at once and is defensible, not the abstract best. Its native unit
is the **tradeoff**: every gain is paid for, and no decision is local, because gains and costs ripple
across coupled dimensions. It treats **margin** as first-class, designing to survive the worst-case
stack-up of many small deviations rather than to the exact edge. It reasons **backward from
consequence** — beginning at the burned board, the recall, the dead patient, and reasoning back to
the decision that permits it, so that correctness is defined negatively, by the exhaustive absence of
ways to fail. And it holds **multiple physical regimes in superposition**, because a trace is at once
an electrical conductor, a thermal element, an antenna, and a mechanical object, and reality does not
decompose into the neat modules a mind prefers.

**Engineering knowledge** is not a pile of facts. A fact is inert. Engineering knowledge is *the
justified rule that governs action* — a fact bound to a consequence, a condition, and a rationale. It
exists in two forms in permanent tension: **codified** (standards, equations, derating tables —
transferable but lossy, stripping the *why it applies here*) and **tacit** (the seasoned engineer's
felt sense of what will fail — rich but trapped in individuals). Today this knowledge **evaporates**,
and the cause is structural: the rationale is never captured as a first-class thing. It lives in a
head, a review comment, a hallway. The artifact records where the copper is, never why. Engineering
is the one discipline that systematically discards its own reasoning at the moment of completion —
**because its medium has always been the dead artifact, which cannot hold a reason.**

**Engineering intent** is the human purpose that seeds the entire act — the answer to "what is this
*for*, and why does it matter" — before any structure exists to serve it. It is the root of all
provenance: every downstream object is legitimate only insofar as it traces back to an intent that
justifies it. Intent is **not** a requirement. A requirement is a derived, formalized, testable
projection of intent into a checkable clause. Intent is the purpose that *generated* the requirement,
the "why" that outlives the "what." A requirement severed from its parent intent is a rule no one can
question, revise, or correctly relax when the world changes — which is why intent must be owned as the
head of the chain, and why a system that *invents* intent it does not have has poisoned every
justification beneath it.

**Engineering intelligence** is the capacity to reason correctly about physical systems, under
uncertainty, toward an intended purpose, and to justify each step. It is not raw knowledge and not
raw computation; it is the disciplined binding of intent, constraint, and physical law into a
committed, defensible design. For a *system* to possess it, three things must be simultaneously true:
it must **own a model of engineering truth** (not a picture of an artifact), it must be able to
**reason over that model toward purpose**, and it must be able to **verify its own conclusions
against reality's laws and refuse what violates them — and say why.** Intelligence here is inseparable
from the discipline of correctness and honesty: a system that proposes confidently but cannot verify,
or fabricates a rationale it does not hold, is not intelligent about engineering — it is *fluent*
about it, which is more dangerous than ignorance.

### Why engineering is not programming — the crux

This is the sharpest line in the document, and it must not be softened.

- **Physical ground truth and irreversibility.** A program runs in a world we authored; its semantics
  are our invention, and a bug is a state you revert. Engineering runs in a world we did not author
  and cannot amend. A mis-sized trace is a burned board; a released design is ten thousand units in
  the field; a mis-set margin is a dead patient. **There is no undo for matter.**
- **Continuity and tolerance versus discrete exactness.** Programming lives in a discrete, exact
  universe where an integer is exactly itself and equality is decidable. Engineering lives in the
  continuum: every quantity carries a tolerance, every value is a distribution, and "correct" is not
  `==` but *"inside the envelope across the worst-case stack-up of all deviations at once."*
- **Multi-physics coupling with no module boundaries.** Software correctness composes — a correct
  function stays correct when called. Physical reality does not respect our abstraction boundaries:
  the electrical, thermal, mechanical, and electromagnetic behaviors of one copper feature *are the
  same feature.* You cannot unit-test a trace's thermal behavior in isolation from its electrical
  behavior.
- **No historical compiler for correctness.** This is the deepest asymmetry. Programming was *given a
  compiler* — a mechanical arbiter that defines "well-formed" and rejects the incorrect by
  construction, before it runs. Engineering **never had one.** There has never been a mechanism that
  says *no, this design violates a law, you may not commit it.* Correctness was always a human check,
  run late, on a finished drawing, by someone who might be tired.

The difference is total: programming manipulates a symbolic world we defined and can revert,
arbitrated by a compiler we built; engineering commits irreversibly to a physical world we did not
define, arbitrated — until now — by nothing but human vigilance.

### Why engineering must become executable — the central thesis

Every pathology above — evaporating knowledge, absent traceability, late and human correctness,
non-reproducibility — has a single root: the medium of engineering is dead. The necessary leap is
exactly the one software already made. Text became executable: it stopped being marks a human
interprets and became a model a machine owns, understands, checks, and runs; the compiler could
reject the incorrect, the type system could make whole error classes *impossible* rather than
*caught*, and the program became a live, inspectable, re-runnable object.

**Engineering must make the same leap: from dead artifact to owned, executable model.** Only then can
the "why" survive the engineer, correctness be enforced by construction, every fact carry its lineage
to intent, the whole process be re-derived under changed assumptions, and reasoning — human or
machine — finally be trusted, because a deterministic model verifies every proposal before it becomes
real. This is why engineering must become executable: not for speed, not for automation, but because
**a description of a thought cannot check itself; a model of it can.**

---

## Part II · Engineering Truth

Engineering is the disciplined production, justification, and preservation of *claims about how a
designed thing will behave in reality* — claims consequential enough that people stake money, safety,
and lives on them. The objects a runtime must own are therefore not shapes and nets but **epistemic
objects**: truths, facts, decisions, evidence, assumptions, constraints, and the lineage that binds
them. A system that owns the geometry but not the epistemology owns the artifact but not the
engineering.

**Engineering truth** is a third epistemic category. To call something true in engineering is not to
assert it and not to prove it: it is to hold a claim that is **justified** (grounded in evidence or
valid derivation), **provenance-bearing** (its grounds are attached, not remembered), and
**revisable-but-auditable** (it may be overturned by new evidence, but never silently). It differs
from **mathematical proof**, which is true by necessity, timeless and context-free; and from
**scientific fact**, which aspires to universal, observer-independent generality. Engineering truth is
**local, purposive, and conditional** — true for *this* design, stackup, environment, and margin. It
is contingent like science, structured like proof, and oriented toward a purpose neither shares.

From this flow the epistemic objects, each of which must be owned:

- **Facts** — claims that are measured or derived, dimensioned, and checkable. A number without units
  is not a fact; it is a rumor with a decimal point. A fact copied into a slide dies the instant an
  upstream value moves; a fact held as a derived node stays live.
- **Decisions** — choices among admissible alternatives, made under judgment that facts alone cannot
  settle. A first-class decision carries its alternatives, rationale, author, confidence, and the
  reasoning that produced it. **Facts constrain decisions, but decisions are not entailed by facts** —
  two competent engineers with identical facts may choose differently and both be right — which is
  exactly why a decision cannot be reconstructed from the artifact and must be owned.
- **Evidence** — the justification substrate (datasheets, measurements, simulations, calculations,
  citations) that converts assertion into justification. It carries a reliability; it is what makes
  truth revisable *responsibly*, because when a source is superseded every claim resting on it becomes
  suspect automatically — but only if the evidence link exists as an object.
- **Assumptions** — unverified premises the design silently rests on, and *the most dangerous objects
  in engineering*, precisely because an assumption is a truth-shaped hole that behaves like a fact
  until reality disproves it. Most failures are not wrong facts or bad decisions; they are unexamined
  assumptions that were true in the lab and false in the field. An assumption must be first-class and
  **dischargeable** — converted to a grounded fact by evidence, converted to an enforced constraint,
  or explicitly accepted as residual risk. An untracked assumption is a latent failure the system
  must refuse to forget.
- **Constraints** — the bounds reality and intent impose: typed, dimensioned, and *enforced*. A
  constraint that is written but not checked is indistinguishable from a wish.
- **Correctness** — satisfaction of every constraint *and* fidelity to intent. The second half is what
  a pure checker misses: a design can pass every rule and still be wrong, because it correctly
  implements the wrong thing. Correctness is a property of the whole, established **by construction**;
  the end-stage check only *confirms* a correctness maintained throughout, or discovers, late and
  expensively, that it was lost long ago.
- **Provenance** — the lineage linking every fact and decision to its evidence and, transitively, to
  intent. *No fact without provenance.* Provenance is what makes truth revisable-but-auditable: revise
  a root and the graph tells you exactly what falls.
- **Memory** — the durable, queryable accumulation of all the above across time. Current state answers
  "what is the design"; memory answers "how did it come to be, what did we already rule out, what did
  we learn." Memory is a first-class asset, not a log.
- **Context** — the situated frame (project, domain, standards, environment, prior decisions) against
  which truth is evaluated. Engineering truth is *indexed to context*; strip the context and a fact is
  unfalsifiable.

**Why one runtime must own all of it.** The argument is epistemic, not architectural. Every object
above shares one property: **its meaning lives in its relations.** A fact means nothing without its
evidence and context; a decision means nothing without its alternatives; an assumption is dangerous
only in relation to what it silently supports; correctness is a property of the whole graph. Scatter
these across human memory, documents, and tools and you do not merely lose convenience — you **sever
the relations that constitute the truth.** A fact in a spreadsheet, its evidence in a PDF, and its
rationale in a chat thread are three orphans; the truth that lived in their connection no longer
exists anywhere. Diffuse knowledge tends thermodynamically toward three failures: **drift** (copies
diverge, with no fact of the matter about which is true), **amnesia** (the "why" evaporates as people
leave), and **untraceability** (a claim cannot be grounded, so it cannot be trusted or safely
revised). Therefore truth must be owned by a single runtime that holds each object *together with its
relations, as a live and versioned thing.* **Ownership is not centralization for efficiency;
ownership is the epistemology** — engineering truth is a relational, revisable, provenance-bearing
object, and such an object can only exist where its relations are kept whole.

---

## Part III · The Sovereign Runtime

If truth must be owned, something must own it. That something is the **runtime**, and its ownership
must be **sovereign** — sole authority not just over what is true, but over the lawful path by which
truth changes. A sovereign that could be edited from anywhere is no sovereign. Here is why each
faculty belongs to the runtime and nowhere else.

- **Truth.** A claim with no owner has no defense. When truth lives in a head it dies with attention;
  in a document it is true only until the next unsynchronized edit; split across tools, each is
  locally correct and the union is globally false. The runtime claims sole authority so truth has one
  voice — and because it alone may declare a fact true, and only after deterministic validation, truth
  becomes **correct by construction** rather than correct by vigilance.
- **State.** There must be exactly one authoritative model reached through exactly one write path.
  Multiple copies are not redundancy; they are a promise to eventually disagree. That **state is the
  fold of an ordered, immutable record of facts** is not an implementation trick but the right
  ontology: if the present is *derived* from what happened, then the present is never a free-floating
  assertion — it is an entailment, and you cannot have a present you cannot explain. Authority and
  accountability become the same object.
- **Memory.** A snapshot is a photograph of an argument mid-sentence. By owning durable history the
  runtime makes the past queryable and load-bearing: supersession is recorded, not erased, so "why
  isn't it X anymore" is a fact, not a memory. This is the difference between a design that *has* a
  history and a design that *is* its history — and it is what lets a design be audited and replayed
  years after its authors have forgotten the reasons, which is exactly when trust matters most.
- **Constraints.** A constraint that cannot be enforced is advice, and advice is ignored under
  deadline. Bounds must reside where writes happen, so a violating change is refused at the moment of
  attempt, not discovered at manufacture. Enforcement is only real when it is unavoidable, and it is
  only unavoidable when the enforcer *is* the gate through which all truth passes.
- **Verification.** Verification by anyone other than the owner of truth is theater — a report about a
  state the reporter cannot bind. If a separate tool checks a design that can then change without
  re-checking, the check certifies a ghost. Correctness must be checked by the authority that commits,
  continuously, so the pass is a property of the *current* truth and not a past screenshot.
- **Reasoning boundaries.** This is the keystone of trustworthy AI-driven engineering. Stochastic
  judgment — from a model or a tired human — is *generative*, and generativity is exactly what you
  must never grant direct write access to truth. The runtime owns a single controlled seam:
  **reasoning may propose; the deterministic core alone may commit, and only after validation.** The
  boundary is not a limitation on the reasoner; it is *the thing that makes the reasoner trustworthy
  at all.* Brilliance and nonsense enter through the same narrow door and meet the same law, so we can
  invite arbitrarily powerful, arbitrarily fallible reasoners into engineering without betting the
  design on their being right. **Trust is not a property of the proposer; it is a property of the
  seam.**
- **Planning.** The lifecycle is not a pipeline that flows once; it fails, loops back, and
  self-corrects. An ad-hoc human workflow is owned by nobody and reproducible by no one. When the
  runtime owns planning as a bounded, executable process, gates that must pass and loop-backs that are
  lawful become properties of the substrate — progress stops depending on who is in the room, the
  design cannot advance past a gate it has not earned, and it *can* retreat when truth demands.
- **Knowledge.** Knowledge that is not owned is re-derived, and re-derivation is the quiet tax that
  keeps engineering slow and forgetful. Because the runtime owns truth, state, memory, and provenance,
  knowledge accrues for free and correctly: every justified decision is already structured, already
  linked to evidence, already replayable. A federation of tools produces exhaust; a sovereign runtime
  produces a growing, trustworthy corpus where each new design stands on the verifiable shoulders of
  every prior one. **This is why the moat compounds.**

**What "Operating System for engineering" means.** An operating system earns its name by being the
sovereign mediator between raw capability and every program that would use it. An OS *for engineering*
is deeper, because it mediates not hardware but *truth*: it owns intent, state, memory, constraints,
verification, reasoning, planning, and knowledge, and exposes them only through lawful, validated,
recorded operations. Every act of engineering becomes a system call against the sovereign — proposed
at the boundary, validated by the core, committed as an accountable event. Sovereignty is not
centralization for its own sake; it is the **precondition for trust.** You cannot safely put powerful,
fallible AI at the heart of engineering unless there exists an authority it cannot bypass, a truth it
can enrich but not overwrite, a record it cannot erase. **Diffuse ownership makes AI a liability;
sovereign ownership makes AI leverage.** That inversion is the whole point.

---

## Part IV · The Principle of Objectification

For the runtime to *own* engineering, engineering must be made of things it can hold. **In Electronics
Agent Kit, everything that carries engineering meaning is promoted to a first-class Engineering
Object** — named, owned, and standing on its own, not buried as ink inside a picture or as a memory
inside a person. This is not a modeling convenience. Objectification is what ownership concretely
*means.*

An object earns six powers that a drawing, a document, or a fact-in-a-head structurally cannot have:

1. **Addressability** — it can be named and referred to; it has a stable identity that survives
   renaming and reformatting.
2. **Provenance** — it can carry its own lineage.
3. **Relationship** — it can be connected in a graph of meaning, not mere spatial adjacency.
4. **Verifiability** — rules can *range over* it, because it is enumerable, typed, and inspectable.
5. **Reasoning** — algorithms and AI can operate on it, because it is structured meaning and not
   pixels or prose. This is what makes AI *drivable* rather than merely suggestive.
6. **Permanence** — it survives the engineer; it is state in the runtime, not a synapse in a person.

The incumbent world loses all six by default, because there these properties live implicitly, and the
implicit is by construction un-owned. This is why knowledge evaporation is not a gap to be patched but
a category consequence: an artifact tool cannot preserve what it never made into an object.

**Why an object and not a value.** Consider a 3.3 V rail. On a schematic, "3V3" is dead ink: it does
not know it is a *voltage* rather than a string, does not know its *tolerance*, does not know its
*source*, does not know *what depends on it.* The instant the quantity was "just drawn," its
engineering meaning was discarded; what remains is a picture of a decision, not the decision. As an
object, the same rail owns its magnitude and unit (so it cannot be confused with a current or added
to a length), carries its tolerance, links to the requirement that justifies it, records the decision
that set it, and is referenced by everything downstream. Now a rule can range over it; now a change
propagates; now AI can reason and the core can verify. **A value is a reading of the world; an object
is a held commitment about it — and engineering is made of commitments, not readings.**

Objects fall into tiers, and the tier explains the *why* — what would be irreparably lost if the thing
were demoted:

- **Purpose-objects** (Requirement, Constraint) — the *why* made addressable; the root and the
  enforceable shadow of intent. Without them, "which requirement justifies this net?" is unanswerable.
- **Physical-quantity-objects** (Voltage, Current, Power) — so dimension, tolerance, source, and
  dependents travel *with* the value, making cross-dimension error impossible rather than merely
  caught.
- **Structural-objects** (Component, Subsystem, Interface, Power Domain, Clock Domain) — the
  organization of the system *as meaning*. A power or clock domain is invisible in a drawing yet must
  be a scope a rule can range over; demoted to a label, it is a note no rule can see.
- **Connectivity-objects** (Signal, Net) — the bridge between logical intent and physical realization;
  the one relationship incumbents already objectify, which is exactly why they get routing correctness
  and nothing else.
- **Reasoning-objects** (Decision, Calculation, Verification, Simulation) — the *act* of engineering
  made durable, inspectable, and replayable, rather than a transient event in a person's afternoon.
- **Judgment-objects** (Evidence, Risk, Tradeoff) — the most fragile knowledge of all, the part that
  lives most exclusively in the senior engineer's head. A Tradeoff records what was weighed and
  *rejected* — the road not taken, which no artifact preserves and which juniors pay dearly to
  relearn.
- **Artifact-objects** (schematic, board, BOM, and every vendor rendering) — the inversion's punchline:
  even the drawing is demoted to *a projection of the model*, re-derivable and verified by
  construction. The picture stops being the truth and becomes an output the truth emits.

The load-bearing principle: **if it isn't an object, it isn't owned; if it isn't owned, it's lost.**
But the principle carries a discipline, or it becomes its own pathology — infinite regress, modeling
every fleeting sub-thought until the model is heavier than the engineering. **Objecthood is conferred
by consequence, not by category.** A thing earns objecthood when, and only when, one of the six powers
is actually needed of it: something must refer to it, it must carry lineage, it must participate in a
relationship, a rule must range over it, an agent must operate on it, or it must outlive the moment.
Promote a thing exactly when its meaning would otherwise be lost, and not one step sooner. This is
what keeps the kernel small and sovereign: everything owned is owned because it must be.

---

## Part V · The Engineering Language

To *own* engineering truth — to be answerable for it, not merely to store it — the runtime must be
able to **express** that truth in a form it can check. This is the affirmative case for an engineering
language, and it survives the strongest objections only in a precise form.

The three inherited media each fail the bar. **Natural language** carries intent but not decidability
("the regulator must stay cool" has no truth value until *cool*, *must*, and *the regulator* are
pinned to quantities, obligations, and referents). **Drawings and netlists** encode geometry and
connectivity but are mute on *why*, on *what must remain true*, and on the physics that adjudicates a
violation. **General-purpose programming languages** encode computation, not engineering meaning — a
machine number for a voltage does not know it is volts, cannot refuse to be added to seconds, and has
no opinion about a derating rule. To own intent, constraint, physics, and provenance *together*, none
suffices, and no accretion of them composes into a checkable whole.

The honest objection is that "language" may be the wrong noun: it smuggles in a surface syntax and
human authorship, when perhaps only the runtime's internal representation matters; and engineering is
plural, so the true object may be a *family of typed representations* rather than one language. The
resolution: these objections are right about the **surface** and wrong about the **substance.** EAK
requires an engineering language in the essential sense — a formal system with typed physical
semantics, an inference relation, and a provenance spine — but it does **not** yet require a
human-facing surface syntax, and may never require a single one. The load-bearing claim is that **the
runtime's internal representation must itself be linguistic** — compositional, typed, denotational,
closed under inference — rather than a passive data schema. *A schema stores; a language means and
entails.* And "family versus one language" is a false dilemma: the layers below are facets of **one
coherent language family**, sharing a universe of typed engineering objects and a provenance spine,
differing only in *illocutionary force* — the kind of statement each makes.

The layers, by role and guarantee (not syntax):

- **Intent Language** — expresses human purpose; guarantees **rootability**, so every downstream
  obligation traces to a named intent and nothing is enforced that no one meant.
- **Requirement Language** — expresses testable, dimensioned obligations ("shall"); guarantees each is
  **falsifiable and dimensioned**, never a vibe.
- **Constraint Language** — a **constraint calculus over typed physical quantities**; guarantees
  **enforceability and unit-soundness** — dimensionally incoherent constraints are inexpressible, and
  satisfaction is mechanically decidable. This is the sharpest edge of ownership: the layer that says
  *no* with authority.
- **Planning Language** — expresses the lifecycle as a bounded, inspectable plan; guarantees the
  process is **an object, not folklore** — auditable, resumable, terminating.
- **Verification Language** — expresses properties that range over objects and yield **judgments with
  provenance**; guarantees every pass/fail is inseparable from the rule and objects that produced it.
- **Reasoning Language** — the seam where stochastic agents meet the deterministic core; expresses
  proposals as bounded, typed claims, never direct mutations; guarantees **containment** — nothing
  enters the owned model except as a claim the runtime independently adjudicates.
- **Execution Language** — how the model is lowered, run, and committed; guarantees **determinism and
  reproducibility** — the same intent yields the same released artifact, replayably.
- **Review Language** — expresses findings and traceability back to humans; guarantees **legibility
  without loss** — every finding renders in human terms while remaining a live link into the formal
  chain, so a person can interrogate *why* down to intent.

**Future DSL, and the honest risk.** An explicit human-facing engineering DSL is plausible on a
10–20 year horizon, but only *downstream* of the internal language, never ahead of it — distilled
from proven, stable concepts, not invented to guess them. It must be earned by three conditions: the
typed-quantity universe and constraint calculus have stopped churning; enough real engineering has
flowed through to reveal empirically which intents are frequent and painful to express; and
conversational, agent-mediated authoring has hit a demonstrated ceiling only direct expression can
break. The governing hazard is **premature language design** — a syntax minted early ossifies the
wrong abstractions and becomes a permanent tax. The discipline: **build the internal language now,
because ownership requires it; resist the surface syntax until the concepts are load-bearing, stable,
and demonstrably underserved by conversation.** A language EAK *is* precedes any language a human
*writes.*

---

## Part VI · The Division of Authority

An engineering artifact is a claim about the world: *this thing will work.* That claim is only as
trustworthy as the chain of reasoning behind it. The layered ownership the architecture enforces is
therefore an **epistemology** — a theory of what can be known, by what faculty, with what warrant.
Each layer holds a distinct *kind* of knowledge with a distinct warrant for its truth, and the central
conviction is that these kinds must never be confused, because a claim justified by one warrant cannot
borrow the authority of another. (This grounds, and does not re-decide, the ownership model of vision
§9.)

- **Mathematics owns formal reasoning** — the domain of *necessary* truth. A theorem is true in every
  possible world; this is why an algorithm's correctness is a mathematical, not empirical, question,
  and why formal method is the only source of certainty a runtime that owns truth can be built upon.
  Its limit: mathematics is **silent about which world we inhabit** — its truths are hypothetical,
  true relative to axioms it does not supply — so it may never set a physical value.
- **Physics owns reality** — the non-negotiable ground truth. Maxwell's equations, junction
  temperature, dielectric loss are discovered, not decided; they hold veto authority nothing above may
  override. When practice and physics conflict, physics wins, always, because practice can be wrong
  and reality cannot. Its limit: **under-determination** — physics permits far more than good
  engineering allows; it draws the outer wall of the possible but names no good place to stand inside
  it.
- **Engineering science owns best practice** — codified professional judgment, the distilled memory of
  the discipline's successes and catastrophes, which translates the wide permission of physics into
  the narrow prescription of practice. Its limit: this knowledge is **contingent and revisable**; a
  standard is a human artifact, and treating its threshold as a law of nature freezes yesterday's
  wisdom into tomorrow's dogma. Practice must always know itself *as* practice, and may never
  contradict physics.
- **Algorithms own deterministic computation** — anything with a computable right answer must be
  answered by the method that produces it *every time*, because reproducibility is the precondition of
  trust and replay. A stochastic guess at a question that has a right answer is a defect no matter how
  often it happens to guess correctly, because its correctness is accidental and its authority is
  counterfeit.
- **AI owns judgment under uncertainty** — the residual, irreducible domain where no algorithm
  suffices: ambiguity, open-ended proposal, natural-language translation, prioritization. Judgment
  under uncertainty is fallible *by definition*, so its output can only ever be a **proposal, never a
  fact**; its epistemic status is permanently *untrusted-until-validated.* The moment a plausible
  guess is allowed to *define* what is correct, the entire chain of justification collapses.
- **Humans own goals** — purpose, value, and the acceptance of risk, none of which can be derived from
  any layer below. To run near a thermal limit to hit a cost target is to decide *what matters*, and
  deciding what matters is exactly what no lower layer is competent to do. Delegating it downward is
  not efficiency; it is the abdication of the one thing that is not a fact at all.

Read as one chain, the hierarchy is a descent of warrant and an ascent of freedom: **nature → formal
method → codified practice → mechanical execution → judgment → purpose.** Each layer is testable and
replaceable in isolation precisely because it holds one kind of warrant and only one. **The cardinal
sin the architecture exists to prevent is the collapse of two layers into one** — the *laundering* of
one warrant through another's authority. Letting AI define correctness launders a guess as a fact;
letting practice override physics launders a convention as a law; letting mathematics assert a
physical value launders a hypothesis as a measurement; letting any layer decide *what for* launders a
value as a computation. Each collapse destroys the very property that made the system trustworthy: the
ability to say, of any claim, *exactly why it is warranted and exactly how far that warrant extends.*

---

## Part VII · The Engineering World Model

A runtime that owns engineering truth faces a choice of ambition. The lower ambition is a **knowledge
graph**: a queryable web of entities and static relations — *this net connects these pins, this part
complies with that clause.* It is a magnificent filing cabinet; its native verbs are *assert* and
*traverse*; it knows what *is.* That is necessary and not sufficient, because a filing cabinet does
not understand the thing it files. It cannot tell you that raising the clock will violate a setup-time
margin, or that removing a decoupling cap *would* introduce a rail sag that *would* trip a brownout
detector three subsystems away. Those are not lookups. They are **consequences**, and consequences do
not live in the edges of a static graph — they live in the laws and behaviors the graph omits.

The sharp distinction: **a knowledge graph stores structure; an Engineering World Model stores the
generator of structure.** The graph is a photograph of state; the world model is a physics engine for
the design — capable of running the design forward in the imagination, before it is run forward in
copper. To truly *understand* engineering, rather than merely record it, the runtime must own a world
model, of which the knowledge graph is only the static projection.

Its constituents, in ascending order of what they let the runtime *do*:

- **Objects** — the nouns (grounded in Part IV); in a world model, never inert data but *bearers of
  behavior.*
- **Relationships** — typed by the physics they conduct; "connected" is not one relation but a family
  (conducts current, propagates an edge, shares a return path, couples capacitively).
- **Physical Laws** — the invariants the model is *compelled to obey*, universal and
  engineer-independent. This is what a knowledge graph structurally cannot hold and what makes the
  model a *world*: embedding laws lets it *derive* consequences no one entered.
- **Constraints** — the bounds intent and practice layer atop the laws; *only meaningful against laws*
  ("stay under 85 °C" is inert until the model can compute the temperature that would obtain).
  Constraints are the questions; laws compute the answers; their meeting is verification.
- **Behaviors** — how objects act over conditions and time; where a fact is a point, a behavior is a
  *function over a domain* (temperature, load, voltage, frequency, age).
- **Interactions** — the couplings and emergent effects across domains, where real engineering fails:
  the regulator that is electrically correct but injects EMI that corrupts an analog line. A graph,
  additive in edges, is blind to emergence by construction; a world model, being generative, can
  surface the effect no component intended.

**Understanding, defined operationally.** A system understands a domain to exactly the degree that it
can (1) **predict consequences** — run the laws forward; (2) **detect contradiction** — recognize when
a proposed reality violates its own governance; (3) **answer counterfactuals** — respond to "what
*would* happen if…," the definitive test, because you cannot query a counterfactual out of a static
store; and (4) **explain why** — trace a consequence back through the laws and constraints that
produced it. Storing that a board exists is inventory; modeling how the board behaves is
understanding. This is the load-bearing link to trustworthy AI: a reasoner unconstrained by a world
model hallucinates plausible engineering; a reasoner *reasoning against* an owned model that predicts,
contradicts, and explains is **drivable**, because every proposal is checked against a reality the
runtime can simulate. The world model is the verification boundary made real.

**The honest limit.** No world model is complete; all models are approximations, and a runtime that
owns truth must not lie about its own reach. The resolution is to make **fidelity itself a
first-class, owned, versioned, improvable object** — the model must know *how well* it models, which
behaviors are high-fidelity and which are stubs, what physics is in-scope and what is deferred. A
world model can say "I do not model EMI yet"; a knowledge graph cannot even form the sentence. This is
the right north star: a static graph is a *finished* thing with a low ceiling; a world model is an
*unfinished* thing with no ceiling, where every increment of fidelity buys new understanding. **The
ambition is the world model even as the reality is a deepening approximation** — and the gap between
them is not failure but the roadmap.

---

## Part VIII · Engineering Correctness

Correctness deserves its own statement, because it is where every part of this philosophy meets.
**Engineering correctness is the simultaneous satisfaction of every governing constraint and fidelity
to the originating intent — established by construction, maintained continuously, and provable on
demand.**

Four claims are packed into that sentence, and all four are non-negotiable:

1. **Both halves, always.** Constraint-satisfaction without fidelity-to-intent is a design that
   correctly implements the wrong thing; fidelity without constraint-satisfaction is a good idea that
   burns. Correctness requires both, and neither alone is even close.
2. **A property of the whole.** Correctness is not the conjunction of locally-correct parts, because
   reality couples what abstraction separates. It is a property of the entire owned model, evaluated
   against the world model that predicts its behavior.
3. **By construction, not by inspection.** The mature form of correctness is that invalid states are
   *unreachable* — the seam refuses them — not that they are *caught late.* This is the compiler
   engineering never had, finally built: a mechanical arbiter that can say *no, this violates a law,
   you may not commit it.* An end-stage check confirms a correctness that was maintained throughout, or
   discovers, expensively, that it was lost long ago.
4. **Provable and re-derivable.** Because truth is owned, versioned, and replayable, a correct design
   is not merely asserted correct — it can be re-derived and re-verified by anyone, offline, forever.
   "Correct" becomes a checkable property of a model, not a signature on a drawing.

Correctness, so defined, is the point where truth (Part II), sovereignty (Part III), objecthood
(Part IV), the language (Part V), the division of authority (Part VI), and the world model (Part VII)
are no longer separate ideas but a single machine: the runtime owns typed objects, expresses their
constraints in a checkable language, evaluates them against a world model bounded by physics and
practice, admits stochastic proposals only through a validating seam, and commits only what survives —
recording why, forever.

---

## Part IX · The principles that will never change

These are constitutional. Technologies, models, languages, and markets will change; these will not. A
future decision that requires violating one of these is not an optimization — it is a departure from
Electronics Agent Kit.

1. **Engineering is an owned, executable model — never a dead artifact.** The picture is a projection
   of the truth; it is never the truth.
2. **The runtime is sovereign over engineering truth** and over the lawful path by which truth
   changes. There is exactly one authority and exactly one write path.
3. **No truth without provenance.** Every fact and decision traces to its evidence and, transitively,
   to intent. Provenance is written *with* the change, never reconstructed after.
4. **Correctness is enforced by construction, not suggested by inspection.** Invalid state is
   unreachable; the safety model is never "propose, and a human will catch it."
5. **Reasoning proposes; the deterministic core disposes.** Stochastic judgment reaches truth only
   through a validating seam. Trust is a property of the seam, never of the proposer.
6. **Determinism and replay are inviolable.** Every design is re-derivable and auditable offline.
   Nothing may introduce hidden nondeterminism into committed truth.
7. **Authority is layered and never laundered.** Nature, formal method, codified practice, mechanical
   execution, judgment, and purpose each own their domain; none may usurp another's warrant.
8. **Physical quantities are typed.** Dimensional correctness is enforced by construction; a value
   always knows what it is.
9. **Honesty over fabrication.** The system never invents intent, knowledge, or provenance it does not
   have; it distinguishes what it *knows* from what it *infers*, and says which.
10. **If it isn't an object, it isn't owned; if it isn't owned, it's lost** — tempered by: *objecthood
    is conferred by consequence, not by category.*
11. **Humans own the goals.** Purpose, value, and the acceptance of risk are never delegated to any
    layer below.
12. **The model of the world is an owned, honest, deepening approximation** — never a static store
    pretending to understanding it does not have.

---

## Part X · Why this product exists

Every discipline that learned to own an executable model of its own work was transformed by it.
Accounting became owned when the ledger became a system of record that enforced its own invariants.
Software became owned when text became code a compiler could check. In each case the shift was not a
better tool for an old task; it was a new substrate that made a class of error *impossible*, made
knowledge *accumulate* instead of evaporate, and made trust *structural* instead of personal.

Electronics engineering never had this. It has world-class instruments for *drawing* the artifact and
nothing that owns the *engineering* — the intent, the constraints, the reasoning, the correctness, the
memory. So its knowledge evaporates when people leave, its correctness is checked late by tired humans,
its rationale is untraceable, its designs are dead endpoints that cannot be re-derived, and its most
powerful new instrument — AI — is untrustworthy precisely because there is no owned truth for it to be
checked against.

**Electronics Agent Kit exists to give electronics engineering the substrate it never had:** an owned,
executable, sovereign runtime that holds engineering truth as first-class living objects, enforces
correctness by construction against a world model bounded by physics and practice, preserves every
reason forever, and turns AI from a liability into leverage by admitting it only through a seam that
verifies everything it proposes. Not a better way to draw boards — a new category, in which the
engineering itself, and not a corpse of it, is finally owned.

Every year that more of engineering moves from human heads and dead files into owned, typed, verified
truth is a year the discipline becomes what programming became when text learned to run. That
monotonic transfer *is* the mission, and this document is its constitution.

---

## Appendix · The canonical questions, answered

**What is Engineering?** The disciplined transformation of intent and constraint into a physically
realizable artifact that behaves correctly under the laws of reality — distinct from science (which
describes) and craft (which cannot transfer its justification).

**What is Engineering Intelligence?** The capacity to reason correctly about physical systems under
uncertainty toward an intended purpose *and to justify each step*; for a system, the union of owning a
model of engineering truth, reasoning over it toward purpose, and verifying its conclusions against
reality — refusing what violates the laws and saying why.

**What is Engineering Truth?** A claim that is justified, provenance-bearing, and
revisable-but-auditable — a third epistemic category: local and purposive like nothing in pure science
or pure proof. It exists only where its relations are kept whole, which is why it must be owned.

**What is Engineering Knowledge?** Justified rules that govern action — facts bound to consequences,
conditions, and rationale — accumulated as a queryable, first-class asset rather than lore that leaves
with the engineer.

**What is Engineering Runtime?** The sovereign substrate that owns intent, state, memory, constraints,
verification, reasoning boundaries, planning, and knowledge, and exposes them only through lawful,
validated, recorded operations — an operating system whose resource is engineering truth itself.

**What is Engineering Language?** The formal, typed, denotational, inference-closed representation in
which the runtime expresses engineering so it can check it — one coherent family (intent, requirement,
constraint, planning, verification, reasoning, execution, review) differing by illocutionary force. A
language the runtime *is*, before any language a human writes.

**What is Engineering World Model?** A model that stores not just structure but the *generator* of
structure — objects, relationships, physical laws, constraints, behaviors, and interactions — rich
enough to predict, contradict, answer counterfactuals, and explain. It is how the runtime
*understands* rather than merely records, and it is the reality against which AI is made trustworthy.

**What is Engineering Correctness?** Simultaneous satisfaction of every governing constraint and
fidelity to intent — a property of the whole, established by construction, maintained continuously,
and provable on demand. The compiler engineering never had, finally built.

**Why does this product exist?** To give electronics engineering the owned, executable substrate every
mature discipline eventually acquires — so that knowledge stops evaporating, correctness is structural,
reasoning is traceable, designs are re-derivable, and AI becomes leverage rather than liability.

**What principles will never change?** The twelve in Part IX — sovereign owned truth, provenance
always, correctness by construction, propose-then-dispose, determinism and replay, unlaundered layered
authority, typed quantities, honesty over fabrication, objecthood by consequence, humans own goals, and
the world model as an honest deepening approximation.

---

*End of canonical philosophy. It grounds every decision in `00-product-vision.md` and everything
beneath it. Companion validation of the repository against this philosophy: see
`project-plans/10-philosophy-alignment-report.md`.*
