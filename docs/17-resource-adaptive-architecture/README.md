# Resource-Adaptive Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines the **resource-adaptive architecture** for EAK. EAK must run smoothly on a wide range of hardware — from 4 GB constrained systems to 32 GB+ workstations — by adapting its resource consumption while preserving **identical engineering truth** across all configurations.

**Core Principle:**

> **LOW-END HARDWARE GETS LOWER VISUAL FIDELITY, NOT LOWER ENGINEERING FUNCTIONALITY.**

A weak machine receives:
- Simpler rendering
- Lower 3D quality
- Fewer visual effects
- Fewer simultaneous heavy operations
- Smaller caches
- Less background activity
- Lazy-loaded resources

But it **retains the same engineering capabilities**:
- Identical engineering data correctness
- Identical electrical values
- Identical component facts
- Identical net relationships
- Identical constraints
- Identical engineering calculations
- Identical project state
- Identical verification logic
- Identical provenance
- Identical saved data
- Identical design integrity

**Performance adapts. Engineering truth does not.**

---

## Product Experience by Hardware Tier

| Tier | RAM | Experience |
|------|-----|------------|
| **SAFE** | ≤ 4 GB | "EAK is lightweight and responsive." Minimal effects, 2D-first, lazy everything. |
| **LOW** | 8 GB | "EAK is smooth with adaptive visual quality." Simplified 3D, bounded caches. |
| **BALANCED** | 16 GB | "EAK is comfortable." Normal UI, moderate 3D, normal caching. |
| **HIGH** | 32 GB+ | "EAK can use richer visualization." High-quality 3D, larger caches, aggressive prefetch. |
| **ULTRA** | 64 GB+ | "EAK uses maximum visual fidelity." Aggressive caching, high concurrency, richest rendering. |

---

## Resource Profile System

EAK maintains a **Resource Profile** that adapts to the host machine and runtime conditions.

### Detected Factors

| Factor | Source | Update Frequency |
|--------|--------|------------------|
| Total RAM | OS query (sysinfo) | Startup + periodic |
| Available RAM | OS query | Every 5s |
| CPU cores / threads | OS query | Startup |
| CPU performance class | Benchmark / CPU flags | Startup |
| GPU availability | OS / Vulkan / Metal / DX12 | Startup |
| GPU VRAM | GPU driver query | Startup |
| Current CPU utilization | OS query | Every 2s |
| Memory pressure (OS) | OS pressure signals | Event-driven |
| Project size (entities) | Kernel state | On project load |
| Active EAK processes | Herdr / process table | Every 10s |
| Active external tools | Process table / Herdr | Every 10s |
| Terminal sessions | Herdr | Event-driven |
| Agent sessions | Phase FSM state | Event-driven |
| Rendering workload | UI frame timing | Per frame |

### Profile Computation

```rust
struct ResourceProfile {
    tier: ResourceTier,        // SAFE, LOW, BALANCED, HIGH, ULTRA
    memory_budget_mb: u64,     // Target max RSS for EAK process
    cpu_budget_percent: u8,    // Target CPU when active
    cache_budget_mb: u64,      // Max cache size
    concurrency_limit: u8,     // Max concurrent heavy tasks
    visual_quality: VisualQuality, // LOW, MEDIUM, HIGH, ULTRA
    gpu_acceleration: bool,    // Use GPU for rendering
    background_work_enabled: bool,
    prefetch_enabled: bool,
    concurrency_multiplier: f32, // Applied to agent/terminal/3D limits
}
```

### Automatic Tier Selection

1. **Startup**: Query hardware → compute initial tier
2. **Runtime**: Monitor memory pressure + CPU → adjust tier up/down
3. **Hysteresis**: Tier changes require sustained condition (30s) + cooldown (60s) to prevent oscillation
4. **Manual Override**: User can lock tier in settings

---

## Adaptive Resource Levels

### SAFE (≤ 4 GB)
| Parameter | Value |
|-----------|-------|
| Memory budget | 300 MB |
| CPU budget | 25% |
| Cache budget | 50 MB |
| Concurrency limit | 1 heavy task |
| Visual quality | LOW |
| GPU acceleration | Disabled |
| Background work | Disabled |
| Prefetch | Disabled |
| Concurrency multiplier | 0.25 |

