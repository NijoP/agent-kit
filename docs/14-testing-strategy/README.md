# Testing Strategy: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines the comprehensive testing strategy for EAK: unit, integration, contract, property, golden, chaos, and verification tests.

---

## Test Pyramid

```
                    ┌─────────────┐
                    │   Chaos     │  ◄── Weekly, production-like
                    ├─────────────┤
                    │ Verification│  ◄── Every PR, engineering rules
                    ├─────────────┤
                    │   Golden    │  ◄── Every PR, round-trip fidelity
                    ├─────────────┤
                    │  Property   │  ◄── Nightly, fuzzing
                    ├─────────────┤
                    │  Contract   │  ◄── Every PR, port trait compliance
                    ├─────────────┤
                    │ Integration │  ◄── Every PR, multi-crate flows
                    ├─────────────┤
                    │    Unit     │  ◄── Every commit, fast
                    └─────────────┘
```

---

## 1. Unit Tests (`cargo test --lib`)

### Scope
- Pure functions in `eak-units`, `eak-domain`, `eak-ports`, `eak-runtime`, `eak-engines`, `eak-compiler`.
- Deterministic validation functions.
- FSM transition logic.

### Requirements
- **Speed**: < 10ms per test, < 30s total.
- **Deterministic**: No randomness, no I/O, no time.
- **Coverage**: ≥ 90% for kernel crates (`eak-runtime`, `eak-ports`, `eak-domain`).

### Patterns
```rust
#[test]
fn fold_requirement_added_preserves_existing() {
    let mut state = EngineeringState::empty();
    let req = Requirement::new("REQ-1", "Must have USB-C");
    let event = EventRecord::new(Event::RequirementAdded(req.clone()), Provenance::fixture());
    
    let new_state = fold(state, event);
    
    assert_eq!(new_state.requirements.len(), 1);
    assert_eq!(new_state.requirements[0].id, req.id);
}
```

---

## 2. Integration Tests (`cargo test --test integration`)

### Scope
- Multi-crate flows: `eak-runtime` + `eak-engines` + `eak-phases` + `eak-store` + `eak-reasoning(fixture)`.
- Full phase execution: Requirement → Component → Schematic → PCB → Verification → Manufacturing.
- CLI commands: `eak run`, `eak replay`, `eak trace`, `eak export`.

### Requirements
- **Speed**: < 60s per test, < 10min total.
- **Isolation**: Each test uses temp directory, independent event log.
- **Fixtures**: `FixtureReasoningEngine` for deterministic model responses.

### Key Integration Tests
| Test | Description |
|------|-------------|
| `full_pipeline_deterministic` | Run all phases with fixture, replay, assert identical |
| `component_selection_flow` | Requirements → component search → evidence → validation → approve |
| `schematic_erc_clean` | Build schematic, run ERC, assert pass |
| `pcb_drc_clean` | Place/route, run DRC, assert pass |
| `kicad_roundtrip` | EAK → KiCad → EAK, assert semantic equivalence |
| `verification_gates` | Power/Clock/SI/DFM run, produce confidence/fidelity |
| `crash_recovery` | Kill kernel mid-run, restart, replay, validate state |

---

## 3. Contract Tests (`cargo test --test contract`)

### Scope
- **Port Trait Compliance** — Every adapter implements its port trait correctly.
- **Capability Seam Validation** — Invalid proposals rejected, valid accepted.
- **Event Schema** — Serialization/deserialization round-trip.

### Port Traits Tested
| Port | Adapters Tested |
|------|-----------------|
| `EventLog` | `JsonLinesEventLog`, `SqliteEventLog`, `PostgresEventLog` |
| `ReasoningEngine` | `FixtureReasoningEngine`, `NvidiaNimAdapter`, `OpenAICompatibleAdapter`, `OllamaAdapter` |
| `KicadSync` | `KicadAdapter` (import, export, render, watch) |
| `TerminalRuntime` | `HerdrAdapter` |

### Contract Test Pattern
```rust
#[test]
fn event_log_append_read_roundtrip() {
    let log = JsonLinesEventLog::new(temp_dir());
    let event = EventRecord::new(Event::RequirementAdded(...), Provenance::fixture());
    
    let seq = log.append(event.clone()).await.unwrap();
    let read = log.read(seq).await.unwrap().unwrap();
    
    assert_eq!(read.event, event.event);
    assert_eq!(read.provenance, event.provenance);
}
```

---

## 4. Property Tests (`cargo test --test proptest`)

### Scope
- **Fold Properties** — Associativity, idempotence, commutativity (where applicable).
- **Validation Functions** — Exhaustive input generation for rule validation.
- **Quantity Arithmetic** — `eak-units` operations (add, mul, div, sqrt) with random quantities.

### Tools
- `proptest` crate.
- Custom strategies for `EntityId`, `Quantity<U>`, `EventRecord`.

### Example
```rust
proptest! {
    #[test]
    fn fold_is_associative(events in vec(any_event(), 1..10)) {
        let state = EngineeringState::empty();
        let folded_seq = events.iter().fold(state, |s, e| fold(s, e.clone()));
        
        // Split at random point
        let split = prop::test_fun(|i: usize| i % events.len());
        let (left, right) = events.split_at(split);
        let folded_split = right.iter().fold(
            left.iter().fold(state, |s, e| fold(s, e.clone())),
            |s, e| fold(s, e.clone())
        );
        
        prop_assert_eq!(folded_seq, folded_split);
    }
}
```

