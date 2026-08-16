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
  FileCode2,
  Package,
  ShieldCheck,
  Layers,
  Box,
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
  tag,
  selected,
  onClick,
}: {
  depth: 0 | 1 | 2 | 3;
  open?: boolean;
  hasChildren?: boolean;
  icon: ReactNode;
  label: ReactNode;
  count?: number;
  tag?: ReactNode;
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
      {tag && <span style={{ marginLeft: "auto" }}>{tag}</span>}
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
  onClick,
}: {
  depth: 0 | 1 | 2 | 3;
  icon: ReactNode;
  label: string;
  count: number;
  children?: ReactNode;
  defaultOpen?: boolean;
  onClick?: () => void;
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
        onClick={onClick ?? (() => setOpen((o) => !o))}
      />
      {open && children}
    </>
  );
}

/**
 * EDA-native design manager (§22): the project as an electronics workbench — Schematics, PCB,
 * Libraries, Manufacturing, BOM, Constraints, Verification, Documentation — over the object cross-
 * probe groups (requirements, components, nets, parts).
 */
export function DesignTree() {
  const store = useStore();
  const { vm, openDesign, select, selected, openDoc, setPanel } = store;
  const active = activeTabOf(store);
  const isDoc = (d: "schematic" | "pcb" | "ir") => active?.kind === "design" && active.doc === d;
  const pick = (id: EntityId) => select(id);
  const openOpen = vm.violations.filter((v) => v.status === "Open");

  return (
    <section className="ide-sidebar" aria-label="Design tree">
      <div className="sidebar-head">Design</div>
      <div className="sidebar-body">
        <div className="tree">
          <Group depth={0} icon={<FolderOpen size={14} />} label="usb-c-temp-sensor" count={1} defaultOpen>
            {/* Schematics */}
            <Row depth={1} icon={<FileCode2 size={14} />} label="Schematics" count={vm.components.length} selected={isDoc("schematic")} onClick={() => openDesign("schematic")} />
            {/* PCB */}
            <Group depth={1} icon={<CircuitBoard size={14} />} label="PCB" count={vm.placements.length} defaultOpen onClick={() => openDesign("pcb")}>
              <Row depth={2} icon={<Box size={14} />} label={`Board · ${vm.board ? `${vm.board.width.magnitude}×${vm.board.height.magnitude}mm` : "—"}`} count={vm.board?.stack.layers.length} selected={isDoc("pcb")} onClick={() => openDesign("pcb")} />
            </Group>
            {/* Manufacturing */}
            <Row depth={1} icon={<Layers size={14} />} label="Manufacturing" count={vm.released ? 1 : 0} selected={isDoc("ir")} onClick={() => openDesign("ir")} />
            {/* Libraries */}
            <Group depth={1} icon={<Package size={14} />} label="Libraries" count={vm.parts.length} onClick={() => setPanel("library")}>
              <Row depth={2} icon={<FileCode2 size={14} />} label="Symbols" count={vm.components.length} onClick={() => openDesign("schematic")} />
              <Row depth={2} icon={<CircuitBoard size={14} />} label="Footprints" count={vm.placements.length} onClick={() => openDesign("pcb")} />
              <Row depth={2} icon={<Package size={14} />} label="Parts" count={vm.parts.length} onClick={() => setPanel("library")} />
            </Group>
            {/* BOM */}
            <Row depth={1} icon={<ListChecks size={14} />} label="BOM" count={vm.parts.length} onClick={() => openDoc("bom/bom.md")} />
            {/* Constraints */}
            <Row depth={1} icon={<Boxes size={14} />} label="Constraints" count={vm.constraints.length} onClick={() => openDoc("docs/03-constraints.md")} />
            {/* Verification */}
            <Group depth={1} icon={<ShieldCheck size={14} />} label="Verification" count={vm.phases.length} onClick={() => store.setBottom("problems")}>
              <Row depth={2} icon={<ListChecks size={14} />} label="Open findings" count={openOpen.length} onClick={() => store.setBottom("problems")} />
              <Row depth={2} icon={<FileText size={14} />} label="Checklist" onClick={() => openDoc("verification/checklist.md")} />
            </Group>
            {/* Documentation */}
            <Group depth={1} icon={<FileText size={14} />} label="Documentation" count={4} onClick={() => openDoc("docs/00-prd.md")}>
              <Row depth={2} icon={<FileText size={14} />} label="PRD" onClick={() => openDoc("docs/00-prd.md")} />
              <Row depth={2} icon={<FileText size={14} />} label="Requirements" onClick={() => openDoc("docs/01-requirements.md")} />
              <Row depth={2} icon={<FileText size={14} />} label="Architecture" onClick={() => openDoc("docs/02-architecture.md")} />
              <Row depth={2} icon={<FileText size={14} />} label="Decisions" onClick={() => openDoc("decisions/decisions.md")} />
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

          <Group depth={0} icon={<Package size={14} />} label="Parts" count={vm.parts.length}>
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