### LOW (8 GB)
| Parameter | Value |
|-----------|-------|
| Memory budget | 600 MB |
| CPU budget | 40% |
| Cache budget | 150 MB |
| Concurrency limit | 2 heavy tasks |
| Visual quality | MEDIUM |
| GPU acceleration | Optional (integrated) |
| Background work | Minimal (event-driven only) |
| Prefetch | Critical only |
| Concurrency multiplier | 0.5 |

### BALANCED (16 GB)
| Parameter | Value |
|-----------|-------|
| Memory budget | 1.2 GB |
| CPU budget | 60% |
| Cache budget | 400 MB |
| Concurrency limit | 4 heavy tasks |
| Visual quality | HIGH |
| GPU acceleration | Enabled |
| Background work | Normal |
| Prefetch | Selective |
| Concurrency multiplier | 1.0 |

### HIGH (32 GB)
| Parameter | Value |
|-----------|-------|
| Memory budget | 2.5 GB |
| CPU budget | 80% |
| Cache budget | 1 GB |
| Concurrency limit | 8 heavy tasks |
| Visual quality | HIGH |
| GPU acceleration | Enabled |
| Background work | Aggressive prefetch |
| Prefetch | Aggressive |
| Concurrency multiplier | 1.5 |

### ULTRA (64 GB+)
| Parameter | Value |
|-----------|-------|
| Memory budget | 4 GB+ |
| CPU budget | 100% |
| Cache budget | 2 GB+ |
| Concurrency limit | 16+ heavy tasks |
| Visual quality | ULTRA |
| GPU acceleration | Enabled (multi-GPU if available) |
| Background work | Maximum |
| Prefetch | Full |
| Concurrency multiplier | 2.0 |

---

## Automatic Adaptation Flow

```
NORMAL OPERATION
       │
       ▼
Memory pressure detected (OS signal or available < threshold)
       │
       ▼
LEVEL 1: Stop optional prefetch
       │
       ▼
LEVEL 2: Reduce cache sizes (LRU eviction)
       │
       ▼
LEVEL 3: Release inactive assets (3D models, datasheets, terminal buffers)
       │
       ▼
LEVEL 4: Reduce visual quality tier
       │
       ▼
LEVEL 5: Reduce optional concurrency (agent/terminal/3D limits)
       │
       ▼
LEVEL 6: Pause non-critical background work
       │
       ▼
LEVEL 7: Notify user (only if critical)
       │
       ▼
CONTINUE ENGINEERING WORK
```

### Recovery (Resources Recover)

```
Resources recover (available > threshold + hysteresis)
       │
       ▼
Wait for cooldown (60s default)
       │
       ▼
LEVEL 6: Resume background work
       │
       ▼
LEVEL 5: Increase concurrency limits
       │
       ▼
LEVEL 4: Restore visual quality
       │
       ▼
LEVEL 3: Reload evicted assets (on demand)
       │
       ▼
LEVEL 2: Grow caches gradually
       │
       ▼
LEVEL 1: Resume prefetch
       │
       ▼
NORMAL OPERATION
```

---

## Visual Quality Adaptation

### UI / Schematic / PCB

| Quality Level | Behavior |
|---------------|----------|
| **LOW** | No animations, no shadows, solid fills only, 1x rendering scale, no antialiasing, immediate mode, no transitions |
| **MEDIUM** | Basic transitions, 1x scale, MSAA 2x, simplified selection highlight, 30 FPS cap |
| **HIGH** | Full animations, 1-2x scale, MSAA 4x, rich selection effects, 60 FPS |
| **ULTRA** | All effects, HiDPI scaling, MSAA 8x, motion blur, 120 FPS if display supports |

### 3D Viewer

| Quality Level | Geometry | Textures | Effects | Concurrent Scenes |
|---------------|----------|----------|---------|-------------------|
| **LOW** | Simplified (LOD max) | Disabled | None | 1 |
| **MEDIUM** | Medium LOD | Compressed | Basic lighting | 1 |
| **HIGH** | High LOD | Full | PBR, shadows | 2 |
| **ULTRA** | Full detail | 4K+ | PBR, RTX, volumetric | 4+ |

### Schematics / PCB

| Quality Level | Rendering |
|---------------|-----------|
| **LOW** | Immediate mode, no selection glow, no grid animation, wireframe only when zoomed out |
| **MEDIUM** | Retained mode, basic selection highlight, static grid |
| **HIGH** | Full effects, smooth pan/zoom, animated grid, live DRC highlight |
| **ULTRA** | All effects, predictive rendering, GPU instancing |

---

## 3D Resource Policy

