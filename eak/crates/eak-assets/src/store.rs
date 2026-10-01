//! SQLite storage for assets.

use crate::asset::{AssetQuery, AssetRecord, AssetState, AssetType};
use crate::facts::{DatasheetFact, FactKind};
use crate::parser::ParseResult;
use anyhow;
use once_cell::sync::Lazy;
use rusqlite::{params, Connection, OptionalExtension, Result, Row};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Global database connection (for simplicity in CLI tools).
static DB_CONNECTION: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the asset database.
pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Create assets table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            component_mpn TEXT NOT NULL,
            manufacturer TEXT NOT NULL,
            category TEXT NOT NULL,
            asset_type TEXT NOT NULL,
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
            last_attempt TEXT,
            source_provider TEXT NOT NULL,
            http_headers TEXT,
            downloaded_at TEXT,
            verified_at TEXT,
            file_signature TEXT,
            parser_version TEXT,
            identity_match INTEGER,
            acquisition_method TEXT NOT NULL DEFAULT 'automatic',
            notes TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
        [],
    )?;

    // Create indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_assets_component ON assets(component_mpn, manufacturer)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_assets_state ON assets(state)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_assets_sha256 ON assets(sha256)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_assets_type ON assets(asset_type)",
        [],
    )?;

    // Create component_assets link table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS component_assets (
            component_mpn TEXT NOT NULL,
            manufacturer TEXT NOT NULL,
            asset_id INTEGER NOT NULL,
            role TEXT NOT NULL DEFAULT 'primary',
            is_primary INTEGER DEFAULT 1,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (component_mpn, manufacturer, asset_id, role),
            FOREIGN KEY (asset_id) REFERENCES assets(id)
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_component_assets_component ON component_assets(component_mpn, manufacturer)",
        [],
    )?;

    // Create datasheet_facts table for extracted facts
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS datasheet_facts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_id INTEGER NOT NULL,
            fact_kind TEXT NOT NULL,
            value_magnitude REAL NOT NULL,
            value_unit TEXT NOT NULL,
            value_dimension TEXT NOT NULL,
            tolerance_type TEXT,
            tolerance_plus REAL,
            tolerance_minus REAL,
            tolerance_relative REAL,
            conditions TEXT, -- JSON array
            provenance_asset_sha256 TEXT,
            provenance_page INTEGER,
            provenance_extracted_text TEXT,
            provenance_parser_version TEXT,
            provenance_extracted_at TEXT,
            provenance_verified INTEGER DEFAULT 0,
            provenance_manually_corrected INTEGER DEFAULT 0,
            provenance_correction_notes TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (asset_id) REFERENCES assets(id)
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_datasheet_facts_asset ON datasheet_facts(asset_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_datasheet_facts_kind ON datasheet_facts(fact_kind)",
        [],
    )?;

    // Create parse_results table for parser output
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS parse_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_id INTEGER NOT NULL UNIQUE,
            page_count INTEGER,
            title TEXT,
            detected_manufacturer TEXT,
            detected_mpn TEXT,
            sections TEXT, -- JSON
            parser_version TEXT,
            parsed_at TEXT,
            warnings TEXT, -- JSON array
            success INTEGER NOT NULL,
            error TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (asset_id) REFERENCES assets(id)
        )
        "#,
        [],
    )?;

    // Create identity_verifications table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS identity_verifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_id INTEGER NOT NULL UNIQUE,
            result TEXT NOT NULL, -- IDENTITY_VERIFIED, IDENTITY_MISMATCH, IDENTITY_UNCERTAIN
            manufacturer_match INTEGER,
            manufacturer_component TEXT,
            manufacturer_datasheet TEXT,
            manufacturer_confidence REAL,
            mpn_match INTEGER,
            mpn_component TEXT,
            mpn_datasheet TEXT,
            mpn_confidence REAL,
            package_match INTEGER,
            package_component TEXT,
            package_datasheet TEXT,
            package_confidence REAL,
            overall_confidence REAL,
            summary TEXT,
            verified_at TEXT NOT NULL,
            FOREIGN KEY (asset_id) REFERENCES assets(id)
        )
        "#,
        [],
    )?;

    // Create cross_check_reports table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS cross_check_reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_id INTEGER NOT NULL,
            component_mpn TEXT NOT NULL,
            component_manufacturer TEXT NOT NULL,
            component_class TEXT NOT NULL,
            assessment TEXT NOT NULL, -- CLEAN, INCOMPLETE, HAS_CONFLICTS, SUSPICIOUS
            total_fields INTEGER,
            matches INTEGER,
            conflicts INTEGER,
            missing_in_datasheet INTEGER,
            datasheet_only INTEGER,
            both_missing INTEGER,
            incomparable INTEGER,
            field_comparisons TEXT, -- JSON
            checked_at TEXT NOT NULL,
            FOREIGN KEY (asset_id) REFERENCES assets(id)
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cross_check_asset ON cross_check_reports(asset_id)",
        [],
    )?;

    Ok(conn)
}

