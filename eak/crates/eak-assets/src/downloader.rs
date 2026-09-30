use crate::asset::{AssetRecord, AssetState, AssetType};
use crate::validation::{compute_sha256, detect_anti_bot, validate_asset, ValidationResult};
use anyhow::{Context, Result};
use std::path::Path;
use std::time::Duration;

/// Result of a download attempt with full provenance.
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub final_url: String,
    pub http_status: u16,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub headers: String, // JSON serialized headers
    pub body: Vec<u8>,
    pub redirect_chain: Vec<String>,
    pub duration_ms: u64,
}

/// HTTP downloader with honest error handling.
pub struct AssetDownloader {
    client: reqwest::blocking::Client,
    max_redirects: usize,
    timeout: Duration,
}

impl Default for AssetDownloader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetDownloader {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .timeout(Duration::from_secs(30))
            .user_agent("EAK-AssetDownloader/1.0 (+https://github.com/electronics-agent-kit)")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            max_redirects: 10,
            timeout: Duration::from_secs(30),
        }
    }

    /// Download an asset from a source URL with full provenance tracking.
    pub fn download(&self, asset: &mut AssetRecord) -> Result<DownloadResult> {
        let start = std::time::Instant::now();
        asset.set_state(AssetState::DownloadAttempted);

        let response = self.client.get(&asset.source_url).send();

        let (final_url, http_status, content_type, content_length, headers, body, redirect_chain) =
            match response {
                Ok(resp) => {
                    let final_url = resp.url().to_string();
                    let http_status = resp.status().as_u16();
                    let content_type = resp
                        .headers()
                        .get(reqwest::header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());
                    let content_length = resp.content_length();

                    // Serialize headers for provenance
                    let headers_json = serde_json::to_string(
                        &resp
                            .headers()
                            .iter()
                            .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("")))
                            .collect::<Vec<_>>(),
                    )
                    .unwrap_or_default();

                    let bytes = resp.bytes().context("Failed to read response body")?;
                    let body = bytes.to_vec();

                    // Track redirect chain (simplified - just final URL)
                    let redirect_chain = vec![asset.source_url.clone(), final_url.clone()];

                    (
                        final_url,
                        http_status,
                        content_type,
                        content_length,
                        headers_json,
                        body,
                        redirect_chain,
                    )
                }
                Err(e) => {
                    asset.record_attempt(None, None);
                    asset.set_state(AssetState::HttpError);
                    asset.failure_reason = Some(format!("Network error: {}", e));
                    anyhow::bail!("Network error: {}", e);
                }
            };

        asset.final_url = Some(final_url.clone());
        asset.http_status = Some(http_status);
        asset.content_type = content_type.clone();
        asset.file_size = content_length;
        asset.http_headers = Some(headers.clone());

        let duration_ms = start.elapsed().as_millis() as u64;

        // Record the attempt
        asset.record_attempt(Some(http_status), content_type.clone());

        // Handle HTTP status codes
        match http_status {
            200 => {
                asset.set_state(AssetState::Downloaded);
            }
            403 => {
                asset.set_state(AssetState::AccessForbidden);
                asset.failure_reason = Some("HTTP 403 Forbidden".to_string());
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body: vec![],
                    redirect_chain,
                    duration_ms,
                });
            }
            404 => {
                asset.set_state(AssetState::SourceNotFound);
                asset.failure_reason = Some("HTTP 404 Not Found".to_string());
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body: vec![],
                    redirect_chain,
                    duration_ms,
                });
            }
            429 => {
                asset.set_state(AssetState::HttpError);
                asset.failure_reason = Some("HTTP 429 Too Many Requests".to_string());
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body: vec![],
                    redirect_chain,
                    duration_ms,
                });
            }
            500..=599 => {
                asset.set_state(AssetState::HttpError);
                asset.failure_reason = Some(format!("HTTP {} Server Error", http_status));
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body: vec![],
                    redirect_chain,
                    duration_ms,
                });
            }
            _ => {
                asset.set_state(AssetState::HttpError);
                asset.failure_reason = Some(format!("HTTP {} {}", http_status, http_status));
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body: vec![],
                    redirect_chain,
                    duration_ms,
                });
            }
        }

        // Check for anti-bot content in response body
        if detect_anti_bot(&body) {
            asset.set_state(AssetState::AccessBlocked);
            asset.failure_reason =
                Some("Anti-bot protection detected (Cloudflare/Turnstile)".to_string());
            return Ok(DownloadResult {
                final_url,
                http_status,
                content_type,
                content_length,
                headers,
                body,
                redirect_chain,
                duration_ms,
            });
        }

        // Check for HTML content when expecting binary
        if let Some(ct) = &content_type {
            let ct_lower = ct.to_lowercase();
            let is_expected_binary = matches!(
                asset.asset_type,
                AssetType::Datasheet | AssetType::AppNote | AssetType::Model3D
            );
            if is_expected_binary
                && (ct_lower.contains("text/html") || ct_lower.contains("text/plain"))
            {
                asset.set_state(AssetState::HtmlResponse);
                asset.failure_reason =
                    Some(format!("Received HTML instead of binary content: {}", ct));
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body,
                    redirect_chain,
                    duration_ms,
                });
            }
        }

        // Save to temporary file for validation
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("eak_download_{}.tmp", asset.component_mpn));
        std::fs::write(&temp_file, &body).context("Failed to write temp file")?;

        // Validate the downloaded content
        let validation = validate_asset(&temp_file, asset.asset_type);

        // Compute SHA-256
        let sha256 = compute_sha256(&temp_file).context("Failed to compute SHA-256")?;
        asset.sha256 = Some(sha256.clone());
        asset.file_size = Some(body.len() as u64);

        match validation {
            ValidationResult::Valid => {
                asset.set_state(AssetState::SignatureValid);
            }
            ValidationResult::InvalidSignature => {
                asset.set_state(AssetState::InvalidSignature);
                asset.failure_reason =
                    Some("File signature doesn't match expected type".to_string());
                std::fs::remove_file(&temp_file).ok();
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body,
                    redirect_chain,
                    duration_ms,
                });
            }
            ValidationResult::HtmlContent => {
                asset.set_state(AssetState::HtmlResponse);
                asset.failure_reason =
                    Some("Downloaded content is HTML, not expected binary format".to_string());
                std::fs::remove_file(&temp_file).ok();
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body,
                    redirect_chain,
                    duration_ms,
                });
            }
            ValidationResult::CloudflareChallenge => {
                asset.set_state(AssetState::CloudflareChallenge);
                asset.failure_reason =
                    Some("Cloudflare challenge detected in downloaded content".to_string());
                std::fs::remove_file(&temp_file).ok();
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body,
                    redirect_chain,
                    duration_ms,
                });
            }
            ValidationResult::ParseFailed => {
                asset.set_state(AssetState::ParseFailed);
                asset.failure_reason = Some("Parser validation failed".to_string());
                std::fs::remove_file(&temp_file).ok();
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body,
                    redirect_chain,
                    duration_ms,
                });
            }
            _ => {
                asset.set_state(AssetState::ParseFailed);
                asset.failure_reason = Some(format!("Validation failed: {:?}", validation));
                std::fs::remove_file(&temp_file).ok();
                return Ok(DownloadResult {
                    final_url,
                    http_status,
                    content_type,
                    content_length,
                    headers,
                    body,
                    redirect_chain,
                    duration_ms,
                });
            }
        }

        // Move temp file to asset store
        let asset_root = Path::new("data/assets");
        let asset_dir = asset_root.join(asset.asset_type.to_string());
        std::fs::create_dir_all(&asset_dir).context("Failed to create asset directory")?;

        let safe_mpn = asset.component_mpn.replace(['/', '\\', ':'], "_");
        let ext = asset.asset_type.extension();
        let filename = format!("{}_{}.{}", asset.manufacturer, safe_mpn, ext);
        let local_path = asset_dir.join(&filename);

        std::fs::copy(&temp_file, &local_path).context("Failed to copy to asset store")?;
        std::fs::remove_file(&temp_file).ok();

        asset.local_path = Some(local_path.clone());

        // Final verification
        let final_validation = validate_asset(&local_path, asset.asset_type);
        if !final_validation.is_success() {
            asset.set_state(AssetState::ParseFailed);
            asset.failure_reason = Some(format!(
                "Post-copy validation failed: {:?}",
                final_validation
            ));
            std::fs::remove_file(&local_path).ok();
            return Ok(DownloadResult {
                final_url,
                http_status,
                content_type,
                content_length,
                headers,
                body,
                redirect_chain,
                duration_ms,
            });
        }

        asset.set_state(AssetState::Verified);
        asset.downloaded_at = Some(chrono::Utc::now().to_rfc3339());
        asset.verified_at = Some(chrono::Utc::now().to_rfc3339());
        asset.identity_match = Some(true);
        asset.acquisition_method = "automatic".to_string();

        Ok(DownloadResult {
            final_url,
            http_status,
            content_type,
            content_length,
            headers,
            body,
            redirect_chain,
            duration_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{AssetRecord, AssetType};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_downloader_creation() {
        let downloader = AssetDownloader::new();
        assert_eq!(downloader.max_redirects, 10);
        assert_eq!(downloader.timeout, Duration::from_secs(30));
    }
}