- **On-demand loading only**: 3D models loaded when tab/view activated
- **LOD system**: Automatic level-of-detail based on zoom and quality tier
- **Unload policy**: Inactive 3D scenes unloaded after 60s (LOW), 300s (HIGH)
- **Streaming**: Large models streamed in chunks (progressive mesh)
- **Concurrent scenes**: Limited by tier (SAFE=1, LOW=1, BALANCED=2, HIGH=4, ULTRA=8)
- **GPU memory budget**: Tracked separately; evict oldest unused models first

---

## Schematic Performance

- **Incremental rendering**: Only dirty regions redrawn
- **Viewport culling**: Off-screen elements not processed
- **Dirty-region tracking**: Per-layer damage rects
- **Simplified effects at LOW**: No selection glow, no grid animation, wireframe at zoom < 25%
- **Selective redraw**: Only affected layers repainted on interaction
- **Bounded overlays**: Tooltips, selection boxes limited to viewport

---

## PCB Performance

- **Viewport culling**: Only visible layers/objects processed
- **Layered rendering**: Each layer rendered to offscreen texture, composited
- **Incremental updates**: Only changed nets/objects re-rendered
- **Simplified effects at LOW**: No zone fill animation, no 3D preview, ratsnest on demand
- **Bounded interactive overlays**: Selection, measurement, DRC markers limited to viewport
- **Incremental DRC**: Background incremental, full on demand

---

## Project Memory Management

**Do NOT load at startup:**
- Full project history
- Every revision
- Every datasheet
- Every evidence artifact
- Every 3D model
- Every library asset
- Every terminal transcript
- Every agent transcript
- Every verification log

**Load hierarchy:**
```
Project Metadata (always)
    │
    ▼
Active Engineering State (current revision)
    │
    ▼
Requested Resource (on demand)
    │
    ▼
Bounded Cache (LRU, size by tier)
    │
    ▼
Release when inactive (configurable TTL)
```

**Cache TTL by tier:**
| Tier | Datasheets | 3D Models | Terminal History | Agent Context | Verification Logs |
|------|------------|-----------|------------------|---------------|-------------------|
| SAFE | 1 hour | 10 min | 500 lines | 1 session | 1 run |
| LOW | 4 hours | 30 min | 2000 lines | 3 sessions | 3 runs |
| BALANCED | 24 hours | 2 hours | 10000 lines | 10 sessions | 10 runs |
| HIGH | 1 week | 24 hours | 50000 lines | 50 sessions | 50 runs |
| ULTRA | 1 month | 1 week | unlimited | unlimited | unlimited |

---

## Component Intelligence Memory Rules

**Do NOT create giant in-memory component database.**

```
Component Index (lightweight, always loaded)
    │
    ▼
Requested MPN
    │
    ▼
Requested Evidence
    │
    ▼
Requested Datasheet (lazy, cached by tier)
    │
    ▼
Requested Symbol (lazy, generated/cached)
    │
    ▼
Requested Footprint (lazy, generated/cached)
    │
    ▼
Requested 3D Model (lazy, streamed)
```

**Cache bounds by tier:** Same TTL table as project memory.

---

## Agent Resource Policy

| Tier | Max Concurrent Agents | Local Model Size | Remote Preferred |
|------|----------------------|------------------|------------------|
| SAFE | 1 | None (remote only) | Yes |
| LOW | 2 | < 1B params | Yes |
| BALANCED | 4 | < 7B params | Optional |
| HIGH | 8 | < 30B params | Optional |
| ULTRA | 16+ | Any | No |

- **Heavy model inference** must be separable from EAK desktop resource budget
- EAK desktop client must remain lightweight even when huge remote models used
- 500B+ model must NOT become desktop system requirement

---

## Model Provider Architecture

EAK desktop must remain lightweight regardless of model size.

| Provider Type | EAK Desktop Cost |
|---------------|------------------|
| Remote (NVIDIA NIM, OpenAI) | Minimal (HTTP client only) |
| Local lightweight (< 1B) | Low (llama.cpp, quantized) |
| Local medium (1-7B) | Medium |
| Local heavy (7-30B) | High (requires HIGH/ULTRA tier) |
| Local massive (30B+) | Not in desktop (use remote gateway) |
| External server (Ollama, vLLM, TGI) | Minimal (HTTP client) |

---

## Terminal / Herdr Resource Policy

