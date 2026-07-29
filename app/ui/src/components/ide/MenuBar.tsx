import { Zap, Search } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useStore } from "../../store/useWorkspaceStore";

interface Item {
  label: string;
  kbd?: string;
  run?: () => void;
  planned?: boolean;
  sep?: boolean;
}

/**
 * Thin menu bar with hardware verbs (Place/Route/Design/Tools/Export from the ECAD idiom). Most
 * items dispatch a store action or open the command palette; unbuilt actions are ◐ planned. The
 * menus are discoverability aids — the ⌘K palette is the real command surface.
 */
export function MenuBar() {
  const store = useStore();
  const [open, setOpen] = useState<string | null>(null);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onDoc = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(null);
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, []);

  const palette = () => store.togglePalette(true);
  const menus: Record<string, Item[]> = {
    File: [
      { label: "New Design", kbd: "⌘N", planned: true },
      { label: "Open…", kbd: "⌘O", planned: true },
      { label: "Import .kicad_pcb…", planned: true },
      { sep: true, label: "" },
      { label: "Export ▸", run: palette },
      { label: "Save / Snapshot", kbd: "⌘S", planned: true },
    ],
    Edit: [
      { label: "Undo", kbd: "⌘Z", planned: true },
      { label: "Redo", kbd: "⇧⌘Z", planned: true },
      { sep: true, label: "" },
      { label: "Preferences…", kbd: "⌘,", run: () => store.setPanel("settings") },
    ],
    View: [
      { label: "Command Palette", kbd: "⌘K", run: palette },
      { sep: true, label: "" },
      { label: "Toggle Sidebar", kbd: "⌘B", run: () => store.toggle("showSidebar") },
      { label: "Toggle Inspector", kbd: "⌘2", run: () => store.toggle("showRightDock") },
      { label: "Toggle Bottom Dock", kbd: "⌘J", run: () => store.toggle("showBottom") },
      { sep: true, label: "" },
      { label: "PCB", kbd: "⌘3", run: () => store.openDesign("pcb") },
      { label: "Schematic", kbd: "⌘4", run: () => store.openDesign("schematic") },
      { label: "Owned Model (IR)", kbd: "⌘5", run: () => store.openDesign("ir") },
    ],
    Place: [
      { label: "Place Component", planned: true },
      { label: "Place Net Label", planned: true },
      { label: "Auto-place (agent)", run: palette },
    ],
    Route: [
      { label: "Route Net", planned: true },
      { label: "Auto-route (agent)", run: palette },
      { label: "Toggle Ratlines", run: () => store.toggleLayer("ratline") },
    ],
    Design: [
      { label: "Run design", kbd: "⌘↵", run: () => store.restart() },
      { label: "Open PRD", run: () => store.openDoc("docs/00-prd.md") },
      { label: "Open Requirements", run: () => store.openDoc("docs/01-requirements.md") },
      { label: "Open Architecture", run: () => store.openDoc("docs/02-architecture.md") },
    ],
    Verify: [
      { label: "Run all checks", run: () => store.setBottom("problems") },
      { label: "Verification checklist", run: () => store.openDoc("verification/checklist.md") },
      { label: "Problems", kbd: "⌘⇧M", run: () => store.setBottom("problems") },
      { label: "Tasks", run: () => store.setBottom("tasks") },
    ],
    Agent: [
      { label: "New instruction", kbd: "⌘L", run: () => store.setPanel("agent") },
      { label: "Proposed change-set", run: () => store.setBottom("agent") },
      { label: "Reasoning: Live ⇄ Cassette", planned: true },
    ],
    Tools: [
      { label: "Select", run: () => store.setTool("select") },
      { label: "Pan", run: () => store.setTool("pan") },
      { label: "Measure", run: () => store.setTool("measure") },
      { sep: true, label: "" },
      { label: "Fit to board", kbd: "⌘0", run: () => store.requestFit() },
    ],
    Export: [
      { label: ".kicad_pcb", planned: true },
      { label: "BOM (CSV)", planned: true },
      { label: "Report (PDF)", planned: true },
    ],
    Help: [
      { label: "User Manual", planned: true },
      { label: "Honesty Legend", planned: true },
      { label: "About EAK", planned: true },
    ],
  };

  return (
    <header className="ide-menubar" ref={ref}>
      <span className="brand">
        <span className="mark">
          <Zap size={16} strokeWidth={2} fill="currentColor" />
        </span>
        EAK
      </span>

      {Object.entries(menus).map(([name, items]) => (
        <div className={`menu ${open === name ? "open" : ""}`} key={name}>
          <button
            className="menu-label"
            onClick={() => setOpen(open === name ? null : name)}
            onMouseEnter={() => open && setOpen(name)}
          >
            {name}
          </button>
          {open === name && (
            <div className="menu-pop">
              {items.map((it, i) =>
                it.sep ? (
                  <div className="menu-sep" key={i} />
                ) : (
                  <button
                    key={i}
                    className={`menu-item ${it.planned ? "disabled" : ""}`}
                    onClick={() => {
                      it.run?.();
                      setOpen(null);
                    }}
                  >
                    {it.label}
                    {it.planned && <span className="plan" style={{ marginLeft: "auto", fontSize: 10, color: "var(--scaffold)" }}>◐</span>}
                    {it.kbd && <span className="kbd">{it.kbd}</span>}
                  </button>
                ),
              )}
            </div>
          )}
        </div>
      ))}

      <button className="cmdbar" onClick={palette}>
        <Search size={13} strokeWidth={1.5} />
        Search &amp; run commands…
        <span className="kbd">⌘K</span>
      </button>

      <div className="winctl" aria-hidden>
        <span className="wc" />
        <span className="wc" />
        <span className="wc" />
      </div>
    </header>
  );
}
