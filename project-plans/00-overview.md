# Electronics Agent Kit — MVP Plan (Canonical Overview)

> **This is the source of truth for the MVP effort.** Every other document in
> `project-plans/` anchors to the definitions, goal, scope, and constraints stated
> here. If another doc contradicts this one, this one wins (or this one gets updated
> deliberately). Written 2026-07-02.

---

## 1. What we are building (the MVP in one paragraph)

A **local, native, AI-native EDA IDE for PCB/electronics design — "Cursor for hardware."**
A desktop application (like VSCode/Cursor) that runs on the engineer's machine, whose
**soul is a superior AI harness** (agentic, multi-step design assistance) **grounded by a
deterministic engineering correctness kernel** that already exists in this repo. The engineer
expresses intent and drives the design through an AI agent; the kernel owns the versioned
engineering state, verifies every action, and keeps everything traceable back to the original
intent. The visual editor/canvas is **reused**, not rebuilt; our effort and our IP go into the
**harness + the correctness kernel.**

One-line pitch: **"Cursor for hardware — an AI harness you can actually trust to design boards,
because a deterministic engineering kernel verifies everything it does."**

## 2. Why this is different (the moat)

Competitors (esp. **Flux**) are **editor-first tools with an AI copilot**: a human designs in a
canvas and the AI *suggests*; the human is the safety net. We are **runtime-first**: a
deterministic kernel *owns* the engineering state and correctness, and the LLM is a **reasoning
engine behind a strict boundary**. The agent physically cannot commit an invalid design past the
kernel's checks; every action is verified, traceable, and deterministically replayable.

- Flux / copilots = **AI that suggests** (probabilistic assist, human catches mistakes).
- Us = **AI you can trust to drive** (verified-by-construction, full traceability, replay).

The moat is **not "we have AI too"** (Flux does). The moat is the **deterministic correctness
kernel + traceability + replay** — the substrate that makes AI-generated hardware trustworthy,
and the thing that is architecturally hard to bolt onto an editor-first product after the fact.

## 3. The goal (what "done" means for THIS effort)

**Not** a market-fit or feature-complete product. **Not** beating KiCad on editing features.
The goal is a **fundable MVP + demo in ~3 months** that raises a **pre-seed round**, so the full
product can be built in the following ~12 months with funding and (eventually) a small team.

Success = a polished local IDE demo of the AI-native loop on curated real examples, a sharp deck,
and early signal (waitlist / design partners), such that a pre-seed investor writes a check.

## 4. The hero demo ("both in one flow")

One curated, hardened example, flawless every time, shown inside the local IDE:

