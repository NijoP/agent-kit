//! Component Intelligence & Search (Phase 5)
//!
//! Provides deterministic component search, parametric filtering, and readiness-aware ranking
//! over the component library. Uses SQLite FTS5 for full-text search and structured metadata
//! for parametric filtering.

use eak_domain::{ComponentClass, PartLifecycle};
use once_cell::sync::Lazy;
use rusqlite::{Connection, Result, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// Global database connection for search (read-only).
static SEARCH_DB: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the search database connection.
pub fn init_search_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Create FTS5 virtual table for full-text search on parts
    conn.execute(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS parts_fts USING fts5(
            mpn, manufacturer, category, subcategory, package, description, keywords,
            content='parts', content_rowid='rowid'
        )
        "#,
        [],
    )?;

    // Create triggers to keep FTS5 in sync with parts table
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS parts_ai AFTER INSERT ON parts BEGIN
            INSERT INTO parts_fts(rowid, mpn, manufacturer, category, subcategory, package, description, keywords)
            VALUES (new.rowid, new.mpn, new.manufacturer, new.category, new.subcategory, new.package, new.description, new.keywords);
        END
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS parts_ad AFTER DELETE ON parts BEGIN
            INSERT INTO parts_fts(parts_fts, rowid, mpn, manufacturer, category, subcategory, package, description, keywords)
            VALUES ('delete', old.rowid, old.mpn, old.manufacturer, old.category, old.subcategory, old.package, old.description, old.keywords);
        END
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS parts_au AFTER UPDATE ON parts BEGIN
            INSERT INTO parts_fts(parts_fts, rowid, mpn, manufacturer, category, subcategory, package, description, keywords)
            VALUES ('delete', old.rowid, old.mpn, old.manufacturer, old.category, old.subcategory, old.package, old.description, old.keywords);
            INSERT INTO parts_fts(rowid, mpn, manufacturer, category, subcategory, package, description, keywords)
            VALUES (new.rowid, new.mpn, new.manufacturer, new.category, new.subcategory, new.package, new.description, new.keywords);
        END
        "#,
        [],
    )?;

    // Populate FTS5 if empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM parts_fts", [], |r| r.get(0))?;
    if count == 0 {
        conn.execute(
            r#"
            INSERT INTO parts_fts(rowid, mpn, manufacturer, category, subcategory, package, description, keywords)
            SELECT rowid, mpn, manufacturer, category, subcategory, package, description, keywords FROM parts
            "#,
            [],
        )?;
    }

    Ok(conn)
}

/// Set the global search database connection.
pub fn set_search_db(conn: Connection) {
    *SEARCH_DB.lock().unwrap() = Some(conn);
}

/// Get the global search database connection.
pub fn get_search_db(db_path: &Path) -> Result<Connection> {
    init_search_db(db_path)
}

/// Search query parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Full-text search query (matches mpn, manufacturer, category, description, keywords)
    pub query: Option<String>,
    /// Filter by exact MPN
    pub mpn: Option<String>,
    /// Filter by manufacturer (case-insensitive)
    pub manufacturer: Option<String>,
    /// Filter by category (Resistor, Capacitor, Inductor, etc.)
    pub category: Option<String>,
    /// Filter by subcategory
    pub subcategory: Option<String>,
    /// Filter by package (e.g., "0402", "QFN-32")
    pub package: Option<String>,
    /// Filter by component class
    pub component_class: Option<ComponentClass>,
    /// Filter by lifecycle status
    pub lifecycle: Option<PartLifecycle>,
    /// Filter by voltage range (volts)
    pub voltage_min: Option<f64>,
    pub voltage_max: Option<f64>,
    /// Filter by current range (amperes)
    pub current_min: Option<f64>,
    pub current_max: Option<f64>,
    /// Filter by resistance range (ohms)
    pub resistance_min: Option<f64>,
    pub resistance_max: Option<f64>,
    /// Filter by capacitance range (farads)
    pub capacitance_min: Option<f64>,
    pub capacitance_max: Option<f64>,
    /// Filter by inductance range (henries)
    pub inductance_min: Option<f64>,
    pub inductance_max: Option<f64>,
    /// Filter by power range (watts)
    pub power_min: Option<f64>,
    pub power_max: Option<f64>,
    /// Filter by frequency range (hertz)
    pub frequency_min: Option<f64>,
    pub frequency_max: Option<f64>,
    /// Filter by temperature range (celsius)
    pub temperature_min: Option<f64>,
    pub temperature_max: Option<f64>,
    /// Filter by package
    pub package_exact: Option<String>,
    /// Limit results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Sort by field
    pub sort_by: Option<SortField>,
    /// Sort direction
    pub sort_desc: Option<bool>,
}

