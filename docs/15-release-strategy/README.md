# Release Strategy: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines versioning, packaging, signed builds, installers, CI/CD, and release cadence for EAK.

---

## Versioning

### Scheme: SemVer + Git Metadata
```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]
```

| Component | Rule |
|-----------|------|
| **MAJOR** | Breaking changes to kernel API, event schema, or capability seams |
| **MINOR** | New capabilities, verification kinds, agent roles, backward-compatible |
| **PATCH** | Bug fixes, performance, documentation, no API changes |
| **PRERELEASE** | `alpha`, `beta`, `rc` (e.g., `0.2.0-beta.1`) |
| **BUILD** | Git commit hash (short), e.g., `+gabc1234` |

### Crate Versioning
- All `eak-*` crates in workspace share **same version** (workspace `version` in `eak/Cargo.toml`).
- `eak-app` (Tauri) versions independently but tracks kernel version in metadata.

### Documentation Versioning
- Each `docs/*/README.md` has `version:` front matter matching release.
- `docs/README.md` lists current version.

---

## Release Cadence

| Channel | Cadence | Audience | Stability |
|---------|---------|----------|-----------|
| **Nightly** | Every merge to `main` | Developers, CI | Unstable, may break |
| **Alpha** | Monthly | Early adopters | Feature complete, known bugs |
| **Beta** | Quarterly | Testers | API stable, verification complete |
| **Release Candidate** | Pre-release | Release validation | Release candidate |
| **Stable** | Quarterly | General users | Production ready |

### V1 Milestone Dates (Target)
| Milestone | Target | Criteria |
|-----------|--------|----------|
| **GATE 1: Developer Preview** | Month 1 | Kernel builds, deterministic replay |
| **GATE 2: Real AI Workflow** | Month 2 | Live model provider, agent proposals |
| **GATE 3: Component-Aware** | Month 3 | Evidence-backed components, assets |
| **GATE 4: Schematic-Capable** | Month 4 | Schematic model, ERC, KiCad round-trip |
| **GATE 5: PCB-Capable** | Month 5 | PCB model, DRC, KiCad round-trip |
| **GATE 6: Verified Workflow** | Month 6 | Power/SI/EMC/DFM, confidence gates |
| **GATE 7: Manufacturing Package** | Month 7 | BOM, Gerber, Drill, PnP, fabricator pass |
| **GATE 8: Public V1** | Month 8 | Signed installers, docs, examples, CI green |
| **GATE 9: Mature Product** | Month 12+ | Project lifecycle, observability, ecosystem |

---

## Build & Packaging

### Kernel Crates (Published to crates.io)
```toml
# eak/Cargo.toml (workspace)
[workspace]
version = "0.1.0"
members = ["crates/*"]

[workspace.package]
publish = true
repository = "https://github.com/.../electronics-agent-kit"
license = "MIT OR Apache-2.0"
authors = ["EAK Contributors"]
```

### Tauri Desktop App (`app/`)
- **Linux**: `.AppImage`, `.deb`, `.rpm`, `.tar.gz` (signed)
- **macOS**: `.dmg` (notarized, signed), `.tar.gz`
- **Windows**: `.msi` (signed), `.exe` (NSIS), `.zip`

### Installer Requirements
- **Code Signing** — All binaries signed (cosign for Linux, Apple Developer ID for macOS, EV cert for Windows).
- **SBOM** — CycloneDX JSON generated for each release (`syft`).
- **Provenance** — SLSA Level 3 build provenance (GitHub Actions OIDC).
- **Checksums** — SHA256 for all artifacts, published with release.

### Build Matrix
| Platform | Arch | CI Runner | Signing |
|----------|------|-----------|---------|
| Linux | x86_64 | `ubuntu-latest` | cosign (keyless) |
| Linux | aarch64 | `ubuntu-latest` (emulated) | cosign |
| macOS | x86_64 | `macos-latest` | Apple Developer ID |
| macOS | arm64 | `macos-latest` | Apple Developer ID |
| Windows | x86_64 | `windows-latest` | EV cert (GitHub Secrets) |

---

## CI/CD Pipeline (GitHub Actions)

