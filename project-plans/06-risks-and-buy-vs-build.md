# Risks & Buy-vs-Build

> Anchored to `00-overview.md` (the source of truth). This doc is the blunt,
> pragmatic risk view: what can kill the 3-month fundable-demo effort, how we
> de-risk it, what we refuse to build, and when we admit the wedge is wrong.
> Written 2026-07-02. Reading frame: **solo founder + heavy AI, ~3 months, a
> pre-seed demo of intent → generate → AI-review, all traceable, on curated
> examples.** The moat is the **kernel + harness**; everything else is a means
> to the demo.

The single most important sentence in this document: **the goal is not a product,
it is a check.** Every risk is scored against "does this stop an investor writing
a pre-seed check in ~3 months," not against "is the software good."

---

## 1. Risk register (top 12)

Likelihood / Impact scored **H/M/L**. Rank ≈ likelihood × impact, gut-adjusted.
"Owner-hours" is the honest cost of the mitigation for a solo founder.

| # | Risk (dimension) | Description | L | I | Mitigation |
|---|---|---|---|---|---|
| 1 | **Generation looks janky / fake** (Technical·Product) | The intent→schematic→placed-and-routed step produces ugly, obviously-wrong, or empty boards on stage. This is *the* demo-killer: it reads as "vaporware with a chat box." | **H** | **H** | Do **not** ship a general generator. Curate 1 hero example + 2 backups end-to-end until flawless (§2). Constrained/assisted route only. Recorded event-stream fixture as hard fallback. Lead the demo with the **bulletproof KiCad-import→AI-review** segment so credibility is banked before any generation runs. |
| 2 | **Solo-founder bandwidth / bus factor** (Execution) | One person owns kernel, harness, Tauri shell, renderer glue, deck, fundraise, and the demo script. Illness, a rabbit-hole, or a bad week directly slips the raise. | **H** | **H** | Ruthless scope (§4). Timebox every non-moat task; if reuse can't do it in ≤2 days, cut it from the demo. AI does the grunt work (glue, tests, boilerplate); founder spends human hours only on kernel↔harness↔demo-script. Weekly "is the hero demo still on rails?" checkpoint. Freeze features 2 weeks before first investor meeting. |
| 3 | **LLM harness unreliable / non-deterministic** (Technical) | Live Claude calls flake, drift, latency-spike, hit rate limits, or produce a different (worse) answer on demo day than in rehearsal. The `live` path is largely unwired today. | **H** | **H** | Kernel is the safety net **by design** — the LLM cannot commit an invalid state past verification, so "wrong" degrades to "rejected + retried," not "broken board." For the demo: **cache/pin** the hero prompt's response as a golden fixture, replay it deterministically, with the live call as an optional "and yes it's really the model" flourish. Constrain outputs to a tool/schema the kernel validates. Retry + timeout + fallback-to-cached wrapper around every call. |
| 4 | **KiCad import edge cases** (Technical) | Real `.kicad_sch` / `.kicad_pcb` files have a decade of format quirks; our parser chokes on a version, a custom footprint, or a hierarchical sheet during the "always works" fallback segment. | **M** | **M** | Only support **curated import files we ship and control**; don't promise "import any board." Pin a KiCad format version. Reuse KiCanvas/kicad-rs parsing rather than hand-rolling. Have 3 pre-vetted import files; the demo uses one, the others are proof it's not a one-off. Fuzz the parser on a corpus offline, but never on stage. |
| 5 | **Tauri + web renderer integration pain** (Technical) | The seam between Tauri (Rust) ↔ webview ↔ KiCanvas ↔ our event stream eats weeks: IPC serialization, canvas lifecycle, focus/DPI/perf bugs, WebGL in webview2/wkwebview differences. | **M** | **H** | Spike this seam in **week 1** as the highest-risk integration — a "hello board renders from a kernel event" walking skeleton before any feature work. Keep the contract dead-simple: kernel emits versioned events → JSON → frontend renders; no bidirectional editing in the demo. If the renderer fights us, fall back to rendering to **SVG/PNG from the kernel** and displaying an image (still a real, kernel-derived board). |
| 6 | **Demo depends on the hardest piece** (Execution·Product) | The "whoa" hinges on generation+placement+route — the least-built, least-deterministic part. If it isn't ready, there's no hero moment. | **H** | **H** | Invert the dependency: make the **AI-review-on-a-real-board** segment the load-bearing wow, and treat generation as the *upside* reveal. Build generation **backwards from one known-good target board** (we know the answer; the harness "arrives" at it). Recorded-stream fallback means the hero flow plays even if live generation is down. Never let the demo's success require the riskiest subsystem to work live. |
| 7 | **Runway runs out before the raise** (Fundraise·Execution) | Pre-funding. Time and cash burn (API costs, parts-API subscription, living costs) exceed the window to a signed check; founder is forced to take a job / stall. | **M** | **H** | Keep spend near-zero: free/OSS reuse, cheap/metered API tiers, cache LLM calls (also fixes #3). Set a **hard demo-ready date at ~week 10**, leaving buffer to pitch while still solvent. Start warm investor conversations early (parallel, not after the build). Have a "extend runway" lever identified (contract work / grant / angel bridge) *before* it's an emergency. |
| 8 | **Flux / incumbent competition** (Market) | Flux (editor-first + copilot) or a well-funded entrant ships a convincing "AI designs your board" story first, or reframes our wedge as a feature. KiCad + a plugin could look "good enough." | **M** | **H** | Don't compete on editor features (we lose) — compete on the **moat narrative**: *verified-by-construction, traceable, replayable* AI, which is architecturally hard to bolt onto an editor-first tool. The deck must make the runtime-first distinction visceral in 30 seconds (§04-competitive-positioning). Move fast on the *demo*, not the platform. Being small and narrow is an advantage here — ship the wedge before they notice it. |
| 9 | **Scope creep back into "build the platform"** (Execution) | The founder's instinct (and the codebase's gravity) pulls toward widening the kernel — more DRC rules, more phases, a real editor — instead of shipping the demo. Feels productive, moves the raise nowhere. | **H** | **H** | The **hero-demo test** (§4): if a task doesn't appear in the hero demo or the deck, it's backlog, not now. Freeze the kernel's *feature* surface; only touch it to serve the demo. Engineering-science backlog #4–6 are **explicitly deferred** unless a curated example needs them on screen. Written "not-doing" list reviewed weekly. |
| 10 | **Investors don't buy the moat / "it's a wrapper"** (Fundraise·Market) | Pre-seed VCs pattern-match to "another GPT wrapper" or "cherry-picked demo," don't believe the kernel is defensible, or don't believe hardware-AI is a venture-scale market. | **M** | **H** | Show the **kernel rejecting a bad AI action live** — the one thing a wrapper cannot do — as the money shot. Lead with traceability (English sentence ↔ board finding) as visible proof of the substrate. Have the "why now / why us / why this is hard to copy" answers tight. Curated is fine (§2) as long as we're honest it's curated; skepticism is disarmed by *offering* to import the investor's own KiCad file (the fallback segment scales to that). |
| 11 | **Kernel↔UI event-stream contract drift** (Technical) | Frontend and backend are built in parallel against the event stream; if the contract wobbles, integration thrashes and the walking skeleton rots. | **M** | **M** | Freeze a **versioned event schema** as the single contract early (it already is versioned in the kernel). Generate TS types from the Rust types (one source of truth). Treat the schema as an API with a changelog; any change is a deliberate, tested bump. Contract tests on the seam. |
| 12 | **Parts / footprint data: cost, licensing, rate limits** (Technical·Legal) | Nexar/Octopart (or similar) API costs money, rate-limits, or has terms that complicate a demo/screenshots; KiCad libs have license/attribution nuances. | **M** | **M** | For the demo, **cache the parts data for the curated examples locally** — no live API dependency on stage, and near-zero cost. Use KiCad's own (permissively licensed) libraries for footprints. Confirm the parts-API terms permit demo/screenshot use before relying on it; keep the live API as a post-raise integration, not a demo-day dependency. |

