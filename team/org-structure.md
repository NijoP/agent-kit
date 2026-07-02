# EAK Autonomous Engineering Team — Org Structure

> Anchored to [`00-plan-and-requirements.md`](00-plan-and-requirements.md) (source of truth). Uses its
> exact 5 layers, 17 role ids, and requirements R1–R10. If this file drifts from that one, that one wins.
> Companion: [`operating-protocols.md`](operating-protocols.md) (ways of working). Written 2026-07-02.

---

## 1. Org chart — star-of-squads, 5 layers, 17 roles

The dispatch model is a **star**: `eak-eng-lead` (Orchestrator) is the hub. It decomposes the roadmap,
dispatches work items to the owning squad, and integrates the results behind frozen contracts. The
**Architect** guards cross-seam coherence; the **TPM** tracks the critical path; the **Quality chapter**
gates every change regardless of which squad produced it; **Product/GTM** runs the raise arc in parallel.

```
                         ┌───────────────────────────────────┐
                         │        FOUNDER / CEO (human)       │
                         │   intent · scope calls · escalations
                         └──────────────────┬────────────────┘
                                            │ intent in / escalations up
        L1 ┌───────────────┐    ┌───────────▼───────────┐    ┌───────────────┐
 Leadership │ eak-architect │◀──▶│      eak-eng-lead     │◀──▶│    eak-tpm    │
     &      │  seam guard,  │coh.│    (ORCHESTRATOR)     │stat│ milestone plan│
   Coord.   │  ADRs, rings  │    │  decompose·dispatch·  │    │ deps · crit.  │
            └───────────────┘    │  integrate·green-gate │    │ path · status │
                                 └───┬──────┬──────┬─────┘    └───────────────┘
                    dispatch work items │      │      │ dispatch work items
              ┌─────────────────────────┘      │      └──────────────────────────┐
              ▼                                 ▼                                 ▼
   L2 ── KERNEL SQUAD ─────────   L3 ── APPLICATION SQUAD ──────   L5 ── PRODUCT / GTM SQUAD ──
   owner: eak-kernel-engineer     owner: eak-desktop-engineer      owner: eak-product-manager
   ┌───────────────────────┐     ┌────────────────────────────┐   ┌────────────────────────┐
   │ eak-kernel-engineer   │     │ eak-desktop-engineer       │   │ eak-product-manager    │
   │ eak-verification-eng  │     │ eak-frontend-engineer      │   │ eak-market-analyst     │
   │ eak-eda-domain-sci    │     │ eak-harness-engineer       │   │ eak-fundraise-lead     │
   └───────────────────────┘     │ eak-canvas-integration-eng │   │ eak-design-lead        │
                                 └────────────────────────────┘   └────────────────────────┘
              │                                 │                                 │
              └────────────┬────────────────────┴───────────────┬────────────────┘
                           ▼   every code/doc change flows through ▼
   L4 ═══════════════ QUALITY & RELIABILITY CHAPTER (spans all squads) ═══════════════
       owner: eak-rust-reviewer  ·  eak-qa-test-engineer  ·  eak-security-reviewer
       gate: build → clippy -D warnings → test → domain review → commit-green-or-revert (R7)
```

Reporting / coordination lines:
- All three **squad owners** report delivery status to `eak-eng-lead`; `eak-tpm` aggregates it against
  `project-plans/03-roadmap.md`.
- `eak-architect` is **not** in a squad — it is a lateral authority the Eng Lead consults on every seam,
  and it can block a merge that violates the clean-architecture rings or the kernel↔app boundary (R4).
- The **Quality chapter** (L4) also spans squads laterally: it reviews every change and holds the
  green-gate; it reports gate pass/fail to the Eng Lead, not to any single squad.
- The **Founder/CEO** sits above the Eng Lead: sets intent, owns scope calls, receives R9 escalations.

---

## 2. Squads (charter · members · owned area · accountable owner)

### Leadership & Coordination (L1) — the hub
- **Charter:** convert founder intent into dispatched, integrated, green work; keep the whole build
  coherent and on the critical path.
