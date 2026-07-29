import { Hash, Link2 } from "lucide-react";
import { useMemo } from "react";
import { useStore } from "../../store/useWorkspaceStore";
import { projectDocs } from "../../docs/projectDocs";
import type { EntityId } from "../../contract/v1";
import { shortId } from "../util";

/** Right dock for a doc tab: the heading outline + the Linked Model (owned entities the doc projects,
 *  each cross-probing to the board). This is what makes the doc traceable, not prose. */
export function OutlinePanel({ path }: { path: string }) {
  const { vm, select, openDesign } = useStore();
  const doc = useMemo(() => projectDocs(vm).find((d) => d.path === path), [vm, path]);

  const headings = useMemo(() => {
    if (!doc) return [];
    return doc.md.split("\n").filter((l) => /^#{1,3}\s/.test(l)).map((l) => {
      const m = /^(#{1,3})\s+(.*)$/.exec(l)!;
      return { level: m[1].length, text: m[2].replace(/\[([^\]]+)\]\([^)]+\)/g, "$1") };
    });
  }, [doc]);

  const nameOf = (id: EntityId): string => {
    const c = vm.components.find((x) => x.id === id);
    if (c) return `${c.refdes} · ${c.class}`;
    const r = vm.requirements.find((x) => x.id === id);
    if (r) return `req ${shortId(id)}`;
    const n = vm.nets.find((x) => x.id === id);
    if (n) return `net ${n.name}`;
    const p = vm.parts.find((x) => x.id === id);
    if (p) return `part ${p.mpn}`;
    const b = vm.blocks.find((x) => x.id === id);
    if (b) return `block ${b.name}`;
    return shortId(id);
  };

  const uniqueLinks = doc ? [...new Set(doc.links)] : [];

  return (
    <div className="ide-rightdock" aria-label="Outline">
      <div className="dock-section" style={{ flexShrink: 0, maxHeight: "45%", borderBottom: "1px solid var(--border-subtle)" }}>
        <div className="dock-head">Outline</div>
        <div className="dock-body">
          {headings.map((h, i) => (
            <div key={i} className="trow" style={{ paddingLeft: 8 + h.level * 12 }}>
              <span className="ticon"><Hash size={12} /></span>
              <span className="lname" style={{ fontSize: 12.5 }}>{h.text}</span>
            </div>
          ))}
          {headings.length === 0 && <div className="empty" style={{ height: 60 }}>No headings.</div>}
        </div>
      </div>
      <div className="dock-section" style={{ flex: 1, minHeight: 0 }}>
        <div className="dock-head">Linked model · {uniqueLinks.length}</div>
        <div className="dock-body">
          {uniqueLinks.map((id) => (
            <button key={id} className="trow" onClick={() => { select(id); openDesign("pcb"); }}>
              <span className="ticon"><Link2 size={12} /></span>
              <span className="lname" style={{ fontSize: 12.5 }}>{nameOf(id)}</span>
              <span className="tcount" style={{ color: "var(--pass)" }}>synced</span>
            </button>
          ))}
          {uniqueLinks.length === 0 && <div className="empty" style={{ height: 60 }}>No linked entities.</div>}
        </div>
      </div>
    </div>
  );
}
