import { create } from "zustand";
import type { EntityId, EventRecord } from "../contract/v1";
import type { EventSource } from "../events/EventSource";
import { emptyViewModel, fold, gateOf, type Gate, type ViewModel } from "./fold";
import type { LayerKey } from "./selectors";

export type DesignDoc = "schematic" | "pcb" | "3d" | "ir";
export type PanelView = "explorer" | "search" | "agent" | "design" | "verify" | "revisions" | "library" | "settings";
export type BottomTab = "problems" | "tasks" | "agent" | "drc" | "log" | "find";
export type Tool = "select" | "pan" | "measure";
export type DocViewMode = "preview" | "source";

/** An open editor tab — a Markdown doc or a design view, exactly like VS Code opens .md next to code. */
export type Tab = { kind: "doc"; path: string } | { kind: "design"; doc: DesignDoc };

export interface Camera {
  x: number;
  y: number;
  zoom: number;
}

const ALL_LAYERS: LayerKey[] = ["topCopper", "bottomCopper", "tracks", "silk", "ratline", "outline", "drill"];
function defaultLayers(): Record<LayerKey, boolean> {
  return Object.fromEntries(ALL_LAYERS.map((k) => [k, true])) as Record<LayerKey, boolean>;
}

function sameTab(a: Tab, b: Tab): boolean {
  return a.kind === b.kind && (a.kind === "doc" ? a.path === (b as { path: string }).path : a.doc === (b as { doc: DesignDoc }).doc);
}

interface WorkspaceStore {
  vm: ViewModel;
  gate: Gate;
  source?: EventSource;
  mode: EventSource["mode"];

  // ide ui state
  activePanel: PanelView;
  tabs: Tab[];
  activeTab: number;
  docView: DocViewMode;
  bottomTab: BottomTab;
  showSidebar: boolean;
  showRightDock: boolean;
  showBottom: boolean;
  paletteOpen: boolean;
  selected?: EntityId;
  layers: Record<LayerKey, boolean>;
  camera: Camera;
  tool: Tool;
  cursorMm?: { x: number; y: number };
  fitRequest: number;
  playing: boolean;

  // wiring
  connect: (source: EventSource) => void;
  ingest: (record: EventRecord) => void;

  // tabs
  openDoc: (path: string) => void;
  openDesign: (doc: DesignDoc) => void;
  setActiveTab: (i: number) => void;
  closeTab: (i: number) => void;
  setDocView: (m: DocViewMode) => void;

  // ui
  setPanel: (v: PanelView) => void;
  setBottom: (t: BottomTab) => void;
  toggle: (p: "showSidebar" | "showRightDock" | "showBottom") => void;
  togglePalette: (open?: boolean) => void;
  select: (id?: EntityId) => void;
  toggleLayer: (k: LayerKey) => void;
  setCamera: (c: Partial<Camera>) => void;
  setTool: (t: Tool) => void;
  setCursorMm: (p?: { x: number; y: number }) => void;
  requestFit: () => void;

  // playback
  play: () => void;
  pause: () => void;
  restart: () => void;
  skipToEnd: () => void;
}

export const useStore = create<WorkspaceStore>((set, get) => ({
  vm: emptyViewModel(),
  gate: { released: false, reason: "in progress" },
  mode: "CASSETTE",

  activePanel: "explorer",
  tabs: [{ kind: "doc", path: "docs/00-prd.md" }],
  activeTab: 0,
  docView: "preview",
  bottomTab: "problems",
  showSidebar: true,
  showRightDock: true,
  showBottom: true,
  paletteOpen: false,
  layers: defaultLayers(),
  camera: { x: 0, y: 0, zoom: 8 },
  tool: "select",
  fitRequest: 0,
  playing: true,

  connect: (source) => {
    if (get().source) return;
    source.subscribe((record) => get().ingest(record));
    set({ source, mode: source.mode });
    source.start();
  },
  ingest: (record) =>
    set((s) => {
      const vm = fold(s.vm, record);
      return { vm, gate: gateOf(vm) };
    }),

  openDoc: (path) =>
    set((s) => {
      const tab: Tab = { kind: "doc", path };
      const i = s.tabs.findIndex((t) => sameTab(t, tab));
      if (i >= 0) return { activeTab: i };
      return { tabs: [...s.tabs, tab], activeTab: s.tabs.length };
    }),
  openDesign: (doc) =>
    set((s) => {
      const tab: Tab = { kind: "design", doc };
      const i = s.tabs.findIndex((t) => sameTab(t, tab));
      if (i >= 0) return { activeTab: i };
      return { tabs: [...s.tabs, tab], activeTab: s.tabs.length };
    }),
  setActiveTab: (activeTab) => set({ activeTab }),
  closeTab: (i) =>
    set((s) => {
      const tabs = s.tabs.filter((_, idx) => idx !== i);
      let activeTab = s.activeTab;
      if (i < activeTab) activeTab -= 1;
      if (activeTab >= tabs.length) activeTab = tabs.length - 1;
      return { tabs, activeTab: Math.max(0, activeTab) };
    }),
  setDocView: (docView) => set({ docView }),

  setPanel: (activePanel) => set((s) => ({ activePanel, showSidebar: s.activePanel === activePanel ? !s.showSidebar : true })),
  setBottom: (bottomTab) => set({ bottomTab, showBottom: true }),
  toggle: (p) => set((s) => ({ [p]: !s[p] }) as Pick<WorkspaceStore, typeof p>),
  togglePalette: (open) => set((s) => ({ paletteOpen: open ?? !s.paletteOpen })),
  select: (selected) => set({ selected }),
  toggleLayer: (k) => set((s) => ({ layers: { ...s.layers, [k]: !s.layers[k] } })),
  setCamera: (c) => set((s) => ({ camera: { ...s.camera, ...c } })),
  setTool: (tool) => set({ tool }),
  setCursorMm: (cursorMm) => set({ cursorMm }),
  requestFit: () => set((s) => ({ fitRequest: s.fitRequest + 1 })),

  play: () => { get().source?.pause?.(); get().source?.start?.(); set({ playing: true }); },
  pause: () => { get().source?.pause?.(); set({ playing: false }); },
  restart: () => {
    set({ vm: emptyViewModel(), gate: { released: false, reason: "in progress" }, selected: undefined });
    get().source?.restart?.();
    set({ playing: true });
  },
  skipToEnd: () => { get().source?.skipToEnd?.(); set({ playing: false }); },
}));

/** The active tab, or undefined if none open. */
export function activeTabOf(s: Pick<WorkspaceStore, "tabs" | "activeTab">): Tab | undefined {
  return s.tabs[s.activeTab];
}
