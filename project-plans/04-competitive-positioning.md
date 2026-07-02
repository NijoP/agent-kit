# Competitive Positioning — vs Flux / KiCad / AI-EDA startups

> Anchors to `00-overview.md` (the source of truth). If this doc contradicts the
> overview, the overview wins. Written 2026-07-02.
>
> **Honesty note up front:** our differentiator (the deterministic correctness
> kernel) is *real and shipped as code* (185 tests, clippy/fmt clean). But the
> **product** around it — the local IDE, the AI harness, the demo — is **not built
> yet** (see overview §7). So throughout this doc, advantages are tagged:
> **[SHIPPED]** = exists in the repo today · **[ON PAPER]** = designed/planned, not
> yet built · **[VERIFY]** = external claim we are not fully certain of and should
> confirm before repeating to an investor. Do not oversell paper as product.

---

## 1. Landscape map

The market splits into four camps: (A) **editor-first tools + an AI copilot**,
(B) **open/pro incumbent editors**, (C) **AI-EDA point startups** (autorouters,
generative schematic, parts/supply-chain), and (D) **silicon-EDA giants** adding AI
(adjacent, not direct). We sit in a fifth, mostly-empty box: **runtime-first, a
correctness kernel that AI drives** — inside a local IDE.

| Player | Camp | What it is | Approach | Core strength |
|---|---|---|---|---|
| **Flux** (flux.ai) | A — editor + copilot | Browser-based EDA (schematic + PCB layout), real-time collaborative | **Editor-first**: human designs in canvas, "Flux Copilot" chat *suggests* parts/answers/edits | Polished modern UX, collaboration ("Figma for hardware"), a **shipped** AI copilot, community/marketplace, a16z-backed. **Primary comparable.** |
| **KiCad** | B — open incumbent | Free open-source EDA suite (schematic + layout + 3D) | **Editor-first**, manual; no native AI. Plugin ecosystem | Free, ubiquitous, huge part/footprint libraries, de-facto **open file format**, enormous user base. Our import/interop target. |
| **Altium Designer** | B — pro incumbent | Professional desktop EDA + Altium 365 cloud | **Editor-first**, pro workflow; adding AI features incrementally [VERIFY scope] | Industry standard for serious hardware teams, deep feature set, ecosystem, enterprise distribution. Owned by Renesas (acq. 2024) [VERIFY]. |
| **JITX** | C — generative (code) | Programmatic hardware design; you write code (a Stanza-based DSL) that *generates* circuits | **Correct-by-construction generation** via constraint solving/search; not an interactive IDE | Closest philosophical cousin: correctness via code + solver, reusable design logic. Code-first (engineers-who-code audience). |
| **Quilter** | C — autorouter/auto-layout | ML-driven automated PCB **placement + routing** | Schematic in → manufacturable board out, automatically | Genuinely automating the hardest manual step (layout/route) with ML. YC-backed [VERIFY]. |
| **DeepPCB** (InstaDeep) | C — autorouter | Cloud AI **autorouter** (RL-based routing) | Netlist/board in → routed board out | Autorouting via reinforcement learning. Narrow, focused. |
| **Cofactr** | C — parts/supply-chain | Component **sourcing / inventory / procurement** platform | Not design — parts data, availability, supply-chain ops | Parts intelligence + supply chain. Complements (not competes with) design tools. YC-backed [VERIFY]. |
| **Diode** (diode.computer) | C — generative (newcomer) | "AI-native PCB design," text/code → schematic/PCB [VERIFY details] | Generative, AI-first | Newer entrant chasing text→hardware. Flag **[VERIFY]** — confirm current product scope before citing. |
| **Circuit Mind** | C — generative schematic | Automated **schematic generation from requirements** (UK) | Requirements/spec → circuit design, automated | Requirements→schematic automation for engineering teams. |
| **CELUS** | C — generative schematic | AI-driven schematic generation from functional blocks (Germany) [VERIFY] | Block-level spec → schematic, "design platform" | Functional-block → schematic automation. **[VERIFY]** current positioning. |
| **Zener** | C — newcomer | Text→hardware / AI design assistant [VERIFY — low confidence] | Generative/assistant | **[VERIFY]** — we are not confident this exists in the form implied; confirm before naming to investors. |
| **SnapMagic** (ex-SnapEDA) | C — parts/CAD libs | CAD symbol/footprint library + some AI search | Parts data, not design | Library breadth; parts discovery. Complementary. |
| **Cadence / Synopsys** | D — silicon-EDA giants | Chip-level (IC) EDA with AI (Cadence Cerebrus, Synopsys DSO.ai) | AI-driven optimization loops on **silicon**, not PCB | Proof that "AI closes the loop in EDA" works at scale — but a **different domain** (ICs). Relevant as narrative precedent, not direct competitor. |
| **Autodesk Eagle / Fusion Electronics** | B — incumbent-ish | Eagle (being folded into Fusion 360) | Editor-first, manual | Legacy hobby/prosumer base; declining as standalone [VERIFY]. |