**Reading the register:** risks 1, 3, 6, 9 (all H/H, all execution/technical) are the
cluster that actually decides success — and every one of them is mitigated by the
**same discipline**: ruthless curation + kernel-as-safety-net + never depend live on
the riskiest piece. That's not a coincidence; it's the strategy (§2, §4).

---

## 2. The demo-jank mitigation plan

Jank is the number-one way this demo dies. The counter is **not** "make generation
robust" (impossible in 3 months) — it's **engineering the demo so jank cannot appear
on stage.** Four pillars:

**a) Ruthless curation.** One hero example (e.g. the USB-C I²C temp sensor from
§4 of the overview), plus two backups, each hardened end-to-end until it is
*flawless every single time*. We do **not** demo arbitrary intent. The prompts,
parts, target board, and review findings are all known and rehearsed. *One
flawless scripted flow beats ten janky ones* — investors remember the one that
worked, not the breadth we didn't show.

**b) Recorded-event-stream fixture as fallback.** Because the kernel is
event-sourced and deterministically replayable, we capture the *entire* hero run
(intent → generate → validate → route → review) as a **golden event stream**. On
stage we can play the live path; if anything flakes (network, LLM drift, latency),
we transparently fall to the recorded stream, which renders through the *same* real
UI and *same* real kernel. The audience sees an identical, real board either way.
This single fixture neutralizes risks #1, #3, and #6 at once.

