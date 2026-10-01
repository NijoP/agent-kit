//! Identity verification for datasheets vs component records.
//!
//! Verifies that a datasheet actually belongs to the component it's linked to,
//! checking manufacturer, MPN, and package where available.

use crate::asset::{AssetRecord, AssetState};
use crate::facts::{DatasheetFact, FactCollection, FactKind};
use crate::parser::ParseResult;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of identity verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentityResult {
    /// All identity fields match
    IdentityVerified,
    /// One or more identity fields mismatch
    IdentityMismatch,
    /// Insufficient information to verify
    IdentityUncertain,
}

impl std::fmt::Display for IdentityResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityResult::IdentityVerified => write!(f, "IDENTITY_VERIFIED"),
            IdentityResult::IdentityMismatch => write!(f, "IDENTITY_MISMATCH"),
            IdentityResult::IdentityUncertain => write!(f, "IDENTITY_UNCERTAIN"),
        }
    }
}

/// Detailed identity verification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityReport {
    /// Overall result
    pub result: IdentityResult,
    /// Manufacturer comparison
    pub manufacturer: FieldComparison,
    /// MPN comparison
    pub mpn: FieldComparison,
    /// Package comparison (if available)
    pub package: Option<FieldComparison>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Human-readable summary
    pub summary: String,
    /// Timestamp
    pub verified_at: String,
}

/// Comparison of a single field between component record and datasheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldComparison {
    /// Field name
    pub field: String,
    /// Value from component record
    pub component_value: Option<String>,
    /// Value extracted from datasheet
    pub datasheet_value: Option<String>,
    /// Whether they match
    pub matches: bool,
    /// Match confidence (0.0 - 1.0)
    pub confidence: f64,
    /// Details about the comparison
    pub details: String,
}

impl FieldComparison {
    fn new(field: &str, component: Option<String>, datasheet: Option<String>) -> Self {
        let (matches, confidence, details) = match (&component, &datasheet) {
            (Some(c), Some(d)) => {
                let c_norm = normalize_for_comparison(c);
                let d_norm = normalize_for_comparison(d);
                let matches = c_norm == d_norm;
                let conf = if matches { 1.0 } else { 0.0 };
                let detail = if matches {
                    "Exact match".to_string()
                } else {
                    format!("Component: '{}', Datasheet: '{}'", c, d)
                };
                (matches, conf, detail)
            }
            (Some(c), None) => (
                false,
                0.3,
                format!("Component has '{}', datasheet has no value", c),
            ),
            (None, Some(d)) => (
                false,
                0.3,
                format!("Component has no value, datasheet has '{}'", d),
            ),
            (None, None) => (true, 0.0, "Neither has a value".to_string()),
        };

        Self {
            field: field.to_string(),
            component_value: component,
            datasheet_value: datasheet,
            matches,
            confidence,
            details,
        }
    }
}

/// Normalize a string for comparison (case-insensitive, trim, remove common suffixes).
fn normalize_for_comparison(s: &str) -> String {
    let s = s.trim().to_lowercase().replace(['-', '_', ' ', '.'], "");
    // Remove common corporate suffixes only at the end of the string
    let suffixes = [
        "incorporated",
        "corporation",
        "technologies",
        "technology",
        "semiconductor",
        "electronics",
        "limited",
        "company",
        "inc",
        "corp",
        "ltd",
        "co",
    ];
    let mut result = s;
    for suffix in suffixes {
        if result.ends_with(suffix) {
            result = result[..result.len() - suffix.len()].to_string();
        }
    }
    result
}

