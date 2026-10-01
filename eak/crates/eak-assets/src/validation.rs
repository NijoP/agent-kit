//! File validation for asset types.

use crate::asset::{AssetState, AssetType};
use std::path::Path;
use std::process::Command;

/// Result of asset validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// File is valid for the asset type
    Valid,
    /// File is empty
    EmptyFile,
    /// File signature doesn't match expected type
    InvalidSignature,
    /// Parser failed to read the file
    ParseFailed,
    /// File appears to be HTML (anti-bot page, login, etc.)
    HtmlContent,
    /// File appears to be a Cloudflare challenge page
    CloudflareChallenge,
    /// File is corrupt or unreadable
    CorruptFile,
    /// Other validation error
    Other(String),
}

impl ValidationResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    pub fn to_asset_state(&self) -> AssetState {
        match self {
            ValidationResult::Valid => AssetState::Parsed,
            ValidationResult::EmptyFile => AssetState::InvalidSignature,
            ValidationResult::InvalidSignature => AssetState::InvalidSignature,
            ValidationResult::ParseFailed => AssetState::ParseFailed,
            ValidationResult::HtmlContent => AssetState::HtmlResponse,
            ValidationResult::CloudflareChallenge => AssetState::CloudflareChallenge,
            ValidationResult::CorruptFile => AssetState::ParseFailed,
            ValidationResult::Other(_) => AssetState::ParseFailed,
        }
    }
}

/// Validate an asset file based on its type.
pub fn validate_asset(file_path: &Path, asset_type: AssetType) -> ValidationResult {
    // 1. Check file exists and is readable
    let metadata = match std::fs::metadata(file_path) {
        Ok(m) => m,
        Err(_) => return ValidationResult::Other("File not found or unreadable".to_string()),
    };

    if metadata.len() == 0 {
        return ValidationResult::EmptyFile;
    }

    // 2. Check magic bytes / file signature
    let mut file = match std::fs::File::open(file_path) {
        Ok(f) => f,
        Err(_) => return ValidationResult::CorruptFile,
    };

    use std::io::Read;
    let mut header = vec![0u8; 32];
    let bytes_read = match file.read(&mut header) {
        Ok(n) => n,
        Err(_) => return ValidationResult::CorruptFile,
    };

    if bytes_read < 4 {
        return ValidationResult::InvalidSignature;
    }

    let expected_magic = asset_type.magic_bytes();
    if !header.starts_with(expected_magic) {
        // Check for HTML content (anti-bot pages, login pages, etc.)
        let header_str = String::from_utf8_lossy(&header[..bytes_read.min(2000)]).to_lowercase();
        if header_str.contains("<html") || header_str.contains("<!doctype") {
            if header_str.contains("cloudflare")
                || header_str.contains("turnstile")
                || header_str.contains("challenge")
            {
                return ValidationResult::CloudflareChallenge;
            }
            return ValidationResult::HtmlContent;
        }
        return ValidationResult::InvalidSignature;
    }

    // 3. Parser validation based on type
    match asset_type {
        AssetType::Datasheet | AssetType::AppNote => validate_pdf(file_path),
        AssetType::Symbol => validate_kicad_symbol(file_path),
        AssetType::Footprint => validate_kicad_footprint(file_path),
        AssetType::Model3D => validate_step(file_path),
    }
}

/// Validate PDF using pdfinfo.
fn validate_pdf(file_path: &Path) -> ValidationResult {
    let output = Command::new("pdfinfo").arg(file_path).output();

    match output {
        Ok(result) => {
            if result.status.success() {
                // Check for suspicious content in pdfinfo output
                let stdout = String::from_utf8_lossy(&result.stdout);
                // pdfinfo outputs "Encrypted: yes" or "Encrypted: no" - check for "Encrypted: yes"
                // Also check for "Error:" which might indicate corruption
                if stdout.contains("Encrypted: yes") || stdout.contains("Error:") {
                    ValidationResult::ParseFailed
                } else {
                    ValidationResult::Valid
                }
            } else {
                ValidationResult::ParseFailed
            }
        }
        Err(_) => {
            // pdfinfo not available - just check signature was valid
            ValidationResult::Valid
        }
    }
}