- **Members:** `eak-eng-lead`, `eak-architect`, `eak-tpm`.
- **Owns end-to-end:** the dispatch loop, cross-seam coherence, and the milestone plan.
- **Accountable owner:** `eak-eng-lead` (it owns integration + the green-gate; the Architect and TPM
  are its lateral advisors, each accountable for their own artifact — ADRs and the plan respectively).

### Kernel Squad (L2) — the moat (Rust engineering runtime)
- **Charter:** keep the deterministic correctness kernel correct, green, and event-sourced; the LLM
  never enters here (R4).
- **Members:** `eak-kernel-engineer`, `eak-verification-engineer`, `eak-eda-domain-scientist`.
- **Owns end-to-end:** `eak-runtime` + the rule engine (DRC/DFM/EMC/ampacity/impedance/thermal) + the
  `engineering-science/` physics/IPC layer + the event-sourced commit path and IR projections.
- **Accountable owner:** `eak-kernel-engineer`.

### Application Squad (L3) — the Tauri IDE + AI harness
- **Charter:** wrap the kernel as a native core and build the thin IDE shell + the LLM reasoning
  boundary; reuse the canvas, don't rebuild it (R10).
- **Members:** `eak-desktop-engineer`, `eak-frontend-engineer`, `eak-harness-engineer`,
  `eak-canvas-integration-engineer`.
- **Owns end-to-end:** `eak-app` (Tauri shell + event bridge + packaging), the IDE panels + live
  engineering-state feed, the agent loop behind the `ReasoningEngine` port, and KiCanvas + KiCad import.
- **Accountable owner:** `eak-desktop-engineer` (owns the shell the rest plug into).

### Quality & Reliability chapter (L4) — the craft chapter that spans squads
- **Charter:** enforce craft consistency and the green-gate across every squad's output; no red commits.
- **Members:** `eak-rust-reviewer`, `eak-qa-test-engineer`, `eak-security-reviewer`.
- **Owns end-to-end:** the gate itself — build/clippy/fmt cleanliness, test-first discipline +
  fixtures/cassettes, and LLM-boundary + supply-chain safety.
- **Accountable owner:** `eak-rust-reviewer` (holds the merge gate; QA + Security are co-gaters on
  their axes).

### Product / GTM Squad (L5) — the fundraise arc (runs in parallel)
- **Charter:** curate the hero demo, hold scope discipline, and build the fundable package.
- **Members:** `eak-product-manager`, `eak-market-analyst`, `eak-fundraise-lead`, `eak-design-lead`.
- **Owns end-to-end:** spec + hero-demo curation + success criteria, positioning
  (`project-plans/04`), deck/one-pager/data-room, waitlist + landing, and demo/landing visual polish.
- **Accountable owner:** `eak-product-manager`.

---

## 3. RACI matrix — one Accountable per deliverable

Columns are the 17 role ids, abbreviated. Rows are the major roadmap deliverables.
**R** = does the work · **A** = single accountable owner (exactly one per row) · **C** = consulted ·
blank = **Informed**.

Column key: `EL`=eak-eng-lead `AR`=eak-architect `TP`=eak-tpm · `KE`=eak-kernel-engineer
`VE`=eak-verification-engineer `DS`=eak-eda-domain-scientist · `DT`=eak-desktop-engineer
`FE`=eak-frontend-engineer `HE`=eak-harness-engineer `CI`=eak-canvas-integration-engineer ·
`RR`=eak-rust-reviewer `QA`=eak-qa-test-engineer `SR`=eak-security-reviewer · `PM`=eak-product-manager
`MA`=eak-market-analyst `FL`=eak-fundraise-lead `DL`=eak-design-lead

