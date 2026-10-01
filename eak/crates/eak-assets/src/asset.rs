//! Asset type definitions and state machine.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Types of assets that can be associated with components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    /// Manufacturer datasheet (PDF)
    Datasheet,
    /// KiCad schematic symbol (.kicad_sym)
    Symbol,
    /// KiCad PCB footprint (.kicad_mod)
    Footprint,
    /// 3D mechanical model (.step, .stp)
    Model3D,
    /// Application note (PDF)
    AppNote,
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetType::Datasheet => write!(f, "datasheet"),
            AssetType::Symbol => write!(f, "symbol"),
            AssetType::Footprint => write!(f, "footprint"),
            AssetType::Model3D => write!(f, "model3d"),
            AssetType::AppNote => write!(f, "app_note"),
        }
    }
}

impl std::str::FromStr for AssetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "datasheet" => Ok(AssetType::Datasheet),
            "symbol" => Ok(AssetType::Symbol),
            "footprint" => Ok(AssetType::Footprint),
            "model3d" | "model_3d" | "3d" | "step" | "stp" => Ok(AssetType::Model3D),
            "app_note" | "appnote" | "application_note" => Ok(AssetType::AppNote),
            _ => Err(format!("Unknown asset type: {}", s)),
        }
    }
}

impl AssetType {
    /// Expected file extension for this asset type.
    pub fn extension(&self) -> &'static str {
        match self {
            AssetType::Datasheet | AssetType::AppNote => "pdf",
            AssetType::Symbol => "kicad_sym",
            AssetType::Footprint => "kicad_mod",
            AssetType::Model3D => "step",
        }
    }

    /// Magic bytes / file signature for validation.
    pub fn magic_bytes(&self) -> &'static [u8] {
        match self {
            AssetType::Datasheet | AssetType::AppNote => b"%PDF-",
            AssetType::Symbol => b"(kicad_symbol_lib",
            AssetType::Footprint => b"(footprint",
            AssetType::Model3D => b"ISO-10303-21",
        }
    }
}

/// Asset acquisition state machine.
///
/// An asset progresses through these states. Terminal states are VERIFIED, BLOCKED, NOT_FOUND,
/// and REQUIRES_MANUAL_ACQUISITION. The system NEVER retries BLOCKED sources automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetState {
    /// Source URL has been identified for this component
    SourceDeclared,
    /// Source URL resolved and accessible
    SourceResolved,
    /// Download attempt initiated
    DownloadAttempted,
    /// HTTP 200 received, content downloaded
    Downloaded,
    /// File magic bytes match expected signature
    SignatureValid,
    /// File parsed successfully by appropriate tool
    Parsed,
    /// Asset identity matches component (MPN, manufacturer)
    IdentityVerified,
    /// Fully verified and available for use
    Verified,
    /// Source returns 404 - no asset at that URL
    SourceNotFound,
    /// HTTP error (5xx, timeout, network)
    HttpError,
    /// Access forbidden (403)
    AccessForbidden,
    /// Received HTML instead of expected binary format
    HtmlResponse,
    /// File signature doesn't match expected type
    InvalidSignature,
    /// Parser failed to read the file
    ParseFailed,
    /// Asset doesn't match component (wrong MPN, manufacturer)
    IdentityMismatch,
    /// Anti-bot protection detected (Cloudflare, etc.)
    AccessBlocked,
    /// Cloudflare Turnstile challenge detected
    CloudflareChallenge,
    /// Asset was manually provided by engineer
    UserProvided,
    /// Requires manual acquisition (no automated source works)
    RequiresManualAcquisition,
}

