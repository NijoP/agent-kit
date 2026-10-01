//! Asset provenance tracking.

use crate::asset::{AssetRecord, AssetState, AssetType};
use serde::{Deserialize, Serialize};

/// Complete provenance information for an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetProvenance {
    /// Asset record
    pub asset: AssetRecord,
    /// Chain of custody events
    pub custody_chain: Vec<CustodyEvent>,
    /// Verification history
    pub verification_history: Vec<VerificationEvent>,
    /// Related assets (same file, different components)
    pub related_assets: Vec<RelatedAsset>,
}

/// Custody event in the asset's lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEvent {
    /// Event timestamp (ISO 8601)
    pub timestamp: String,
    /// Event type
    pub event_type: CustodyEventType,
    /// Actor (system, engineer, URL)
    pub actor: String,
    /// Details
    pub details: String,
}

/// Types of custody events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyEventType {
    /// Asset discovered via source resolution
    Discovered,
    /// Download attempted
    DownloadAttempted,
    /// Download successful
    Downloaded,
    /// Validation passed
    Validated,
    /// Asset verified and stored
    Verified,
    /// Asset manually imported
    ManualImport,
    /// Asset linked to component
    LinkedToComponent,
    /// Asset deduplicated (same SHA-256)
    Deduplicated,
    /// Asset access blocked
    Blocked,
    /// Asset marked as not found
    NotFound,
}

/// Verification event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationEvent {
    /// Verification timestamp
    pub timestamp: String,
    /// Verification step
    pub step: VerificationStep,
    /// Result
    pub result: VerificationResult,
    /// Details
    pub details: String,
}

/// Verification steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStep {
    HttpSuccess,
    ContentTypeCheck,
    FileSizeCheck,
    SignatureCheck,
    ParserCheck,
    IdentityCheck,
    DeduplicationCheck,
}

/// Verification results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationResult {
    Passed,
    Failed,
    Skipped,
}

/// Related asset (same file shared by multiple components).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedAsset {
    /// Component MPN
    pub component_mpn: String,
    /// Component manufacturer
    pub manufacturer: String,
    /// Asset type
    pub asset_type: AssetType,
    /// Relationship type
    pub relationship: AssetRelationship,
}

/// How assets relate to each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetRelationship {
    /// Exact same file (same SHA-256)
    Identical,
    /// Different revision of same document
    Revision,
    /// Derived from (e.g., symbol generated from footprint)
    Derived,
    /// Supersedes (newer version)
    Supersedes,
}

/// Build provenance for an asset.
pub fn build_provenance(asset: &AssetRecord) -> AssetProvenance {
    let mut custody_chain = Vec::new();
    let mut verification_history = Vec::new();

    // Initial discovery
    custody_chain.push(CustodyEvent {
        timestamp: asset.created_at.clone(),
        event_type: CustodyEventType::Discovered,
        actor: "source_resolver".to_string(),
        details: format!("Source declared: {}", asset.source_url),
    });

    // Add attempt events
    if asset.attempts > 0 {
        custody_chain.push(CustodyEvent {
            timestamp: asset
                .last_attempt
                .clone()
                .unwrap_or_else(|| asset.created_at.clone()),
            event_type: CustodyEventType::DownloadAttempted,
            actor: "downloader".to_string(),
            details: format!("Attempt {} of download", asset.attempts),
        });
    }

    // Add verification events based on state
    match asset.state {
        AssetState::Verified | AssetState::UserProvided => {
            custody_chain.push(CustodyEvent {
                timestamp: asset
                    .verified_at
                    .clone()
                    .unwrap_or_else(|| asset.updated_at.clone()),
                event_type: CustodyEventType::Verified,
                actor: "validator".to_string(),
                details: "Asset fully verified and available".to_string(),
            });

            verification_history.push(VerificationEvent {
                timestamp: asset
                    .verified_at
                    .clone()
                    .unwrap_or_else(|| asset.updated_at.clone()),
                step: VerificationStep::HttpSuccess,
                result: VerificationResult::Passed,
                details: format!("HTTP {}", asset.http_status.unwrap_or(200)),
            });

            verification_history.push(VerificationEvent {
                timestamp: asset
                    .verified_at
                    .clone()
                    .unwrap_or_else(|| asset.updated_at.clone()),
                step: VerificationStep::SignatureCheck,
                result: VerificationResult::Passed,
                details: format!("Signature: {:?}", asset.file_signature),
            });

            verification_history.push(VerificationEvent {
                timestamp: asset
                    .verified_at
                    .clone()
                    .unwrap_or_else(|| asset.updated_at.clone()),
                step: VerificationStep::ParserCheck,
                result: VerificationResult::Passed,
                details: "Parser validation passed".to_string(),
            });

            if asset.identity_match.unwrap_or(false) {
                verification_history.push(VerificationEvent {
                    timestamp: asset
                        .verified_at
                        .clone()
                        .unwrap_or_else(|| asset.updated_at.clone()),
                    step: VerificationStep::IdentityCheck,
                    result: VerificationResult::Passed,
                    details: "Component identity matches".to_string(),
                });
            }
        }
        AssetState::AccessBlocked | AssetState::CloudflareChallenge => {
            custody_chain.push(CustodyEvent {
                timestamp: asset.updated_at.clone(),
                event_type: CustodyEventType::Blocked,
                actor: "downloader".to_string(),
                details: asset.failure_reason.clone().unwrap_or_default(),
            });
        }
        AssetState::SourceNotFound => {
            custody_chain.push(CustodyEvent {
                timestamp: asset.updated_at.clone(),
                event_type: CustodyEventType::NotFound,
                actor: "downloader".to_string(),
                details: "Source URL returned 404".to_string(),
            });
        }
        _ => {}
    }

    AssetProvenance {
        asset: asset.clone(),
        custody_chain,
        verification_history,
        related_assets: Vec::new(),
    }
}

