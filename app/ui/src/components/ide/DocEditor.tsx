import { Eye, Code2, RefreshCw } from "lucide-react";
import { useMemo } from "react";
import { useStore } from "../../store/useWorkspaceStore";
import { projectDocs } from "../../docs/projectDocs";
import { Markdown } from "./Markdown";

/** A Markdown doc tab — a live projection of the owned model, with a preview⇄source toggle, a sync
 *  badge, and inline entity chips that cross-probe to the board/inspector. */
export function DocEditor({ path }: { path: string }) {
  const { vm, docView, setDocView, select, openDesign } = useStore();
  const doc = useMemo(() => projectDocs(vm).find((d) => d.path === path), [vm, path]);

  if (!doc) return <div className="pcb-empty">Document not found.</div>;

  const onEntity = (id: string) => {
    select(id);
    openDesign("pcb"); // cross-probe: reveal the linked entity on the board
  };

  return (
    <div className="doc">
      <div className="doc-bar">
        <div className="doc-viewtoggle">
          <button className={`iconbtn ${docView === "preview" ? "on" : ""}`} title="Preview" onClick={() => setDocView("preview")}><Eye size={14} strokeWidth={1.5} /></button>
          <button className={`iconbtn ${docView === "source" ? "on" : ""}`} title="Source" onClick={() => setDocView("source")}><Code2 size={14} strokeWidth={1.5} /></button>
        </div>
        <span className="grow" />
        <span className="sync-badge" title="This document is a projection of the owned model and is always in sync with the kernel.">
          <RefreshCw size={12} strokeWidth={1.75} /> synced
        </span>
        <span className="doc-links">{doc.links.length} linked</span>
      </div>
      <div className="doc-body">
        {docView === "preview" ? (
          <div className="doc-page">
            <Markdown md={doc.md} onEntity={onEntity} />
          </div>
        ) : (
          <pre className="doc-source">{doc.md}</pre>
        )}
      </div>
    </div>
  );
}
