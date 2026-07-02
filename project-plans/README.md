# Electronics Agent Kit — MVP Project Plans

Strategic plan to take the existing engineering kernel to a **fundable MVP** in ~3 months:
a **local, native, AI-native EDA IDE — "Cursor for hardware"** (Tauri desktop app; the existing
Rust kernel is the native backend; AI harness grounded by a deterministic correctness kernel;
canvas reused, not rebuilt). Goal = raise a pre-seed, then build the real product over ~12 months.

Authored 2026-07-02 via canonical-first multi-agent planning (one canonical overview + 7 specialist
drafts, each grounded in `00-overview.md` and the real `eak/` kernel).

## Read in this order

| # | Doc | What it answers |
|---|-----|-----------------|
| 0 | [00-overview.md](00-overview.md) | **Source of truth** — what we're building, the moat, the goal, the hero demo, scope, constraints. Read first. |
| 1 | [01-product-spec.md](01-product-spec.md) | The MVP product: the IDE experience, harness features, the second-by-second hero-demo script, success criteria. |
| 2 | [02-technical-architecture.md](02-technical-architecture.md) | Tauri + kernel-as-native-core, the event-stream contract (the seam), the AI harness, reuse strategy, KiCad import. |
| 3 | [03-roadmap.md](03-roadmap.md) | **The 13-week roadmap** — 4 phases (Spine → Hero Flow → Polish & Signal → Raise), week-by-week deliverables, critical path. |
| 4 | [04-competitive-positioning.md](04-competitive-positioning.md) | vs Flux / KiCad / AI-EDA startups; the one-line differentiation; defensibility; honest weaknesses. |
| 5 | [05-fundraise-plan.md](05-fundraise-plan.md) | Fundable artifact set, deck outline, traction/waitlist plan, investor strategy, objection handling, odds. |
| 6 | [06-risks-and-buy-vs-build.md](06-risks-and-buy-vs-build.md) | Risk register, the demo-jank mitigation plan, buy-vs-build table, scope guardrails, kill/pivot criteria. |
| 7 | [07-engineering-backlog.md](07-engineering-backlog.md) | 8 epics → concrete tasks (BUILD vs REUSE, S/M/L), dependencies, and the "start here Monday" first-week list. |

## TL;DR

- **Product:** Cursor-for-hardware, local Tauri IDE; the moat is the **deterministic correctness
  kernel** that makes AI hardware design *verifiable*, not just *suggested* (the Flux differentiator).
- **Goal:** a polished demo of the intent → generate → AI-review loop (traceable, on curated
  examples) + a waitlist + a deck → **pre-seed raise**. Not a KiCad-beating editor.
- **Do NOT build:** the canvas/editor, part data, autorouting — **reuse** (KiCanvas, KiCad formats,
  a parts API). **Build only the moat:** the kernel (done) + the AI harness + the thin IDE shell.
- **Parallel build unlock:** freeze the **kernel event-stream contract** early; then the frontend
  (against a recorded "event cassette") and the backend proceed simultaneously behind that seam.

## The critical-path technical finding (both architecture + backlog agents)

The kernel today commits **synchronously** (`RuntimeCore::commit`) and `Orchestrator::run` only
returns at the *end* of a run — there is **no mid-run event observer**. Live UI streaming therefore
requires **one real kernel addition**: a small **`EventSink` observer hook on the single commit
path**, emitting the already-serializable `eak_ports::Event` stream. This is the load-bearing item
that unblocks the whole IDE, and it belongs in a new outermost-ring crate (`eak-app`) beside
`eak-cli` — the clean-architecture ring guard still holds.

## Start here (week 1)

Per [07-engineering-backlog.md](07-engineering-backlog.md) E1/E2 and [03-roadmap.md](03-roadmap.md)
Phase "Spine": stand up **`eak-app`** (Tauri) with the Rust kernel as its native backend, add the
`EventSink` commit hook, and stream one real `default_workflow()` run into a native window. That
single spine proves the architecture and everything else hangs off it.

## Status

- Plans: **complete** (this folder).
- Next action: build the Tauri + kernel spine (E1) — offered as the first implementation commit.
- These are living documents; `00-overview.md` is authoritative — update it deliberately when scope changes.