impl fmt::Display for AssetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetState::SourceDeclared => write!(f, "SOURCE_DECLARED"),
            AssetState::SourceResolved => write!(f, "SOURCE_RESOLVED"),
            AssetState::DownloadAttempted => write!(f, "DOWNLOAD_ATTEMPTED"),
            AssetState::Downloaded => write!(f, "DOWNLOADED"),
            AssetState::SignatureValid => write!(f, "SIGNATURE_VALID"),
            AssetState::Parsed => write!(f, "PARSED"),
            AssetState::IdentityVerified => write!(f, "IDENTITY_VERIFIED"),
            AssetState::Verified => write!(f, "VERIFIED"),
            AssetState::SourceNotFound => write!(f, "SOURCE_NOT_FOUND"),
            AssetState::HttpError => write!(f, "HTTP_ERROR"),
            AssetState::AccessForbidden => write!(f, "ACCESS_FORBIDDEN"),
            AssetState::HtmlResponse => write!(f, "HTML_RESPONSE"),
            AssetState::InvalidSignature => write!(f, "INVALID_SIGNATURE"),
            AssetState::ParseFailed => write!(f, "PARSE_FAILED"),
            AssetState::IdentityMismatch => write!(f, "IDENTITY_MISMATCH"),
            AssetState::AccessBlocked => write!(f, "ACCESS_BLOCKED"),
            AssetState::CloudflareChallenge => write!(f, "CLOUDFLARE_CHALLENGE"),
            AssetState::UserProvided => write!(f, "USER_PROVIDED"),
            AssetState::RequiresManualAcquisition => write!(f, "REQUIRES_MANUAL_ACQUISITION"),
        }
    }
}

impl AssetState {
    /// Returns true if this is a terminal state (no further automatic transitions).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AssetState::Verified
                | AssetState::SourceNotFound
                | AssetState::AccessForbidden
                | AssetState::HtmlResponse
                | AssetState::InvalidSignature
                | AssetState::ParseFailed
                | AssetState::IdentityMismatch
                | AssetState::AccessBlocked
                | AssetState::CloudflareChallenge
                | AssetState::UserProvided
                | AssetState::RequiresManualAcquisition
        )
    }

    /// Returns true if the asset is available for use.
    pub fn is_available(&self) -> bool {
        matches!(self, AssetState::Verified | AssetState::UserProvided)
    }

    /// Returns true if automatic acquisition should be retried.
    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            AssetState::HttpError | AssetState::DownloadAttempted | AssetState::SourceResolved
        )
    }

    /// Returns true if this state indicates a blocking condition that should NOT be retried.
    pub fn is_blocked(&self) -> bool {
        matches!(
            self,
            AssetState::AccessBlocked
                | AssetState::CloudflareChallenge
                | AssetState::AccessForbidden
                | AssetState::SourceNotFound
        )
    }
}

/// Complete asset record with full provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRecord {
    /// Unique asset ID (assigned by database)
    pub id: Option<i64>,
    /// Component this asset belongs to (by MPN)
    pub component_mpn: String,
    /// Component manufacturer
    pub manufacturer: String,
    /// Component category (resistor, capacitor, etc.)
    pub category: String,
    /// Type of asset
    pub asset_type: AssetType,
    /// Source URL where asset was found
    pub source_url: String,
    /// Final URL after redirects
    pub final_url: Option<String>,
    /// HTTP status code from download
    pub http_status: Option<u16>,
    /// Content-Type header from response
    pub content_type: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Local filesystem path (relative to asset root)
    pub local_path: Option<PathBuf>,
    /// SHA-256 hash of file content
    pub sha256: Option<String>,
    /// Current state in the state machine
    pub state: AssetState,
    /// Failure reason if not verified
    pub failure_reason: Option<String>,
    /// Number of acquisition attempts
    pub attempts: u32,
    /// Timestamp of last attempt (ISO 8601)
    pub last_attempt: Option<String>,
    /// Source provider name (manufacturer, distributor, etc.)
    pub source_provider: String,
    /// HTTP response headers (JSON)
    pub http_headers: Option<String>,
    /// Timestamp when downloaded (ISO 8601)
    pub downloaded_at: Option<String>,
    /// Timestamp when verified (ISO 8601)
    pub verified_at: Option<String>,
    /// File signature detected (magic bytes as hex)
    pub file_signature: Option<String>,
    /// Parser version used for validation
    pub parser_version: Option<String>,
    /// Whether asset identity matches component
    pub identity_match: Option<bool>,
    /// Acquisition method: "automatic" or "manual"
    pub acquisition_method: String,
    /// Additional notes
    pub notes: Option<String>,
    /// Created timestamp (ISO 8601)
    pub created_at: String,
    /// Updated timestamp (ISO 8601)
    pub updated_at: String,
}