---

## 5. Golden Tests (`cargo test --test golden`)

### Scope
- **KiCad Round-trip** — Golden `.kicad_sch`/`.kicad_pcb` files → import → export → compare normalized S-expressions.
- **Render Output** — Generated PNG/PDF compared to golden images (pixel diff < 0.1%).
- **Export Formats** — Gerber, Drill, BOM, PnP compared to golden files.
- **Event Log Replay** — Golden event logs → replay → assert final state hash.

### Golden File Management
```
tests/golden/
├── kicad/
│   ├── import/
│   │   ├── simple_sch.kicad_sch
│   │   └── simple_pcb.kicad_pcb
│   ├── export/
│   │   ├── expected_sch.sexp
│   │   └── expected_pcb.sexp
│   └── render/
│       ├── schematic_golden.png
│       └── pcb_golden.png
├── exports/
│   ├── bom_golden.csv
│   ├── gerber_golden/
│   └── drill_golden.drl
└── event_logs/
    ├── pipeline_10_events.jsonl
    └── pipeline_10_events_state.hash
```

### Update Protocol
- Golden files updated **only** via `cargo test --test golden -- --update-golden`.
- Requires PR with explanation.
- Reviewed for intentional changes.

---

## 6. Chaos Tests (`cargo test --test chaos`)

### Scope
- **Randomized Phase Execution** — Random order, random retries, random failures.
- **Concurrent Access** — Multiple readers (UI, CLI, Herdr) during write.
- **Resource Exhaustion** — Disk full, memory pressure, file descriptor limit.
- **Network Partition** — Model provider unavailable, KiCad CLI timeout.
- **Corrupted Inputs** — Malformed KiCad files, truncated event log, bad JSONL.

### Tools
- `chaos-mesh` concepts (adapted for local).
- `fail-rs` for fault injection.
- Custom chaos harness.

### Example
```rust
#[test]
fn chaos_random_phase_failures() {
    let harness = ChaosHarness::new()
        .with_fault(Fault::RandomPhaseFailure(0.1))
        .with_fault(Fault::ModelProviderTimeout(0.05))
        .with_fault(Fault::DiskFull(0.01));
    
    let result = harness.run_full_pipeline();
    
    // Kernel must recover, no corrupted state
    assert!(result.recovered);
    assert_eq!(result.final_state_hash, expected_hash);
}
```

---

## 7. Verification Tests (`cargo test --test verification`)

### Scope
- **Rule Correctness** — Each verification rule tested against known-good/bad designs.
- **Confidence Calibration** — Confidence scores match empirical accuracy.
- **Fidelity Comparison** — Analytical vs Simulation vs Measured on same design.
- **Regression** — Previously fixed violations stay fixed.

### Test Data
```
tests/verification/
├── designs/
│   ├── clean_design/          # Passes all
│   ├── erc_violations/        # Known ERC errors
│   ├── drc_violations/        # Known DRC errors
│   ├── power_issues/          # Voltage drop, decap
│   ├── clock_issues/          # Skew, jitter
│   ├── si_issues/             # Impedance, crosstalk
│   └── dfm_issues/            # Min trace, annular ring
└── expected_results/
    ├── clean_design.ron
    ├── erc_violations.ron
    └── ...
```

---

## Test Infrastructure

### CI Pipeline (GitHub Actions)
```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
      - name: Unit tests
        run: cargo test --workspace --lib
      - name: Integration tests
        run: cargo test --workspace --test integration
      - name: Contract tests
        run: cargo test --workspace --test contract
      - name: Golden tests
        run: cargo test --workspace --test golden
      - name: Verification tests
        run: cargo test --workspace --test verification
      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Format
        run: cargo fmt --workspace --check
      - name: Doc tests
        run: cargo test --workspace --doc
```

### Nightly Jobs
- Property tests (proptest).
- Chaos tests.
- Long-running verification tests.
- Dependency audit (`cargo deny`).

### Test Data Management
- **Fixtures** in `tests/fixtures/` (small, committed).
- **Golden files** in `tests/golden/` (committed, LFS for large).
- **Large test designs** — Downloaded on-demand in CI (cached).

---

## Coverage Targets

| Crate | Unit | Integration | Contract |
|-------|------|-------------|----------|
| `eak-units` | 95% | — | — |
| `eak-domain` | 90% | — | — |
| `eak-ports` | 80% | — | 100% |
| `eak-runtime` | 90% | 80% | — |
| `eak-engines` | 85% | 80% | — |
| `eak-compiler` | 85% | 70% | — |
| `eak-phases` | 80% | 75% | — |
| `eak-store` | 70% | 80% | 100% |
| `eak-reasoning` | 60% | 70% | 100% |
| `eak-kicad` | 60% | 70% | 100% |
| `eak-assets` | 70% | 60% | — |

**Overall**: ≥ 80% line coverage on kernel crates.

---

## Next Document

[Release Strategy →](../15-release-strategy/README.md)