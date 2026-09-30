# EAK Autonomous Engineering Team — Plan & Requirements (Canonical)

> **Source of truth for the team design.** Every other file in `team/` anchors to the roster,
> layers, and operating model defined here. If another file contradicts this one, this one wins
> (or update this one deliberately). Written 2026-07-02.

---

## 1. Purpose

Stand up an **AI-agent engineering organization** that builds the Electronics Agent Kit MVP
(a local, AI-native EDA IDE — "Cursor for hardware") **autonomously, fast, and at high quality**,
executing the 13-week roadmap in [`../project-plans/`](../project-plans/) while keeping the Rust
correctness kernel green at every step. The human founder is the **CEO / Product Owner** — sets
intent, approves scope, and makes the calls agents escalate; the agents do the building.

This is not a metaphor: each role below is a **reusable Claude Code subagent definition**
(frontmatter + system prompt) in `team/agents/`. Copied into `.claude/agents/`, they become
invocable `subagent_type`s that an orchestrator agent dispatches work to. The org is real and
runnable.

## 2. Requirements the team must satisfy

**Mission requirements**
- R1 — Execute the MVP roadmap (Spine → Hero Flow → Polish & Signal → Raise) on schedule.
- R2 — Keep the kernel **green**: `cargo build` + `cargo clippy -D warnings` + `cargo test` + fmt
  clean on every commit (the shipped discipline; 186 tests today).
- R3 — Ship the **hero demo** (intent → generate → AI-review, traceable, curated) inside the
  native IDE, plus the fundraise artifacts (deck, waitlist, demo video).
- R4 — Preserve the **moat discipline**: deterministic kernel, LLM strictly behind the boundary,
  full traceability, event-sourced replay (P2/P3/P4/P9). No agent may weaken these.

**Operating requirements**
- R5 — **Canonical-first**: a source-of-truth doc/interface is written before parallel work fans
  out against it (how `project-plans/` and this folder were built).
- R6 — **Sole-writer per file**: parallel agents never edit the same file; work is partitioned by
  file/component to avoid conflicts. Isolate with git worktrees when agents mutate code in parallel.
- R7 — **Green-gate before commit**: build → clippy → test → domain review → commit → push. No red
  commits; commit-green-or-revert.
- R8 — **Hand-off contracts**: teams integrate only through fixed seams (the kernel **event
  stream** is the frontend↔backend contract; IR schemas are the phase seams). Freeze the contract,
  then build both sides in parallel.
- R9 — **Escalate, don't guess**: a genuine product/scope/architecture fork goes to the founder
  (CEO) or the Architect; agents pick sensible defaults for everything else and state them.
- R10 — **Cost & focus discipline**: narrow to what appears in the hero demo; reuse over build for
  anything that isn't the moat (kernel + harness).

