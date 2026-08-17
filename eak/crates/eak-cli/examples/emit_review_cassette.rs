//! Emit a verification cassette: run the real **import → verify-only review** over a `.kicad_pcb`
//! and stream every committed `EventRecord` as JSONL — the honest, kernel-produced source for the
//! TUSK Review-dock demo fixture (`app/ui/public/fixtures/review.jsonl`).
//!
//! The board is fed through the same capability seam a generated design uses (no back door, P3):
//! `import_design` → seed the default fabrication floor → drive `verify_only_workflow` through the
//! ordinary `Orchestrator`. A `.kicad_pcb` carries copper but no process class, so seeding the
//! default floor is what gives `drc-trace-width` / `drc-copper-clearance` a bound to evaluate. The
//! returned `RuntimeCore`'s in-memory log is then serialized verbatim, one `EventRecord` per line —
//! the exact bytes a live `EventSink` would forward to the UI (so the fold in the browser stays
//! parity with the kernel).
//!
//! ```text
//! cargo run -p eak-cli --example emit_review_cassette -- \
//!     eak/crates/eak-cli/fixtures/review_board.kicad_pcb \
//!     app/ui/public/fixtures/review.jsonl
//! ```

use std::fs;
use std::io::Write;
use std::path::Path;

use eak_cli::{import_design, seed_default_fabrication_floor, verify_only_workflow};
use eak_runtime::Orchestrator;

fn main() {
    let mut args = std::env::args().skip(1);
    let (src, out) = match (args.next(), args.next()) {
        (Some(s), Some(o)) => (s, o),
        _ => {
            eprintln!("usage: emit_review_cassette <board.kicad_pcb> <events.jsonl>");
            std::process::exit(2);
        }
    };

    let board = fs::read_to_string(&src).expect("read board");
    let mut core = import_design(&board).expect("import through the real seam");
    seed_default_fabrication_floor(&mut core).expect("seed default fabrication floor");
    let mut plan = verify_only_workflow();
    let outcomes = Orchestrator::new().run(&mut plan, &mut core);

    for (name, outcome) in &outcomes {
        eprintln!("phase {name}: {outcome:?}");
    }

    let records = core.log().read_all().expect("read committed log");
    let mut file = fs::File::create(Path::new(&out)).expect("open output");
    for record in &records {
        writeln!(file, "{}", serde_json::to_string(record).expect("serialize event")).expect("write");
    }
    eprintln!("wrote {} events -> {out}", records.len());
}