**Read of the map:** *lots* of companies are automating **one step** (route, or
place, or schematic-gen, or parts) and *one* (Flux) is doing editor+copilot well.
**Nobody in the PCB camp is selling a deterministic correctness *runtime* that an
AI drives inside a local IDE.** That empty box is our thesis.

---

## 2. Positioning table — us vs the field

| Axis | **Us (EAK)** | **Flux** | **KiCad** | **Autorouter startups** (Quilter/DeepPCB) | **Generative** (JITX/Circuit Mind/Diode) |
|---|---|---|---|---|---|
| **Architecture** | **Runtime-first**: deterministic kernel owns engineering state; UI/AI are clients of it | **Editor-first** + copilot bolted on | **Editor-first**, no AI | Point tool (layout only) | Generator (schematic/board out) |
| **AI role** | **Verified-drives**: LLM proposes, kernel *gates* — invalid design cannot commit **[ON PAPER]** | **Suggests**: copilot advises, human is the safety net | None native | ML does one step end-to-end (black-box) | Generates; correctness varies by tool |
| **Trust / traceability** | Every action **traces to the original intent sentence**; event-sourced; **[kernel SHIPPED, UI ON PAPER]** | Copilot output is unverified suggestion | Manual DRC, human-owned | Result is a routed board; limited "why" | JITX has solver-backed rationale; others vary |
| **Deterministic replay** | **Yes** — event-sourced, replayable **[SHIPPED]** | No | N/A (manual) | No (stochastic ML) | Mostly no |
| **Local-first / offline** | **Yes** — Tauri native, Rust core, offline-capable **[ON PAPER]** | **No** — browser/cloud | **Yes** — desktop | Cloud | Mixed (often cloud) |
| **Verification depth** | 8 DRC + ERC/DFM/EMC/ampacity/impedance/thermal, typed quantities **[SHIPPED]** | Basic checks | Mature DRC (manual) | Route-legality only | Varies |
| **Maturity / users** | **Pre-product**: kernel only, **no UI, no users** | **Shipped, real users**, funded | **Millions of users**, decade+ mature | Shipped, early customers | Early-to-mid, some customers |
| **Moat** | The **substrate** (correctness kernel + traceability + replay) — hard to bolt on later | UX + community + collaboration | Ecosystem + ubiquity + format | ML models / route quality | DSL/solver IP (JITX) or data |

**How to read this table honestly:** on the *architecture / trust / determinism*
rows we are genuinely differentiated and the kernel is real. On the
*maturity / users* row we are **last** — everyone else has a product and we have a
demo-to-be. The positioning bet is that the architecture rows matter more *for the
specific promise of "AI you can trust to drive hardware"* than the maturity row —
because none of the shipped products can make that promise.

---

## 3. The one-sentence differentiation + proof points

**One sentence:**
> **Everyone else bolts an AI *copilot* onto an editor and hopes the human catches
> its mistakes; we built the deterministic correctness *runtime* first, so the AI
> can actually *drive* the board and every action it takes is verified, traceable,
> and replayable.**

**Supporting proof points:**

1. **Correctness kernel [SHIPPED].** A deterministic Rust kernel with a 15-phase
   pipeline, a verification engine (8 DRC rules + ERC/DFM/EMC/ampacity/controlled-
   impedance/thermal), and typed physical quantities. 185 tests, clippy + fmt clean.
   The AI *cannot* push an invalid design past it — correctness is *by construction*,
   not by review. (This is the load-bearing claim; it is code, not a slide.)
2. **Traceability [kernel SHIPPED, surfacing ON PAPER].** Event-sourced state means
   every generated artifact (a part choice, a net, a DRC finding) traces back to the
   original English intent sentence. "Why is this resistor here?" has a deterministic
   answer, not an LLM's after-the-fact guess.
3. **Deterministic replay [SHIPPED].** The whole design history is an event log you
   can replay identically. Audit, debug, and reproducibility — the things a
   probabilistic copilot structurally cannot offer.
4. **Local-first native IDE [ON PAPER].** Tauri + Rust core runs on the engineer's
   machine, offline-capable, IP stays local. Flux is browser/cloud-only; this is a
   real wedge for teams who won't put board IP in someone else's cloud.