1. **Intent** — engineer types a hardware goal in English (e.g. "USB-C powered I²C temperature
   sensor, < 1 W").
2. **Generate** — the AI harness (real LLM behind the kernel boundary) streams: requirements →
   architecture (blocks) → part selection / BOM → schematic/netlist, each **validated live by the
   kernel**, with the **traceability graph filling in** as it goes.
3. **Starter board** — placement + a *constrained/assisted* route completes and renders on the
   canvas (NOT a general autorouter — curated for reliability).
4. **AI review** — kernel runs DRC/DFM/EMC/ampacity/impedance/thermal; the AI **explains each
   finding + suggests a fix**; every issue **traces back to the original English sentence**.
5. **The "whoa"** — one sentence → a checked, explained, fully-traceable board, AI reasoning
   visible, kernel guaranteeing correctness — inside a native local IDE.

Bulletproof fallback segment: **import a real KiCad board → AI review** always works (just parse +
run the existing rules), so the demo never fully depends on generation.

## 5. Architecture at a glance (see 02-technical-architecture.md for detail)

- **Shell:** **Tauri** desktop app (native, local-first, offline-capable). Chosen over Electron
  because **Tauri's backend is Rust** — so our existing Rust kernel *becomes* the app's native core.
- **Backend = the existing Rust engineering kernel** (this repo's `eak/` workspace): deterministic,
  event-sourced, 15-phase pipeline, verification engine, typed quantities, IR projections, replay.
- **Frontend = a web UI shell** embedded by Tauri: intent/agent chat · design canvas · live
  engineering-state / traceability / DRC + reasoning panels.
- **Canvas/renderer = REUSED**, never built (KiCanvas / KiCad file formats / KiCad engine).
- **Contract between them = the kernel's event stream** (already versioned) surfaced to the UI —
  this is the seam that lets frontend + backend be built in parallel.
- **LLM = real Claude API** wired through the kernel's reasoning boundary (the `live` feature).

## 6. Scope — narrow the design surface, not the vision

We cannot build a full schematic-capture + PCB-layout editor + agent + kernel integration in 3
months. So we narrow **what design capabilities are real**, and **reuse** everything that isn't our
edge:

| Layer | MVP approach |
|---|---|
| IDE shell (Tauri + panels) | **Build** — small; this is our chrome. |
| Canvas / rendering | **Reuse/embed** — never build. |
| Correctness kernel | **Already built** — our edge. |
| AI harness (agent loop) | **Build** — the soul; where effort goes. |
| Design generation (routing/layout) | **Assisted + curated**, not general autorouting (year-1). |
| Parts / footprints / datasheets | **Reuse** — KiCad libs + a parts API (Nexar/Octopart). |

**In scope:** the local IDE shell, the AI harness grounded by the kernel, the intent→generate→
review→explain loop on curated examples, KiCad import, live traceability, a polished demo.

**Out of scope (year-1, post-raise):** general autorouting, a full schematic/layout editor to rival
KiCad, broad part coverage, collaboration/cloud, manufacturing-grade output for arbitrary boards.

## 7. Current state (what already exists)

- A **deterministic Rust engineering kernel** (`eak/`): Phases 1–2 + full Phase-3 lifecycle +
  hardening + the engineering-science B-series. **185 tests, clippy + fmt clean.**
- **15-phase pipeline** lowers a captured intent end-to-end to a released Manufacturing IR.
- **Verification engine** with 8 DRC rules + ERC/DFM/EMC/constraint/BOM + ampacity, controlled-
  impedance (microstrip), and thermal (T_j) checks; typed physical quantities; event-sourced state
  with deterministic replay; IR projections at every phase boundary.
- A **top-level `engineering-science/`** documentation layer (59 docs) grounding the physics.
- **No UI, no users, no packaging as a desktop app, LLM reasoning largely unwired (`live` feature).**

## 8. Constraints & operating assumptions

- **Team:** solo founder + heavy AI-assisted development.
- **Timeline:** ~3 months to a fundable demo; ~12 months post-raise to the real product.
- **Budget:** pre-funding — reuse/buy over build wherever it isn't the moat; keep spend low.
- **Bias:** ship narrow and polished over broad and janky; curate the demo ruthlessly.
- **Canonical git remote:** github.com/NijoP/electronics-agent-kit (push only there).

## 9. Document map (this folder)

- `00-overview.md` — **this file**, the source of truth.
- `01-product-spec.md` — MVP product definition, the IDE UX, hero demo, in/out scope.
- `02-technical-architecture.md` — Tauri + kernel + frontend + reuse + event contract + tech choices.
- `03-roadmap.md` — the 3-month week-by-week roadmap, milestones, dependencies, deliverables.
- `04-competitive-positioning.md` — vs Flux/KiCad/others, the moat, the one-line pitch.
- `05-fundraise-plan.md` — fundable artifacts, waitlist/design partners, deck, investor strategy.
- `06-risks-and-buy-vs-build.md` — risks + mitigations, the demo-jank plan, buy-vs-build decisions.
- `07-engineering-backlog.md` — concrete engineering tasks mapped to the roadmap.
- `README.md` — index + reading order.
