import { useStore } from "../../store/useWorkspaceStore";
import { quantity, shortId } from "../util";

/** The raw owned model — a compact projection of exactly what the kernel holds. */
export function IrView() {
  const { vm } = useStore();
  const ir = {
    intent: vm.intent?.statement,
    requirements: vm.requirements.map((r) => ({ id: shortId(r.id), statement: r.statement, category: r.category })),
    blocks: vm.blocks.map((b) => ({ id: shortId(b.id), name: b.name })),
    nets: vm.nets.map((n) => ({ name: n.name, class: n.class, members: n.members.length })),
    parts: vm.parts.map((p) => ({ mpn: p.mpn, mfr: p.manufacturer, lifecycle: p.lifecycle })),
    board: vm.board
      ? { size: `${quantity(vm.board.width)} × ${quantity(vm.board.height)}`, layers: vm.board.stack.layers.length }
      : undefined,
    placements: vm.placements.length,
    tracks: vm.tracks.length,
    released: vm.released,
  };
  return (
    <pre style={{ position: "absolute", inset: 0, overflow: "auto", margin: 0, padding: 16, fontFamily: "var(--font-mono)", fontSize: 12, lineHeight: "18px", color: "var(--text-secondary)", whiteSpace: "pre" }}>
      {JSON.stringify(ir, null, 2)}
    </pre>
  );
}
