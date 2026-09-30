# EAK Documentation Index

This directory contains the canonical documentation for the Electronics Agent Kit (EAK), organized by architectural layer.

## Documentation Hierarchy

1. [Product Vision](1-product-vision/README.md) — Why EAK exists, what it is, and what it is not.
2. [Product Specification](2-product-spec/README.md) — User-facing capabilities, interfaces, and acceptance criteria.
3. [System Architecture](3-system-architecture/README.md) — High-level system decomposition, runtime boundaries, and data flows.
4. [Engineering Model](4-engineering-model/README.md) — The canonical engineering data model (entities, relationships, provenance).
5. [Component Architecture](5-component-architecture/README.md) — Component intelligence: sourcing, evidence, symbols, footprints, 3D.
6. [Model/Provider Architecture](6-model-provider-architecture/README.md) — Reasoning provider abstraction, capabilities, credentials, streaming.
7. [Agent Architecture](7-agent-architecture/README.md) — Agent roles, proposal/validation loop, capability seams, bounded retries.
8. [Terminal/Herdr Architecture](8-terminal-herdr-architecture/README.md) — Integration with Herdr for persistent sessions, PTY, workspaces.
9. [KiCad Integration Architecture](9-kicad-integration-architecture/README.md) — KiCad as an edge EDA implementation peripheral.
10. [UI Architecture](10-ui-architecture/README.md) — UI as a projection/editor surface, not authority.
11. [Verification Architecture](11-verification-architecture/README.md) — Electrical rules, power, signal integrity, DFM, confidence/fidelity.
12. [Project/Persistence Architecture](12-project-persistence-architecture/README.md) — Project lifecycle, revision, snapshots, crash recovery.
13. [Development Workflow](13-development-workflow/README.md) — Agent change → tests → lint → docs → AI review → no-mistakes gate → PR.
14. [Testing Strategy](14-testing-strategy/README.md) — Unit, integration, contract, property, golden, chaos, and verification tests.
15. [Release Strategy](15-release-strategy/README.md) — Versioning, packaging, signed builds, installers, CI/CD.
16. [Master Implementation Roadmap](16-master-implementation-roadmap/README.md) — Dependency-driven stages with measurable gates.

## Reading Order

For new contributors, read in numerical order. Each document assumes knowledge of the previous ones.

## Architecture Principles (Non-Negotiable)

All documents must adhere to the [12 non-negotiable architecture principles](1-product-vision/README.md#non-negotiable-architecture-principles).

## Versioning

Documentation version follows the EAK product version. Each document carries a `version:` field in its front matter.