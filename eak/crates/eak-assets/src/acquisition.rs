use crate::asset::{AssetQuery, AssetRecord, AssetState, AssetType};
use crate::downloader::AssetDownloader;
use crate::source::{resolve_sources, should_attempt_automatic, AssetSource};
use crate::store::{init_db, upsert_asset};
use crate::validation::ValidationResult;
use anyhow::{Context, Result};
use std::path::Path;

/// Result of an acquisition attempt.
#[derive(Debug, Clone)]
pub struct AcquisitionResult {
    pub asset: AssetRecord,
    pub source: AssetSource,
    pub success: bool,
    pub error: Option<String>,
}

/// Configuration for acquisition behavior.
#[derive(Debug, Clone)]
pub struct AcquisitionConfig {
    pub max_attempts_per_source: u32,
    pub retry_blocked_after_hours: Option<u32>,
    pub prefer_manual_on_blocked: bool,
}

impl Default for AcquisitionConfig {
    fn default() -> Self {
        Self {
            max_attempts_per_source: 3,
            retry_blocked_after_hours: Some(24),
            prefer_manual_on_blocked: true,
        }
    }
}

/// Orchestrates the full acquisition pipeline:
/// Source Resolution → Download → Validate → Deduplicate → Store
pub struct AcquisitionPipeline {
    downloader: AssetDownloader,
    config: AcquisitionConfig,
}

impl Default for AcquisitionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl AcquisitionPipeline {
    pub fn new() -> Self {
        Self {
            downloader: AssetDownloader::new(),
            config: AcquisitionConfig::default(),
        }
    }

    pub fn with_config(config: AcquisitionConfig) -> Self {
        Self {
            downloader: AssetDownloader::new(),
            config,
        }
    }

    /// Acquire a single asset type for a component.
    pub fn acquire_asset(
        &self,
        db_path: &Path,
        mpn: &str,
        manufacturer: &str,
        category: &str,
        asset_type: AssetType,
    ) -> Result<AcquisitionResult> {
        let query = AssetQuery {
            mpn: mpn.to_string(),
            manufacturer: manufacturer.to_string(),
            asset_types: vec![asset_type],
            only_verified: false,
        };

        // Step 1: Resolve sources
        let sources = resolve_sources(&query);
        if sources.is_empty() {
            return Ok(AcquisitionResult {
                asset: AssetRecord::new(
                    mpn.to_string(),
                    manufacturer.to_string(),
                    category.to_string(),
                    asset_type,
                    "manual_import".to_string(),
                    "manual".to_string(),
                ),
                source: AssetSource::default(),
                success: false,
                error: Some("No sources available".to_string()),
            });
        }

        // Step 2: Try each source in priority order
        for source in &sources {
            if !should_attempt_automatic(source) {
                if self.config.prefer_manual_on_blocked && source.automation.to_string() == "Manual"
                {
                    continue; // Manual handled separately
                }
                continue;
            }

            // Create asset record in SOURCE_DECLARED state
            let mut asset = AssetRecord::new(
                mpn.to_string(),
                manufacturer.to_string(),
                category.to_string(),
                asset_type,
                build_source_url(source, mpn, manufacturer),
                source.name.clone(),
            );
            asset.source_provider = source.name.clone();

            // Step 3: Download with full provenance
            match self.downloader.download(&mut asset) {
                Ok(_) => {
                    if asset.state == AssetState::Verified {
                        // Step 4: Store with deduplication
                        let conn = init_db(db_path).context("Failed to open database")?;
                        let asset_id =
                            upsert_asset(&conn, &asset).context("Failed to store asset")?;

                        asset.id = Some(asset_id);
                        asset.set_state(AssetState::Verified);

                        return Ok(AcquisitionResult {
                            asset,
                            source: source.clone(),
                            success: true,
                            error: None,
                        });
                    } else {
                        // Download failed but didn't error - record failure and try next source
                        let _error = asset
                            .failure_reason
                            .clone()
                            .unwrap_or_else(|| "Unknown failure".to_string());
                        continue;
                    }
                }
                Err(_e) => {
                    // Network error - try next source
                    continue;
                }
            }
        }

        // All automatic sources exhausted
        // Try manual as fallback if configured
        if self.config.prefer_manual_on_blocked {
            for source in &sources {
                if source.source_type.to_string() == "ManualAcquisition" {
                    let mut asset = AssetRecord::new(
                        mpn.to_string(),
                        manufacturer.to_string(),
                        category.to_string(),
                        asset_type,
                        "manual_import".to_string(),
                        source.name.clone(),
                    );
                    asset.set_state(AssetState::RequiresManualAcquisition);
                    asset.failure_reason = Some(
                        "All automatic sources blocked; requires manual acquisition".to_string(),
                    );
                    asset.source_provider = "manual".to_string();

                    return Ok(AcquisitionResult {
                        asset,
                        source: source.clone(),
                        success: false,
                        error: Some("Requires manual acquisition".to_string()),
                    });
                }
            }
        }

        Ok(AcquisitionResult {
            asset: AssetRecord::new(
                mpn.to_string(),
                manufacturer.to_string(),
                category.to_string(),
                asset_type,
                "none".to_string(),
                "none".to_string(),
            ),
            source: AssetSource::default(),
            success: false,
            error: Some("All sources exhausted".to_string()),
        })
    }

