# Electronics Agent Kit — 3-Month Roadmap (to a fundable MVP demo)

> Anchored to `00-overview.md` (source of truth). This is the executable plan: open it
> Monday and know exactly what to build. Horizon = **13 weeks** to a hardened demo + a
> pre-seed raise, then a high-level **months 4–15** post-raise arc. Written 2026-07-02.

**One sentence:** freeze the kernel↔UI event contract first, prove the kernel streams a real
run into a native Tauri window by week 3, land the bulletproof **KiCad-import → AI-review**
path by week 4, wire the live LLM by week 5, get the full **intent → generate → review** hero
flow working on **one curated example** by week 8, then spend the back third making it
flawless, packaging it, and turning it into a deck + waitlist + investor meetings.

---

## 1. Phases

| Phase | Weeks | Goal | Entry criteria | Exit criteria | Single most important outcome |
|---|---|---|---|---|---|
| **A — Spine** | 1–3 | Kernel becomes the native core of a Tauri app; the event contract is frozen so FE + BE can proceed in parallel. | Kernel exists (185 tests, clippy/fmt clean); no UI. | A **real** fixture-driven pipeline run streams events into a native window; contract v1 frozen; FE building against mocks. | The seam is real and proven end-to-end (no LLM yet). |
| **B — Hero Flow** | 4–8 | The intent→generate→review→explain loop works on one curated example, plus the bulletproof KiCad-import→review fallback. | Contract frozen; kernel streams into the window; canvas embeddable. | Full hero flow runs **once, honestly**, on the curated example; live LLM behind the kernel; traceability fills in. | The "whoa" exists as software, at least once. |
| **C — Polish & Signal** | 9–11 | Make the one example flawless and demo-safe; package as a local app; stand up the landing + waitlist; capture a backup video. | Hero flow runs once. | Demo runs 10/10 in dry-runs; signed local build; landing + waitlist live; backup video recorded. | A demo that never breaks + first market signal. |
| **D — Raise** | 12–13 | Turn the demo into a fundable package and get in front of investors. | Demo hardened; video in hand; waitlist collecting. | Deck + one-pager + data room done; investor list built; first meetings booked; design-partner conversations open. | Checks in motion. |

### Dependency spine (why the order is what it is)

```
W1 Tauri+kernel-as-lib ─▶ W2 FREEZE CONTRACT ─┬─▶ (agents) frontend panels ───────────────┐
                                              ├─▶ (agents) canvas/KiCanvas embed ──────────┤
                                              └─▶ W3 real run streams into window ─▶ W4 KiCad import→review (FALLBACK)
                                                                                     │
W5 wire live LLM (behind port) ──────────────────────────────────────────────────────┘
        │
        └─▶ W6 requirements+architecture ─▶ W7 BOM+netlist ─▶ W8 board+review = HERO END-TO-END
                                                                    │
                                                     W9 harden ─▶ W10 package/polish ─▶ W11 demo-proof+waitlist
                                                                                              │
                                                                                    W12 deck ─▶ W13 outreach+buffer
```

Everything left of "FREEZE CONTRACT" is single-track (founder). Everything right of it forks:
the founder drives the backend/reasoning line (W3→W5→W6→W7→W8); agents drive the FE + canvas
line in parallel; the two lines converge at W8. The **KiCad-import→review fallback (W4) is built
before generation is attempted** so a working demo exists independent of the generation risk.

---

## 2. Week-by-week (weeks 1–13)

Legend for honesty column — **REAL** = genuinely computed live; **CURATED** = real code paths but
inputs/params tuned for one example; **CASSETTE** = deterministic recorded LLM output replayed;
**REUSED** = third-party (KiCanvas/KiCad/parts API); **FAKE** = scripted/stubbed for the demo only.

