import { useState } from "react";
import { useStore } from "../../store/useWorkspaceStore";
import type { ViewModel } from "../../store/fold";
import type { EntityId } from "../../contract/v1";
import { quantity, shortId } from "../util";
import { Pill } from "../Badge";
import {
  netColor,
  pinsForComponent,
  netsForComponent,
  pinOfNet,
  netState,
  componentPads,
  bomLineForPart,
  componentsForPart,
  partOfComponent,
  netClassColor,
} from "../../store/selectors";
import { Waypoints, Cpu, Package, Boxes, Layers } from "lucide-react";

type Tab = "props" | "trace";

export function Inspector() {
  const { vm, selected } = useStore();
  const [tab, setTab] = useState<Tab>("props");

  return (
    <div className="dock-section inspector">
      <div className="dock-head" style={{ padding: 0 }}>
        <div className="tabs" style={{ paddingLeft: 10, display: "flex", gap: 2 }}>
          <button className={`tab ${tab === "props" ? "active" : ""}`} onClick={() => setTab("props")}>Inspector</button>
          <button className={`tab ${tab === "trace" ? "active" : ""}`} onClick={() => setTab("trace")}>Trace</button>
        </div>
      </div>
      <div className="dock-body">
        {!selected && <Summary vm={vm} />}
        {selected && tab === "props" && <ObjectInspector vm={vm} id={selected} />}
        {selected && tab === "trace" && <Trace vm={vm} id={selected} />}
      </div>
    </div>
  );
}

function Row({ k, v, color }: { k: string; v: React.ReactNode; color?: string }) {
  return (
    <div className="prop-row">
      <span className="key">{k}</span>
      <span className="val" style={color ? { color } : undefined}>{v}</span>
    </div>
  );
}

function Section({ icon, title, children }: { icon?: React.ReactNode; title: string; children: React.ReactNode }) {
  return (
    <div className="ins-section">
      <div className="ins-head">{icon}{title}</div>
      <div className="ins-props">{children}</div>
    </div>
  );
}