/// Set the global database connection.
pub fn set_global_db(conn: Connection) {
    *DB_CONNECTION.lock().unwrap() = Some(conn);
}

/// Get the global database connection.
pub fn get_global_db(db_path: &Path) -> Result<Connection> {
    init_db(db_path)
}

/// Insert or update an asset record.
pub fn upsert_asset(conn: &Connection, asset: &AssetRecord) -> Result<i64> {
    let sha256 = asset.sha256.as_deref().unwrap_or("");

    // Check if asset with same SHA-256 already exists (deduplication)
    if !sha256.is_empty() {
        let mut stmt = conn.prepare("SELECT id FROM assets WHERE sha256 = ?1")?;
        if let Ok(existing_id) = stmt.query_row(params![sha256], |row| row.get::<_, i64>(0)) {
            // Asset already exists, link to this component
            link_asset_to_component(
                conn,
                existing_id,
                &asset.component_mpn,
                &asset.manufacturer,
                "primary",
                true,
            )?;
            return Ok(existing_id);
        }
    }

    // Insert new asset
    conn.execute(
        r#"
        INSERT INTO assets (
            component_mpn, manufacturer, category, asset_type,
            source_url, final_url, http_status, content_type, file_size,
            local_path, sha256, state, failure_reason, attempts, last_attempt,
            source_provider, http_headers, downloaded_at, verified_at,
            file_signature, parser_version, identity_match,
            acquisition_method, notes, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26
        )
        "#,
        params![
            asset.component_mpn,
            asset.manufacturer,
            asset.category,
            asset.asset_type.to_string(),
            asset.source_url,
            asset.final_url,
            asset.http_status,
            asset.content_type,
            asset.file_size,
            asset
                .local_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            sha256,
            asset.state.to_string(),
            asset.failure_reason,
            asset.attempts,
            asset.last_attempt,
            asset.source_provider,
            asset.http_headers,
            asset.downloaded_at,
            asset.verified_at,
            asset.file_signature,
            asset.parser_version,
            asset.identity_match.map(|b| if b { 1 } else { 0 }),
            asset.acquisition_method,
            asset.notes,
            asset.created_at,
            asset.updated_at,
        ],
    )?;

    let asset_id = conn.last_insert_rowid();

    // Link to component
    link_asset_to_component(
        conn,
        asset_id,
        &asset.component_mpn,
        &asset.manufacturer,
        "primary",
        true,
    )?;

    Ok(asset_id)
}

/// Update an existing asset record.
pub fn update_asset(conn: &Connection, asset: &AssetRecord) -> Result<()> {
    let asset_id = asset.id.ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("Asset ID required for update".into())
    })?;

    conn.execute(
        r#"
        UPDATE assets SET
            component_mpn = ?1,
            manufacturer = ?2,
            category = ?3,
            asset_type = ?4,
            source_url = ?5,
            final_url = ?6,
            http_status = ?7,
            content_type = ?8,
            file_size = ?9,
            local_path = ?10,
            sha256 = ?11,
            state = ?12,
            failure_reason = ?13,
            attempts = ?14,
            last_attempt = ?15,
            source_provider = ?16,
            http_headers = ?17,
            downloaded_at = ?18,
            verified_at = ?19,
            file_signature = ?20,
            parser_version = ?21,
            identity_match = ?22,
            acquisition_method = ?23,
            notes = ?24,
            updated_at = ?25
        WHERE id = ?26
        "#,
        params![
            asset.component_mpn,
            asset.manufacturer,
            asset.category,
            asset.asset_type.to_string(),
            asset.source_url,
            asset.final_url,
            asset.http_status,
            asset.content_type,
            asset.file_size,
            asset
                .local_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            asset.sha256.as_deref().unwrap_or(""),
            asset.state.to_string(),
            asset.failure_reason,
            asset.attempts,
            asset.last_attempt,
            asset.source_provider,
            asset.http_headers,
            asset.downloaded_at,
            asset.verified_at,
            asset.file_signature,
            asset.parser_version,
            asset.identity_match.map(|b| if b { 1 } else { 0 }),
            asset.acquisition_method,
            asset.notes,
            asset.updated_at,
            asset_id,
        ],
    )?;

    Ok(())
}

/// Link asset to component.
pub fn link_asset_to_component(
    conn: &Connection,
    asset_id: i64,
    component_mpn: &str,
    manufacturer: &str,
    role: &str,
    is_primary: bool,
) -> Result<()> {
    conn.execute(
        r#"
        INSERT OR REPLACE INTO component_assets (
            component_mpn, manufacturer, asset_id, role, is_primary, notes, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            component_mpn,
            manufacturer,
            asset_id,
            role,
            if is_primary { 1 } else { 0 },
            "",
            chrono::Utc::now().to_rfc3339(),
        ],
    )?;

    Ok(())
}