/// Sort fields for search results.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortField {
    Mpn,
    Manufacturer,
    Category,
    Package,
    Lifecycle,
    ReadinessScore,
    MetadataCompleteness,
}

/// Search result with intelligence metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentIntelligenceResult {
    /// Manufacturer part number
    pub mpn: String,
    /// Manufacturer name
    pub manufacturer: String,
    /// Component category
    pub category: String,
    /// Component subcategory
    pub subcategory: Option<String>,
    /// Package code
    pub package: Option<String>,
    /// Component class
    pub component_class: ComponentClass,
    /// Lifecycle status
    pub lifecycle: PartLifecycle,
    /// Description
    pub description: Option<String>,
    /// Keywords
    pub keywords: Option<String>,
    /// Physical quantities from metadata
    pub physical_quantities: HashMap<String, PhysicalQuantityValue>,
    /// Readiness score (0.0 - 1.0)
    pub readiness_score: f64,
    /// Metadata completeness (0.0 - 1.0)
    pub metadata_completeness: f64,
    /// Provenance information
    pub provenance: ProvenanceInfo,
    /// Ranking score (higher is better)
    pub ranking_score: f64,
    /// Reason for ranking
    pub ranking_reason: String,
    /// Matched fields (for highlighting)
    pub matched_fields: Vec<String>,
}

/// Physical quantity value for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalQuantityValue {
    pub magnitude: f64,
    pub unit: String,
    pub dimension: String,
    pub tolerance: Option<String>,
}

/// Provenance information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceInfo {
    pub source_url: Option<String>,
    pub datasheet_url: Option<String>,
    pub verified: bool,
    pub acquisition_method: Option<String>,
}

/// Search results container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub results: Vec<ComponentIntelligenceResult>,
    pub total_count: usize,
    pub query: SearchQuery,
}

/// Readiness scoring weights.
const READINESS_WEIGHTS: ReadinessWeights = ReadinessWeights {
    has_mpn: 0.15,
    has_manufacturer: 0.15,
    has_category: 0.10,
    has_package: 0.10,
    has_physical_quantities: 0.20,
    has_description: 0.05,
    has_keywords: 0.05,
    lifecycle_active: 0.10,
    has_datasheet_url: 0.10,
};

/// Readiness scoring weights configuration.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct ReadinessWeights {
    has_mpn: f64,
    has_manufacturer: f64,
    has_category: f64,
    has_package: f64,
    has_physical_quantities: f64,
    has_description: f64,
    has_keywords: f64,
    lifecycle_active: f64,
    has_datasheet_url: f64,
}

/// Ranking weights for search results.
const RANKING_WEIGHTS: RankingWeights = RankingWeights {
    text_match: 0.30,
    readiness_score: 0.25,
    metadata_completeness: 0.20,
    lifecycle_bonus: 0.15,
    exact_mpn_match: 0.15,
};

/// Ranking weights configuration.
#[derive(Debug, Clone, Copy)]
struct RankingWeights {
    text_match: f64,
    readiness_score: f64,
    metadata_completeness: f64,
    lifecycle_bonus: f64,
    exact_mpn_match: f64,
}