### Workflow: `release.yml`
```yaml
on:
  push:
    tags: ['v*']
  workflow_dispatch:
    inputs:
      version:
        type: string
        required: true

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: No-mistakes gate
        run: no-mistakes axi run --intent "Release v${{ github.ref_name }}"
      - name: Cargo test (all)
        run: cargo test --workspace --all-targets

  build-kernel:
    needs: validate
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build release
        run: cargo build --workspace --release --target ${{ matrix.target }}
      - name: Upload artifacts
        uses: actions/upload-artifact@v4

  build-app:
    needs: validate
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Setup Tauri
        uses: tauri-apps/tauri-action@v0
      - name: Build app
        run: cargo tauri build
      - name: Sign artifacts
        # Platform-specific signing steps
      - name: Upload artifacts
        uses: actions/upload-artifact@v4

  sbom:
    needs: [build-kernel, build-app]
    runs-on: ubuntu-latest
    steps:
      - name: Generate SBOM
        run: syft packages dir:artifacts -o cyclonedx-json=sbom.json
      - name: Attest SLSA
        uses: slsa-framework/slsa-github-generator@v1

  release:
    needs: [build-kernel, build-app, sbom]
    runs-on: ubuntu-latest
    permissions:
      contents: write
      id-token: write
    steps:
      - name: Download artifacts
        uses: actions/download-artifact@v4
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/*
          generate_release_notes: true
      - name: Publish to crates.io
        run: cargo publish --workspace
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
```

---

## Signing & Verification

### Linux (cosign keyless)
```bash
cosign sign-blob --yes --bundle=bundle.sig artifact.tar.gz
cosign verify-blob --bundle=bundle.sig artifact.tar.gz
```

### macOS (Apple Developer ID)
```bash
codesign --sign "Developer ID Application: ..." --timestamp --options runtime app.dmg
spctl -a -v app.dmg
xcrun notarytool submit app.dmg --keychain-profile "NOTARY" --wait
xcrun stapler staple app.dmg
```

### Windows (EV Certificate)
```bash
signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 app.msi
signtool verify /pa /v app.msi
```

### Verification by Users
```bash
# Verify SHA256
sha256sum -c CHECKSUMS.sha256

# Verify cosign (Linux)
cosign verify-blob --bundle=bundle.sig artifact.tar.gz

# Verify notarization (macOS)
spctl -a -v app.dmg
```

---

## Release Artifacts Checklist

| Artifact | Required | Signed | SBOM | Notes |
|----------|----------|--------|------|-------|
| Kernel crates (crates.io) | ✅ | ✅ (cargo) | ✅ | `cargo publish` |
| `eak` CLI binary | ✅ | ✅ | ✅ | All platforms |
| `eak-app` Tauri installers | ✅ | ✅ | ✅ | All platforms |
| Source tarball | ✅ | ✅ | ✅ | GitHub Release |
| SBOM (CycloneDX) | ✅ | — | — | `sbom.json` |
| SLSA Provenance | ✅ | — | — | `provenance.intoto.jsonl` |
| Checksums (SHA256) | ✅ | — | — | `CHECKSUMS.sha256` |
| Changelog | ✅ | — | — | `CHANGELOG.md` |

---

## Rollback Procedure

1. **Identify Issue** — Critical bug in stable release.
2. **Yank Crates** — `cargo yank eak-runtime@0.1.2` (all kernel crates).
3. **Delete GitHub Release** — Remove artifacts, mark as "Pre-release".
4. **Hotfix Branch** — `hotfix/v0.1.3` from `v0.1.1` (last good).
5. **Fix + Test** — Minimal fix, full no-mistakes gate.
6. **Re-release** — `v0.1.3` with hotfix.
7. **Post-mortem** — Published within 48h.

---

## Deprecation Policy

- **API Deprecation** — Minimum 2 minor versions (e.g., deprecate in 0.3, remove in 0.5).
- **Event Schema Changes** — Migration path required; old events replayable.
- **Capability Seam Changes** — New capability added; old deprecated with adapter.
- **CLI Command Changes** — `--deprecated` flag, hidden help, 2 versions.

---

## Next Document

[Master Implementation Roadmap →](../16-master-implementation-roadmap/README.md)