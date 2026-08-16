import { Library, PackageSearch, Search as SearchIcon, ScanLine, CircuitBoard } from "lucide-react";
import { useMemo, useState } from "react";
import { useStore } from "../../store/useWorkspaceStore";
import { componentsForPart, bomLineForPart } from "../../store/selectors";
import { Pill } from "../Badge";

/**
 * Engineering search over the owned parts (USER-MANUAL §10). The catalog is a projection of
 * committed `Part` facts — every row here is a real, orderable entity bound to the components it
 * realizes. Catalog ingestion beyond the current design is ◐ planned; nothing is fabricated.
 */
export function LibraryPanel() {
  const { vm, select, selected } = useStore();
  const [query, setQuery] = useState("");
  const [manufacturer, setManufacturer] = useState<string>("all");
  const [lifecycle, setLifecycle] = useState<string>("all");
  const [klass, setKlass] = useState<string>("all");

  const manufacturers = useMemo(
    () => [...new Set(vm.parts.map((p) => p.manufacturer))].sort(),
    [vm.parts],
  );
  const lifecycles = useMemo(
    () => [...new Set(vm.parts.map((p) => p.lifecycle))].sort(),
    [vm.parts],
  );
  const classes = useMemo(() => {
    const out = new Set<string>();
    for (const p of vm.parts) for (const c of componentsForPart(vm, p.id)) out.add(c.class);
    return [...out].sort();
  }, [vm.parts, vm.components, vm.bomLines]);

  const results = useMemo(() => {
    const q = query.trim().toLowerCase();
    return vm.parts.filter((p) => {
      if (manufacturer !== "all" && p.manufacturer !== manufacturer) return false;
      if (lifecycle !== "all" && p.lifecycle !== lifecycle) return false;
      if (q) {
        const comps = componentsForPart(vm, p.id).map((c) => c.refdes).join(" ");
        const hay = `${p.mpn} ${p.manufacturer} ${comps}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      if (klass !== "all") {
        if (!componentsForPart(vm, p.id).some((c) => c.class === klass)) return false;
      }
      return true;
    });
  }, [vm, query, manufacturer, lifecycle, klass]);

  return (
    <section className="ide-sidebar" aria-label="Library">
      <div className="sidebar-head">
        Library
        <span className="head-actions"><PackageSearch size={13} strokeWidth={1.5} /></span>
      </div>
      <div className="sidebar-body">
        <div style={{ padding: 12, display: "flex", flexDirection: "column", gap: 8 }}>
          <div className="agent-input" style={{ margin: 0 }}>
            <SearchIcon size={14} strokeWidth={1.5} style={{ color: "var(--text-muted)" }} />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search MPN, manufacturer, refdes…"
              aria-label="Search parts"
            />
          </div>
          <div style={{ display: "flex", gap: 6 }}>
            <select className="lib-select" value={manufacturer} onChange={(e) => setManufacturer(e.target.value)} aria-label="Manufacturer">
              <option value="all">Manufacturer</option>
              {manufacturers.map((m) => <option key={m} value={m}>{m}</option>)}
            </select>
            <select className="lib-select" value={lifecycle} onChange={(e) => setLifecycle(e.target.value)} aria-label="Lifecycle">
              <option value="all">Lifecycle</option>
              {lifecycles.map((m) => <option key={m} value={m}>{m}</option>)}
            </select>
            <select className="lib-select" value={klass} onChange={(e) => setKlass(e.target.value)} aria-label="Component class">
              <option value="all">Class</option>
              {classes.map((m) => <option key={m} value={m}>{m}</option>)}
            </select>
          </div>
        </div>

        <div className="lib-stats">
          <div className="lib-stat"><ScanLine size={13} strokeWidth={1.5} /> {vm.components.length} symbols</div>
          <div className="lib-stat"><CircuitBoard size={13} strokeWidth={1.5} /> {vm.placements.length} footprints</div>
          <div className="lib-stat"><Library size={13} strokeWidth={1.5} /> {vm.parts.length} parts</div>
        </div>

        <div className="tree" style={{ paddingTop: 0 }}>
          {results.length === 0 && (
            <div className="empty" style={{ height: 160 }}>
              {vm.parts.length === 0
                ? <>
                    <Library size={20} strokeWidth={1.5} />
                    <div>No parts sourced yet. Parts land here the moment the agent orders them through the BOM seam.</div>
                    <span className="mono" style={{ color: "var(--scaffold)", fontSize: 11 }}>catalog ingestion ◐ planned</span>
                  </>
                : "No parts match those filters."}
            </div>
          )}
          {results.map((p) => {
            const comps = componentsForPart(vm, p.id);
            const line = bomLineForPart(vm, p.id);
            const lifecycleTone = p.lifecycle === "Active" ? "pass" : p.lifecycle === "Nrnd" ? "warn" : "error";
            return (
              <button key={p.id} className={`lib-card ${selected === p.id ? "sel" : ""}`} onClick={() => select(p.id)}>
                <span className="lib-card-row">
                  <span className="lib-part-main">
                    <span className="lib-mpn">{p.mpn}</span>
                    <span className="lib-mfr">{p.manufacturer}</span>
                  </span>
                  <span className="lib-part-side">
                    <Pill tone={lifecycleTone}>{p.lifecycle}</Pill>
                    <span className="lib-qty">{line?.quantity ?? 0}×</span>
                  </span>
                </span>
                <span className="lib-refdeses">{comps.length ? comps.map((c) => c.refdes).join(" · ") : "not yet placed"}</span>
              </button>
            );
          })}
        </div>
      </div>
    </section>
  );
}