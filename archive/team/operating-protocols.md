# EAK Autonomous Engineering Team — Operating Protocols

> Anchored to [`00-plan-and-requirements.md`](00-plan-and-requirements.md) (source of truth) and the
> roster/RACI in [`org-structure.md`](org-structure.md). These are the enforceable ways of working:
> concrete rules, not aspirations. Each maps to a requirement R1–R10. Written 2026-07-02.

---

## 1. Canonical-first fan-out (R5)

**Rule:** a source-of-truth artifact is written and frozen **before** any parallel work fans out
against it. No agent starts building against a moving interface.

- The order is always: (a) owning senior role drafts the canonical doc/interface → (b) Architect (or
  Eng Lead) reviews and freezes it → (c) Eng Lead fans work items out to squads.
- Canonical artifacts and their owners: the **kernel event contract** (Architect), **IR phase schemas**
  (Kernel Engineer), **product spec / hero-demo definition** (Product Manager), **this protocol set**
  (Eng Lead).
- A frozen canonical artifact is edited only via §4's version-bump path — never in place.
- Litmus test before dispatch: *"Is the seam every downstream agent keys off of written down and
  frozen?"* If no, do not fan out.

## 2. Sole-writer-per-file + git-worktree isolation (R6)

**Rule:** exactly one agent may write any given file at a time; concurrent code mutation happens in
isolated git worktrees.

- The Eng Lead partitions each dispatched batch by **file/component ownership** and records the
  owner. Two agents never receive overlapping file sets.
- When two+ agents mutate code concurrently, each gets its own worktree + branch:
  ```
  git worktree add ../eak-<squad>-<item>  -b feat/<squad>/<item>
  # agent works only inside its worktree, only on its owned files
  git worktree remove ../eak-<squad>-<item>   # after merge
  ```
- Shared/append-only files (e.g. a manifest, a lockfile) get a **single designated writer**; other
  agents request the edit through that writer rather than touching it.
- Merges happen at the **frozen seam** only (§4), so branches integrate cleanly instead of colliding.

## 3. The green-gate (R7)

**Rule:** every commit is green. The pipeline below runs before any commit; a failure at any stage
means **commit-green-or-revert** — never leave the tree red.

| # | Stage | Command / action | Gatekeeper (RACI) |
|---|-------|------------------|-------------------|
| 1 | build | `cargo build` (+ `eak-app` desktop bundle build in CI) | eak-rust-reviewer |
| 2 | lint  | `cargo clippy -- -D warnings` + `cargo fmt --check` | eak-rust-reviewer |
| 3 | test  | `cargo test` (all pass; new behavior has a new test) | eak-qa-test-engineer |
| 4 | domain review | the relevant reviewer signs off (§6) | per-domain reviewer |
| 5 | commit | commit **only if 1–4 pass**; else revert the change | eak-eng-lead |
| 6 | push  | push to canonical remote after green | eak-eng-lead |

- `cargo test` count is a ratchet: it may go **up**, never down, without an explicit approved reason.
- Kernel green (R2) is non-negotiable — the moat's whole value is that it is always correct.
- CI mirrors stages 1–3; a red CI blocks the push even if local was green.

## 4. Hand-off contracts — freeze-then-parallelize (R8)

**Rule:** squads integrate **only** through fixed seams; a seam is frozen before both sides build,
then built in parallel.

- **The two seams:**
  1. **Kernel event stream = the frontend↔backend contract.** The serialized `eak_ports::Event`
     stream (command envelope in; projection/diagnostic/event envelope out, `schemaVersion`-tagged) is
     the only way the IDE and the kernel talk.
  2. **IR schemas = the phase seams.** Each pipeline phase hands off a typed IR; downstream phases
     depend on the schema, not the producing code.
- **Freeze-then-parallelize:** write the seam → Architect signs off → tag it (`v1`) → Application
  builds against a recorded **event cassette** while Kernel builds the real emitter → converge by merge.
- **Changing a frozen seam** requires: an ADR, an Architect sign-off, and a **version bump** (`v1`→`v2`)
  — never an in-place edit. Downstream owners are notified (they are `I`/`C` on that seam in the RACI).

## 5. Definition-of-Done template (every role fills this)

A work item is **Done** only when every box is checked. Reviewers reject items with unchecked boxes.

```
DoD — <work-item>  (owner: <role-id>, squad: <squad>)
[ ] Scope: item traces to a roadmap row/deliverable and passes the §8 hero-demo test
[ ] Canonical: built against a frozen seam/spec (R5); no moving interfaces touched
[ ] Sole-writer: only my owned files changed; worktree isolated if concurrent (R6)
[ ] Green-gate: build + clippy -D warnings + fmt + test all pass locally and in CI (R7)
[ ] Tests: new/changed behavior has a test; test count did not drop
[ ] Domain review: the gating reviewer for this axis approved (§6)
[ ] Contract: if a seam changed, ADR + version bump + Architect sign-off (R8)
[ ] Traceability/moat: no weakening of determinism, LLM-boundary, or replay (R4)
[ ] Honesty: REAL vs CURATED vs CASSETTE vs REUSED vs FAKE labeled (roadmap legend)
[ ] Reported: status posted to Eng Lead/TPM; escalations (if any) raised per §7
```

