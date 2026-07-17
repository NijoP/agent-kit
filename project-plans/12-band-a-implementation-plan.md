# Band A (Phase 3) — Implementation Plan (epistemic Maps)

> The file-level construction plan for the **next phase of EAK development**: the epistemic Band A.
> Anchored to `00-product-vision.md` (§5 responsibilities 2/4/5, Principle 7 honesty),
> `01-engineering-philosophy.md` (Parts II/IV — the judgment objects), `02-engineering-world-model.md`
> (§Band A, Maps 4/6/10/11/46), and `11-build-roadmap.md` (Phase 3, exit criteria). Written 2026-07-17.
> **No code yet** — this document is for review; on approval the squad builds increment-by-increment.

---

## 0. Why this is the next phase

Phases 0–2 are **built and green in the tree today**: 11 crates, 236 tests, the deterministic
substrate + skeleton Maps (Intent→…→Track) + IR + verification + the global manufacturing gate.
Verified in code: `eak-domain/src/lib.rs` (objects), `eak-ports/src/lib.rs` (`Event`),
`eak-runtime/src/{protocol.rs,state.rs,runtime_core.rs}` (seam + fold + commit),
`eak-phases/src/manufacturing_generation.rs` (the global gate).

Band A is the **cheapest, highest-leverage** next brick, and — critically — it is **buildable in this
environment**: pure kernel Rust (domain objects + events + gates), **no heavy numerics, no
frontend/Tauri, no webkit2gtk**. It models what the runtime does not yet own: *the reasoning about
the design*. It is the layer that makes the AI **demonstrably honest** (it must declare and discharge
its assumptions; release blocks while a critical one is open) — the marquee "AI you can trust" story.

Every Band-A object is a **repetition of the proven Phase-2 pattern** through the same seam — new
risk is content, not structure (`02 §Part III`).

### Exit criteria (from `11 §Phase 3`) — the plan is done when all three hold
1. A run **records its assumptions and refuses to release with an undischarged *critical* one**.
2. **Every predicted/derived fact carries a fidelity tag.**
3. **A rejected alternative is preserved as a Tradeoff.**

---

## 1. The construction pattern (the reusable 7-touchpoint recipe)

Every Band-A object follows the exact pattern the codebase already uses for `Constraint`, `Violation`,
`Waiver`, etc. Each increment touches these seven points, tests-first:

| # | File | What is added | Existing exemplar to copy |
|---|---|---|---|
| 1 | `eak-domain/src/lib.rs` | the `struct` (`id: EntityId` + fields) + status/kind `enum`s + a `validate() -> Result<(), DomainError>` | `Constraint` (L179) + `ConstraintStatus` (L170) |
| 2 | `eak-ports/src/lib.rs` | new `Event` variant(s) | `ConstraintCommitted` / `WaiverGranted` |
| 3 | `eak-runtime/src/state.rs` | field on `EngineeringState` + a fold arm in `apply()` + an accessor + (if it gates) a query method | `constraints` + arm L78 + `open_blocking_violations()` L188 |
| 4 | `eak-runtime/src/protocol.rs` | `CapabilityRequest` variant(s) + an `AgentContext` reader | `CreateConstraint` (L56) + `constraints()` (L153) |
| 5 | `eak-runtime/src/runtime_core.rs` | a `handle_*` seam handler that **re-validates then emits**, wired into the `invoke` dispatch | `handle_create_constraint` (L148) |
| 6 | `eak-phases/…` | gate/rule change (only inc. 1 touches the global gate) | `manufacturing_generation.rs` CheckingGate (L57) |
| 7 | tests (each crate) | domain-validate · seam-accept · **seam-reject** · fold · **byte-identical replay** · gate · e2e | mirror the Phase-2 test suites |

**Reuse `DomainError`** — new objects reuse `EmptyStatement` / `EmptyField(&'static str)` /
`Inconsistent(&'static str)` rather than inventing variants (as `Constraint::validate` already does).
**Fold discipline:** any new *state-bearing* `Event` variant MUST get an explicit `apply()` arm — the
`_ => {}` catch-all in `state.rs:118` will otherwise silently diverge on replay (P4). This is the one
sharp edge; every increment's replay test guards it.

---

## 2. Increment sequence (one object per increment)

Ordered by value and dependency. Each is independently revertable, green-gated, and replay-verified.