/// Find assets for a component query.
pub fn find_assets(conn: &Connection, query: &AssetQuery) -> Result<Vec<AssetRecord>> {
    let mut sql = String::from(
        r#"
        SELECT a.* FROM assets a
        INNER JOIN component_assets ca ON a.id = ca.asset_id
        WHERE ca.component_mpn = ?1 AND ca.manufacturer = ?2
        "#,
    );

    let mut param_strings = vec![query.mpn.clone(), query.manufacturer.clone()];

    if !query.asset_types.is_empty() {
        let placeholders = query
            .asset_types
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        sql.push_str(&format!(" AND a.asset_type IN ({})", placeholders));
        for at in &query.asset_types {
            param_strings.push(at.to_string());
        }
    }

    if query.only_verified {
        sql.push_str(" AND a.state IN ('VERIFIED', 'USER_PROVIDED')");
    }

    sql.push_str(" ORDER BY a.created_at DESC");

    // Build params from the owned strings
    let params: Vec<&str> = param_strings.iter().map(|s| s.as_str()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let asset_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), row_to_asset)?;

    let mut assets = Vec::new();
    for asset in asset_iter {
        assets.push(asset?);
    }

    Ok(assets)
}

/// Get asset by ID.
pub fn get_asset_by_id(conn: &Connection, id: i64) -> Result<Option<AssetRecord>> {
    let mut stmt = conn.prepare("SELECT * FROM assets WHERE id = ?1")?;
    let asset = stmt.query_row(params![id], row_to_asset).optional()?;
    Ok(asset)
}

/// Get asset by SHA-256 (for deduplication).
pub fn get_asset_by_sha256(conn: &Connection, sha256: &str) -> Result<Option<AssetRecord>> {
    let mut stmt = conn.prepare("SELECT * FROM assets WHERE sha256 = ?1")?;
    let asset = stmt.query_row(params![sha256], row_to_asset).optional()?;
    Ok(asset)
}

/// Get all assets with a specific state.
pub fn get_assets_by_state(conn: &Connection, state: AssetState) -> Result<Vec<AssetRecord>> {
    let mut stmt =
        conn.prepare("SELECT * FROM assets WHERE state = ?1 ORDER BY created_at DESC")?;
    let asset_iter = stmt.query_map(params![state.to_string()], row_to_asset)?;

    let mut assets = Vec::new();
    for asset in asset_iter {
        assets.push(asset?);
    }

    Ok(assets)
}

/// Get asset statistics.
pub fn get_asset_stats(conn: &Connection) -> Result<AssetStats> {
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM assets", [], |r| r.get(0))?;
    let verified: i64 = conn.query_row(
        "SELECT COUNT(*) FROM assets WHERE state IN ('VERIFIED', 'USER_PROVIDED')",
        [],
        |r| r.get(0),
    )?;
    let blocked: i64 = conn.query_row(
        "SELECT COUNT(*) FROM assets WHERE state IN ('ACCESS_BLOCKED', 'CLOUDFLARE_CHALLENGE', 'ACCESS_FORBIDDEN')",
        [],
        |r| r.get(0)
    )?;
    let not_found: i64 = conn.query_row(
        "SELECT COUNT(*) FROM assets WHERE state = 'SOURCE_NOT_FOUND'",
        [],
        |r| r.get(0),
    )?;
    let pending: i64 = conn.query_row(
        "SELECT COUNT(*) FROM assets WHERE state NOT IN ('VERIFIED', 'USER_PROVIDED', 'ACCESS_BLOCKED', 'CLOUDFLARE_CHALLENGE', 'ACCESS_FORBIDDEN', 'SOURCE_NOT_FOUND')",
        [],
        |r| r.get(0)
    )?;

    let by_type: Vec<(String, i64)> = {
        let mut stmt =
            conn.prepare("SELECT asset_type, COUNT(*) FROM assets GROUP BY asset_type")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<Result<Vec<_>>>()?
    };

    let by_method: Vec<(String, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT acquisition_method, COUNT(*) FROM assets GROUP BY acquisition_method",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<Result<Vec<_>>>()?
    };

    Ok(AssetStats {
        total,
        verified,
        blocked,
        not_found,
        pending,
        by_type,
        by_method,
    })
}

/// Asset statistics.
#[derive(Debug, Clone, Serialize)]
pub struct AssetStats {
    pub total: i64,
    pub verified: i64,
    pub blocked: i64,
    pub not_found: i64,
    pub pending: i64,
    pub by_type: Vec<(String, i64)>,
    pub by_method: Vec<(String, i64)>,
}

