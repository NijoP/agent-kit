import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { parseEventLog } from "../events/parse";
import { foldAll } from "../store/fold";
import { projectDocs } from "./projectDocs";
import { tasks } from "./tasks";

const here = dirname(fileURLToPath(import.meta.url));
const fixture = resolve(here, "../../public/fixtures/hero.jsonl");
const vm = foldAll(parseEventLog(readFileSync(fixture, "utf8")));

describe("projectDocs — docs are a faithful projection of the owned model", () => {
  const docs = projectDocs(vm);

  it("projects the full engineering doc set", () => {
    const paths = docs.map((d) => d.path);
    expect(paths).toContain("docs/00-prd.md");
    expect(paths).toContain("docs/01-requirements.md");
    expect(paths).toContain("docs/02-architecture.md");
    expect(paths).toContain("bom/bom.md");
    expect(paths).toContain("verification/checklist.md");
  });

  it("surfaces every owned requirement, net, and part in the docs", () => {
    const all = docs.map((d) => d.md).join("\n");
    for (const r of vm.requirements) expect(all).toContain(r.statement);
    for (const n of vm.nets) expect(all).toContain(n.name);
    for (const p of vm.parts) expect(all).toContain(p.mpn);
  });

  it("every doc backlink resolves to a real committed entity (traceable, never invented)", () => {
    const ids = new Set<string>([
      ...(vm.intent ? [vm.intent.id] : []),
      ...vm.requirements.map((r) => r.id),
      ...vm.blocks.map((b) => b.id),
      ...vm.components.map((c) => c.id),
      ...vm.nets.map((n) => n.id),
      ...vm.parts.map((p) => p.id),
      ...vm.constraints.map((c) => c.id),
      ...vm.assumptions.map((a) => a.id),
      ...vm.tradeoffs.map((t) => t.id),
    ]);
    for (const d of docs) for (const l of d.links) expect(ids.has(l)).toBe(true);
  });

  it("derives engineering tasks from the model", () => {
    const t = tasks(vm);
    // hero is fully routed + released, so no error tasks, but parts get second-source suggestions
    expect(t.length).toBeGreaterThan(0);
    expect(t.every((x) => x.title.length > 0)).toBe(true);
  });
});
