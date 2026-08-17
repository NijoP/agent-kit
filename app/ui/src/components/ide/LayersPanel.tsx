import { Eye, EyeOff, Layers as LayersIcon } from "lucide-react";
import { useStore } from "../../store/useWorkspaceStore";
import { layerModel } from "../../store/selectors";
import { quantity } from "../util";

/**
 * The EasyEDA-style layers panel — a layer list over the real stack-up: the top/bottom copper and
 * the owned laminate/core facts from `board.stack`. Every layer row toggles its canvas projection.
 */
export function LayersPanel() {
  const { vm, layers, toggleLayer, selected } = useStore();
  const model = layerModel(vm);
  const board = vm.board;

  // Active copper layer = the side of the currently selected component's placement, else Top.
  const activeSide = (() => {
    if (selected) {
      const pl = vm.placements.find((p) => p.component === selected);
      if (pl) return pl.side;
    }
    return "Top";
  })();

  return (
    <div className="dock-section layers">
      <div className="dock-head"><LayersIcon size={12} strokeWidth={1.75} /> Layers</div>
      <div className="dock-body">
        {board && board.stack.layers.length > 0 && (
          <div className="stackup">
            <div className="stackup-head">Stack-up · {board.stack.layers.length} layers</div>
            {board.stack.layers.map((l, i) => (
              <div key={i}>
                <div className={`stackup-row ${i === (activeSide === "Top" ? 0 : board.stack.layers.length - 1) ? "activ-layer" : ""}`}>
                  <span className="swatch" style={{ background: i === 0 ? "var(--ecad-copper-top)" : i === board.stack.layers.length - 1 ? "var(--ecad-copper-bottom)" : "var(--ecad-stack-substrate)" }} />
                  <span className="slab">{l.role} · {i === 0 ? "Top" : i === board.stack.layers.length - 1 ? "Bottom" : `Layer ${i + 1}`}</span>
                  <span className="sval">{quantity(l.copper_thickness)} Cu · Er {l.dielectric_er}</span>
                </div>
                {i < board.stack.layers.length - 1 && (
                  <div className="stackup-sub">{quantity(l.dielectric_height)} laminate · tan δ {l.loss_tangent}</div>
                )}
              </div>
            ))}
            {board.stack.layers.length > 0 && (
              <div className="stackup-row">
                <span className="swatch" style={{ background: "var(--ecad-stack-substrate)" }} />
                <span className="slab">Total</span>
                <span className="sval">{board.width.magnitude} × {board.height.magnitude} mm</span>
              </div>
            )}
          </div>
        )}
        {model.map((l) => {
          const on = layers[l.key];
          const isActive = l.side === activeSide;
          return (
            <div className={`layer-row ${on ? "" : "off"}`} key={l.key} onClick={() => toggleLayer(l.key)}>
              <span className="layer-swatch" style={{ background: l.color, boxShadow: isActive ? `0 0 0 1px var(--accent)` : undefined }} />
              <span className="lname">{l.name}{isActive && <span style={{ color: "var(--accent)", marginLeft: 6 }}>●</span>}</span>
              <span className="lcount">{l.count}</span>
              <span className="eye">{on ? <Eye size={14} strokeWidth={1.5} /> : <EyeOff size={14} strokeWidth={1.5} />}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}