//! Datasheet parser for extracting structured facts from verified PDF datasheets.
//!
//! This module provides deterministic, testable PDF parsing without ML dependencies.
//! It uses system tools (pdftotext, pdfinfo) for text extraction and metadata.

use crate::asset::{AssetRecord, AssetState, AssetType};
use crate::facts::{DatasheetFact, FactKind, ParameterFact, ProvenanceInfo};
use crate::validation::ValidationResult;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

/// Configuration for the datasheet parser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Parser version identifier
    pub version: String,
    /// Maximum pages to parse (0 = all)
    pub max_pages: usize,
    /// Whether to extract tables
    pub extract_tables: bool,
    /// Confidence threshold for parameter extraction
    pub confidence_threshold: f64,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            max_pages: 0,
            extract_tables: true,
            confidence_threshold: 0.7,
        }
    }
}

/// Result of parsing a datasheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// The asset ID that was parsed
    pub asset_id: i64,
    /// Number of pages in the PDF
    pub page_count: usize,
    /// Detected title
    pub title: Option<String>,
    /// Detected manufacturer
    pub manufacturer: Option<String>,
    /// Detected MPN (Manufacturer Part Number)
    pub mpn: Option<String>,
    /// Extracted facts
    pub facts: Vec<DatasheetFact>,
    /// Section headers detected
    pub sections: Vec<SectionInfo>,
    /// Parser version used
    pub parser_version: String,
    /// Timestamp of parsing
    pub parsed_at: String,
    /// Any warnings during parsing
    pub warnings: Vec<String>,
    /// Whether parsing succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Information about a detected section in the datasheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInfo {
    /// Section title/heading
    pub title: String,
    /// Page number where section starts (1-indexed)
    pub page: usize,
    /// Approximate character offset in full text
    pub offset: usize,
    /// Sub-sections
    pub subsections: Vec<SectionInfo>,
}

/// Text content extracted from a PDF page.
#[derive(Debug, Clone)]
struct PageText {
    page_num: usize,
    text: String,
}

/// Main parser entry point.
pub struct DatasheetParser {
    config: ParserConfig,
}

impl DatasheetParser {
    /// Create a new parser with default configuration.
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    /// Create a new parser with custom configuration.
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parse a datasheet PDF and extract structured facts.
    pub fn parse(&self, asset: &AssetRecord) -> Result<ParseResult> {
        let local_path = asset
            .local_path
            .as_ref()
            .context("Asset has no local path")?;

        if !local_path.exists() {
            bail!("Asset file not found: {}", local_path.display());
        }

        // Verify it's a datasheet type
        if asset.asset_type != AssetType::Datasheet && asset.asset_type != AssetType::AppNote {
            bail!("Asset is not a datasheet or app note");
        }

        let started_at = Utc::now().to_rfc3339();
        let mut warnings = Vec::new();

        // Get page count and basic info
        let page_count = self.get_page_count(local_path)?;
        let max_pages = if self.config.max_pages > 0 {
            self.config.max_pages.min(page_count)
        } else {
            page_count
        };

        // Extract text from all pages
        let pages = self.extract_text(local_path, max_pages)?;

        if pages.is_empty() {
            warnings.push("No text extracted from PDF".to_string());
        }

        // Detect title, manufacturer, MPN
        let title = self.detect_title(&pages);
        let manufacturer = self.detect_manufacturer(&pages, asset);
        let mpn = self.detect_mpn(&pages, asset);

        // Detect sections
        let sections = self.detect_sections(&pages);

        // Extract facts
        let facts = self.extract_facts(&pages, asset, &sections)?;

        let parsed_at = Utc::now().to_rfc3339();

        Ok(ParseResult {
            asset_id: asset.id.unwrap_or(0),
            page_count,
            title,
            manufacturer,
            mpn,
            facts,
            sections,
            parser_version: self.config.version.clone(),
            parsed_at,
            warnings,
            success: true,
            error: None,
        })
    }

    /// Get page count using pdfinfo.
    fn get_page_count(&self, path: &Path) -> Result<usize> {
        let output = Command::new("pdfinfo").arg(path).output()?;

        if !output.status.success() {
            bail!(
                "pdfinfo failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Pages:") {
                let count = line["Pages:".len()..].trim().parse()?;
                return Ok(count);
            }
        }
        bail!("Could not determine page count from pdfinfo output");
    }

