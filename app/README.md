# eak-app — the Electronics Agent Kit desktop IDE (spine scaffold)

The native, local-first shell for the MVP ("Cursor for hardware"). A **Tauri** app whose
**backend is the existing Rust kernel** (`../eak/`) and whose frontend renders the kernel's
**live event stream**. This directory is the *spine* per
[`../project-plans/03-roadmap.md`](../project-plans/03-roadmap.md) (Phase: Spine) and
[`../project-plans/07-engineering-backlog.md`](../project-plans/07-engineering-backlog.md) (E1/E2).

> **Why it lives outside `eak/`:** it depends on Tauri (webkit2gtk etc.), which the CI/sandbox
> can't build, so it is deliberately **not** a member of the `eak/` Cargo workspace — the kernel
> build stays green and independent. You build/run this on your machine, where the Tauri
> prerequisites exist.

## What the spine already gives you (verified in the kernel)

- `eak_ports::EventSink` — a live observer invoked once per event on the kernel's single commit
  path (`RuntimeCore::commit`), right after append+fold. This is the seam the UI subscribes to.
- `eak_cli::run_with_sink(reasoning, &cfg, Some(sink))` — runs the full 15-phase workflow and
  streams every `EventRecord` (`{seq, timestamp, event}`, already `Serialize`) to your sink as it
  happens. `None` is byte-identical to `run_with` (determinism/replay preserved, P4).

The desktop app is a thin bridge: **a sink that forwards each `EventRecord` to the webview as a
Tauri event**, and a frontend that renders the resulting live engineering-state feed.

## Prerequisites (on your machine)

- Rust toolchain (same as the kernel).
- Tauri v2 system deps — see https://v2.tauri.app/start/prerequisites/ (Linux: `webkit2gtk`,
  `libgtk`, etc.).
- Node + a package manager (for the frontend dev server / bundling), or serve `ui/` statically.
- Tauri CLI: `cargo install tauri-cli` (or `npm i -D @tauri-apps/cli`).

## Getting it running (recommended path)

The fastest correct route is to let Tauri generate the standard shell, then drop in the two
EAK-specific pieces already here:

1. Generate a Tauri v2 scaffold in this folder (or merge into it):
   `npm create tauri-app@latest` → choose vanilla/TS, frontend dir `ui/`.
2. Replace the generated `src-tauri/src/main.rs` with [`src-tauri/src/main.rs`](src-tauri/src/main.rs)
   here (the `EventSink`→Tauri-event bridge + the `start_run` command).
3. Add the kernel path-dependencies from [`src-tauri/Cargo.toml`](src-tauri/Cargo.toml) here to the
   generated `Cargo.toml`.
4. Use [`ui/index.html`](ui/index.html) as the frontend (listens for `eak://event` and renders the
   live feed).
5. `cargo tauri dev` → a native window opens; click **Run** → watch a real pipeline run stream in.

> The generated `tauri.conf.json` / `build.rs` from step 1 are version-specific, so we let Tauri's
> own tooling produce them rather than hand-write (and mis-pin) them here. The **custom** code —
> the bridge and the UI — is what's provided in this folder.

## Milestone this proves

> *"The kernel is the native core of a desktop app, and one real pipeline run streams live into a
> native window."* — the Spine exit criterion. Everything else (KiCanvas canvas, the agent panel,
> KiCad import) hangs off this bridge.
