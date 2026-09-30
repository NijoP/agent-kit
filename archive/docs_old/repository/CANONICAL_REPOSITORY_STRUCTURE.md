# Canonical Repository Structure — Electronics Agent Kit

> **Design only. Nothing in the repository has been modified, moved, renamed, or deleted.**
> This document is the output of an 8-swarm repository-architecture analysis (Scanner · Folder ·
> Documentation · Code · Vision-Alignment · Simplification · Canonical-Design · Safety). It proposes
> ONE canonical structure and a safe, incremental, reversible migration plan. **No file operations
> may begin until the founder approves.** The Product Vision (`project-plans/00-product-vision.md`) is
> the highest authority; every folder must justify its existence against it. Written 2026-07-10.

**Headline:** the repository is **architecturally sound** — the clean-architecture rings, the
dependency direction, and the runtime are correct and compiler-guarded. **Zero folders warrant
deletion.** The problems are *hygiene and drift*, not rot: one exact 17-file duplication, a three-way
"agents" naming collision, two coexisting doc generations with mixed-in reports, one misfiled folder,
and missing standard files (root README / LICENSE / CI). All are fixable by **merge / archive / rename
/ dedup**, never deletion.

---

## 1. Current repository structure

**Counts:** 7 top-level directories (6 tracked + `.claude/`) · 64 directories · ~311 files · **247
markdown** · 46 Rust · 14 TOML. No root `README`, `LICENSE`, `CONTRIBUTING`, or CI (`.github/`).

```
ELECTRONICS AGENT KIT DEVELOPMENT/
├── app/                    Tauri desktop shell scaffold (src-tauri bridge + ui/) — outside eak/ workspace
├── docs/                   Architecture spec (17 subdirs: foundation, core, compiler, engineering, data,
│                           crosscutting, presentation, integration, collaboration, knowledge, quality,
│                           governance, decisions[ADRs], agents, state-machines) + README/CONVENTIONS/GLOSSARY
├── eak/                    Rust workspace — 11 crates (the runtime = the product)
├── engineering-science/    Knowledge/warrant layer — 8 subdirs (physics, mathematics, electrical, pcb,
│                           manufacturing, industry, runtime-mapping, compliance)
├── project-plans/          Product/strategy — TWO generations (see below) + reports/ledgers, 16 files
├── team/                   Autonomous org — 17 role charters + protocols/structure
└── .claude/agents/         17 agent configs — BYTE-IDENTICAL mirror of team/agents/
```

**Two doc generations coexist in `project-plans/`** (the central finding):

| Prefix | Older (2026-07-02, committed) | Newer (2026-07-10, canonical) |
|---|---|---|
| 00 | `00-overview.md` ("MVP source of truth") | `00-product-vision.md` (**priority 1 — wins all**) |
| 01 | `01-product-spec.md` | `01-engineering-philosophy.md` (**priority 2**) |
| 02 | `02-technical-architecture.md` | `02-engineering-world-model.md` (**priority 3**) |

Plus reports/ledgers intermixed: `08-overnight-execution-2026-07-03.md` (ledger), `09-legacy-framing-audit.md`,
`10-philosophy-alignment-report.md` (reports), `11-build-roadmap.md` (canonical construction order),
and GTM docs `04`–`07`. `project-plans/README.md` still indexes only the *older* generation.

---

## 2. Repository health