    /// Extract text from PDF pages using pdftotext.
    fn extract_text(&self, path: &Path, max_pages: usize) -> Result<Vec<PageText>> {
        let mut pages = Vec::new();

        for page_num in 1..=max_pages {
            let output = Command::new("pdftotext")
                .args([
                    "-f",
                    &page_num.to_string(),
                    "-l",
                    &page_num.to_string(),
                    "-layout",
                    path.to_str().unwrap(),
                    "-",
                ])
                .output()?;

            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).to_string();
                pages.push(PageText { page_num, text });
            } else {
                // Try without -layout as fallback
                let output = Command::new("pdftotext")
                    .args([
                        "-f",
                        &page_num.to_string(),
                        "-l",
                        &page_num.to_string(),
                        path.to_str().unwrap(),
                        "-",
                    ])
                    .output()?;
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    pages.push(PageText { page_num, text });
                }
            }
        }

        Ok(pages)
    }

    /// Detect document title from first few pages.
    fn detect_title(&self, pages: &[PageText]) -> Option<String> {
        // Look at first 2 pages for title
        for page in pages.iter().take(2) {
            let lines: Vec<&str> = page.text.lines().collect();
            for (i, line) in lines.iter().enumerate().take(10) {
                let trimmed = line.trim();
                if trimmed.len() > 5 && trimmed.len() < 200 {
                    // Skip common non-title patterns
                    if !trimmed.to_lowercase().contains("page")
                        && !trimmed.to_lowercase().contains("rev")
                        && !trimmed.to_lowercase().contains("draft")
                        && !trimmed
                            .chars()
                            .all(|c| c.is_ascii_digit() || c.is_ascii_punctuation())
                    {
                        // Check if next line looks like a subtitle or manufacturer
                        if i + 1 < lines.len() {
                            let next = lines[i + 1].trim();
                            if next.len() > 2 {
                                return Some(trimmed.to_string());
                            }
                        } else {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Detect manufacturer from text and asset record.
    fn detect_manufacturer(&self, pages: &[PageText], asset: &AssetRecord) -> Option<String> {
        // First, check asset record
        if !asset.manufacturer.is_empty() {
            return Some(asset.manufacturer.clone());
        }

        // Common manufacturer names to search for
        let manufacturers = [
            "Texas Instruments",
            "TI",
            "Analog Devices",
            "ADI",
            "STMicroelectronics",
            "STM",
            "Infineon",
            "NXP",
            "ON Semiconductor",
            "onsemi",
            "Microchip",
            "Renesas",
            "Maxim Integrated",
            "Maxim",
            "Diodes Incorporated",
            "Diodes",
            "Vishay",
            "Yageo",
            "Murata",
            "TDK",
            "Panasonic",
            "Samsung",
            "Kemet",
            "AVX",
            "Bourns",
            "Wurth",
            "Würth",
            "TT Electronics",
            "TT Electronics",
            "ROHM",
            "Semtech",
            "Skyworks",
            "Qorvo",
            "Broadcom",
            "Cypress",
            "Lattice",
            "Xilinx",
            "Altera",
            "Intel",
            "AMD",
            "Qualcomm",
            "MediaTek",
            "Realtek",
            "Silicon Labs",
            "Silabs",
            "Nordic",
            "Espressif",
            "ST",
            "NXP Semiconductors",
        ];

        let full_text = pages
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let full_lower = full_text.to_lowercase();

        for mfr in &manufacturers {
            if full_lower.contains(&mfr.to_lowercase()) {
                return Some(mfr.to_string());
            }
        }

        None
    }

    /// Detect MPN from text and asset record.
    fn detect_mpn(&self, pages: &[PageText], asset: &AssetRecord) -> Option<String> {
        // First, check asset record
        if !asset.component_mpn.is_empty() {
            return Some(asset.component_mpn.clone());
        }

        // Look for MPN patterns in first few pages
        for page in pages.iter().take(3) {
            let text = &page.text;

            // Pattern: alphanumeric with dashes, dots, underscores (common MPN formats)
            let mpn_regex = regex::Regex::new(r"(?i)(?:part\s*(?:number|no|#)?|mpn|ordering\s*code|device\s*(?:number|name)?)\s*[:=]\s*([A-Z0-9][A-Z0-9\-\._/]{3,})").ok()?;
            if let Some(caps) = mpn_regex.captures(text) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().trim().to_string());
                }
            }

            // Pattern: typical MPN format like LM358, LM7805, ATMEGA328P, etc.
            let generic_mpn = regex::Regex::new(r"\b([A-Z]{2,}[0-9]{2,}[A-Z0-9\-\._]*)\b").ok()?;
            for caps in generic_mpn.captures_iter(text) {
                if let Some(m) = caps.get(1) {
                    let candidate = m.as_str();
                    // Filter out common false positives
                    if !candidate
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                    {
                        continue;
                    }
                    if candidate.len() >= 4 && candidate.len() <= 30 {
                        return Some(candidate.to_string());
                    }
                }
            }
        }

        None
    }

    /// Detect section headers in the document.
    fn detect_sections(&self, pages: &[PageText]) -> Vec<SectionInfo> {
        let mut sections = Vec::new();
        let mut char_offset = 0;

        let section_keywords = [
            "electrical characteristics",
            "absolute maximum ratings",
            "recommended operating conditions",
            "thermal characteristics",
            "pin configuration",
            "pin description",
            "pinout",
            "package",
            "mechanical",
            "dimensions",
            "footprint",
            "land pattern",
            "typical performance",
            "typical characteristics",
            "application information",
            "reference design",
            "layout",
            "ordering information",
            "part marking",
            "revision history",
            "datasheet",
            "specifications",
            "features",
            "description",
            "functional block diagram",
            "functional description",
            "theory of operation",
            "register map",
            "register description",
            "timing",
            "ac characteristics",
            "dc characteristics",
            "power supply",
            "voltage",
            "current",
            "temperature",
        ];

        for page in pages {
            let lines: Vec<&str> = page.text.lines().collect();
            for line in lines {
                let trimmed = line.trim();
                let lower = trimmed.to_lowercase();

                // Check if line looks like a section header
                if trimmed.len() > 3 && trimmed.len() < 100 {
                    for keyword in &section_keywords {
                        if lower.contains(keyword) {
                            // Check if it's likely a header (all caps, or title case, or ends with colon)
                            let is_header = trimmed.chars().all(|c| {
                                c.is_ascii_uppercase()
                                    || c.is_ascii_whitespace()
                                    || c.is_ascii_punctuation()
                            }) || trimmed.ends_with(':')
                                || (trimmed.split_whitespace().count() <= 6
                                    && trimmed
                                        .chars()
                                        .next()
                                        .map(|c| c.is_ascii_uppercase())
                                        .unwrap_or(false));

                            if is_header {
                                sections.push(SectionInfo {
                                    title: trimmed.to_string(),
                                    page: page.page_num,
                                    offset: char_offset + page.text.find(trimmed).unwrap_or(0),
                                    subsections: Vec::new(),
                                });
                                break;
                            }
                        }
                    }
                }
            }
            char_offset += page.text.len() + 1;
        }

        sections
    }

    /// Extract structured facts from the parsed text.
    fn extract_facts(
        &self,
        pages: &[PageText],
        asset: &AssetRecord,
        sections: &[SectionInfo],
    ) -> Result<Vec<DatasheetFact>> {
        let mut facts = Vec::new();

        // Combine all text for searching
        let full_text = pages
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Define parameter extraction patterns
        let patterns = self.build_extraction_patterns();

        for (kind, pattern) in patterns {
            for caps in pattern.captures_iter(&full_text) {
                if let (Some(value_match), Some(unit_match)) = (caps.get(1), caps.get(2)) {
                    let value_str = value_match.as_str().trim();
                    let unit_str = unit_match.as_str().trim();

                    // Parse value (handle ranges, min/typ/max)
                    if let Some(fact) = self.parse_parameter_fact(
                        kind,
                        value_str,
                        unit_str,
                        &full_text,
                        value_match.start(),
                        asset,
                        pages,
                    )? {
                        facts.push(fact);
                    }
                }
            }
        }

        // Also extract package info
        if let Some(package_fact) = self.extract_package_info(&full_text, asset, pages)? {
            facts.push(package_fact);
        }

        // Extract temperature range
        if let Some(temp_facts) = self.extract_temperature_range(&full_text, asset, pages)? {
            facts.extend(temp_facts);
        }

        // Extract lifecycle/status if mentioned
        if let Some(lifecycle_fact) = self.extract_lifecycle(&full_text, asset, pages)? {
            facts.push(lifecycle_fact);
        }

        Ok(facts)
    }

    /// Build regex patterns for parameter extraction.
    fn build_extraction_patterns(&self) -> Vec<(FactKind, regex::Regex)> {
        let mut patterns = Vec::new();

        // Voltage patterns
        patterns.push((
            FactKind::Voltage,
            regex::Regex::new(r"(?i)(?:voltage|V(?:in|out|cc|dd|ss|ref|bat|gate|drain|source|ce|be|gs|ds)?)\s*[:=]\s*([\d\.]+(?:\s*[\-\–]\s*[\d\.]+)?)\s*(V|mV|kV|µV|uV)\b").unwrap(),
        ));

        // Current patterns
        patterns.push((
            FactKind::Current,
            regex::Regex::new(r"(?i)(?:current|I(?:n|out|cc|dd|ss|ref|bat|gate|drain|source|ce|be|gs|ds|f|r)?)\s*[:=]\s*([\d\.]+(?:\s*[\-\–]\s*[\d\.]+)?)\s*(A|mA|µA|uA|nA|pA|kA)\b").unwrap(),
        ));

        // Resistance patterns
        patterns.push((
            FactKind::Resistance,
            regex::Regex::new(r"(?i)(?:resistance|impedance|R(?:ds|ds\(on\)|on|th|jc|ja)?)\s*[:=]\s*([\d\.]+(?:\s*[\-\–]\s*[\d\.]+)?)\s*(Ω|ohm|kΩ|kohm|MΩ|Mohm|mΩ|mohm)\b").unwrap(),
        ));

        // Capacitance patterns
        patterns.push((
            FactKind::Capacitance,
            regex::Regex::new(r"(?i)(?:capacitance|C(?:in|out|iss|oss|rss|jc|ja|load)?)\s*[:=]\s*([\d\.]+(?:\s*[\-\–]\s*[\d\.]+)?)\s*(F|µF|uF|nF|pF|fF)\b").unwrap(),
        ));

        // Inductance patterns
        patterns.push((
            FactKind::Inductance,
            regex::Regex::new(r"(?i)(?:inductance|L)\s*[:=]\s*([\d\.]+(?:\s*[\-\–]\s*[\d\.]+)?)\s*(H|µH|uH|mH|nH)\b").unwrap(),
        ));

        // Power patterns
        patterns.push((
            FactKind::Power,
            regex::Regex::new(r"(?i)(?:power|P(?:d|tot|max|diss)?)\s*[:=]\s*([\d\.]+(?:\s*[\-\–]\s*[\d\.]+)?)\s*(W|mW|µW|uW|kW)\b").unwrap(),
        ));

        // Frequency patterns
        patterns.push((
            FactKind::Frequency,
            regex::Regex::new(r"(?i)(?:frequency|f(?:osc|clk|sw|max|min|typ)?|bandwidth|GBW|gain.?bandwidth)\s*[:=]\s*([\d\.]+(?:\s*[\-\–]\s*[\d\.]+)?)\s*(Hz|kHz|MHz|GHz|THz)\b").unwrap(),
        ));

        // Temperature patterns
        patterns.push((
            FactKind::Temperature,
            regex::Regex::new(r"(?i)(?:temperature|T(?:a|j|stg|op|min|max|amb)?)\s*[:=]\s*([\-\d\.]+(?:\s*[\-\–]\s*[\-\d\.]+)?)\s*(°C|degC|C|K)\b").unwrap(),
        ));

        // Tolerance patterns
        patterns.push((
            FactKind::Tolerance,
            regex::Regex::new(r"(?i)(?:tolerance|accuracy|precision)\s*[:=]\s*([\d\.]+)\s*%\b")
                .unwrap(),
        ));

        // Dimension patterns
        patterns.push((
            FactKind::Dimension,
            regex::Regex::new(r"(?i)(?:dimension|length|width|height|thickness|pitch|diameter)\s*[:=]\s*([\d\.]+(?:\s*[\-\–]\s*[\d\.]+)?)\s*(mm|cm|m|mil|in|µm|um)\b").unwrap(),
        ));

        patterns
    }

    /// Parse a parameter fact from matched text.
    fn parse_parameter_fact(
        &self,
        kind: FactKind,
        value_str: &str,
        unit_str: &str,
        full_text: &str,
        match_start: usize,
        asset: &AssetRecord,
        pages: &[PageText],
    ) -> Result<Option<DatasheetFact>> {
        // Handle ranges (min-max or min-typ-max)
        let (value, tolerance) = self.parse_value_with_tolerance(value_str)?;

        // Map unit string to Unit enum
        let unit = self.parse_unit(unit_str, kind)?;

        // Create PhysicalQuantity
        let pq = eak_units::PhysicalQuantity::new(value, unit);

        // Find which page this match is on
        let page = self.find_page_for_offset(pages, match_start);

        // Build provenance
        let provenance = ProvenanceInfo {
            asset_id: asset.id.unwrap_or(0),
            asset_sha256: asset.sha256.clone().unwrap_or_default(),
            page,
            extracted_text: self.extract_context(full_text, match_start, 100),
            parser_version: self.config.version.clone(),
            extracted_at: Utc::now().to_rfc3339(),
            verified: false,
            manually_corrected: false,
            correction_notes: None,
        };

        let fact = DatasheetFact {
            id: None,
            asset_id: asset.id.unwrap_or(0),
            kind,
            parameter: ParameterFact {
                value: pq,
                tolerance,
                conditions: self.extract_conditions(full_text, match_start),
            },
            provenance,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        Ok(Some(fact))
    }

    /// Parse a value string that may contain ranges or tolerances.
    fn parse_value_with_tolerance(
        &self,
        value_str: &str,
    ) -> Result<(f64, Option<eak_units::Tolerance>)> {
        let cleaned = value_str.replace('–', "-").replace('—', "-");

        // Check for min-max range (e.g., "3.0 - 3.6" or "3.0-3.6")
        if cleaned.contains('-') && !cleaned.starts_with('-') {
            let parts: Vec<&str> = cleaned.split('-').collect();
            if parts.len() == 2 {
                let min = parts[0].trim().parse::<f64>()?;
                let max = parts[1].trim().parse::<f64>()?;
                let nominal = (min + max) / 2.0;
                let plus = max - nominal;
                let minus = nominal - min;
                return Ok((
                    nominal,
                    Some(eak_units::Tolerance::Absolute { plus, minus }),
                ));
            }
        }

        // Check for ± notation
        if cleaned.contains("±") || cleaned.contains("+/-") {
            let cleaned = cleaned.replace("±", "+/-");
            let parts: Vec<&str> = cleaned.split("+/-").collect();
            if parts.len() == 2 {
                let nominal = parts[0].trim().parse::<f64>()?;
                let tol = parts[1].trim().parse::<f64>()?;
                return Ok((
                    nominal,
                    Some(eak_units::Tolerance::Absolute {
                        plus: tol,
                        minus: tol,
                    }),
                ));
            }
        }

        // Check for percentage tolerance in the value (e.g., "10k ±5%")
        // This is handled separately by the tolerance pattern

        // Simple value
        let value = cleaned.parse::<f64>()?;
        Ok((value, None))
    }

    /// Map unit string to Unit enum.
    fn parse_unit(&self, unit_str: &str, kind: FactKind) -> Result<eak_units::Unit> {
        use eak_units::Unit::*;

        let unit = match (kind, unit_str.to_lowercase().as_str()) {
            (FactKind::Voltage, "v")
            | (FactKind::Voltage, "volt")
            | (FactKind::Voltage, "volts") => Volt,
            (FactKind::Voltage, "mv")
            | (FactKind::Voltage, "millivolt")
            | (FactKind::Voltage, "millivolts") => Millivolt,
            (FactKind::Voltage, "kv") | (FactKind::Voltage, "kilovolt") => Volt, // Map to Volt, handle scaling separately
            (FactKind::Voltage, "µv")
            | (FactKind::Voltage, "uv")
            | (FactKind::Voltage, "microvolt") => Millivolt, // Map to Millivolt

            (FactKind::Current, "a")
            | (FactKind::Current, "amp")
            | (FactKind::Current, "ampere")
            | (FactKind::Current, "amperes") => Ampere,
            (FactKind::Current, "ma")
            | (FactKind::Current, "milliamp")
            | (FactKind::Current, "milliamps") => Milliampere,
            (FactKind::Current, "µa")
            | (FactKind::Current, "ua")
            | (FactKind::Current, "microamp") => Milliampere, // Map to Milliampere
            (FactKind::Current, "na") | (FactKind::Current, "nanoamp") => Milliampere, // Map to Milliampere
            (FactKind::Current, "pa") | (FactKind::Current, "picoamp") => Milliampere, // Map to Milliampere

            (FactKind::Resistance, "ω")
            | (FactKind::Resistance, "ohm")
            | (FactKind::Resistance, "ohms") => Ohm,
            (FactKind::Resistance, "kω")
            | (FactKind::Resistance, "kohm")
            | (FactKind::Resistance, "kilohm") => Kilohm,
            (FactKind::Resistance, "mω")
            | (FactKind::Resistance, "mohm")
            | (FactKind::Resistance, "milliohm") => Ohm, // Map to Ohm
            (FactKind::Resistance, "mω") | (FactKind::Resistance, "megohm") => Kilohm, // Map to Kilohm

            (FactKind::Capacitance, "f")
            | (FactKind::Capacitance, "farad")
            | (FactKind::Capacitance, "farads") => Farad,
            (FactKind::Capacitance, "µf")
            | (FactKind::Capacitance, "uf")
            | (FactKind::Capacitance, "microfarad") => Microfarad,
            (FactKind::Capacitance, "nf") | (FactKind::Capacitance, "nanofarad") => Nanofarad,
            (FactKind::Capacitance, "pf") | (FactKind::Capacitance, "picofarad") => Picofarad,

            (FactKind::Inductance, "h")
            | (FactKind::Inductance, "henry")
            | (FactKind::Inductance, "henries") => Henry,
            (FactKind::Inductance, "µh")
            | (FactKind::Inductance, "uh")
            | (FactKind::Inductance, "microhenry") => Microhenry,
            (FactKind::Inductance, "mh") | (FactKind::Inductance, "millihenry") => Henry, // Map to Henry
            (FactKind::Inductance, "nh") | (FactKind::Inductance, "nanohenry") => Nanohenry,

            (FactKind::Power, "w") | (FactKind::Power, "watt") | (FactKind::Power, "watts") => Watt,
            (FactKind::Power, "mw") | (FactKind::Power, "milliwatt") => Milliwatt,
            (FactKind::Power, "µw") | (FactKind::Power, "uw") | (FactKind::Power, "microwatt") => {
                Milliwatt
            } // Map to Milliwatt
            (FactKind::Power, "kw") | (FactKind::Power, "kilowatt") => Watt, // Map to Watt

            (FactKind::Frequency, "hz") | (FactKind::Frequency, "hertz") => Hertz,
            (FactKind::Frequency, "khz") | (FactKind::Frequency, "kilohertz") => Kilohertz,
            (FactKind::Frequency, "mhz") | (FactKind::Frequency, "megahertz") => Megahertz,
            (FactKind::Frequency, "ghz") | (FactKind::Frequency, "gigahertz") => Megahertz, // Map to Megahertz

            (FactKind::Temperature, "°c")
            | (FactKind::Temperature, "degc")
            | (FactKind::Temperature, "c")
            | (FactKind::Temperature, "celsius") => DegreeCelsius,
            (FactKind::Temperature, "k") | (FactKind::Temperature, "kelvin") => Kelvin,

            (FactKind::Dimension, "mm")
            | (FactKind::Dimension, "millimeter")
            | (FactKind::Dimension, "millimetre") => Millimetre,
            (FactKind::Dimension, "cm")
            | (FactKind::Dimension, "centimeter")
            | (FactKind::Dimension, "centimetre") => Millimetre, // Map to Millimetre
            (FactKind::Dimension, "m")
            | (FactKind::Dimension, "meter")
            | (FactKind::Dimension, "metre") => Metre,
            (FactKind::Dimension, "mil") => Mil,
            (FactKind::Dimension, "in")
            | (FactKind::Dimension, "inch")
            | (FactKind::Dimension, "inches") => Millimetre, // Map to Millimetre
            (FactKind::Dimension, "µm")
            | (FactKind::Dimension, "um")
            | (FactKind::Dimension, "micrometer")
            | (FactKind::Dimension, "micrometre") => Millimetre, // Map to Millimetre

            _ => bail!("Unknown unit '{}' for {:?}", unit_str, kind),
        };

        Ok(unit)
    }

    /// Find which page a character offset falls on.
    fn find_page_for_offset(&self, pages: &[PageText], offset: usize) -> usize {
        let mut current = 0;
        for page in pages {
            if offset < current + page.text.len() {
                return page.page_num;
            }
            current += page.text.len() + 1;
        }
        pages.last().map(|p| p.page_num).unwrap_or(1)
    }

    /// Extract context around a match position.
    fn extract_context(&self, text: &str, pos: usize, radius: usize) -> String {
        let start = pos.saturating_sub(radius);
        let end = (pos + radius).min(text.len());
        text[start..end].to_string()
    }

    /// Extract test conditions from surrounding text.
    fn extract_conditions(&self, text: &str, pos: usize) -> Vec<String> {
        let mut conditions = Vec::new();
        let context = self.extract_context(text, pos, 200);
        let lower = context.to_lowercase();

        // Common condition patterns
        let condition_patterns = [
            (
                "temperature",
                r"(?i)(?:at|@|T[aj])\s*=?\s*([\-\d\.]+\s*°?C)",
            ),
            (
                "voltage",
                r"(?i)(?:at|@|V(?:cc|dd|in|out))\s*=?\s*([\d\.]+\s*V)",
            ),
            (
                "current",
                r"(?i)(?:at|@|I(?:out|in))\s*=?\s*([\d\.]+\s*[Aa])",
            ),
            ("frequency", r"(?i)(?:at|@|f)\s*=?\s*([\d\.]+\s*Hz)"),
        ];

        for (name, pattern) in condition_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(&context) {
                    if let Some(m) = caps.get(1) {
                        conditions.push(format!("{}={}", name, m.as_str().trim()));
                    }
                }
            }
        }

        conditions
    }

    /// Extract package information.
    fn extract_package_info(
        &self,
        text: &str,
        asset: &AssetRecord,
        pages: &[PageText],
    ) -> Result<Option<DatasheetFact>> {
        let package_patterns = [
            r"(?i)package\s*[:=]\s*([A-Z0-9\-\._/]{2,})",
            r"(?i)(?:case|outline)\s*[:=]\s*([A-Z0-9\-\._/]{2,})",
            r"\b(SOIC|SO|SOP|TSOP|TSSOP|MSOP|QFN|DFN|QFP|LQFP|TQFP|BGA|CSP|WLCSP|DIP|SOT|TO|DPAK|IPAK|TO-?\d+)\b",
            r"(?i)footprint\s*[:=]\s*([A-Z0-9\-\._/]{2,})",
        ];

        for pattern in &package_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    if let Some(m) = caps.get(1) {
                        let package = m.as_str().trim().to_string();
                        let page = self.find_page_for_offset(pages, m.start());

                        let provenance = ProvenanceInfo {
                            asset_id: asset.id.unwrap_or(0),
                            asset_sha256: asset.sha256.clone().unwrap_or_default(),
                            page,
                            extracted_text: self.extract_context(text, m.start(), 100),
                            parser_version: self.config.version.clone(),
                            extracted_at: Utc::now().to_rfc3339(),
                            verified: false,
                            manually_corrected: false,
                            correction_notes: None,
                        };

                        return Ok(Some(DatasheetFact {
                            id: None,
                            asset_id: asset.id.unwrap_or(0),
                            kind: FactKind::Package,
                            parameter: ParameterFact {
                                value: eak_units::PhysicalQuantity::new(
                                    0.0,
                                    eak_units::Unit::Unitless,
                                ), // Package is string, not numeric
                                tolerance: None,
                                conditions: vec![package.clone()],
                            },
                            provenance,
                            created_at: Utc::now().to_rfc3339(),
                            updated_at: Utc::now().to_rfc3339(),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Extract temperature range.
    fn extract_temperature_range(
        &self,
        text: &str,
        asset: &AssetRecord,
        pages: &[PageText],
    ) -> Result<Option<Vec<DatasheetFact>>> {
        let mut facts = Vec::new();

        // Look for operating temperature range
        let temp_range_pattern = regex::Regex::new(r"(?i)(?:operating|junction|storage|ambient)\s*temperature\s*(?:range)?\s*[:=]\s*([\-\d\.]+)\s*[~\-\–]\s*([\-\d\.]+)\s*(°C|degC|C|K)")
            .map_err(|e| anyhow::anyhow!("Failed to compile regex: {}", e))?;

        for caps in temp_range_pattern.captures_iter(text) {
            if let (Some(min_match), Some(max_match), Some(unit_match)) =
                (caps.get(1), caps.get(2), caps.get(3))
            {
                let min_val = min_match
                    .as_str()
                    .trim()
                    .parse::<f64>()
                    .map_err(|e| anyhow::anyhow!("Failed to parse min temperature: {}", e))?;
                let max_val = max_match
                    .as_str()
                    .trim()
                    .parse::<f64>()
                    .map_err(|e| anyhow::anyhow!("Failed to parse max temperature: {}", e))?;
                let unit = self.parse_unit(unit_match.as_str(), FactKind::Temperature)?;

                let page = self.find_page_for_offset(pages, caps.get(0).unwrap().start());

                let provenance = ProvenanceInfo {
                    asset_id: asset.id.unwrap_or(0),
                    asset_sha256: asset.sha256.clone().unwrap_or_default(),
                    page,
                    extracted_text: self.extract_context(text, caps.get(0).unwrap().start(), 100),
                    parser_version: self.config.version.clone(),
                    extracted_at: Utc::now().to_rfc3339(),
                    verified: false,
                    manually_corrected: false,
                    correction_notes: None,
                };

                // Min temperature
                facts.push(DatasheetFact {
                    id: None,
                    asset_id: asset.id.unwrap_or(0),
                    kind: FactKind::TemperatureMin,
                    parameter: ParameterFact {
                        value: eak_units::PhysicalQuantity::new(min_val, unit),
                        tolerance: None,
                        conditions: vec![],
                    },
                    provenance: provenance.clone(),
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                });

                // Max temperature
                facts.push(DatasheetFact {
                    id: None,
                    asset_id: asset.id.unwrap_or(0),
                    kind: FactKind::TemperatureMax,
                    parameter: ParameterFact {
                        value: eak_units::PhysicalQuantity::new(max_val, unit),
                        tolerance: None,
                        conditions: vec![],
                    },
                    provenance,
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                });
            }
        }

        if facts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(facts))
        }
    }

    /// Extract lifecycle/status information.
    fn extract_lifecycle(
        &self,
        text: &str,
        asset: &AssetRecord,
        pages: &[PageText],
    ) -> Result<Option<DatasheetFact>> {
        let lifecycle_keywords = [
            ("Active", "active"),
            ("Obsolete", "obsolete"),
            ("End of Life", "end.?of.?life|eol"),
            ("NRND", "nrnd|not.?recommended.?for.?new.?design"),
            ("Last Time Buy", "last.?time.?buy|ltb"),
            ("Preliminary", "preliminary"),
            ("Production", "production"),
        ];

        for (status, pattern) in lifecycle_keywords {
            if let Ok(re) = regex::Regex::new(&format!(r"(?i){}", pattern)) {
                if let Some(m) = re.find(text) {
                    let page = self.find_page_for_offset(pages, m.start());

                    let provenance = ProvenanceInfo {
                        asset_id: asset.id.unwrap_or(0),
                        asset_sha256: asset.sha256.clone().unwrap_or_default(),
                        page,
                        extracted_text: self.extract_context(text, m.start(), 100),
                        parser_version: self.config.version.clone(),
                        extracted_at: Utc::now().to_rfc3339(),
                        verified: false,
                        manually_corrected: false,
                        correction_notes: None,
                    };

                    return Ok(Some(DatasheetFact {
                        id: None,
                        asset_id: asset.id.unwrap_or(0),
                        kind: FactKind::LifecycleStatus,
                        parameter: ParameterFact {
                            value: eak_units::PhysicalQuantity::new(0.0, eak_units::Unit::Unitless),
                            tolerance: None,
                            conditions: vec![status.to_string()],
                        },
                        provenance,
                        created_at: Utc::now().to_rfc3339(),
                        updated_at: Utc::now().to_rfc3339(),
                    }));
                }
            }
        }

        Ok(None)
    }
}

impl Default for DatasheetParser {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level function to parse a datasheet asset and update its state.
pub fn parse_datasheet(asset: &mut AssetRecord, db_path: &Path) -> Result<ParseResult> {
    let parser = DatasheetParser::new();
    let result = parser.parse(asset)?;

    // Update asset state to PARSED if successful
    if result.success {
        asset.set_state(AssetState::Parsed);
        asset.parser_version = Some(result.parser_version.clone());

        // Update in database
        let conn = crate::store::init_db(db_path)?;
        crate::store::update_asset(&conn, asset)?;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{AssetRecord, AssetState, AssetType};
    use crate::facts::FactKind;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Minimal valid PDF for testing
    const TEST_PDF_CONTENT: &[u8] = &[
        0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x34, 0x0a, 0x31, 0x20, 0x30, 0x20, 0x6f, 0x62,
        0x6a, 0x0a, 0x3c, 0x3c, 0x20, 0x2f, 0x54, 0x79, 0x70, 0x65, 0x20, 0x2f, 0x43, 0x61, 0x74,
        0x61, 0x6c, 0x6f, 0x67, 0x20, 0x2f, 0x50, 0x61, 0x67, 0x65, 0x73, 0x20, 0x32, 0x20, 0x30,
        0x20, 0x52, 0x20, 0x3e, 0x3e, 0x0a, 0x65, 0x6e, 0x64, 0x6f, 0x62, 0x6a, 0x0a, 0x32, 0x20,
        0x30, 0x20, 0x6f, 0x62, 0x6a, 0x0a, 0x3c, 0x3c, 0x20, 0x2f, 0x54, 0x79, 0x70, 0x65, 0x20,
        0x2f, 0x50, 0x61, 0x67, 0x65, 0x73, 0x20, 0x2f, 0x4b, 0x69, 0x64, 0x73, 0x20, 0x5b, 0x33,
        0x20, 0x30, 0x20, 0x52, 0x5d, 0x20, 0x2f, 0x43, 0x6f, 0x75, 0x6e, 0x74, 0x20, 0x31, 0x20,
        0x3e, 0x3e, 0x0a, 0x65, 0x6e, 0x64, 0x6f, 0x62, 0x6a, 0x0a, 0x33, 0x20, 0x30, 0x20, 0x6f,
        0x62, 0x6a, 0x0a, 0x3c, 0x3c, 0x20, 0x2f, 0x54, 0x79, 0x70, 0x65, 0x20, 0x2f, 0x50, 0x61,
        0x67, 0x65, 0x20, 0x2f, 0x50, 0x61, 0x72, 0x65, 0x6e, 0x74, 0x20, 0x32, 0x20, 0x30, 0x20,
        0x52, 0x20, 0x2f, 0x4d, 0x65, 0x64, 0x69, 0x61, 0x42, 0x6f, 0x78, 0x20, 0x5b, 0x30, 0x20,
        0x30, 0x20, 0x36, 0x31, 0x32, 0x20, 0x37, 0x39, 0x32, 0x5d, 0x20, 0x3e, 0x3e, 0x0a, 0x65,
        0x6e, 0x64, 0x6f, 0x62, 0x6a, 0x0a, 0x78, 0x72, 0x65, 0x66, 0x0a, 0x30, 0x20, 0x34, 0x0a,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x20, 0x36, 0x35,
        0x35, 0x33, 0x35, 0x20, 0x66, 0x20, 0x0a, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x39, 0x20, 0x30, 0x30, 0x30, 0x30, 0x20, 0x6e, 0x20, 0x0a, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x35, 0x38, 0x20, 0x30, 0x30, 0x30, 0x30, 0x20, 0x6e,
        0x20, 0x0a, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x31, 0x31, 0x35, 0x20, 0x30,
        0x30, 0x30, 0x30, 0x20, 0x6e, 0x20, 0x0a, 0x74, 0x72, 0x61, 0x69, 0x6c, 0x65, 0x72, 0x0a,
        0x3c, 0x3c, 0x20, 0x2f, 0x53, 0x69, 0x7a, 0x65, 0x20, 0x34, 0x20, 0x2f, 0x52, 0x6f, 0x6f,
        0x74, 0x20, 0x31, 0x20, 0x30, 0x20, 0x52, 0x20, 0x3e, 0x3e, 0x0a, 0x73, 0x74, 0x61, 0x72,
        0x74, 0x78, 0x72, 0x65, 0x66, 0x0a, 0x32, 0x30, 0x30, 0x0a, 0x25, 0x25, 0x45, 0x4f, 0x46,
        0x0a,
    ];

    fn create_test_asset() -> AssetRecord {
        let mut asset = AssetRecord::new(
            "LM358".to_string(),
            "Texas Instruments".to_string(),
            "Analog IC".to_string(),
            AssetType::Datasheet,
            "https://www.ti.com/lit/ds/symlink/lm358.pdf".to_string(),
            "Texas Instruments".to_string(),
        );
        asset.set_state(AssetState::Verified);
        asset.sha256 = Some("test_hash".to_string());
        asset.local_path = Some(std::path::PathBuf::from("test.pdf"));
        asset.file_size = Some(TEST_PDF_CONTENT.len() as u64);
        asset
    }

    #[test]
    fn test_parser_creation() {
        let parser = DatasheetParser::new();
        assert_eq!(parser.config.version, "1.0.0");
    }

    #[test]
    fn test_parse_value_with_tolerance() {
        let parser = DatasheetParser::new();

        // Simple value
        let (val, tol) = parser.parse_value_with_tolerance("3.3").unwrap();
        assert_eq!(val, 3.3);
        assert!(tol.is_none());

        // Range
        let (val, tol) = parser.parse_value_with_tolerance("3.0 - 3.6").unwrap();
        assert!((val - 3.3).abs() < 0.01);
        assert!(tol.is_some());

        // ± notation
        let (val, tol) = parser.parse_value_with_tolerance("3.3 ± 0.3").unwrap();
        assert_eq!(val, 3.3);
        assert!(tol.is_some());
    }

    #[test]
    fn test_parse_unit() {
        let parser = DatasheetParser::new();

        assert_eq!(
            parser.parse_unit("V", FactKind::Voltage).unwrap(),
            eak_units::Unit::Volt
        );
        assert_eq!(
            parser.parse_unit("mV", FactKind::Voltage).unwrap(),
            eak_units::Unit::Millivolt
        );
        assert_eq!(
            parser.parse_unit("A", FactKind::Current).unwrap(),
            eak_units::Unit::Ampere
        );
        assert_eq!(
            parser.parse_unit("mA", FactKind::Current).unwrap(),
            eak_units::Unit::Milliampere
        );
        assert_eq!(
            parser.parse_unit("Ω", FactKind::Resistance).unwrap(),
            eak_units::Unit::Ohm
        );
        assert_eq!(
            parser.parse_unit("kohm", FactKind::Resistance).unwrap(),
            eak_units::Unit::Kilohm
        );
        assert_eq!(
            parser.parse_unit("F", FactKind::Capacitance).unwrap(),
            eak_units::Unit::Farad
        );
        assert_eq!(
            parser.parse_unit("µF", FactKind::Capacitance).unwrap(),
            eak_units::Unit::Microfarad
        );
        assert_eq!(
            parser.parse_unit("H", FactKind::Inductance).unwrap(),
            eak_units::Unit::Henry
        );
        assert_eq!(
            parser.parse_unit("W", FactKind::Power).unwrap(),
            eak_units::Unit::Watt
        );
        assert_eq!(
            parser.parse_unit("Hz", FactKind::Frequency).unwrap(),
            eak_units::Unit::Hertz
        );
        assert_eq!(
            parser.parse_unit("°C", FactKind::Temperature).unwrap(),
            eak_units::Unit::DegreeCelsius
        );
        assert_eq!(
            parser.parse_unit("mm", FactKind::Dimension).unwrap(),
            eak_units::Unit::Millimetre
        );
    }

    #[test]
    fn test_find_page_for_offset() {
        let parser = DatasheetParser::new();
        let pages = vec![
            PageText {
                page_num: 1,
                text: "Page 1 content".to_string(),
            },
            PageText {
                page_num: 2,
                text: "Page 2 content".to_string(),
            },
        ];

        assert_eq!(parser.find_page_for_offset(&pages, 5), 1);
        assert_eq!(parser.find_page_for_offset(&pages, 20), 2);
    }

    #[test]
    fn test_extract_context() {
        let parser = DatasheetParser::new();
        let text =
            "This is a test sentence with some keywords like voltage = 3.3V and current = 10mA.";
        let pos = text.find("voltage").unwrap();
        let context = parser.extract_context(text, pos, 20);
        assert!(context.contains("voltage"));
        assert!(context.len() <= 40);
    }
}
