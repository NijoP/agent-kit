# UI Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines the UI as a **projection/editor surface, not authority**. The Tauri desktop app renders the kernel's live event stream and provides editing affordances that propose changes via kernel capability seams.

---

## Design Principles

1. **UI is a Projection** — Reads from kernel state (via `get_state` or live `EventSink`). Never writes directly.
2. **Edits are Proposals** — User interactions (drag component, draw wire, change property) → `CapabilityInvocation` → kernel validates → commits.
3. **Live Event Stream** — UI subscribes to `EventSink` for real-time updates (event-driven, not polling).
4. **Offline-First** — All kernel operations local; UI works air-gapped.
5. **Multi-Pane Terminal** — Integrated via Herdr for agent logs, verification output, shell access.
6. **KiCad as Editor** — Schematic/PCB editing primarily in KiCad; UI provides project overview, verification dashboard, component palette.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      TAURI APP (app/)                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Frontend   │  │  IPC Bridge │  │  Herdr      │             │
│  │  (React/TS) │◄─►│  (Tauri)    │◄─►│  Integration│             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│         │                │                │                     │
│         ▼                ▼                ▼                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    EAK KERNEL (eak-cli)                 │   │
│  │  (linked as library, runs in-process)                   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Frontend Stack (`app/ui/`)

| Technology | Version | Purpose |
|------------|---------|---------|
| React | 18 | Component framework |
| TypeScript | 5 | Type safety |
| Vite | 5 | Build tool |
| xterm.js | 5 | Terminal emulation (Herdr panes) |
| React Flow | 11 | Graph visualization (schematic/PCB topology, provenance) |
| Tailwind CSS | 3 | Styling |
| Zustand | 4 | Client state (UI only, not engineering state) |
| TanStack Query | 5 | Server state (kernel queries) |
| Tauri API | 2 | IPC, filesystem, window management |

---

## IPC Commands (Tauri)

Defined in `app/src-tauri/src/main.rs` and exposed to frontend via `invoke()`.

| Command | Description | Returns |
|---------|-------------|---------|
| `start_run` | Launch full pipeline or single phase | `EventStream` (async generator) |
| `get_state` | Query canonical model at snapshot | `EngineeringModel` (serialized) |
| `propose_capability` | Submit capability invocation | `ProposalResult` (accepted/rejected) |
| `sync_kicad` | Import/export/render/watch KiCad project | `KicadSyncResult` |
| `list_projects` | List all projects | `Vec<ProjectSummary>` |
| `create_project` | Create new project | `Project` |
| `open_project` | Load project, start Herdr session | `SessionInfo` |
| `attach_terminal` | Get PTY info for Herdr pane | `AttachInfo` |
| `export_manufacturing` | Generate BOM, Gerber, etc. | `ManufacturingPackage` |

---

## Frontend State Management

### Kernel State (Authoritative)
- **Never stored in frontend**. Always fetched via `get_state` or streamed via `EventSink`.
- Frontend caches last known state for instant render; invalidated by `EventRecord` stream.

### UI State (Ephemeral)
- Panel layouts, selected tabs, filter settings, zoom levels.
- Stored in `Zustand` + persisted to `localStorage`.

### Agent State (Observed)
- Agent status (running, waiting, error) from `AgentMetadata` events.
- Displayed in agent panel; not editable.

---

## UI Layout (Desktop)

```
┌─────────────────────────────────────────────────────────────────┐
│  Menu Bar: Project ▼  Run ▼  View ▼  Tools ▼  Help ▼           │
├──────────────┬────────────────────────────────────┬────────────┤
│              │                                    │            │
│  PROJECT     │         MAIN CONTENT AREA          │  AGENT     │
│  EXPLORER    │                                    │  PANEL     │
│              │  ┌────────────────────────────┐   │            │
│  - Projects  │  │  SCHEMATIC / PCB /         │   │  - Lead    │
│  - Components│  │  VERIFICATION / BOM        │   │  - Research│
│  - Evidence  │  │  (React Flow canvas)       │   │  - Comp    │
│  - Verification           (read-only)        │   │  - Sch     │
│  - Manufacturing         (KiCad for edit)    │   │  - PCB     │
│              │  └────────────────────────────┘   │  - Verify  │
│              │                                    │  - Mfg     │
│              │  ┌────────────────────────────┐   │            │
│              │  │  TERMINAL (Herdr panes)    │   │            │
│              │  │  [Requirements] [Schematic]│   │            │
│              │  │  [PCB] [Verification]      │   │            │
│              │  └────────────────────────────┘   │            │
└──────────────┴────────────────────────────────────┴────────────┘
```

