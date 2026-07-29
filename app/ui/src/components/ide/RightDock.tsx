import { useStore, activeTabOf } from "../../store/useWorkspaceStore";
import { LayersPanel } from "./LayersPanel";
import { Inspector } from "./Inspector";
import { OutlinePanel } from "./OutlinePanel";

/** Context-sensitive right dock: for a doc tab → Outline + Linked model; for a design tab → the
 *  EasyEDA-style Layers panel over the Inspector. */
export function RightDock() {
  const store = useStore();
  const active = activeTabOf(store);
  if (active?.kind === "doc") return <OutlinePanel path={active.path} />;
  return (
    <aside className="ide-rightdock" aria-label="Layers and inspector">
      <LayersPanel />
      <Inspector />
    </aside>
  );
}
