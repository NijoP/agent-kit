import { Files, Search, Sparkles, CircuitBoard, ShieldCheck, GitBranch, Library, Settings } from "lucide-react";
import { useStore, type PanelView } from "../../store/useWorkspaceStore";

const TOP: { view: PanelView; icon: typeof Files; label: string }[] = [
  { view: "explorer", icon: Files, label: "Explorer" },
  { view: "search", icon: Search, label: "Search" },
  { view: "agent", icon: Sparkles, label: "Agent" },
  { view: "design", icon: CircuitBoard, label: "Design" },
  { view: "library", icon: Library, label: "Library" },
  { view: "verify", icon: ShieldCheck, label: "Verify" },
  { view: "revisions", icon: GitBranch, label: "Revisions" },
];

export function ActivityBar() {
  const { activePanel, showSidebar, setPanel } = useStore();
  const isActive = (v: PanelView) => activePanel === v && showSidebar;
  return (
    <nav className="ide-activitybar" aria-label="Activity">
      {TOP.map(({ view, icon: Icon, label }) => (
        <button key={view} className={`abtn ${isActive(view) ? "active" : ""}`} onClick={() => setPanel(view)} title={label} aria-label={label}>
          <Icon size={20} strokeWidth={1.5} />
        </button>
      ))}
      <span className="spacer" />
      <button className={`abtn ${isActive("settings") ? "active" : ""}`} onClick={() => setPanel("settings")} title="Settings" aria-label="Settings">
        <Settings size={20} strokeWidth={1.5} />
      </button>
    </nav>
  );
}