**Non-goals (explicitly out of the team's mandate for the MVP)**
- General autorouting, a KiCad-beating editor, broad part coverage, cloud/collaboration — deferred
  to the funded year (agents must resist scope creep back into these; see the guardrails).

## 3. Org-design rationale (grounded in real structures)

We compose four well-known models, adapting each to an autonomous agent swarm:

| Real-world model | What we borrow | How we adapt it |
|---|---|---|
| **Functional org** (discipline teams) | Deep specialists per discipline (kernel, frontend, QA) | Each specialist is a focused subagent with a narrow, deep mandate. |
| **Spotify squads / tribes / chapters / guilds** | Cross-functional **squads** owning a slice end-to-end; **chapters/guilds** for craft consistency | Squads = Kernel, Application; chapters = the Quality reviewers who span squads. |
| **Amazon two-pizza teams** | Small, autonomous, single-owner teams | Each squad is small and owns its component's DoD end-to-end. |
| **EM / TL / IC hierarchy + RACI** | Clear leadership, a single accountable owner per deliverable | An Eng-Lead **Orchestrator** dispatches; a RACI matrix names one Accountable per artifact. |

**Dispatch model:** a star-of-squads. The **Eng Lead (Orchestrator)** decomposes the roadmap into
work items and dispatches each to the owning squad; the **Architect** guards coherence across
seams; the **TPM** tracks dependencies and status. Squads execute in parallel behind frozen
contracts; **Quality** reviewers gate every change; **Product/GTM** runs the fundraise arc in
parallel with engineering.

## 4. The org — five layers, 17 roles

Each role's `id` is its subagent name; full duties live in `team/agents/<id>.md`. "Reuse" marks
roles that map to an existing ECC agent (wrap/point to it rather than re-author from scratch).

**L1 — Leadership & Coordination**
| id | Title | One-line mandate |
|---|---|---|
| `eak-eng-lead` | Eng Lead / Orchestrator | Decompose the roadmap, dispatch to squads, integrate, enforce the green-gate. |
| `eak-architect` | Chief Architect | Own the clean-architecture rings, the kernel↔app boundary, ADRs, cross-seam coherence. *(reuse: ecc:architect)* |
| `eak-tpm` | Technical Program Manager | Own the milestone plan, dependencies, critical path, and status against `project-plans/03`. |

**L2 — Kernel Squad (the Rust engineering runtime — the moat)**
| id | Title | Mandate |
|---|---|---|
| `eak-kernel-engineer` | Rust Kernel Engineer | Core runtime, event-sourcing/commit path, IR projections, phase state machines. |
| `eak-verification-engineer` | Verification Engineer | The rule engine — DRC/DFM/EMC/ampacity/impedance/thermal; correctness gates. |
| `eak-eda-domain-scientist` | EDA Domain / Physics Scientist | The `engineering-science/` layer; physics + IPC-standard correctness; numeric test vectors. |

**L3 — Application Squad (the Tauri IDE + AI harness)**
| id | Title | Mandate |
|---|---|---|
| `eak-desktop-engineer` | Desktop / Tauri Engineer | The `eak-app` shell, kernel-as-native-core, event bridge, packaging. |
| `eak-frontend-engineer` | Frontend / UI Engineer | IDE shell, panels, the live engineering-state feed, taste. *(pairs with: design-taste-frontend skill)* |
| `eak-harness-engineer` | AI Harness Engineer | The LLM reasoning boundary, the agent loop, tools/prompts — grounded by the kernel. |
| `eak-canvas-integration-engineer` | Canvas / Interop Engineer | KiCanvas rendering + KiCad import/export (the review-path fuel). |

**L4 — Quality & Reliability (the chapter that spans squads)**
| id | Title | Mandate |
|---|---|---|
| `eak-rust-reviewer` | Rust Reviewer / Build-Fixer | Review every Rust change; keep build/clippy/fmt green. *(reuse: ecc:rust-reviewer, ecc:rust-build-resolver)* |
| `eak-qa-test-engineer` | QA / Test Engineer (TDD) | Test-first discipline, fixtures/cassettes, coverage, regression guards. |
| `eak-security-reviewer` | Security Reviewer | Secrets, LLM-boundary safety, dependency + supply-chain risk. *(reuse: ecc:security-reviewer)* |

**L5 — Product & Go-To-Market (the fundraise arc)**
| id | Title | Mandate |
|---|---|---|
| `eak-product-manager` | Product Manager | Spec, scope discipline, hero-demo curation, success criteria. |
| `eak-market-analyst` | Competitive / Market Analyst | Positioning vs Flux/KiCad/AI-EDA; keep `project-plans/04` live. |
| `eak-fundraise-lead` | Fundraise / Storytelling Lead | Deck, demo script, waitlist, investor materials, objection handling. |
| `eak-design-lead` | Design Lead | UX/visual polish for the demo and landing page. *(pairs with: design/brandkit skills)* |

## 5. How the team operates (summary — detail in operating-protocols.md)

1. **Intake:** founder states intent → Eng Lead + TPM turn it into work items mapped to the roadmap.
2. **Design:** Architect (or the owning senior role) writes the canonical seam/interface first (R5).
3. **Fan-out:** Eng Lead dispatches items to squads; squads work in parallel behind frozen contracts
   (R6/R8), isolated in worktrees when mutating code concurrently.
4. **Gate:** every change passes the green-gate (R7) and a Quality-chapter review before commit.
5. **Integrate:** Eng Lead integrates, resolves cross-seam issues with the Architect, commits+pushes.
6. **Report:** TPM updates status vs the critical path; escalations (R9) go to the founder.
7. **Cadence:** work in small, shippable increments (the kernel's proven increment discipline).

## 6. Folder map

- `00-plan-and-requirements.md` — **this file** (source of truth).
- `org-structure.md` — org chart, layers, squads, the RACI matrix, the dispatch model.
- `operating-protocols.md` — ways of working: canonical-first, green-gate, hand-off contracts,
  definition-of-done, review gates, escalation, worktree isolation, cadence.
- `agents/<id>.md` — one Claude Code subagent definition per role (frontmatter + system prompt).
- `README.md` — org chart + roster index + how to install and run the autonomous team.

## 7. Success criteria for the team itself

The team is "built" when: (a) all 17 role definitions exist and are installable to `.claude/agents/`;
(b) the org-structure + protocols docs let the founder run an autonomous build loop
(intent → dispatch → gate → commit) without re-deciding process each time; (c) the RACI names one
accountable owner for every roadmap deliverable; and (d) the whole team is anchored, conflict-free,
to this plan and the `project-plans/` roadmap.
