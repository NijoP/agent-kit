# EAK Asset Architecture

## Overview
This document describes the asset management architecture for the EAK component library system. It defines how assets (datasheets, symbols, footprints, 3D models) are discovered, acquired, verified, stored, and linked to component records.

## Core Principles

1. **Asset != Component** - A component is a design entity; assets are physical files that provide realizations of that component.
2. **Provenance First** - Every asset must have auditable provenance (source, acquisition method, verification status).
3. **Verification is Mandatory** - An asset is not "verified" until it passes structural validation (PDF signature, STEP syntax, etc.).
4. **Honest Blocking** - Anti-bot protection, paywalls, and access controls are recorded as `BLOCKED`, not retried indefinitely.
5. **Multiple Assets Per Component** - One component can have multiple assets of the same type (datasheet + application note).

## Asset Types

| Type | Extensions | Validation | Typical Size |
|------|------------|------------|--------------|
| Datasheet | `.pdf` | PDF signature + pdfinfo parse | 100KB - 5MB |
| Symbol | `.kicad_sym` | KiCad S-expression parse | 1-10KB |
| Footprint | `.kicad_mod` | KiCad S-expression parse | 1-10KB |
| 3D Model | `.step`, `.stp` | STEP header parse (ISO-10303) | 50KB - 5MB |
| Application Note | `.pdf` | PDF signature + parse | 100KB - 2MB |

## Asset State Machine

```
DISCOVERED
    | (source URL identified)
    v
SOURCE_FOUND
    | (download initiated)
    v
DOWNLOADING
    | (HTTP 200 + content)
    v
DOWNLOADED
    | (magic bytes match)
    v
SIGNATURE_VALID
    | (parser succeeds)
    v
PARSED
    | (identity matches component)
    v
IDENTITY_VERIFIED
    |
    v
VERIFIED ──────────────────────────┐
    |                              |
    v                              v
BLOCKED                        NOT_FOUND
    |                              |
    └──────────────────────────────┘
         (terminal states)

Additional states:
- INVALID_SIGNATURE (magic bytes mismatch)
- PARSE_FAILED (parser error)
- IDENTITY_MISMATCH (asset doesn't match component)
- ACCESS_FORBIDDEN (403)
- SOURCE_NOT_FOUND (404)
- HTTP_ERROR (5xx, timeout, network)
- CLOUDFLARE_CHALLENGE (anti-bot detected)
- USER_PROVIDED (manually added by engineer)
```

## Asset Provenance Model

Each asset record contains:

```sql
CREATE TABLE assets (
    id INTEGER PRIMARY KEY,
    component_id INTEGER REFERENCES parts(rowid),
    asset_type TEXT NOT NULL CHECK (asset_type IN ('datasheet', 'symbol', 'footprint', 'model3d', 'app_note')),
    source_url TEXT NOT NULL,
    final_url TEXT,
    http_status INTEGER,
    content_type TEXT,
    file_size INTEGER,
    local_path TEXT UNIQUE,
    sha256 TEXT NOT NULL,
    state TEXT NOT NULL,
    failure_reason TEXT,
    attempts INTEGER DEFAULT 0,
    source_provider TEXT,
    http_headers JSON,
    downloaded_at TIMESTAMP,
    verified_at TIMESTAMP,
    sha256 TEXT NOT NULL,
    file_signature TEXT,
    parser_version TEXT,
    identity_match BOOLEAN,
    notes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (component_id) REFERENCES parts(rowid)
);

CREATE INDEX idx_assets_component ON assets(component_id);
CREATE INDEX idx_assets_state ON assets(state);
CREATE INDEX idx_assets_sha256 ON assets(sha256);
```

## Asset Provider Abstraction

```rust
pub trait AssetProvider: Send + Sync {
    async fn discover(&self, component: &ComponentQuery) -> Result<Vec<AssetSource>, ProviderError>;
    
    async fn acquire(&self, source: &AssetSource, dest: &Path) -> Result<AssetRecord, ProviderError>;
    
    fn provider_id(&self) -> &'static str;
    
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct AssetSource {
    pub url: String,
    pub asset_type: AssetType,
    pub provider: &'static str,
    pub confidence: f32,
    pub metadata: HashMap<String, String>,
}

pub enum AssetType {
    Datasheet,
    Symbol,
    Footprint,
    Model3D,
    AppNote,
}
```

### Provider Implementations

