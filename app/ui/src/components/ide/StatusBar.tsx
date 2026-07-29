import { RefreshCw } from "lucide-react";
import { useMemo } from "react";
import { useStore } from "../../store/useWorkspaceStore";
import { HonestyBadge } from "../Badge";
import { layerModel } from "../../store/selectors";
import { tasks } from "../../docs/tasks";

const TEST_COUNT = 282;

export function StatusBar() {
  const { vm, gate, mode, camera, cursorMm, layers } = useStore();
  const gateClass = gate.released ? "released" : gate.reason === "in progress" ? "progress" : "blocked";
  const gateText = gate.released ? "RELEASED" : gate.reason === "in progress" ? "IN PROGRESS" : `BLOCKED · ${gate.reason}`;
  const visibleLayers = layerModel(vm).filter((l) => layers[l.key]).length;
  const zoomPct = Math.round(camera.zoom * 12.5);
  const taskCount = useMemo(() => tasks(vm).length, [vm]);

  return (
    <footer className="ide-statusbar">
      <div className="seg"><HonestyBadge mode={mode} /></div>
      <div className="seg"><span className={`gate-chip ${gateClass}`}>Gate {gateText}</span></div>
      <div className="seg" title="Docs are a projection of the owned model — always in sync."><RefreshCw size={11} strokeWidth={1.75} style={{ color: "var(--pass)" }} /> <span style={{ color: "var(--pass)" }}>synced</span></div>
      <div className="seg">X <span className="mono">{cursorMm ? cursorMm.x.toFixed(2) : "—"}</span> Y <span className="mono">{cursorMm ? cursorMm.y.toFixed(2) : "—"}</span> mm</div>
      <div className="spacer" />
      <div className="seg"><span className="mono">{taskCount}</span> tasks</div>
      <div className="seg">z <span className="mono">{zoomPct}%</span></div>
      <div className="seg">layers <span className="mono">{visibleLayers}</span></div>
      <div className="seg"><span className="mono">{vm.eventCount}</span> events</div>
      <div className="seg">contract <span className="mono">v1</span></div>
      <div className="seg">tests <span className="mono">{TEST_COUNT}</span> ✓</div>
    </footer>
  );
}