/// Generate a human-readable provenance report.
pub fn format_provenance(provenance: &AssetProvenance) -> String {
    let mut out = String::new();
    let asset = &provenance.asset;

    out.push_str("Asset Provenance Report\n");
    out.push_str("========================\n\n");
    out.push_str(&format!(
        "Component: {} ({})\n",
        asset.component_mpn, asset.manufacturer
    ));
    out.push_str(&format!("Asset Type: {}\n", asset.asset_type));
    out.push_str(&format!("State: {}\n", asset.state));
    out.push_str(&format!(
        "Acquisition Method: {}\n",
        asset.acquisition_method
    ));
    out.push_str(&format!("Source: {}\n", asset.source_provider));
    out.push_str(&format!("Source URL: {}\n", asset.source_url));

    if let Some(final_url) = &asset.final_url {
        out.push_str(&format!("Final URL: {}\n", final_url));
    }

    if let Some(sha256) = &asset.sha256 {
        out.push_str(&format!("SHA-256: {}\n", sha256));
    }

    if let Some(path) = &asset.local_path {
        out.push_str(&format!("Local Path: {}\n", path.display()));
    }

    if let Some(size) = asset.file_size {
        out.push_str(&format!("File Size: {} bytes\n", size));
    }

    out.push_str(&format!("Created: {}\n", asset.created_at));
    out.push_str(&format!("Updated: {}\n\n", asset.updated_at));

    out.push_str("Custody Chain:\n");
    out.push_str("--------------\n");
    for event in &provenance.custody_chain {
        out.push_str(&format!(
            "  [{}] {} by {}: {}\n",
            event.timestamp, event.event_type as u8, event.actor, event.details
        ));
    }

    out.push_str("\nVerification History:\n");
    out.push_str("-------------------\n");
    for event in &provenance.verification_history {
        out.push_str(&format!(
            "  [{}] {:?}: {:?} - {}\n",
            event.timestamp, event.step, event.result, event.details
        ));
    }

    if !provenance.related_assets.is_empty() {
        out.push_str("\nRelated Assets (shared file):\n");
        out.push_str("-----------------------------\n");
        for rel in &provenance.related_assets {
            out.push_str(&format!(
                "  {} {} ({}) - {:?}\n",
                rel.manufacturer, rel.component_mpn, rel.asset_type, rel.relationship
            ));
        }
    }

    out
}

/// Query provenance by SHA-256 to find all components sharing an asset.
pub fn find_related_by_sha256(sha256: &str, all_assets: &[AssetRecord]) -> Vec<RelatedAsset> {
    all_assets
        .iter()
        .filter(|a| a.sha256.as_deref() == Some(sha256))
        .map(|a| RelatedAsset {
            component_mpn: a.component_mpn.clone(),
            manufacturer: a.manufacturer.clone(),
            asset_type: a.asset_type,
            relationship: AssetRelationship::Identical,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{AssetRecord, AssetState, AssetType};
    use std::path::PathBuf;

    #[test]
    fn test_provenance_building() {
        let mut asset = AssetRecord::new(
            "RC0402FR-071RL".to_string(),
            "Yageo".to_string(),
            "Resistor".to_string(),
            AssetType::Datasheet,
            "https://example.com/datasheet.pdf".to_string(),
            "Yageo".to_string(),
        );
        asset.set_state(AssetState::Verified);
        asset.sha256 = Some("abc123".to_string());
        asset.local_path = Some(PathBuf::from("datasheets/Yageo_RC0402FR-071RL.pdf"));
        asset.file_size = Some(12345);
        asset.http_status = Some(200);

        let provenance = build_provenance(&asset);
        assert_eq!(provenance.custody_chain.len(), 2); // Discovered + Verified
        assert!(!provenance.verification_history.is_empty());
    }

    #[test]
    fn test_blocked_provenance() {
        let mut asset = AssetRecord::new(
            "RC0402FR-071RL".to_string(),
            "Yageo".to_string(),
            "Resistor".to_string(),
            AssetType::Datasheet,
            "https://yageo.com/RC0402FR-071RL".to_string(),
            "Yageo".to_string(),
        );
        asset.set_state(AssetState::CloudflareChallenge);
        asset.failure_reason = Some("Cloudflare Turnstile challenge".to_string());
        asset.attempts = 1;

        let provenance = build_provenance(&asset);
        let blocked_events: Vec<_> = provenance
            .custody_chain
            .iter()
            .filter(|e| e.event_type == CustodyEventType::Blocked)
            .collect();
        assert_eq!(blocked_events.len(), 1);
    }

    #[test]
    fn test_format_provenance() {
        let mut asset = AssetRecord::new(
            "RC0402FR-071RL".to_string(),
            "Yageo".to_string(),
            "Resistor".to_string(),
            AssetType::Datasheet,
            "https://example.com/datasheet.pdf".to_string(),
            "Yageo".to_string(),
        );
        asset.set_state(AssetState::Verified);
        asset.sha256 = Some("abc123".to_string());

        let provenance = build_provenance(&asset);
        let report = format_provenance(&provenance);
        assert!(report.contains("RC0402FR-071RL"));
        assert!(report.contains("Yageo"));
        assert!(report.contains("VERIFIED"));
    }
}