**c) The bulletproof KiCad-import → AI-review segment.** This segment has **no
generation and no LLM in the critical path**: parse a real, pre-vetted KiCad board
→ run the *already-built, 185-test* kernel rules (DRC/DFM/EMC/ampacity/impedance/
thermal) → AI narrates the findings. It works because it's just parsing + running
existing deterministic code. We **lead** with this to bank credibility, then reveal
generation as upside. If generation were to totally collapse, this segment alone is
still a real, differentiated demo.

**d) "One flawless flow" discipline.** No live typing of novel prompts. No "let's
try something the audience suggests" (offer that only as a *post-pitch* optional,
clearly framed as experimental). No feature shown that isn't on rails. Every demo
beat has a rehearsed happy path and a defined fallback.

### What is real vs curated vs staged — and the line we will not cross

| Element | Status | Honest framing |
|---|---|---|
| The correctness kernel + its checks | **Real** | Fully built, 185 tests, runs live on any board. This is the truth we're selling. |
| KiCad import → AI review | **Real** | Genuinely parses + checks a real board live. Files are curated (ours), the *engine* is real. |
| Traceability graph (intent ↔ findings) | **Real** | Produced by the kernel, not hand-drawn. |
| LLM reasoning / explanations | **Real model, pinned response** | A real Claude call produced it; we cache/replay for determinism. It's the model's actual output, just not re-rolled live. |
| Intent → generated board | **Real pipeline, curated target** | The harness really runs; the example is chosen so the harness reliably lands a known-good board. |
| Placement + route | **Curated / assisted** | Constrained to work for the hero example — not a general autorouter. |
| The specific examples shown | **Curated** | Hand-picked and hardened, not random. |

**The line we will not cross:** everything shown must be something the system *can
actually do* — real kernel, real model output, real traceability. We may **curate**
(pick easy-for-us examples), **pin** (cache a real response for determinism), and
**stage** (rehearse, script, pre-load data). We will **never fabricate** a result the
system can't produce, fake the kernel's verdict, hand-draw a "generated" board, or
claim generality we don't have. Curated + honest = a normal pre-seed demo.
Fabricated = fraud, and it also fails technical diligence the moment an investor asks
"can I try my own board?" — which is exactly why the import segment must scale to
their file. **If an investor asks whether it's curated, the answer is "yes, and
here's the real engine running on something you bring."**

---

## 3. Buy-vs-build decision table

Default is **REUSE/BUY**. We **BUILD** only the moat: the correctness kernel (already
built) and the AI harness. Everything else is a means to the demo and must not
consume moat-hours.

