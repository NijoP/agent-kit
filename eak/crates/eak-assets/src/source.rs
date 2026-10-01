//! Source resolution and acquisition strategies.

use crate::asset::{AssetQuery, AssetType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Source classification for acquisition strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    /// Manufacturer website
    Manufacturer,
    /// Distributor website (DigiKey, Mouser, etc.)
    Distributor,
    /// Public repository (GitHub, GitLab)
    PublicRepository,
    /// Commercial API (Octopart, SnapEDA, etc.)
    CommercialApi,
    /// Static PDF endpoint (direct PDF URL)
    StaticPdfEndpoint,
    /// Dynamic web page (requires JavaScript)
    DynamicWebPage,
    /// Browser-only (requires full browser automation)
    BrowserOnly,
    /// Manual acquisition required
    ManualAcquisition,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Manufacturer => write!(f, "Manufacturer"),
            SourceType::Distributor => write!(f, "Distributor"),
            SourceType::PublicRepository => write!(f, "PublicRepository"),
            SourceType::CommercialApi => write!(f, "CommercialApi"),
            SourceType::StaticPdfEndpoint => write!(f, "StaticPdfEndpoint"),
            SourceType::DynamicWebPage => write!(f, "DynamicWebPage"),
            SourceType::BrowserOnly => write!(f, "BrowserOnly"),
            SourceType::ManualAcquisition => write!(f, "ManualAcquisition"),
        }
    }
}

/// Authentication requirement for a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthRequirement {
    /// No authentication needed
    None,
    /// Session cookies required
    SessionCookies,
    /// API key required
    ApiKey,
    /// OAuth / token required
    OAuth,
    /// Requires human interaction (CAPTCHA, login)
    HumanInteraction,
}

/// Automation feasibility assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationFeasibility {
    /// Fully automatable with simple HTTP
    High,
    /// Automatable with browser automation
    Medium,
    /// Requires advanced stealth + proxies
    Low,
    /// Not automatable, requires manual
    None,
    /// Manual only
    Manual,
}

impl fmt::Display for AutomationFeasibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AutomationFeasibility::High => write!(f, "High"),
            AutomationFeasibility::Medium => write!(f, "Medium"),
            AutomationFeasibility::Low => write!(f, "Low"),
            AutomationFeasibility::None => write!(f, "None"),
            AutomationFeasibility::Manual => write!(f, "Manual"),
        }
    }
}

/// Source definition with acquisition metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSource {
    /// Unique source identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Source type classification
    pub source_type: SourceType,
    /// URL pattern with {MPN} and {MANUFACTURER} placeholders
    pub url_pattern: String,
    /// Authentication requirement
    pub auth: AuthRequirement,
    /// Automation feasibility
    pub automation: AutomationFeasibility,
    /// Asset types this source can provide
    pub asset_types: Vec<AssetType>,
    /// Reliability score (0.0 - 1.0)
    pub reliability: f32,
    /// Provenance fields required
    pub provenance_fields: Vec<String>,
    /// Whether this source has been tested
    pub tested: bool,
    /// Test result if tested
    pub test_result: Option<SourceTestResult>,
    /// Failure reason if blocked
    pub failure_reason: Option<String>,
    /// Priority for source resolution (lower = higher priority)
    pub priority: u32,
    /// Rate limit (requests per minute)
    pub rate_limit_rpm: Option<u32>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Source test result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceTestResult {
    /// Source works reliably
    Working,
    /// Source partially works (some assets, some not)
    Partial,
    /// Source blocked by anti-bot
    Blocked,
    /// Source not tested yet
    Untested,
    /// Source designed but not implemented
    Design,
}