| Wk | Focus | Concrete deliverables | Demo capability unlocked | Honesty |
|---|---|---|---|---|
| **1** | Tauri spine + kernel-as-lib | Tauri app scaffold; `eak-runtime` linked into the Tauri Rust backend as a library; one `command→runtime→response` round-trip over a Tauri command; CI builds the desktop app. | App window opens; a button triggers a real kernel call and shows a real result. | REAL plumbing, no AI |
| **2** | **Freeze the event contract** | v1 of the Presentation/Query seam (commands in; projections + diagnostics/events out) serialized as versioned JSON; a **mock event-stream player** that replays a captured run; schema + example fixtures checked in. | FE can be built against the mock stream with zero backend. | REAL contract + fixtures |
| **3** | Kernel streams a real run | Wire the kernel's **event-sourced stream** (15-phase pipeline, fixture reasoning) through the contract into the window; a live-updating engineering-state / event panel. | **De-risk #1:** a *real* pipeline run streams into a native window (no LLM). | REAL run, fixture reasoning |
| **4** | **Bulletproof fallback: KiCad import → AI-review** | Import a real `.kicad_pcb`; embed **KiCanvas** and render it; run the existing DRC/DFM/EMC/ampacity/impedance/thermal rules; show findings in a review panel (fixture-explained). | Import a real board → see it rendered → see real checks + findings. This is the demo's safety net, built early. | REAL checks · REUSED canvas |
| **5** | **Wire the live LLM** | Turn on the `live` Anthropic adapter behind the `ReasoningEngine` port; stream real model reasoning; **record cassettes** of every good run for deterministic replay; token/latency budget + error handling. | AI review findings are explained by a **real** model; runs are replayable. | REAL live LLM + CASSETTE |
| **6** | Generate: intent → requirements → architecture | Intent chat box; agent generates requirements → functional blocks, **each validated live by the kernel**; traceability graph starts filling in (sentence → requirement → block). | Type the goal → watch requirements + architecture stream in, kernel-checked. | REAL (curated prompt) |
| **7** | Generate: parts/BOM → schematic/netlist | Part selection + BOM via parts API (Nexar/Octopart) for the curated example; schematic/netlist generation; each capability kernel-validated; traceability links surface on hover. | Generation continues to a checked BOM + netlist, fully traced to the sentence. | CURATED gen · REUSED parts |
| **8** | Starter board + AI review (**hero end-to-end**) | Curated/assisted placement + a **constrained** route for the one example, rendered on canvas; full review pass (DRC/DFM/EMC/ampacity/impedance/thermal) with AI explanation + suggested fix, each tracing back to intent. | **De-risk #4:** one sentence → checked, explained, traceable board — end to end, once. | CURATED board · REAL checks |
| **9** | Curate + harden the one example | Pin seeds/params so the example is repeatable; edge/error handling; graceful degradation + a "safe mode" toggle that falls back to cassette replay; kill flakiness. | The hero flow runs the same way every time. | CURATED + CASSETTE |
| **10** | Package + IDE polish | Signed local build (target the demo machine's OS first); offline path; performance pass; visual polish of panels (design-taste); keyboard-driven demo script v1. | A double-clickable local app that looks like a product. | REAL app |
| **11** | Demo-proof + landing + waitlist | 10 clean dry-runs; **record a backup demo video** (never present live-only); landing page + waitlist live; instrument signups; seed a few design-partner outreach threads. | Shippable demo (live or recorded) + a page collecting signal. | REAL demo + REUSED landing |
| **12** | Deck + narrative | Pitch deck (problem, moat = kernel+traceability+replay, demo, market, ask); one-pager; positioning vs Flux/KiCad; light data room; investor target list (names, warm paths). | A coherent fundraising package around the demo. | — |
| **13** | Outreach + buffer | Send outreach; book first meetings; run design-partner calls; final demo hardening; **schedule buffer** absorbs slippage. | Investor meetings booked; the demo survives contact. | — |

### Week detail — concrete task checklists

Each week's "done" = every box checked. Weeks with a **[GATE]** are hard gates (§3).

- **W1 — Tauri spine**
  - [ ] `create-tauri-app`; commit a running window; wire CI to build the desktop bundle.
  - [ ] Add `eak-runtime` as a path dependency of the Tauri Rust backend (kernel = native core).
  - [ ] One Tauri `#[command]` that calls into the runtime and returns a real value to the webview.
  - [ ] Decide the in-process transport (Tauri command + event emit) — record it against `docs/integration/ipc.md`.
- **W2 — Freeze the contract [GATE]**
  - [ ] Serialize the Presentation/Query seam: command envelope (in) + projection/diagnostic/event envelope (out) as **versioned** JSON (`schemaVersion`).
  - [ ] Capture one full fixture run's event stream to a file; write a **mock stream player** that replays it with realistic timing.
  - [ ] Publish schema + example fixtures + a short "FE contract README" so agents can start with zero backend.
  - [ ] Tag the contract `v1`; changes after this require a version bump, not an edit.
- **W3 — Real run into a window [GATE]**
  - [ ] Stream the event-sourced 15-phase pipeline (fixture reasoning) through the contract to the webview.
  - [ ] Live engineering-state / event-log panel that updates as events arrive.
  - [ ] Prove backpressure/ordering are sane for a full run; no dropped events.
- **W4 — KiCad import → AI-review (fallback)**
  - [ ] Parse a real `.kicad_pcb` into the kernel's board projection.
  - [ ] Embed **KiCanvas**; render the imported board in a canvas panel.
  - [ ] Run existing DRC/DFM/EMC/ampacity/impedance/thermal rules; surface findings in a review panel (fixture explanations).
  - [ ] This path must run standalone — it is the demo's floor.
- **W5 — Wire the live LLM [GATE]**
  - [ ] Build with `--features live`; route the `ReasoningEngine` port to the Anthropic adapter.
  - [ ] Record every good call as a **cassette**; verify replay reproduces the run bit-for-bit (determinism).
  - [ ] Token/latency budget, timeout + retry, and a graceful "fall back to cassette" error path.
- **W6 — Generate: requirements → architecture**
  - [ ] Intent chat input → requirement-planning + architecture agents; **kernel validates each capability** before commit.
  - [ ] Traceability graph panel begins filling (sentence → requirement → functional block).
  - [ ] Curate the prompt/scaffold for the one hero example.
- **W7 — Generate: BOM → schematic/netlist**
  - [ ] Parts/BOM via Nexar/Octopart for the curated example; footprints from KiCad libs.
  - [ ] Schematic/netlist generation; each step kernel-validated; ERC/BOM checks run.
  - [ ] Traceability links surface on hover (entity ↔ originating sentence).
- **W8 — Board + review = hero end-to-end [GATE]**
  - [ ] Curated/assisted placement + a **constrained** route for the one example; render on canvas.
  - [ ] Full review pass with AI explanation + suggested fix per finding; each finding traces to intent.
  - [ ] Walk the whole flow once, top to bottom, and record the cassette of the golden run.
- **W9 — Harden the one example**
  - [ ] Pin seeds/params; make the golden run byte-reproducible.
  - [ ] Edge/error handling; a **safe-mode toggle** that serves the cassette if live wobbles.
  - [ ] Remove every source of flakiness on the demo path.
- **W10 — Package + polish**
  - [ ] Signed local build for the demo machine's OS first; verify offline launch.
  - [ ] Performance pass (stream + canvas); visual polish of panels (design-taste skill).
  - [ ] Write demo script v1 (keyboard-driven, no mouse hunting).
- **W11 — Demo-proof + signal**
  - [ ] 10 consecutive clean dry-runs logged; fix anything that breaks even once.
  - [ ] Record the **backup demo video** (the artifact you actually send).
  - [ ] Landing page + waitlist live + analytics; open 3–5 design-partner threads.
- **W12 — Deck + narrative**
  - [ ] Deck: problem · moat (kernel + traceability + replay) · demo · market · team · ask.
  - [ ] One-pager + light data room; positioning vs Flux/KiCad (see `04-competitive-positioning.md`).
  - [ ] Investor target list with warm-intro paths.
- **W13 — Outreach + buffer**
  - [ ] Send outreach; book the first meetings; run design-partner calls.
  - [ ] Final demo hardening; buffer absorbs any W1–W11 slippage.

> Engineering tasks map into `07-engineering-backlog.md`; keep that backlog as the ticket-level
> breakdown and this roadmap as the week-level intent.

---

## 3. Critical path (must not slip)

These four gate everything downstream. If one is late, the whole tail moves — protect them first.

1. **Event contract frozen (end of W2).** Every parallel workstream (all frontend, the mock
   stream, the canvas panel) keys off this. Freeze it early and version it; changing it later
   is the single most expensive mistake available.
2. **Kernel streams a real run into a native window (end of W3).** Proves Tauri + kernel +
   contract compose. Until this is real, everything else is theory.
3. **Live LLM wired through the reasoning boundary (end of W5).** The `live` adapter behind the
   `ReasoningEngine` port + cassette recording. This is what makes the demo "AI you can trust,"
   not "AI-flavored." Fixtures keep W1–W4 unblocked, but this must land on time.
4. **Hero flow end-to-end on one example (end of W8).** The whole raise rests on the "whoa."
   Everything after W8 assumes it exists at least once.

> Rule: **W2, W3, W5, W8 are hard gates.** If a gate is at risk, pull scope from that week's
> polish, not from the gate. If the gate itself is at risk, invoke the contingency (§7).

---

## 4. Parallelization (founder vs AI agents)

The frozen contract (W2) is the enabling seam: it lets the founder work **backend-down** and AI
agents work **frontend-up** against a mock stream, meeting in the middle.

| Track | Owner | Builds | Depends on |
|---|---|---|---|
| **Kernel / harness / backend** | **Founder** (deep, load-bearing, correctness-critical) | Tauri Rust backend, kernel-as-lib wiring, event-stream serialization, the `live` reasoning adapter + cassettes, capability/generation curation, KiCad import glue. | The kernel (exists); the contract (W2). |
| **Frontend shell** | **AI agents**, founder-reviewed | Intent/chat UI, engineering-state panel, traceability graph viz, DRC/review panel, canvas host, layout/polish — all against the **mock event stream**. | The contract + mock player (W2). |
| **Canvas / rendering** | **AI agents** (integration only) | Embed KiCanvas; render imported/generated boards; overlay findings. | KiCanvas (REUSED); board projection in the contract. |
| **Parts / footprints** | **AI agents** (integration only) | Parts API (Nexar/Octopart) calls for the curated BOM; KiCad lib lookups. | Parts API key; the curated example's part list. |
| **Fundraise assets** | **Founder** (+ agents for copy/design) | Deck, one-pager, landing, waitlist, demo video. | A working demo (W8) + brand. |

**Why it works:** the founder's scarce time goes to the moat (kernel, reasoning boundary,
curation); the parallelizable, mockable UI surface is delegated to agents and only needs the
contract, not a finished backend. The contract is what converts a solo founder into two tracks.

