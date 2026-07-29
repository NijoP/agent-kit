import { useState } from "react";
import { useStore } from "../../store/useWorkspaceStore";
import type { ViewModel } from "../../store/fold";
import type { EntityId } from "../../contract/v1";
import { quantity, shortId } from "../util";

type Tab = "props" | "trace";

export function Inspector() {
  const { vm, selected } = useStore();
  const [tab, setTab] = useState<Tab>("props");

  return (
    <div className="dock-section inspector">
      <div className="dock-head" style={{ padding: 0 }}>
        <div className="tabs" style={{ paddingLeft: 10, display: "flex", gap: 2 }}>
          <button className={`tab ${tab === "props" ? "active" : ""}`} onClick={() => setTab("props")}>Properties</button>
          <button className={`tab ${tab === "trace" ? "active" : ""}`} onClick={() => setTab("trace")}>Trace</button>
        </div>
      </div>
      <div className="dock-body">
        {!selected && <Summary vm={vm} />}
        {selected && tab === "props" && <Props vm={vm} id={selected} />}
        {selected && tab === "trace" && <Trace vm={vm} id={selected} />}
      </div>
    </div>
  );
}

function Summary({ vm }: { vm: ViewModel }) {
  const rows: [string, number][] = [
    ["Requirements", vm.requirements.length],
    ["Blocks", vm.blocks.length],
    ["Components", vm.components.length],
    ["Nets", vm.nets.length],
    ["Parts", vm.parts.length],
    ["Placements", vm.placements.length],
    ["Tracks", vm.tracks.length],
    ["Assumptions", vm.assumptions.length],
  ];
  return (
    <div className="props">
      <div style={{ color: "var(--text-muted)", fontSize: 12, padding: "0 0 8px" }}>Select an object to inspect. Design summary:</div>
      {rows.map(([k, v]) => (
        <div className="prop-row" key={k}>
          <span className="key">{k}</span>
          <span className="val">{v}</span>
        </div>
      ))}
    </div>
  );
}

function Props({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const rows = propsFor(vm, id);
  return (
    <div className="props">
      {rows.map(([k, v]) => (
        <div className="prop-row" key={k}>
          <span className="key">{k}</span>
          <span className="val">{v}</span>
        </div>
      ))}
    </div>
  );
}

function Trace({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const nameOf = (e: EntityId): { kind: string; label: string } => {
    if (vm.intent && vm.intent.id === e) return { kind: "Intent", label: vm.intent.statement };
    const r = vm.requirements.find((x) => x.id === e);
    if (r) return { kind: "Requirement", label: r.statement };
    const b = vm.blocks.find((x) => x.id === e);
    if (b) return { kind: "Block", label: b.name };
    const c = vm.components.find((x) => x.id === e);
    if (c) return { kind: "Component", label: `${c.refdes} · ${c.class}` };
    const n = vm.nets.find((x) => x.id === e);
    if (n) return { kind: "Net", label: n.name };
    const p = vm.parts.find((x) => x.id === e);
    if (p) return { kind: "Part", label: p.mpn };
    return { kind: "Entity", label: shortId(e) };
  };
  const chain: EntityId[] = [id];
  const guard = new Set<EntityId>([id]);
  let cursor: EntityId | undefined = id;
  while (cursor) {
    const up = vm.provenance.find((l) => l.from === cursor && !guard.has(l.to));
    if (!up) break;
    chain.push(up.to);
    guard.add(up.to);
    cursor = up.to;
  }
  return (
    <div className="trace-chain">
      {chain.map((e, i) => {
        const { kind, label } = nameOf(e);
        return (
          <div key={e}>
            <div className="trace-node">
              <span className="kind">{kind}</span>
              <span style={{ color: "var(--text-primary)" }}>{label.slice(0, 38)}</span>
            </div>
            {i < chain.length - 1 && <div className="trace-arrow">↑</div>}
          </div>
        );
      })}
      {chain.length === 1 && <div className="empty" style={{ height: 70 }}>No upstream links recorded yet.</div>}
    </div>
  );
}

function propsFor(vm: ViewModel, id: EntityId): [string, string][] {
  const c = vm.components.find((x) => x.id === id);
  if (c) return [["refdes", c.refdes], ["class", c.class], ["value", quantity(c.value)], ["origin", c.origin], ["id", shortId(c.id)]];
  const r = vm.requirements.find((x) => x.id === id);
  if (r) return [["category", r.category], ["priority", r.priority], ["status", r.status], ["criterion", r.acceptance_criterion.slice(0, 34)], ["id", shortId(r.id)]];
  const n = vm.nets.find((x) => x.id === id);
  if (n) return [["name", n.name], ["class", n.class], ["members", String(n.members.length)], ["current", quantity(n.current)], ["id", shortId(n.id)]];
  const p = vm.parts.find((x) => x.id === id);
  if (p) return [["mpn", p.mpn], ["manufacturer", p.manufacturer], ["lifecycle", p.lifecycle], ["id", shortId(p.id)]];
  const b = vm.blocks.find((x) => x.id === id);
  if (b) return [["name", b.name], ["function", b.function.slice(0, 34)], ["requirements", String(b.requirements.length)], ["id", shortId(b.id)]];
  return [["id", shortId(id)]];
}
