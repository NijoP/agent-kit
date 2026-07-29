import type { ViewModel } from "../store/fold";
import type { Component, EntityId } from "../contract/v1";
import { quantity, shortId } from "../components/util";
import { gateOf } from "../store/fold";

/**
 * projectDocs — the heart of "docs = a living projection of the owned model".
 *
 * A PURE function `ViewModel → DocNode[]`: it renders the kernel's committed facts (intent,
 * requirements, blocks, nets, parts, verification) into the engineering document set the engineer and
 * AI collaborate over. The docs are GENERATED from owned truth, so they can never drift from it — the
 * kernel remains the source of truth (vision §1/§6). Inline `[label](eak:<id>)` links point back at the
 * exact owned entity each statement projects, so a doc is traceable, not prose.
 *
 * Honesty (Principle 7): every doc is projected from committed facts only. Where the model holds no
 * rationale (e.g. no recorded tradeoffs), the doc says so rather than inventing a "why".
 */

export interface DocNode {
  /** Path within the virtual project, e.g. "docs/01-requirements.md". Encodes the folder. */
  path: string;
  title: string;
  md: string;
  /** Owned entity ids this doc projects — the backlinks that keep it in sync. */
  links: EntityId[];
}

const link = (label: string, id: EntityId) => `[${label}](eak:${id})`;

function componentsForRequirement(vm: ViewModel, reqId: EntityId): Component[] {
  const blockIds = new Set(vm.blocks.filter((b) => b.requirements.includes(reqId)).map((b) => b.id));
  return vm.components.filter((c) => blockIds.has(c.from_block));
}

function frontMatter(title: string): string {
  return `> **Projection of owned truth — always in sync with the kernel.** Edit through the agent; the runtime validates and re-projects.\n\n# ${title}\n\n`;
}