| Dimension | Grade | Evidence |
|---|---|---|
| **Clean-architecture rings / dependency direction** | **A** | 11 crates map 1:1 to rings; every edge points inward; `kernel_has_no_outward_dependencies` guard passes (`eak-runtime/src/lib.rs:28`). Cargo forbids cycles. |
| **Runtime integrity** (determinism, seam, replay) | **A** | Single commit path, capability seam, byte-identical replay — all intact and tested (236 tests). |
| **Code placement** | **A−** | Rings correct; drift: `eak-kicad` undocumented in README + not in `[workspace.dependencies]`; `eak-engines` mislabeled "trivial sequencer" (it owns 15 rules). |
| **Intra-crate organization** | **B−** | `eak-domain` (1,392-line single `lib.rs`) and `eak-engines` (3,304-line single `lib.rs`) are monoliths flat by ring, not by Map. (A *code* task — `02 §Part IV`; not part of this repo-structure migration.) |
| **Documentation organization** | **B** | `docs/` ring tree is the best-organized cluster; but two-generation supersession, reports mixed with living canon, stale index. |
| **Duplication** | **C** | `team/agents/` ⇄ `.claude/agents/` = 17 byte-identical files (dual maintenance). 3-way "agents" naming collision. |
| **Repo hygiene** | **C** | No root README, LICENSE, CONTRIBUTING, or CI. |
| **Vision alignment (structure)** | **A−** | Folders structurally align; legacy "Cursor/EDA-IDE" framing in older docs is a *content* issue tracked by `09-legacy-framing-audit.md`, not a structure issue. |
| **Overall** | **B+** | Healthy core; needs one cleanup pass. No structural rot; no deletions. |

---

## 3. Folder purpose — why every top-level folder exists

| Folder | Purpose (the concern it owns) | Owner | Justified by vision? |
|---|---|---|---|
| `eak/` | **The runtime = the product.** Deterministic kernel + compiler + backend, as a Rust workspace. | Kernel/Eng | **Yes** — `01` "the runtime is the product." |
| `docs/` | **Architecture** — the *what/how* design specs + ADRs, ring-structured. | Architecture | Yes — the design of the sovereign runtime. |
| `engineering-science/` | **Engineering science** — codified physics/math/practice → the thresholds the runtime enforces (the warrant layer). | EDA scientist | Yes — vision §9.4–9.6. |
| `project-plans/` | **Research / product canon** — vision, philosophy, world-model, roadmaps (highest authority). | Founder/PM | Yes — home of the canonical vision. |
| `app/` | **Frontend shell** (Tauri) — an *application on the substrate*; the kernel↔UI bridge. Deliberately outside `eak/`. | Desktop/FE | Yes — an interface to the runtime (vision §2). |
| `team/` | **The autonomous org** — role charters + operating protocols (governance). | Org/Founder | Yes — how the product gets built. |
| `.claude/` | **Tooling harness** — Claude Code agent configs (operative mirror of `team/agents/`). | Tooling | Yes — the build harness. |

**Every top-level folder justifies its existence.** The only folder that fails the test is a
*subfolder*: `engineering-science/compliance/` holds audit *reports*, not codified science (§5).

---

## 4. Folders to KEEP

All seven top-level folders, and all `docs/` and `engineering-science/` and `eak/crates/`
subfolders **except** the two singled out below — **KEEP, unchanged in location.** This includes the
three placements one might be tempted to "fix" but must not:

