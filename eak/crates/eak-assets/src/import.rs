//! Manual asset import functionality.

use crate::asset::{AssetRecord, AssetState, AssetType};
use crate::provenance::{build_provenance, format_provenance};
use crate::store::{get_asset_by_sha256, init_db, upsert_asset};
use crate::validation::{compute_sha256, validate_asset};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

/// Import an asset file manually.
pub fn import_asset(
    db_path: &Path,
    file_path: &Path,
    mpn: &str,
    manufacturer: &str,
    category: &str,
    asset_type: AssetType,
    imported_by: &str,
) -> Result<AssetRecord> {
    // Validate file exists
    if !file_path.exists() {
        bail!("File not found: {}", file_path.display());
    }

    // Validate file against asset type
    let validation_result = validate_asset(file_path, asset_type);
    if !validation_result.is_success() {
        bail!("File validation failed: {:?}", validation_result);
    }

    // Compute SHA-256
    let sha256 = compute_sha256(file_path).context("Failed to compute SHA-256")?;

    // Get file size
    let file_size = fs::metadata(file_path)
        .context("Failed to get file metadata")?
        .len();

    // Initialize database
    let conn = init_db(db_path).context("Failed to initialize database")?;

    // Check for deduplication
    if let Ok(Some(existing)) = get_asset_by_sha256(&conn, &sha256) {
        println!(
            "Asset with SHA-256 {} already exists (ID: {}), linking to component",
            &sha256[..16],
            existing.id.unwrap_or(0)
        );

        // Link existing asset to this component
        crate::store::link_asset_to_component(
            &conn,
            existing.id.unwrap(),
            mpn,
            manufacturer,
            "primary",
            true,
        )?;

        return Ok(existing);
    }

    // Determine asset root directory
    let asset_root = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("assets");

    let asset_dir = asset_root.join(asset_type.to_string());
    fs::create_dir_all(&asset_dir).context("Failed to create asset directory")?;

    // Generate local filename
    let safe_mpn = mpn.replace(['/', '\\', ':'], "_");
    let ext = asset_type.extension();
    let filename = format!("{}_{}.{}", manufacturer, safe_mpn, ext);
    let local_path = asset_dir.join(&filename);

    // Copy file to asset store
    fs::copy(file_path, &local_path).context("Failed to copy file to asset store")?;

    // Create asset record
    let mut asset = AssetRecord::new_manual(
        mpn.to_string(),
        manufacturer.to_string(),
        category.to_string(),
        asset_type,
        local_path.clone(),
        sha256,
        file_size,
        imported_by.to_string(),
    );

    // Verify signature
    if !asset.verify_signature() {
        fs::remove_file(&local_path).ok();
        bail!("File signature verification failed after copy");
    }

    // Re-validate after copy
    let validation_result = validate_asset(&local_path, asset_type);
    if !validation_result.is_success() {
        fs::remove_file(&local_path).ok();
        bail!("File validation failed after copy: {:?}", validation_result);
    }

    asset.set_state(AssetState::UserProvided);
    asset.identity_match = Some(true);

    // Store in database
    let asset_id = upsert_asset(&conn, &asset).context("Failed to store asset in database")?;

    asset.id = Some(asset_id);

    // Build and display provenance
    let provenance = build_provenance(&asset);
    println!("{}", format_provenance(&provenance));

    Ok(asset)
}

