import { ArrowRight, CornerDownLeft, Check, X, Sparkles } from "lucide-react";
import { useEffect, useRef } from "react";
import { useStore } from "../../store/useWorkspaceStore";
import type { Activity } from "../../store/fold";
import { shortId } from "../util";

const LANE_ROLE: Record<Activity["lane"], string> = {
  you: "You",
  reason: "Agent · reasoning",
  commit: "Agent",
  milestone: "Agent · milestone",
  assumption: "Agent · assumption",
  violation: "Agent · finding",
  tradeoff: "Agent · tradeoff",
  risk: "Agent · risk",
};

export function AgentPanel() {
  const { vm, select, selected } = useStore();
  const bodyRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = bodyRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [vm.activity.length]);

  const openAssumptions = new Set(vm.assumptions.filter((a) => a.status === "Open").map((a) => a.id));

  return (
    <section className="ide-sidebar" aria-label="Agent">
      <div className="sidebar-head">Agent</div>
      <div className="sidebar-body" ref={bodyRef}>
        <div className="feed">
          {vm.activity.length === 0 && (
            <div className="empty" style={{ height: 160 }}>
              <Sparkles size={20} strokeWidth={1.5} />
              The AI engineer's reasoning streams here.
            </div>
          )}
          {vm.activity.map((a) => {
            const isOpen = a.lane === "assumption" && openAssumptions.has(a.entity ?? "");
            return (
              <div key={`${a.seq}-${a.kind}`} className={`msg ${a.lane === "you" ? "you" : ""}`}>
                <div className="msg-role">
                  {a.lane === "reason" && <Sparkles size={12} strokeWidth={1.5} />}
                  {LANE_ROLE[a.lane]}
                </div>
                <div className="msg-title">{a.title}</div>
                {a.detail && <div className="msg-detail">{a.detail}</div>}
                {a.entity && (
                  <button
                    className="trace-link"
                    onClick={() => select(a.entity)}
                    style={selected === a.entity ? { color: "var(--accent)" } : undefined}
                  >
                    → trace · {shortId(a.entity)}
                  </button>
                )}
                {isOpen && (
                  <div className="checkpoint">
                    <button className="btn success sm">
                      <Check size={13} strokeWidth={2} /> Approve
                    </button>
                    <button className="btn ghost sm">
                      <X size={13} strokeWidth={2} /> Reject
                    </button>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>
      <div className="agent-input">
        <span style={{ color: "var(--text-muted)", display: "inline-flex" }}>
          <ArrowRight size={14} strokeWidth={1.5} />
        </span>
        <input placeholder="Describe intent or ask the agent…" aria-label="Instruction" />
        <span className="kbd">
          <CornerDownLeft size={11} strokeWidth={1.5} />
        </span>
      </div>
    </section>
  );
}
