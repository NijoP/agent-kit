import { useStore } from "../../store/useWorkspaceStore";
import { shortId } from "../util";

export function PartsPanel() {
  const { vm, select, selected } = useStore();
  return (
    <section className="ide-sidebar" aria-label="Parts">
      <div className="sidebar-head">
        Parts / BOM
        <span style={{ marginLeft: "auto", fontFamily: "var(--font-mono)", color: "var(--text-muted)" }}>{vm.parts.length}</span>
      </div>
      <div className="sidebar-body">
        {vm.parts.length === 0 && <div className="empty" style={{ height: 120 }}>Parts appear after BOM planning.</div>}
        <div className="tree">
          {vm.parts.map((p) => (
            <button
              key={p.id}
              className={`trow depth0 ${selected === p.id ? "sel" : ""}`}
              onClick={() => select(p.id)}
            >
              <span className="refdes">{p.mpn}</span>
              <span className="tcount">{p.manufacturer} · {p.lifecycle} · {shortId(p.id)}</span>
            </button>
          ))}
        </div>
      </div>
    </section>
  );
}