### Increment 1 — **Assumption** + the honesty gate  *(the headline; delivers exit criterion 1)*

Map 10. `Assumption{statement, rests-on, status}`, dischargeable → grounded fact / enforced
constraint / accepted risk.

- **Domain** (`eak-domain`):
  ```
  enum AssumptionCriticality { Critical, Normal }
  enum AssumptionStatus      { Open, Discharged, Invalidated }
  enum DischargeResolution   { GroundedFact, EnforcedConstraint, AcceptedRisk }
  struct Assumption {
      id: EntityId, statement: String,
      rests_on: EntityId,               // the reasoning step / decision / requirement it presumes on
      criticality: AssumptionCriticality,
      status: AssumptionStatus,
      discharge: Option<Discharge>,     // set when discharged
  }
  struct Discharge { resolution: DischargeResolution, target: EntityId, decided_by: String }
  // validate(): non-empty statement (reuse EmptyStatement); Discharge required iff status==Discharged
  ```
- **Events** (`eak-ports`): `AssumptionRaised { assumption }`, `AssumptionDischarged { assumption: EntityId, discharge: Discharge }`.
- **State** (`state.rs`): `assumptions: Vec<Assumption>`; two fold arms (raise pushes; discharge finds by id, flips `status`, sets `discharge`); accessor `assumption(id)`; **gate query** `undischarged_critical_assumptions() -> Vec<&Assumption>` (mirror of `open_blocking_violations`).
- **Protocol** (`protocol.rs`): `RaiseAssumption { assumption, links }`, `DischargeAssumption { assumption: EntityId, discharge: Discharge }`; reader `assumptions()`.
- **Seam** (`runtime_core.rs`): `handle_raise_assumption` re-validates (non-empty statement, `rests_on` resolves to an existing entity); `handle_discharge_assumption` requires the target assumption to exist and be `Open`, and the `Discharge.target` entity to exist. Reject → nothing enters the log.
- **Gate** (`manufacturing_generation.rs` CheckingGate): extend the block condition — release is refused if `open_blocking_violations > 0` **OR** `undischarged_critical_assumptions > 0`, with a distinct message. This is the honesty gate.
- **ADR-0018** — Assumption object, discharge semantics, and the gate extension.
- **Tests:** validate (empty stmt rejected; Discharged-without-Discharge rejected) · seam accepts valid raise · seam rejects empty/dangling `rests_on` · discharge flips status + records resolution · gate **blocks** with an open Critical · gate **passes** when it is discharged, and when only Normal remain open · **replay byte-identical** with assumptions in the log · e2e: a full pipeline run that raises then discharges a critical assumption releases; one that leaves it open is `Blocked`.
- **Exit:** exit criterion 1 met.

### Increment 2 — **ModelFidelity** (a tag on derived facts)  *(delivers exit criterion 2)*

Map 6. `ModelFidelity{concern, method, confidence, scope}`. A trust-tag on every derived fact.
Modeled as **advisory attached metadata**, exactly like the existing `ViolationExplanation` pattern
(`state.rs:23`) — it carries **no `EntityId` of its own**, references a target, and folds into its own
store, so it can never usurp an object's authority.

- **Domain:** `enum FidelityMethod { Assumed, FirstOrderFloor, Calculated, Simulated, Measured }`; `struct ModelFidelity { concern: String, method: FidelityMethod, confidence: f64, scope: String }`. Invariant: `confidence ∈ [0,1]`.
- **Event:** `FidelityTagged { target: EntityId, fidelity: ModelFidelity, reasoning_call_seq: Option<Seq> }`.
- **State:** `fidelity_tags: Vec<(EntityId, ModelFidelity)>` (insertion order); accessor `fidelity_for(target)`.
- **Boundary:** validated at emit (confidence range) — mirrors how `ViolationExplained` is emitted as audit/advisory metadata rather than a full capability object. (Option considered & rejected for v0: a first-class `Fidelity` entity with its own id — heavier, no lifecycle need yet.)
- **Rule (non-blocking):** an optional `Warning`-severity check "derived fact carries no fidelity tag" so the honesty is *visible*; **not** a release blocker in v0.
- **ADR:** folded into ADR-0018 or a short ADR-0019 note (tag-not-entity decision).
- **Tests:** confidence-range validation · fold · accessor · **replay byte-identical** · the warning rule fires on an untagged derived fact.
- **Exit:** exit criterion 2 met (every predicted fact the pipeline emits carries a tag; the rule proves coverage).

