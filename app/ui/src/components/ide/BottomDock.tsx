import { CircleCheck, OctagonAlert, TriangleAlert, ChevronDown, Search, ListTodo, Sparkles, CircleDot, Crosshair, MessageSquareText, ShieldQuestion } from "lucide-react";
import { useMemo, useState } from "react";
import { useStore, type BottomTab } from "../../store/useWorkspaceStore";
import { tasks } from "../../docs/tasks";
import { shortId } from "../util";
import type { ViewModel } from "../../store/fold";
import type { Violation } from "../../contract/v1";

const TABS: { id: BottomTab; label: string }[] = [
  { id: "problems", label: "Problems" },
  { id: "tasks", label: "Tasks" },
  { id: "agent", label: "Agent" },
  { id: "drc", label: "DRC" },
  { id: "log", label: "Log" },
  { id: "find", label: "Find" },
];

function ViolationRow({ vm, v }: { vm: ViewModel; v: Violation }) {
  const { select } = useStore();
  const [explained, setExplained] = useState(false);
  const explanation = vm.explanations[v.id];
  const color = v.severity === "Error" ? "var(--error)" : v.severity === "Warning" ? "var(--warn)" : "var(--text-muted)";
  const Glyph = v.severity === "Error" ? OctagonAlert : v.severity === "Warning" ? TriangleAlert : CircleDot;
  return (
    <div>
      <div className="drc-row">
        <span className="glyph" style={{ color }}><Glyph size={15} strokeWidth={1.75} /></span>
        <button className="msg" onClick={() => v.subjects[0] && select(v.subjects[0])}>
          <span className="rule" style={{ color, marginRight: 8 }}>{v.rule}</span>
          {v.message}
        </button>
        {v.subjects[0] && (
          <button className="subj" style={{ background: "none", border: "none" }} title="Go to subject" onClick={() => select(v.subjects[0])}>
            <Crosshair size={11} strokeWidth={1.75} /> {shortId(v.subjects[0])}
          </button>
        )}
        <div className="actions">
          <button
            className="btn ghost sm"
            title="Explain this finding (kernel explanation, when available)"
            onClick={() => setExplained((x) => !x)}
          >
            <MessageSquareText size={12} strokeWidth={1.75} /> Explain
          </button>
          <button className="btn ghost sm" title="Submit a waiver via the seam" disabled style={{ opacity: 0.45, pointerEvents: "none" }}>
            <ShieldQuestion size={12} strokeWidth={1.75} /> Waive <span style={{ color: "var(--scaffold)" }}>◐</span>
          </button>
        </div>
      </div>
      {explained && (
        <div className="viol-expl">
          <h4>Explanation</h4>
          {explanation ? (
            <>
              <p>{explanation.explanation}</p>
              {explanation.suggestedFix && <p className="fix"><strong>Suggested fix:</strong> {explanation.suggestedFix}</p>}
            </>
          ) : (
            <p style={{ color: "var(--text-muted)" }}>
              No kernel explanation recorded for this finding yet — the explainer writes one when the violation is raised with reasoning enabled.
            </p>
          )}
        </div>
      )}
    </div>
  );
}

