# Development Workflow: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines the development workflow for EAK contributors, integrating the no-mistakes quality gate, backpass for evidence-backed improvements, and Firstmate for orchestration.

---

## Core Principle

**Agent change → tests → lint → documentation check → AI review → no-mistakes gate → clean branch/PR → merge authority.**

No bypassing verification. No direct pushes to main.

---

## Toolchain

| Tool | Purpose |
|------|---------|
| **cargo** | Build, test, clippy, fmt |
| **no-mistakes** | Quality gate (tests, lint, docs, AI review, CI) |
| **backpass** | Evidence-backed instruction/memory improvements |
| **Firstmate** | Development orchestration, PR management, CI |
| **gh-axi** | GitHub operations |
| **chrome-devtools-axi** | Browser automation for UI tests |

---

## Branch Strategy

| Branch | Purpose |
|--------|---------|
| `main` | Protected, only merges via no-mistakes gate |
| `fm/*` | Firstmate-managed feature branches (this task: `fm/EAK-Product-Reset`) |
| `sm/*` | Secondmate branches (platform architecture) |
| `release/v*` | Release branches (cut from main) |

### Rules
1. **Never push directly to `main`**.
2. **All changes on feature branches** (`fm/...`, `sm/...`).
3. **no-mistakes validates** before PR creation.
4. **PR requires** passing CI, AI review, no-mistakes gate.
5. **Merge authority** — Firstmate merges after gate passes.

---

## Development Loop

```
1. START TASK
   │
   ▼
2. CREATE BRANCH (fm/<slug>)
   │
   ▼
3. WRITE CODE + TESTS
   │
   ▼
4. RUN LOCAL VALIDATION
   cargo check --workspace
   cargo test --workspace
   cargo clippy --all-targets -- -D warnings
   cargo fmt --check
   │
   ▼
5. UPDATE DOCUMENTATION
   (corresponding docs/ files)
   │
   ▼
6. RUN NO-MISTAKES
   no-mistakes axi run --intent "<captain's intent>"
   │
   ▼
7. RESPOND TO GATES
   - Test failures → fix code
   - Lint failures → fix code
   - Doc failures → update docs
   - AI review findings → address or escalate
   - Ask-user findings → escalate to Firstmate
   │
   ▼
8. NO-MISTAKES GATE PASSES
   │
   ▼
9. FIRSTMATE CREATES PR
   │
   ▼
10. CI RUNS (GitHub Actions)
    │
    ▼
11. MERGE (by Firstmate)
```

---

## No-Mistakes Integration

### Running the Gate
```bash
# From repository root
no-mistakes axi run --intent "Execute the 24-hour EAK product maturity reset..."
```

### Gate Types
| Gate | Description | Resolution |
|------|-------------|------------|
| **Test** | `cargo test --workspace` fails | Fix code, re-run |
| **Lint** | `cargo clippy -D warnings` fails | Fix warnings |
| **Format** | `cargo fmt --check` fails | Run `cargo fmt` |
| **Docs** | Documentation check fails | Update docs |
| **AI Review** | AI reviewer finds issues | Address or escalate |
| **Ask-User** | Requires human decision | Escalate to Firstmate (never answer yourself) |

### Ask-User Findings Protocol
1. No-mistakes produces `nm-<run>-findings.txt` with finding IDs.
2. **Worker escalates**: `needs-decision [key=nm-<run>-<step>]: ask-user findings=<id1>,<id2>... file=/path/to/findings.txt`
3. Firstmate applies `ask-user-authority`, gets captain decision.
4. Worker feeds decision to gate: `no-mistakes axi respond ...`
5. Pipeline applies fix — **worker never implements fix**.

---

## Backpass Integration

### Purpose
Evidence-backed memory/instruction improvements for agents. **Never** allows model speculation to rewrite architectural truth.

### When to Use
- After completing a significant task with learnings.
- When agent instructions need refinement based on observed failures.
- When documentation patterns need extraction.

### Process
1. Collect evidence (test results, error patterns, review feedback).
2. Run `backpass` with evidence corpus.
3. Backpass proposes instruction updates.
4. **Human reviews** proposals before applying.
5. Applied updates go through normal no-mistakes gate.

---

## Firstmate Orchestration

### Role
Firstmate is the **development control plane** for this repository. It:
- Manages branch lifecycle.
- Runs no-mistakes gates.
- Creates PRs.
- Monitors CI.
- Merges after gate passes.

### Worker Interaction
- Worker receives tasks via Firstmate inbox (`/home/dev/firstmate/state/<task>.inbox/`).
- Worker reports status via status file (`/home/dev/firstmate/state/<task>.status`).
- Worker **does not** create PRs, merge, or manage CI.

---

## Code Standards

### Rust
- **Edition**: 2021
- **Clippy**: `-D warnings` (deny all)
- **Format**: `cargo fmt` (standard)
- **Dependencies**: Minimal, audited (`cargo-deny` in CI)
- **Unsafe**: Forbidden except in `eak-units` (verified) and FFI boundaries (documented)

### TypeScript (Frontend)
- **Strict**: `strict: true`
- **ESLint**: Recommended + TypeScript ESLint
- **Prettier**: Standard config
- **Tests**: Vitest + React Testing Library

### Documentation
- **Markdown**: CommonMark + front matter (version, status)
- **Diagrams**: Mermaid (rendered in GitHub)
- **Cross-refs**: Relative links (validated in CI)

---

## Testing Requirements

| Level | Command | When |
|-------|---------|------|
| Unit | `cargo test --lib` | Every commit |
| Integration | `cargo test --test integration` | Every PR |
| Contract | `cargo test --test contract` | Every PR |
| Property | `cargo test --test proptest` | Nightly |
| Golden | `cargo test --test golden` | Every PR |
| Chaos | `cargo test --test chaos` | Weekly |
| Verification | `cargo test --test verification` | Every PR |

### Test Organization
```
eak/
├── crates/
│   ├── eak-runtime/
│   │   ├── src/
│   │   └── tests/
│   │       ├── integration_tests.rs
│   │       ├── contract_tests.rs
│   │       └── golden_tests.rs
│   └── ...
```

---

## Documentation Workflow

### When to Update Docs
- New capability seam → update System Architecture + Capability docs.
- New entity/attribute → update Engineering Model.
- New verification rule → update Verification Architecture.
- New agent role/phase → update Agent Architecture.
- New UI feature → update UI Architecture.

### Doc Validation
- `cargo test --doc` — Rust doc tests pass.
- `markdownlint` — Markdown style.
- `linkcheck` — No broken internal links.
- `frontmatter` — All docs have `version`, `status`.

---

## Release Process

1. **Cut Release Branch** — `release/vX.Y.Z` from `main`.
2. **Version Bump** — `Cargo.toml` versions, `docs/*/README.md` front matter.
3. **Changelog** — Generated from conventional commits.
4. **No-Mistakes Gate** — Full validation on release branch.
5. **CI Build** — Signed binaries, installers, SBOM.
6. **Tag** — `git tag vX.Y.Z` (signed).
7. **Publish** — GitHub Releases, crates.io (kernel crates), npm (frontend).
8. **Announce** — Blog, Discord, mailing list.

---

## Emergency Hotfix

1. Branch from latest release tag: `hotfix/vX.Y.Z+1`.
2. Minimal fix + test.
3. No-mistakes gate (expedited).
4. Cherry-pick to `main` after release.
5. Post-mortem within 48h.

---

## Next Document

[Testing Strategy →](../14-testing-strategy/README.md)