### Panels

1. **Project Explorer** — Tree view: Project → Revisions → Requirements, Components, Schematic, PCB, Verification, Manufacturing. Click → loads model slice into main area.

2. **Main Content Area** — Tabbed:
   - **Schematic View** — React Flow graph of components/nets (read-only). Double-click → opens KiCad.
   - **PCB View** — React Flow placement/routing graph (read-only). Double-click → opens KiCad.
   - **Verification Dashboard** — Tables of checks, confidence meters, fidelity badges.
   - **BOM Table** — Sortable, filterable, exportable.
   - **Provenance Graph** — Interactive graph of fact lineage (requirement → component → net → verification).

3. **Agent Panel** — Live status of each agent (spinner, last proposal, retry count). Click → attaches to Herdr pane for that agent.

4. **Terminal (Herdr)** — Tabbed panes per workspace (phase). xterm.js instances connected to Herdr PTY sockets.

---

## Event Stream Handling

```typescript
// Frontend event stream subscription
const eventStream = await invoke<EventStream>('start_run', { intent, config });

for await (const event of eventStream) {
  // Update kernel state cache
  kernelStateCache.apply(event);
  
  // Trigger UI updates
  if (event.isProvenanceEvent()) {
    provenanceGraph.add(event);
  }
  if (event.isVerificationEvent()) {
    verificationDashboard.update(event);
  }
  if (event.isAgentStatusEvent()) {
    agentPanel.updateAgent(event);
  }
}
```

### Event Types for UI
| Event | UI Action |
|-------|-----------|
| `RequirementAdded` | Add to requirements tree |
| `ComponentSelected` | Add to component palette |
| `SchematicElementAdded` | Update schematic graph |
| `PcbElementPlaced` | Update PCB graph |
| `VerificationRun` | Update verification dashboard |
| `AgentStatusChanged` | Update agent panel badge |
| `ApprovalGateOpened` | Show approval toast/modal |
| `KicadSyncCompleted` | Refresh schematic/PCB views |

---

## Editing Affordances (Proposals)

| User Action | Capability | Validation |
|-------------|------------|------------|
| Drag component from palette to schematic | `SchematicEdit { AddComponentInstance }` | ERC, footprint exists |
| Draw wire between pins | `SchematicEdit { AddWire }` | Connectivity, no shorts |
| Change component value/footprint | `SchematicEdit { UpdateProperties }` | Component exists, footprint matches |
| Move component on PCB | `PcbEdit { MovePlacement }` | DRC, keepout zones |
| Route trace | `PcbEdit { AddRoute }` | DRC, impedance, length match |
| Change stackup | `PcbEdit { UpdateStackup }` | Layer count, impedance calc |
| Approve component | `ComponentApprove` | All validation gates passed |
| Request verification | `VerificationRequest` | Model ready |

**All edits go through `propose_capability` IPC.** Frontend shows optimistic UI (local prediction) but reverts on kernel rejection.

---

## KiCad Integration in UI

- **"Open in KiCad" Button** — Launches KiCad with project file. Sets env vars for EAK libraries.
- **"Sync from KiCad" Button** — Triggers `sync_kicad { action: "import" }`.
- **"Render" Button** — Triggers `sync_kicad { action: "render", kind: "SchematicPdf" }` → opens PDF in system viewer.
- **File Watcher** — Background `sync_kicad { action: "watch" }` streams changes → proposes edits.

---

## Theming & Accessibility

- **Themes**: Light, Dark, High Contrast (follows system).
- **Color Blind Safe**: Verification status uses icons + colors.
- **Keyboard Navigation**: Full tab order, shortcuts for common actions.
- **Screen Reader**: ARIA labels on all interactive elements.

---

## Performance Targets

| Metric | Target |
|--------|--------|
| Initial load (cold) | < 2s |
| Event stream latency (kernel → UI) | < 50ms |
| Schematic graph render (500 components) | < 500ms |
| PCB graph render (1000 routes) | < 1s |
| Terminal keystroke latency | < 10ms |

---

## Next Document

[Verification Architecture →](../11-verification-architecture/README.md)