    /// Acquire multiple asset types for a component.
    pub fn acquire_component_assets(
        &self,
        db_path: &Path,
        mpn: &str,
        manufacturer: &str,
        category: &str,
        asset_types: &[AssetType],
    ) -> Vec<AcquisitionResult> {
        let mut results = Vec::new();
        for asset_type in asset_types {
            match self.acquire_asset(db_path, mpn, manufacturer, category, *asset_type) {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(AcquisitionResult {
                        asset: AssetRecord::new(
                            mpn.to_string(),
                            manufacturer.to_string(),
                            category.to_string(),
                            *asset_type,
                            "error".to_string(),
                            "error".to_string(),
                        ),
                        source: AssetSource::default(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        results
    }
}

/// Build URL for a source and component.
pub fn build_source_url(source: &AssetSource, mpn: &str, manufacturer: &str) -> String {
    source
        .url_pattern
        .replace("{MPN}", mpn)
        .replace("{MANUFACTURER}", manufacturer)
}

/// High-level function to acquire all standard assets for a component.
pub fn acquire_component(
    db_path: &Path,
    mpn: &str,
    manufacturer: &str,
    category: &str,
) -> Result<Vec<AcquisitionResult>> {
    let pipeline = AcquisitionPipeline::new();
    let asset_types = vec![
        AssetType::Datasheet,
        AssetType::Symbol,
        AssetType::Footprint,
        AssetType::Model3D,
    ];
    Ok(pipeline.acquire_component_assets(db_path, mpn, manufacturer, category, &asset_types))
}

/// Check if a component has all required assets verified.
pub fn check_component_ready(
    db_path: &Path,
    mpn: &str,
    manufacturer: &str,
    required_types: &[AssetType],
) -> Result<bool> {
    let conn = init_db(db_path).context("Failed to open database")?;

    for asset_type in required_types {
        let query = AssetQuery {
            mpn: mpn.to_string(),
            manufacturer: manufacturer.to_string(),
            asset_types: vec![*asset_type],
            only_verified: true,
        };

        let assets = crate::store::find_assets(&conn, &query)?;
        if assets.is_empty() {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{AssetState, AssetType};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_acquisition_pipeline_creation() {
        let pipeline = AcquisitionPipeline::new();
        assert_eq!(pipeline.config.max_attempts_per_source, 3);
    }

    #[test]
    fn test_build_source_url() {
        let source = AssetSource {
            id: "test".to_string(),
            name: "Test".to_string(),
            source_type: crate::source::SourceType::Manufacturer,
            url_pattern: "https://example.com/{MANUFACTURER}/{MPN}.pdf".to_string(),
            auth: crate::source::AuthRequirement::None,
            automation: crate::source::AutomationFeasibility::High,
            asset_types: vec![],
            reliability: 1.0,
            provenance_fields: vec![],
            tested: false,
            test_result: None,
            failure_reason: None,
            priority: 1,
            rate_limit_rpm: None,
            metadata: std::collections::HashMap::new(),
        };

        let url = build_source_url(&source, "RC0402FR-071RL", "Yageo");
        assert_eq!(url, "https://example.com/Yageo/RC0402FR-071RL.pdf");
    }
}