| Provider | Source | Auth | Rate Limit | Notes |
|----------|--------|------|------------|-------|
| `ManufacturerProvider` | Manufacturer websites | None/Session | Respect robots.txt | HTML scraping required |
| `DistributorProvider` | DigiKey, Mouser, Farnell, Arrow | API key | Per docs | Official APIs preferred |
| `CommercialApiProvider` | Octopart, SnapEDA, ComponentSearchEngine | API key | Per plan | Best for datasheets |
| `LocalFilesystemProvider` | Local directories | None | None | User-provided assets |
| `UserProvidedProvider` | Manual upload | None | None | Engineer override |

## File Validation Pipeline

```rust
pub fn validate_asset(file_path: &Path, expected_type: AssetType) -> ValidationResult {
    // 1. File exists and readable
    let metadata = fs::metadata(file_path)?;
    if metadata.len() == 0 { return Err(ValidationError::EmptyFile); }
    
    // 2. Magic bytes / file signature
    let mut file = File::open(file_path)?;
    let mut header = [0u8; 16];
    file.read_exact(&mut header)?;
    
    let signature = match expected_type {
        AssetType::Datasheet | AssetType::AppNote => {
            if header.starts_with(b"%PDF-") { "PDF" } else { "UNKNOWN" }
        }
        AssetType::Symbol => {
            if header.starts_with(b"(kicad_symbol_lib") { "KICAD_SYM" } else { "UNKNOWN" }
        }
        AssetType::Footprint => {
            if header.starts_with(b"(footprint") { "KICAD_MOD" } else { "UNKNOWN" }
        }
        AssetType::Model3D => {
            if header.starts_with(b"ISO-10303") || &header[0..4] == b"ISO-" { "STEP" } else { "UNKNOWN" }
        }
    };
    
    if signature == "UNKNOWN" {
        return Err(ValidationError::InvalidSignature);
    }
    
    // Parser validation
    match expected_type {
        AssetType::Datasheet | AssetType::AppNote => {
            Command::new("pdfinfo").arg(file_path).output()?.status.success()
        }
        AssetType::Symbol | AssetType::Footprint => {
            let content = fs::read_to_string(file_path)?;
            content.contains("(kicad_symbol_lib") || content.contains("(footprint")
        }
        AssetType::Model3D => {
            let content = fs::read_to_string(file_path)?;
            content.contains("ISO-10303-21") || content.contains("FILE_SCHEMA")
        }
    }
}
```

## Download Pipeline

```rust
pub async fn download_asset(
    provider: &dyn AssetProvider,
    source: &AssetSource,
    dest_dir: &Path,
    component: &ComponentQuery,
) -> Result<AssetRecord, DownloadError> {
    // 1. Check if already exists and verified
    if let Some(existing) = find_verified_asset(&component, source.asset_type) {
        return Ok(existing);
    }
    
    // 2. Check storage space
    check_storage_space(dest_dir, MIN_FREE_SPACE_BYTES)?;
    
    // 2. Attempt download with retries
    let mut attempts = 0;
    let max_attempts = 3;
    
    while attempts < max_attempts {
        attempts += 1;
        
        match provider.acquire(source, &temp_path).await {
            Ok(record) => {
                if validate_asset(&temp_path, source.asset_type).is_ok() {
                    let sha256 = compute_sha256(&temp_path);
                    
                    if let Some(existing) = find_by_sha256(&sha256) {
                        link_component_to_asset(&component, &existing);
                        return Ok(existing);
                    }
                    
                    let final_path = dest_dir / format!("{}_{}.{}", 
                        sanitize(component.manufacturer), 
                        sanitize(component.mpn),
                        extension_for(source.asset_type)
                    );
                    fs::rename(&temp_path, &final_path)?;
                    
                    return Ok(AssetRecord {
                        state: AssetState::VERIFIED,
                        sha256: compute_sha256(&final_path),
                        local_path: final_path.to_string_lossy().to_string(),
                    });
                }
            }
            Err(e) => {
                log::warn!("Attempt {}/{} failed: {}", attempts, max_attempts, e);
                if attempts < max_attempts {
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
                }
            }
        }
    }
    
    Err(DownloadError::MaxAttemptsExceeded)
}
```

## Component-to-Asset Linking

```sql
CREATE TABLE component_assets (
    component_id INTEGER REFERENCES parts(rowid),
    asset_id INTEGER REFERENCES assets(id),
    role TEXT NOT NULL,
    is_primary BOOLEAN DEFAULT FALSE,
    notes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (component_id, asset_id, role)
);

CREATE TRIGGER ensure_single_primary
BEFORE INSERT ON component_assets
WHEN NEW.is_primary = 1
BEGIN
    UPDATE component_assets 
    SET is_primary = 0 
    WHERE component_id = NEW.component_id 
      AND role = NEW.role 
      AND is_primary = 1;
END;
```