- **`eak/` and every crate** — moving any crate breaks Cargo `path=`/workspace deps and the guard test. **Freeze.**
- **`app/` outside the `eak/` workspace** — deliberate (Tauri's webkit2gtk/Node sysdeps must not gate the kernel build); its `../../eak/crates/...` path-deps only survive because neither `app/` nor `eak/` moves. **Freeze.**
- **`engineering-science/` at top level** — it is the vision §9 warrant layer; MVP *defers implementing* it, but the knowledge is permanent. **Keep at root** (extract only `compliance/`, §5).
- **`docs/decisions/` (ADRs)** — the correct and only ADR home; well-numbered 0001–0017. **Keep.**

---

## 5. Folders to MERGE

| Folder | Merge into | Why |
|---|---|---|
| `engineering-science/compliance/` (3 files) | `project-plans/reports/` | It is a point-in-time architecture-review **audit** (32 findings + repair backlog), **not** codified engineering science. It pollutes the permanent knowledge layer, and its own report is already flagged **stale** (the code has advanced past it). Reports belong with reports. |
| `.claude/agents/` (17 files) | conceptually merge with `team/agents/` → **one authored source** | The 17 files are **byte-identical**. `team/agents/` is the authored source; `.claude/agents/` is the operative mirror Claude Code loads. Keep *both on disk* but generate `.claude/agents/` **from** `team/agents/` via a sync script — ending the dual hand-maintenance (the legacy-framing audit literally says "edit both mirrors"). |

---

## 6. Folders to ARCHIVE

**No existing folder is archived wholesale.** Archiving happens at the *file* level (§10) into two
**new** subfolders that the migration creates:

- `project-plans/archive/` — the superseded 2026-07-02 doc generation + the dated ledger.
- `project-plans/reports/` — point-in-time reports (audits, alignment, health, compliance).

These are additive homes for existing files, not deletions.

---

## 7. Folders recommended for DELETION

**None.** This is a deliberate, evidence-backed conclusion, not an omission.

Every folder either (a) owns a real, vision-justified concern (§3), (b) is corrected by *merge* into
a better home (§5, `compliance/`), or (c) is a *duplication* resolved by making one copy generated
rather than deleted (§5, `.claude/agents/` must physically remain for the tooling to load subagents —
**deleting or gitignoring it would break Claude Code**). Nothing in the tree is dead, orphaned, or
unjustified. **Deletion is the wrong instrument here; the right instruments are merge, archive,
rename, and dedup.**

---

## 8. Markdown files to MOVE

| From | To | Reason | Link blast radius (rewrite in same commit) |
|---|---|---|---|
| `project-plans/09-legacy-framing-audit.md` | `project-plans/reports/legacy-framing-audit.md` | Point-in-time report, not living canon. | `00-product-vision.md:9` links to it. |
| `project-plans/10-philosophy-alignment-report.md` | `project-plans/reports/philosophy-alignment.md` | Point-in-time report. | `01-engineering-philosophy.md` links to it. |
| `docs/architecture-health-report.md` | `project-plans/reports/architecture-health.md` | Dated audit at the `docs/` root among living indexes. | Low. |
| `engineering-science/compliance/*.md` (3) | `project-plans/reports/engineering-science-compliance/` | Audit reports misfiled in the knowledge layer (§5). | **10 science docs** link `../compliance/…`; `engineering-science/README.md` lists it. |
| `docs/agents/*` (14) | `docs/domain-agents/*` | Kill the 3-way "agents" naming collision; these are *in-product design agents*, distinct from org roles. | **HIGH: 241 links across 68 files + 2 Rust `//!` comments** (`eak-phases/src/{manufacturing_generation.rs:18,agent.rs:7}`) + `docs/README.md` index. Pure string substitution; do last. |

---

## 9. Markdown files to MERGE / REWRITE

| File | Action | Reason |
|---|---|---|
| `project-plans/README.md` | **Rewrite** as the canonical read-order index (lead with 00/01/02 canonical, list roadmaps, point archive/ + reports/). | Currently indexes only the *superseded* generation — the single highest-leverage fix; the entry point routes readers to deprecated framing. |
| `team/agents/*` ⇄ `.claude/agents/*` | **Merge to one source** (`team/agents/` authored; `.claude/agents/` generated by `scripts/`). | 17 byte-identical duplicates; ends dual maintenance. |
| Roadmap trio `03-roadmap.md` · `07-engineering-backlog.md` · `11-build-roadmap.md` | **Reconcile (cross-reference, don't merge)** | Three roadmap-family docs at different altitudes (Map-arrival / ticket-level / construction-order). Add an explicit "how these relate" note; `07` should be re-expressed per Map (already flagged in `03`). |

---

## 10. Markdown files to ARCHIVE

| File | To | Reason |
|---|---|---|
| `project-plans/00-overview.md` | `project-plans/archive/` | Superseded by `00-product-vision.md` (which explicitly names it subordinate). |
| `project-plans/01-product-spec.md` | `project-plans/archive/` | Superseded generation; MVP-slice detail can be salvaged into `03`/`04` first. |
| `project-plans/02-technical-architecture.md` | `project-plans/archive/` | Superseded by `02-engineering-world-model.md`; also carries the ADR-0017-rejected "synthetic spine" text. |
| `project-plans/08-overnight-execution-2026-07-03.md` | `project-plans/archive/` | A dated one-off execution ledger, not a living plan. |

**Condition (from Safety swarm):** these three plans are still cited as "source of truth" by live
agent definitions (`team/agents/` + `.claude/agents/`) and peer plans (`04`–`07`, README). **Repoint
those citations to `00-product-vision.md` (+ `04-build-roadmap.md`) *before or with* the archive
move**, or the operative agents stay anchored to archived docs.

---

## 11. The canonical repository tree (the 10-year ideal)

Every folder has one clear responsibility. **New** = additive; **renamed/moved** noted inline;
everything else is KEEP-in-place.

```
electronics-agent-kit/
├── README.md                     NEW · root entry point → routes to the canonical vision + the doc-tree map
├── LICENSE                       NEW · MIT OR Apache-2.0 (matches eak/Cargo.toml)
├── CONTRIBUTING.md               NEW · how to work here (green-gate · rings · canonical-first)
├── .gitignore
├── .github/workflows/            NEW · CI green-gate (build+clippy+fmt+test, runs from eak/, asserts the guard test)
│
├── project-plans/                RESEARCH / PRODUCT CANON — the highest authority (vision → roadmaps)
│   ├── 00-product-vision.md          priority 1
│   ├── 01-engineering-philosophy.md  priority 2
│   ├── 02-engineering-world-model.md priority 3 (the Map atlas)
│   ├── 03-roadmap.md                 Map-arrival roadmap
│   ├── 04-build-roadmap.md           construction order (renumbered from 11-)
│   ├── gtm/                          go-to-market (competitive · fundraise · risks · backlog)  [optional regroup]
│   ├── reports/                  NEW · point-in-time reports (audits, alignment, health, compliance) — not living canon
│   ├── archive/                  NEW · superseded generation (00-overview, 01-product-spec, 02-technical-architecture, 08-ledger)
│   └── README.md                     REWRITTEN · canonical read-order index
│
├── docs/                         ARCHITECTURE — design specs (the what/how of the runtime), ring-structured
│   ├── foundation/ core/ compiler/ engineering/ data/ crosscutting/
│   ├── presentation/ integration/ collaboration/ knowledge/ quality/ governance/
│   ├── decisions/                    ADRs — single source of truth (0001–00NN)
│   ├── domain-agents/            RENAMED from agents/ · in-product design agents (drc, erc, bom, placement, …)
│   ├── state-machines/               the 15 phase-FSM specs
│   ├── repository/               NEW · THIS document + future repo-structure canon
│   └── README.md · CONVENTIONS.md · GLOSSARY.md
│
├── engineering-science/          ENGINEERING SCIENCE — the warrant layer (physics/math/practice → thresholds)
│   ├── physics/ mathematics/ electrical/ pcb/ manufacturing/ industry/
│   ├── runtime-mapping/              crosswalk: science concept → real runtime artifact
│   └── README.md                     (compliance/ MOVED OUT → project-plans/reports/)
│
├── eak/                          RUNTIME + COMPILER + BACKEND — the product (Rust workspace = workspace root)
│   ├── Cargo.toml · Cargo.lock · rust-toolchain.toml · README.md (add eak-kicad row; relabel eak-engines)
│   └── crates/
│       ├── eak-units/                Entities — typed physical quantities
│       ├── eak-domain/               Entities — engineering objects   (future: Map-cluster submodules — code task)
│       ├── eak-ports/                Use-case — Event + port traits
│       ├── eak-runtime/              Use-case — the sovereign kernel (commit path · seam · replay · guard test)
│       ├── eak-engines/              Domain — VerificationEngine + rules (future: rule-group submodules — code task)
│       ├── eak-compiler/             Domain — IR projections
│       ├── eak-phases/               Instances — phase FSMs + agents (already Map-organized: the reorg template)
│       ├── eak-store/                Adapters — append-only event log
│       ├── eak-reasoning/            Adapters — fixture + live reasoning
│       ├── eak-kicad/                Adapters — vendor-format scaffold (quarantined; expiry-tagged per vision §10)
│       ├── eak-cli/                  Drivers — composition root
│       ├── eak-solvers/          RESERVED · Adapters — numeric solvers → fidelity-tagged Evidence (V2/Band C)
│       └── eak-memory/           RESERVED · Adapters/knowledge tier — cross-project Engineering Memory (V3/Band D)
│
├── app/                          FRONTEND SHELL (Tauri) — application on the substrate; OUTSIDE eak/ by design
│   ├── src-tauri/                    backend bridge (EventSink → webview); path-deps into eak/
│   ├── ui/                           frontend (webview)
│   └── README.md
│
├── team/                         THE AUTONOMOUS ORG — roles + protocols (governance)
│   ├── agents/                       canonical role charters (SINGLE SOURCE; .claude/agents generated from here)
│   ├── operating-protocols.md · org-structure.md · 00-plan-and-requirements.md · README.md
│
├── examples/                     NEW · curated reference designs / sample boards (NOT in-crate test fixtures)
├── tools/                        NEW · developer tooling
├── scripts/                      NEW · build / CI / migration / agent-sync scripts
│
└── .claude/agents/               TOOLING HARNESS · GENERATED mirror of team/agents/ (stays on disk; synced by scripts/)
```

**The 13 concerns → canonical homes** (the Swarm-7 mapping):

| Concern | Home | Concern | Home |
|---|---|---|---|
| Research | `project-plans/` | Documentation | `docs/` + READMEs |
| Architecture | `docs/` (+ `docs/decisions/`) | Learning | `eak/crates/eak-memory` (future) + `engineering-science/` |
| Runtime | `eak/crates/eak-runtime` | Examples | `examples/` (new) |
| Compiler | `eak/crates/eak-compiler` | Tools | `tools/` (new) |
| Engineering Science | `engineering-science/` | Scripts | `scripts/` (new) |
| Frontend | `app/ui` | Testing | `eak/crates/*/tests` + fixtures (+ future e2e in `app/`) |
| Backend | `eak/` (+ `app/src-tauri` bridge) | | |

---

## 12. Migration plan (safe · incremental · reversible)

**Global rules (from the Safety swarm — conditions of GO):**
- **One phase = one commit**, green-gate + a **broken-link check** after each; never batch phases.
- Use **`git mv`** (never delete+recreate) so history is preserved and reversal is a `git mv` back or `git revert`.
- **Rewrite every inbound cross-link in the *same* commit** as the move that breaks it.
- **`.claude/agents/` stays tracked and on disk** — never gitignore/delete it (tooling loads from it).
- **CI runs from `eak/`** (`working-directory: eak`) so `kernel_has_no_outward_dependencies` actually executes.
- **Freeze `eak/` and `app/` paths** — no crate moves in this migration (separate, higher-risk change).
- Commit the current uncommitted canonical docs first (the 6 untracked `project-plans/` files) so the baseline is clean.

Ordered **low-risk → high-churn**; stop after any phase and reassess.

| Phase | Action | Risk | Reversal | Verify |
|---|---|---|---|---|
| **0 · Additive prep** | Add root `README.md`, `LICENSE`, `CONTRIBUTING.md`, `.github/workflows/green-gate.yml` (runs from `eak/`), empty `examples/` `tools/` `scripts/`. (`docs/repository/` already exists via this report.) | **None** (purely additive) | Delete the additions | CI green from `eak/`; guard test runs |
| **1 · Consolidate reports** | Create `project-plans/reports/`; `git mv` `09`, `10`, `docs/architecture-health-report.md`, and `engineering-science/compliance/*` in; rewrite the ~10 compliance links, the vision/philosophy links, and the `engineering-science/README.md` layer table. | **Low** | `git mv` back | link-check clean |
| **2 · Repoint, then archive superseded** | FIRST repoint "source of truth" refs in `team/agents/`, `.claude/agents/`, and peer plans (`04`–`07`, README) from `00-overview`/`01-product-spec`/`02-technical-architecture` → `00-product-vision` (+ `04-build-roadmap`). THEN create `project-plans/archive/`; `git mv` the 3 superseded docs + `08-ledger` in. | **Low-Med** (semantic: don't strand agents on archived docs) | revert repoint + `git mv` back | no live doc cites `archive/` as source of truth; link-check |
| **3 · Rewrite the index** | Rewrite `project-plans/README.md` to the canonical read-order (00/01/02 + roadmaps; link `archive/` + `reports/`). | **Low** | `git revert` | index matches tree |
| **4 · De-duplicate agents** | Add `scripts/sync-agents.*` (generate `.claude/agents/` from `team/agents/`); add a CI check that fails on a `team/agents ≠ .claude/agents` diff; document `team/agents/` as the single source. Keep `.claude/agents/` on disk. | **Med** (operational) | remove script/check | `.claude/agents/` still loads; CI diff-check passes |
| **5 · Rename `docs/agents/`** (optional, high churn) | `git mv docs/agents docs/domain-agents`; scripted substitution `agents/`→`domain-agents/` over the **68-file / 241-link** set + the 2 `eak-phases` `//!` comments + `docs/README.md` index. | **Med churn / Low severity** | `git mv` back + revert substitution | link-check clean; `cargo build` (confirm comments only) |
| **6 · GTM regroup + renumber** (optional, cosmetic) | Move `04`–`07` → `project-plans/gtm/`; renumber `11-build-roadmap` → `04-build-roadmap`; rewrite inbound links. | **Med churn / lowest value** | `git mv` back | link-check clean |
| **7 · Code doc fixes** (separate track) | In `eak/README.md`: add the `eak-kicad` (Adapters) row; note `eak-cli`/`eak-kicad` are intentionally *not* in `[workspace.dependencies]` (composition root + adapter — do **not** "fix" by adding them); relabel the `eak-engines` row (VerificationEngine, 15 rules). | **Low** (prose in `eak/`) | `git revert` | `cargo build` unaffected |

**Deferred to a separate code task (NOT this migration):** the intra-crate Map-submodule reorg of
`eak-domain`/`eak-engines` (`02 §Part IV`) — behavior-preserving module reorganization using
`eak-phases` as the template; and the eventual creation of `eak-solvers`/`eak-memory` (add each to the
guard's forbidden list on creation; `eak-memory` must be a port-backed adapter, never a kernel dep).

**Overall safety verdict (Swarm 8): GO-WITH-CONDITIONS.** No change moves a crate, edits a Cargo
`path=`/workspace dep, or touches the dependency-rule guard; determinism/replay, the capability seam,
and the §9 ownership model are untouched. Phases 4–5 *strengthen* vision alignment (disambiguating
in-product agents from org roles; keeping the science layer as science, not audit snapshots).

---

## STOP — awaiting approval

This is the canonical design and migration plan. **No files have been moved, renamed, deleted, or
otherwise modified** (this report is the only file created). Per instruction, I now **stop and wait
for your approval.** On approval, I will execute the migration **one phase at a time**, running the
green-gate + link-check and verifying the repository after each step, starting with the zero-risk
additive Phase 0 and pausing for confirmation before any higher-churn phase (5–6).
