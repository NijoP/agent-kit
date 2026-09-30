# Project/Persistence Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines project lifecycle, revision control, snapshots, crash recovery, event log architecture, and persistence adapters.

---

## Project Model

```rust
struct Project {
    id: EntityId,                    // ULID
    name: String,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    root_path: PathBuf,              // Project directory
    settings: ProjectSettings,
    current_revision: EntityId,      // Head revision
}

struct ProjectSettings {
    default_grid_mm: f32,
    pcb_stackup_template: Option<EntityId>,
    verification_profile: VerificationProfile,
    ki_cad_version: String,
    model_provider_preferences: HashMap<AgentRole, String>,
    export_defaults: ExportDefaults,
}
```

### Project Directory Layout
```
<project_root>/
├── eak.project.json          # Project metadata (id, name, settings)
├── event_log.jsonl           # Append-only event log (primary)
├── event_log.sqlite          # Optional: SQLite index for fast query
├── snapshots/                # Periodic state snapshots (compressed)
│   ├── snapshot_000001.msgpack.zst
│   └── ...
├── kicad/                    # KiCad project (synced)
│   ├── <project>.kicad_pro
│   ├── <project>.kicad_sch
│   ├── <project>.kicad_pcb
│   └── lib/
├── assets/                   # Component assets (symbols, footprints, 3D)
├── datasheets/               # Cached PDFs
├── exports/                  # Manufacturing outputs (gerber, bom, etc.)
├── reports/                  # Verification reports, review artifacts
└── herdr/                    # Herdr session state (managed by Herdr)
```

---

## Event Log Architecture

### Format: JSON Lines (Primary)
- One JSON object per line.
- Immutable, append-only.
- Content-addressed: each event has `blake3_hash` of its canonical JSON.
- Sequence number (`seq`) provides total order.

```json
{"seq": 1, "timestamp": "2025-01-15T10:30:00Z", "event": {"RequirementAdded": {...}}, "provenance": {...}, "correlation_id": "..."}
{"seq": 2, "timestamp": "2025-01-15T10:30:01Z", "event": {"ComponentSelected": {...}}, "provenance": {...}, "correlation_id": "..."}
```

### Event Schema Evolution
- Each event variant has `schema_version`.
- Kernel migrations (`eak-runtime/src/migrations/`) transform old events on replay.
- Migrations are pure functions, tested for idempotence.

### Indexing (SQLite Adapter)
- `event_log.sqlite` provides:
  - B-tree on `seq` for fast seek.
  - FTS5 on event content for search.
  - Materialized views for common queries (e.g., `latest_state`).
- SQLite is **derived**, not authoritative. Rebuilt from JSONL on demand.

---

## State Fold (Deterministic Reconstruction)

```rust
fn fold(state: EngineeringState, event: EventRecord) -> EngineeringState {
    match event.event {
        Event::RequirementAdded(r) => state.add_requirement(r),
        Event::ComponentSelected(c) => state.add_component(c),
        // ... 50+ variants
    }
}
```

### Properties
- **Pure** — No side effects, no I/O, no randomness.
- **Deterministic** — Same event log → identical state.
- **Associative** — `fold(fold(s, e1), e2) == fold(s, [e1, e2])`.
- **Parallelizable** — Can fold segments in parallel then merge (CRDT-like for independent entities).

---

## Snapshots (Checkpointing)

### Purpose
- Fast startup: replay from latest snapshot + tail of log.
- Time-travel: jump to any snapshot.

### Snapshot Format
- **MessagePack + Zstd** — Compact, fast decode.
- Contains full `EngineeringState` (all entities, indices).
- Written every `N` events (configurable, default 1000) or on `Revision` creation.

### Snapshot Metadata
```rust
struct SnapshotMeta {
    seq: u64,                        // Event sequence at snapshot
    revision_id: Option<EntityId>,   // If snapshot == revision
    timestamp: DateTime<Utc>,
    size_bytes: u64,
    hash: Blake3Hash,                // Content hash
}
```

### Startup Recovery
1. Find latest snapshot (`snapshots/*.msgpack.zst`).
2. Deserialize → `EngineeringState`.
3. Replay events from `snapshot.seq + 1` to end of log.
4. Verify final state hash matches expected (if revision exists).

---

## Revision Control

### Revision (Immutable Snapshot with Label)
```rust
struct Revision {
    id: EntityId,
    project_id: EntityId,
    parent_revision_id: Option<EntityId>,
    event_seq: u64,
    label: String,                   // e.g., "v1.0-rc1", "pre-schematic-review"
    message: String,                 // Human description
    author: Provenance,
    created_at: DateTime<Utc>,
    tags: Vec<String>,               // "release", "review", "milestone"
}
```

