# Electronics Agent Kit — Product Spec (MVP)

> Anchored to `00-overview.md` (source of truth). This doc turns that overview into a
> concrete product: what the MVP is, who it's for, how the IDE feels, the hero demo an
> investor watches, scope boundaries, and what "fundable-done" means. Written 2026-07-02.

---

## 1. Product definition

**The MVP is: a local, native AI-native EDA IDE — "Cursor for hardware."** A Tauri desktop
app whose native core is the existing Rust correctness kernel, whose soul is an agentic AI
harness, and whose canvas is reused (KiCanvas / KiCad formats). The engineer states intent in
English; the agent drives; the kernel verifies every action and keeps it traceable to intent.

**The MVP IS:**
- A **desktop IDE** (offline-capable, local-first) — not a web app, not a cloud service.
- An **AI harness you can trust to drive**: every agent action is executed as a kernel
  capability and **must pass verification before it commits**.
- A **generate loop**: English intent → requirements → architecture → parts/BOM → schematic →
  starter placement + assisted route, each validated live.
- An **AI review loop**: import a real KiCad board → run the full check suite → AI explains
  every finding and suggests a fix, in plain English.
- **Fully traceable**: every requirement, part, net, and finding links back to the sentence
  that caused it; the whole session **replays deterministically** to identical state.
- **Curated and hardened**: a small set of examples that work flawlessly, every time.

