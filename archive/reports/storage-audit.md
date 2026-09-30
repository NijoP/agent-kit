# EAK Storage Audit Report

## Executive Summary
Storage audit performed on the EAK repository to understand current disk usage and plan for future asset storage requirements.

## Current Disk Usage (as of audit)

| Path | Size | Notes |
|------|------|-------|
| `/home/dev/electronics-agent-kit/data/` | 392K | Database + imports + assets |
| `/home/dev/electronics-agent-kit/data/assets/` | 140K | Symbols, footprints, empty datasheets/models3d |
| `/home/dev/electronics-agent-kit/data/imports/` | 72K | Day 1 CSV imports |
| `/home/dev/electronics-agent-kit/scripts/` | 115M | Day 1 scripts (mostly target/ build artifacts) |
| `/home/dev/electronics-agent-kit/eak/target/` | 3.0G | Rust build artifacts (cargo target) |
| `/home/dev/electronics-agent-kit/.git/` | 5.4M | Git repository |
| **Total (excluding target/)** | **~5.5M** | Source + data + git |
| **Available space** | **91G** | On /dev/sda3 (219G total, 117G used) |

## Database Size
- `data/library.db`: 180KB (500 component records with FTS5)
- Expected growth: ~360 bytes/record for metadata + FTS overhead

## Asset Directory Structure
```
data/assets/
├── datasheets/     (0 bytes - 0 real PDFs, all blocked)
├── symbols/        12K (3 KiCad symbol files)
├── footprints/     12K (4 KiCad footprint files)
└── models3d/       0 bytes (no STEP models available)
```

## Projected Storage Requirements

### Day 1 (Current - 500 components)
| Asset Type | Count | Est. Size/Item | Total |
|------------|-------|----------------|-------|
| Datasheets (PDF) | 500 | 500KB avg | ~250MB |
| Symbols (KiCad) | 3 generic | 4KB avg | ~12KB |
| Footprints | 4 packages | 4KB avg | ~16KB |
| 3D Models (STEP) | 67 | 200KB avg | ~13MB |
| **Total** | | | **~263MB** |

### Full Library Target (3,500 components)
| Asset Type | Count | Est. Size/Item | Total |
|------------|-------|----------------|-------|
| Datasheets | 3,500 | 500KB avg | ~1.75GB |
| Symbols | ~50 unique | 4KB avg | ~200KB |
| Footprints | ~20 packages | 4KB avg | ~80KB |
| 3D Models | ~500 | 200KB avg | ~100MB |
| **Total** | | | **~1.85GB** |

### 10,000 Components (Future)
| Asset Type | Count | Est. Size/Item | Total |
|------------|-------|----------------|-------|
| Datasheets | 10,000 | 500KB avg | ~5GB |
| Symbols | ~100 unique | 4KB avg | ~400KB |
| Footprints | ~50 packages | 4KB avg | ~200KB |
| 3D Models | ~1,500 | 200KB avg | ~300MB |
| **Total** | | | **~5.3GB** |

## Storage Threshold Recommendations

| Threshold | Action |
|-----------|--------|
| 80% (175GB used) | Warning: Prepare for cleanup/archival |
| 90% (197GB used) | Block new asset downloads |
| 95% (208GB used) | Emergency: require manual intervention |

## Git Repository Strategy

### Currently Tracked
- Source code (Rust, TypeScript, Python)
- Documentation (docs/, reports/, story/)
- Configuration files (Cargo.toml, package.json, etc.)
- Build scripts (scripts/day1/*, but not target/ directories)

### Currently NOT Tracked (in .gitignore)
- `eak/target/` - Rust build artifacts
- `app/src-tauri/target/` - Tauri build artifacts
- `app/node_modules/` - npm dependencies
- `app/dist/` - Vite build output
- `app/src-tauri/gen/` - Tauri generated code
- `**/*.rs.bk` - Rust backup files

### Large Binary Asset Policy (RECOMMENDED)
| Asset Type | Git Strategy |
|------------|--------------|
| Database (`library.db`) | **Track** - Small (180KB), versioned with schema |
| Symbols/Footprints | **Track** - Small text files, versioned |
| Datasheets (PDF) | **Git LFS** or external object storage |
| 3D Models (STEP) | **Git LFS** or external object storage |
| Build artifacts | **Never track** |

### Git LFS Configuration (Recommended)
```bash
# .gitattributes
*.pdf filter=lfs diff=lfs merge=lfs -text
*.step filter=lfs diff=lfs merge=lfs -text
*.stp filter=lfs diff=lfs merge=lfs -text
```

## Storage Threshold Implementation

### Recommended Implementation
```rust
// In asset downloader / library manager
const MIN_FREE_SPACE_BYTES: u64 = 30 * 1024 * 1024 * 1024; // 30GB minimum

fn check_storage_space(path: &Path) -> Result<(), StorageError> {
    let fs = std::fs::metadata(path)?;
    // Use statvfs on Unix or GetDiskFreeSpaceEx on Windows
    let free_space = get_free_space(path)?;
    if free_space < MIN_FREE_SPACE_BYTES {
        return Err(StorageError::InsufficientSpace(free_space));
    }
    Ok(())
}
```

### Integration Points
1. Asset downloader: Check before each download batch
2. Library import: Check before bulk import
3. Build system: Check before cargo build (cargo needs ~2-3GB for target/)

## Current State Summary

| Metric | Value | Status |
|--------|-------|--------|
| Total repository (excl target) | ~5.5MB | ✅ Minimal |
| Available disk space | 91GB | ✅ Abundant |
| Database size | 180KB | ✅ Tiny |
| Asset directory | 140KB | ✅ Empty (no real assets) |
| Build artifacts | 3.0GB | ⚠️ Large (target/ excluded from git) |
| Git repo | 5.4MB | ✅ Healthy |

## Recommendations

1. **Implement storage threshold check** in asset downloader before each download batch
2. **Add `.gitattributes` for Git LFS** for future PDF/STEP assets
3. **Document Git LFS setup** in CONTRIBUTING.md
4. **Consider external object storage** (S3, Cloudflare R2) for production asset hosting
5. **Add storage check to CI/CD** to prevent pipeline failures
6. **Archive old build artifacts** periodically (cargo clean)
7. **Monitor target/ directory size** in CI/CD