---

## 4. "Why can't Flux / KiCad just add this?"

Short answer: **because the moat is the substrate, not the LLM — and you cannot
retrofit a deterministic runtime under a product that was architected editor-first.**

- **It's an architecture inversion, not a feature.** Flux and KiCad are built so the
  **canvas/editor owns the truth** and the human mutates it directly; an AI copilot is
  a *guest* that emits suggestions into that human-owned state. Our design **inverts
  ownership**: the **kernel owns the truth**, and *both* the human and the AI are
  clients that can only mutate state through verified, event-sourced transactions.
  Bolting that on means re-plumbing how every edit in the app is applied, undone,
  validated, and stored. That's a rewrite of the core, not a plugin.
- **Determinism fights their strengths.** Flux's advantages (real-time collaborative
  cloud canvas, fluid direct manipulation) are in tension with a strict, event-sourced,
  deterministic-replay core. You don't casually add "every state transition is a
  verified, replayable event" to a live collaborative editor.
- **"They have AI too" is not the moat.** Flux's copilot is real and probably better
  UX than ours will be at first. That's fine — **suggesting** is the easy 80%. The hard,
  defensible part is the **verification substrate that lets AI *drive* safely**, and
  that is exactly what an editor-first product is structurally worst-positioned to add.
- **KiCad specifically** is a manual, human-in-the-loop editor with a plugin API; a
  plugin can *call* checks but cannot make the core *own* a verified event-sourced
  engineering state. The open format is a gift to us (we import it), not a moat for them
  against this.

**Caveat (honest):** "architecturally hard" ≠ "impossible." A well-funded incumbent
*could* build a parallel runtime given time and money. Our defensibility is a **head
start on the substrate + focus**, not a patent. See §7 risks.

---

## 5. Honest weaknesses — where competitors are ahead, and how to neutralize

| Weakness (real) | Who's ahead | Severity | How we neutralize it in the pitch |
|---|---|---|---|
| **Product maturity** — we have a kernel, no shipped IDE/UI/agent | Flux, KiCad, Altium, all startups | High | Reframe the ask: pre-seed funds *productizing an already-hard-part-done kernel*, not starting from zero. Lead with the **one hero demo** (overview §4), not breadth. Show the kernel is real (tests, replay) so "no UI" reads as "de-risked core, UI is the fundable step." |
| **Zero users / no distribution** | Everyone | High | Don't claim traction we lack. Convert to a *plan*: waitlist + 2-3 named design partners as the pre-seed signal (see 05-fundraise-plan). Position KiCad's install base as our **import funnel**, not a wall. |
| **Funding** | Flux (a16z), Altium, YC startups | High | Solo + AI-assisted = **extreme capital efficiency** is the story. "They raised to build the editor *and* the AI; we reuse the editor and already built the hard kernel, so the same dollar goes further." |
| **UX / polish** | Flux (excellent), Altium | Med-High | Concede it. We are **not** out-UX-ing Flux in year 1. We **reuse** the canvas (KiCanvas) and compete on *trust*, not pixels. The demo must still feel premium on the one curated flow. |
| **Autorouting / layout automation** | Quilter, DeepPCB | Medium | Explicitly **out of scope year-1** (overview §6). We do *assisted/curated* routing and can *partner with / wrap* an autorouter later. Don't fight on their axis. |
| **Parts / supply-chain data** | Cofactr, Nexar/Octopart, SnapMagic | Medium | **Buy, don't build** — reuse a parts API. Not our moat; saying so is a strength (focus). |
| **Generative breadth** | JITX, Circuit Mind, CELUS | Medium | They generate; **we generate *and verify with a gate*.** Position generation as commoditizing and **verification as the durable layer** on top of anyone's generator (including, eventually, theirs). |
| **Domain credibility of a solo founder** | Incumbent teams | Medium | Lean on the shipped engineering-science layer (59 docs) + kernel test rigor as proof of depth. |

**Meta-point for the deck:** the honest framing *is* the persuasive framing. "Here's
exactly where they beat us, here's why it doesn't kill the thesis, here's the one thing
we do that none of them can" reads as founder maturity to a pre-seed investor.

---

## 6. Category framing — what to call ourselves

Four candidate frames, scored for a **pre-seed raise**:

