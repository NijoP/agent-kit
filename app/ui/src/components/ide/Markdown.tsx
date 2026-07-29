import { type ReactNode } from "react";

/**
 * A compact, dependency-free Markdown renderer for the engineering docs (which EAK generates itself,
 * so the content is trusted — no untrusted HTML). Supports headings, paragraphs, blockquotes, lists,
 * GFM pipe tables, **bold**, `code`, and links. Links whose href starts with `eak:` render as an
 * entity CHIP that cross-probes to the owned model (doc → board/inspector) — the live doc↔model bridge.
 */
export function Markdown({ md, onEntity }: { md: string; onEntity?: (id: string) => void }) {
  const lines = md.replace(/\r/g, "").split("\n");
  const blocks: ReactNode[] = [];
  let i = 0;
  let key = 0;

  const inline = (text: string): ReactNode[] => {
    const out: ReactNode[] = [];
    const re = /(\*\*([^*]+)\*\*)|(`([^`]+)`)|(\[([^\]]+)\]\(([^)]+)\))/g;
    let last = 0;
    let m: RegExpExecArray | null;
    let k = 0;
    while ((m = re.exec(text))) {
      if (m.index > last) out.push(text.slice(last, m.index));
      if (m[1]) out.push(<strong key={k++}>{m[2]}</strong>);
      else if (m[3]) out.push(<code key={k++} className="md-code">{m[4]}</code>);
      else if (m[5]) {
        const label = m[6];
        const href = m[7];
        if (href.startsWith("eak:")) {
          const id = href.slice(4);
          out.push(
            <button key={k++} className="md-chip" onClick={() => onEntity?.(id)}>
              {label}
            </button>,
          );
        } else {
          out.push(<a key={k++} href={href} className="md-link" target="_blank" rel="noreferrer">{label}</a>);
        }
      }
      last = m.index + m[0].length;
    }
    if (last < text.length) out.push(text.slice(last));
    return out;
  };

  while (i < lines.length) {
    const line = lines[i];
    if (line.trim() === "") { i++; continue; }

    // heading (rendered as a styled div — semantics via aria; avoids dynamic-tag typing)
    const h = /^(#{1,4})\s+(.*)$/.exec(line);
    if (h) {
      const level = h[1].length;
      blocks.push(
        <div key={key++} role="heading" aria-level={level} className={`md-h md-h${level}`}>
          {inline(h[2])}
        </div>,
      );
      i++;
      continue;
    }

    // blockquote
    if (line.startsWith(">")) {
      const buf: string[] = [];
      while (i < lines.length && lines[i].startsWith(">")) { buf.push(lines[i].replace(/^>\s?/, "")); i++; }
      blocks.push(<blockquote key={key++} className="md-quote">{inline(buf.join(" "))}</blockquote>);
      continue;
    }

    // table
    if (line.startsWith("|")) {
      const rows: string[] = [];
      while (i < lines.length && lines[i].startsWith("|")) { rows.push(lines[i]); i++; }
      const cells = (r: string) => r.split("|").slice(1, -1).map((c) => c.trim());
      const header = cells(rows[0]);
      const body = rows.slice(2); // skip the --- separator
      blocks.push(
        <table key={key++} className="md-table">
          <thead><tr>{header.map((c, ci) => <th key={ci}>{inline(c)}</th>)}</tr></thead>
          <tbody>{body.map((r, ri) => <tr key={ri}>{cells(r).map((c, ci) => <td key={ci}>{inline(c)}</td>)}</tr>)}</tbody>
        </table>,
      );
      continue;
    }

    // list (supports [ ] / [x] checkboxes)
    if (/^[-*]\s+/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^[-*]\s+/.test(lines[i])) { items.push(lines[i].replace(/^[-*]\s+/, "")); i++; }
      blocks.push(
        <ul key={key++} className="md-list">
          {items.map((it, ii) => {
            const box = /^\[( |x)\]\s+/.exec(it);
            if (box) {
              return (
                <li key={ii} className="md-check">
                  <span className={`md-box ${box[1] === "x" ? "done" : ""}`}>{box[1] === "x" ? "✓" : ""}</span>
                  {inline(it.replace(/^\[( |x)\]\s+/, ""))}
                </li>
              );
            }
            return <li key={ii}>{inline(it)}</li>;
          })}
        </ul>,
      );
      continue;
    }

    // paragraph
    const buf: string[] = [];
    while (i < lines.length && lines[i].trim() !== "" && !/^([#>|]|[-*]\s)/.test(lines[i])) { buf.push(lines[i]); i++; }
    blocks.push(<p key={key++} className="md-p">{inline(buf.join(" "))}</p>);
  }

  return <div className="markdown">{blocks}</div>;
}