function FidelityFor({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const tags = vm.fidelityTags.filter((t) => t.target === id);
  if (tags.length === 0) return null;
  return (
    <Section title="Fidelity">
      {tags.map((t, i) => (
        <div key={i} style={{ fontSize: 11.5, color: "var(--text-secondary)", padding: "2px 0" }}>
          <span style={{ color: "var(--text-muted)" }}>{t.fidelity.concern}: </span>
          <span style={{ color: "var(--warn)" }}>{t.fidelity.method}</span>
          <span style={{ color: "var(--text-muted)" }}> · {t.fidelity.confidence}%</span>
        </div>
      ))}
    </Section>
  );
}

function NetSection({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const net = vm.nets.find((n) => n.id === id);
  if (!net) return null;
  const color = netColor(net);
  const state = netState(vm, net.id);
  const padCount = componentPads(vm, net.id).length;
  return (
    <>
      <Section icon={<Waypoints size={12} strokeWidth={1.75} />} title="Net">
        <Row k="name" v={net.name} color={color} />
        <Row k="class" v={net.class} color={color} />
        <Row k="origin" v={net.origin} />
        <Row k="members" v={String(net.members.length)} />
        <Row k="routing" v={state} />
        <Row k="pads (footprints)" v={String(padCount)} />
        {net.current && <Row k="current" v={quantity(net.current)} />}
        {net.impedance_target && <Row k="impedance target" v={quantity(net.impedance_target)} />}
      </Section>
      <Section title="Member pins">
        {net.members.map((pinId) => {
          const pin = vm.pins.find((p) => p.id === pinId);
          const comp = vm.components.find((c) => c.id === pin?.component);
          return (
            <button key={pinId} className="ins-link" onClick={() => useStore.getState().select(pinId)}>
              {comp?.refdes}.{pin?.designation} <span style={{ color: "var(--text-muted)" }}>· {pin?.electrical_type}</span>
            </button>
          );
        })}
        {net.members.length === 0 && <span className="ins-note">No pins committed on this net.</span>}
      </Section>
    </>
  );
}

function ComponentSection({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const c = vm.components.find((x) => x.id === id);
  if (!c) return null;
  const pins = pinsForComponent(vm, c.id);
  const nets = netsForComponent(vm, c.id);
  const part = partOfComponent(vm, c.id);
  const block = vm.blocks.find((b) => b.id === c.from_block);
  const pl = vm.placements.find((p) => p.component === c.id);
  return (
    <>
      <Section icon={<Cpu size={12} strokeWidth={1.75} />} title="Component">
        <div className="ins-title">{c.refdes}</div>
        <div className="ins-sub">{c.class}{c.value ? ` · ${quantity(c.value)}` : ""} · {c.origin}</div>
        <Row k="block" v={block?.name ?? "—"} />
        {pl && (
          <>
            <Row k="placed" v={`${pl.x.magnitude}, ${pl.y.magnitude} mm`} />
            <Row k="side" v={pl.side} />
          </>
        )}
        {part && <Row k="part" v={part.mpn} />}
      </Section>
      <Section title="Pins">
        {pins.map((p) => {
          const net = pinOfNet(vm, p.id);
          return (
            <button key={p.id} className="ins-link" onClick={() => useStore.getState().select(p.id)}>
              {p.designation} <span style={{ color: "var(--text-muted)" }}>· {p.electrical_type}</span>
              {net && <span style={{ color: netColor(net) }}> · {net.name}</span>}
            </button>
          );
        })}
      </Section>
      {nets.length > 0 && (
        <Section title="Nets">
          {nets.map((n) => (
            <button key={n.id} className="ins-link" onClick={() => useStore.getState().select(n.id)}>
              {n.name} <span style={{ color: netColor(n) }}>· {n.class}</span>
            </button>
          ))}
        </Section>
      )}
    </>
  );
}

function PartSection({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const p = vm.parts.find((x) => x.id === id);
  if (!p) return null;
  const line = bomLineForPart(vm, p.id);
  const comps = componentsForPart(vm, p.id);
  return (
    <>
      <Section icon={<Package size={12} strokeWidth={1.75} />} title="Part">
        <div className="ins-title">{p.mpn}</div>
        <div className="ins-sub">{p.manufacturer}</div>
        <Row k="lifecycle" v={<Pill tone={p.lifecycle === "Active" ? "pass" : p.lifecycle === "Nrnd" ? "warn" : "error"}>{p.lifecycle}</Pill>} />
        <Row k="quantity" v={String(line?.quantity ?? 0)} />
        {p.datasheet && <Row k="datasheet" v={p.datasheet.slice(0, 30)} />}
      </Section>
      <Section title="Realizes">
        {comps.length === 0 && <span className="ins-note">Not yet bound to a placed component.</span>}
        {comps.map((c) => (
          <button key={c.id} className="ins-link" onClick={() => useStore.getState().select(c.id)}>
            {c.refdes} <span style={{ color: "var(--text-muted)" }}>· {c.class}</span>
          </button>
        ))}
      </Section>
    </>
  );
}

function PinSection({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const pin = vm.pins.find((p) => p.id === id);
  if (!pin) return null;
  const comp = vm.components.find((c) => c.id === pin.component);
  const net = pinOfNet(vm, id);
  return (
    <Section title="Pin">
      <Row k="designation" v={pin.designation} />
      <Row k="type" v={pin.electrical_type} />
      <Row k="component" v={comp ? `${comp.refdes} · ${comp.class}` : "—"} />
      <Row k="net" v={net ? net.name : "—"} color={net ? netColor(net) : undefined} />
    </Section>
  );
}

function TrackSection({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const t = vm.tracks.find((x) => x.id === id);
  if (!t) return null;
  const net = vm.nets.find((n) => n.id === t.net);
  return (
    <Section icon={<Layers size={12} strokeWidth={1.75} />} title="Track">
      <Row k="net" v={net?.name ?? "—"} color={net ? netColor(net) : undefined} />
      <Row k="layer" v={t.layer} />
      <Row k="width" v={quantity(t.width)} />
      <Row k="from" v={`${t.x1.magnitude}, ${t.y1.magnitude} mm`} />
      <Row k="to" v={`${t.x2.magnitude}, ${t.y2.magnitude} mm`} />
    </Section>
  );
}

function ViolationSection({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const v = vm.violations.find((x) => x.id === id);
  if (!v) return null;
  const explanation = vm.explanations[v.id];
  const tone = v.severity === "Error" ? "error" : v.severity === "Warning" ? "warn" : "accent";
  return (
    <>
      <Section title="Violation">
        <div className="ins-title" style={{ color: v.severity === "Error" ? "var(--error)" : v.severity === "Warning" ? "var(--warn)" : "var(--text-primary)" }}>{v.rule}</div>
        <div className="ins-sub">{v.message}</div>
        <Row k="severity" v={<Pill tone={tone}>{v.severity}</Pill>} />
        <Row k="status" v={v.status} />
      </Section>
      <Section title="Subjects">
        {v.subjects.map((s) => {
          const c = vm.components.find((x) => x.id === s);
          const n = vm.nets.find((x) => x.id === s);
          const label = c ? `${c.refdes} · ${c.class}` : n ? n.name : shortId(s);
          return (
            <button key={s} className="ins-link" onClick={() => useStore.getState().select(s)}>
              → {label}
            </button>
          );
        })}
      </Section>
      {explanation && (
        <Section title="Explanation">
          <div style={{ fontSize: 12.5, lineHeight: 17, color: "var(--text-secondary)" }}>{explanation.explanation}</div>
          {explanation.suggestedFix && (
            <div className="ins-note fix" style={{ color: "var(--pass)" }}>
              <strong>Fix:</strong> {explanation.suggestedFix}
            </div>
          )}
        </Section>
      )}
    </>
  );
}

function BoardSection({ vm, id }: { vm: ViewModel; id: EntityId }) {
  const b = vm.board;
  if (!b || b.id !== id) return null;
  return (
    <Section icon={<Boxes size={12} strokeWidth={1.75} />} title="Board">
      <Row k="size" v={`${b.width.magnitude} × ${b.height.magnitude} mm`} />
      <Row k="stack layers" v={String(b.stack.layers.length)} />
      {b.stack.layers.map((l, i) => (
        <Row key={i} k={`layer ${i + 1}`} v={`${l.role} · ${quantity(l.copper_thickness)} Cu · Er ${l.dielectric_er}`} />
      ))}
    </Section>
  );
}

function ObjectInspector({ vm, id }: { vm: ViewModel; id: EntityId }) {
  return (
    <>
      <ComponentSection vm={vm} id={id} />
      <NetSection vm={vm} id={id} />
      <PartSection vm={vm} id={id} />
      <PinSection vm={vm} id={id} />
      <TrackSection vm={vm} id={id} />
      <ViolationSection vm={vm} id={id} />
      <BoardSection vm={vm} id={id} />
      <FidelityFor vm={vm} id={id} />
    </>
  );
}

function Summary({ vm }: { vm: ViewModel }) {
  const classes = new Map<"Power" | "Ground" | "Signal", number>([["Power", 0], ["Ground", 0], ["Signal", 0]]);
  for (const n of vm.nets) classes.set(n.class, (classes.get(n.class) ?? 0) + 1);
  const rows: [string, number][] = [
    ["Components", vm.components.length],
    ["Nets", vm.nets.length],
    ["Pins", vm.pins.length],
    ["Parts", vm.parts.length],
    ["Placements", vm.placements.length],
    ["Tracks", vm.tracks.length],
    ["Requirements", vm.requirements.length],
    ["Blocks", vm.blocks.length],
    ["Assumptions", vm.assumptions.length],
    ["Open findings", vm.violations.filter((v) => v.status === "Open").length],
  ];
  return (
    <div>
      <div className="ins-section">
        <div className="ins-head"><Boxes size={12} strokeWidth={1.75} />Design summary</div>
        <div className="ins-props">
          {rows.map(([k, v]) => <Row key={k} k={k} v={v} />)}
        </div>
        <div className="ins-note">Select any object — a component, net, pin, track, part or violation — to inspect it here.</div>
      </div>
      {vm.nets.length > 0 && (
        <div className="ins-section">
          <div className="ins-head"><Waypoints size={12} strokeWidth={1.75} />Net classes</div>
          {[...classes.entries()].filter(([, c]) => c > 0).map(([cls, count]) => (
            <Row key={cls} k={`${cls} nets`} v={count} color={netClassColor(cls)} />
          ))}
        </div>
      )}
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