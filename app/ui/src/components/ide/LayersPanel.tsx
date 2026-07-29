import { Eye, EyeOff } from "lucide-react";
import { useStore } from "../../store/useWorkspaceStore";
import { layerModel } from "../../store/selectors";

export function LayersPanel() {
  const { vm, layers, toggleLayer } = useStore();
  const model = layerModel(vm);
  return (
    <div className="dock-section layers">
      <div className="dock-head">Layers</div>
      <div className="dock-body">
        {model.map((l) => {
          const on = layers[l.key];
          return (
            <div className={`layer-row ${on ? "" : "off"}`} key={l.key} onClick={() => toggleLayer(l.key)}>
              <span className="layer-swatch" style={{ background: l.color }} />
              <span className="lname">{l.name}</span>
              <span className="lcount">{l.count}</span>
              <span className="eye">{on ? <Eye size={14} strokeWidth={1.5} /> : <EyeOff size={14} strokeWidth={1.5} />}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
