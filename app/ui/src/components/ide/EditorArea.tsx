import { Maximize2, Plus, Minus, MousePointer2, Hand, Ruler, Play, Pause, SkipForward, RotateCcw, Layers as LayersIcon, FileText, CircuitBoard, FileCode2, Boxes, X } from "lucide-react";
import { useStore, activeTabOf, type Tab } from "../../store/useWorkspaceStore";
import { fileName } from "../../docs/projectDocs";
import { PcbCanvas } from "./PcbCanvas";
import { SchematicView } from "./SchematicView";
import { IrView } from "./IrView";
import { DocEditor } from "./DocEditor";

function tabLabel(t: Tab): { label: string; icon: JSX.Element } {
  if (t.kind === "doc") return { label: fileName(t.path), icon: <FileText size={13} strokeWidth={1.5} /> };
  const map = { schematic: FileCode2, pcb: CircuitBoard, "3d": Boxes, ir: Boxes } as const;
  const Icon = map[t.doc];
  return { label: t.doc === "ir" ? "IR" : t.doc === "3d" ? "3D" : t.doc[0].toUpperCase() + t.doc.slice(1), icon: <Icon size={13} strokeWidth={1.5} /> };
}

export function EditorArea() {
  const store = useStore();
  const { tabs, activeTab, setActiveTab, closeTab, tool, setTool, camera, setCamera, requestFit, toggleLayer, playing, play, pause, restart, skipToEnd } = store;
  const active = activeTabOf(store);
  const zoomBy = (f: number) => setCamera({ zoom: Math.max(1, Math.min(200, camera.zoom * f)) });

  return (
    <div className="ide-editor">
      {/* tab strip */}
      <div className="tabbar">
        {tabs.map((t, i) => {
          const { label, icon } = tabLabel(t);
          return (
            <div key={i} className={`etab ${i === activeTab ? "active" : ""}`} onClick={() => setActiveTab(i)}>
              <span className="etab-icon">{icon}</span>
              <span className="etab-label">{label}</span>
              <button className="etab-close" onClick={(e) => { e.stopPropagation(); closeTab(i); }}><X size={12} strokeWidth={1.75} /></button>
            </div>
          );
        })}
        {tabs.length === 0 && <div className="etab-empty">No open editors</div>}
      </div>

      {/* context toolbar (PCB only) */}
      {active?.kind === "design" && active.doc === "pcb" && (
        <div className="ctxbar">
          <button className="iconbtn" title="Fit to board" onClick={requestFit}><Maximize2 size={15} strokeWidth={1.5} /></button>
          <button className="iconbtn" title="Zoom in" onClick={() => zoomBy(1.2)}><Plus size={15} strokeWidth={1.5} /></button>
          <button className="iconbtn" title="Zoom out" onClick={() => zoomBy(1 / 1.2)}><Minus size={15} strokeWidth={1.5} /></button>
          <span className="sep" />
          <button className={`iconbtn ${tool === "select" ? "on" : ""}`} title="Select" onClick={() => setTool("select")}><MousePointer2 size={15} strokeWidth={1.5} /></button>
          <button className={`iconbtn ${tool === "pan" ? "on" : ""}`} title="Pan" onClick={() => setTool("pan")}><Hand size={15} strokeWidth={1.5} /></button>
          <button className={`iconbtn ${tool === "measure" ? "on" : ""}`} title="Measure" onClick={() => setTool("measure")}><Ruler size={15} strokeWidth={1.5} /></button>
          <span className="sep" />
          <button className="iconbtn" title="Toggle ratlines" onClick={() => toggleLayer("ratline")}><LayersIcon size={15} strokeWidth={1.5} /></button>
          <span className="grow" />
          <span className="proj-tag" title="A projection of the owned model — never the source of truth.">projection</span>
          <span className="sep" />
          <button className={`iconbtn ${playing ? "on" : ""}`} title={playing ? "Pause replay" : "Play replay"} onClick={() => (playing ? pause() : play())}>{playing ? <Pause size={15} strokeWidth={1.5} /> : <Play size={15} strokeWidth={1.5} />}</button>
          <button className="iconbtn" title="Skip to end" onClick={skipToEnd}><SkipForward size={15} strokeWidth={1.5} /></button>
          <button className="iconbtn" title="Restart run" onClick={restart}><RotateCcw size={15} strokeWidth={1.5} /></button>
        </div>
      )}

      <div className="editor-stage">
        {!active && <div className="pcb-empty">Open a document or design view from the Explorer.</div>}
        {active?.kind === "doc" && <DocEditor path={active.path} />}
        {active?.kind === "design" && active.doc === "pcb" && <PcbCanvas />}
        {active?.kind === "design" && active.doc === "schematic" && <SchematicView />}
        {active?.kind === "design" && active.doc === "ir" && <IrView />}
        {active?.kind === "design" && active.doc === "3d" && (
          <div className="pcb-empty"><LayersIcon size={22} strokeWidth={1.2} style={{ opacity: 0.5 }} /><div>3D board view — <span className="mono" style={{ color: "var(--scaffold)" }}>◐ planned</span></div></div>
        )}
      </div>
    </div>
  );
}
