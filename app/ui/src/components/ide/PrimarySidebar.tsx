import { GitBranch, Search, ShieldCheck, CircleCheck, CircleDashed } from "lucide-react";
import { useStore } from "../../store/useWorkspaceStore";
import { HonestyBadge } from "../Badge";
import { Explorer } from "./Explorer";
import { DesignTree } from "./DesignTree";
import { AgentPanel } from "./AgentPanel";
import { LibraryPanel } from "./LibraryPanel";

function SearchPanel() {
  return (
    <section className="ide-sidebar" aria-label="Search">
      <div className="sidebar-head">Search</div>
      <div className="sidebar-body">
        <div style={{ padding: 12 }}>
          <div className="agent-input" style={{ margin: 0 }}>
            <Search size={14} strokeWidth={1.5} style={{ color: "var(--text-muted)" }} />
            <input placeholder="Search docs, parts, nets…" aria-label="Search" />
          </div>
        </div>
        <div className="empty" style={{ height: 120 }}>Full-text + semantic search across the project. <span className="mono" style={{ color: "var(--scaffold)" }}>◐ planned</span></div>
      </div>
    </section>
  );
}

function VerifyPanel() {
  const { vm, gate, openDoc } = useStore();
  return (
    <section className="ide-sidebar" aria-label="Verify">
      <div className="sidebar-head">Verify</div>
      <div className="sidebar-body">
        <div style={{ padding: 12 }}>
          <div className={`gate-chip ${gate.released ? "released" : "progress"}`} style={{ marginBottom: 12 }}>
            <ShieldCheck size={12} strokeWidth={1.75} /> Gate {gate.released ? "RELEASED" : (gate.reason ?? "…")}
          </div>
        </div>
        <div className="tree">
          {vm.phases.map((p) => (
            <div key={p.name} className="trow depth0">
              <span className="ticon">{p.status === "done" ? <CircleCheck size={13} style={{ color: "var(--pass)" }} /> : <CircleDashed size={13} style={{ color: "var(--text-muted)" }} />}</span>
              <span className="lname" style={{ fontSize: 12.5 }}>{p.name}</span>
            </div>
          ))}
        </div>
        <div style={{ padding: 12 }}>
          <button className="btn ghost sm" onClick={() => openDoc("verification/checklist.md")}>Open checklist →</button>
        </div>
      </div>
    </section>
  );
}

function RevisionsPanel() {
  return (
    <section className="ide-sidebar" aria-label="Revisions">
      <div className="sidebar-head">Revisions</div>
      <div className="sidebar-body">
        <div className="empty"><GitBranch size={20} strokeWidth={1.5} /> Git-for-hardware — every point in the event history is a taggable, diffable position. <span className="mono" style={{ color: "var(--scaffold)" }}>◐ planned</span></div>
      </div>
    </section>
  );
}

function SettingsPanel() {
  const { mode } = useStore();
  return (
    <section className="ide-sidebar" aria-label="Settings">
      <div className="sidebar-head">Settings</div>
      <div className="sidebar-body">
        <div className="props" style={{ padding: 14 }}>
          <div className="prop-row"><span className="key">Reasoning mode</span><span className="val" style={{ display: "flex", justifyContent: "flex-end" }}><HonestyBadge mode={mode} /></span></div>
          <div className="prop-row"><span className="key">Theme</span><span className="val">Graphite</span></div>
          <div className="prop-row"><span className="key">Docs</span><span className="val">projection · always in sync</span></div>
          <div className="prop-row"><span className="key">Motion</span><span className="val">Full · respects OS</span></div>
        </div>
      </div>
    </section>
  );
}

export function PrimarySidebar() {
  const { activePanel } = useStore();
  switch (activePanel) {
    case "explorer": return <Explorer />;
    case "search": return <SearchPanel />;
    case "agent": return <AgentPanel />;
    case "design": return <DesignTree />;
    case "library": return <LibraryPanel />;
    case "verify": return <VerifyPanel />;
    case "revisions": return <RevisionsPanel />;
    case "settings": return <SettingsPanel />;
  }
}