export function BottomDock() {
  const { vm, gate, bottomTab, setBottom, toggle, select, openDoc } = useStore();
  const open = vm.violations.filter((v) => v.status === "Open");
  const taskList = useMemo(() => tasks(vm), [vm]);
  const problems = open.length;

  const badge = (n: number, ok = false) => <span className={`badge-num ${ok ? "ok" : ""}`}>{n}</span>;

  return (
    <section className="ide-bottomdock" aria-label="Bottom dock">
      <div className="dock-tabs">
        {TABS.map((t) => (
          <button key={t.id} className={`dock-tab ${bottomTab === t.id ? "active" : ""}`} onClick={() => setBottom(t.id)}>
            {t.label}
            {t.id === "problems" && badge(problems, problems === 0)}
            {t.id === "tasks" && badge(taskList.length, false)}
            {t.id === "drc" && badge(open.length, open.length === 0)}
          </button>
        ))}
        <span className="grow" />
        {!gate.released && gate.reason && (
          <span className={`gate-chip ${gate.reason === "in progress" ? "progress" : "blocked"}`} style={{ marginRight: 10 }}>
            Gate {gate.reason === "in progress" ? "IN PROGRESS" : `BLOCKED · ${gate.reason}`}
          </span>
        )}
        <span className="sync-badge" title="Docs are a projection of the owned model — always in sync."><CircleDot size={11} strokeWidth={2} /> docs ⇄ model: in sync</span>
        <button className="iconbtn" title="Collapse" onClick={() => toggle("showBottom")}><ChevronDown size={14} strokeWidth={1.5} /></button>
      </div>

      <div className="dock-content">
        {bottomTab === "problems" && (
          problems === 0 ? (
            <div className="drc-clean"><CircleCheck size={16} strokeWidth={1.75} /> No problems. Docs are in sync with the model and the gate is {gate.released ? "RELEASED" : "in progress"}.</div>
          ) : (
            <>
              <div className="viol-group">Blocking findings · gate {gate.released ? "released" : "blocked"}</div>
              {open.map((v) => <ViolationRow key={v.id} vm={vm} v={v} />)}
            </>
          )
        )}

        {bottomTab === "tasks" && (
          <div>
            {taskList.map((t) => {
              const Glyph = t.severity === "error" ? OctagonAlert : t.severity === "warn" ? TriangleAlert : ListTodo;
              const color = t.severity === "error" ? "var(--error)" : t.severity === "warn" ? "var(--warn)" : "var(--text-muted)";
              return (
                <div className="drc-row" key={t.id}>
                  <span className="glyph" style={{ color }}><Glyph size={14} strokeWidth={1.75} /></span>
                  <button className="msg" onClick={() => t.entity && select(t.entity)}>{t.title}</button>
                  <button className="subj" style={{ background: "none", border: "none" }} onClick={() => openDoc(t.source)}>{t.source}</button>
                </div>
              );
            })}
            {taskList.length === 0 && <div className="drc-clean"><CircleCheck size={16} strokeWidth={1.75} /> No open tasks.</div>}
          </div>
        )}

        {bottomTab === "agent" && (
          <div style={{ padding: 12 }}>
            <div className="changeset">
              <div className="changeset-head"><Sparkles size={13} strokeWidth={1.5} /> Agent · engineering partner</div>
              <div className="changeset-body">
                Describe intent in the Agent panel and the partner proposes a change-set — the docs it will write, the model deltas, and new tasks — for your approval before anything is committed.
                <div className="changeset-example">
                  <span className="cs-line add">+ decisions/ADR-0002-esd-protection.md</span>
                  <span className="cs-line add">+ part · TVS diode on VBUS</span>
                  <span className="cs-line add">+ task · verify clamping voltage</span>
                </div>
                <span className="mono" style={{ color: "var(--scaffold)", fontSize: 11 }}>◐ live generation next — the seam &amp; UI are ready</span>
              </div>
            </div>
          </div>
        )}

        {bottomTab === "drc" && (
          open.length === 0 ? (
            <div className="drc-clean"><CircleCheck size={16} strokeWidth={1.75} /> {gate.released ? "No blocking findings. Gate RELEASED." : "No findings raised yet."}</div>
          ) : (
            <>
              <div className="viol-group">Constraint · ERC · BOM · DRC · EMC · DFM findings — grouped by rule</div>
              {open.map((v) => <ViolationRow key={v.id} vm={vm} v={v} />)}
            </>
          )
        )}

        {bottomTab === "log" && (
          <div>
            {vm.activity.map((a) => (
              <div className="log-row" key={`${a.seq}-${a.kind}`}>
                <span className="lseq">#{a.seq}</span><span className="lkind">{a.kind}</span>
                <span className="lbody">{a.title}{a.detail ? ` — ${a.detail}` : ""}</span>
              </div>
            ))}
            {vm.activity.length === 0 && <div className="empty" style={{ height: 100 }}>The committed-event audit stream appears here.</div>}
          </div>
        )}

        {bottomTab === "find" && (
          <div className="empty" style={{ height: 120 }}><Search size={18} strokeWidth={1.5} /> Search docs, parts, nets, requirements. <span className="mono" style={{ color: "var(--scaffold)" }}>◐ planned</span></div>
        )}
      </div>
    </section>
  );
}