/// Import multiple assets from a directory.
pub fn import_directory(
    db_path: &Path,
    dir_path: &Path,
    mpn: &str,
    manufacturer: &str,
    category: &str,
    imported_by: &str,
) -> Result<Vec<AssetRecord>> {
    let mut imported = Vec::new();

    for entry in fs::read_dir(dir_path).context("Failed to read import directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            // Try to determine asset type from extension
            let asset_type = match path.extension().and_then(|s| s.to_str()) {
                Some("pdf") => AssetType::Datasheet,
                Some("kicad_sym") => AssetType::Symbol,
                Some("kicad_mod") => AssetType::Footprint,
                Some("step") | Some("stp") => AssetType::Model3D,
                _ => continue, // Skip unknown extensions
            };

            println!("Importing {} as {:?}...", path.display(), asset_type);

            match import_asset(
                db_path,
                &path,
                mpn,
                manufacturer,
                category,
                asset_type,
                imported_by,
            ) {
                Ok(asset) => {
                    println!("  ✓ Imported successfully (ID: {})", asset.id.unwrap_or(0));
                    imported.push(asset);
                }
                Err(e) => {
                    eprintln!("  ✗ Failed to import {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(imported)
}

/// Verify an existing asset in the database.
pub fn verify_asset(db_path: &Path, asset_id: i64) -> Result<bool> {
    let conn = init_db(db_path).context("Failed to initialize database")?;

    let asset = crate::store::get_asset_by_id(&conn, asset_id)?
        .ok_or_else(|| anyhow::anyhow!("Asset not found: {}", asset_id))?;

    let local_path = asset
        .local_path
        .ok_or_else(|| anyhow::anyhow!("Asset has no local path"))?;

    if !local_path.exists() {
        bail!("Asset file missing: {}", local_path.display());
    }

    let validation_result = validate_asset(&local_path, asset.asset_type);
    let is_valid = validation_result.is_success();

    if is_valid {
        // Recompute SHA-256 to ensure file hasn't changed
        let new_sha256 = compute_sha256(&local_path).context("Failed to compute SHA-256")?;

        if asset.sha256.as_deref() != Some(&new_sha256) {
            bail!(
                "SHA-256 mismatch! File has been modified. Expected: {}, Got: {}",
                asset.sha256.unwrap_or_default(),
                new_sha256
            );
        }

        println!("Asset {} verified successfully", asset_id);
    } else {
        println!(
            "Asset {} validation failed: {:?}",
            asset_id, validation_result
        );
    }

    Ok(is_valid)
}

/// List all assets for a component.
pub fn list_component_assets(
    db_path: &Path,
    mpn: &str,
    manufacturer: &str,
) -> Result<Vec<AssetRecord>> {
    let conn = init_db(db_path).context("Failed to initialize database")?;

    let query = crate::asset::AssetQuery::new(mpn.to_string(), manufacturer.to_string());
    let assets = crate::store::find_assets(&conn, &query)?;

    Ok(assets)
}

/// Export asset manifest to CSV.
pub fn export_manifest(db_path: &Path, output_path: &Path) -> Result<usize> {
    let conn = init_db(db_path).context("Failed to initialize database")?;

    let mut stmt =
        conn.prepare("SELECT * FROM assets ORDER BY component_mpn, manufacturer, asset_type")?;
    let assets = stmt.query_map([], crate::store::row_to_asset)?;

    let mut writer = csv::Writer::from_path(output_path).context("Failed to create CSV writer")?;

    let mut count = 0;
    for asset_result in assets {
        let asset = asset_result?;
        writer
            .write_record(&[
                asset.component_mpn,
                asset.manufacturer,
                asset.category,
                asset.asset_type.to_string(),
                asset.source_url,
                asset.final_url.unwrap_or_default(),
                asset.http_status.map(|s| s.to_string()).unwrap_or_default(),
                asset.content_type.unwrap_or_default(),
                asset.file_size.map(|s| s.to_string()).unwrap_or_default(),
                asset
                    .local_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                asset.sha256.unwrap_or_default(),
                asset.state.to_string(),
                asset.failure_reason.unwrap_or_default(),
                asset.attempts.to_string(),
                asset.last_attempt.unwrap_or_default(),
            ])
            .context("Failed to write CSV record")?;
        count += 1;
    }

    writer.flush().context("Failed to flush CSV")?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{AssetState, AssetType};
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    // Valid minimal PDF for testing - using exact bytes that work with pdfinfo
    const TEST_PDF_CONTENT: &[u8] = &[
        0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x34, 0x0a, // %PDF-1.4\n
        0x31, 0x20, 0x30, 0x20, 0x6f, 0x62, 0x6a, 0x0a, // 1 0 obj\n
        0x3c, 0x3c, 0x20, 0x2f, 0x54, 0x79, 0x70, 0x65, 0x20, 0x2f, 0x43, 0x61, 0x74, 0x61, 0x6c,
        0x6f, 0x67, 0x20, 0x2f, 0x50, 0x61, 0x67, 0x65, 0x73, 0x20, 0x32, 0x20, 0x30, 0x20, 0x52,
        0x20, 0x3e, 0x3e, 0x0a, // << /Type /Catalog /Pages 2 0 R >>\n
        0x65, 0x6e, 0x64, 0x6f, 0x62, 0x6a, 0x0a, // endobj\n
        0x32, 0x20, 0x30, 0x20, 0x6f, 0x62, 0x6a, 0x0a, // 2 0 obj\n
        0x3c, 0x3c, 0x20, 0x2f, 0x54, 0x79, 0x70, 0x65, 0x20, 0x2f, 0x50, 0x61, 0x67, 0x65, 0x73,
        0x20, 0x2f, 0x4b, 0x69, 0x64, 0x73, 0x20, 0x5b, 0x33, 0x20, 0x30, 0x20, 0x52, 0x5d, 0x20,
        0x2f, 0x43, 0x6f, 0x75, 0x6e, 0x74, 0x20, 0x31, 0x20, 0x3e, 0x3e,
        0x0a, // << /Type /Pages /Kids [3 0 R] /Count 1 >>\n
        0x65, 0x6e, 0x64, 0x6f, 0x62, 0x6a, 0x0a, // endobj\n
        0x33, 0x20, 0x30, 0x20, 0x6f, 0x62, 0x6a, 0x0a, // 3 0 obj\n
        0x3c, 0x3c, 0x20, 0x2f, 0x54, 0x79, 0x70, 0x65, 0x20, 0x2f, 0x50, 0x61, 0x67, 0x65, 0x20,
        0x2f, 0x50, 0x61, 0x72, 0x65, 0x6e, 0x74, 0x20, 0x32, 0x20, 0x30, 0x20, 0x52, 0x20, 0x2f,
        0x4d, 0x65, 0x64, 0x69, 0x61, 0x42, 0x6f, 0x78, 0x20, 0x5b, 0x30, 0x20, 0x30, 0x20, 0x36,
        0x31, 0x32, 0x20, 0x37, 0x39, 0x32, 0x5d, 0x20, 0x3e, 0x3e,
        0x0a, // << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\n
        0x65, 0x6e, 0x64, 0x6f, 0x62, 0x6a, 0x0a, // endobj\n
        0x78, 0x72, 0x65, 0x66, 0x0a, // xref\n
        0x30, 0x20, 0x34, 0x0a, // 0 4\n
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x20, 0x36, 0x35,
        0x35, 0x33, 0x35, 0x20, 0x66, 0x20, 0x0a, // 0000000000 65535 f \n
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x39, 0x20, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x20, 0x6e, 0x20, 0x0a, // 0000000009 00000 n \n
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x35, 0x38, 0x20, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x20, 0x6e, 0x20, 0x0a, // 0000000058 00000 n \n
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x31, 0x31, 0x35, 0x20, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x20, 0x6e, 0x20, 0x0a, // 0000000115 00000 n \n
        0x74, 0x72, 0x61, 0x69, 0x6c, 0x65, 0x72, 0x0a, // trailer\n
        0x3c, 0x3c, 0x20, 0x2f, 0x53, 0x69, 0x7a, 0x65, 0x20, 0x34, 0x20, 0x2f, 0x52, 0x6f, 0x6f,
        0x74, 0x20, 0x31, 0x20, 0x30, 0x20, 0x52, 0x20, 0x3e, 0x3e,
        0x0a, // << /Size 4 /Root 1 0 R >>\n
        0x73, 0x74, 0x61, 0x72, 0x74, 0x78, 0x72, 0x65, 0x66, 0x0a, // startxref\n
        0x32, 0x30, 0x30, 0x0a, // 200\n
        0x25, 0x25, 0x45, 0x4f, 0x46, 0x0a, // %%EOF\n
    ];

    #[test]
    fn test_import_pdf() {
        let temp_db = NamedTempFile::new().unwrap();
        let _temp_dir = tempdir().unwrap();

        // Create a valid PDF file
        let mut pdf_file = NamedTempFile::new().unwrap();
        pdf_file.write_all(TEST_PDF_CONTENT).unwrap();

        let asset = import_asset(
            temp_db.path(),
            pdf_file.path(),
            "TEST123",
            "TestMfr",
            "Resistor",
            AssetType::Datasheet,
            "test_user",
        )
        .unwrap();

        assert_eq!(asset.component_mpn, "TEST123");
        assert_eq!(asset.manufacturer, "TestMfr");
        assert_eq!(asset.asset_type, AssetType::Datasheet);
        assert_eq!(asset.state, AssetState::UserProvided);
        assert!(asset.sha256.is_some());
        assert!(asset.local_path.is_some());
        assert!(asset.local_path.as_ref().unwrap().exists());
    }

    #[test]
    fn test_import_rejects_invalid_pdf() {
        let temp_db = NamedTempFile::new().unwrap();

        // Create an invalid PDF (HTML content)
        let mut html_file = NamedTempFile::new().unwrap();
        html_file
            .write_all(b"<html><body>Not a PDF</body></html>")
            .unwrap();

        let result = import_asset(
            temp_db.path(),
            html_file.path(),
            "TEST123",
            "TestMfr",
            "Resistor",
            AssetType::Datasheet,
            "test_user",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_import_deduplication() {
        let temp_db = NamedTempFile::new().unwrap();
        let _temp_dir = tempdir().unwrap();

        // Create a valid PDF file
        let mut pdf_file = NamedTempFile::new().unwrap();
        pdf_file.write_all(TEST_PDF_CONTENT).unwrap();

        // Import first time
        let asset1 = import_asset(
            temp_db.path(),
            pdf_file.path(),
            "TEST123",
            "TestMfr",
            "Resistor",
            AssetType::Datasheet,
            "test_user",
        )
        .unwrap();

        // Import same file for different MPN
        let asset2 = import_asset(
            temp_db.path(),
            pdf_file.path(),
            "TEST456",
            "TestMfr",
            "Resistor",
            AssetType::Datasheet,
            "test_user",
        )
        .unwrap();

        // Should be deduplicated (same ID)
        assert_eq!(asset1.id, asset2.id);
    }

    #[test]
    fn test_verify_asset() {
        let temp_db = NamedTempFile::new().unwrap();
        let _temp_dir = tempdir().unwrap();

        let mut pdf_file = NamedTempFile::new().unwrap();
        pdf_file.write_all(TEST_PDF_CONTENT).unwrap();

        let asset = import_asset(
            temp_db.path(),
            pdf_file.path(),
            "TEST123",
            "TestMfr",
            "Resistor",
            AssetType::Datasheet,
            "test_user",
        )
        .unwrap();

        let asset_id = asset.id.unwrap();

        // Verify should succeed
        let result = verify_asset(temp_db.path(), asset_id).unwrap();
        assert!(result);
    }

    #[test]
    fn test_list_component_assets() {
        let temp_db = NamedTempFile::new().unwrap();
        let _temp_dir = tempdir().unwrap();

        let mut pdf_file = NamedTempFile::new().unwrap();
        pdf_file.write_all(TEST_PDF_CONTENT).unwrap();

        import_asset(
            temp_db.path(),
            pdf_file.path(),
            "TEST123",
            "TestMfr",
            "Resistor",
            AssetType::Datasheet,
            "test_user",
        )
        .unwrap();

        let assets = list_component_assets(temp_db.path(), "TEST123", "TestMfr").unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].component_mpn, "TEST123");
    }
}