/// Validate KiCad symbol file.
fn validate_kicad_symbol(file_path: &Path) -> ValidationResult {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return ValidationResult::ParseFailed,
    };

    // Basic KiCad symbol validation
    if content.contains("(kicad_symbol_lib") && content.contains(")") {
        ValidationResult::Valid
    } else {
        ValidationResult::ParseFailed
    }
}

/// Validate KiCad footprint file.
fn validate_kicad_footprint(file_path: &Path) -> ValidationResult {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return ValidationResult::ParseFailed,
    };

    // Basic KiCad footprint validation
    if content.contains("(footprint") && content.contains(")") {
        ValidationResult::Valid
    } else {
        ValidationResult::ParseFailed
    }
}

/// Validate STEP file.
fn validate_step(file_path: &Path) -> ValidationResult {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return ValidationResult::ParseFailed,
    };

    // STEP file must have ISO-10303-21 header and FILE_SCHEMA
    if content.contains("ISO-10303-21") && content.contains("FILE_SCHEMA") {
        ValidationResult::Valid
    } else {
        ValidationResult::ParseFailed
    }
}

/// Compute SHA-256 hash of a file.
pub fn compute_sha256(file_path: &Path) -> Result<String, std::io::Error> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Check if a file is likely an HTML anti-bot page.
pub fn detect_anti_bot(content: &[u8]) -> bool {
    let content_str = String::from_utf8_lossy(content).to_lowercase();
    content_str.contains("cloudflare")
        || content_str.contains("turnstile")
        || content_str.contains("challenge")
        || (content_str.contains("<html") && content_str.contains("captcha"))
        || (content_str.contains("<html") && content_str.contains("access denied"))
        || (content_str.contains("<html") && content_str.contains("please wait"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_pdf_signature_validation() {
        // Valid PDF - minimal but complete PDF structure that works with pdfinfo
        let mut file = NamedTempFile::new().unwrap();
        // This is a minimal valid PDF that pdfinfo can parse
        file.write_all(&[
            0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x34, 0x0a, // %PDF-1.4\n
            0x31, 0x20, 0x30, 0x20, 0x6f, 0x62, 0x6a, 0x0a, // 1 0 obj\n
            0x3c, 0x3c, 0x20, 0x2f, 0x54, 0x79, 0x70, 0x65, 0x20, 0x2f, 0x43, 0x61, 0x74, 0x61,
            0x6c, 0x6f, 0x67, 0x20, 0x2f, 0x50, 0x61, 0x67, 0x65, 0x73, 0x20, 0x32, 0x20, 0x30,
            0x20, 0x52, 0x20, 0x3e, 0x3e, 0x0a, // << /Type /Catalog /Pages 2 0 R >>\n
            0x65, 0x6e, 0x64, 0x6f, 0x62, 0x6a, 0x0a, // endobj\n
            0x32, 0x20, 0x30, 0x20, 0x6f, 0x62, 0x6a, 0x0a, // 2 0 obj\n
            0x3c, 0x3c, 0x20, 0x2f, 0x54, 0x79, 0x70, 0x65, 0x20, 0x2f, 0x50, 0x61, 0x67, 0x65,
            0x73, 0x20, 0x2f, 0x4b, 0x69, 0x64, 0x73, 0x20, 0x5b, 0x33, 0x20, 0x30, 0x20, 0x52,
            0x5d, 0x20, 0x2f, 0x43, 0x6f, 0x75, 0x6e, 0x74, 0x20, 0x31, 0x20, 0x3e, 0x3e,
            0x0a, // << /Type /Pages /Kids [3 0 R] /Count 1 >>\n
            0x65, 0x6e, 0x64, 0x6f, 0x62, 0x6a, 0x0a, // endobj\n
            0x33, 0x20, 0x30, 0x20, 0x6f, 0x62, 0x6a, 0x0a, // 3 0 obj\n
            0x3c, 0x3c, 0x20, 0x2f, 0x54, 0x79, 0x70, 0x65, 0x20, 0x2f, 0x50, 0x61, 0x67, 0x65,
            0x20, 0x2f, 0x50, 0x61, 0x72, 0x65, 0x6e, 0x74, 0x20, 0x32, 0x20, 0x30, 0x20, 0x52,
            0x20, 0x2f, 0x4d, 0x65, 0x64, 0x69, 0x61, 0x42, 0x6f, 0x78, 0x20, 0x5b, 0x30, 0x20,
            0x30, 0x20, 0x36, 0x31, 0x32, 0x20, 0x37, 0x39, 0x32, 0x5d, 0x20, 0x3e, 0x3e,
            0x0a, // << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\n
            0x65, 0x6e, 0x64, 0x6f, 0x62, 0x6a, 0x0a, // endobj\n
            0x78, 0x72, 0x65, 0x66, 0x0a, // xref\n
            0x30, 0x20, 0x34, 0x0a, // 0 4\n
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x20, 0x36,
            0x35, 0x35, 0x33, 0x35, 0x20, 0x66, 0x20, 0x0a, // 0000000000 65535 f \n
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x39, 0x20, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x20, 0x6e, 0x20, 0x0a, // 0000000009 00000 n \n
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x35, 0x38, 0x20, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x20, 0x6e, 0x20, 0x0a, // 0000000058 00000 n \n
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x31, 0x31, 0x35, 0x20, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x20, 0x6e, 0x20, 0x0a, // 0000000115 00000 n \n
            0x74, 0x72, 0x61, 0x69, 0x6c, 0x65, 0x72, 0x0a, // trailer\n
            0x3c, 0x3c, 0x20, 0x2f, 0x53, 0x69, 0x7a, 0x65, 0x20, 0x34, 0x20, 0x2f, 0x52, 0x6f,
            0x6f, 0x74, 0x20, 0x31, 0x20, 0x30, 0x20, 0x52, 0x20, 0x3e, 0x3e,
            0x0a, // << /Size 4 /Root 1 0 R >>\n
            0x73, 0x74, 0x61, 0x72, 0x74, 0x78, 0x72, 0x65, 0x66, 0x0a, // startxref\n
            0x32, 0x30, 0x30, 0x0a, // 200\n
            0x25, 0x25, 0x45, 0x4f, 0x46, 0x0a, // %%EOF\n
        ])
        .unwrap();
        let result = validate_asset(file.path(), AssetType::Datasheet);
        assert_eq!(result, ValidationResult::Valid);

        // HTML pretending to be PDF (not a Cloudflare page)
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"<html><body>Access denied - please log in</body></html>")
            .unwrap();
        let result = validate_asset(file.path(), AssetType::Datasheet);
        assert_eq!(result, ValidationResult::HtmlContent);

        // Cloudflare challenge page
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"<html><body>Cloudflare Turnstile challenge page</body></html>")
            .unwrap();
        let result = validate_asset(file.path(), AssetType::Datasheet);
        assert_eq!(result, ValidationResult::CloudflareChallenge);
    }

    #[test]
    fn test_kicad_symbol_validation() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"(kicad_symbol_lib (version 20211014) (generator kicad_symbol_editor))")
            .unwrap();
        let result = validate_asset(file.path(), AssetType::Symbol);
        assert_eq!(result, ValidationResult::Valid);
    }

    #[test]
    fn test_kicad_footprint_validation() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"(footprint \"Resistor_0402\" (version 20211014) (generator pcbnew))")
            .unwrap();
        let result = validate_asset(file.path(), AssetType::Footprint);
        assert_eq!(result, ValidationResult::Valid);
    }

    #[test]
    fn test_step_validation() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"ISO-10303-21\nHEADER;\nFILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\nENDSEC;\n")
            .unwrap();
        let result = validate_asset(file.path(), AssetType::Model3D);
        assert_eq!(result, ValidationResult::Valid);
    }

    #[test]
    fn test_empty_file_rejected() {
        let file = NamedTempFile::new().unwrap();
        let result = validate_asset(file.path(), AssetType::Datasheet);
        assert_eq!(result, ValidationResult::EmptyFile);
    }

    #[test]
    fn test_sha256_computation() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        let hash = compute_sha256(file.path()).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_anti_bot_detection() {
        assert!(detect_anti_bot(b"<html>Cloudflare challenge</html>"));
        assert!(detect_anti_bot(b"<html>Turnstile verification</html>"));
        assert!(detect_anti_bot(b"<html>Please wait while we verify</html>"));
        assert!(!detect_anti_bot(b"%PDF-1.4 valid pdf"));
        assert!(!detect_anti_bot(b"(kicad_symbol_lib ...)"));
    }
}
