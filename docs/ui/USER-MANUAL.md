# Electronics Agent Kit — User Manual

> **Product:** Electronics Agent Kit (EAK) — the AI‑native engineering operating system for hardware.
> **Edition:** Desktop (local‑first). **Interface version:** contract `v1`. **Manual rev:** 2026‑07‑17.
>
> This manual is written in the style of a premium appliance guide: it walks you through every
> **screen**, **menu tab**, and **button**, from the moment you open the application. It doubles as the
> canonical **UX + CMF specification** for the frontend squad — the screens it describes are the target
> the interface is built to. Where a screen is not yet built, it is marked **◐ planned**.
>
> **Design language in one line:** near‑black graphite surfaces, one electric‑signal accent, hairline
> borders instead of shadows, calm motion, data‑dense but quiet. Minimalist, precise, honest.

---

## Contents

1. [Before you begin](#1-before-you-begin)
2. [Overview — what EAK is](#2-overview--what-eak-is)
3. [What's included](#3-whats-included)
4. [The interface at a glance](#4-the-interface-at-a-glance) *(annotated diagram + legend)*
5. [Getting started](#5-getting-started) *(open → home → first design)*
6. [Screen‑by‑screen walkthrough](#6-screen-by-screen-walkthrough)
7. [Menu & command reference](#7-menu--command-reference)
8. [Buttons & controls reference](#8-buttons--controls-reference)
9. [Status indicators & honesty badges](#9-status-indicators--honesty-badges)
10. [Keyboard shortcuts](#10-keyboard-shortcuts)
11. [Design language — minimalist UI & CMF spec](#11-design-language--minimalist-ui--cmf-spec)
12. [Troubleshooting & screen states](#12-troubleshooting--screen-states)
13. [Glossary](#13-glossary)

---

## 1. Before you begin

**System requirements**

| | Minimum |
|---|---|
| OS | Linux (webkit2gtk), macOS 12+, or Windows 10+ |
| Display | 1440 × 900 or larger (the workspace is designed for ≥ 1280 wide) |
| Runtime | Bundled — EAK ships its engineering kernel as its native core; no cloud account required to run a design |
| Network | Optional. Needed only for **Live** reasoning; **Cassette** mode runs fully offline |

**Two words you will see everywhere.** EAK never blurs what is real:

- **The kernel owns the truth.** Every part, net, and check you see is an *owned fact* the kernel
  validated — never something the AI merely "said."
- **Honesty labels.** Every surface is tagged **REAL · CURATED · CASSETTE · SCAFFOLD** so you always
  know whether a result was computed live, tuned for one example, replayed from a recording, or is a
  throwaway preview. See [§9](#9-status-indicators--honesty-badges).

---

## 2. Overview — what EAK is

EAK is a desktop application where you **describe what you want to build in plain language**, and an AI
engineer designs it **inside a kernel that checks every step**. You watch requirements, blocks, parts,
nets, placement, routing, and verification appear live — each one traceable back to the sentence you
typed, each derived number wearing its confidence.

It is not a drawing tool you push shapes around in. It is closer to **"Cursor for hardware"**: you
converse with an engineering agent, the agent proposes, and the **kernel validates or rejects** every
proposal at a single seam. Crucially, EAK is built to be **honest** — it declares its own assumptions
and **refuses to release a design while a critical assumption is undischarged**.

Three ideas shape every screen:

1. **Everything on screen is a projection.** The panels are *views* of owned truth, never the truth
   itself. Close a panel and nothing is lost.
2. **You own the goals; the AI owns the work.** Every agent action lands at a **human checkpoint** you
   Approve or Reject.
3. **Traceability is always one hover away.** Any object can show its chain back to intent.

---

## 3. What's included

| Item | Description |
|---|---|
| **EAK desktop app** | The workspace (this manual). The engineering kernel is bundled as its native core. |
| **Sample design** | The hero example — *"USB‑C powered I²C temperature sensor, < 1 W"* — preloaded so you can run end‑to‑end on first launch. |
| **Cassette library** | Recorded AI runs that replay deterministically offline — the app works with no network. |
| **Import bridge** | Open an existing `.kicad_pcb` and run EAK's checks over it *(review‑only scaffold)*. |

---

## 4. The interface at a glance

When a design is open, EAK shows a single **Workspace** window. Learn these six regions once; every
task in this manual refers back to them by number.

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│ ⚡ Electronics Agent Kit         ⌘K   Search & run commands…                  ▢  –  ✕    │ ①  Title bar
├────┬──────────────────────────┬────────────────────────────────┬────────────────────────┤
│    │  AGENT                   │  BOARD ▾   BLOCKS   IR          │  INSPECTOR             │
│ ▣  │ ┌──────────────────────┐ │ ┌────────────────────────────┐ │ ┌────────────────────┐ │
│ ⌗  │ │ You                  │ │ │                            │ │ │ State │ Trace │ ⓘ  │ │
│ ⎇  │ │  USB‑C I²C temp <1W  │ │ │                            │ │ ├────────────────────┤ │
│ ⚙  │ │                      │ │ │       [ board render ]     │ │ │ ▾ Requirements  4  │ │
│    │ │ Agent                │ │ │                            │ │ │ ▾ Blocks        3  │ │ ⑦  Inspector
│ ④  │ │  ⟳ reasoning…        │ │ │                            │ │ │ ▸ Parts / BOM   6  │ │
│    │ │  ▸ Assumption ▲ crit │ │ │                            │ │ │ ▸ Nets          9  │ │
│    │ │    [Approve] [Reject] │ │ │                            │ │ │ ▸ Placement     6  │ │
│    │ └──────────────────────┘ │ └────────────────────────────┘ │ │ ▸ Tracks       12  │ │
│    │  ⌨ Describe intent…   ▶  │  ③  Canvas                     │ └────────────────────┘ │
│    │  ②  Agent panel          │                                │                        │
├────┴──────────────────────────┴────────────────────────────────┴────────────────────────┤
│ REVIEW    Violations 2 · Waivers 1 · Gate ⛔ BLOCKED                          ▴ collapse  │ ⑤  Review dock
│  ⛔ DRC   Track width 0.15 mm < 0.20 mm floor      → net N3        [ Explain ] [ Fix ]    │
│  ⚠ BOM   U2 has no second‑source                   → part U2       [ Explain ] [ Waive ]  │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│ ● CASSETTE   Gate BLOCKED   Fidelity 3 calc · 1 assumed   contract v1   tests 235 ✓       │ ⑥  Status bar
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

**Legend — the six regions (memorize these)**

| № | Region | What it is | Detail |
|---|---|---|---|
| ① | **Title bar** | App mark, the global **Command bar** (`⌘K` / `Ctrl+K`), window controls. Minimal chrome by design. | → [§6.8](#68-command-palette) |
| ② | **Agent panel** *(left)* | The conversation with your AI engineer: your intent, its reasoning, its assumptions, and the **Approve / Reject** checkpoints. | → [§6.4](#64-agent-panel) |
| ③ | **Canvas** *(center)* | The rendered projection of the design. Tabs switch between **Board**, **Blocks** (architecture), and **IR** (the raw owned model). | → [§6.5](#65-canvas) |
| ④ | **Activity rail** *(far left)* | Icon rail that switches what the left panel shows: **▣ Agent · ⌗ Designs · ⎇ Revisions · ⚙ Settings**. | → [§8](#8-buttons--controls-reference) |
| ⑤ | **Review dock** *(bottom)* | Every check the kernel ran: violations, waivers, and the **manufacturing‑gate verdict**. Each row **Explains**, **Fixes**, or **Waives**. | → [§6.7](#67-review-dock) |
| ⑥ | **Status bar** *(bottom strip)* | At‑a‑glance truth: reasoning mode, gate verdict, fidelity summary, contract version, test count. | → [§9](#9-status-indicators--honesty-badges) |
| ⑦ | **Inspector** *(right)* | Tabbed detail: **State** (the live engineering tree), **Trace** (provenance to intent), **ⓘ** (properties of the selected object). | → [§6.6](#66-inspector) |

> **Spatial rule (how to read this manual):** regions are always named by their fixed position — *left*
> panel, *center* canvas, *right* inspector, *bottom* dock, *top* bar. On‑screen labels are printed in
> **bold** so the text maps 1:1 to what you see.

---

## 5. Getting started

### 5.1 Open the application

Launch **Electronics Agent Kit** from your applications menu (or `eak` on the command line). A native
window opens to the **Home** screen — no browser, no login wall.

### 5.2 The Home screen

```
┌───────────────────────────────────────────────────────────────────────┐
│                                                                        │
│                     ⚡  Electronics Agent Kit                          │
│               Describe hardware. Watch it get engineered.              │
│                                                                        │
│      ┌──────────────────────────────────────────────────────────┐     │
│      │  ⌨  Describe what you want to build…                    ▶ │     │  ← Intent bar
│      └──────────────────────────────────────────────────────────┘     │
│         e.g. “USB‑C powered I²C temperature sensor, < 1 W”             │
│                                                                        │
│   [ + New design ]   [ ⌗ Open… ]   [ ⬇ Import .kicad_pcb for review ]  │
│                                                                        │
│   Recent                                                               │
│    ▸ usb‑c‑temp‑sensor        edited 2 min ago     Gate ✓ RELEASED     │
│    ▸ buck‑3v3‑2a              edited yesterday      Gate ⛔ BLOCKED      │
│                                                                        │
│   ● CASSETTE mode · offline ready                       contract v1    │
└───────────────────────────────────────────────────────────────────────┘
```

| № | Control | Function |
|---|---|---|
| 1 | **Intent bar** | Type a one‑line design intent and press **▶ Run** (or `Enter`) to create a design and start engineering it immediately. |
| 2 | **+ New design** | Create an empty design and open the Workspace, then converse with the agent there. |
| 3 | **⌗ Open…** | Open a saved design. |
| 4 | **⬇ Import .kicad_pcb for review** | Load an existing board and run EAK's checks over it *(review‑only)*. → [§6.10](#610-import-for-review) |
| 5 | **Recent** | Your recent designs, each showing its last **gate verdict**. Click to reopen. |

### 5.3 Your first run (5 steps)

1. In the **Intent bar**, **type**: `USB‑C powered I²C temperature sensor, < 1 W`.
2. **Press** `Enter`. The **Workspace** opens and the run begins.
3. Watch the **Agent panel** (left): the AI reasons, then raises an **Assumption** (e.g. *"ambient ≤ 70 °C"*) at a checkpoint.
4. **Click Approve** on the assumption. Requirements, blocks, parts, and nets stream into the **Inspector** (right) and the board draws in the **Canvas** (center).
5. Read the **Review dock** (bottom): the **gate verdict** shows **RELEASED** or **BLOCKED**. If blocked, click **Explain** on a violation to see why.

> **Checkpoint — what you should see now:** a rendered board in the center, a populated engineering
> tree on the right, a green or red gate verdict at the bottom, and a status bar reading
> `● CASSETTE · Gate …`. You have run a full design end‑to‑end.

---

## 6. Screen‑by‑screen walkthrough

Each screen below follows the same shape: **purpose → wireframe → controls → checkpoint.**

### 6.1 Home / Launch

*Covered in [§5.2](#52-the-home-screen).* Purpose: choose how to start — a new intent, an existing
design, or an import for review. It is the only screen with no open design.

### 6.2 New Design (Intent entry)

**Purpose.** Capture the design intent — the single sentence everything else traces back to.

You can enter intent two ways: from the **Home** intent bar, or, inside the Workspace, from the
**Agent panel** input at any time. Intent is not a throwaway prompt — it becomes the **owned
`IntentCaptured` fact** at the root of the traceability chain.

**Controls**

| Control | Function |
|---|---|
| **Intent field** | Free text. One clear sentence works best (*power source, function, key constraint*). |
| **▶ Run** | Commit the intent and start the pipeline. |
| **Model ▾** *(optional)* | Choose the reasoning source: **Cassette** (offline, deterministic) or **Live**. → [§6.11](#611-settings--preferences) |

> **Checkpoint:** the intent appears as the first message in the **Agent panel** and as the root node
> in the **Trace** tab.

### 6.3 The Workspace

**Purpose.** The one window where a design lives. It is the six‑region layout from [§4](#4-the-interface-at-a-glance).

**Rearranging the workspace.** Every panel is dockable and collapsible:

- **Drag** a panel's title to move it; **double‑click** the divider to collapse.
- **Layout presets** (top‑right, or **View > Layout**): **Focus** (canvas only), **Engineer** (all
  panels), **Review** (canvas + review dock enlarged). Pick a preset to reshape the whole window at once.
- Any panel can be toggled from the **View** menu or the **Command bar**.

> **Minimalism note:** by default EAK opens in **Engineer** layout. If a panel is empty (e.g. no
> review has run yet) it shows a quiet empty‑state hint rather than blank space — never clutter.

### 6.4 Agent panel

**Purpose.** Converse with your AI engineer and approve or reject its work. This is the heart of the
AI‑native flow — the equivalent of a code editor's agent chat, but for hardware.

```
┌── AGENT ─────────────────────────────┐
│ You                                  │
│  USB‑C I²C temperature sensor, <1 W  │
│                                      │
│ Agent · Requirement                  │
│  Derived 4 requirements  ✓ committed │
│  ▸ REQ‑1 Input 5 V USB‑C  [→ trace]  │
│                                      │
│ Agent · Assumption   ▲ CRITICAL      │
│  “Ambient operating temp ≤ 70 °C.”   │
│  Rests on: REQ‑2                     │
│  ┌─────────────┬──────────────┐      │
│  │  ✓ Approve  │  ✕  Reject    │      │  ← human checkpoint
│  └─────────────┴──────────────┘      │
│                                      │
│ Agent · Tradeoff                     │
│  Chose TMP102 over TMP117            │
│  (cost ↓, accuracy ↓)  [→ compare]   │
├──────────────────────────────────────┤
│ ⌨ Describe intent or ask…         ▶  │
└──────────────────────────────────────┘
```

**How to read a message.** Each agent message is typed by what it produced — **Requirement**,
**Assumption**, **Tradeoff**, **Risk**, **Part**, **Violation**. Data‑bearing messages carry a
`[→ trace]` link (jumps the **Trace** tab to that object) and, where relevant, a **fidelity tag**
showing how a number was obtained (assumed / first‑order / calculated / simulated / measured).

**Controls**

| Control | Function |
|---|---|
| **✓ Approve / ✕ Reject** | Resolve a checkpoint. **Reject** sends the item back with your reason; nothing enters the design until you approve. |
| **[→ trace]** | Reveal this object's provenance in the **Trace** tab. |
| **[→ compare]** | Open a **Tradeoff** to see the rejected alternatives and the rationale. |
| **⟳ Regenerate** *(hover a message)* | Ask the agent to try that step again. |
| **Intent input ▶** | Send a new instruction or question at any time. |

> **Checkpoint:** an approved **Critical assumption** clears the honesty gate; you will see the
> **Gate** in the status bar move from **BLOCKED (assumption open)** toward **RELEASED**.

### 6.5 Canvas

**Purpose.** The rendered projection of the design. The center tabs switch what you're looking at:

| Tab | Shows |
|---|---|
| **Board** | The physical PCB — footprints, placement, copper, nets. Rendered via the embedded viewer. **SCAFFOLD**‑tagged (a projection, never the source of truth). |
| **Blocks** | The functional architecture — blocks and their connections (the "what it is" before the "where it goes"). |
| **IR** | The raw **owned model** (the intermediate representation) for engineers who want to see exactly what the kernel holds. |

**Canvas controls (top‑right of the canvas)**

| Icon | Control | Function |
|---|---|---|
| ⤢ | **Fit** | Fit the design to the view. |
| ＋ / － | **Zoom** | Zoom in/out (also scroll / pinch). |
| ▦ | **Layers** | Toggle copper / silkscreen / courtyard visibility (Board tab). |
| ◎ | **Cross‑probe** | Select an object here to highlight it in the Inspector and Review — and vice‑versa. |

> **Checkpoint:** click a footprint on the **Board**; its part highlights in the **Inspector**, its
> requirement chain lights up in **Trace**, and any violation on it flags in the **Review** dock.

### 6.6 Inspector

**Purpose.** The right‑hand detail column. Three tabs.

**State** — the live engineering tree, folded from the kernel's event stream as the run proceeds:

```
State │ Trace │ ⓘ
──────────────────────
▾ Requirements      4
   REQ‑1  Input 5 V USB‑C
   REQ‑2  I²C temperature readout
▾ Blocks            3
   Power ▸  Sensor ▸  Connector ▸
▸ Parts / BOM       6
▸ Nets              9
▸ Placement         6
▸ Tracks           12
```

**Trace** — the provenance viewer. Select any object (here or on the canvas) and see its chain:
`intent sentence → requirement → block → part → net`. Hovering a link **highlights the whole path**
across every panel. This is the "AI you can trust" made visible: nothing exists without a reason.

**ⓘ Properties** — the fields of the selected object (value, units, origin, **fidelity tag**, the
capability call that created it). Read‑only projections; you change the design by instructing the
agent, not by editing here.

> **Checkpoint:** every leaf in the **State** tree can be selected, and every selection fills **Trace**
> and **ⓘ**. If an object had no trace, it would not be here — the kernel does not surface un‑owned facts.

### 6.7 Review dock

**Purpose.** Every check the kernel ran, and the single verdict that matters: can this design be
released to manufacturing?

```
REVIEW   Violations 2 · Waivers 1 · Gate ⛔ BLOCKED                        ▴
 ⛔ DRC   Track width 0.15 mm < 0.20 mm floor     → net N3    [Explain][Fix]
 ⚠ BOM   U2 has no second‑source                  → part U2   [Explain][Waive]
 ✓ ERC   All nets terminated                                  —
```

**Controls**

| Control | Function |
|---|---|
| **Explain** | The agent explains the violation in plain language — the rule, the number, and why it matters. |
| **Fix** | The agent proposes a change to resolve it; the fix returns to a **checkpoint** for your approval. |
| **Waive** | Record a **Waiver** — an owned, auditable decision to accept a finding, with your reason. Not a delete: the finding and the waiver both stay in history. |
| **Gate chip** | The manufacturing‑gate verdict: **RELEASED** or **BLOCKED**, with the blocking reason (open violation *or* undischarged critical assumption). |
| **▴ / ▾** | Collapse or expand the dock. |

> **Severity colors:** ⛔ blocking error · ⚠ warning · ✓ pass. See [§11](#11-design-language--minimalist-ui--cmf-spec).

### 6.8 Command palette

**Purpose.** Reach *every* action from one key, so the chrome stays empty. This is EAK's minimalism
engine — most menus exist only as discoverability aids for what the palette already does.

- **Open:** `⌘K` (macOS) / `Ctrl+K` (Windows/Linux), or click the **Command bar** in the title bar.
- **Modes** (type a prefix): plain text = *run a command*; `>` = *actions*; `@` = *jump to an object*
  (requirement, part, net); `?` = *ask the agent*.

```
┌─ ⌘K ─────────────────────────────────────────┐
│ >                                            │
│  › Run design                        ⌘↵      │
│  › Verify only (no synthesis)                │
│  › Import .kicad_pcb for review…             │
│  › Toggle Review dock                 ⌘J     │
│  › Switch reasoning: Live ⇄ Cassette         │
│  › New revision (snapshot)                   │
└──────────────────────────────────────────────┘
```

> **Checkpoint:** press `Ctrl+K`, type `verify`, press `Enter` — a verification‑only pass runs and the
> **Review dock** refreshes without re‑synthesizing the design.

### 6.9 Revisions *(◐ planned)*

**Purpose.** Git‑for‑hardware. Because the kernel is event‑sourced, any point in a design's history is
a named position you can tag and compare.

- **⎇ Revisions** (activity rail) lists tagged snapshots.
- **New revision** (Command bar) tags the current position with a label + message.
- **Diff** two revisions to see what was added, changed, or superseded — computed by replaying history,
  not by guessing.

### 6.10 Import for review

**Purpose.** Run EAK's owned checks over a board you already have — a working review path that does not
depend on generation. **SCAFFOLD**, expiry‑tagged: it exists to prove the review engine, not to make EAK
a general PCB editor.

1. **Home > ⬇ Import .kicad_pcb for review** (or **File > Import**).
2. Select a `.kicad_pcb`. EAK parses it and funnels every entity through the **same capability seam** a
   generated design uses — no back door.
3. The board renders in the **Canvas**; the **Review dock** fills with real findings.

> **Checkpoint:** the status bar reads `● SCAFFOLD (import)` and the gate verdict reflects real checks
> over the imported board.

### 6.11 Settings / preferences

**Purpose.** Configure reasoning, appearance, and the demo‑safe defaults. Reached via **⚙** (activity
rail) or **Edit > Preferences**.

| Group | Setting | Options |
|---|---|---|
| **Reasoning** | Mode | **Cassette** (offline, deterministic — default) · **Live** (calls the model) |
| | Cassette | Choose which recorded run replays |
| | Timeout / fallback | If **Live** stalls, fall back to a cassette automatically |
| **Appearance** | Theme | **Graphite** (dark, default) · **Graphite Light** *(◐ planned)* |
| | Density | **Comfortable** · **Compact** |
| | Motion | **Full** · **Reduced** (respects OS "reduce motion") |
| **Workspace** | Default layout | Focus · Engineer · Review |
| | Honesty labels | Always show (recommended) |

### 6.12 Export

**Purpose.** Emit the design's artifacts. **File > Export** (or Command bar → *Export*).

| Export | Produces |
|---|---|
| **`.kicad_pcb`** | The board, for downstream ECAD. A projection of owned truth. |
| **BOM (CSV)** | The validated bill of materials. |
| **Report (PDF)** | Intent → requirements → verdict, with every finding and its trace — the honest record. |
| **Snapshot** | A named revision of the current design. |

---

## 7. Menu & command reference

EAK keeps a **thin menu bar**; anything here is also in the Command bar (`⌘K`). Menu paths are written
**Menu > Item**.

### File
| Item | Shortcut | Action |
|---|---|---|
| **New Design** | `Ctrl+N` | Create an empty design. |
| **Open…** | `Ctrl+O` | Open a saved design. |
| **Import .kicad_pcb…** | — | Load a board for review *(scaffold)*. |
| **Save / Snapshot** | `Ctrl+S` | Tag the current position as a revision. |
| **Export ▸** | — | `.kicad_pcb` · BOM · Report · Snapshot. |
| **Close Design** | `Ctrl+W` | Return to Home. |
| **Quit** | `Ctrl+Q` | Exit EAK. |

### Edit
| Item | Shortcut | Action |
|---|---|---|
| **Undo / Redo** | `Ctrl+Z` / `Ctrl+Shift+Z` | Step through your *instructions* (the design is history‑based). |
| **Copy** | `Ctrl+C` | Copy the selected object's id / value. |
| **Preferences…** | `Ctrl+,` | Open Settings. → [§6.11](#611-settings--preferences) |

### View
| Item | Shortcut | Action |
|---|---|---|
| **Command Palette** | `Ctrl+K` | Open the Command bar. |
| **Toggle Agent** | `Ctrl+1` | Show/hide the left Agent panel. |
| **Toggle Inspector** | `Ctrl+2` | Show/hide the right Inspector. |
| **Toggle Review** | `Ctrl+J` | Show/hide the bottom Review dock. |
| **Canvas: Board / Blocks / IR** | `Ctrl+3/4/5` | Switch the center tab. |
| **Layout ▸** | — | Focus · Engineer · Review. |
| **Zoom In / Out / Fit** | `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | Canvas zoom. |

### Design
| Item | Shortcut | Action |
|---|---|---|
| **Run** | `Ctrl+Enter` | Run the full pipeline on the current intent. |
| **Verify only** | — | Re‑run checks without re‑synthesizing. |
| **Re‑run from…** | — | Re‑run from a chosen phase. |
| **Revisions ▸** | — | New revision · Compare. |

### Agent
| Item | Shortcut | Action |
|---|---|---|
| **New instruction** | `Ctrl+L` | Focus the Agent input. |
| **Reasoning: Live ⇄ Cassette** | — | Switch the reasoning source. |
| **Assumptions…** | — | List open/discharged assumptions. |
| **Model & keys…** | — | Configure the live model. |

### Help
| Item | Action |
|---|---|
| **User Manual** | Open this manual. |
| **Keyboard Shortcuts** | Show the shortcut sheet. → [§10](#10-keyboard-shortcuts) |
| **Honesty Legend** | Explain REAL / CURATED / CASSETTE / SCAFFOLD. |
| **About EAK** | Version, contract version, licenses. |

---

## 8. Buttons & controls reference

**Activity rail (④, far left)** — one‑click switch of the left panel:

| Icon | Name | Opens |
|---|---|---|
| ▣ | **Agent** | The conversation panel *(default)*. |
| ⌗ | **Designs** | Your designs and recent files. |
| ⎇ | **Revisions** | History snapshots and diffs *(◐ planned)*. |
| ⚙ | **Settings** | Preferences. |

**Title bar (①)**

| Control | Function |
|---|---|
| **⚡ EAK mark** | Home / about. |
| **Command bar `⌘K`** | Search objects, run commands, ask the agent. |
| **▢ – ✕** | Standard window controls. |

**Primary action buttons** (consistent everywhere):

| Button | Meaning | Style |
|---|---|---|
| **▶ Run** | Start / re‑run engineering. | Primary (accent‑filled). |
| **✓ Approve** | Accept an agent proposal at a checkpoint. | Success. |
| **✕ Reject** | Send a proposal back with a reason. | Quiet/ghost. |
| **Explain** | Plain‑language reasoning for a finding. | Ghost. |
| **Fix** | Agent proposes a resolving change. | Ghost → checkpoint. |
| **Waive** | Record an auditable acceptance. | Ghost, warning‑tinted. |

> **One‑primary rule:** a screen shows **at most one accent‑filled button** at a time (usually **Run**
> or **Approve**). Everything else is a quiet ghost button. This is what keeps the UI calm.

---

## 9. Status indicators & honesty badges

The **status bar (⑥)** is the app's conscience — read left to right:

```
● CASSETTE   Gate BLOCKED   Fidelity 3 calc · 1 assumed   contract v1   tests 235 ✓
```

| Indicator | Meaning |
|---|---|
| **● Reasoning mode** | **REAL/LIVE** (green), **CASSETTE** (violet), **CURATED** (amber), **SCAFFOLD** (gray, dashed). Never hidden. |
| **Gate** | **RELEASED** (green) or **BLOCKED** (red) + reason. The single most important status. |
| **Fidelity** | Summary of how the design's numbers were obtained — *assumed / first‑order / calculated / simulated / measured*. |
| **contract vN** | The interface contract version — proves the view matches the kernel it's projecting. |
| **tests N ✓** | Kernel self‑test count (dev/demo builds). |

**Honesty badges** appear inline on any surface that isn't fully live:

| Badge | Color | Means |
|---|---|---|
| **REAL** | green | Computed live, this run. |
| **CURATED** | amber | Real code, inputs tuned for one example. |
| **CASSETTE** | violet | A recorded run replayed deterministically. |
| **SCAFFOLD** | gray, dashed border | A throwaway preview (e.g. the board render, the import bridge) — expiry‑tagged, never a standing capability. |

> **Promise:** investors and users forgive curation, not deception. EAK labels every claim so
> live‑versus‑curated is never blurred.

---

## 10. Keyboard shortcuts

| Action | Windows / Linux | macOS |
|---|---|---|
| Command palette | `Ctrl+K` | `⌘K` |
| Run design | `Ctrl+Enter` | `⌘↵` |
| New instruction to agent | `Ctrl+L` | `⌘L` |
| New design | `Ctrl+N` | `⌘N` |
| Open | `Ctrl+O` | `⌘O` |
| Save / snapshot | `Ctrl+S` | `⌘S` |
| Toggle Agent / Inspector / Review | `Ctrl+1` / `Ctrl+2` / `Ctrl+J` | `⌘1` / `⌘2` / `⌘J` |
| Canvas Board / Blocks / IR | `Ctrl+3` / `4` / `5` | `⌘3` / `4` / `5` |
| Zoom in / out / fit | `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | `⌘=` / `⌘-` / `⌘0` |
| Approve / Reject checkpoint | `Ctrl+↵` / `Ctrl+⌫` | `⌘↵` / `⌘⌫` |
| Preferences | `Ctrl+,` | `⌘,` |

> Modifiers are spelled out in‑app (Control, Command, Shift). Shortcuts are shown next to every command
> in the palette so you learn them by using them.

---

## 11. Design language — minimalist UI & CMF spec

This is the specification that makes the screens above look and feel premium: **minimalist, precise,
honest**. It is grounded in the visual language of Linear, Vercel/Geist, Raycast, Warp, and Radix. Build
to these exact tokens; do not improvise a second system.

### 11.1 Identity

**Theme name:** *Graphite + Signal.* Near‑black graphite surfaces carry the work; a single
electric‑signal accent marks only what is live, focused, or primary. The feeling is a precision
instrument, not a consumer app.

### 11.2 C — Color (tokens)

Dark theme is the default. Rule: **near‑black, never pure black**; surfaces get *lighter* as they rise
toward the user; **one chromatic accent, used scarcely.**

| Token | Hex | Use |
|---|---|---|
| `--bg-app` | `#0A0B0D` | Window floor. |
| `--surface-base` | `#101317` | Panels (Agent, Inspector, Review). |
| `--surface-raised` | `#161A1F` | Cards, message bubbles, popovers. |
| `--surface-overlay` | `#1C2127` | Hover / selected rows, menus. |
| `--border-subtle` | `rgba(255,255,255,0.06)` | Default hairline between regions. |
| `--border-strong` | `rgba(255,255,255,0.12)` | Focused container, active tab underline. |
| `--text-primary` | `#E7EAEE` (~90 %) | Primary text — never pure `#fff`. |
| `--text-secondary` | `#9BA3AD` | Labels, secondary values. |
| `--text-muted` | `#646C77` | Hints, disabled, metadata. |
| `--accent` | `#4CC2FF` | **Signal.** Primary button, focus ring, active trace path, live dot. |
| `--accent-pressed` | `#37A7E4` | Accent pressed / active. |

**Semantic (status) — desaturated for dark:**

| Token | Hex | Use |
|---|---|---|
| `--pass` | `#48B98A` | ✓ passing checks, RELEASED gate, REAL badge. |
| `--warn` | `#E0A93B` | ⚠ warnings, CURATED badge, Waive. |
| `--error` | `#FF5C5C` | ⛔ blocking violations, BLOCKED gate. |
| `--cassette` | `#8B7CF6` | CASSETTE badge / replay mode. |
| `--scaffold` | `#5B6672` | SCAFFOLD badge (with a **dashed** border). |

> **Accent discipline:** on any given screen, `--accent` appears on **one** element at a time (the
> single primary action or the current focus). If two things glow, one is wrong.

### 11.3 M — Material (surfaces)

- **Hierarchy comes from hairline borders + a one‑step surface lift, not drop shadows.** Cards are
  `--surface-raised` on `--surface-base` with a `--border-subtle` outline. No glowing box‑shadows.
- **Shadows are reserved for truly floating layers** — the command palette, menus, dialogs:
  `0 8px 24px rgba(0,0,0,0.45)` plus an inset top highlight `inset 0 1px 0 rgba(255,255,255,0.04)`.
- **Blur/glass** only on the command palette overlay and modal scrims (`backdrop-filter: blur(12px)`),
  never on ordinary panels.
- **Layer order** (each one step lighter): base → card → popover → menu → modal.
- Optional very‑subtle grain on large dark fields (the canvas backdrop) to prevent banding — barely
  perceptible.

### 11.4 F — Finish (motion & interaction)

- **Timings:** micro‑interactions `120–160 ms`; panel open/close `200 ms`; page/layout transitions
  `240 ms`. Easing `cubic-bezier(0.4, 0, 0.2, 1)`; enter `ease-out`, exit `ease-in`.
- **Hover:** prefer an **opacity/brightness shift** (e.g. row background `--surface-overlay`, ghost
  buttons `text → primary`) over a color swap. No jumpy scale on hover.
- **Press:** `1px` translate or `0.98` scale — a small, tactile acknowledgment.
- **Focus:** always a visible `:focus-visible` ring — `2px` `--accent` at `~40 %` alpha, `2px` offset.
  Never remove focus outlines.
- **Live feedback:** the reasoning **● dot** pulses gently while the agent works; streaming rows fade+rise
  in (`8px`, `160 ms`) — enough to feel alive, never enough to distract.
- **Reduced motion:** honor OS "reduce motion" — cross‑fades replace movement, the pulse becomes static.

### 11.5 Typography

- **UI family:** `Inter` (or `Geist Sans`), with system fallback. **Mono family:** `ui-monospace,
  "Geist Mono", SFMono-Regular, Menlo` for ids, values, nets, and the IR — data always in mono.
- **Weights:** `400` body · `500` labels/medium · `600` headings. Avoid `700+` except a display mark.
- **Negative letter‑spacing on large sizes:** `−0.01em` at 18 px, up to `−0.03em` at 28 px+. This is the
  pro tell.
- **Scale (data‑dense):** `12/16` caption · `13/18` mono/data · `14/20` body‑sm (default UI) · `16/24`
  body · `18/28` panel titles · `24/32` · `32/40` display (Home only).

### 11.6 Spacing, radius, density

- **4 px base grid:** `2, 4, 8, 12, 16, 24, 32, 48`. Pro density: `8 px` vertical rhythm inside panels,
  `8–12 px` component padding, inputs `8px 12px`.
- **Radius (restrained):** buttons/inputs `6px`; cards/panels `8px`; badges/pills `9999px`. **Never**
  over‑round (>12 px reads consumer/cheap). Nested radius = outer − padding.
- **Panels:** `16px` internal padding, `1px --border-subtle` dividers between regions, `12px` gutters.

### 11.7 Iconography

- One line‑icon set (e.g. Lucide/Phosphor), `1.5px` stroke, `16–20px`. **No emoji as UI icons** (the
  `⚡ ▣ ⌗ ⎇ ⚙` glyphs in this manual are wireframe stand‑ins for real line icons).
- Icons are `--text-secondary` at rest, `--text-primary` on hover, `--accent` only when active.

### 11.8 Anti‑patterns — do not ship these

The tells that make a UI look cheap or AI‑generated. Explicitly banned:

1. Blue→purple **hero gradients** or gradient text.
2. **Colored 3–4 px left‑border cards** and the "three rounded feature cards + soft shadow + thin‑line
   icon" triptych.
3. **Over‑rounded** corners everywhere (16 px+) and glowing `box-shadow`.
4. Pure `#000` background with pure `#fff` 100 %‑opacity text (halation on OLED).
5. Default Tailwind palette, emoji icons, Inter at default letter‑spacing.
6. Uniform‑lightness surfaces (no elevation) or dead flat grays with zero hue.

> **The through‑line:** hierarchy from *contrast and space*, not decoration. One accent. Borders over
> shadows. Calm motion. If a screen feels busy, remove — don't add.

---

## 12. Troubleshooting & screen states

| Symptom | Likely cause | Fix |
|---|---|---|
| Agent shows **⟳ reasoning…** and never resolves | **Live** mode stalled / no network | Status bar → **switch to Cassette**, or set a live **timeout/fallback** in Settings. |
| **Canvas is empty** after a run | Placement/routing not reached, or the run **BLOCKED** early | Read the **Review dock** — an open critical assumption or violation stops synthesis. Resolve the checkpoint. |
| **Gate stays BLOCKED** after fixing a violation | An **undischarged critical assumption** remains | Open the **Agent panel** → **Approve** (discharge) the assumption. Gate clears. |
| Everything reads **CASSETTE** | Offline / demo default | Expected. Switch to **Live** in Settings for computed reasoning. |
| Import fails | `.kicad_pcb` failed to parse, or an entity was **rejected at the seam** | This is not a bug — the kernel re‑validated the board and declined a bad entity. See the error's reason. |
| Panel missing | Toggled off | **View** menu (or `Ctrl+1/2/J`) to bring it back, or pick a **Layout** preset. |

**Empty states (by design, never blank):**

- No design open → **Home** screen with the intent bar.
- Review not yet run → *"Run a design or import a board to see checks."*
- No violations → a single green line: *"✓ No blocking findings. Gate RELEASED."*

---

## 13. Glossary

| Term | Meaning |
|---|---|
| **Intent** | Your one‑line description of what to build — the root of all traceability. |
| **Requirement / Block / Net / Part / Placement / Track** | Owned engineering objects the kernel validates and the panels project. |
| **Capability seam** | The single gate every proposal (AI or import) passes through to become an owned fact — no back doors. |
| **Assumption** | A stated presumption the AI must **discharge**; a *critical* open one **blocks release**. |
| **Fidelity tag** | How a number was obtained: assumed · first‑order · calculated · simulated · measured. |
| **Tradeoff** | A recorded choice with its rejected alternatives and rationale preserved. |
| **Waiver** | An owned, auditable decision to accept a finding. |
| **Manufacturing gate** | The final check: **RELEASED** or **BLOCKED**, with a reason. |
| **Revision** | A named position in the design's event history (git‑for‑hardware). |
| **Honesty label** | REAL / CURATED / CASSETTE / SCAFFOLD — how real a surface is. |
| **Contract vN** | The versioned interface between the kernel (truth) and the UI (projection). |

---

*This manual describes the target interface. Regions ①–⑦, all menus, and the CMF tokens in §11 are the
build spec for the frontend squad; screens marked ◐ planned are not yet implemented. Where this manual
and `project-plans/00-product-vision.md` ever disagree, the vision wins.*