### Revision Graph
- Linear by default (each revision has one parent).
- Branching: `Revision` can have multiple children (explicit `branch_from`).
- Merging: Not in V1 (future: three-way merge of engineering models).

### Creating a Revision
1. User/Lead Agent triggers `CreateRevision` capability.
2. Kernel writes current state as snapshot.
3. Kernel appends `RevisionCreated` event with `event_seq = current_seq`.
4. `Project.current_revision` updated.

### Diffing Revisions
```rust
struct RevisionDiff {
    from_rev: EntityId,
    to_rev: EntityId,
    requirements: Diff<Requirement>,
    components: Diff<Component>,
    schematic: Diff<Schematic>,
    pcb: Diff<Pcb>,
    verification: Diff<VerificationResult>,
    bom: Diff<Bom>,
}
```
- Diff algorithm: structural (entity ID based), not textual.
- Shows added/removed/modified entities with provenance.

---

## Crash Recovery

### Failure Modes
| Mode | Detection | Recovery |
|------|-----------|----------|
| **Kernel panic** | Process exit | Restart kernel → replay from last snapshot |
| **Power loss** | Incomplete JSONL line | Truncate incomplete line → replay |
| **Disk full** | Write error | Alert → pause → resume after space freed |
| **Corrupted event** | JSON parse error / hash mismatch | Quarantine event → replay from before → alert |

### Recovery Procedure
1. **Integrity Check** — Scan event log for valid JSONL + hash match.
2. **Find Last Good Snapshot** — Verify snapshot hash.
3. **Replay** — Fold from snapshot to end of valid log.
4. **Validate** — Run ERC/DRC on recovered state.
5. **Report** — Generate recovery report with any data loss.

### Guarantees
- **No silent corruption** — Hash mismatch = hard error.
- **Bounded data loss** — At most events since last snapshot (≤1000 by default).
- **Deterministic** — Recovery produces same state as uninterrupted run.

---

## Persistence Adapters (`eak-store`)

### Trait
```rust
#[async_trait]
pub trait EventLog: Send + Sync {
    async fn append(&self, event: EventRecord) -> Result<u64, StoreError>; // Returns seq
    async fn read(&self, seq: u64) -> Result<Option<EventRecord>, StoreError>;
    async fn read_range(&self, start: u64, end: u64) -> Result<Vec<EventRecord>, StoreError>;
    async fn tail(&self, from_seq: u64) -> Result<EventStream, StoreError>;
    async fn snapshot(&self, state: &EngineeringState, meta: SnapshotMeta) -> Result<(), StoreError>;
    async fn load_snapshot(&self, seq: u64) -> Result<Option<(EngineeringState, SnapshotMeta)>, StoreError>;
}
```

### Implementations

| Adapter | Backend | Use Case |
|---------|---------|----------|
| `JsonLinesEventLog` | File (append-only) | Primary, air-gapped, simple |
| `SqliteEventLog` | SQLite (WAL mode) | Fast query, indexing, concurrent readers |
| `PostgresEventLog` | PostgreSQL | Multi-user, remote, Herdr session backend |

### Configuration
```toml
[store]
primary = "jsonl"           # Always write here first
index = "sqlite"            # Optional async indexer
remote = "postgres"         # Optional replication
snapshot_interval = 1000    # Events
snapshot_compression = "zstd"
```

---

## Concurrency & Locking

- **Single Writer** — Kernel is sole writer to event log (enforced by architecture).
- **Multiple Readers** — UI, Herdr, CLI, CI all read via `EventLog` trait.
- **SQLite WAL** — Allows concurrent readers during write.
- **File Lock** — `JsonLinesEventLog` uses advisory lock (`flock`) for safety.

---

## Backup & Export

### Project Archive (`.eakproj`)
- Zipped project directory (excluding `herdr/`, `event_log.sqlite`).
- Contains: `eak.project.json`, `event_log.jsonl`, `snapshots/`, `kicad/`, `assets/`, `exports/`.
- Portable, versioned, can be imported into another EAK instance.

### Git Integration (Optional)
- `event_log.jsonl` can be tracked in Git (line-based, mergeable with care).
- Snapshots binary — store in Git LFS or artifact store.
- `.eakproj` export for sharing with non-EAK users.

---

## Next Document

[Development Workflow →](../13-development-workflow/README.md)