impl AssetRecord {
    /// Create a new asset record in SOURCE_DECLARED state.
    pub fn new(
        component_mpn: String,
        manufacturer: String,
        category: String,
        asset_type: AssetType,
        source_url: String,
        source_provider: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: None,
            component_mpn,
            manufacturer,
            category,
            asset_type,
            source_url,
            final_url: None,
            http_status: None,
            content_type: None,
            file_size: None,
            local_path: None,
            sha256: None,
            state: AssetState::SourceDeclared,
            failure_reason: None,
            attempts: 0,
            last_attempt: None,
            source_provider,
            http_headers: None,
            downloaded_at: None,
            verified_at: None,
            file_signature: None,
            parser_version: None,
            identity_match: None,
            acquisition_method: "automatic".to_string(),
            notes: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Create a new manual import record.
    #[allow(clippy::too_many_arguments)]
    pub fn new_manual(
        component_mpn: String,
        manufacturer: String,
        category: String,
        asset_type: AssetType,
        local_path: PathBuf,
        sha256: String,
        file_size: u64,
        imported_by: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let mut record = Self {
            id: None,
            component_mpn,
            manufacturer,
            category,
            asset_type,
            source_url: "manual_import".to_string(),
            final_url: None,
            http_status: None,
            content_type: None,
            file_size: Some(file_size),
            local_path: Some(local_path),
            sha256: Some(sha256),
            state: AssetState::UserProvided,
            failure_reason: None,
            attempts: 1,
            last_attempt: Some(now.clone()),
            source_provider: "manual".to_string(),
            http_headers: None,
            downloaded_at: Some(now.clone()),
            verified_at: Some(now.clone()),
            file_signature: None,
            parser_version: None,
            identity_match: Some(true),
            acquisition_method: "manual".to_string(),
            notes: Some(format!("Imported by {}", imported_by)),
            created_at: now.clone(),
            updated_at: now,
        };
        record.verify_signature();
        record
    }

    /// Update state and timestamp.
    pub fn set_state(&mut self, state: AssetState) {
        self.state = state;
        self.updated_at = chrono::Utc::now().to_rfc3339();
        if state == AssetState::Verified {
            self.verified_at = Some(self.updated_at.clone());
        }
    }

    /// Record a download attempt.
    pub fn record_attempt(&mut self, http_status: Option<u16>, content_type: Option<String>) {
        self.attempts += 1;
        self.last_attempt = Some(chrono::Utc::now().to_rfc3339());
        self.http_status = http_status;
        self.content_type = content_type;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Verify file signature matches asset type.
    pub fn verify_signature(&mut self) -> bool {
        if let Some(path) = &self.local_path {
            if let Ok(mut file) = std::fs::File::open(path) {
                use std::io::Read;
                let mut header = vec![0u8; 16];
                if file.read_exact(&mut header).is_ok() {
                    let magic = self.asset_type.magic_bytes();
                    let matches = header.starts_with(magic);
                    self.file_signature =
                        Some(hex::encode(&header[..magic.len().min(header.len())]));
                    self.identity_match = Some(matches);
                    return matches;
                }
            }
        }
        false
    }
}

/// Query for finding assets for a component.
#[derive(Debug, Clone)]
pub struct AssetQuery {
    /// Component MPN
    pub mpn: String,
    /// Component manufacturer
    pub manufacturer: String,
    /// Asset types to find
    pub asset_types: Vec<AssetType>,
    /// Only return verified assets
    pub only_verified: bool,
}

impl AssetQuery {
    pub fn new(mpn: String, manufacturer: String) -> Self {
        Self {
            mpn,
            manufacturer,
            asset_types: vec![
                AssetType::Datasheet,
                AssetType::Symbol,
                AssetType::Footprint,
                AssetType::Model3D,
            ],
            only_verified: true,
        }
    }

    pub fn for_datasheet(mpn: String, manufacturer: String) -> Self {
        Self {
            mpn,
            manufacturer,
            asset_types: vec![AssetType::Datasheet],
            only_verified: true,
        }
    }
}