| Frame | Pro | Con | Verdict |
|---|---|---|---|
| **"AI EDA"** | Obvious, searchable | Crowded, undifferentiated — puts us in a bucket with 10 startups; invites "how are you different from Flux?" as the *first* question | **Avoid as the headline** |
| **"Verifiable / autonomous hardware design"** | Captures the real differentiator (verify, drive) | "Autonomous" over-promises what's shipped (we're curated/assisted year-1); can trigger a credibility gap | Use the *word* "verifiable," not "autonomous" |
| **"The correctness runtime for AI hardware"** | Most *accurate* to the moat; category-defining; defensible | Abstract; a generalist investor may not instantly get "runtime"; needs one more sentence to land | **The technical thesis / second line** |
| **"Cursor for hardware"** | Instant comprehension, hot analogy, gets the meeting; matches overview §1 | Analogy alone invites "so it's a copilot?" — the exact thing we're *not* | **The hook / first line** |

**Recommendation:** lead with the analogy, immediately qualify with the moat.

> **Headline:** *"Cursor for hardware — built on a deterministic correctness runtime,
> so the AI can actually be trusted to drive the board."*

**Justification for pre-seed specifically:**
- Pre-seed checks are written on **narrative + analogy + founder + a wedge**, not on
  metrics. "Cursor for hardware" is the fastest path to *comprehension and a meeting*
  — it borrows a category everyone already believes in.
- But "Cursor for hardware" *unqualified* collapses us into Flux (a copilot). The
  **"correctness runtime"** clause is what converts the analogy into a *differentiated*
  bet in the same breath — it answers "why won't Flux just win?" before it's asked.
- Avoid "AI EDA" (commoditized) as the lead and "autonomous" (over-claims vs shipped)
  entirely at this stage. Grow into "autonomous" once the harness is real.

---

## 7. Risks from competitors + mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **Flux adds a "verification / trust" layer** and claims the same story | Medium | High | (a) It's an architecture inversion for them (§4) — hard and slow. (b) **Speed + focus**: ship the verified-drive demo before they pivot. (c) Deepen the moat where they're weakest: **local-first** (they're cloud-only) and **deterministic replay/traceability**. (d) Own the *language* of "correctness runtime" early so a bolt-on reads as a follow. |
| **A big incumbent moves** (Altium/Cadence/Synopsys bring PCB "AI + verification") | Low-Med (PCB); High (silicon precedent) | High | Incumbents move slowly on new architectures and protect existing revenue. Stay **narrow + local-first + startup-fast**. Cadence/Synopsys AI is *silicon*, not PCB — different domain, buys us runway. Be acquirable/partner-able rather than trying to out-enterprise them. |
| **An autorouter/generative startup (Quilter, JITX) extends into a full IDE with verification** | Medium | Med-High | They're deep on *one* step; a full trustworthy IDE is a different scope. **Partner/wrap** their generator under our verification layer rather than compete head-on. Our gate makes *their* output more trustworthy — a wedge for collaboration. |
| **KiCad ecosystem ships a credible open-source AI plugin** | Medium | Medium | Plugins can *call* checks but can't make the core *own* verified event-sourced state (§4). We ride KiCad as an **import funnel**, not a competitor; contribute to format interop to stay friendly. |
| **"Verification" gets commoditized** by an OSS DRC/AI-check library | Low-Med | Medium | Our moat is the **integrated runtime** (event-sourced state + replay + intent-traceability + typed quantities), not any single check. Individual rules being open is fine — the substrate is the defensible whole. |
| **We're too slow / demo slips** and Flux ships "trust" first | Medium | High | The single most controllable risk. Ruthless scope (one hero demo, curated), reuse over build, kernel already done. This is a **roadmap-execution** risk more than a competitive one — see 03-roadmap / 06-risks. |
| **Market timing** — investors decide "AI hardware design" is over-hyped/crowded | Medium | Medium | Differentiate *out* of the hype bucket with the correctness framing; don't sell "another AI EDA." Anchor to a concrete, verifiable demo, not vibes. |

---

## 8. Summary for the deck (one slide)

- **Category:** *Cursor for hardware — on a deterministic correctness runtime.*
- **Everyone else:** editor-first tool + an AI that **suggests** (human = safety net),
  or a point tool that automates one step.
- **Us:** runtime-first — the AI **drives**, the kernel **verifies**; every action is
  **traceable to intent** and **deterministically replayable**; **local-first**.
- **Moat:** the substrate, not the LLM — architecturally hard to bolt onto an
  editor-first product after the fact.
- **Honest status:** kernel is **real and tested today**; the IDE + harness + demo are
  **the fundable next step**, not yet shipped.

> Reminder for anyone using this doc: keep the **[SHIPPED] / [ON PAPER] / [VERIFY]**
> tags honest in the room. The strongest version of this pitch is the truthful one —
> a real hard core, a clear-eyed read of stronger-but-differently-positioned rivals,
> and one thing we do that none of them structurally can.
