import { ChevronDown, ChevronRight, FileText, Folder, FolderOpen, CircuitBoard, FileCode2, Boxes } from "lucide-react";
import { useMemo, useState } from "react";
import { useStore, activeTabOf, type DesignDoc } from "../../store/useWorkspaceStore";
import { projectDocs, docTree, fileName } from "../../docs/projectDocs";

const FOLDER_LABEL: Record<string, string> = {
  docs: "docs",
  decisions: "decisions",
  bom: "bom",
  verification: "verification",
};

const DESIGN: { doc: DesignDoc; label: string; icon: typeof CircuitBoard }[] = [
  { doc: "schematic", label: "Schematic", icon: FileCode2 },
  { doc: "pcb", label: "PCB", icon: CircuitBoard },
  { doc: "ir", label: "Owned Model (IR)", icon: Boxes },
];

function FolderRow({ name, open, onClick }: { name: string; open: boolean; onClick: () => void }) {
  return (
    <button className="trow depth1" onClick={onClick}>
      <span className="chev">{open ? <ChevronDown size={13} /> : <ChevronRight size={13} />}</span>
      <span className="ticon">{open ? <FolderOpen size={14} /> : <Folder size={14} />}</span>
      <span className="lname">{FOLDER_LABEL[name] ?? name}</span>
    </button>
  );
}

export function Explorer() {
  const store = useStore();
  const { vm, openDoc, openDesign } = store;
  const active = activeTabOf(store);
  const folders = useMemo(() => docTree(projectDocs(vm)), [vm]);
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});
  const [designOpen, setDesignOpen] = useState(true);

  const isActiveDoc = (path: string) => active?.kind === "doc" && active.path === path;
  const isActiveDesign = (doc: DesignDoc) => active?.kind === "design" && active.doc === doc;

  return (
    <section className="ide-sidebar" aria-label="Explorer">
      <div className="sidebar-head">Explorer</div>
      <div className="sidebar-body">
        <div className="tree">
          <button className="trow depth0"><span className="chev"><ChevronDown size={13} /></span><span className="ticon"><FolderOpen size={14} /></span><span className="lname" style={{ fontWeight: 600 }}>usb-c-temp-sensor</span></button>

          {folders.map((f) => {
            const open = !collapsed[f.name];
            return (
              <div key={f.name}>
                <FolderRow name={f.name} open={open} onClick={() => setCollapsed((c) => ({ ...c, [f.name]: open }))} />
                {open &&
                  f.docs.map((d) => (
                    <button key={d.path} className={`trow depth2 ${isActiveDoc(d.path) ? "sel" : ""}`} onClick={() => openDoc(d.path)}>
                      <span className="chev" />
                      <span className="ticon"><FileText size={13} /></span>
                      <span className="lname">{fileName(d.path)}</span>
                    </button>
                  ))}
              </div>
            );
          })}

          {/* Design group — the artifacts as openable projections */}
          <button className="trow depth1" onClick={() => setDesignOpen((o) => !o)}>
            <span className="chev">{designOpen ? <ChevronDown size={13} /> : <ChevronRight size={13} />}</span>
            <span className="ticon"><CircuitBoard size={14} /></span>
            <span className="lname">Design</span>
          </button>
          {designOpen &&
            DESIGN.map(({ doc, label, icon: Icon }) => (
              <button key={doc} className={`trow depth2 ${isActiveDesign(doc) ? "sel" : ""}`} onClick={() => openDesign(doc)}>
                <span className="chev" />
                <span className="ticon"><Icon size={13} /></span>
                <span className="lname">{label}</span>
              </button>
            ))}
        </div>
      </div>
    </section>
  );
}
