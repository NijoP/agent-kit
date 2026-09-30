# eak-app — Electronics Agent Kit Desktop IDE

The native, local-first shell for the Electronics Agent Kit (EAK). A **Tauri** app whose **backend is the EAK Rust kernel** (`../eak/`) and whose frontend renders the kernel's **live event stream**. This directory is the *spine* per the [Master Implementation Roadmap](../docs/16-master-implementation-roadmap/) (Stage 1: Foundation Product).

> **Why it lives outside `eak/`:** it depends on Tauri (webkit2gtk etc.), which the CI/sandbox can't build, so it is deliberately **not** a member of the `eak/` Cargo workspace — the kernel build stays green and independent. You build/run this on your machine, where the Tauri prerequisites exist.

## What the Spine Gives You

- `eak_ports::EventSink` — a live observer invoked once per event on the kernel's single commit path (`RuntimeCore::commit`), right after append+fold. This is the seam the UI subscribes to.
- `eak_cli::run_with_sink(reasoning, &cfg, Some(sink))` — runs the full pipeline and streams every `EventRecord` (`{seq, timestamp, event}`, already `Serialize`) to your sink as it happens. `None` is byte-identical to `run_with` (determinism/replay preserved).

The desktop app is a thin bridge: **a sink that forwards each `EventRecord` to the webview as a Tauri event**, and a frontend that renders the resulting live engineering-state feed.

## Prerequisites (on your machine)

- Rust toolchain (same as the kernel).
- Tauri v2 system deps — see https://v2.tauri.app/start/prerequisites/ (Linux: `webkit2gtk`, `libgtk`, etc.).
- Node + a package manager (for the frontend dev server / bundling), or serve `ui/` statically.
- Tauri CLI: `cargo install tauri-cli` (or `npm i -D @tauri-apps/cli`).

## Getting It Running (Recommended Path)

The fastest correct route is to let Tauri generate the standard shell, then drop in the two EAK-specific pieces already here:

1. Generate a Tauri v2 scaffold in this folder (or merge into it):
   `npm create tauri-app@latest` → choose vanilla/TS, frontend dir `ui/`.
2. Replace the generated `src-tauri/src/main.rs` with [`src-tauri/src/main.rs`](src-tauri/src/main.rs) here (the `EventSink`→Tauri-event bridge + the `start_run` command).
3. Add the kernel path-dependencies from [`src-tauri/Cargo.toml`](src-tauri/Cargo.toml) here to the generated `Cargo.toml`.
4. Use the provided [`tauri.conf.json`](src-tauri/tauri.conf.json) as a reference for EAK branding (productName, identifier, bundle metadata).
5. Use [`ui/index.html`](ui/index.html) as the frontend (listens for `eak://event` and renders the live feed).
6. `cargo tauri dev` → a native window opens; click **Run** → watch a real pipeline run stream in.

> The generated `tauri.conf.json` / `build.rs` from step 1 are version-specific. The **custom** code — the bridge and the UI — is what's provided in this folder. The `tauri.conf.json` here serves as the canonical branding reference.

## Milestone This Proves (Stage 1 / Gate 1)

> *"The kernel is the native core of a desktop app, and one real pipeline run streams live into a native window."* — the Spine exit criterion. Everything else (schematic/PCB canvas, agent panel, KiCad import) hangs off this bridge.

## Architecture

See [UI Architecture](../docs/10-ui-architecture/) and [Terminal/Herdr Architecture](../docs/8-terminal-herdr-architecture/).

## Frontend Stack (`ui/`)

- React 18 + TypeScript 5 + Vite 5
- xterm.js for Herdr terminal panes
- React Flow for schematic/PCB topology visualization
- Tailwind CSS + Zustand + TanStack Query
- Tauri API for IPC

## IPC Commands (Exposed to Frontend)

| Command | Description |
|---------|-------------|
| `start_run` | Launch pipeline/phase, returns `EventStream` |
| `get_state` | Query canonical model at snapshot |
| `propose_capability` | Submit capability invocation (validated by kernel) |
| `sync_kicad` | Import/export/render/watch KiCad project |
| `list_projects` | List all projects |
| `create_project` | Create new project |
| `open_project` | Load project, start Herdr session |
| `attach_terminal` | Get PTY info for Herdr pane |
| `export_manufacturing` | Generate BOM, Gerber, Drill, PnP |

## Development

```sh
# Frontend dev server
cd ui && npm run dev

# Tauri dev (frontend + backend)
cargo tauri dev

# Build release
cargo tauri build
```

## License

MIT OR Apache-2.0