## Blocked Source Handling

```rust
match error {
    DownloadError::CloudflareChallenge => {
        record_asset_state(AssetState::CLOUDFLARE_CHALLENGE);
        // DO NOT RETRY - record and move on
    }
    DownloadError::AccessForbidden => {
        record_asset_state(AssetState::ACCESS_FORBIDDEN);
    }
    DownloadError::NotFound => {
        record_asset_state(AssetState::NOT_FOUND);
    }
    DownloadError::HttpError(status) if status >= 500 => {
        retry_with_backoff();
    }
}
```

**Key Rule**: Never loop indefinitely on blocked sources. Record the failure, preserve the source URL for manual investigation, and continue with other components.

## Asset Sharing / Deduplication

Multiple components can share the same physical asset file:

```sql
-- Example: Many 0402 resistors share the same generic footprint
SELECT component_id, asset_id, role 
FROM component_assets 
WHERE asset_id = (SELECT id FROM assets WHERE local_path LIKE '%Resistor_0402%')
```

## Migration Strategy

### Phase 1: Add assets table (non-breaking)
```sql
-- Add assets table
-- Add component_assets link table
-- Keep existing parts.datasheet column for backward compatibility
```

### Phase 2: Populate from manifest
```sql
-- Import from datasheet_manifest.csv
-- Link existing parts to verified assets (currently 0)
-- Mark blocked assets with state = 'ACCESS_BLOCKED'
```

### Phase 3: Deprecate parts.datasheet
```sql
-- After migration verified
-- ALTER TABLE parts DROP COLUMN datasheet;
-- Update PartCatalog to return asset IDs instead of URLs
```

## Testing Requirements

| Test | Description |
|------|-------------|
| `test_pdf_signature_validation` | Rejects HTML with .pdf extension |
| `test_step_signature_validation` | Rejects non-STEP files |
| `test_kicad_symbol_parse` | Valid KiCad symbol parses |
| `test_kicad_footprint_parse` | Valid KiCad footprint parses |
| `test_html_rejection` | HTML saved as .pdf is rejected |
| `test_cloudflare_challenge_detection` | Cloudflare page detected and blocked |
| `test_403_handling` | 403 recorded as ACCESS_FORBIDDEN |
| `test_404_handling` | 404 recorded as SOURCE_NOT_FOUND |
| `test_sha256_deduplication` | Same file downloaded twice links to same asset |
| `test_storage_threshold` | Download blocked when space < threshold |
| `test_idempotent_import` | Re-importing manifest doesn't duplicate |
| `test_asset_sharing` | Multiple components can reference same asset |

## Integration Points

1. **PartCatalog** - Returns `Vec<AssetSource>` instead of just URLs
2. **BOM Planning** - Checks asset availability before selecting parts
3. **Schematic Generator** - Verifies symbol/footprint assets exist
4. **PCB Placement** - Verifies footprint assets exist
5. **Manufacturing Generation** - Packages all verified assets for fab
5. **Agent** - Can request asset acquisition via `AssetProvider` tool

## Configuration

```toml
# config/assets.toml
[storage]
min_free_space_gb = 30
asset_root = "data/assets"
datasheets_dir = "datasheets"
symbols_dir = "symbols"
footprints_dir = "footprints"
models3d_dir = "models3d"

[download]
max_concurrent = 4
retry_attempts = 3
retry_backoff_base_secs = 2
timeout_secs = 30
user_agent = "EAK/0.1 (+https://github.com/electronics-agent-kit)"

[providers]
manufacturer_enabled = true
distributor_enabled = true
commercial_api_enabled = false
local_filesystem_enabled = true

[validation]
pdf_require_pdfinfo = true
step_require_header = true
kicad_require_sexpr = true
```

## Future Extensions

1. **Commercial API Integration** - Plug in Octopart/SnapEDA when API keys available
2. **Versioned Assets** - Track datasheet revisions (rev A, rev B)
3. **Asset Expiration** - Flag datasheets older than N years for review
4. **Delta Sync** - Periodic re-check of blocked sources
5. **User Library** - Separate namespace for user-provided assets
6. **Asset Search** - Full-text search across datasheet content
7. **BOM Export** - Package all verified assets for manufacturing