### Increment 3 — **Risk**  *(auditable risk posture; Principle 11)*

Map 46. `Risk{likelihood, severity, mitigation, residual, owner}`; the **human owns acceptance** of
residual risk (`00` Principle 11 — humans own goals).

- **Domain:** `enum RiskLikelihood { Low, Medium, High }`, `enum RiskSeverity { Low, Medium, High, Critical }`, `enum RiskStatus { Open, Mitigated, Accepted }`; `struct Risk { id, statement, likelihood, severity, mitigation: String, residual: RiskSeverity, owner: String, status }`. validate(): non-empty statement + owner.
- **Events:** `RiskRaised { risk }`, `RiskAccepted { risk: EntityId, accepted_by: String }`.
- **State:** `risks: Vec<Risk>`; accessor; query `unaccepted_critical_risks()` (read-only in v0).
- **Protocol/seam:** `RaiseRisk { risk, links }`, `AcceptRisk { risk, accepted_by }`; `AcceptRisk` requires the risk to exist and folds `status → Accepted`. Reader `risks()`.
- **Gate:** **does NOT block release in v0** — risk is *tracked truth*; the human owns acceptance. (Deepening later: auto-aggregate Risk from open Assumptions/Violations, and optionally gate on unaccepted Critical residual.)
- **ADR-0020** — Risk object + the human-acceptance authority boundary.
- **Tests:** validate · seam accept/reject · fold (raise + accept transition) · **replay** · aggregation read (`unaccepted_critical_risks`).

### Increment 4 — **Objective / Tradeoff**  *(delivers exit criterion 3)*

Map 11. `Objective`, `Tradeoff{alternatives, criteria, chosen, rejected, rationale}` — the
weighed-and-rejected space, preserved.

- **Domain:** `struct Objective { id, statement, weight: f64, source: EntityId }`; `struct Alternative { label, description, scores: Vec<f64>, rejected: bool }`; `struct Tradeoff { id, question, alternatives: Vec<Alternative>, criteria: Vec<String>, chosen: usize, rationale: String, decided_by: String }`. validate(): ≥2 alternatives · `chosen` in range · the chosen one is not `rejected` · ≥1 preserved rejected alternative (reuse `Inconsistent`).
- **Events:** `ObjectiveRecorded { objective }`, `TradeoffRecorded { tradeoff }`.
- **State:** `objectives`, `tradeoffs`; accessors.
- **Protocol/seam:** `RecordObjective { objective, links }`, `RecordTradeoff { tradeoff, links }`; validation as above. Readers `objectives()`, `tradeoffs()`.
- **Link:** a `Decision` may cite the `Tradeoff` it resolved via a `ProvenanceLink` (`RelationType`) — ties the choice to its rejected space (`02 §Map 11`).
- **ADR-0021.**
- **Tests:** validate (reject single-alternative; reject no-rejected-preserved; reject chosen-is-rejected) · seam · fold · **replay** · Decision→Tradeoff link resolves.
- **Exit:** exit criterion 3 met.

### Increment 5 — **Change/Revision spine** *(optional / deepen — git-for-hardware v0)*

Map 4. Implements the already-accepted **ADR-0008** (design version control). Heavier: reads event-log
ranges (`eak-store`/`eak-runtime`), so it is sequenced **last** and may be deferred to the Stage-1 tail.

- **v0 scope only:** `struct Revision { id, label: String, at_seq: Seq, message: String }` (a named tag on a log position) + a semantic `Diff` between two revisions (entities added/changed/superseded, computed by folding the two prefixes). **Branch/Merge deferred.**
- **Event:** `RevisionTagged { revision }`. **State:** `revisions: Vec<Revision>`.
- **Tests:** tag a revision · diff two revisions of the same design · **replay**.
- **Recommendation:** ship increments 1–4 first (they satisfy all three exit criteria); treat 5 as a fast-follow.

---

## 3. Ordering rationale (why not permute)

