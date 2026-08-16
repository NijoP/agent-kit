import { FileCode2 } from "lucide-react";
import { useStore } from "../../store/useWorkspaceStore";
import { schematicGraph, highlightNetOf } from "../../store/selectors";
import { quantity } from "../util";

const STICK = 16;

/**
 * Schematic-sheet projection — node boxes for each owned component, pins on the left edge, and one
 * colored, labelled rail per net. Pure geometry derived from committed pins/nets; the sheet can
 * never show a connection the kernel didn't commit. Cross-probing is wired: click a node, a pin, or
 * a rail and the same selection lights up the PCB.
 */
export function SchematicView() {
  const { vm, select, selected } = useStore();
  if (vm.components.length === 0)
    return <div className="pcb-empty">Components appear here as synthesis commits them.</div>;

  const g = schematicGraph(vm);
  const hotNet = highlightNetOf(vm, selected);

  const isHot = (netId: string) => (selected === netId || hotNet === netId) && selected !== undefined;

  return (
    <div className="schematic">
      <svg className="schematic-svg" viewBox={`0 0 ${g.w} ${g.h}`} preserveAspectRatio="xMidYMin meet">
        {/* rails (under nodes' stubs so junction dots read clearly) */}
        {g.rails.map((r) => {
          const hot = isHot(r.net.id);
          return (
            <g key={r.net.id} onClick={(e) => { e.stopPropagation(); select(r.net.id); }} style={{ cursor: "pointer" }}>
              <line
                className={`schematic-rail ${r.net.class === "Power" ? "power" : ""} ${r.net.impedance_target ? "hot" : ""} ${hot ? "sel" : ""}`}
                x1={6}
                y1={r.y}
                x2={g.w - 6}
                y2={r.y}
                stroke={r.color}
              />
              <text className="schematic-netlabel" x={10} y={r.y - 6} fill={r.color}>
                {r.net.name}
                {r.net.current ? ` · ${quantity(r.net.current)}` : ""}
              </text>
            </g>
          );
        })}

        {/* nodes */}
        {g.nodes.map((n) => {
          const sel = selected === n.component.id;
          const cx = n.x + n.w / 2;
          return (
            <g key={n.component.id} onClick={(e) => { e.stopPropagation(); select(n.component.id); }} style={{ cursor: "pointer" }}>
              <rect className={`schematic-node ${sel ? "sel" : ""}`} x={n.x} y={n.y} width={n.w} height={n.h} />
              <text className="schematic-refdes" x={cx} y={n.y + 30} textAnchor="middle">{n.component.refdes}</text>
              <text className="schematic-klass" x={cx} y={n.y + 46} textAnchor="middle">{n.component.class}</text>
              {n.component.value && (
                <text className="schematic-klass" x={cx} y={n.y + 60} textAnchor="middle">{quantity(n.component.value)}</text>
              )}

              {n.pins.map((p) => {
                const pinX = n.x;
                const pinY = n.y + p.y;
                const stubX = pinX - STICK;
                const rail = g.rails.find((r) => r.net.members.includes(p.pin.id));
                const railY = rail?.y;
                const pinSelected = selected === p.pin.id;
                return (
                  <g key={p.pin.id} onClick={(e) => { e.stopPropagation(); select(p.pin.id); }} style={{ cursor: "pointer" }}>
                    <line className="schematic-stub" x1={pinX} y1={pinY} x2={stubX} y2={pinY} />
                    {rail && <line className="schematic-stub" x1={stubX} y1={pinY} x2={stubX} y2={railY} />}
                    <circle className={`schematic-pin ${pinSelected ? "schematic-junction" : ""}`} cx={stubX} cy={pinY} r={3} />
                    {rail && <circle className="schematic-junction" cx={stubX} cy={railY} r={3.4} />}
                    <text className="schematic-pin-label" x={pinX + 6} y={pinY + 3}>
                      {p.designation}
                    </text>
                    <text className="schematic-pin-type" x={stubX - 5} y={pinY - 4} textAnchor="end">
                      {p.type}
                    </text>
                  </g>
                );
              })}
            </g>
          );
        })}
      </svg>
      <div className="schematic-note">
        <FileCode2 size={12} strokeWidth={1.5} />
        schematic projection — derived from committed nets ({vm.nets.length} rails · {vm.components.length} nodes)
      </div>
    </div>
  );
}