/// Convert database row to AssetRecord.
pub fn row_to_asset(row: &Row) -> Result<AssetRecord> {
    let asset_type_str: String = row.get("asset_type")?;
    let asset_type = match asset_type_str.as_str() {
        "Datasheet" => AssetType::Datasheet,
        "Symbol" => AssetType::Symbol,
        "Footprint" => AssetType::Footprint,
        "Model3D" => AssetType::Model3D,
        "AppNote" => AssetType::AppNote,
        _ => AssetType::Datasheet,
    };

    let state_str: String = row.get("state")?;
    let state = match state_str.as_str() {
        "SOURCE_DECLARED" => AssetState::SourceDeclared,
        "SOURCE_RESOLVED" => AssetState::SourceResolved,
        "DOWNLOAD_ATTEMPTED" => AssetState::DownloadAttempted,
        "DOWNLOADED" => AssetState::Downloaded,
        "SIGNATURE_VALID" => AssetState::SignatureValid,
        "PARSED" => AssetState::Parsed,
        "IDENTITY_VERIFIED" => AssetState::IdentityVerified,
        "VERIFIED" => AssetState::Verified,
        "SOURCE_NOT_FOUND" => AssetState::SourceNotFound,
        "HTTP_ERROR" => AssetState::HttpError,
        "ACCESS_FORBIDDEN" => AssetState::AccessForbidden,
        "HTML_RESPONSE" => AssetState::HtmlResponse,
        "INVALID_SIGNATURE" => AssetState::InvalidSignature,
        "PARSE_FAILED" => AssetState::ParseFailed,
        "IDENTITY_MISMATCH" => AssetState::IdentityMismatch,
        "ACCESS_BLOCKED" => AssetState::AccessBlocked,
        "CLOUDFLARE_CHALLENGE" => AssetState::CloudflareChallenge,
        "USER_PROVIDED" => AssetState::UserProvided,
        "REQUIRES_MANUAL_ACQUISITION" => AssetState::RequiresManualAcquisition,
        _ => AssetState::SourceDeclared,
    };

    let local_path: Option<String> = row.get("local_path")?;
    let identity_match: Option<i64> = row.get("identity_match")?;

    Ok(AssetRecord {
        id: Some(row.get("id")?),
        component_mpn: row.get("component_mpn")?,
        manufacturer: row.get("manufacturer")?,
        category: row.get("category")?,
        asset_type,
        source_url: row.get("source_url")?,
        final_url: row.get("final_url")?,
        http_status: row.get("http_status")?,
        content_type: row.get("content_type")?,
        file_size: row.get("file_size")?,
        local_path: local_path.map(PathBuf::from),
        sha256: row.get("sha256")?,
        state,
        failure_reason: row.get("failure_reason")?,
        attempts: row.get("attempts")?,
        last_attempt: row.get("last_attempt")?,
        source_provider: row.get("source_provider")?,
        http_headers: row.get("http_headers")?,
        downloaded_at: row.get("downloaded_at")?,
        verified_at: row.get("verified_at")?,
        file_signature: row.get("file_signature")?,
        parser_version: row.get("parser_version")?,
        identity_match: identity_match.map(|v| v != 0),
        acquisition_method: row.get("acquisition_method")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// Import manifest CSV into database.
pub fn import_manifest(conn: &Connection, manifest_path: &Path) -> anyhow::Result<usize> {
    let mut reader = csv::Reader::from_path(manifest_path)?;
    let mut count = 0;

    for result in reader.deserialize() {
        let record: ManifestRecord = result?;

        let state = match record.state.as_str() {
            "VERIFIED" => AssetState::Verified,
            "ACCESS_BLOCKED" => AssetState::AccessBlocked,
            "SOURCE_NOT_FOUND" => AssetState::SourceNotFound,
            "CLOUDFLARE_CHALLENGE" => AssetState::CloudflareChallenge,
            "HTTP_ERROR" => AssetState::HttpError,
            "ACCESS_FORBIDDEN" => AssetState::AccessForbidden,
            "HTML_RESPONSE" => AssetState::HtmlResponse,
            "INVALID_SIGNATURE" => AssetState::InvalidSignature,
            "PARSE_FAILED" => AssetState::ParseFailed,
            "IDENTITY_MISMATCH" => AssetState::IdentityMismatch,
            "USER_PROVIDED" => AssetState::UserProvided,
            _ => AssetState::SourceDeclared,
        };

        let last_attempt = record.last_attempt.clone();

        let asset = AssetRecord {
            id: None,
            component_mpn: record.mpn,
            manufacturer: record.manufacturer,
            category: record.category,
            asset_type: AssetType::Datasheet,
            source_url: record.source_url,
            final_url: if record.final_url.is_empty() {
                None
            } else {
                Some(record.final_url)
            },
            http_status: if record.http_status == 0 {
                None
            } else {
                Some(record.http_status)
            },
            content_type: if record.content_type.is_empty() {
                None
            } else {
                Some(record.content_type)
            },
            file_size: if record.file_size == 0 {
                None
            } else {
                Some(record.file_size)
            },
            local_path: if record.local_path.is_empty() {
                None
            } else {
                Some(PathBuf::from(record.local_path))
            },
            sha256: if record.sha256.is_empty() {
                None
            } else {
                Some(record.sha256)
            },
            state,
            failure_reason: if record.failure_reason.is_empty() {
                None
            } else {
                Some(record.failure_reason)
            },
            attempts: record.attempts,
            last_attempt: if last_attempt.is_empty() {
                None
            } else {
                Some(last_attempt.clone())
            },
            source_provider: "day1_import".to_string(),
            http_headers: None,
            downloaded_at: None,
            verified_at: None,
            file_signature: None,
            parser_version: None,
            identity_match: None,
            acquisition_method: "automatic".to_string(),
            notes: Some("Imported from Day 1 manifest".to_string()),
            created_at: last_attempt.clone(),
            updated_at: last_attempt,
        };

        upsert_asset(conn, &asset)?;
        count += 1;
    }

    Ok(count)
}

/// Manifest record from CSV.
#[derive(Debug, Deserialize)]
struct ManifestRecord {
    mpn: String,
    manufacturer: String,
    category: String,
    source_url: String,
    final_url: String,
    http_status: u16,
    content_type: String,
    file_size: u64,
    local_path: String,
    sha256: String,
    state: String,
    failure_reason: String,
    attempts: u32,
    last_attempt: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{AssetRecord, AssetState, AssetType};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_db_init_and_upsert() {
        let temp_db = NamedTempFile::new().unwrap();
        let conn = init_db(temp_db.path()).unwrap();

        let asset = AssetRecord::new(
            "TEST123".to_string(),
            "TestMfr".to_string(),
            "Resistor".to_string(),
            AssetType::Datasheet,
            "https://example.com/datasheet.pdf".to_string(),
            "TestMfr".to_string(),
        );

        let id = upsert_asset(&conn, &asset).unwrap();
        assert!(id > 0);

        let retrieved = get_asset_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(retrieved.component_mpn, "TEST123");
        assert_eq!(retrieved.state, AssetState::SourceDeclared);
    }

    #[test]
    fn test_deduplication_by_sha256() {
        let temp_db = NamedTempFile::new().unwrap();
        let conn = init_db(temp_db.path()).unwrap();

        let mut asset1 = AssetRecord::new(
            "TEST123".to_string(),
            "TestMfr".to_string(),
            "Resistor".to_string(),
            AssetType::Datasheet,
            "https://example.com/datasheet.pdf".to_string(),
            "TestMfr".to_string(),
        );
        asset1.set_state(AssetState::Verified);
        asset1.sha256 = Some("abc123".to_string());
        asset1.local_path = Some(PathBuf::from("datasheets/test.pdf"));
        asset1.file_size = Some(1000);

        let id1 = upsert_asset(&conn, &asset1).unwrap();

        // Try to insert same SHA-256 with different MPN
        let mut asset2 = AssetRecord::new(
            "TEST456".to_string(),
            "TestMfr".to_string(),
            "Resistor".to_string(),
            AssetType::Datasheet,
            "https://example.com/datasheet2.pdf".to_string(),
            "TestMfr".to_string(),
        );
        asset2.set_state(AssetState::Verified);
        asset2.sha256 = Some("abc123".to_string()); // Same SHA-256
        asset2.local_path = Some(PathBuf::from("datasheets/test2.pdf"));
        asset2.file_size = Some(1000);

        let id2 = upsert_asset(&conn, &asset2).unwrap();

        // Should return same ID (deduplicated)
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_find_assets() {
        let temp_db = NamedTempFile::new().unwrap();
        let conn = init_db(temp_db.path()).unwrap();

        let mut asset = AssetRecord::new(
            "RC0402FR-071RL".to_string(),
            "Yageo".to_string(),
            "Resistor".to_string(),
            AssetType::Datasheet,
            "https://example.com/datasheet.pdf".to_string(),
            "Yageo".to_string(),
        );
        asset.set_state(AssetState::Verified);

        upsert_asset(&conn, &asset).unwrap();

        let query = crate::asset::AssetQuery::for_datasheet(
            "RC0402FR-071RL".to_string(),
            "Yageo".to_string(),
        );
        let assets = find_assets(&conn, &query).unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].component_mpn, "RC0402FR-071RL");
    }

    #[test]
    fn test_asset_stats() {
        let temp_db = NamedTempFile::new().unwrap();
        let conn = init_db(temp_db.path()).unwrap();

        // Insert verified asset
        let mut asset1 = AssetRecord::new(
            "TEST1".to_string(),
            "Mfr1".to_string(),
            "Resistor".to_string(),
            AssetType::Datasheet,
            "https://example.com/1.pdf".to_string(),
            "Mfr1".to_string(),
        );
        asset1.set_state(AssetState::Verified);
        upsert_asset(&conn, &asset1).unwrap();

        // Insert blocked asset
        let mut asset2 = AssetRecord::new(
            "TEST2".to_string(),
            "Mfr2".to_string(),
            "Capacitor".to_string(),
            AssetType::Datasheet,
            "https://example.com/2.pdf".to_string(),
            "Mfr2".to_string(),
        );
        asset2.set_state(AssetState::AccessBlocked);
        upsert_asset(&conn, &asset2).unwrap();

        let stats = get_asset_stats(&conn).unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.verified, 1);
        assert_eq!(stats.blocked, 1);
    }
}

/// Store extracted datasheet facts.
pub fn store_datasheet_facts(
    conn: &Connection,
    asset_id: i64,
    facts: &[DatasheetFact],
) -> Result<()> {
    for fact in facts {
        let tolerance_json = serde_json::to_string(&fact.parameter.tolerance).unwrap_or_default();
        let conditions_json = serde_json::to_string(&fact.parameter.conditions).unwrap_or_default();

        conn.execute(
            r#"
            INSERT INTO datasheet_facts (
                asset_id, fact_kind,
                value_magnitude, value_unit, value_dimension,
                tolerance_type, tolerance_plus, tolerance_minus, tolerance_relative,
                conditions,
                provenance_asset_sha256, provenance_page, provenance_extracted_text,
                provenance_parser_version, provenance_extracted_at,
                provenance_verified, provenance_manually_corrected, provenance_correction_notes,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20
            )
            "#,
            params![
                asset_id,
                format!("{:?}", fact.kind),
                fact.parameter.value.magnitude,
                format!("{:?}", fact.parameter.value.unit),
                format!("{:?}", fact.parameter.value.dimension()),
                fact.parameter.tolerance.as_ref().map(|t| format!("{:?}", t)).unwrap_or_default(),
                fact.parameter.tolerance.as_ref().and_then(|t| match t {
                    eak_units::Tolerance::Absolute { plus, .. } => Some(*plus),
                    _ => None,
                }),
                fact.parameter.tolerance.as_ref().and_then(|t| match t {
                    eak_units::Tolerance::Absolute { minus, .. } => Some(*minus),
                    _ => None,
                }),
                fact.parameter.tolerance.as_ref().and_then(|t| match t {
                    eak_units::Tolerance::Relative(r) => Some(*r),
                    _ => None,
                }),
                conditions_json,
                fact.provenance.asset_sha256,
                fact.provenance.page as i64,
                fact.provenance.extracted_text,
                fact.provenance.parser_version,
                fact.provenance.extracted_at,
                if fact.provenance.verified { 1 } else { 0 },
                if fact.provenance.manually_corrected { 1 } else { 0 },
                fact.provenance.correction_notes,
                fact.created_at,
                fact.updated_at,
            ],
        )?;
    }
    Ok(())
}

/// Get datasheet facts for an asset.
pub fn get_datasheet_facts(conn: &Connection, asset_id: i64) -> Result<Vec<DatasheetFact>> {
    let mut stmt =
        conn.prepare("SELECT * FROM datasheet_facts WHERE asset_id = ?1 ORDER BY fact_kind")?;
    let rows = stmt.query_map(params![asset_id], |row| {
        Ok(DatasheetFact {
            id: Some(row.get("id")?),
            asset_id: row.get("asset_id")?,
            kind: match row.get::<_, String>("fact_kind")?.as_str() {
                "Voltage" => FactKind::Voltage,
                "Current" => FactKind::Current,
                "Resistance" => FactKind::Resistance,
                "Capacitance" => FactKind::Capacitance,
                "Inductance" => FactKind::Inductance,
                "Power" => FactKind::Power,
                "Frequency" => FactKind::Frequency,
                "Temperature" => FactKind::Temperature,
                "TemperatureMin" => FactKind::TemperatureMin,
                "TemperatureMax" => FactKind::TemperatureMax,
                "Manufacturer" => FactKind::Manufacturer,
                "Mpn" => FactKind::Mpn,
                "Package" => FactKind::Package,
                "LifecycleStatus" => FactKind::LifecycleStatus,
                "Tolerance" => FactKind::Tolerance,
                "Dimension" => FactKind::Dimension,
                _ => FactKind::Other,
            },
            parameter: crate::facts::ParameterFact {
                value: eak_units::PhysicalQuantity::new(
                    row.get("value_magnitude")?,
                    match row.get::<_, String>("value_unit")?.as_str() {
                        "Volt" => eak_units::Unit::Volt,
                        "Millivolt" => eak_units::Unit::Millivolt,
                        "Ampere" => eak_units::Unit::Ampere,
                        "Milliampere" => eak_units::Unit::Milliampere,
                        "Ohm" => eak_units::Unit::Ohm,
                        "Kilohm" => eak_units::Unit::Kilohm,
                        "Farad" => eak_units::Unit::Farad,
                        "Microfarad" => eak_units::Unit::Microfarad,
                        "Nanofarad" => eak_units::Unit::Nanofarad,
                        "Picofarad" => eak_units::Unit::Picofarad,
                        "Henry" => eak_units::Unit::Henry,
                        "Microhenry" => eak_units::Unit::Microhenry,
                        "Nanohenry" => eak_units::Unit::Nanohenry,
                        "Watt" => eak_units::Unit::Watt,
                        "Milliwatt" => eak_units::Unit::Milliwatt,
                        "Hertz" => eak_units::Unit::Hertz,
                        "Kilohertz" => eak_units::Unit::Kilohertz,
                        "Megahertz" => eak_units::Unit::Megahertz,
                        "DegreeCelsius" => eak_units::Unit::DegreeCelsius,
                        "Kelvin" => eak_units::Unit::Kelvin,
                        "Unitless" => eak_units::Unit::Unitless,
                        _ => eak_units::Unit::Unitless,
                    },
                ),
                tolerance: None,
                conditions: serde_json::from_str(
                    &row.get::<_, String>("conditions").unwrap_or_default(),
                )
                .unwrap_or_default(),
            },
            provenance: crate::facts::ProvenanceInfo {
                asset_id: row.get("asset_id")?,
                asset_sha256: row.get("provenance_asset_sha256")?,
                page: row.get("provenance_page")?,
                extracted_text: row.get("provenance_extracted_text")?,
                parser_version: row.get("provenance_parser_version")?,
                extracted_at: row.get("provenance_extracted_at")?,
                verified: row.get::<_, i64>("provenance_verified")? != 0,
                manually_corrected: row.get::<_, i64>("provenance_manually_corrected")? != 0,
                correction_notes: row.get("provenance_correction_notes")?,
            },
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    })?;

    let mut facts = Vec::new();
    for fact in rows {
        facts.push(fact?);
    }
    Ok(facts)
}

/// Store parse result.
pub fn store_parse_result(conn: &Connection, result: &ParseResult) -> Result<i64> {
    let sections_json = serde_json::to_string(&result.sections).unwrap_or_default();
    let warnings_json = serde_json::to_string(&result.warnings).unwrap_or_default();

    conn.execute(
        r#"
        INSERT OR REPLACE INTO parse_results (
            asset_id, page_count, title, detected_manufacturer, detected_mpn,
            sections, parser_version, parsed_at, warnings, success, error,
            created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
        params![
            result.asset_id,
            result.page_count as i64,
            result.title,
            result.manufacturer,
            result.mpn,
            sections_json,
            result.parser_version,
            result.parsed_at,
            warnings_json,
            if result.success { 1 } else { 0 },
            result.error,
            result.parsed_at,
            result.parsed_at,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get parse result for an asset.
pub fn get_parse_result(conn: &Connection, asset_id: i64) -> Result<Option<ParseResult>> {
    let mut stmt = conn.prepare("SELECT * FROM parse_results WHERE asset_id = ?1")?;
    let result = stmt
        .query_row(params![asset_id], |row| {
            Ok(ParseResult {
                asset_id: row.get("asset_id")?,
                page_count: row.get("page_count")?,
                title: row.get("title")?,
                manufacturer: row.get("detected_manufacturer")?,
                mpn: row.get("detected_mpn")?,
                facts: Vec::new(),
                sections: serde_json::from_str(
                    &row.get::<_, String>("sections").unwrap_or_default(),
                )
                .unwrap_or_default(),
                parser_version: row.get("parser_version")?,
                parsed_at: row.get("parsed_at")?,
                warnings: serde_json::from_str(
                    &row.get::<_, String>("warnings").unwrap_or_default(),
                )
                .unwrap_or_default(),
                success: row.get::<_, i64>("success")? != 0,
                error: row.get("error")?,
            })
        })
        .optional()?;
    Ok(result)
}

/// Store identity verification result.
pub fn store_identity_verification(
    conn: &Connection,
    asset_id: i64,
    report: &crate::identity::IdentityReport,
) -> Result<i64> {
    conn.execute(
        r#"
        INSERT OR REPLACE INTO identity_verifications (
            asset_id, result,
            manufacturer_match, manufacturer_component, manufacturer_datasheet, manufacturer_confidence,
            mpn_match, mpn_component, mpn_datasheet, mpn_confidence,
            package_match, package_component, package_datasheet, package_confidence,
            overall_confidence, summary, verified_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
        "#,
        params![
            asset_id,
            format!("{:?}", report.result),
            if report.manufacturer.matches { 1 } else { 0 },
            report.manufacturer.component_value,
            report.manufacturer.datasheet_value,
            report.manufacturer.confidence,
            if report.mpn.matches { 1 } else { 0 },
            report.mpn.component_value,
            report.mpn.datasheet_value,
            report.mpn.confidence,
            report.package.as_ref().map(|p| if p.matches { 1 } else { 0 }).unwrap_or(0),
            report.package.as_ref().and_then(|p| p.component_value.clone()),
            report.package.as_ref().and_then(|p| p.datasheet_value.clone()),
            report.package.as_ref().map(|p| p.confidence).unwrap_or(0.0),
            report.confidence,
            report.summary,
            report.verified_at,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get identity verification for an asset.
pub fn get_identity_verification(
    conn: &Connection,
    asset_id: i64,
) -> Result<Option<crate::identity::IdentityReport>> {
    let mut stmt = conn.prepare("SELECT * FROM identity_verifications WHERE asset_id = ?1")?;
    let result = stmt
        .query_row(params![asset_id], |row| {
            Ok(crate::identity::IdentityReport {
                result: match row.get::<_, String>("result")?.as_str() {
                    "IdentityVerified" => crate::identity::IdentityResult::IdentityVerified,
                    "IdentityMismatch" => crate::identity::IdentityResult::IdentityMismatch,
                    "IdentityUncertain" => crate::identity::IdentityResult::IdentityUncertain,
                    _ => crate::identity::IdentityResult::IdentityUncertain,
                },
                manufacturer: crate::identity::FieldComparison {
                    field: "manufacturer".to_string(),
                    component_value: row.get("manufacturer_component")?,
                    datasheet_value: row.get("manufacturer_datasheet")?,
                    matches: row.get::<_, i64>("manufacturer_match")? != 0,
                    confidence: row.get("manufacturer_confidence")?,
                    details: String::new(),
                },
                mpn: crate::identity::FieldComparison {
                    field: "mpn".to_string(),
                    component_value: row.get("mpn_component")?,
                    datasheet_value: row.get("mpn_datasheet")?,
                    matches: row.get::<_, i64>("mpn_match")? != 0,
                    confidence: row.get("mpn_confidence")?,
                    details: String::new(),
                },
                package: None,
                confidence: row.get("overall_confidence")?,
                summary: row.get("summary")?,
                verified_at: row.get("verified_at")?,
            })
        })
        .optional()?;
    Ok(result)
}

/// Store cross-check report.
pub fn store_cross_check_report(
    conn: &Connection,
    asset_id: i64,
    report: &crate::crosscheck::CrossCheckReport,
) -> Result<i64> {
    let comparisons_json = serde_json::to_string(&report.field_comparisons).unwrap_or_default();

    conn.execute(
        r#"
        INSERT INTO cross_check_reports (
            asset_id, component_mpn, component_manufacturer, component_class,
            assessment, total_fields, matches, conflicts, missing_in_datasheet,
            datasheet_only, both_missing, incomparable, field_comparisons, checked_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        "#,
        params![
            asset_id,
            report.component_mpn,
            report.component_manufacturer,
            format!("{:?}", report.component_class),
            format!("{:?}", report.assessment),
            report.summary.total_fields_checked as i64,
            report.summary.matches as i64,
            report.summary.conflicts as i64,
            report.summary.missing_in_datasheet as i64,
            report.summary.datasheet_only as i64,
            report.summary.both_missing as i64,
            report.summary.incomparable as i64,
            comparisons_json,
            report.checked_at,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get cross-check reports for an asset.
pub fn get_cross_check_reports(
    conn: &Connection,
    asset_id: i64,
) -> Result<Vec<crate::crosscheck::CrossCheckReport>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM cross_check_reports WHERE asset_id = ?1 ORDER BY checked_at DESC",
    )?;
    let rows = stmt.query_map(params![asset_id], |row| {
        Ok(crate::crosscheck::CrossCheckReport {
            component_mpn: row.get("component_mpn")?,
            component_manufacturer: row.get("component_manufacturer")?,
            component_class: match row.get::<_, String>("component_class")?.as_str() {
                "Passive" => eak_domain::ComponentClass::Resistor,
                "Diode" => eak_domain::ComponentClass::DiodeRectifier,
                "Transistor" => eak_domain::ComponentClass::TransistorBjt,
                "AnalogIc" => eak_domain::ComponentClass::AnalogOpAmp,
                "PowerManagement" => eak_domain::ComponentClass::RegulatorLdo,
                "DigitalLogic" => eak_domain::ComponentClass::LogicGate,
                "Mcu" => eak_domain::ComponentClass::Mcu,
                "Memory" => eak_domain::ComponentClass::MemoryFlash,
                "Communication" => eak_domain::ComponentClass::CommUart,
                "Sensor" => eak_domain::ComponentClass::SensorTemperature,
                "RfWireless" => eak_domain::ComponentClass::RfTransceiver,
                "Audio" => eak_domain::ComponentClass::AudioCodec,
                "Protection" => eak_domain::ComponentClass::ProtectionTvs,
                "Connector" => eak_domain::ComponentClass::ConnectorHeader,
                "Electromechanical" => eak_domain::ComponentClass::ElectromechSwitch,
                "Specialized" => eak_domain::ComponentClass::SpecializedCrystal,
                _ => eak_domain::ComponentClass::AnalogOpAmp,
            },
            field_comparisons: serde_json::from_str(
                &row.get::<_, String>("field_comparisons")
                    .unwrap_or_default(),
            )
            .unwrap_or_default(),
            summary: crate::crosscheck::CrossCheckSummary {
                total_fields_checked: row.get("total_fields")?,
                matches: row.get("matches")?,
                conflicts: row.get("conflicts")?,
                missing_in_datasheet: row.get("missing_in_datasheet")?,
                datasheet_only: row.get("datasheet_only")?,
                both_missing: row.get("both_missing")?,
                incomparable: row.get("incomparable")?,
            },
            assessment: match row.get::<_, String>("assessment")?.as_str() {
                "Clean" => crate::crosscheck::CrossCheckAssessment::Clean,
                "Incomplete" => crate::crosscheck::CrossCheckAssessment::Incomplete,
                "HasConflicts" => crate::crosscheck::CrossCheckAssessment::HasConflicts,
                "Suspicious" => crate::crosscheck::CrossCheckAssessment::Suspicious,
                _ => crate::crosscheck::CrossCheckAssessment::Clean,
            },
            checked_at: row.get("checked_at")?,
        })
    })?;

    let mut reports = Vec::new();
    for r in rows {
        reports.push(r?);
    }
    Ok(reports)
}