---

## 5. De-risking milestones (earliest proof-down of each big risk)

| Risk | Proven down at | What proves it |
|---|---|---|
| "Tauri + our Rust kernel can't actually compose into a native app." | **W1** | Kernel linked as a lib; a real command round-trips through a Tauri command. |
| "The kernel↔UI seam is wrong / will churn forever." | **W2** | Contract v1 frozen + versioned; mock player replays a captured run. |
| "The kernel can't drive a live UI from its event stream." | **W3** | A real 15-phase run streams into a native window (fixture reasoning). |
| "We have no demo if generation is flaky." | **W4** | KiCad-import → AI-review works end to end — the bulletproof segment exists before generation is attempted. |
| "The LLM can't be trusted / isn't reproducible." | **W5** | `live` adapter behind the port + cassette replay = real reasoning *and* deterministic runs. |
| "Generation past requirements is vapor." | **W6–W7** | Requirements → architecture → BOM → netlist stream in, each kernel-validated. |
| "The full 'whoa' doesn't exist." | **W8** | Intent → checked, explained, traceable board, end to end, once. |
| "It breaks when a stranger watches." | **W9–W11** | 10/10 dry-runs + a recorded backup video + safe-mode fallback. |
| "No one wants this." | **W11+** | Waitlist signups + design-partner conversations. |

