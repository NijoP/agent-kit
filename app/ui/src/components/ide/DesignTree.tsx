import {
  ChevronDown,
  ChevronRight,
  FolderOpen,
  CircuitBoard,
  Waypoints,
  FileText,
  Cpu,
  Boxes,
  ListChecks,
} from "lucide-react";
import { useState, type ReactNode } from "react";
import { useStore, activeTabOf } from "../../store/useWorkspaceStore";
import type { EntityId } from "../../contract/v1";
import { shortId } from "../util";

function Row({
  depth,
  open,
  hasChildren,
  icon,
  label,
  count,
  selected,
  onClick,
}: {
  depth: 0 | 1 | 2 | 3;
  open?: boolean;
  hasChildren?: boolean;
  icon: ReactNode;
  label: ReactNode;
  count?: number;
  selected?: boolean;
  onClick?: () => void;
}) {
  return (
    <button className={`trow depth${depth} ${selected ? "sel" : ""}`} onClick={onClick}>
      <span className="chev">
        {hasChildren ? open ? <ChevronDown size={13} /> : <ChevronRight size={13} /> : null}
      </span>
      <span className="ticon">{icon}</span>
      <span className="lname" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        {label}
      </span>
      {count !== undefined && <span className="tcount">{count}</span>}
    </button>
  );
}

function Group({
  depth,
  icon,
  label,
  count,
  children,
  defaultOpen,
}: {
  depth: 0 | 1 | 2 | 3;
  icon: ReactNode;
  label: string;
  count: number;
  children?: ReactNode;
  defaultOpen?: boolean;
}) {
  const [open, setOpen] = useState(defaultOpen ?? false);
  return (
    <>
      <Row
        depth={depth}
        open={open}
        hasChildren={count > 0}
        icon={icon}
        label={label}
        count={count}
        onClick={() => setOpen((o) => !o)}
      />
      {open && children}
    </>
  );
}

export function DesignTree() {
  const store = useStore();
  const { vm, openDesign, select, selected } = store;
  const active = activeTabOf(store);
  const isDoc = (d: "schematic" | "pcb" | "ir") => active?.kind === "design" && active.doc === d;
  const pick = (id: EntityId) => select(id);

  return (
    <section className="ide-sidebar" aria-label="Design tree">
      <div className="sidebar-head">Design</div>
      <div className="sidebar-body">
        <div className="tree">
          <Group depth={0} icon={<FolderOpen size={14} />} label="usb-c-temp-sensor" count={3} defaultOpen>
            <Group depth={1} icon={<Boxes size={14} />} label="Board1" count={3} defaultOpen>
              <Row depth={2} icon={<FileText size={14} />} label="Schematic" selected={isDoc("schematic")} onClick={() => openDesign("schematic")} />
              <Row depth={2} icon={<CircuitBoard size={14} />} label="PCB" selected={isDoc("pcb")} onClick={() => openDesign("pcb")} />
              <Row depth={2} icon={<ListChecks size={14} />} label="Owned Model (IR)" selected={isDoc("ir")} onClick={() => openDesign("ir")} />
            </Group>
          </Group>

          <Group depth={0} icon={<ListChecks size={14} />} label="Requirements" count={vm.requirements.length}>
            {vm.requirements.map((r) => (
              <Row
                key={r.id}
                depth={1}
                icon={<span style={{ width: 14 }} />}
                label={r.statement.slice(0, 30)}
                selected={selected === r.id}
                onClick={() => pick(r.id)}
              />
            ))}
          </Group>

          <Group depth={0} icon={<Cpu size={14} />} label="Components" count={vm.components.length} defaultOpen>
            {vm.components.map((c) => (
              <Row
                key={c.id}
                depth={1}
                icon={<span style={{ width: 14 }} />}
                label={
                  <>
                    <span className="refdes">{c.refdes}</span>{" "}
                    <span style={{ color: "var(--text-muted)" }}>{c.class}</span>
                  </>
                }
                selected={selected === c.id}
                onClick={() => pick(c.id)}
              />
            ))}
          </Group>

          <Group depth={0} icon={<Waypoints size={14} />} label="Nets" count={vm.nets.length}>
            {vm.nets.map((n) => (
              <Row
                key={n.id}
                depth={1}
                icon={<span style={{ width: 14 }} />}
                label={n.name}
                count={n.members.length}
                selected={selected === n.id}
                onClick={() => pick(n.id)}
              />
            ))}
          </Group>

          <Group depth={0} icon={<Cpu size={14} />} label="Parts / BOM" count={vm.parts.length}>
            {vm.parts.map((p) => (
              <Row
                key={p.id}
                depth={1}
                icon={<span style={{ width: 14 }} />}
                label={
                  <>
                    <span className="refdes">{p.mpn}</span>{" "}
                    <span style={{ color: "var(--text-muted)" }}>{shortId(p.id)}</span>
                  </>
                }
                selected={selected === p.id}
                onClick={() => pick(p.id)}
              />
            ))}
          </Group>
        </div>
      </div>
    </section>
  );
}
