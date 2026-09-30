# EAK Autonomous Engineering Team

The AI-agent engineering organization that builds the Electronics Agent Kit MVP — a local,
AI-native EDA IDE ("Cursor for hardware") — **autonomously**, executing the
[`../project-plans/`](../project-plans/) roadmap while keeping the Rust correctness kernel green.

The human founder is the **CEO / Product Owner**: set intent, approve scope, decide escalations.
The agents do the building. Every role below is a real, runnable Claude Code subagent.

## Read in this order

| # | Doc | What it is |
|---|-----|-----------|
| 0 | [00-plan-and-requirements.md](00-plan-and-requirements.md) | **Source of truth** — why the team exists, requirements R1–R10, org-design rationale, the 17-role roster, operating model. |
| 1 | [org-structure.md](org-structure.md) | Org chart, squad charters, the 12×17 RACI matrix, the dispatch & integration model. |
| 2 | [operating-protocols.md](operating-protocols.md) | Ways of working: canonical-first, the green-gate, hand-off contracts, DoD, review gates, escalation, cadence, the work loop. |
| 3 | [agents/](agents/) | One Claude Code subagent definition per role (frontmatter + system prompt). |

## Org chart (star-of-squads, Eng-Lead-dispatched)

```
                          FOUNDER / CEO (intent, scope, escalations)
                                        │
                        ┌───────────────┴───────────────┐
                        │      L1 LEADERSHIP             │
                        │  eak-eng-lead (Orchestrator)   │  ← decomposes & dispatches
                        │  eak-architect   eak-tpm       │
                        └───────────────┬───────────────┘
          ┌──────────────────┬──────────┴───────┬───────────────────┐
   ┌──────┴──────┐    ┌──────┴──────┐    ┌───────┴───────┐   ┌───────┴───────┐
   │ L2 KERNEL   │    │ L3 APP      │    │ L4 QUALITY    │   │ L5 PRODUCT/GTM│
   │ SQUAD       │    │ SQUAD       │    │ (chapter)     │   │               │
   ├─────────────┤    ├─────────────┤    ├───────────────┤   ├───────────────┤
   │ kernel-eng  │    │ desktop-eng │    │ rust-reviewer │   │ product-mgr   │
   │ verification│    │ frontend-eng│    │ qa-test-eng   │   │ market-analyst│
   │ eda-domain  │    │ harness-eng │    │ security-rev  │   │ fundraise-lead│
   │ -scientist  │    │ canvas-integ│    └───────┬───────┘   │ design-lead   │
   └─────────────┘    └─────────────┘   gates every change   └───────────────┘
```

## The 17 roles

**L1 — Leadership:** `eak-eng-lead` · `eak-architect` · `eak-tpm`
**L2 — Kernel:** `eak-kernel-engineer` · `eak-verification-engineer` · `eak-eda-domain-scientist`
**L3 — Application:** `eak-desktop-engineer` · `eak-frontend-engineer` · `eak-harness-engineer` · `eak-canvas-integration-engineer`
**L4 — Quality:** `eak-rust-reviewer` · `eak-qa-test-engineer` · `eak-security-reviewer`
**L5 — Product/GTM:** `eak-product-manager` · `eak-market-analyst` · `eak-fundraise-lead` · `eak-design-lead`

## How to run the autonomous team

1. **Install the roster as subagents** — copy the definitions so Claude Code can dispatch to them:
   ```bash
   mkdir -p .claude/agents && cp team/agents/*.md .claude/agents/
   ```
   Each file's `name` becomes an invocable `subagent_type` (e.g. `eak-kernel-engineer`).
2. **Drive through the Eng Lead.** Give the founder-level intent to `eak-eng-lead` (the
   Orchestrator). It runs the loop — **intake → design (canonical-first) → fan-out to squads →
   green-gate → integrate → report** — spawning the specialist agents in parallel behind frozen
   contracts (the kernel event stream; IR schemas), isolating parallel code work in git worktrees.
3. **Gate everything.** No change ships until the Quality chapter passes it and the green-gate is
   clean (`cargo build` + `cargo clippy -D warnings` + `cargo test` + `cargo fmt --check`).
4. **Escalate genuine forks.** Scope/product decisions → founder; architecture/seam changes →
   `eak-architect`. Everything else the squads decide and state.

## Non-negotiables every agent inherits

- **The moat is sacred:** deterministic single commit path (P2), seam re-validation (P3),
  replay/determinism (P4), typed quantities (P9). No agent weakens these.
- **Green or revert:** commits are always green; red never ships.
- **Reuse over build** for anything that isn't the moat (kernel + harness).
- **Hero-demo test:** if a task doesn't serve the hero demo or the raise, it's out of scope for the MVP.

Anchored to [00-plan-and-requirements.md](00-plan-and-requirements.md) and the
[project roadmap](../project-plans/). Reuse-marked roles (`eak-architect`, `eak-rust-reviewer`,
`eak-security-reviewer`) wrap existing ECC agents and add only EAK-specific guardrails.