---

## 6. Post-raise roadmap (months 4–15) — the arc investors are buying

The MVP demo is a *proof*, not the product. The funded year turns the curated hero flow into a
tool real engineers use daily. High-level, quarter by quarter:

| Quarter | Theme | Builds toward |
|---|---|---|
| **M4–M6** | **From one example to many** | Generalize generation beyond the single curated board; broaden part/footprint coverage; harden the agent loop across more intents; onboard the first design partners on real (small) boards. |
| **M7–M9** | **Real routing + real editing** | Move from constrained/curated routing toward a **general assisted autorouter**; a genuine schematic/layout editing surface (still canvas-reused where possible) so users can drive, not just watch; round-trip edits back through the kernel with full traceability. |
| **M10–M12** | **Depth + trust at scale** | Deeper verification (more DRC/DFM/EMC/thermal coverage, manufacturing-grade output for supported board classes); replay/traceability as a first-class collaboration + audit feature; first small team hires (kernel + frontend). |
| **M13–M15** | **Product + early revenue** | Move design partners to paid; multi-board projects; the beginnings of collaboration/cloud sync; a repeatable onboarding that doesn't require the founder in the room. Set up the seed raise. |

**The arc for the deck:** *pre-seed* = "we proved AI can be trusted to drive hardware design,
grounded by a deterministic kernel, in a native IDE." *Funded year* = "we widen the design
surface (routing, editing, parts) and get real engineers designing real boards on it." *Seed* =
"users + revenue + a defensible correctness substrate competitors can't bolt on."