**Do NOT automatically launch:**
- Claude / Codex / Pi / OpenCode
- KiCad
- Multiple research processes
- Multiple build processes
- Multiple agents

**Default startup:** Lightweight shell only.

**On-demand launch:** User explicitly starts agent/tool.

**Inactive sessions:**
- PTY buffers bounded (SAFE=1000 lines, LOW=5000, BALANCED=20000, HIGH=100000, ULTRA=unlimited)
- No rendering for hidden panes
- PTY output throttled when pane not visible

**Herdr integration principles (from Herdr architecture):**
- Event-driven updates, not polling
- Hidden panes don't render
- PTY output throttled when not visible
- Agent detection lightweight
- Session restore loads state lazily

---

## Background Task Policy

**Every background task must declare:**
- `why`: Purpose
- `interval`: Run frequency
- `cpu_budget_percent`: Max CPU per run
- `memory_budget_mb`: Max memory per run
- `cancellable`: Whether it can be interrupted
- `tier_requirement`: Minimum tier to run

**No large background workload simply because machine is available.**

---

## Resource Pressure Levels

| Level | Trigger | Action |
|-------|---------|--------|
| 0 | Normal | Normal operation |
| 1 | Available RAM < 20% of total | Stop optional prefetch |
| 2 | Available RAM < 15% | Reduce caches (LRU evict 25%) |
| 3 | Available RAM < 10% | Release inactive assets (3D, datasheets, terminal buffers > TTL) |
| 4 | Available RAM < 5% | Reduce visual quality tier |
| 5 | Available RAM < 3% | Reduce concurrency limits (agents, terminals, 3D) |
| 6 | Available RAM < 2% | Pause non-critical background work |
| 7 | Available RAM < 1% | Notify user (toast + log), emergency GC |

**Never:** Silently destroy work, corrupt project state, lose unsaved data.

---

## Cross-Platform Resource Detection

| Platform | RAM | CPU | GPU | Pressure Signals |
|----------|-----|-----|-----|------------------|
| Linux | `/proc/meminfo`, `sysinfo` | `/proc/cpuinfo`, `sched_getaffinity` | DRM/Vulkan | `oom_score`, `memory.pressure` cgroup |
| Windows | `GlobalMemoryStatusEx` | `GetSystemInfo`, `GetLogicalProcessorInformation` | DXGI/WDDM | `CreateMemoryResourceNotification` |
| macOS | `sysctl hw.memsize` | `sysctl hw.logicalcpu` | Metal/IOKit | `vm_pressure_monitor` |

**Abstraction:** `ResourceMonitor` trait in `eak-ports`, platform adapters in `eak-runtime` / `app/`.

---

## Performance-First Architecture Decisions

When choosing between implementations, prefer:

| Criterion | Preference |
|-----------|------------|
| Memory usage | Lower |
| Idle CPU | Lower |
| Rendering | Incremental |
| Resource loading | Lazy |
| Data duplication | Avoid |
| Cancellation | Supported |
| Polling | Event-driven instead |
| Scaling | With project size AND hardware |
| Correctness | Never sacrificed |

---

## Benchmarking System

### Required Measurements

| Scenario | Metrics |
|----------|---------|
| Cold startup | Time to interactive, peak RSS, CPU |
| Idle (empty) | Steady-state RSS, CPU % |
| Idle (project open) | Steady-state RSS, CPU % |
| Project open | Time, peak RSS, CPU |
| Project navigation | Time, RSS delta |
| Schematic pan/zoom | Frame time, GPU time |
| PCB pan/zoom | Frame time, GPU time |
| 3D model load | Time, peak GPU/CPU memory |
| Verification run | Time, CPU, peak RSS |
| Agent activation | Time, CPU, network |
| Terminal spawn | Time, PTY overhead |
| Save project | Time, disk I/O, RSS |
| Load project | Time, peak RSS |
| Close project | Time, cleanup RSS |
| Reopen project | Time, peak RSS |

### Test Environments

| Environment | Method |
|-------------|--------|
| 4 GB constrained | cgroup / VM / physical |
| 8 GB | Physical / VM |
| 16 GB | Physical |
| 32 GB+ | Physical |

If physical unavailable: use `systemd-run --scope -p MemoryMax=...` or VM.

---

## Adaptive Quality Validation

**Changing resource mode MUST NOT change:**

- Design data
- Net connectivity
- Component identity
- Electrical values
- Constraints
- Verification results
- Project history
- Saved artifacts
- Provenance
- Engineering calculations

