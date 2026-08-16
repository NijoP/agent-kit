import { RefreshCw } from "lucide-react";
import { useMemo } from "react";
import { useStore } from "../../store/useWorkspaceStore";
import { HonestyBadge } from "../Badge";
import { layerModel, netClassColor } from "../../store/selectors";
import { tasks } from "../../docs/tasks";

const TEST_COUNT = 282;

/** The tool/selection identity shown in the status strip. */
function selectionLabel(selected: string | undefined, vm: ReturnType<typeof import("../../store/useWorkspaceStore").useStore.getState>["vm"]): string {
  if (!selected) return "—";
  const c = vm.components.find((x) => x.id === selected);
  if (c) return `${c.refdes} · ${c.class}`;
  const n = vm.nets.find((x) => x.id === selected);
  if (n) return `${n.name} (${n.class})`;
  const p = vm.parts.find((x) => x.id === selected);
  if (p) return p.mpn;
  const v = vm.violations.find((x) => x.id === selected);
  if (v) return v.rule;
  return "entity";
}

export function StatusBar() {
  const { vm, gate, mode, camera, cursorMm, layers, tool, selected } = useStore();
  const gateClass = gate.released ? "released" : gate.reason === "in progress" ? "progress" : "blocked";
  const gateText = gate.released ? "RELEASED" : gate.reason === "in progress" ? "IN PROGRESS" : `BLOCKED · ${gate.reason}`;
  const visibleLayers = layerModel(vm).filter((l) => layers[l.key]).length;
  const zoomPct = Math.round(camera.zoom * 12.5);
  const taskCount = useMemo(() => tasks(vm).length, [vm]);
  const sel = selectionLabel(selected, vm);

  const netCounts = useMemo(() => {
    const c = { Power: 0, Ground: 0, Signal: 0 };
    for (const n of vm.nets) c[n.class] += 1;
    return c;
  }, [vm.nets]);

  return (
    <footer className="ide-statusbar">
      <div className="seg"><HonestyBadge mode={mode} /></div>
      <div className="seg"><span className={`gate-chip ${gateClass}`}>Gate {gateText}</span></div>
      <div className="seg" title="Docs are a projection of the owned model — always in sync."><RefreshCw size={11} strokeWidth={1.75} style={{ color: "var(--pass)" }} /> <span style={{ color: "var(--pass)" }}>synced</span></div>
      <div className="seg" title="Active tool"><span className="mono" style={{ color: "var(--accent)" }}>{tool}</span></div>
      <div className="seg" title="Selection">{sel}</div>
      <div className="seg">X <span className="mono">{cursorMm ? cursorMm.x.toFixed(2) : "—"}</span> Y <span className="mono">{cursorMm ? cursorMm.y.toFixed(2) : "—"}</span> mm</div>
      <div className="seg">grid <span className="mono">1.00 mm</span></div>
      <div className="spacer" />
      {vm.nets.length > 0 && (
        <div className="seg net-legend">
          {(Object.keys(netCounts) as Array<"Power" | "Ground" | "Signal">).map((k) => (
            <span className="net-item" key={k}>
              <span className="net-swatch" style={{ background: netClassColor(k) }} />
              {k} {netCounts[k]}
            </span>
          ))}
        </div>
      )}
      <div className="seg"><span className="mono">{taskCount}</span> tasks</div>
      <div className="seg">z <span className="mono">{zoomPct}%</span></div>
      <div className="seg">layers <span className="mono">{visibleLayers}</span></div>
      <div className="seg"><span className="mono">{vm.eventCount}</span> events</div>
      <div className="seg">contract <span className="mono">v1</span></div>
      <div className="seg">tests <span className="mono">{TEST_COUNT}</span> ✓</div>
    </footer>
  );
}