/// Execute a search query.
pub fn search(db_path: &Path, query: SearchQuery) -> Result<SearchResults> {
    let conn = init_search_db(db_path)?;

    let mut sql = String::from(
        r#"
        SELECT p.rowid, p.mpn, p.manufacturer, p.category, p.subcategory, p.package,
               p.voltage_v, p.capacitance_f, p.resistance_ohm, p.inductance_h,
               p.tolerance, p.temp_coeff, p.power_w, p.stock, p.price_cny, p.moq,
               p.description, p.keywords, p.metadata_json, p.lifecycle
        FROM parts p
        WHERE 1=1
        "#,
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // Full-text search
    if let Some(q) = &query.query {
        if !q.trim().is_empty() {
            sql.push_str(" AND p.rowid IN (SELECT rowid FROM parts_fts WHERE parts_fts MATCH ?)");
            params_vec.push(Box::new(q.clone()));
        }
    }

    // Exact MPN filter
    if let Some(mpn) = &query.mpn {
        sql.push_str(" AND p.mpn = ?");
        params_vec.push(Box::new(mpn.clone()));
    }

    // Manufacturer filter (case-insensitive)
    if let Some(mfr) = &query.manufacturer {
        sql.push_str(" AND LOWER(p.manufacturer) = LOWER(?)");
        params_vec.push(Box::new(mfr.clone()));
    }

    // Category filter
    if let Some(cat) = &query.category {
        sql.push_str(" AND p.category = ?");
        params_vec.push(Box::new(cat.clone()));
    }

    // Subcategory filter
    if let Some(sub) = &query.subcategory {
        sql.push_str(" AND p.subcategory = ?");
        params_vec.push(Box::new(sub.clone()));
    }

    // Package filter
    if let Some(pkg) = &query.package {
        sql.push_str(" AND p.package = ?");
        params_vec.push(Box::new(pkg.clone()));
    }

    // Component class filter (maps to category)
    if let Some(cls) = &query.component_class {
        let cat = component_class_to_category(*cls);
        sql.push_str(" AND p.category = ?");
        params_vec.push(Box::new(cat));
    }

    // Lifecycle filter
    if let Some(lc) = &query.lifecycle {
        sql.push_str(" AND p.lifecycle = ?");
        let lc_str = match lc {
            PartLifecycle::Active => "Active",
            PartLifecycle::Nrnd => "NRND",
            PartLifecycle::Eol => "EOL",
        };
        params_vec.push(Box::new(lc_str.to_string()));
    }

    // Voltage range
    if let Some(v) = query.voltage_min {
        sql.push_str(" AND p.voltage_v >= ?");
        params_vec.push(Box::new(v));
    }
    if let Some(v) = query.voltage_max {
        sql.push_str(" AND p.voltage_v <= ?");
        params_vec.push(Box::new(v));
    }

    // Current range (not directly in schema, would need metadata_json)
    // For now, skip - would require JSON extraction

    // Resistance range
    if let Some(v) = query.resistance_min {
        sql.push_str(" AND p.resistance_ohm >= ?");
        params_vec.push(Box::new(v));
    }
    if let Some(v) = query.resistance_max {
        sql.push_str(" AND p.resistance_ohm <= ?");
        params_vec.push(Box::new(v));
    }

    // Capacitance range
    if let Some(v) = query.capacitance_min {
        sql.push_str(" AND p.capacitance_f >= ?");
        params_vec.push(Box::new(v));
    }
    if let Some(v) = query.capacitance_max {
        sql.push_str(" AND p.capacitance_f <= ?");
        params_vec.push(Box::new(v));
    }

    // Inductance range
    if let Some(v) = query.inductance_min {
        sql.push_str(" AND p.inductance_h >= ?");
        params_vec.push(Box::new(v));
    }
    if let Some(v) = query.inductance_max {
        sql.push_str(" AND p.inductance_h <= ?");
        params_vec.push(Box::new(v));
    }

    // Power range
    if let Some(v) = query.power_min {
        sql.push_str(" AND p.power_w >= ?");
        params_vec.push(Box::new(v));
    }
    if let Some(v) = query.power_max {
        sql.push_str(" AND p.power_w <= ?");
        params_vec.push(Box::new(v));
    }

    // Package exact match
    if let Some(pkg) = &query.package_exact {
        sql.push_str(" AND p.package = ?");
        params_vec.push(Box::new(pkg.clone()));
    }

    // Sorting
    let sort_field = query.sort_by.unwrap_or(SortField::ReadinessScore);
    let sort_desc = query.sort_desc.unwrap_or(true);
    let sort_column = match sort_field {
        SortField::Mpn => "mpn",
        SortField::Manufacturer => "manufacturer",
        SortField::Category => "category",
        SortField::Package => "package",
        SortField::Lifecycle => "lifecycle",
        SortField::ReadinessScore => "readiness_score", // Will need computed column
        SortField::MetadataCompleteness => "metadata_completeness", // Will need computed column
    };
    sql.push_str(&format!(
        " ORDER BY {} {}",
        sort_column,
        if sort_desc { "DESC" } else { "ASC" }
    ));

    // Limit and offset
    if let Some(limit) = query.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    if let Some(offset) = query.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }

    // Execute query
    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let rows = stmt.query_map(rusqlite::params_from_iter(param_refs.iter()), row_to_result)?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    // Get total count (without limit/offset)
    let count_sql = sql
        .replace(
            &format!(
                " ORDER BY {} {}",
                sort_column,
                if sort_desc { "DESC" } else { "ASC" }
            ),
            "",
        )
        .replace(
            "SELECT p.rowid, p.mpn, p.manufacturer, p.category, p.subcategory, p.package,
               p.voltage_v, p.capacitance_f, p.resistance_ohm, p.inductance_h,
               p.tolerance, p.temp_coeff, p.power_w, p.stock, p.price_cny, p.moq,
               p.description, p.keywords, p.metadata_json, p.lifecycle",
            "SELECT COUNT(*)",
        );
    let total_count: usize = conn.query_row(
        &count_sql,
        rusqlite::params_from_iter(param_refs.iter()),
        |r| r.get(0),
    )?;

    // Compute readiness and ranking scores
    for result in &mut results {
        result.readiness_score = compute_readiness_score(result);
        result.metadata_completeness = compute_metadata_completeness(result);
        result.ranking_score = compute_ranking_score(result, &query);
        result.ranking_reason = generate_ranking_reason(result, &query);
    }

    // Re-sort by ranking score if not sorting by a specific field
    if query.sort_by.is_none() {
        results.sort_by(|a, b| {
            b.ranking_score
                .partial_cmp(&a.ranking_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    Ok(SearchResults {
        results,
        total_count,
        query,
    })
}

/// Convert ComponentClass to category string.
fn component_class_to_category(class: ComponentClass) -> String {
    match class {
        ComponentClass::Resistor => "Resistor".to_string(),
        ComponentClass::Capacitor => "Capacitor".to_string(),
        ComponentClass::Ic => "IC".to_string(), // Would need mapping
        ComponentClass::Regulator => "Regulator".to_string(),
        ComponentClass::Connector => "Connector".to_string(),
    }
}

/// Convert database row to search result.
fn row_to_result(row: &Row) -> Result<ComponentIntelligenceResult> {
    let mpn: String = row.get("mpn")?;
    let manufacturer: String = row.get("manufacturer")?;
    let category: String = row.get("category")?;
    let subcategory: Option<String> = row.get("subcategory")?;
    let package: Option<String> = row.get("package")?;
    let voltage_v: Option<f64> = row.get("voltage_v")?;
    let capacitance_f: Option<f64> = row.get("capacitance_f")?;
    let resistance_ohm: Option<f64> = row.get("resistance_ohm")?;
    let inductance_h: Option<f64> = row.get("inductance_h")?;
    let tolerance: Option<String> = row.get("tolerance")?;
    let _temp_coeff: Option<String> = row.get("temp_coeff")?;
    let power_w: Option<f64> = row.get("power_w")?;
    let description: Option<String> = row.get("description")?;
    let keywords: Option<String> = row.get("keywords")?;
    let metadata_json: Option<String> = row.get("metadata_json")?;
    let lifecycle_str: String = row.get("lifecycle")?;

    let lifecycle = match lifecycle_str.as_str() {
        "Active" => PartLifecycle::Active,
        "NRND" => PartLifecycle::Nrnd,
        "EOL" => PartLifecycle::Eol,
        _ => PartLifecycle::Active,
    };

    // Parse metadata_json for physical quantities
    let mut physical_quantities = HashMap::new();
    if let Some(json_str) = metadata_json {
        if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&json_str) {
            extract_physical_quantities(&metadata, &mut physical_quantities);
        }
    }

    // Also extract from legacy columns
    if let Some(v) = voltage_v {
        physical_quantities.insert(
            "voltage".to_string(),
            PhysicalQuantityValue {
                magnitude: v,
                unit: "V".to_string(),
                dimension: "Voltage".to_string(),
                tolerance: None,
            },
        );
    }
    if let Some(v) = capacitance_f {
        physical_quantities.insert(
            "capacitance".to_string(),
            PhysicalQuantityValue {
                magnitude: v,
                unit: "F".to_string(),
                dimension: "Capacitance".to_string(),
                tolerance: None,
            },
        );
    }
    if let Some(v) = resistance_ohm {
        physical_quantities.insert(
            "resistance".to_string(),
            PhysicalQuantityValue {
                magnitude: v,
                unit: "Ω".to_string(),
                dimension: "Resistance".to_string(),
                tolerance: tolerance.clone(),
            },
        );
    }
    if let Some(v) = inductance_h {
        physical_quantities.insert(
            "inductance".to_string(),
            PhysicalQuantityValue {
                magnitude: v,
                unit: "H".to_string(),
                dimension: "Inductance".to_string(),
                tolerance: None,
            },
        );
    }
    if let Some(v) = power_w {
        physical_quantities.insert(
            "power".to_string(),
            PhysicalQuantityValue {
                magnitude: v,
                unit: "W".to_string(),
                dimension: "Power".to_string(),
                tolerance: None,
            },
        );
    }

    // Determine component class from category
    let component_class = match category.as_str() {
        "Resistor" => ComponentClass::Resistor,
        "Capacitor" => ComponentClass::Capacitor,
        "Inductor" => ComponentClass::Ic, // Would need better mapping
        "Regulator" => ComponentClass::Regulator,
        "Connector" => ComponentClass::Connector,
        _ => ComponentClass::Ic,
    };

    Ok(ComponentIntelligenceResult {
        mpn,
        manufacturer,
        category,
        subcategory,
        package,
        component_class,
        lifecycle,
        description,
        keywords,
        physical_quantities,
        readiness_score: 0.0,       // Will be computed
        metadata_completeness: 0.0, // Will be computed
        provenance: ProvenanceInfo {
            source_url: None,
            datasheet_url: None,
            verified: false,
            acquisition_method: None,
        },
        ranking_score: 0.0,            // Will be computed
        ranking_reason: String::new(), // Will be computed
        matched_fields: Vec::new(),
    })
}

/// Extract physical quantities from metadata JSON.
fn extract_physical_quantities(
    metadata: &serde_json::Value,
    quantities: &mut HashMap<String, PhysicalQuantityValue>,
) {
    if let Some(core) = metadata.get("core") {
        if let Some(_obj) = core.as_object() {
            for (key, value) in _obj {
                if let Some(_obj) = value.as_object() {
                    if let (Some(mag), Some(unit), Some(dim)) = (
                        value.get("magnitude").and_then(|v| v.as_f64()),
                        value.get("unit").and_then(|v| v.as_str()),
                        value.get("dimension").and_then(|v| v.as_str()),
                    ) {
                        quantities.insert(
                            key.clone(),
                            PhysicalQuantityValue {
                                magnitude: mag,
                                unit: unit.to_string(),
                                dimension: dim.to_string(),
                                tolerance: value
                                    .get("tolerance")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                            },
                        );
                    }
                }
            }
        }
    }

    if let Some(family) = metadata.get("family") {
        if let Some(data) = family.get("data") {
            if let Some(_obj) = data.as_object() {
                for (key, value) in _obj {
                    if let Some(_obj) = value.as_object() {
                        if let (Some(mag), Some(unit), Some(dim)) = (
                            value.get("magnitude").and_then(|v| v.as_f64()),
                            value.get("unit").and_then(|v| v.as_str()),
                            value.get("dimension").and_then(|v| v.as_str()),
                        ) {
                            quantities.insert(
                                key.clone(),
                                PhysicalQuantityValue {
                                    magnitude: mag,
                                    unit: unit.to_string(),
                                    dimension: dim.to_string(),
                                    tolerance: value
                                        .get("tolerance")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Compute readiness score (0.0 - 1.0).
pub fn compute_readiness_score(result: &ComponentIntelligenceResult) -> f64 {
    let mut score = 0.0;

    if !result.mpn.is_empty() {
        score += READINESS_WEIGHTS.has_mpn;
    }
    if !result.manufacturer.is_empty() {
        score += READINESS_WEIGHTS.has_manufacturer;
    }
    if !result.category.is_empty() {
        score += READINESS_WEIGHTS.has_category;
    }
    if result.package.is_some() {
        score += READINESS_WEIGHTS.has_package;
    }
    if !result.physical_quantities.is_empty() {
        score += READINESS_WEIGHTS.has_physical_quantities;
    }
    if result.description.is_some() && !result.description.as_ref().unwrap().is_empty() {
        score += READINESS_WEIGHTS.has_description;
    }
    if result.keywords.is_some() && !result.keywords.as_ref().unwrap().is_empty() {
        score += READINESS_WEIGHTS.has_keywords;
    }
    if matches!(result.lifecycle, PartLifecycle::Active) {
        score += READINESS_WEIGHTS.lifecycle_active;
    }
    // has_datasheet_url would need provenance

    score.min(1.0)
}

/// Compute metadata completeness (0.0 - 1.0).
pub fn compute_metadata_completeness(result: &ComponentIntelligenceResult) -> f64 {
    let mut fields = 0;
    let mut total = 0;

    // Core fields
    total += 1;
    if !result.mpn.is_empty() {
        fields += 1;
    }
    total += 1;
    if !result.manufacturer.is_empty() {
        fields += 1;
    }
    total += 1;
    if !result.category.is_empty() {
        fields += 1;
    }
    total += 1;
    if result.package.is_some() {
        fields += 1;
    }
    total += 1;
    if result.description.is_some() && !result.description.as_ref().unwrap().is_empty() {
        fields += 1;
    }
    total += 1;
    if result.keywords.is_some() && !result.keywords.as_ref().unwrap().is_empty() {
        fields += 1;
    }
    total += 1;
    if !result.physical_quantities.is_empty() {
        fields += 1;
    }
    total += 1;
    if result.subcategory.is_some() {
        fields += 1;
    }

    if total > 0 {
        fields as f64 / total as f64
    } else {
        0.0
    }
}

/// Compute ranking score.
pub fn compute_ranking_score(result: &ComponentIntelligenceResult, query: &SearchQuery) -> f64 {
    let mut score = 0.0;

    // Text match score (simplified - would use FTS5 rank in practice)
    let text_match = if query.query.is_some() { 0.5 } else { 0.0 };
    score += text_match * RANKING_WEIGHTS.text_match;

    // Readiness score
    score += result.readiness_score * RANKING_WEIGHTS.readiness_score;

    // Metadata completeness
    score += result.metadata_completeness * RANKING_WEIGHTS.metadata_completeness;

    // Lifecycle bonus
    if matches!(result.lifecycle, PartLifecycle::Active) {
        score += RANKING_WEIGHTS.lifecycle_bonus;
    }

    // Exact MPN match bonus
    if let Some(mpn) = &query.mpn {
        if mpn.eq_ignore_ascii_case(&result.mpn) {
            score += RANKING_WEIGHTS.exact_mpn_match;
        }
    }

    score
}

/// Generate human-readable ranking reason.
fn generate_ranking_reason(result: &ComponentIntelligenceResult, query: &SearchQuery) -> String {
    let mut reasons: Vec<String> = Vec::new();

    if matches!(result.lifecycle, PartLifecycle::Active) {
        reasons.push("active lifecycle".to_string());
    }
    if !result.physical_quantities.is_empty() {
        reasons.push(format!(
            "{} physical quantities",
            result.physical_quantities.len()
        ));
    }
    if result.metadata_completeness > 0.7 {
        reasons.push("high metadata completeness".to_string());
    }
    if query
        .mpn
        .as_ref()
        .map(|m| m.eq_ignore_ascii_case(&result.mpn))
        .unwrap_or(false)
    {
        reasons.push("exact MPN match".to_string());
    }
    if query.query.is_some() {
        reasons.push("text match".to_string());
    }

    if reasons.is_empty() {
        "default ranking".to_string()
    } else {
        reasons.join(", ")
    }
}

/// Search by exact MPN.
pub fn search_by_mpn(db_path: &Path, mpn: &str) -> Result<Option<ComponentIntelligenceResult>> {
    let query = SearchQuery {
        mpn: Some(mpn.to_string()),
        limit: Some(1),
        ..Default::default()
    };
    let results = search(db_path, query)?;
    Ok(results.results.into_iter().next())
}

/// Search by manufacturer.
pub fn search_by_manufacturer(
    db_path: &Path,
    manufacturer: &str,
    limit: usize,
) -> Result<SearchResults> {
    let query = SearchQuery {
        manufacturer: Some(manufacturer.to_string()),
        limit: Some(limit),
        ..Default::default()
    };
    search(db_path, query)
}

/// Search by category.
pub fn search_by_category(db_path: &Path, category: &str, limit: usize) -> Result<SearchResults> {
    let query = SearchQuery {
        category: Some(category.to_string()),
        limit: Some(limit),
        ..Default::default()
    };
    search(db_path, query)
}

/// Search by parameter range.
pub fn search_by_parameter(
    db_path: &Path,
    param: &str,
    min: Option<f64>,
    max: Option<f64>,
    limit: usize,
) -> Result<SearchResults> {
    let mut query = SearchQuery {
        limit: Some(limit),
        ..Default::default()
    };

    match param {
        "voltage" => {
            query.voltage_min = min;
            query.voltage_max = max;
        }
        "resistance" => {
            query.resistance_min = min;
            query.resistance_max = max;
        }
        "capacitance" => {
            query.capacitance_min = min;
            query.capacitance_max = max;
        }
        "inductance" => {
            query.inductance_min = min;
            query.inductance_max = max;
        }
        "power" => {
            query.power_min = min;
            query.power_max = max;
        }
        _ => {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "Unknown parameter: {}",
                param
            ))
            ) )
        }
    }

    search(db_path, query)
}

/// Get component by MPN with full intelligence.
pub fn get_component_intelligence(
    db_path: &Path,
    mpn: &str,
    manufacturer: &str,
) -> Result<Option<ComponentIntelligenceResult>> {
    let query = SearchQuery {
        mpn: Some(mpn.to_string()),
        manufacturer: Some(manufacturer.to_string()),
        limit: Some(1),
        ..Default::default()
    };
    let results = search(db_path, query)?;
    Ok(results.results.into_iter().next())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn setup_test_db() -> (NamedTempFile, PathBuf) {
        let temp_db = NamedTempFile::new().unwrap();
        let db_path = temp_db.path().to_path_buf();

        let conn = init_search_db(&db_path).unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO parts (mpn, manufacturer, category, subcategory, package, voltage_v, capacitance_f, resistance_ohm, inductance_h, tolerance, temp_coeff, power_w, description, keywords, lifecycle) VALUES
            ('RC0402FR-0710KL', 'Yageo', 'Resistor', 'Thin Film', '0402', NULL, NULL, 10000.0, NULL, '±1%', '±100ppm/°C', 0.0625, '0402 10kΩ 1% Resistor', 'resistor 0402 thin-film', 'Active'),
            ('CL05A104KA5NNNC', 'Samsung', 'Capacitor', 'MLCC', '0402', 16.0, 0.0000001, NULL, NULL, '±10%', NULL, NULL, '0402 100nF 16V X7R', 'capacitor 0402 mlcc', 'Active'),
            ('74404064100', 'Würth Elektronik', 'Inductor', 'Power', '0805', NULL, NULL, NULL, 0.00001, '±20%', NULL, 0.5, '0805 10µH Power Inductor', 'inductor 0805 power', 'Active'),
            ('LM1117-3.3', 'Texas Instruments', 'Regulator', 'LDO', 'SOT-223', 3.3, NULL, NULL, NULL, NULL, NULL, 0.8, '3.3V LDO Regulator', 'regulator ldo 3.3v', 'EOL'),
            ('STM32L010F4P6', 'STMicroelectronics', 'IC', 'MCU', 'TSSOP-20', 3.3, NULL, NULL, NULL, NULL, NULL, 0.05, 'STM32L0 Ultra-low-power MCU', 'mcu stm32 ultra-low-power', 'Active')",
            [],
        ).unwrap();

        (temp_db, db_path)
    }

    #[test]
    fn test_search_by_mpn() {
        let (_temp, db_path) = setup_test_db();
        let result = search_by_mpn(&db_path, "RC0402FR-0710KL").unwrap();
        assert!(result.is_some());
        let comp = result.unwrap();
        assert_eq!(comp.mpn, "RC0402FR-0710KL");
        assert_eq!(comp.manufacturer, "Yageo");
        assert_eq!(comp.category, "Resistor");
    }

    #[test]
    fn test_search_by_manufacturer() {
        let (_temp, db_path) = setup_test_db();
        let results = search_by_manufacturer(&db_path, "Yageo", 10).unwrap();
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].manufacturer, "Yageo");
    }

    #[test]
    fn test_search_by_category() {
        let (_temp, db_path) = setup_test_db();
        let results = search_by_category(&db_path, "Resistor", 10).unwrap();
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].category, "Resistor");
    }

    #[test]
    fn test_search_by_parameter() {
        let (_temp, db_path) = setup_test_db();
        let results =
            search_by_parameter(&db_path, "resistance", Some(5000.0), Some(15000.0), 10).unwrap();
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].mpn, "RC0402FR-0710KL");
    }

    #[test]
    fn test_search_by_voltage() {
        let (_temp, db_path) = setup_test_db();
        let results = search_by_parameter(&db_path, "voltage", Some(10.0), Some(20.0), 10).unwrap();
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].mpn, "CL05A104KA5NNNC");
    }

    #[test]
    fn test_full_text_search() {
        let (_temp, db_path) = setup_test_db();
        let query = SearchQuery {
            query: Some("resistor".to_string()),
            limit: Some(10),
            ..Default::default()
        };
        let results = search(&db_path, query).unwrap();
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].category, "Resistor");
    }

    #[test]
    fn test_readiness_score() {
        let (_temp, db_path) = setup_test_db();
        let result = search_by_mpn(&db_path, "RC0402FR-0710KL").unwrap().unwrap();
        assert!(result.readiness_score > 0.5);
        assert!(result.readiness_score <= 1.0);
    }

    #[test]
    fn test_metadata_completeness() {
        let (_temp, db_path) = setup_test_db();
        let result = search_by_mpn(&db_path, "RC0402FR-0710KL").unwrap().unwrap();
        assert!(result.metadata_completeness > 0.5);
        assert!(result.metadata_completeness <= 1.0);
    }

    #[test]
    fn test_ranking_score() {
        let (_temp, db_path) = setup_test_db();
        let query = SearchQuery {
            query: Some("resistor".to_string()),
            limit: Some(10),
            ..Default::default()
        };
        let results = search(&db_path, query).unwrap();
        assert!(!results.results.is_empty());
        for r in &results.results {
            assert!(r.ranking_score >= 0.0);
        }
    }

    #[test]
    fn test_exact_mpn_match_ranking() {
        let (_temp, db_path) = setup_test_db();
        let query = SearchQuery {
            mpn: Some("RC0402FR-0710KL".to_string()),
            limit: Some(10),
            ..Default::default()
        };
        let results = search(&db_path, query).unwrap();
        assert_eq!(results.results.len(), 1);
        assert!(results.results[0].ranking_score > 0.5); // Should have exact match bonus
    }

    #[test]
    fn test_lifecycle_filter() {
        let (_temp, db_path) = setup_test_db();
        let query = SearchQuery {
            lifecycle: Some(PartLifecycle::Active),
            limit: Some(10),
            ..Default::default()
        };
        let results = search(&db_path, query).unwrap();
        for r in &results.results {
            assert_eq!(r.lifecycle, PartLifecycle::Active);
        }
    }

    #[test]
    fn test_empty_result() {
        let (_temp, db_path) = setup_test_db();
        let query = SearchQuery {
            mpn: Some("NONEXISTENT".to_string()),
            limit: Some(10),
            ..Default::default()
        };
        let results = search(&db_path, query).unwrap();
        assert_eq!(results.results.len(), 0);
    }

    #[test]
    fn test_deterministic_ordering() {
        let (_temp, db_path) = setup_test_db();
        let query = SearchQuery {
            category: Some("Resistor".to_string()),
            limit: Some(10),
            ..Default::default()
        };
        let results1 = search(&db_path, query.clone()).unwrap();
        let results2 = search(&db_path, query).unwrap();
        assert_eq!(results1.results.len(), results2.results.len());
        for (r1, r2) in results1.results.iter().zip(results2.results.iter()) {
            assert_eq!(r1.mpn, r2.mpn);
        }
    }
}