**Only these may change:**
- Visual fidelity
- Rendering speed
- Cache hit rates
- Background work latency
- Concurrent task limits
- Prefetch behavior

---

## Product Requirement

EAK must eventually advertise:

> **"Adaptive performance for a wide range of hardware."**

**Documentation must distinguish:**

| Status | Meaning |
|--------|---------|
| TESTED | Validated on real hardware |
| SUPPORTED | Architecture supports, not yet tested |
| EXPERIMENTAL | May have issues |
| NOT TESTED | No validation |

---

## Minimum User Expectation

An older 8 GB laptop must be able to:

- [ ] Install EAK
- [ ] Launch EAK (< 5s cold start)
- [ ] Navigate EAK UI
- [ ] Open a project
- [ ] Access available EAK features
- [ ] Interact with supported schematic functionality
- [ ] Interact with supported PCB functionality
- [ ] Inspect verification results
- [ ] Use remote AI provider
- [ ] Use terminal functionality
- [ ] Save/load projects

**With lower visual quality where necessary.**

**Responsiveness prioritized over visual effects.**

---

## Important Distinction

**DO NOT IMPLEMENT:** "LOW RAM = DISABLE FEATURES"

**INSTEAD IMPLEMENT:** "LOW RAM = REDUCE COST OF FEATURES"

| Feature | Low RAM Approach |
|---------|------------------|
| 3D | Lower quality, lazy load, unload inactive |
| Schematic | Simpler rendering, incremental, culling |
| PCB | Less expensive effects, culling, incremental |
| Agent system | Less concurrency, remote preferred |
| Library | Lazy loading, bounded cache |
| Terminal | Bounded buffers, hidden panes sleep |
| Verification | Same logic, less parallel |

---

## Definition of Done

This requirement is complete when:

- [ ] **Resource profiling** exists (detects hardware, monitors runtime)
- [ ] **Adaptive modes** exist (SAFE, LOW, BALANCED, HIGH, ULTRA)
- [ ] **Automatic adaptation** exists (pressure detection, tier adjustment, hysteresis)
- [ ] **Visual quality scaling** exists (UI, schematic, PCB, 3D)
- [ ] **3D quality scaling** exists (LOD, streaming, unload policy)
- [ ] **Schematic rendering scales** (incremental, culling, simplified effects)
- [ ] **PCB rendering scales** (culling, layered, incremental)
- [ ] **Cache sizing scales** (bounded by tier, LRU eviction)
- [ ] **Agent concurrency scales** (bounded by tier, remote preferred)
- [ ] **Terminal workload scales** (bounded buffers, hidden pane sleep)
- [ ] **Background processing scales** (task budgets, tier requirements)
- [ ] **Resource-pressure response** exists (7 levels, recovery with hysteresis)
- [ ] **Low-end behavior** defined and tested
- [ ] **Cross-platform behavior** defined (Linux, Windows, macOS adapters)
- [ ] **Benchmarks exist** (reproducible, measurable)
- [ ] **Real performance measurements** exist (documented)
- [ ] **Engineering correctness identical** across all modes (validated)
- [ ] **Documentation explains** adaptive architecture (this document)

---

## Development Instruction

### Phase 1: Audit (Current Sprint)
1. **Resource Architecture Audit** — What already exists, what's wasteful
2. **Current Resource Risks** — Polling, duplicate state, expensive UI renders, terminal/agent multiplication, KiCad/3D heaviness
3. **Adaptive Resource Design** — This document
4. **Implementation Plan** — Staged rollout
5. **Benchmark Plan** — Reproducible measurements
6. **Definition of Done** — This document

### Phase 2: Implementation (Future Sprints)
- Add `ResourceMonitor` port to `eak-ports`
- Implement platform adapters (`eak-runtime` / `app/`)
- Add `ResourceProfile` to kernel state
- Implement tier selection and hysteresis
- Add visual quality settings to UI
- Implement LOD/streaming for 3D
- Add cache bounds to all caches
- Add resource budgets to background tasks
- Add pressure detection and response
- Create benchmark suite

---

## Next Documents

- [Product Vision](../1-product-vision/README.md) — Updated with resource-adaptive principle
- [System Architecture](../3-system-architecture/README.md) — Updated with resource-adaptive boundaries
- [Master Roadmap](../16-master-implementation-roadmap/README.md) — Updated with resource-adaptive stages

---

*End of Resource-Adaptive Architecture*