## 6. Review gates — which reviewer gates what

| Change touches…                          | Gating reviewer (must approve)        |
|------------------------------------------|---------------------------------------|
| any Rust (build/clippy/fmt correctness)  | `eak-rust-reviewer`                    |
| kernel rules / physics / IPC correctness | `eak-verification-engineer` + `eak-eda-domain-scientist` |
| tests, fixtures, cassettes, coverage     | `eak-qa-test-engineer`                 |
| LLM boundary, secrets, deps/supply-chain | `eak-security-reviewer`               |
| a frozen seam (event contract / IR)      | `eak-architect` (+ version bump)      |
| IDE UI / visual taste                    | `eak-frontend-engineer` + `eak-design-lead` |
| scope / hero-demo curation               | `eak-product-manager`                 |

- A change can require **multiple** gates (e.g. a new DRC rule with a live-LLM explanation needs
  rust-reviewer + verification + security). All required gates must pass before the Eng Lead commits.
- Reviewers gate on their axis only; the Eng Lead owns the final commit-green decision (§3).

## 7. Escalation rules (R9) — escalate, don't guess

Default behavior: for anything that isn't a genuine fork, the agent **picks a sensible default, states
it explicitly, and proceeds.** Escalate only real forks, to the right owner:

| Fork type | Goes to | Examples |
|-----------|---------|----------|
| Product / scope / priority / "is this in the demo?" | **Founder/CEO** (via Product Manager) | cut a feature, change the hero example, spend real money on a parts API |
| Architecture / seam / boundary / ring violation | **eak-architect** | changing a frozen contract, kernel↔app boundary question, new outer-ring crate |
| Cross-squad dependency / critical-path slip | **eak-eng-lead + eak-tpm** | a gate week (W2/W3/W5/W8) at risk, a squad blocked on another |
| Everything else | **stays in the squad** | naming, local refactor, test structure, default params (state the default) |

- **Never guess on the moat.** Determinism, the LLM boundary, traceability, and replay (R4) are not
  agent-discretion — any change that could weaken them escalates to the Architect.
- Escalations are one message: *context · the fork · options · the agent's recommendation.*

## 8. Scope-discipline guardrails (R10)

**The test — for every proposed work item:** *"Does this appear in the hero demo (intent → generate →
AI-review, traceable, on the curated example), or is it the moat that makes the demo trustworthy?"*
If neither, **don't build it** — reuse or defer.

- **Reuse over build** for anything that isn't the moat: canvas (KiCanvas), board formats (KiCad),
  parts data (Nexar/Octopart), landing/waitlist tooling. Build only the kernel + harness + thin shell.
- **Explicit non-goals — resist drift back into these** (per `00` §2): general autorouting, a
  KiCad-beating editor, broad part coverage, cloud/collaboration. These are the funded-year, not the MVP.
- The Product Manager owns this guardrail; any agent proposing out-of-scope work must escalate it as a
  scope fork (§7), not just start building.

## 9. Increment cadence

**Rule:** small, shippable, green increments — mirror the kernel's proven increment discipline (the
existing history is a chain of `Phase N (increment M): <one capability>` green commits).

- One increment = one coherent capability behind the green-gate, committed and pushed before the next.
- Commit message style: `Phase <n> (increment <m>): <capability>` for kernel/app work; keep each
  commit revertable in isolation.
- Prefer many small green commits over one large risky merge — it keeps `git bisect` cheap and the
  tree always demoable (there is always a runnable build to show, per roadmap §8).
- A gate week (W2/W3/W5/W8) ends with the gate's increment landed green, or the contingency (roadmap
  §7) is invoked — polish is cut before the gate.

## 10. The standard work loop

Every dispatched item runs this loop; the Eng Lead runs it for the batch.

```
  intake ──▶ design ──▶ fan-out ──▶ gate ──▶ integrate ──▶ report
    │          │           │          │          │            │
 founder    owning       Eng Lead   L4 green-  Eng Lead     TPM updates
 intent →   senior       routes to  gate (§3)  merges at    critical path;
 Eng Lead   writes &     squads,    + domain   the frozen   escalations (§7)
 + TPM →    freezes the  worktree-  review     seam (§4),   go to founder;
 work       canonical    isolated   (§6) per   commit-      loop back to
 items      seam (§1)    (§2)       item DoD   green-or-    intake for the
            (R5)         (R6)       (§5/§6)    revert (R7)  next increment
```

1. **Intake** — founder states intent; Eng Lead + TPM turn it into roadmap-mapped work items.
2. **Design** — the owning senior role writes and freezes the canonical seam/interface (§1, R5).
3. **Fan-out** — Eng Lead dispatches items to squads, sole-writer-partitioned, worktree-isolated (§2).
4. **Gate** — each item passes the green-gate + its domain review against the DoD (§3, §5, §6).
5. **Integrate** — Eng Lead merges at the frozen seam, resolves cross-seam issues with the Architect,
   commits-green-or-reverts, pushes to the canonical remote (§4, R7).
6. **Report** — TPM updates status vs the critical path; escalations go to the founder (§7); the loop
   returns to intake for the next increment (§9).