/// Resolve sources for a component query.
pub fn resolve_sources(query: &AssetQuery) -> Vec<AssetSource> {
    let mut sources = get_all_sources();

    // Filter by asset type
    sources.retain(|s| s.asset_types.iter().any(|t| query.asset_types.contains(t)));

    // Filter by manufacturer if we have specific sources
    sources.retain(|s| {
        // Keep generic sources (distributors, search engines) for all manufacturers
        // Keep manufacturer-specific sources only for matching manufacturer
        match s.source_type {
            SourceType::Manufacturer => s
                .metadata
                .get("manufacturer")
                .map(|m| m.eq_ignore_ascii_case(&query.manufacturer))
                .unwrap_or(false),
            _ => true,
        }
    });

    // Sort by priority (lower = higher priority), then by reliability (higher = better)
    sources.sort_by(|a, b| {
        a.priority.cmp(&b.priority).then_with(|| {
            b.reliability
                .partial_cmp(&a.reliability)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    sources
}

/// Get all known sources from the source matrix.
fn get_all_sources() -> Vec<AssetSource> {
    vec![
        // Texas Instruments - WORKING
        AssetSource {
            id: "ti_datasheet".to_string(),
            name: "Texas Instruments".to_string(),
            source_type: SourceType::Manufacturer,
            url_pattern: "https://www.ti.com/lit/ds/symlink/{MPN}.pdf".to_string(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::High,
            asset_types: vec![AssetType::Datasheet],
            reliability: 0.95,
            provenance_fields: vec![
                "source_url".to_string(),
                "manufacturer".to_string(),
                "mpn".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
            ],
            tested: true,
            test_result: Some(SourceTestResult::Working),
            failure_reason: None,
            priority: 10,
            rate_limit_rpm: Some(60),
            metadata: {
                let mut m = HashMap::new();
                m.insert("manufacturer".to_string(), "Texas Instruments".to_string());
                m.insert(
                    "note".to_string(),
                    "Direct PDF links work reliably".to_string(),
                );
                m
            },
        },
        // Yageo - BLOCKED
        AssetSource {
            id: "yageo_datasheet".to_string(),
            name: "Yageo".to_string(),
            source_type: SourceType::Manufacturer,
            url_pattern: "https://www.yageo.com/en/Products/Detail/{MPN}".to_string(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::None,
            asset_types: vec![AssetType::Datasheet],
            reliability: 0.0,
            provenance_fields: vec![
                "source_url".to_string(),
                "manufacturer".to_string(),
                "mpn".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
            ],
            tested: true,
            test_result: Some(SourceTestResult::Blocked),
            failure_reason: Some(
                "Cloudflare Turnstile challenge, Next.js dynamic rendering, 404 on direct PDF URLs"
                    .to_string(),
            ),
            priority: 100,
            rate_limit_rpm: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("manufacturer".to_string(), "Yageo".to_string());
                m
            },
        },
        // Vishay - BLOCKED
        AssetSource {
            id: "vishay_datasheet".to_string(),
            name: "Vishay".to_string(),
            source_type: SourceType::Manufacturer,
            url_pattern: "https://www.vishay.com/docs/{DOC_ID}/{MPN}.pdf".to_string(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::None,
            asset_types: vec![AssetType::Datasheet],
            reliability: 0.0,
            provenance_fields: vec![
                "source_url".to_string(),
                "manufacturer".to_string(),
                "mpn".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
            ],
            tested: true,
            test_result: Some(SourceTestResult::Blocked),
            failure_reason: Some(
                "404 on direct PDF, 403 on docs site, no predictable URL pattern".to_string(),
            ),
            priority: 100,
            rate_limit_rpm: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("manufacturer".to_string(), "Vishay".to_string());
                m
            },
        },
        // Murata - BLOCKED
        AssetSource {
            id: "murata_datasheet".to_string(),
            name: "Murata".to_string(),
            source_type: SourceType::Manufacturer,
            url_pattern: "https://www.murata.com/en-global/products/{MPN}".to_string(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::None,
            asset_types: vec![AssetType::Datasheet, AssetType::Model3D],
            reliability: 0.0,
            provenance_fields: vec![
                "source_url".to_string(),
                "manufacturer".to_string(),
                "mpn".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
            ],
            tested: true,
            test_result: Some(SourceTestResult::Blocked),
            failure_reason: Some(
                "Next.js site, 404 on API endpoints, Cloudflare protection".to_string(),
            ),
            priority: 100,
            rate_limit_rpm: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("manufacturer".to_string(), "Murata".to_string());
                m
            },
        },
        // DigiKey - BLOCKED (web)
        AssetSource {
            id: "digikey_datasheet".to_string(),
            name: "DigiKey".to_string(),
            source_type: SourceType::Distributor,
            url_pattern: "https://www.digikey.com/en/products/detail/{MANUFACTURER}/{MPN}"
                .to_string(),
            auth: AuthRequirement::SessionCookies,
            automation: AutomationFeasibility::Low,
            asset_types: vec![AssetType::Datasheet],
            reliability: 0.1,
            provenance_fields: vec![
                "source_url".to_string(),
                "distributor".to_string(),
                "manufacturer".to_string(),
                "mpn".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
            ],
            tested: true,
            test_result: Some(SourceTestResult::Blocked),
            failure_reason: Some(
                "Cloudflare Turnstile, JavaScript-rendered PDF links, bot detection".to_string(),
            ),
            priority: 50,
            rate_limit_rpm: Some(10),
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "note".to_string(),
                    "Requires stealth browser + residential proxies".to_string(),
                );
                m
            },
        },
        // Farnell - PARTIAL
        AssetSource {
            id: "farnell_datasheet".to_string(),
            name: "Farnell".to_string(),
            source_type: SourceType::Distributor,
            url_pattern: "https://www.farnell.com/search?q={MPN}".to_string(),
            auth: AuthRequirement::SessionCookies,
            automation: AutomationFeasibility::Medium,
            asset_types: vec![AssetType::Datasheet],
            reliability: 0.4,
            provenance_fields: vec![
                "source_url".to_string(),
                "distributor".to_string(),
                "manufacturer".to_string(),
                "mpn".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
            ],
            tested: true,
            test_result: Some(SourceTestResult::Partial),
            failure_reason: Some("Numeric IDs work but require JS search to discover".to_string()),
            priority: 60,
            rate_limit_rpm: Some(30),
            metadata: HashMap::new(),
        },
        // GitHub KiCad Libraries - WORKING
        AssetSource {
            id: "github_kicad_symbols".to_string(),
            name: "GitHub KiCad Symbols".to_string(),
            source_type: SourceType::PublicRepository,
            url_pattern: "https://github.com/kicad/kicad-symbols".to_string(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::High,
            asset_types: vec![AssetType::Symbol],
            reliability: 0.99,
            provenance_fields: vec![
                "source_url".to_string(),
                "repository".to_string(),
                "commit_hash".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
            ],
            tested: true,
            test_result: Some(SourceTestResult::Working),
            failure_reason: None,
            priority: 5,
            rate_limit_rpm: Some(100),
            metadata: HashMap::new(),
        },
        AssetSource {
            id: "github_kicad_footprints".to_string(),
            name: "GitHub KiCad Footprints".to_string(),
            source_type: SourceType::PublicRepository,
            url_pattern: "https://github.com/kicad/kicad-footprints".to_string(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::High,
            asset_types: vec![AssetType::Footprint],
            reliability: 0.99,
            provenance_fields: vec![
                "source_url".to_string(),
                "repository".to_string(),
                "commit_hash".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
            ],
            tested: true,
            test_result: Some(SourceTestResult::Working),
            failure_reason: None,
            priority: 5,
            rate_limit_rpm: Some(100),
            metadata: HashMap::new(),
        },
        AssetSource {
            id: "github_kicad_3d".to_string(),
            name: "GitHub KiCad 3D Models".to_string(),
            source_type: SourceType::PublicRepository,
            url_pattern: "https://github.com/kicad/kicad-packages3D".to_string(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::High,
            asset_types: vec![AssetType::Model3D],
            reliability: 0.95,
            provenance_fields: vec![
                "source_url".to_string(),
                "repository".to_string(),
                "commit_hash".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
            ],
            tested: true,
            test_result: Some(SourceTestResult::Working),
            failure_reason: None,
            priority: 5,
            rate_limit_rpm: Some(100),
            metadata: HashMap::new(),
        },
        // Manual acquisition - DESIGN
        AssetSource {
            id: "manual_import".to_string(),
            name: "Manual Import".to_string(),
            source_type: SourceType::ManualAcquisition,
            url_pattern: "file:///path/to/asset".to_string(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::Manual,
            asset_types: vec![
                AssetType::Datasheet,
                AssetType::Symbol,
                AssetType::Footprint,
                AssetType::Model3D,
                AssetType::AppNote,
            ],
            reliability: 1.0,
            provenance_fields: vec![
                "source_path".to_string(),
                "imported_by".to_string(),
                "acquisition_timestamp".to_string(),
                "sha256".to_string(),
                "notes".to_string(),
            ],
            tested: false,
            test_result: Some(SourceTestResult::Design),
            failure_reason: None,
            priority: 1,
            rate_limit_rpm: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("note".to_string(), "Engineer manually downloads and imports assets. Highest trust. Required for blocked manufacturers.".to_string());
                m
            },
        },
    ]
}

/// Build URL for a source and component.
pub fn build_source_url(source: &AssetSource, mpn: &str, manufacturer: &str) -> String {
    source
        .url_pattern
        .replace("{MPN}", mpn)
        .replace("{MANUFACTURER}", manufacturer)
}

/// Check if a source should be attempted for automatic acquisition.
pub fn should_attempt_automatic(source: &AssetSource) -> bool {
    matches!(
        source.automation,
        AutomationFeasibility::High | AutomationFeasibility::Medium
    ) && source.reliability > 0.5
}

/// Get the best source for a component and asset type.
pub fn get_best_source(query: &AssetQuery) -> Option<AssetSource> {
    let sources = resolve_sources(query);
    sources.into_iter().find(should_attempt_automatic)
}

/// Record source test result.
pub fn record_test_result(
    source_id: &str,
    result: SourceTestResult,
    failure_reason: Option<String>,
) {
    // This would update the source matrix in practice
    // For now, we log it
    eprintln!(
        "Source test result: {} = {:?} ({:?})",
        source_id, result, failure_reason
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetQuery;

    #[test]
    fn test_resolve_sources_for_datasheet() {
        let query = AssetQuery::for_datasheet("RC0402FR-071RL".to_string(), "Yageo".to_string());
        let sources = resolve_sources(&query);

        // Should include manufacturer-specific (Yageo) and generic (DigiKey, Farnell, Manual)
        assert!(!sources.is_empty());

        // Yageo source should be included
        assert!(sources.iter().any(|s| s.id == "yageo_datasheet"));

        // Manual should always be included
        assert!(sources.iter().any(|s| s.id == "manual_import"));
    }

    #[test]
    fn test_resolve_sources_for_symbol() {
        let mut query = AssetQuery::new("RC0402FR-071RL".to_string(), "Yageo".to_string());
        query.asset_types = vec![crate::asset::AssetType::Symbol];

        let sources = resolve_sources(&query);
        assert!(sources.iter().any(|s| s.id == "github_kicad_symbols"));
    }

    #[test]
    fn test_build_source_url() {
        let source = AssetSource {
            id: "test".to_string(),
            name: "Test".to_string(),
            source_type: SourceType::Manufacturer,
            url_pattern: "https://example.com/{MANUFACTURER}/{MPN}.pdf".to_string(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::High,
            asset_types: vec![],
            reliability: 1.0,
            provenance_fields: vec![],
            tested: false,
            test_result: None,
            failure_reason: None,
            priority: 1,
            rate_limit_rpm: None,
            metadata: HashMap::new(),
        };

        let url = build_source_url(&source, "RC0402FR-071RL", "Yageo");
        assert_eq!(url, "https://example.com/Yageo/RC0402FR-071RL.pdf");
    }

    #[test]
    fn test_should_attempt_automatic() {
        let working_source = AssetSource {
            automation: AutomationFeasibility::High,
            reliability: 0.9,
            ..Default::default()
        };
        assert!(should_attempt_automatic(&working_source));

        let blocked_source = AssetSource {
            automation: AutomationFeasibility::None,
            reliability: 0.0,
            ..Default::default()
        };
        assert!(!should_attempt_automatic(&blocked_source));

        let low_reliability = AssetSource {
            automation: AutomationFeasibility::High,
            reliability: 0.3,
            ..Default::default()
        };
        assert!(!should_attempt_automatic(&low_reliability));
    }
}

impl Default for AssetSource {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            source_type: SourceType::Manufacturer,
            url_pattern: String::new(),
            auth: AuthRequirement::None,
            automation: AutomationFeasibility::None,
            asset_types: Vec::new(),
            reliability: 0.0,
            provenance_fields: Vec::new(),
            tested: false,
            test_result: None,
            failure_reason: None,
            priority: 100,
            rate_limit_rpm: None,
            metadata: HashMap::new(),
        }
    }
}