/// Verify identity between a component record and parsed datasheet.
pub fn verify_identity(
    component_mpn: &str,
    component_manufacturer: &str,
    component_package: Option<&str>,
    parse_result: &ParseResult,
    facts: &FactCollection,
) -> IdentityReport {
    // Manufacturer comparison
    let manufacturer = FieldComparison::new(
        "manufacturer",
        Some(component_manufacturer.to_string()),
        parse_result.manufacturer.clone(),
    );

    // MPN comparison
    let mpn = FieldComparison::new(
        "mpn",
        Some(component_mpn.to_string()),
        parse_result.mpn.clone(),
    );

    // Package comparison (if available in component record or datasheet)
    let datasheet_package = facts
        .get_facts_by_kind(FactKind::Package)
        .first()
        .and_then(|f| f.parameter.conditions.first().map(|s| s.as_str()));

    let package = match (component_package, datasheet_package) {
        (Some(comp), Some(ds)) => Some(FieldComparison::new(
            "package",
            Some(comp.to_string()),
            Some(ds.to_string()),
        )),
        (Some(comp), None) => Some(FieldComparison::new(
            "package",
            Some(comp.to_string()),
            None,
        )),
        (None, Some(ds)) => Some(FieldComparison::new("package", None, Some(ds.to_string()))),
        (None, None) => None,
    };

    // Determine overall result
    let manufacturer_match = manufacturer.matches;
    let mpn_match = mpn.matches;
    let package_match = package.as_ref().map(|p| p.matches).unwrap_or(true); // Package is optional

    // Check if we have enough information from datasheet
    let has_manufacturer_info = parse_result.manufacturer.is_some();
    let has_mpn_info = parse_result.mpn.is_some();

    let result = if manufacturer_match && mpn_match && package_match {
        IdentityResult::IdentityVerified
    } else if !has_manufacturer_info && !has_mpn_info {
        // No identity information in datasheet at all - uncertain
        IdentityResult::IdentityUncertain
    } else if !manufacturer_match || !mpn_match || !package_match {
        // Have some info but it mismatches (manufacturer, MPN, or package)
        IdentityResult::IdentityMismatch
    } else {
        IdentityResult::IdentityUncertain
    };

    // Calculate confidence
    let mut confidence_sum = 0.0;
    let mut confidence_count = 0;
    for field in [&manufacturer, &mpn] {
        confidence_sum += field.confidence;
        confidence_count += 1;
    }
    if let Some(ref p) = package {
        confidence_sum += p.confidence;
        confidence_count += 1;
    }
    let confidence = if confidence_count > 0 {
        confidence_sum / confidence_count as f64
    } else {
        0.0
    };

    // Build summary
    let mut summary_parts = Vec::new();
    if manufacturer_match {
        summary_parts.push("manufacturer ✓");
    } else {
        summary_parts.push("manufacturer ✗");
    }
    if mpn_match {
        summary_parts.push("MPN ✓");
    } else {
        summary_parts.push("MPN ✗");
    }
    if let Some(ref p) = package {
        if p.matches {
            summary_parts.push("package ✓");
        } else {
            summary_parts.push("package ✗");
        }
    } else {
        summary_parts.push("package ?");
    }
    let summary = format!("Identity: {} ({})", result, summary_parts.join(", "));

    IdentityReport {
        result,
        manufacturer,
        mpn,
        package,
        confidence,
        summary,
        verified_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Verify identity using asset record and parse result directly.
pub fn verify_identity_from_asset(
    asset: &AssetRecord,
    parse_result: &ParseResult,
    facts: &FactCollection,
) -> IdentityReport {
    verify_identity(
        &asset.component_mpn,
        &asset.manufacturer,
        None, // Component package not in AssetRecord, would need ComponentMetadata
        parse_result,
        facts,
    )
}

/// Update asset state based on identity verification result.
pub fn apply_identity_result(asset: &mut AssetRecord, report: &IdentityReport) {
    match report.result {
        IdentityResult::IdentityVerified => {
            asset.set_state(AssetState::IdentityVerified);
            asset.identity_match = Some(true);
        }
        IdentityResult::IdentityMismatch => {
            asset.set_state(AssetState::IdentityMismatch);
            asset.identity_match = Some(false);
            asset.failure_reason = Some(format!("Identity mismatch: {}", report.summary));
        }
        IdentityResult::IdentityUncertain => {
            asset.identity_match = None;
            asset.failure_reason = Some(format!("Identity uncertain: {}", report.summary));
        }
    }
}

/// High-level function to verify identity and update asset.
pub fn verify_and_update_identity(
    asset: &mut AssetRecord,
    parse_result: &ParseResult,
    facts: &FactCollection,
    db_path: &std::path::Path,
) -> Result<IdentityReport> {
    let report = verify_identity_from_asset(asset, parse_result, facts);
    apply_identity_result(asset, &report);

    // Update in database
    let conn = crate::store::init_db(db_path)?;
    crate::store::update_asset(&conn, asset)?;

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{AssetRecord, AssetState, AssetType};
    use crate::facts::{DatasheetFact, FactCollection, FactKind, ParameterFact, ProvenanceInfo};
    use crate::parser::ParseResult;
    use eak_units::{PhysicalQuantity, Unit};
    use std::path::PathBuf;

    fn create_test_parse_result(manufacturer: Option<String>, mpn: Option<String>) -> ParseResult {
        ParseResult {
            asset_id: 1,
            page_count: 10,
            title: Some("LM358 Dual Op-Amp".to_string()),
            manufacturer,
            mpn,
            facts: Vec::new(),
            sections: Vec::new(),
            parser_version: "1.0.0".to_string(),
            parsed_at: chrono::Utc::now().to_rfc3339(),
            warnings: Vec::new(),
            success: true,
            error: None,
        }
    }

    fn create_test_fact_collection() -> FactCollection {
        FactCollection::new(1, "1.0.0".to_string())
    }

    #[test]
    fn test_identity_verified_exact_match() {
        let parse_result = create_test_parse_result(
            Some("Texas Instruments".to_string()),
            Some("LM358".to_string()),
        );
        let facts = create_test_fact_collection();

        let report = verify_identity("LM358", "Texas Instruments", None, &parse_result, &facts);

        assert_eq!(report.result, IdentityResult::IdentityVerified);
        assert!(report.manufacturer.matches);
        assert!(report.mpn.matches);
        assert!(report.confidence > 0.9);
    }

    #[test]
    fn test_identity_mismatch_manufacturer() {
        let parse_result = create_test_parse_result(
            Some("Analog Devices".to_string()),
            Some("LM358".to_string()),
        );
        let facts = create_test_fact_collection();

        let report = verify_identity("LM358", "Texas Instruments", None, &parse_result, &facts);

        assert_eq!(report.result, IdentityResult::IdentityMismatch);
        assert!(!report.manufacturer.matches);
        assert!(report.mpn.matches);
    }

    #[test]
    fn test_identity_mismatch_mpn() {
        let parse_result = create_test_parse_result(
            Some("Texas Instruments".to_string()),
            Some("LM324".to_string()),
        );
        let facts = create_test_fact_collection();

        let report = verify_identity("LM358", "Texas Instruments", None, &parse_result, &facts);

        assert_eq!(report.result, IdentityResult::IdentityMismatch);
        assert!(report.manufacturer.matches);
        assert!(!report.mpn.matches);
    }

    #[test]
    fn test_identity_uncertain_missing_datasheet_values() {
        let parse_result = create_test_parse_result(None, None);
        let facts = create_test_fact_collection();

        let report = verify_identity("LM358", "Texas Instruments", None, &parse_result, &facts);

        assert_eq!(report.result, IdentityResult::IdentityUncertain);
        assert!(!report.manufacturer.matches);
        assert!(!report.mpn.matches);
    }

    #[test]
    fn test_identity_case_insensitive() {
        let parse_result = create_test_parse_result(
            Some("TEXAS INSTRUMENTS".to_string()),
            Some("lm358".to_string()),
        );
        let facts = create_test_fact_collection();

        let report = verify_identity("LM358", "Texas Instruments", None, &parse_result, &facts);

        assert_eq!(report.result, IdentityResult::IdentityVerified);
    }

    #[test]
    fn test_identity_with_package_match() {
        let mut parse_result = create_test_parse_result(
            Some("Texas Instruments".to_string()),
            Some("LM358".to_string()),
        );
        let mut facts = create_test_fact_collection();

        // Add package fact
        let pkg_fact = DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::Package,
            parameter: ParameterFact {
                value: PhysicalQuantity::new(0.0, Unit::Unitless),
                tolerance: None,
                conditions: vec!["SOIC-8".to_string()],
            },
            provenance: ProvenanceInfo {
                asset_id: 1,
                asset_sha256: String::new(),
                page: 1,
                extracted_text: "Package: SOIC-8".to_string(),
                parser_version: "1.0.0".to_string(),
                extracted_at: chrono::Utc::now().to_rfc3339(),
                verified: false,
                manually_corrected: false,
                correction_notes: None,
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        facts.add_fact(pkg_fact);

        let report = verify_identity(
            "LM358",
            "Texas Instruments",
            Some("SOIC-8"),
            &parse_result,
            &facts,
        );

        assert_eq!(report.result, IdentityResult::IdentityVerified);
        assert!(report.package.is_some());
        assert!(report.package.unwrap().matches);
    }

    #[test]
    fn test_identity_with_package_mismatch() {
        let mut parse_result = create_test_parse_result(
            Some("Texas Instruments".to_string()),
            Some("LM358".to_string()),
        );
        let mut facts = create_test_fact_collection();

        let pkg_fact = DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::Package,
            parameter: ParameterFact {
                value: PhysicalQuantity::new(0.0, Unit::Unitless),
                tolerance: None,
                conditions: vec!["TSSOP-14".to_string()],
            },
            provenance: ProvenanceInfo {
                asset_id: 1,
                asset_sha256: String::new(),
                page: 1,
                extracted_text: "Package: TSSOP-14".to_string(),
                parser_version: "1.0.0".to_string(),
                extracted_at: chrono::Utc::now().to_rfc3339(),
                verified: false,
                manually_corrected: false,
                correction_notes: None,
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        facts.add_fact(pkg_fact);

        let report = verify_identity(
            "LM358",
            "Texas Instruments",
            Some("SOIC-8"),
            &parse_result,
            &facts,
        );

        assert_eq!(report.result, IdentityResult::IdentityMismatch);
        assert!(report.package.is_some());
        assert!(!report.package.unwrap().matches);
    }

    #[test]
    fn test_normalize_for_comparison() {
        assert_eq!(
            normalize_for_comparison("Texas Instruments"),
            "texasinstruments"
        );
        assert_eq!(normalize_for_comparison("TI"), "ti");
        assert_eq!(normalize_for_comparison("STMicroelectronics"), "stmicro");
        assert_eq!(normalize_for_comparison("LM358-N"), "lm358n");
        assert_eq!(normalize_for_comparison("LM358"), "lm358");
    }

    #[test]
    fn test_apply_identity_result() {
        let mut asset = AssetRecord::new(
            "LM358".to_string(),
            "Texas Instruments".to_string(),
            "Analog IC".to_string(),
            AssetType::Datasheet,
            "https://example.com/ds.pdf".to_string(),
            "Texas Instruments".to_string(),
        );
        asset.id = Some(1);

        let parse_result = create_test_parse_result(
            Some("Texas Instruments".to_string()),
            Some("LM358".to_string()),
        );
        let facts = create_test_fact_collection();
        let report = verify_identity("LM358", "Texas Instruments", None, &parse_result, &facts);

        apply_identity_result(&mut asset, &report);

        assert_eq!(asset.state, AssetState::IdentityVerified);
        assert_eq!(asset.identity_match, Some(true));
    }
}