| Component | Decision | Specific tool / library | Rationale |
|---|---|---|---|
| **Desktop shell** | **BUY/REUSE** | **Tauri** | Rust backend = our kernel *is* the native core; local-first, small binary, offline. Zero reason to build a shell. |
| **Canvas / renderer** | **REUSE — never build** | **KiCanvas** (WebGL KiCad renderer) in the webview; fall back to kernel-rendered **SVG/PNG** | Rendering a PCB/schematic well is years of work and *not* the moat. KiCanvas already renders KiCad formats. SVG fallback de-risks the integration (#5). |
| **Schematic / PCB parsing** | **REUSE** | KiCanvas / **kicad-rs**-style parsers; pinned KiCad format version | Format parsing is a solved, thankless problem. Reuse and constrain to curated files (#4). |
| **Parts / footprint data** | **REUSE + cache** | **KiCad standard libraries** (footprints/symbols) + **Nexar/Octopart** for part metadata, **cached locally** for demo | Building a parts DB is a company by itself. Cache for the demo → no live dependency, near-zero cost (#12). |
| **Part search** | **BUY/REUSE (thin)** | Nexar/Octopart API behind a thin kernel-side adapter; cached results for curated examples | Search UX is not the moat; a thin adapter + cache is enough for the demo. |
| **Autorouting** | **BUILD — but tiny & curated (NOT general)** | Constrained/assisted routing for the hero example only; optionally lean on existing routing where trivial | A general autorouter is a moon-shot and out of scope (overview §6). We build *just enough* deterministic, curated routing to land the known-good hero board. |
| **LLM** | **BUY** | **Claude API** (Anthropic) via the kernel's `live` reasoning boundary; responses cached/pinned for the demo | Never train/host a model. The moat is the *kernel around* the LLM, not the LLM. Caching also fixes determinism (#3). |
| **AI harness (agent loop)** | **BUILD — the soul** | Our own agentic loop wired through the kernel's verification boundary + event stream | This *is* the product's soul (overview §6). The one place unbounded founder-hours are justified. |
| **Correctness kernel** | **ALREADY BUILT — the moat** | This repo's `eak/` Rust workspace (15-phase pipeline, verification, replay) | Our edge. Only touch it to serve the demo (#9). Do not widen its feature surface. |
| **Auth / accounts** | **BUY or SKIP** | **Skip for demo** (local app, no login). If needed later: an off-the-shelf provider (e.g. Clerk/Auth0/Supabase) | A local demo needs no auth. Building it now is pure scope creep. |
| **Telemetry / analytics** | **BUY (minimal) or SKIP** | Minimal local logging for the demo; PostHog/Sentry-class tool later | Not needed to raise. A tiny bit of instrumentation for waitlist signal at most. |
| **Packaging / distribution** | **BUY/REUSE** | **Tauri bundler** (`.dmg`/`.AppImage`/`.msi`), code-signing as needed | Tauri gives packaging almost free. No custom installer work. |

**One rule behind the whole table:** if a component is not the kernel or the harness
and it would cost more than a couple of days to build, we reuse, cache, stub, or cut
it. Founder-hours are the scarcest resource; spend them only on the moat and the demo
script.

---

## 4. Scope-discipline guardrails

The failure mode for a technical solo founder is *building the impressive platform
instead of shipping the fundable demo.* Concrete guardrails:

- **The hero-demo test.** For any proposed work, ask: *"Does this appear in the hero
  demo or the deck?"* If no → it goes on the backlog, not this week. This is the
  master filter; every other guardrail is a special case of it.
- **The moat test.** *"Is this the kernel or the harness?"* If no → reuse/buy/cut; do
  not hand-build it. (See §3.)
- **Freeze the kernel's feature surface.** The kernel is done enough (185 tests). New
  DRC rules, new phases, engineering-science backlog #4–6 are **deferred** unless a
  curated on-screen example literally needs them. No "while I'm in here" kernel work.
- **The 2-day reuse timebox.** If a non-moat capability can't be reused/stubbed into
  the demo in ≤2 days, cut the capability from the demo, not the timeline.
- **A written "NOT doing" list.** Maintain it next to the roadmap: general
  autorouting, a real editor, broad part coverage, collaboration/cloud, arbitrary-board
  manufacturing output, auth, multi-example generality. Re-read it weekly. Adding
  something back is a deliberate, logged decision.
- **Demo-on-rails rule.** No feature exists for the demo until it has a rehearsed
  happy path *and* a defined fallback (§2). "Works on my machine once" is not "in the
  demo."
- **Two-week feature freeze** before the first investor meeting: only polish,
  rehearsal, and deck after that point.
- **Weekly single question:** *"If I had to demo tomorrow, does the hero flow play
  flawlessly end-to-end?"* If no, everything else is a distraction until it's yes.

---

## 5. Kill / pivot criteria

Honest tripwires. If we hit one, we **narrow or pivot the wedge** rather than grind on
a dying plan. Better to pivot in week 6 than to burn the runway proving a wrong bet.

| Signal (by ~week) | What it means | Action |
|---|---|---|
| By ~wk 4: the **Tauri↔KiCanvas↔event-stream** skeleton still can't render a kernel-emitted board reliably | The integration seam (#5) is deeper than budgeted | **Narrow:** switch renderer to kernel-emitted SVG/PNG; drop interactive canvas from the demo. |
| By ~wk 6: **generation** can't reliably land even the one curated hero board | The riskiest piece (#6) won't come together in time | **Pivot the wedge to the pure AI-reviewer:** demo = import real board → kernel checks → AI explains + traces. Reframe the whole pitch around "trustworthy AI review of hardware," generation as roadmap. This is a *strong* fallback product, not a defeat. |
| Ongoing: the kernel keeps rejecting the harness with no path to a valid board | Harness↔kernel loop may be fundamentally mismatched for generation | **Narrow to review-only** (as above); generation becomes a post-raise research bet. |
| By ~wk 8–10: even the curated flow reads as "janky/wrapper" in honest self-review or friendly-investor feedback | The core "whoa" isn't landing (#1, #10) | **Simplify hard:** cut to the single most bulletproof segment (import→review→trace) and make *that* flawless and beautiful. One undeniable thing beats a broad shaky thing. |
| Warm investor conversations return consistent "this is a feature, not a company" / no moat belief | Market/moat thesis (#8, #10) isn't resonating | **Pivot the narrative** before pivoting the product: sharpen the runtime-first/traceability story; if still flat, reconsider whether the wedge is review-tooling, a KiCad plugin, or a dev-tool for hardware teams. |
| Runway hits the pre-set **week-10 hard line** without a demo-ready hero flow | Time thesis (#7) broken | **Stop building, start extending runway** (contract/angel/grant) and ship the best *narrow* demo you have; don't spend the last dollar polishing breadth. |

**The default pivot** (worth internalizing now): *if generation won't come together,
the product is the AI reviewer that you can trust — grounded by the same kernel, with
the same traceability, on real imported boards.* That fallback is real today (the
import→review path uses only shipped code) and is itself a fundable wedge. We are
never empty-handed.

---

## 6. Dependency risks

We deliberately stand on reused components to move fast. Each is a single point of
failure; each needs a known fallback so no external dependency can sink the demo.

| Dependency | Role | Failure mode | Fallback / mitigation |
|---|---|---|---|
| **KiCanvas** (renderer) | Renders schematic/PCB in the webview | Rendering bugs, webview/DPI/WebGL issues, unmaintained, license snag | Kernel emits **SVG/PNG** of the board (deterministic, from our own state) and we display an image. Real board, no third-party render dependency. Vendor/pin the version. |
| **KiCad file formats + libs** | Import boards; footprint/symbol data | Format-version drift, exotic constructs, license/attribution nuances | Ship & control curated files only; **pin a format version**; use permissively-licensed KiCad standard libs; parse via reused libraries, fuzzed offline. |
| **Parts API (Nexar/Octopart)** | Part metadata / search | Cost, rate limits, downtime, terms restricting demo use | **Cache locally** for curated examples → no live dependency on stage; confirm demo/screenshot terms; treat live API as post-raise. |
| **Claude API** | LLM reasoning/explanations in the harness | Latency, drift, rate limits, outage, cost | **Cache/pin** the hero responses (real model output) and replay deterministically; kernel rejects any invalid live output anyway; retry+timeout+fallback wrapper; recorded-event-stream fixture covers total outage. |
| **Tauri** | Desktop shell + packaging | Platform-specific webview quirks, packaging/signing friction | Mature, actively maintained; target **one primary OS** for the demo; SVG-render fallback reduces reliance on webview WebGL; keep the shell thin so a shell issue is contained. |
| **Rust kernel (`eak/`)** | The moat, the safety net | Our own bug surfaces on stage | It's the *most* trusted piece (185 tests, deterministic replay). Freeze its surface (#9); every demo path has a recorded golden replay; new work is additive and tested. |

**Cross-cutting principle:** the demo must be able to run **fully offline from cached
fixtures**. Every live dependency (LLM, parts API, network) is a *nice-to-have
flourish* layered on top of a self-contained, deterministic, kernel-driven core — not
a load-bearing requirement. If the venue Wi-Fi dies, the hero demo still plays,
because the kernel's event-sourced replay and cached responses make the whole flow
reproducible from disk. That property — designed into the kernel, not bolted on — is
both the demo's safety net and, not coincidentally, the moat we're selling.