---

## 7. Contingency (the demo ships even if generation disappoints)

The demo is **non-negotiable**; the fidelity of generation is the adjustable variable. Descend
this ladder only as far as needed — always keep the layer below as a live fallback.

| If behind on… | Cut / substitute | Result |
|---|---|---|
| Live generation is flaky (W6–W8) | Replay a **recorded cassette** of a known-good generation run instead of calling the model live on stage. | Demo still shows real reasoning + real kernel checks; just not sampled live. |
| Assisted placement/routing won't converge (W8) | Ship a **pre-baked** placement + route for the one example (curated artifact), rendered on canvas. | The board still appears, checked and explained; only the layout step is canned. |
| Generation as a whole is not demo-safe (by W9) | Lead the demo with the **KiCad-import → AI-review** path (built W4) and present generation as "streaming preview." | The bulletproof segment carries the demo; import→review always works (parse + run existing rules). |
| Packaging/signing slips (W10) | Demo from the **founder's dev machine**; ship the **recorded video** as the primary artifact. | No dependence on a distributable build for the raise. |
| Time collapses entirely | Demo = **import a real board → AI reviews it, explains findings, traces them** + a crisp vision deck for the generation arc. | Still a fundable story: trustworthy AI review on a real kernel, today; generation as the funded roadmap. |

**Non-negotiables (never cut):** the deterministic kernel doing **real** checks; **traceability**
back to intent; a demo that runs the same way every time; an honest line between what is live and
what is curated (investors forgive curation, not deception).

**Schedule buffer:** W13 is deliberately light — it absorbs slippage from B/C. If W1–W11 land on
time, W13 becomes extra outreach. If they don't, W13 is the shock absorber before any milestone
is sacrificed.

---

## 8. Operating rhythm (how a solo founder actually runs this)

- **Monday:** pick this week's row (§2) + its checklist; decide the founder-vs-agent split (§4);
  brief the agents with the frozen contract + the week's FE tickets.
- **Midweek:** integrate the agent-built FE against the real backend as it comes online; the
  contract means integration is a merge, not a rewrite.
- **Friday:** run the gate check — does this week's "done" hold? If a **[GATE]** week, the gate
  outranks all polish. Record a short demo GIF of whatever is newly real (proof + future deck fuel).
- **Cadence guardrails:** freeze the contract once (W2) and resist re-opening it; keep the golden
  run replayable from W5 onward so you can always demo *something*; never let the tree go a week
  without the KiCad-import→review path building (it is the parachute).

**What "good" looks like at each checkpoint:** W3 = a real run visibly streaming; W5 = a live
model explanation you can reproduce offline; W8 = one sentence → a checked, traceable board;
W11 = 10/10 dry-runs + a video + first signups; W13 = meetings on the calendar.

---

### How to read this Monday morning

1. This week's row in §2 tells you the focus + the exact deliverables.
2. §3 tells you whether you're on the critical path (if yes, protect the gate above all).
3. §4 tells you what to hand to agents vs keep for yourself.
4. If you're behind, §7 tells you what to cut — **top of the ladder first, demo always ships.**