Increment **1 (Assumption)** is first: it alone lands the marquee capability and the gate that defines
"honest AI." **2 (Fidelity)** is next because it is the cheapest and it is a *tag* every later
predicted fact (esp. Band C) will carry — build the tag before there are many facts to tag.
**3 (Risk)** and **4 (Tradeoff)** are siblings of `Decision`/`Evidence` and slot straight onto the
seam; Risk before Tradeoff only because Assumption-discharge can resolve *to* an accepted Risk (inc. 1
references the concept; inc. 3 makes it a real object). **5** is last (log-range machinery).

Soft parallelism: 3 and 4 can be built concurrently once 1's ADR freezes the seam-addition convention.

---

## 4. Scope guard — what Band A deliberately does NOT do

To keep the kernel small and sovereign (`00 §12.3`, the boil-the-ocean risk):

- **No heavy numerics / no solver** — Fidelity is a *tag*, not a computed confidence; that is Band C's
  `eak-solvers` boundary.
- **No new IR band** — the Logical-Electrical IR is Band B.
- **No cross-project memory** — `eak-memory` is Band D.
- **No frontend / Tauri / renderer** — surfacing is Phase 4 and needs the founder's machine; nothing
  here does.
- **No auto-derived Risk aggregation** in v0 — Risk is entered and owned; aggregation is a later deepen.

If an increment starts reaching for any of the above, it has left Band A.

---

## 5. Cross-cutting disciplines (hold on every increment)

- **Tests/fixtures first** — a failing test before the object it verifies.
- **Green-gate every increment** — `cargo build` + `clippy -D warnings` + `fmt` + tests; the **test
  count ratchets up, never down**; commit only green. Expect 236 → ~275+ across increments 1–4.
- **Replay-as-CI** — each increment's exit gate includes a **byte-identical replay** assertion
  (`EngineeringState::canonical_json` equality across a re-fold). Determinism is a test, not a hope.
- **Canonical-first** — the Band-A seam additions are frozen by an ADR **before** parallel work forks;
  changing a frozen seam later is an ADR + version bump, never an edit.
- **One object per increment** — each Map added object-by-object through the seam, independently
  revertable.

---

## 6. Squad / RACI

| Agent | Responsibility |
|---|---|
| `eak-architect` | Writes **ADR-0018–0021**; freezes each increment's seam contract before code forks. |
| `eak-kernel-engineer` | Domain objects, `Event` variants, fold arms, seam handlers, `AgentContext` readers. |
| `eak-qa-test-engineer` | Fixtures + tests **first** (TDD); the replay byte-identity assertions. |
| `eak-verification-engineer` | The honesty-gate extension (inc. 1) + the non-blocking fidelity rule (inc. 2). |
| `eak-rust-reviewer` | Green-gate each increment (ownership/clippy/fmt) before commit. |
| `eak-harness-engineer` | *(fast-follow)* wire the reasoning agents to actually **raise** assumptions during a run + a cassette, so the demo shows AI declaring its own assumptions. |
| `eak-eng-lead` | Orchestrates the intake→fan-out→gate→integrate loop across increments. |

---

## 7. Risks & open questions (decide before/at inc. 1's ADR)

1. **Who raises assumptions?** v0 can seed them via a capability call from a phase; the *honest* story
   is the reasoning agent declaring them (harness-engineer fast-follow). Decide whether inc. 1 ships
   with a live agent path or a fixture/cassette that raises one. *(Recommend: fixture in inc. 1, live
   agent as the fast-follow — keeps inc. 1 pure-kernel and deterministic.)*
2. **Fidelity as tag vs entity** — plan assumes *tag* (advisory store). Confirm in the ADR; reversing
   later is a bigger change.
3. **Does the honesty gate block on Critical only, or on any open assumption?** Plan says **Critical
   only** (matches `11`'s "undischarged *critical*"). Normal open assumptions are surfaced, not
   blocking.
4. **Discharge-to-Risk coupling** — inc. 1 references `AcceptedRisk` as a discharge resolution but
   `Risk` lands in inc. 3. v0: `Discharge.target` is a free `EntityId`; it only *must* resolve to a
   `Risk` once inc. 3 exists (tighten the seam check then).

---

*On approval: `eak-architect` writes ADR-0018 and freezes increment 1's seam; `eak-qa` writes the
failing tests; `eak-kernel-engineer` builds to green. Nothing is committed except green, replay-verified
increments. This plan anchors to `00-product-vision.md`; where it and the vision ever disagree, the
vision wins.*