| Deliverable (roadmap)          | EL | AR | TP | KE | VE | DS | DT | FE | HE | CI | RR | QA | SR | PM | MA | FL | DL |
|--------------------------------|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
| kernel event-contract (W2)     | C  | **A** | C |  R |    |    |  R | C  | C  | C  | C  | C  |    |    |    |    |    |
| Tauri spine (W1)               | C  | C  |    |  R |    |    | **A** |  |    |    | C  | C  |    |    |    |    |    |
| KiCad import (W4)              |    | C  |    |  R |  C |    |  R |    |    | **A** | C  | C |    | C  |    |    |    |
| AI harness (W5)                | C  | C  |    |  R |    | C  |    |    | **A** |  |  C |  C |  C |    |    |    |    |
| hero-flow generation (W6–8)    | **A** | C |  C |  R |  R |  C |    | C  |  R |  R |    | C  |    | C  |    |    |    |
| hero-flow review (W4/W8)       |    | C  |    |    | **A** | C |   |  R |  R |    |    | C  |    | C  |    |    |    |
| IDE UI (panels/feed)           |    | C  |    |    |    |    |  C | **A** | C |  R | C  | C  |    | C  |    |    |  R |
| demo polish (W10)              | C  |    |    |    |    |    |    |  R |    |    |    | C  |    |  R |  C |    | **A** |
| deck (W12)                     | C  |    |    |    |    |    |    |    |    |    |    |    |    |  C |  R | **A** |  R |
| waitlist (W11)                 |    |    |    |    |    |    |    |  C |    |    |    |    |    |  C | **A** |  R |  R |
| correctness/verification (R2/R4)| C |  C |    |  C |  R | **A** |   |    |    |    | C  |  R |    |    |    |    |    |
| packaging (W10)                | C  |  C |    |  R |    |    | **A** |  |    |    | C  | C  | C  |    |    |    |    |

Every row has exactly one **A**. `eak-tpm` never holds **A** on a build artifact by design — it is
accountable only for the milestone plan (tracked in §2 Leadership charter), and is Consulted/Informed
on everything so it can maintain the critical path.

---

## 4. Dispatch & integration model

**Decompose → route.** `eak-eng-lead` reads the week's roadmap row (`project-plans/03` §2), splits it
into file/component-scoped work items (R6), and routes each to the squad that owns that component
(§2). It never hands two agents the same file.

**Freeze-then-parallelize (R5/R8).** Before fan-out, the seam is written *once* and frozen: the
**kernel event stream** is the frontend↔backend contract, and the **IR schemas** are the phase seams.
The Architect signs off the seam; the Eng Lead tags it (e.g. contract `v1`). Only then do squads fan
out — Application builds the UI against a recorded **event cassette** while Kernel builds the real
emitter, and they converge by a merge, not a rewrite.

**Parallel behind frozen contracts.** Squads run concurrently, each isolated in its own git worktree
when mutating code (R6). Because they integrate only through the frozen seam, their branches don't
collide; the Eng Lead integrates them at the seam.

**Quality gates every change.** No squad self-merges. Every change crosses the L4 chapter's
green-gate (build → clippy -D warnings → test → domain review) and gets a domain review from the
relevant reviewer before the Eng Lead commits-green-or-reverts and pushes (R7).

**The Architect guards seams.** The Architect has a standing veto on any change that crosses the
kernel↔app boundary, violates the clean-architecture rings, or would edit a frozen contract without a
version bump (R4/R8). Seam changes require an ADR.

**Product/GTM runs in parallel.** The L5 squad does not wait on engineering: it curates the hero demo,
holds scope (R10), and builds positioning/deck/waitlist against the roadmap's expected capabilities,
syncing with the Eng Lead only at the demo-integration points (W8, W11, W12).

---

## 5. Structures this adapts

This org composes four proven models (per `00-plan-and-requirements.md` §3):
- **Functional org** — deep discipline specialists (kernel, frontend, QA) as narrow, deep subagents.
- **Spotify squads / chapters / guilds** — cross-functional **squads** (Kernel, Application, Product)
  own a slice end-to-end; the **Quality chapter** spans squads for craft consistency; the Architect
  acts as an architecture guild-of-one.
- **Amazon two-pizza teams** — each squad is small, single-owner, and owns its component's DoD.
- **EM / TL / IC hierarchy + RACI** — the Eng-Lead Orchestrator (EM/TL) dispatches; a RACI matrix
  (§3) names exactly one Accountable owner per deliverable.
