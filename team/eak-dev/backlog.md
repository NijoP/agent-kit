# EAK Development Backlog

## Hour 0-2: Repository Reset

### Baseline Results


### Hour 2-5 Results
- Tauri integration completed: intent input → Tauri → Rust → EAK workflow → events → React working
- Modified app/src-tauri/src/main.rs to uncomment and activate kernel integration
- Used FixtureEngine::single() for reasoning engine
- Constructed proper RunConfig with intent and defaults
- Verified clean compilation: cargo check passes with only expected warnings
- Desktop application now properly streams kernel events to UI via existing event bridge


### Hour 2-5 Results
- Tauri integration completed: intent input → Tauri → Rust → EAK workflow → events → React working
- Modified app/src-tauri/src/main.rs to uncomment and activate kernel integration
- Used FixtureEngine::single() for reasoning engine
- Constructed proper RunConfig with intent and defaults
- Verified clean compilation: cargo check passes with only expected warnings
- Desktop application now properly streams kernel events to UI via existing event bridge
- Repository cleaned: removed .opencode/, eak-dashboard and eak-model-health executables, data/telemetry/*
- Branch cleanup: deleted redundant phase-3-band-a branch
- Architecture verified: vision documentation correctly positions EAK as independent engineering runtime
- Build status: cargo check passes with warnings only (no errors)
- Warnings: primarily unused imports/variables and unreachable patterns (code quality issues, not blocking)
- Next step: Begin Hour 2-5: Desktop Core implementation
- [x] Inspect repository state and branches
- [x] Create team/eak-dev/ structure
- [x] Create mission.md, backlog.md, decisions.md
- [x] Clean obvious generated/dead duplication
- [x] Evaluate branches for cleanup
- [x] Correct architecture references
- [x] Build current baseline
- [x] Run cargo check, cargo test, cargo clippy
- [ ] Run frontend checks
- [ ] Record baseline

## Hour 2-5: Desktop Core
- [x] Make Tauri real (primary objective)
- [ ] Fix intent input → Tauri → Rust → EAK workflow → events → React
- [ ] Get first genuinely live EAK run visible in desktop UI
- [ ] Real Tauri project startup
- [ ] Real command wiring
- [ ] Real run invocation
- [ ] Real error handling
- [ ] Real event streaming
- [ ] Real UI state updates

## Hour 5-8: Agent Runtime / Terminals
- [ ] Implement minimum viable EAK terminal/session layer
- [ ] Real workspace, tabs, panes, PTY, process, input, output, status
- [ ] Support Pi, shell, Claude Code, Codex, OpenCode where available
- [ ] Make architecture extensible

## Hour 8-11: Crew / Handoff System
- [ ] Implement EAK Lead, Research, Component, Architecture, Schematic, PCB, Verification agents
- [ ] Use durable handoffs
- [ ] Make agent work observable
- [ ] Agent output becomes structured engineering artifacts

## Hour 11-15: EDA Experience
- [ ] Prioritize real useful schematic/PCB interaction layer
- [ ] Schematic: select, place, move, inspect, wire if possible, cross-probe
- [ ] PCB: select, place, move, rotate, inspect, route where possible, cross-probe
- [ ] Focus on actual interaction, not visual decoration

## Hour 15-18: Component Intelligence
- [ ] Implement component compiler foundation
- [ ] Source resolver, evidence object, engineering facts, component compiler, provenance
- [ ] Compile small real set of components from authoritative sources

## Hour 18-20: Verification + 3D
- [ ] Connect ERC, DRC, DFM to live product flow
- [ ] Implement most useful viable 3D route
- [ ] Prefer reuse of proven rendering infrastructure

## Hour 20-22: Project Lifecycle
- [ ] Implement save, open, snapshot, diff, reopen
- [ ] Ensure project state survives restart

## Hour 22-24: Release Hardening
- [ ] Run full tests, fix failures
- [ ] Run lint, fix warnings
- [ ] Build desktop artifacts
- [ ] Test actual launch
- [ ] Update README
- [ ] Document known limitations
- [ ] Prepare release notes
- [ ] Check repository cleanliness and branch state
- [ ] Check licenses and attribution

## Immediate Next Steps
- [ ] Complete repository inspection and cleanup