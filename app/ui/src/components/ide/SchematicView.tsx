import { useStore } from "../../store/useWorkspaceStore";

/**
 * Minimal schematic projection for v1: the functional blocks and the nets that join them, as a
 * readable card graph. A full schematic-sheet renderer is ◐ planned; this honestly shows the owned
 * architecture without faking symbol geometry.
 */
export function SchematicView() {
  const { vm, select, selected } = useStore();
  if (vm.blocks.length === 0)
    return <div className="pcb-empty">Functional architecture appears here after synthesis.</div>;
  return (
    <div style={{ position: "absolute", inset: 0, overflow: "auto", padding: 20 }}>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(210px, 1fr))", gap: 12, alignContent: "start" }}>
        {vm.blocks.map((b) => {
          const parts = vm.components.filter((c) => c.from_block === b.id);
          return (
            <div
              key={b.id}
              className="block-card"
              onClick={() => select(b.id)}
              style={selected === b.id ? { borderColor: "var(--accent)" } : undefined}
            >
              <h3>{b.name}</h3>
              <p>{b.function}</p>
              <div className="meta">
                {b.requirements.length} req{b.requirements.length === 1 ? "" : "s"}
                {parts.length > 0 && ` · ${parts.map((p) => p.refdes).join(", ")}`}
              </div>
            </div>
          );
        })}
      </div>
      <div style={{ marginTop: 18, color: "var(--text-muted)", fontSize: 12 }}>
        Nets: {vm.nets.map((n) => n.name).join(" · ") || "—"}
      </div>
    </div>
  );
}
