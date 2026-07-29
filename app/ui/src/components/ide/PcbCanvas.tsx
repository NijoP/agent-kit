import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { CircuitBoard } from "lucide-react";
import { useStore } from "../../store/useWorkspaceStore";
import { pads, ratlines, boardBounds } from "../../store/selectors";

interface Size {
  w: number;
  h: number;
}

/** Pick a ruler tick step (mm) so on-screen spacing stays ~45–90px. */
function tickStep(zoom: number): number {
  const candidates = [0.5, 1, 2, 5, 10, 20, 50, 100];
  for (const s of candidates) if (s * zoom >= 45) return s;
  return 100;
}

export function PcbCanvas() {
  const store = useStore();
  const { vm, camera, layers, tool, selected, setCamera, setCursorMm, select, fitRequest } = store;
  const vpRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<Size>({ w: 0, h: 0 });
  const panning = useRef<{ x: number; y: number } | null>(null);

  // measure the viewport
  useLayoutEffect(() => {
    const el = vpRef.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setSize({ w: el.clientWidth, h: el.clientHeight }));
    ro.observe(el);
    setSize({ w: el.clientWidth, h: el.clientHeight });
    return () => ro.disconnect();
  }, []);

  const bounds = boardBounds(vm);

  // fit-to-board on first data, on resize-from-zero, and on explicit fit request
  const lastFit = useRef(-1);
  useEffect(() => {
    if (!bounds || size.w === 0 || size.h === 0) return;
    if (lastFit.current === fitRequest && camera.zoom !== 8) return; // only auto-fit once unless requested
    const pad = 0.88;
    const zoom = Math.min(size.w / bounds.w, size.h / bounds.h) * pad;
    setCamera({ zoom, x: (size.w - bounds.w * zoom) / 2, y: (size.h - bounds.h * zoom) / 2 });
    lastFit.current = fitRequest;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bounds?.w, bounds?.h, size.w, size.h, fitRequest]);

  const localXY = (e: React.PointerEvent | React.WheelEvent) => {
    const r = vpRef.current!.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };
  const toMm = (sx: number, sy: number) => ({
    x: (sx - camera.x) / camera.zoom,
    y: (sy - camera.y) / camera.zoom,
  });

  const onWheel = (e: React.WheelEvent) => {
    const { x: sx, y: sy } = localXY(e);
    const world = toMm(sx, sy);
    const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
    const zoom = Math.max(1, Math.min(200, camera.zoom * factor));
    setCamera({ zoom, x: sx - world.x * zoom, y: sy - world.y * zoom });
  };
  const onPointerDown = (e: React.PointerEvent) => {
    if (tool === "pan" || e.button === 1 || e.buttons === 4) {
      panning.current = { x: e.clientX, y: e.clientY };
      vpRef.current?.setPointerCapture(e.pointerId);
    }
  };
  const onPointerMove = (e: React.PointerEvent) => {
    const { x: sx, y: sy } = localXY(e);
    setCursorMm(toMm(sx, sy));
    if (panning.current) {
      const dx = e.clientX - panning.current.x;
      const dy = e.clientY - panning.current.y;
      panning.current = { x: e.clientX, y: e.clientY };
      setCamera({ x: camera.x + dx, y: camera.y + dy });
    }
  };
  const onPointerUp = (e: React.PointerEvent) => {
    panning.current = null;
    vpRef.current?.releasePointerCapture?.(e.pointerId);
  };

  if (!bounds) {
    return (
      <div className="pcb-empty">
        <CircuitBoard size={22} strokeWidth={1.2} style={{ opacity: 0.5 }} />
        <div>The board draws here as placement &amp; routing commit.</div>
      </div>
    );
  }

  const step = tickStep(camera.zoom);
  const majorEvery = 5; // every N ticks is a labelled major
  const ticks: number[] = [];
  for (let m = 0; m <= Math.max(bounds.w, bounds.h) + step; m += step) ticks.push(Number(m.toFixed(3)));
  const g = `translate(${camera.x},${camera.y}) scale(${camera.zoom})`;

  const padList = layers.topCopper || layers.bottomCopper || layers.silk || layers.drill ? pads(vm) : [];
  const rats = layers.ratline ? ratlines(vm) : [];
  const drcTargets = new Set(vm.violations.filter((v) => v.status === "Open").flatMap((v) => v.subjects));

  return (
    <div className={`pcb ${tool === "pan" ? "pan" : ""} ${panning.current ? "panning" : ""}`}>
      <div className="ruler-corner" />
      {/* top ruler */}
      <div className="ruler top">
        <svg width="100%" height="100%">
          {ticks.map((m, i) => {
            const x = m * camera.zoom + camera.x;
            if (x < 0 || x > size.w) return null;
            const major = i % majorEvery === 0;
            return (
              <g key={m}>
                <line className="ruler-tick" x1={x} y1={major ? 8 : 14} x2={x} y2={22} />
                {major && <text className="ruler-label" x={x + 2} y={9}>{m}</text>}
              </g>
            );
          })}
        </svg>
      </div>
      {/* left ruler */}
      <div className="ruler left">
        <svg width="100%" height="100%">
          {ticks.map((m, i) => {
            const y = m * camera.zoom + camera.y;
            if (y < 0 || y > size.h) return null;
            const major = i % majorEvery === 0;
            return (
              <g key={m}>
                <line className="ruler-tick" x1={major ? 8 : 14} y1={y} x2={22} y2={y} />
                {major && <text className="ruler-label" x={2} y={y - 2}>{m}</text>}
              </g>
            );
          })}
        </svg>
      </div>

      {/* board viewport */}
      <div
        className="pcb-viewport"
        ref={vpRef}
        onWheel={onWheel}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={() => setCursorMm(undefined)}
      >
        <svg className="pcb-svg">
          <g transform={g}>
            {/* board outline */}
            {layers.outline && <rect className="brd-outline" x={0} y={0} width={bounds.w} height={bounds.h} rx={0.6} />}
            {/* grid */}
            {ticks.map((m) => (
              <g key={`gr${m}`}>
                {m <= bounds.w && <line className="brd-grid" x1={m} y1={0} x2={m} y2={bounds.h} />}
                {m <= bounds.h && <line className="brd-grid" x1={0} y1={m} x2={bounds.w} y2={m} />}
              </g>
            ))}

            {/* ratlines (under copper) */}
            {rats.map((r, i) => (
              <line key={`rt${i}`} className="ratline" x1={r.x1} y1={r.y1} x2={r.x2} y2={r.y2} />
            ))}

            {/* routed tracks */}
            {layers.tracks &&
              vm.tracks.map((t) => (
                <line
                  key={t.id}
                  className={`track ${t.layer === "Bottom" ? "bottom" : "top"} ${selected === t.net ? "sel" : ""}`}
                  x1={t.x1.magnitude}
                  y1={t.y1.magnitude}
                  x2={t.x2.magnitude}
                  y2={t.y2.magnitude}
                  strokeWidth={Math.max(0.12, t.width.magnitude)}
                />
              ))}

            {/* copper pads (footprints) */}
            {padList
              .filter((p) => (p.side === "Top" ? layers.topCopper : layers.bottomCopper))
              .map((p) => {
                const sel = selected === p.component;
                return (
                  <g key={p.id} onClick={(e) => { e.stopPropagation(); select(p.component); }}>
                    <rect
                      className={`pad ${p.side === "Bottom" ? "bottom" : "top"} ${sel ? "sel" : ""}`}
                      x={p.x}
                      y={p.y}
                      width={p.w}
                      height={p.h}
                      rx={0.3}
                    />
                    {layers.drill && (
                      <circle className="pad-hole" cx={p.cx} cy={p.cy} r={Math.min(p.w, p.h) * 0.12} />
                    )}
                    {layers.silk && (
                      <text className="silk" x={p.cx} y={p.cy + 0.6} fontSize={Math.min(2.4, p.h * 0.3)}>
                        {p.refdes}
                      </text>
                    )}
                    {drcTargets.has(p.component) && (
                      <rect className="drc-marker" x={p.x - 0.4} y={p.y - 0.4} width={p.w + 0.8} height={p.h + 0.8} />
                    )}
                  </g>
                );
              })}
          </g>
        </svg>
      </div>
    </div>
  );
}