export function projectDocs(vm: ViewModel): DocNode[] {
  const docs: DocNode[] = [];
  const gate = gateOf(vm);

  // ── 00 · PRD ────────────────────────────────────────────────────────────────────────────────
  {
    const links: EntityId[] = [];
    let md = frontMatter("Product Requirements — " + (vm.intent?.statement ?? "untitled"));
    md += `## Intent (captured)\n\n`;
    if (vm.intent) {
      links.push(vm.intent.id);
      md += `> ${vm.intent.statement}\n\n_Source: ${vm.intent.source}. This is the root of every provenance chain._\n\n`;
    } else md += `_No intent captured yet._\n\n`;
    md += `## Goals\n\nThe design must satisfy the following, each an owned, verified requirement:\n\n`;
    for (const r of vm.requirements) {
      links.push(r.id);
      md += `- **${r.priority}** — ${r.statement} ${link("→ " + shortId(r.id), r.id)}\n`;
    }
    md += `\n## Status\n\n- Manufacturing gate: **${gate.released ? "RELEASED ✅" : gate.reason ?? "in progress"}**\n`;
    md += `- ${vm.components.length} components · ${vm.nets.length} nets · ${vm.parts.length} sourced parts · ${vm.tracks.length} routed tracks\n`;
    docs.push({ path: "docs/00-prd.md", title: "PRD", md, links });
  }

  // ── 01 · Requirements ───────────────────────────────────────────────────────────────────────
  {
    const links: EntityId[] = [];
    let md = frontMatter("Requirements");
    md += `Every requirement is a typed, owned object rooted in the intent and linked to the parts that realize it.\n\n`;
    md += `| ID | Requirement | Category | Priority | Realized by |\n|---|---|---|---|---|\n`;
    for (const r of vm.requirements) {
      links.push(r.id);
      const comps = componentsForRequirement(vm, r.id);
      comps.forEach((c) => links.push(c.id));
      const by = comps.length ? comps.map((c) => link(c.refdes, c.id)).join(", ") : "—";
      md += `| ${link(shortId(r.id), r.id)} | ${r.statement} | ${r.category} | ${r.priority} | ${by} |\n`;
    }
    md += `\n## Acceptance criteria\n\n`;
    for (const r of vm.requirements) {
      md += `- **${shortId(r.id)}** — ${r.acceptance_criterion || "_none stated_"}\n`;
    }
    docs.push({ path: "docs/01-requirements.md", title: "Requirements", md, links });
  }

  // ── 02 · Architecture ───────────────────────────────────────────────────────────────────────
  {
    const links: EntityId[] = [];
    let md = frontMatter("Architecture");
    md += `## Functional blocks\n\n`;
    for (const b of vm.blocks) {
      links.push(b.id);
      const comps = vm.components.filter((c) => c.from_block === b.id);
      comps.forEach((c) => links.push(c.id));
      md += `### ${b.name}\n\n${b.function}\n\n`;
      md += `- Realizes: ${b.requirements.map((r) => shortId(r)).join(", ") || "—"}\n`;
      md += `- Components: ${comps.map((c) => link(`${c.refdes} (${c.class})`, c.id)).join(", ") || "—"}\n\n`;
    }
    md += `## Nets\n\n| Net | Class | Pins |\n|---|---|---|\n`;
    for (const n of vm.nets) {
      links.push(n.id);
      md += `| ${link(n.name, n.id)} | ${n.class} | ${n.members.length} |\n`;
    }
    docs.push({ path: "docs/02-architecture.md", title: "Architecture", md, links });
  }

  // ── 03 · Constraints ────────────────────────────────────────────────────────────────────────
  {
    const links: EntityId[] = [];
    let md = frontMatter("Constraints");
    md += `Typed, dimensioned bounds derived from the requirements — enforced by construction (P6).\n\n`;
    if (vm.constraints.length === 0) md += `_No constraints extracted yet._\n`;
    else {
      md += `| Constraint | Bound | Derived from |\n|---|---|---|\n`;
      for (const c of vm.constraints) {
        links.push(c.id);
        md += `| ${c.statement} | ${c.kind} ${quantity(c.bound)} | ${link(shortId(c.subject_requirement), c.subject_requirement)} |\n`;
      }
    }
    if (vm.fidelityTags.length) {
      md += `\n## Fidelity of derived numbers\n\n`;
      for (const t of vm.fidelityTags) md += `- ${t.fidelity.concern}: **${t.fidelity.method}** (confidence ${t.fidelity.confidence})\n`;
    }
    docs.push({ path: "docs/03-constraints.md", title: "Constraints", md, links });
  }

  // ── bom/bom.md ──────────────────────────────────────────────────────────────────────────────
  {
    const links: EntityId[] = [];
    let md = frontMatter("Bill of Materials");
    md += `| MPN | Manufacturer | Lifecycle | Realizes |\n|---|---|---|---|\n`;
    for (const p of vm.parts) {
      links.push(p.id);
      const line = vm.bomLines.find((l) => l.part === p.id);
      const comps = (line?.components ?? []).map((cid) => vm.components.find((c) => c.id === cid)).filter(Boolean);
      comps.forEach((c) => c && links.push(c.id));
      const by = comps.map((c) => c && link(c.refdes, c.id)).filter(Boolean).join(", ") || "—";
      md += `| ${link(p.mpn, p.id)} | ${p.manufacturer} | ${p.lifecycle} | ${by} |\n`;
    }
    if (vm.parts.length === 0) md += `_No parts sourced yet._\n`;
    docs.push({ path: "bom/bom.md", title: "BOM", md, links });
  }

  // ── decisions/ ──────────────────────────────────────────────────────────────────────────────
  {
    const links: EntityId[] = [];
    let md = frontMatter("Design Decisions");
    if (vm.tradeoffs.length === 0) {
      md += `_No tradeoffs have been recorded for this design yet. When the agent weighs alternatives, each choice — and the options it rejected and why — is captured here as owned truth (never fabricated)._\n`;
    } else {
      for (const t of vm.tradeoffs) {
        links.push(t.id);
        const chosen = t.alternatives[t.chosen];
        md += `## ${t.question}\n\n**Chosen: ${chosen?.label}** — ${t.rationale} _(decided by ${t.decided_by})_\n\n`;
        md += `| Alternative | Rejected | Notes |\n|---|---|---|\n`;
        for (const a of t.alternatives) md += `| ${a.label} | ${a.rejected ? "yes" : "**chosen**"} | ${a.description} |\n`;
        md += `\n`;
      }
    }
    docs.push({ path: "decisions/decisions.md", title: "Decisions", md, links });
  }

  // ── verification/checklist.md ───────────────────────────────────────────────────────────────
  {
    const links: EntityId[] = [];
    let md = frontMatter("Verification Checklist");
    const donePhases = vm.phases.filter((p) => p.status === "done");
    md += `## Gate\n\n**${gate.released ? "✅ RELEASED" : "⛔ " + (gate.reason ?? "in progress")}**\n\n`;
    md += `## Checks run\n\n`;
    for (const p of vm.phases) {
      const box = p.status === "done" ? "x" : " ";
      md += `- [${box}] ${p.name}\n`;
    }
    if (donePhases.length === 0) md += `_No verification phases have run yet._\n`;
    md += `\n## Open findings\n\n`;
    const open = vm.violations.filter((v) => v.status === "Open");
    if (open.length === 0) md += `- [x] No blocking findings. \n`;
    else for (const v of open) { links.push(...v.subjects); md += `- [ ] **${v.rule}** — ${v.message}\n`; }
    md += `\n## Assumptions\n\n`;
    if (vm.assumptions.length === 0) md += `_No assumptions declared._\n`;
    else for (const a of vm.assumptions) { links.push(a.id); md += `- ${a.status === "Open" ? "[ ]" : "[x]"} ${a.criticality === "Critical" ? "**CRITICAL** " : ""}${a.statement}\n`; }
    docs.push({ path: "verification/checklist.md", title: "Verification", md, links });
  }

  return docs;
}

/** The doc tree, grouped by folder (for the Explorer). */
export interface DocFolder {
  name: string;
  docs: DocNode[];
}
export function docTree(docs: DocNode[]): DocFolder[] {
  const byFolder = new Map<string, DocNode[]>();
  for (const d of docs) {
    const folder = d.path.includes("/") ? d.path.slice(0, d.path.lastIndexOf("/")) : "";
    if (!byFolder.has(folder)) byFolder.set(folder, []);
    byFolder.get(folder)!.push(d);
  }
  return [...byFolder.entries()].map(([name, ds]) => ({ name, docs: ds }));
}

export function fileName(path: string): string {
  return path.includes("/") ? path.slice(path.lastIndexOf("/") + 1) : path;
}