**The MVP IS NOT:**
- **Not a KiCad replacement** — we don't out-edit KiCad; we reuse its canvas and formats.
- **Not a general autorouter** — routing is assisted + curated, not arbitrary-board.
- **Not broad part coverage** — curated libraries + one parts API, not the whole market.
- **Not collaboration / cloud / multi-user** — single engineer, single machine.
- **Not manufacturing-grade output for arbitrary boards** — the released IR is demo-grade.
- **Not a chatbot bolted onto an editor** (that's Flux); the kernel, not the human, is the
  safety net.

---

## 2. Target user

**Primary: the hardware engineer at a small startup who has no senior to review their board.**
Secondary: **juniors / new-grads** doing real boards without a mentor, and **serious makers**
graduating from breakouts to their first fabricated multi-layer PCB.

| | Who they are | Their pain today | Why they try us |
|---|---|---|---|
| **Startup HW eng** | 1–3 person hardware team, ships boards on deadlines | No senior reviewer; a missed DRC/EMC/thermal issue = a $2–10k respin + weeks lost | An always-on "senior reviewer" that catches issues *and explains them* before fab |
| **Junior / new-grad** | Knows theory, thin on practice | KiCad DRC is cryptic; datasheets are dense; no one to ask "is this right?" | Plain-English explanations grounded in real design state — learns while working |
| **Serious maker** | Moving from breakouts to a real 2–4 layer board | Doesn't know what they don't know (ampacity, impedance, decoupling) | Types intent, gets a *checked* starter board to learn from and iterate on |

**Shared pain:** the expertise gap is invisible until the board comes back broken. Existing
tools *edit*; they don't *reason* and they don't *teach*. **Why they'd try it now:** it's a
free/cheap local download that behaves like a senior engineer reviewing over their shoulder —
low commitment, immediate "it caught something I'd have missed" payoff.

---

## 3. The IDE experience

### Window layout

```
┌───────────────────────────────────────────────────────────────────────────────┐
│  Intent bar:  “USB-C powered I²C temp sensor, < 1 W”      [Generate] [Review] ● │  top bar
├──────────────┬─────────────────────────────────────────┬──────────────────────┤
│ LEFT RAIL    │            CENTER CANVAS                 │  RIGHT AGENT PANEL     │
│ Design tree  │   (reused KiCanvas: schematic / board)   │  Agent thread          │
│ • Intent doc │                                          │  • streaming reasoning │
│ • Requirements│  tabs:  [Schematic] [Board] [3D]        │  • proposed actions    │
│ • Blocks     │                                          │    [Accept] [Reject]   │
│ • BOM/parts  │   selection highlights net/part          │  • ✓ verified badges   │
│ • Nets/Layers│                                          │  • inline explain/fix  │
│ Examples ▾   │                                          │                        │
│ Import KiCad │                                          │                        │
├──────────────┴─────────────────────────────────────────┴──────────────────────┤
│ BOTTOM DOCK:  [ Checks ]  [ Traceability ]  [ History ]                          │
│  Checks: DRC · ERC · DFM · EMC · Ampacity · Impedance · Thermal  (by severity)   │
│  Traceability: provenance graph  •  History: event log / replay scrubber         │
└───────────────────────────────────────────────────────────────────────────────┘
```

- **Left rail** — the design *navigator*: a typed tree of the kernel's engineering state
  (intent → requirements → blocks → BOM → nets → layers), plus the curated **Examples** menu and
  **Import KiCad** entry point.
- **Center canvas** — the **reused** renderer (KiCanvas / KiCad engine). We never build editing
  primitives; we drive them. Selecting a net or part here syncs the agent and the checks dock.
- **Right agent panel** — the harness: the agent thread, streamed reasoning, and **proposed
  actions with Accept/Reject** and green **verified** badges. This is where the product lives.
- **Bottom dock** — **Checks** (the full verification suite, filterable by severity), **Traceability**
  (the live provenance graph), and **History** (the event log + a replay scrubber).

### Intelligent features (harness) — and why each needs the kernel

| Feature | What it does | Why only the kernel makes it possible |
|---|---|---|
| **1. Natural-language → subsystem** | "Add a USB-C power path, < 1 W" spawns a validated block (parts, nets, constraints) projected onto the canvas. | The agent's proposal is executed as a **kernel capability** against the event-sourced entity graph; the output is real typed state, not text an editor has to trust. |
| **2. Verified auto-actions** | The agent applies multi-step edits (place decoupling caps, size a trace) autonomously. | Every action **runs through verification before commit**. The agent *physically cannot* land a design that fails DRC/ERC — the commit path goes through the kernel, not around it. |
| **3. Inline explain** | Click any part/net/finding → the agent explains it (why this value, what constraint it meets). | Explanations are grounded in the kernel's **typed quantities + provenance chain**, so they're about the real design, not hallucinated. |
| **4. Verified fix** | For any finding, the agent proposes a fix; the kernel **re-verifies the candidate** and only offers it if it clears. | The fix is checked by the same deterministic rule engine that found the issue — proposal and proof are one loop. |
| **5. Live traceability** | Every requirement/block/part/net/finding links back to the English sentence that produced it; the graph fills in as generation streams. | The runtime is **event-sourced with provenance on every event** (the `trace` capability already exists); traceability is a projection, not a bolt-on. |
| **6. AI review of an imported board** | Import a KiCad board → run DRC/ERC/DFM/EMC/ampacity/impedance/thermal → agent narrates each finding + severity + fix. | The kernel *judges* with a deterministic rule suite; the AI only *narrates*. Reliable review = deterministic checks + LLM explanation, which an editor-first copilot can't guarantee. |
| **7. Show-your-work replay** | Scrub design history; every agent decision is a recorded event; the session replays to identical state. | The runtime's **deterministic replay** (already proven, `replay` command) means the demo — and any customer's session — is reproducible and auditable. |

The through-line: **the LLM reasons, the kernel rules.** Each feature is a thin harness capability
over a kernel primitive that already exists (capabilities, verification, provenance, replay). That
boundary is the moat — a copilot bolted onto an editor cannot retrofit it.

---

## 4. The hero demo (second-by-second)

One curated example — **"USB-C powered I²C temperature sensor, < 1 W"** — shown in the local IDE.
Target runtime ~3.5 minutes. The whole thing is deterministic and replayable, so it's flawless
every take.

| Beat | Time | On screen | The point |
|---|---|---|---|
| **0. Cold open** | 0:00–0:10 | Native IDE opens locally (no browser, no login). Empty canvas, agent panel idle, intent bar focused. | "This is a real desktop app running on my machine." |
| **1. Intent** | 0:10–0:25 | Founder types the one sentence and hits **Generate**. Agent panel: *"Planning requirements…"* | One English sentence is the whole input. |
| **2a. Requirements** | 0:25–0:50 | Left rail fills with parsed **requirements** (5 V USB-C in, 3.3 V rail, I²C, < 1 W). Traceability dock draws the first edges from the sentence. | The kernel captured intent as typed, traceable state. |
| **2b. Architecture** | 0:50–1:15 | Agent streams **blocks** (USB-C input → regulator → sensor + decoupling). Blocks appear in the tree; each is validated live (green ticks). | Reasoning is visible *and* checked as it lands. |
| **2c. Parts / BOM** | 1:15–1:45 | Agent selects parts (regulator, sensor, passives); **BOM** populates. Each part traces to the requirement that drove it. | Real part selection, grounded and traceable — not a picture of a schematic. |
| **2d. Schematic** | 1:45–2:10 | Canvas **Schematic** tab renders the netlist. ERC runs live; checks dock shows green. | A connected, electrically-checked schematic from one sentence. |
| **3. Starter board** | 2:10–2:40 | **Board** tab: placement + an **assisted/curated route** completes and renders. | A physical board, not just a schematic — the curated path that always works. |
| **4a. AI review** | 2:40–3:05 | Checks dock runs the full suite (DRC/DFM/EMC/**ampacity/impedance/thermal**). One finding surfaces (e.g. a trace under ampacity margin). | The kernel judges the board with real physics. |
| **4b. Explain + verified fix** | 3:05–3:25 | Click the finding → agent explains it in plain English and offers **[Accept fix]**. Accept → kernel re-verifies → finding clears, badge goes green. | AI you can *trust to fix*, because the kernel re-checks. |
| **4c. Trace to intent** | 3:25–3:35 | Click the finding's trace → the graph highlights the path all the way back to the original English sentence. | Every atom of the board is accountable to intent. |
| **5. The "whoa"** | 3:35–3:45 | Pull back: one sentence → checked, explained, fully-traceable board, AI reasoning visible, kernel guaranteeing correctness — all local. | The single takeaway the investor remembers. |

**Bulletproof fallback (never shown to fail):** if generation is ever risky live, open with
**Import a real KiCad board → AI review** (beats 4a–4c only). That path just parses + runs the
existing rule engine, so it works every time and still lands the core "AI review you can trust."

---

## 5. Scope table (year-1 MVP)

| Capability | In scope (MVP) | Out of scope (year-1, post-raise) |
|---|---|---|
| **IDE shell** | Tauri app, panels, intent bar, checks/trace/history docks | Plugins/extensions, theming marketplace |
| **Canvas** | Reused KiCanvas render (schematic + board), selection sync | Building our own editor / editing primitives |
| **Generation** | Intent → requirements → arch → BOM → schematic on **curated** examples | Arbitrary-design generation, generalized synthesis |
| **Routing/layout** | Assisted + curated route for demo boards | General autorouter, arbitrary-board layout |
| **Verification** | Full existing suite: 8 DRC + ERC/DFM/EMC/BOM/constraint + ampacity/impedance/thermal | New physics domains (SI/PI sims, full 3D EM) |
| **AI review** | KiCad import → checked report + explanations + fixes | Support for every EDA format; auto-repair at scale |
| **Parts** | Curated KiCad libs + one parts API (Nexar/Octopart) | Broad market coverage, price/stock optimization |
| **Traceability / replay** | Live provenance graph, deterministic replay/scrub | Team audit trails, cloud history, sign-off workflows |
| **Collaboration** | — | Multi-user, cloud sync, real-time co-editing |
| **Output** | Demo-grade Manufacturing IR / export on curated boards | Manufacturing-grade output for arbitrary boards |
| **Platform** | Local desktop, offline-capable, single OS target for demo | Multi-OS hardening, mobile, web |

---

## 6. Success criteria (fundable-done)

"Done" is **not** a feature count. It's the point where a pre-seed investor writes a check.
Measured by signals, not scope:

| Signal | Target for "fundable" |
|---|---|
| **Hero demo reliability** | Generate loop runs **flawlessly on ≥ 3 curated examples**, ≥ 20 consecutive clean takes each (deterministic replay makes this provable). |
| **AI-review reliability** | KiCad import → review works on **≥ 10 real third-party boards** with correct, non-embarrassing findings + explanations. |
| **Demo artifacts** | A **≤ 4-min recorded demo video** + a **live-runnable local build** + a sharp deck anchored to the moat. |
| **Waitlist** | **≥ 250 qualified signups** (target-user, not randoms) from a landing page + demo video. |
| **Design partners** | **≥ 5 design-partner conversations booked**, **≥ 2–3 verbally committed** to trial the tool. |
| **Investor signal** | **≥ 3 investor meetings** where the demo lands and a follow-up / term-sheet conversation opens. |
| **Narrative proof** | One-line pitch, moat, and "why now" hold up under a skeptical hardware investor's questions. |

If the generate demo works on 3 curated boards, review works on 10 real ones, 250 people want
in, and 3 partners say "let me try it" — **that's the raise.** Everything else is post-raise.

---

## 7. User journeys

**Journey A — "Review my KiCad board" (the wedge).**
A junior at a 2-person hardware startup has a 4-layer board due for fab tomorrow and no one to
check it. She opens the IDE, clicks **Import KiCad**, and picks her project. The board renders on
the canvas; the kernel runs the full suite. The checks dock shows 3 findings: a trace below
ampacity margin on the 3.3 V rail, insufficient decoupling near the MCU, and an impedance
mismatch on the USB pair. She clicks the ampacity finding — the agent explains *why* in plain
English and offers **[Accept fix]** (widen the trace to X mm). She accepts; the kernel re-verifies;
the finding clears. She ships with confidence. **Value in < 5 minutes, zero design work by us.**

**Journey B — "Generate a starter design from intent" (the wow).**
A maker moving off breakouts types *"USB-C powered I²C temp sensor, < 1 W"* and hits **Generate**.
The agent streams requirements → blocks → parts → schematic → starter board, each checked live,
the traceability graph filling in. In ~3 minutes he has a **checked** starter board he can learn
from and iterate on — including decoupling and trace sizing he wouldn't have known to add. He
uses **inline explain** to understand each choice. **A senior engineer's starting point, from one
sentence.**

**Journey C — "Explain it like a senior would" (the stickiness).**
An engineer inherits a design and doesn't understand a net. She clicks it on the canvas; the agent
explains what it is, what constraint it satisfies, and traces it back to the requirement that
created it. She asks *"why this regulator?"* in the agent panel and gets a grounded answer from
the BOM's provenance. **The tool doesn't just check — it teaches, grounded in real state.**

---

## 8. One-line pitch (carried from the overview)

**"Cursor for hardware — an AI harness you can actually trust to design boards, because a
deterministic engineering kernel verifies everything it does."**
