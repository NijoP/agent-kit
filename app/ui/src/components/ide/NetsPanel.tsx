import { useStore } from "../../store/useWorkspaceStore";

export function NetsPanel() {
  const { vm, select, selected } = useStore();
  return (
    <section className="ide-sidebar" aria-label="Nets">
      <div className="sidebar-head">
        Nets<span style={{ marginLeft: "auto", fontFamily: "var(--font-mono)", color: "var(--text-muted)" }}>{vm.nets.length}</span>
      </div>
      <div className="sidebar-body">
        {vm.nets.length === 0 && <div className="empty" style={{ height: 120 }}>Nets appear after synthesis.</div>}
        <div className="tree">
          {vm.nets.map((n) => (
            <button
              key={n.id}
              className={`trow depth0 ${selected === n.id ? "sel" : ""}`}
              onClick={() => select(n.id)}
            >
              <span
                className="layer-swatch"
                style={{ background: n.class === "Power" ? "var(--ecad-copper-top)" : n.class === "Ground" ? "var(--ecad-copper-bottom)" : "var(--ecad-ratline)" }}
              />
              <span className="lname">{n.name}</span>
              <span className="tcount">{n.class} · {n.members.length}</span>
            </button>
          ))}
        </div>
      </div>
    </section>
  );
}
