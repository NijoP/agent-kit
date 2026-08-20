//! lib-import — CSV to SQLite importer for EAK component library

use clap::Parser;
use csv::ReaderBuilder;
use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lib-import", about = "Import component CSV into EAK library database")]
struct Args {
    /// Path to CSV file
    #[arg(long)]
    csv: PathBuf,

    /// Path to SQLite database
    #[arg(long)]
    db: PathBuf,
}

#[derive(Debug, serde::Deserialize)]
struct PartRow {
    lcsc_id: Option<String>,
    mpn: String,
    manufacturer: String,
    category: String,
    subcategory: Option<String>,
    package: Option<String>,
    voltage_v: Option<f64>,
    capacitance_f: Option<f64>,
    resistance_ohm: Option<f64>,
    inductance_h: Option<f64>,
    tolerance: Option<String>,
    temp_coeff: Option<String>,
    power_w: Option<f64>,
    stock: Option<i64>,
    price_cny: Option<f64>,
    moq: Option<i64>,
    description: Option<String>,
    keywords: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut conn = Connection::open(&args.db)?;
    let tx = conn.transaction()?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(&args.csv)?;

    let mut inserted = 0;
    let mut skipped = 0;

    for result in rdr.deserialize() {
        let row: PartRow = result?;
        
        // Normalize empty strings to NULL
        let lcsc_id = row.lcsc_id.filter(|s| !s.trim().is_empty());
        let subcategory = row.subcategory.filter(|s| !s.trim().is_empty());
        let package = row.package.filter(|s| !s.trim().is_empty());
        let tolerance = row.tolerance.filter(|s| !s.trim().is_empty());
        let temp_coeff = row.temp_coeff.filter(|s| !s.trim().is_empty());
        let description = row.description.filter(|s| !s.trim().is_empty());
        let keywords = row.keywords.filter(|s| !s.trim().is_empty());

        // Check if already exists (idempotent)
        let exists: bool = tx.query_row(
            "SELECT 1 FROM parts WHERE manufacturer = ?1 AND mpn = ?2",
            params![row.manufacturer, row.mpn],
            |_| Ok(true),
        ).unwrap_or(false);

        if exists {
            skipped += 1;
            continue;
        }

        tx.execute(
            "INSERT INTO parts (lcsc_id, mpn, manufacturer, category, subcategory, package, voltage_v, capacitance_f, resistance_ohm, inductance_h, tolerance, temp_coeff, power_w, stock, price_cny, moq, description, keywords)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                lcsc_id,
                row.mpn,
                row.manufacturer,
                row.category,
                subcategory,
                package,
                row.voltage_v,
                row.capacitance_f,
                row.resistance_ohm,
                row.inductance_h,
                tolerance,
                temp_coeff,
                row.power_w,
                row.stock,
                row.price_cny,
                row.moq,
                description,
                keywords,
            ],
        )?;

        inserted += 1;
    }

    tx.commit()?;
    println!("Import complete: {} inserted, {} skipped (duplicates)", inserted, skipped);
    Ok(())
}