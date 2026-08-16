import { Search, CornerDownLeft } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "../../store/useWorkspaceStore";

interface Command {
  label: string;
  hint?: string;
  run: () => void;
}

export function CommandPalette() {
  const store = useStore();
  const { paletteOpen, togglePalette, vm } = store;
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const commands = useMemo<Command[]>(
    () => [
      { label: "Open: PRD", run: () => store.openDoc("docs/00-prd.md") },
      { label: "Open: Requirements", run: () => store.openDoc("docs/01-requirements.md") },
      { label: "Open: Architecture", run: () => store.openDoc("docs/02-architecture.md") },
      { label: "Open: BOM", run: () => store.openDoc("bom/bom.md") },
      { label: "Open: Verification checklist", run: () => store.openDoc("verification/checklist.md") },
      { label: "View: PCB", hint: "⌘3", run: () => store.openDesign("pcb") },
      { label: "View: Schematic", hint: "⌘4", run: () => store.openDesign("schematic") },
      { label: "View: Owned Model (IR)", hint: "⌘5", run: () => store.openDesign("ir") },
      { label: "Verify: Problems", run: () => store.setBottom("problems") },
      { label: "Verify: Tasks", run: () => store.setBottom("tasks") },
      { label: "Design: Run design", run: () => store.restart() },
      { label: "Design: Run DRC", run: () => store.setBottom("drc") },
      { label: "Route: Toggle ratlines", run: () => store.toggleLayer("ratline") },
      { label: "View: Library", run: () => store.setPanel("library") },
      { label: "Tools: Fit to board", hint: "⌘0", run: () => store.requestFit() },
      { label: "Tools: Select", run: () => store.setTool("select") },
      { label: "Tools: Pan", run: () => store.setTool("pan") },
      { label: "Tools: Measure", run: () => store.setTool("measure") },
      { label: "View: Toggle Sidebar", hint: "⌘B", run: () => store.toggle("showSidebar") },
      { label: "View: Toggle Inspector", hint: "⌘2", run: () => store.toggle("showRightDock") },
      { label: "View: Toggle Bottom Dock", hint: "⌘J", run: () => store.toggle("showBottom") },
      { label: "View: Toggle Top Copper", run: () => store.toggleLayer("topCopper") },
      { label: "View: Toggle Bottom Copper", run: () => store.toggleLayer("bottomCopper") },
      { label: "View: Toggle Silkscreen", run: () => store.toggleLayer("silk") },
      { label: "View: Toggle Board Outline", run: () => store.toggleLayer("outline") },
      { label: "Go to Agent", run: () => store.setPanel("agent") },
      { label: "Go to Settings", run: () => store.setPanel("settings") },
      ...vm.nets.map((n): Command => ({ label: `Go to net: ${n.name}`, run: () => { store.openDesign("pcb"); store.select(n.id); } })),
      ...vm.components.map((c): Command => ({ label: `Go to component: ${c.refdes}`, run: () => { store.openDesign("pcb"); store.select(c.id); } })),
      ...vm.parts.map((p): Command => ({ label: `Go to part: ${p.mpn}`, run: () => store.select(p.id) })),
      ...vm.violations.filter((v) => v.status === "Open").map((v): Command => ({ label: `Go to finding: ${v.rule}`, run: () => { store.openDesign("pcb"); store.select(v.id); } })),
    ],
    [store, vm],
  );

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return q ? commands.filter((c) => c.label.toLowerCase().includes(q)) : commands;
  }, [commands, query]);

  useEffect(() => {
    if (paletteOpen) {
      setQuery("");
      setActive(0);
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [paletteOpen]);
  useEffect(() => setActive(0), [query]);

  if (!paletteOpen) return null;
  const exec = (c?: Command) => {
    if (!c) return;
    c.run();
    togglePalette(false);
  };

  return (
    <div className="palette-scrim" onClick={() => togglePalette(false)}>
      <div className="palette" onClick={(e) => e.stopPropagation()} role="dialog" aria-label="Command palette">
        <div className="palette-input">
          <Search size={16} strokeWidth={1.5} color="var(--text-muted)" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Run a command, jump to an object, or ask the agent…"
            onKeyDown={(e) => {
              if (e.key === "ArrowDown") { e.preventDefault(); setActive((a) => Math.min(a + 1, filtered.length - 1)); }
              else if (e.key === "ArrowUp") { e.preventDefault(); setActive((a) => Math.max(a - 1, 0)); }
              else if (e.key === "Enter") exec(filtered[active]);
              else if (e.key === "Escape") togglePalette(false);
            }}
          />
          <span className="kbd">esc</span>
        </div>
        <div className="palette-list">
          {filtered.length === 0 && <div className="palette-empty">No matching commands.</div>}
          {filtered.map((c, i) => (
            <div
              key={c.label}
              className={`palette-item ${i === active ? "active" : ""}`}
              onMouseEnter={() => setActive(i)}
              onClick={() => exec(c)}
            >
              {c.label}
              {c.hint ? <span className="kbd">{c.hint}</span> : i === active ? <span className="kbd"><CornerDownLeft size={11} strokeWidth={1.5} /></span> : null}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
