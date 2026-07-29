import { useEffect } from "react";
import { Workspace } from "./components/ide/Workspace";
import { useStore } from "./store/useWorkspaceStore";
// docs-first IDE
import { FixturePlayer } from "./events/FixturePlayer";
import { TauriBridge } from "./events/TauriBridge";
import type { EventSource } from "./events/EventSource";

/** True when running inside the packaged Tauri desktop shell (vs a plain browser dev build). */
function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

export default function App() {
  const { connect, togglePalette, openDesign, setPanel, toggle, requestFit, source } = useStore();

  // Pick the event source once: live Tauri bridge when packaged, else replay the captured hero
  // cassette (real kernel output) in the browser. Same fold either way.
  useEffect(() => {
    if (source) return;
    const src: EventSource = inTauri() ? new TauriBridge() : new FixturePlayer("/fixtures/hero.jsonl");
    connect(src);
  }, [connect, source]);

  // Global keyboard map.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;
      const k = e.key.toLowerCase();
      if (k === "k") { e.preventDefault(); togglePalette(); }
      else if (k === "b") { e.preventDefault(); toggle("showSidebar"); }
      else if (k === "l") { e.preventDefault(); setPanel("agent"); }
      else if (e.key === "2") { e.preventDefault(); toggle("showRightDock"); }
      else if (k === "j") { e.preventDefault(); toggle("showBottom"); }
      else if (e.key === "3") { e.preventDefault(); openDesign("pcb"); }
      else if (e.key === "4") { e.preventDefault(); openDesign("schematic"); }
      else if (e.key === "5") { e.preventDefault(); openDesign("ir"); }
      else if (e.key === "0") { e.preventDefault(); requestFit(); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [togglePalette, toggle, openDesign, setPanel, requestFit]);

  return <Workspace />;
}
