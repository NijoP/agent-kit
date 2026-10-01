//! `eak` — composition root + CLI driver (the Frameworks & Drivers ring).
//!
//! This is the only place concrete technology is chosen and wired (the file event log,
//! the reasoning adapter, the seeded id source, the clock). The command logic lives in
//! library functions so it is testable without spawning a process; `main.rs` is a thin
//! shell over [`run_cli`].

use eak_assets::{
    asset::{AssetRecord, AssetState, AssetType},
    crosscheck::{self, cross_check_metadata, CrossCheckAssessment},
    facts::{DatasheetFact, FactCollection, FactKind, ParameterFact},
    identity::{self, verify_identity_from_asset, IdentityResult},
    import::{import_asset, import_directory, list_component_assets, verify_asset},
    parser::{parse_datasheet, DatasheetParser, ParseResult},
    store::{
        self, get_asset_by_id, get_cross_check_reports, get_datasheet_facts,
        get_identity_verification, get_parse_result, init_db, store_cross_check_report,
        store_datasheet_facts, store_identity_verification, store_parse_result, update_asset,
    },
};
use eak_domain::{
    ComponentClass, Decision, Evidence, EvidenceKind, Priority, ProvenanceLink, RelationType,
    Requirement, RequirementCategory,
};
use eak_kicad::ImportedComponent;
use eak_phases::{
    BomPlanningMachine, BomVerificationMachine, ComponentPlacementMachine,
    ConstraintExtractionMachine, ConstraintVerificationMachine, DfmVerificationMachine,
    DrcVerificationMachine, EmcAnalysisMachine, EngineeringAnalysisMachine, ErcVerificationMachine,
    ManufacturingGenerationMachine, PcbFloorPlanningMachine, RequirementPlanningMachine,
    RoutingPlanningMachine, SchematicPlanningMachine,
};
use eak_ports::{EventSink, ReasoningEngine};
use eak_reasoning::{Cassette, FixtureEngine};
use eak_runtime::{
    replay, AgentContext, Autonomy, CapabilityRequest, Clock, LogicalClock, LoopBack, Orchestrator,
    RuntimeCore, SeededIdSource, SystemClock, WorkflowPlan,
};
use eak_store::FileEventLog;
use eak_units::{PhysicalQuantity, Unit};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use eak_domain::{EntityId, RelationType as Relation, RequirementStatus};
pub use eak_runtime::{EngineeringState, PhaseOutcome};

const DEFAULT_CASSETTE: &str = include_str!("../fixtures/default_cassette.json");

#[derive(Debug, Clone, Copy)]
pub enum ReasoningChoice {
    Fixture,
    Live,
}

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub intent: String,
    pub reasoning: ReasoningChoice,
    pub cassette: Option<PathBuf>,
    pub log: PathBuf,
    pub model: String,
    pub seed: u64,
    /// Use a logical (counter) clock for a fully reproducible run (tests).
    pub deterministic_clock: bool,
}

pub struct RunReport {
    pub outcomes: Vec<(String, PhaseOutcome)>,
    pub state: EngineeringState,
    pub log_path: PathBuf,
}

#[derive(Debug)]
pub enum CliError {
    Msg(String),
    /// A design could not be imported. Either the `.kicad_pcb` failed to parse, or a proposed
    /// import entity was declined at the capability seam (P3) — never a back door; the runtime
    /// re-validated the proposal and rejected it, exactly as it would a generated one.
    Import(String),
}
impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Msg(m) => write!(f, "{m}"),
            CliError::Import(m) => write!(f, "import failed: {m}"),
        }
    }
}
impl std::error::Error for CliError {}
impl From<eak_ports::StoreError> for CliError {
    fn from(e: eak_ports::StoreError) -> Self {
        CliError::Msg(e.to_string())
    }
}
impl From<eak_kicad::ImportError> for CliError {
    fn from(e: eak_kicad::ImportError) -> Self {
        CliError::Import(e.to_string())
    }
}
impl From<eak_runtime::CapabilityError> for CliError {
    fn from(e: eak_runtime::CapabilityError) -> Self {
        CliError::Import(e.to_string())
    }
}
impl From<rusqlite::Error> for CliError {
    fn from(e: rusqlite::Error) -> Self {
        CliError::Msg(e.to_string())
    }
}
impl From<eak_ports::provider_config::ProviderConfigError> for CliError {
    fn from(e: eak_ports::provider_config::ProviderConfigError) -> Self {
        CliError::Msg(e.to_string())
    }
}
impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Msg(e.to_string())
    }
}
impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Msg(e.to_string())
    }
}

fn build_reasoning(cfg: &RunConfig) -> Result<Box<dyn ReasoningEngine>, CliError> {
    match cfg.reasoning {
        ReasoningChoice::Fixture => {
            let engine = match &cfg.cassette {
                Some(path) => {
                    FixtureEngine::load(path).map_err(|e| CliError::Msg(e.to_string()))?
                }
                None => {
                    let cassette: Cassette = serde_json::from_str(DEFAULT_CASSETTE)
                        .map_err(|e| CliError::Msg(e.to_string()))?;
                    FixtureEngine::from_cassette(cassette)
                }
            };
            Ok(Box::new(engine))
        }
        ReasoningChoice::Live => {
            #[cfg(feature = "live")]
            {
                let engine = eak_reasoning::AnthropicEngine::from_env(cfg.model.clone())
                    .map_err(|e| CliError::Msg(e.to_string()))?;
                Ok(Box::new(engine))
            }
            #[cfg(not(feature = "live"))]
            {
                let _ = cfg;
                Err(CliError::Msg(
                    "live reasoning requires building with --features live".into(),
                ))
            }
        }
    }
}

/// Run the default workflow on a design intent, building the reasoning engine from `cfg`.
/// Starts a fresh event log at `cfg.log`.
pub fn run(cfg: &RunConfig) -> Result<RunReport, CliError> {
    let reasoning = build_reasoning(cfg)?;
    run_with(reasoning, cfg)
}

/// Run the default workflow with a caller-supplied reasoning engine. Tests use this to
/// inject a fixture that drives a specific scenario (consistent / contradictory / waived)
/// without going through cassette files.
pub fn run_with(
    reasoning: Box<dyn ReasoningEngine>,
    cfg: &RunConfig,
) -> Result<RunReport, CliError> {
    run_with_sink(reasoning, cfg, None)
}

/// Like [`run_with`], but also attaches a live [`EventSink`] so the caller observes every event as
/// it commits — the streaming entry point a desktop UI (the `eak-app` Tauri shell) drives. Passing
/// `None` behaves exactly like [`run_with`]. The sink only *observes*: the run, its event log, and
/// its replay are byte-identical whether or not one is attached (P4). The sink is set before
/// `capture_intent`, so the very first (intent) event streams too.
pub fn run_with_sink(
    reasoning: Box<dyn ReasoningEngine>,
    cfg: &RunConfig,
    sink: Option<Box<dyn EventSink>>,
) -> Result<RunReport, CliError> {
    let _ = std::fs::remove_file(&cfg.log); // a run starts a fresh project history
    let log = FileEventLog::open(&cfg.log)?;
    let ids = Box::new(SeededIdSource::new(cfg.seed));
    let clock: Box<dyn Clock> = if cfg.deterministic_clock {
        Box::new(LogicalClock::new())
    } else {
        Box::new(SystemClock)
    };

    let mut core = RuntimeCore::new(Box::new(log), reasoning, ids, clock, Autonomy::Autonomous);
    if let Some(sink) = sink {
        core.set_sink(sink);
    }
    core.capture_intent(&cfg.intent, "engineer")?;

    let mut plan = default_workflow();
    let outcomes = Orchestrator::new().run(&mut plan, &mut core);

    Ok(RunReport {
        outcomes,
        state: core.state.clone(),
        log_path: cfg.log.clone(),
    })
}

// ----------------------------- KiCad import driver (epic E5 / B1) -----------------------------

/// A minimal append-only in-memory [`eak_ports::EventLog`] backing a self-contained
/// [`import_design`] run. An import is a pure transformation (parse -> seam -> fold), so it needs
/// no on-disk history to feed the runtime. The recorded events remain queryable through
/// `RuntimeCore::log()`, which is how a caller (and the tests) prove each imported entity went
/// through `commit`, not a side channel.
struct MemoryEventLog {
    records: Vec<eak_ports::EventRecord>,
}
impl eak_ports::EventLog for MemoryEventLog {
    fn append(
        &mut self,
        events: &[(eak_ports::Timestamp, eak_ports::Event)],
    ) -> Result<Vec<eak_ports::Seq>, eak_ports::StoreError> {
        let mut seqs = Vec::new();
        for (ts, ev) in events {
            let seq = self.records.len() as eak_ports::Seq;
            self.records.push(eak_ports::EventRecord {
                seq,
                timestamp: *ts,
                event: ev.clone(),
            });
            seqs.push(seq);
        }
        Ok(seqs)
    }
    fn read_all(&self) -> Result<Vec<eak_ports::EventRecord>, eak_ports::StoreError> {
        Ok(self.records.clone())
    }
    fn next_seq(&self) -> eak_ports::Seq {
        self.records.len() as eak_ports::Seq
    }
}

/// Construct a runtime for an import: an in-memory log (imports keep no persistent history), a
/// logical clock + seeded ids so the run is reproducible (P4), and the default fixture reasoning
/// engine — which import never calls, since importing a finished board involves no reasoning.
fn import_core() -> Result<RuntimeCore, CliError> {
    let cassette: Cassette =
        serde_json::from_str(DEFAULT_CASSETTE).map_err(|e| CliError::Msg(e.to_string()))?;
    let reasoning: Box<dyn ReasoningEngine> = Box::new(FixtureEngine::from_cassette(cassette));
    Ok(RuntimeCore::new(
        Box::new(MemoryEventLog { records: vec![] }),
        reasoning,
        Box::new(SeededIdSource::new(1)),
        Box::new(LogicalClock::new()),
        Autonomy::Autonomous,
    ))
}

/// Feed an imported KiCad design through the **real capability seam**, so an imported board earns
/// the same P3 re-validation and the same append-only event log as a generated one — no back door.
///
/// Parses `kicad_src` into an [`eak_kicad::ImportedDesign`], then commits its entities — in
/// dependency order — through [`RuntimeCore::invoke`]: the single [`Board`](eak_domain::Board)
/// outline first, then (F2) each imported footprint as a realized [`Component`](eak_domain::Component)
/// (+its [`Pin`](eak_domain::Pin)s) with a [`Placement`](eak_domain::Placement), then each
/// [`Net`](eak_domain::Net), then each routed [`Track`](eak_domain::Track). Every entity funnels
/// through the runtime's `commit` (stamp -> append -> fold), identical to the generation path. Any
/// parse failure or seam rejection is surfaced as a [`CliError`] — the model (and, here, the
/// importer) is never trusted to bypass validation.
///
/// **Pad→pin→net membership (G1):** components are realized *before* nets, so by the time each
/// [`Net`](eak_domain::Net) is created its member pins are already committed. As each footprint is
/// realized, the importer's per-pad net annotation ([`ImportedComponent::pin_nets`]) is folded into a
/// `net-index -> committed-pin-ids` map keyed by the KiCad net index (which equals the domain net's
/// [`EntityId`]); each net is then created with `members` set to the real pin ids for its index. The
/// nets stay [`NetOrigin::Physical`](eak_domain::NetOrigin), a net with no pads on it keeps empty
/// members (still valid Physical), and because only *committed* pin ids ever enter the map the
/// `CreateNet` seam's phantom-pin rule (ADR-0016) still rejects any member that is not a committed
/// pin — the membership is real, never fabricated.
///
/// **Footprints (F2, ADR-0017):** the `RealizeComponent` seam requires ≥1 pin and, for a
/// [`Synthesized`](eak_domain::ComponentOrigin::Synthesized) component, a committed originating
/// block. An imported footprint carries [`ComponentOrigin::Imported`](eak_domain::ComponentOrigin)
/// with a null `from_block`: an imported board declares no functional decomposition, so the seam
/// accepts it without a block rather than fabricating a synthetic intent/requirement/block spine
/// (which would poison traceability — the architect rejected that). The ≥1-pin rule still holds
/// honestly: the pins are the footprint's real pads. NO `IntentCaptured` / `RequirementCommitted` /
/// `FunctionalBlockCommitted` is emitted for an import — the log carries only what the board actually
/// declares. Since G1 an imported net carries its real pad members (see the membership note above),
/// so a realized pin is joined to its net — through the seam, never fabricated.
///
/// The runtime is returned so the caller can read the reconstructed state (`core.state`) and the
/// recorded event log (`core.log()`), which together prove the import went through `commit`.
pub fn import_design(kicad_src: &str) -> Result<RuntimeCore, CliError> {
    let design = eak_kicad::import_kicad_pcb(kicad_src)?;
    let mut core = import_core()?;

    // Dependency order: the outline must exist before any net, track, or placement (routing/placement
    // seams require a board). Each `invoke` re-validates at the seam (P3) and, on success, commits.
    core.invoke(CapabilityRequest::CreateBoard {
        board: design.board,
        links: vec![],
    })?;

    // F2/G1 (ADR-0017): land imported footprints as Components + Placements through the real seam
    // FIRST, before the nets — so each pad's Pin is committed (its EntityId real) by the time the net
    // it belongs to is created, and the net can carry that pin as a genuine member. Each component is
    // already tagged `ComponentOrigin::Imported` with a null `from_block` by the importer, so
    // RealizeComponent accepts it WITHOUT a fabricated intent/requirement/block spine — an imported
    // board declares no functional decomposition, and the seam re-validates that honestly (≥1 pin, a
    // board exists, no double-placement) exactly as it does for a generated part. As each footprint is
    // realized, its `pin_nets` annotation is folded into `net_members`: net-index -> committed pin
    // ids. A copper-only import iterates an empty vec, so it commits no component and the map stays
    // empty — byte-for-byte unchanged from before.
    let mut net_members: BTreeMap<u128, Vec<EntityId>> = BTreeMap::new();
    for imported in design.components {
        let ImportedComponent {
            component,
            pins,
            placement,
            pin_nets,
        } = imported;
        // Record pad -> pin -> net BEFORE `pins` is moved into the seam. Only kept net indices appear
        // in `pin_nets` (the importer drops unconnected/dangling pads to `None`), so every recorded id
        // is a real pin about to be committed — no phantom ever enters the map.
        for (pin, net_idx) in pins.iter().zip(pin_nets.iter()) {
            if let Some(idx) = net_idx {
                net_members.entry(*idx).or_default().push(pin.id);
            }
        }
        core.invoke(CapabilityRequest::RealizeComponent {
            component,
            pins,
            links: vec![],
        })?;
        core.invoke(CapabilityRequest::PlaceComponent {
            placement,
            links: vec![],
        })?;
    }

    // Nets, now carrying their real committed pin members (G1). The KiCad net index equals the domain
    // net's EntityId, so `net_members[net.id.0]` are the pins that landed on this net; a net with no
    // pads keeps empty members (still a valid `Physical` net). The nets stay `NetOrigin::Physical`,
    // and because every listed member is an already-committed pin, the CreateNet seam's phantom-pin
    // rule (ADR-0016) passes for real reasons — it would still reject any fabricated id.
    for mut net in design.nets {
        if let Some(members) = net_members.get(&net.id.0) {
            net.members = members.clone();
        }
        core.invoke(CapabilityRequest::CreateNet { net, links: vec![] })?;
    }

    // Tracks last: RouteNet re-validates that the realized net is committed, which it now is.
    for track in design.tracks {
        core.invoke(CapabilityRequest::RouteNet {
            track,
            links: vec![],
        })?;
    }

    Ok(core)
}

/// The **verify-only** workflow (epic E5 / increment B2): ONLY the verification-family machines,
/// in their canonical order, with every SYNTHESIS phase skipped. This is the review half of the
/// "import → AI-review always works" fallback — it runs the existing deterministic rule set over an
/// already-populated design (e.g. an imported `.kicad_pcb`, see [`import_and_verify`]) and surfaces
/// its violations, without generating a single entity.
///
/// The six phases are exactly the rule-check machines [`default_workflow`] runs, constructed the
/// same way — Constraint Verification -> ERC Verification -> BOM Verification -> DRC Verification ->
/// DFM Verification -> EMC Analysis. Each reads committed state and raises violations; none
/// synthesizes. On an import-only design most stay silent (no constraints, no realized schematic, no
/// BOM, no placements), and the DRC family is the one that reads the imported copper — so an import
/// review that finds a defect finds it there.
///
/// EXCLUDED, and why (honesty over coverage): every synthesis phase is omitted because it would
/// fabricate design data the review must not invent — Requirement/Schematic/Floor/Placement/Routing
/// planning, Manufacturing Generation (the terminal *generator* + global gate), and Constraint
/// Extraction. **Engineering Analysis is also excluded**: despite the name it SYNTHESIZES a
/// functional block per requirement AND hard-requires a design intent (`ctx.design_intent()`),
/// neither of which an import provides — it would error, not review. The plan is LINEAR (no
/// loop-backs): a loop-back's target is a synthesis phase that is not present here, so a failed gate
/// stops the review at that gate with the violation recorded — precisely the "here is what is wrong
/// with your board" outcome the fallback wants.
pub fn verify_only_workflow() -> WorkflowPlan {
    WorkflowPlan::new(vec![
        Box::new(ConstraintVerificationMachine::new()),
        Box::new(ErcVerificationMachine::new()),
        Box::new(BomVerificationMachine::new()),
        Box::new(DrcVerificationMachine::new()),
        Box::new(DfmVerificationMachine::new()),
        Box::new(EmcAnalysisMachine::new()),
    ])
}

// ----------------- Default fabrication process floor (epic E5 / increment E5.1) -----------------
//
// A `.kicad_pcb` carries copper geometry but NOT a fabrication process class, so the two geometric
// DRC rules that need a stated process floor — `drc-trace-width` (Fabrication Length slot 0) and
// `drc-copper-clearance` (slot 2) — have nothing to check against on a bare import and correctly
// stay silent. That left "import -> AI-review" blind to a too-thin trace or a too-close copper pair
// (only `drc-unrouted-net` fired, since it reads topology alone). Seeding a *default* floor — a real
// Fabrication requirement committed through the same capability the synthesis path uses — gives
// those rules a floor so the review catches real geometry defects. It is a DEFAULT, explicitly
// overridable: a design that states its own Fabrication process window supersedes it.

/// Default minimum manufacturable **trace width**, in millimetres. 0.15 mm ≈ 6 mil is the
/// standard-capability minimum trace/space every mainstream fabricator quotes for an IPC-2221
/// generic-design board at IPC-A-600 / IPC-6012 **Class 2** (dedicated service — most commercial
/// work), the conservative "6 mil minimum trace/space" the engineering-science layer names as the
/// canonical process floor (`engineering-science/manufacturing/dfm-principles.md` §process-
/// capability; `.../ipc-standards.md` §1, §5). A DEFAULT, not a measured fab limit — a real process
/// window overrides it by stating its own `Fabrication` requirement.
const DEFAULT_MIN_TRACE_WIDTH_MM: f64 = 0.15;

/// Default minimum copper-to-copper **clearance**, in millimetres — the same IPC-2221 Class-2
/// 6 mil (0.15 mm) minimum space (width and spacing share the standard-capability floor; cited as
/// [`DEFAULT_MIN_TRACE_WIDTH_MM`]). Overridable by a stated process window.
const DEFAULT_MIN_CLEARANCE_MM: f64 = 0.15;

/// Default board-edge keep-out band, in millimetres — slot 1 of the positional Fabrication-target
/// contract. Set to 0.5 mm, IDENTICAL to `eak-engines`' own `DFM_EDGE_CLEARANCE_FALLBACK_MM` (the
/// keep-out the DFM edge rules already apply to an import when no Fabrication requirement is stated),
/// so seeding the floor leaves DFM edge behaviour on an import UNCHANGED. It exists only to occupy
/// slot 1 so the clearance floor lands in slot 2 (IPC-2221 edge clearance, `.../ipc-standards.md`).
const DEFAULT_EDGE_KEEPOUT_MM: f64 = 0.5;

/// The default fabrication process-floor targets an imported `.kicad_pcb` is seeded with, in the
/// documented POSITIONAL SLOT CONTRACT `eak-engines`' `fabrication_length_targets` reads: slot 0 =
/// minimum trace width (`drc-trace-width`), slot 1 = board-edge keep-out (the DFM edge rules),
/// slot 2 = minimum copper-to-copper clearance (`drc-copper-clearance`). All three slots are filled
/// because the contract is positional — the clearance floor is only reachable at slot 2. A DEFAULT,
/// explicitly overridable by a stated `Fabrication` requirement; see the per-constant citations.
pub fn default_fabrication_floor() -> Vec<PhysicalQuantity> {
    vec![
        PhysicalQuantity::new(DEFAULT_MIN_TRACE_WIDTH_MM, Unit::Millimetre), // slot 0
        PhysicalQuantity::new(DEFAULT_EDGE_KEEPOUT_MM, Unit::Millimetre),    // slot 1
        PhysicalQuantity::new(DEFAULT_MIN_CLEARANCE_MM, Unit::Millimetre),   // slot 2
    ]
}

/// Seed the imported design with the [`default_fabrication_floor`] as a
/// [`Fabrication`](RequirementCategory::Fabrication) [`Requirement`], committed through the SAME
/// sanctioned capability the synthesis path uses — [`CapabilityRequest::CreateRequirement`] — so it
/// is stamped, appended, and folded by the runtime's single `commit` seam (P2/P3), never poked into
/// state. This is exactly how the requirement agent installs a process floor derived from a design
/// intent; here the value is a documented default instead of an intent-derived one.
///
/// The requirement is rooted in the imported [`Board`](eak_domain::Board) — a real committed entity,
/// since an Accepted requirement needs a non-null source ([`Requirement::validate`]) — and carries a
/// `StandardClause` [`Evidence`] citing IPC-2221 Class 2, with `JustifiedBy` / `DerivedFrom` /
/// `Supports` provenance links, so the seeded floor is as traceable as a synthesised one. With no
/// board there is no copper to check, so nothing is seeded (the geometry rules are silent without a
/// board regardless). Returns the seam error unchanged if the runtime rejects the proposal (P3) —
/// never a back door.
/// Install the default IPC-2221 Class 2 fabrication process floor through the real `CreateRequirement`
/// seam, so the geometric DRC rules (`drc-trace-width`, `drc-copper-clearance`) have a bound to
/// evaluate over imported copper (which carries no process class). Public so the review-cassette
/// emitter (`examples/emit_review_cassette.rs`) can reproduce `import_and_verify`'s seeding step.
pub fn seed_default_fabrication_floor(core: &mut RuntimeCore) -> Result<(), CliError> {
    let Some(source) = core.state.board.as_ref().map(|b| b.id) else {
        return Ok(());
    };
    let rid = core.fresh_id();
    let did = core.fresh_id();
    let eid = core.fresh_id();
    let link_justified = core.fresh_id();
    let link_derived = core.fresh_id();
    let link_supports = core.fresh_id();
    core.invoke(CapabilityRequest::CreateRequirement {
        requirement: Requirement {
            id: rid,
            statement: "Imported board defaults to the IPC-2221 Class 2 fabrication process floor"
                .into(),
            category: RequirementCategory::Fabrication,
            priority: Priority::High,
            acceptance_criterion: format!(
                "every trace >= {DEFAULT_MIN_TRACE_WIDTH_MM} mm wide and every copper gap \
                 >= {DEFAULT_MIN_CLEARANCE_MM} mm"
            ),
            status: RequirementStatus::Accepted,
            source,
            targets: default_fabrication_floor(),
        },
        decision: Decision {
            id: did,
            subject: rid,
            rationale: "No fab process was imported with the board; apply a conservative, \
                        overridable default floor so geometric DRC can run"
                .into(),
            decider: "ImportDefaultFabFloor".into(),
            reasoning_call_seq: None,
            evidence: vec![eid],
            confidence: 1.0,
        },
        evidence: vec![Evidence {
            id: eid,
            kind: EvidenceKind::StandardClause,
            content_reference: "IPC-2221 generic design, Class 2 standard capability: 6 mil \
                                (0.15 mm) minimum trace width and copper spacing"
                .into(),
            source: "IPC-2221".into(),
            reliability: 1.0,
        }],
        links: vec![
            ProvenanceLink {
                id: link_justified,
                from: rid,
                to: did,
                relation: RelationType::JustifiedBy,
            },
            ProvenanceLink {
                id: link_derived,
                from: rid,
                to: source,
                relation: RelationType::DerivedFrom,
            },
            ProvenanceLink {
                id: link_supports,
                from: did,
                to: eid,
                relation: RelationType::Supports,
            },
        ],
    })?;
    Ok(())
}

/// Import a `.kicad_pcb` and run the **verify-only** review over it (epic E5 / increment B2) — the
/// import -> AI-review fallback, end to end and with no back door. Populates a [`RuntimeCore`] via the
/// real capability seam ([`import_design`]), then drives [`verify_only_workflow`] over that SAME core
/// through the ordinary [`Orchestrator`], so every violation is minted at the runtime's commit seam
/// (P3) and folded into state exactly as a generated design's would be — the review reuses the real
/// runtime, never a side channel.
///
/// Before the review runs, the imported design is seeded with the [`default_fabrication_floor`]
/// (increment E5.1) via [`seed_default_fabrication_floor`] — a `Fabrication` requirement committed
/// through the real `CreateRequirement` seam. A `.kicad_pcb` carries copper but no process class, so
/// without this the geometric DRC rules (`drc-trace-width`, `drc-copper-clearance`) have no floor and
/// stay silent; seeding the default gives them one, so an imported too-thin trace or too-close copper
/// pair is now caught by the review. It is a DEFAULT — a design that states its own process window
/// overrides it.
///
/// The verification machines otherwise tolerate an import-only design (P4/P9 honesty): with no
/// constraints, realized schematic, BOM, or placements, the constraint / ERC / BOM / DFM / EMC gates
/// find nothing, so a defect in the imported copper surfaces at the DRC gate. The plan is linear, so
/// it stops at the first failing gate with that violation recorded. Returns the per-phase outcomes
/// and the reconstructed [`EngineeringState`], which carries the raised violations and the
/// provenance links that make each one traceable back to the net/track it implicates.
pub fn import_and_verify(kicad_src: &str) -> Result<RunReport, CliError> {
    let mut core = import_design(kicad_src)?;
    // Seed the default fabrication process floor (E5.1) BEFORE verifying, through the real
    // CreateRequirement seam, so `drc-trace-width` / `drc-copper-clearance` have a floor to evaluate.
    seed_default_fabrication_floor(&mut core)?;
    let mut plan = verify_only_workflow();
    let outcomes = Orchestrator::new().run(&mut plan, &mut core);
    Ok(RunReport {
        outcomes,
        state: core.state.clone(),
        // An import runs on an in-memory event log (see `import_core`), so there is no on-disk path;
        // every committed event remains queryable through the returned state's fold.
        log_path: PathBuf::from("<in-memory import log>"),
    })
}

/// The default Phase-3 workflow: Requirement Planning -> Engineering Analysis ->
/// Constraint Extraction -> Constraint Verification -> Schematic Planning -> ERC
/// Verification -> BOM Planning -> BOM Verification -> PCB Floor Planning ->
/// Component Placement -> Routing Planning -> DRC Verification -> DFM Verification ->
/// EMC Analysis -> Manufacturing Generation. Only Requirement Planning reasons (P3); every other
/// phase here is deterministic, so a run replays bit-identically (P4).
///
/// Manufacturing Generation is the terminal phase and the GLOBAL gate: it has no loop-back of its
/// own (it is reached only once every per-phase gate upstream has passed), and it releases the
/// design — lowering the terminal Manufacturing IR — iff no open blocking violation remains
/// anywhere; otherwise it reports `Blocked`.
///
/// Six correctness-loop edges bound the self-correction: a failed constraint verification
/// routes back to extraction, a failed ERC routes back to schematic planning, a failed
/// BOM verification routes back to BOM planning, a failed DRC routes back to routing planning
/// (clearance/geometry defects are routing defects), a failed DFM routes back to component
/// placement (manufacturability defects are usually placement-driven), and a failed EMC analysis
/// routes back to routing planning (emissions/coupling are routing-dominated — a re-route is what
/// changes the trace geometry) — each the canonical loop-back target, capped at `max_retries` 2.
/// Because extraction, schematic planning, BOM planning, routing planning, and component placement
/// are *idempotent* (a re-entry produces the identical artifact), the loop is a no-op recovery for
/// a design that is genuinely infeasible: it deterministically exhausts the retries and then
/// surfaces the open, blocking violation rather than looping forever (P13). The loop-back is the
/// seam where a future reasoning-assisted re-synthesis (or a human waiver between passes) can
/// actually change the artifact and clear the violation.
fn default_workflow() -> WorkflowPlan {
    WorkflowPlan::with_loopbacks(
        vec![
            Box::new(RequirementPlanningMachine::new()),
            Box::new(EngineeringAnalysisMachine::new()),
            Box::new(ConstraintExtractionMachine::new()),
            Box::new(ConstraintVerificationMachine::new()),
            Box::new(SchematicPlanningMachine::new()),
            Box::new(ErcVerificationMachine::new()),
            Box::new(BomPlanningMachine::new()),
            Box::new(BomVerificationMachine::new()),
            Box::new(PcbFloorPlanningMachine::new()),
            Box::new(ComponentPlacementMachine::new()),
            Box::new(RoutingPlanningMachine::new()),
            Box::new(DrcVerificationMachine::new()),
            Box::new(DfmVerificationMachine::new()),
            Box::new(EmcAnalysisMachine::new()),
            Box::new(ManufacturingGenerationMachine::new()),
        ],
        vec![
            LoopBack {
                from: "ConstraintVerification".into(),
                to: "ConstraintExtraction".into(),
                max_retries: 2,
            },
            LoopBack {
                from: "ErcVerification".into(),
                to: "SchematicPlanning".into(),
                max_retries: 2,
            },
            LoopBack {
                from: "BomVerification".into(),
                to: "BomPlanning".into(),
                max_retries: 2,
            },
            LoopBack {
                from: "DrcVerification".into(),
                to: "RoutingPlanning".into(),
                max_retries: 2,
            },
            LoopBack {
                from: "DfmVerification".into(),
                to: "ComponentPlacement".into(),
                max_retries: 2,
            },
            LoopBack {
                from: "EmcAnalysis".into(),
                to: "RoutingPlanning".into(),
                max_retries: 2,
            },
        ],
    )
}

/// Replay an event log into a reconstructed [`EngineeringState`] (no model, no clock).
pub fn replay_cmd(log_path: &Path) -> Result<EngineeringState, CliError> {
    let log = FileEventLog::open(log_path)?;
    let state = replay(&log)?;
    Ok(state)
}

/// Render the provenance chain for a requirement (by short or full hex id).
pub fn trace_cmd(log_path: &Path, requirement: &str) -> Result<String, CliError> {
    let log = FileEventLog::open(log_path)?;
    let state = replay(&log)?;
    let req = state
        .requirements
        .iter()
        .find(|r| r.id.to_hex() == requirement || r.id.short() == requirement)
        .ok_or_else(|| CliError::Msg(format!("requirement {requirement} not found")))?;

    let mut out = String::new();
    out.push_str(&format!(
        "Requirement {} [{:?} / {:?}]\n  \"{}\"\n  acceptance: {}\n",
        req.id.short(),
        req.category,
        req.status,
        req.statement,
        req.acceptance_criterion
    ));

    if let Some(link) = state
        .links
        .iter()
        .find(|l| l.from == req.id && l.relation == RelationType::JustifiedBy)
    {
        if let Some(dec) = state.decision(link.to) {
            out.push_str(&format!(
                "  +- Decision {} by {} (confidence {:.2}, reasoning call #{})\n     rationale: {}\n",
                dec.id.short(),
                dec.decider,
                dec.confidence,
                dec.reasoning_call_seq
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "none".into()),
                dec.rationale
            ));
            for evid in &dec.evidence {
                if let Some(ev) = state.evidence_item(*evid) {
                    out.push_str(&format!(
                        "     +- Evidence {} ({:?}) from {}\n",
                        ev.id.short(),
                        ev.kind,
                        ev.source
                    ));
                }
            }
        }
    }

    if let Some(intent) = &state.intent {
        if req.source == intent.id {
            out.push_str(&format!(
                "  +- derives from Design Intent {}: \"{}\"\n",
                intent.id.short(),
                intent.statement
            ));
        }
    }

    Ok(out)
}

// ----------------------------- CLI plumbing -----------------------------

use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "eak",
    version,
    about = "Electronics Agent Kit — Phase 1 runtime CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run Requirement Planning (+ Engineering Analysis stub) on a design intent.
    Run {
        #[arg(long)]
        intent: String,
        #[arg(long, default_value = "fixture")]
        reasoning: String,
        #[arg(long)]
        cassette: Option<PathBuf>,
        #[arg(long, default_value = "eak-events.jsonl")]
        log: PathBuf,
        #[arg(long, default_value = "claude-opus-4-8")]
        model: String,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long)]
        show_state: bool,
        /// Use a logical clock so the run is fully reproducible.
        #[arg(long)]
        deterministic: bool,
    },
    /// Replay an event log and print the reconstructed state.
    Replay {
        #[arg(long)]
        log: PathBuf,
        #[arg(long)]
        show_state: bool,
    },
    /// Print the provenance chain for a requirement (by short or full id).
    Trace {
        #[arg(long)]
        log: PathBuf,
        requirement: String,
    },
    /// List all configured providers.
    ListProviders {
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
    },
    /// List available models for a provider.
    ListModels {
        #[arg(long)]
        provider: String,
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
    },
    /// Test connectivity to a provider.
    TestConnection {
        #[arg(long)]
        provider: String,
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
    },
    /// Add or update a provider configuration.
    ConfigureProvider {
        #[arg(long)]
        provider_id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        default_model: Option<String>,
        #[arg(long)]
        credential_store: Option<String>,
        #[arg(long)]
        credential_key: Option<String>,
        #[arg(long, value_parser = parse_header)]
        extra_header: Vec<String>,
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
    },
    /// Parse a datasheet PDF and extract structured facts.
    AssetParse {
        /// Asset ID to parse
        #[arg(long)]
        id: i64,
        /// Database path
        #[arg(long, default_value = "data/library.db")]
        db: PathBuf,
    },
    /// List extracted facts for a datasheet asset.
    AssetFacts {
        /// Asset ID
        #[arg(long)]
        id: i64,
        /// Filter by fact kind (e.g., Voltage, Current, Resistance)
        #[arg(long)]
        kind: Option<String>,
        /// Database path
        #[arg(long, default_value = "data/library.db")]
        db: PathBuf,
    },
    /// Cross-check datasheet facts against component metadata.
    AssetCompare {
        /// Asset ID
        #[arg(long)]
        id: i64,
        /// Component class (e.g., Passive, Diode, Transistor)
        #[arg(long)]
        class: String,
        /// Database path
        #[arg(long, default_value = "data/library.db")]
        db: PathBuf,
    },
    /// Verify datasheet identity matches component record.
    AssetVerifyIdentity {
        /// Asset ID
        #[arg(long)]
        id: i64,
        /// Database path
        #[arg(long, default_value = "data/library.db")]
        db: PathBuf,
    },
}

fn print_run(report: &RunReport, show_state: bool) {
    println!("Run complete. Event log: {}", report.log_path.display());
    for (phase, outcome) in &report.outcomes {
        let status = match outcome {
            PhaseOutcome::Success => "OK".to_string(),
            PhaseOutcome::Failed(r) => format!("FAILED: {r}"),
        };
        println!("  phase {phase:<22} {status}");
    }
    println!(
        "  requirements: {}  decisions: {}  evidence: {}  provenance links: {}",
        report.state.requirements.len(),
        report.state.decisions.len(),
        report.state.evidence.len(),
        report.state.links.len()
    );
    for r in &report.state.requirements {
        println!("    - [{}] {}", r.id.short(), r.statement);
    }
    if show_state {
        println!("\n{}", report.state.canonical_json());
    }
}

fn print_replay(state: &EngineeringState, show_state: bool) {
    println!(
        "Replayed: {} requirements, {} decisions, {} evidence, {} links",
        state.requirements.len(),
        state.decisions.len(),
        state.evidence.len(),
        state.links.len()
    );
    if show_state {
        println!("\n{}", state.canonical_json());
    }
}

/// Handle the list-providers command.
fn handle_list_providers(project_dir: &Path) -> Result<(), CliError> {
    use eak_ports::provider_config::ProviderProjectConfig;
    let config = ProviderProjectConfig::load(project_dir)?;

    if config.providers.list().is_empty() {
        println!("No providers configured.");
        return Ok(());
    }

    println!("Configured providers:");
    for provider_id in config.providers.list() {
        let cfg = config.providers.get(provider_id).unwrap();
        let status = if cfg.enabled { "enabled" } else { "disabled" };
        let default_model = cfg
            .default_model
            .as_ref()
            .map(|m| m.0.as_str())
            .unwrap_or("none");
        let endpoint = cfg
            .endpoint
            .as_ref()
            .map(|e| e.as_str())
            .unwrap_or("default");
        println!("  {} ({})", provider_id, status);
        println!("    name: {}", cfg.name);
        println!("    endpoint: {}", endpoint);
        println!("    default_model: {}", default_model);
        if let Some(cred_ref) = &cfg.credential_ref {
            println!("    credential: {}:{}", cred_ref.store, cred_ref.key);
        }
        if let Some(headers) = &cfg.extra_headers {
            if !headers.is_empty() {
                println!("    extra_headers: {:?}", headers);
            }
        }
    }

    if let Some(active) = config.active_provider {
        println!("\nActive provider: {}", active);
    }

    Ok(())
}

/// Handle the list-models command.
fn handle_list_models(provider: &str, project_dir: &Path) -> Result<(), CliError> {
    use eak_ports::{provider_config::ProviderProjectConfig, ProviderId};
    let provider_id = ProviderId(provider.to_string());
    let config = ProviderProjectConfig::load(project_dir)?;

    if config.providers.get(&provider_id).is_none() {
        return Err(CliError::Msg(format!("Provider '{}' not found", provider)));
    }

    let models = config.models.list_for_provider(&provider_id);
    if models.is_empty() {
        println!("No models configured for provider '{}'.", provider);
        return Ok(());
    }

    println!("Models for provider '{}':", provider);
    for model in models {
        let status = if model.enabled { "enabled" } else { "disabled" };
        let caps: Vec<String> = model
            .capabilities
            .iter()
            .map(|c| format!("{:?}", c))
            .collect();
        println!("  {} ({})", model.model_id, status);
        println!("    capabilities: {}", caps.join(", "));
        if let Some(params) = model.parameters.temperature {
            println!("    temperature: {}", params);
        }
        if let Some(max_tokens) = model.parameters.max_tokens {
            println!("    max_tokens: {}", max_tokens);
        }
    }

    Ok(())
}

/// Handle the test-connection command.
fn handle_test_connection(provider: &str, project_dir: &Path) -> Result<(), CliError> {
    use eak_ports::{
        provider_config::ProviderProjectConfig, CredentialStore, EnvCredentialStore,
        ProviderFactory, ProviderId,
    };
    let provider_id = ProviderId(provider.to_string());
    let config = ProviderProjectConfig::load(project_dir)?;

    let provider_config = config
        .providers
        .get(&provider_id)
        .ok_or_else(|| CliError::Msg(format!("Provider '{}' not found", provider)))?;

    if !provider_config.enabled {
        return Err(CliError::Msg(format!(
            "Provider '{}' is disabled",
            provider
        )));
    }

    let factory = ProviderFactory::new(Box::new(EnvCredentialStore));
    let result = eak_ports::test_provider_connection(&factory, provider_config);

    if result.success {
        println!("✓ Connection successful: {}", result.message);
        if let Some(details) = result.details {
            if let Some(models) = details.get("models") {
                println!("  Models found: {}", models);
            }
        }
    } else {
        println!("✗ Connection failed: {}", result.message);
        return Err(CliError::Msg(result.message));
    }

    Ok(())
}

/// Handle the configure-provider command.
fn handle_configure_provider(
    provider_id: &str,
    name: &str,
    endpoint: Option<String>,
    default_model: Option<String>,
    credential_store: Option<String>,
    credential_key: Option<String>,
    extra_headers: Vec<String>,
    project_dir: &Path,
) -> Result<(), CliError> {
    use eak_ports::{
        provider_config::ProviderProjectConfig, CredentialRef, ModelId, ProviderConfig, ProviderId,
    };
    use std::collections::HashMap;

    let provider_id = ProviderId(provider_id.to_string());
    let mut config = ProviderProjectConfig::load(project_dir)?;

    let credential_ref = match (credential_store, credential_key) {
        (Some(store), Some(key)) => Some(CredentialRef::new(store, key)),
        (None, None) => None,
        _ => {
            return Err(CliError::Msg(
                "Both --credential-store and --credential-key must be provided together"
                    .to_string(),
            ))
        }
    };

    // Parse extra headers from "Key:Value" format
    let parsed_headers: HashMap<String, String> = extra_headers
        .into_iter()
        .map(|h| {
            let parts: Vec<&str> = h.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(CliError::Msg(format!(
                    "Header must be in format 'Key:Value', got: {}",
                    h
                )));
            }
            Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    let provider_config = ProviderConfig {
        id: provider_id.clone(),
        name: name.to_string(),
        enabled: true,
        endpoint,
        credential_ref,
        default_model: default_model.map(ModelId),
        extra_headers: if parsed_headers.is_empty() {
            None
        } else {
            Some(parsed_headers)
        },
    };

    config.register_provider(provider_config);
    config.save(project_dir)?;

    println!("Provider '{}' configured successfully.", provider_id);
    Ok(())
}

/// Handle asset parse command.
fn handle_asset_parse(id: i64, db: PathBuf) -> Result<(), CliError> {
    let conn = init_db(&db).map_err(|e| CliError::Msg(e.to_string()))?;

    let mut asset = get_asset_by_id(&conn, id)?
        .ok_or_else(|| CliError::Msg(format!("Asset not found: {}", id)))?;

    if asset.asset_type != AssetType::Datasheet && asset.asset_type != AssetType::AppNote {
        return Err(CliError::Msg(
            "Asset is not a datasheet or app note".to_string(),
        ));
    }

    if asset.state != AssetState::UserProvided && asset.state != AssetState::Verified {
        return Err(CliError::Msg(format!(
            "Asset must be verified or user-provided, current state: {}",
            asset.state
        )));
    }

    let result = parse_datasheet(&mut asset, &db).map_err(|e| CliError::Msg(e.to_string()))?;

    // Store parse result
    store_parse_result(&conn, &result).map_err(|e| CliError::Msg(e.to_string()))?;

    // Store facts
    store_datasheet_facts(&conn, id, &result.facts).map_err(|e| CliError::Msg(e.to_string()))?;

    println!("Datasheet parsed successfully:");
    println!("  Asset ID: {}", id);
    println!("  Pages: {}", result.page_count);
    println!(
        "  Title: {}",
        result.title.unwrap_or_else(|| "unknown".to_string())
    );
    println!(
        "  Manufacturer: {}",
        result.manufacturer.unwrap_or_else(|| "unknown".to_string())
    );
    println!(
        "  MPN: {}",
        result.mpn.unwrap_or_else(|| "unknown".to_string())
    );
    println!("  Facts extracted: {}", result.facts.len());
    println!("  Sections found: {}", result.sections.len());
    if !result.warnings.is_empty() {
        println!("  Warnings:");
        for w in &result.warnings {
            println!("    - {}", w);
        }
    }

    Ok(())
}

/// Handle asset facts command.
fn handle_asset_facts(id: i64, kind: Option<String>, db: PathBuf) -> Result<(), CliError> {
    let conn = init_db(&db).map_err(|e| CliError::Msg(e.to_string()))?;

    let facts = get_datasheet_facts(&conn, id).map_err(|e| CliError::Msg(e.to_string()))?;

    if facts.is_empty() {
        println!(
            "No facts found for asset {}. Run 'eak asset-parse --id {}' first.",
            id, id
        );
        return Ok(());
    }

    let filter_kind = kind.and_then(|k| match k.to_lowercase().as_str() {
        "voltage" => Some(FactKind::Voltage),
        "current" => Some(FactKind::Current),
        "resistance" => Some(FactKind::Resistance),
        "capacitance" => Some(FactKind::Capacitance),
        "inductance" => Some(FactKind::Inductance),
        "power" => Some(FactKind::Power),
        "frequency" => Some(FactKind::Frequency),
        "temperature" => Some(FactKind::Temperature),
        "packag" => Some(FactKind::Package),
        "mpn" => Some(FactKind::Mpn),
        "manufacturer" => Some(FactKind::Manufacturer),
        _ => None,
    });

    println!("Facts for asset {}:", id);
    println!("{:-<80}", "");
    for fact in facts {
        if let Some(fk) = filter_kind {
            if fact.kind != fk {
                continue;
            }
        }
        let verified = if fact.provenance.verified { " ✓" } else { "" };
        let corrected = if fact.provenance.manually_corrected {
            " (corrected)"
        } else {
            ""
        };
        println!(
            "  [{:?}] {} = {}{}{}",
            fact.kind,
            DatasheetFact::kind_description(fact.kind),
            fact.parameter.value,
            verified,
            corrected
        );
        if !fact.parameter.conditions.is_empty() {
            println!("      Conditions: {}", fact.parameter.conditions.join(", "));
        }
        println!(
            "      Page: {}, Extracted: {}",
            fact.provenance.page,
            &fact.provenance.extracted_at[..19]
        );
        println!(
            "      Source text: \"{}\"",
            fact.provenance
                .extracted_text
                .chars()
                .take(80)
                .collect::<String>()
        );
    }
    println!("{:-<80}", "");
    Ok(())
}

/// Handle asset compare command.
fn handle_asset_compare(id: i64, class_str: String, db: PathBuf) -> Result<(), CliError> {
    let conn = init_db(&db).map_err(|e| CliError::Msg(e.to_string()))?;

    let asset = get_asset_by_id(&conn, id)?
        .ok_or_else(|| CliError::Msg(format!("Asset not found: {}", id)))?;

    let facts = get_datasheet_facts(&conn, id).map_err(|e| CliError::Msg(e.to_string()))?;

    if facts.is_empty() {
        return Err(CliError::Msg(
            "No facts found. Run 'eak asset-parse --id <id>' first.".to_string(),
        ));
    }

    // Parse component class
    let component_class = match class_str.to_lowercase().as_str() {
        "resistor" | "passive" => ComponentClass::Resistor,
        "capacitor" => ComponentClass::Capacitor,
        "ic" | "analogic" | "analog" => ComponentClass::AnalogOpAmp,
        "regulator" | "powermanagement" | "power" => ComponentClass::RegulatorLdo,
        "connector" => ComponentClass::ConnectorHeader,
        _ => {
            return Err(CliError::Msg(format!(
            "Unknown component class: {} (valid: resistor, capacitor, ic, regulator, connector)",
            class_str
        )))
        }
    };

    // Build fact collection
    let mut fact_coll = FactCollection::new(id, "1.0.0".to_string());
    for f in facts {
        fact_coll.add_fact(f);
    }

    // Create minimal component metadata for comparison
    let metadata = crosscheck::ComponentMetadata::with_family(component_class);

    // Run cross-check
    let report = cross_check_metadata(
        &asset.component_mpn,
        &asset.manufacturer,
        component_class,
        &metadata,
        &fact_coll,
    );

    // Store report
    store_cross_check_report(&conn, id, &report).map_err(|e| CliError::Msg(e.to_string()))?;

    // Print results
    println!(
        "Cross-check report for {} ({})",
        asset.component_mpn, asset.manufacturer
    );
    println!("Component class: {:?}", component_class);
    println!("Assessment: {}", report.assessment);
    println!(
        "Summary: {} checked, {} matches, {} conflicts, {} missing in datasheet, {} datasheet-only",
        report.summary.total_fields_checked,
        report.summary.matches,
        report.summary.conflicts,
        report.summary.missing_in_datasheet,
        report.summary.datasheet_only
    );

    if report.summary.conflicts > 0 {
        println!("\nCONFLICTS:");
        for fc in &report.field_comparisons {
            if fc.comparison == crate::crosscheck::ComparisonType::Conflict {
                println!(
                    "  [{:?}] {}: metadata={} vs datasheet={}",
                    fc.fact_kind,
                    fc.name,
                    fc.metadata_value
                        .as_ref()
                        .map(|v| v.value.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    fc.datasheet_value
                        .as_ref()
                        .map(|v| v.value.to_string())
                        .unwrap_or_else(|| "none".to_string())
                );
            }
        }
    }

    if report.summary.datasheet_only > 0 {
        println!("\nDATASHEET-ONLY FACTS:");
        for fc in &report.field_comparisons {
            if fc.comparison == crate::crosscheck::ComparisonType::DatasheetOnly {
                println!(
                    "  [{:?}] {}: {}",
                    fc.fact_kind,
                    fc.name,
                    fc.datasheet_value
                        .as_ref()
                        .map(|v| v.value.to_string())
                        .unwrap_or_else(|| "none".to_string())
                );
            }
        }
    }

    if report.summary.missing_in_datasheet > 0 {
        println!("\nMISSING IN DATASHEET:");
        for fc in &report.field_comparisons {
            if fc.comparison == crate::crosscheck::ComparisonType::MissingInDatasheet {
                println!(
                    "  [{:?}] {}: metadata={}",
                    fc.fact_kind,
                    fc.name,
                    fc.metadata_value
                        .as_ref()
                        .map(|v| v.value.to_string())
                        .unwrap_or_else(|| "none".to_string())
                );
            }
        }
    }

    Ok(())
}

/// Handle asset verify-identity command.
fn handle_asset_verify_identity(id: i64, db: PathBuf) -> Result<(), CliError> {
    let conn = init_db(&db).map_err(|e| CliError::Msg(e.to_string()))?;

    let asset = get_asset_by_id(&conn, id)?
        .ok_or_else(|| CliError::Msg(format!("Asset not found: {}", id)))?;

    let parse_result = get_parse_result(&conn, id)
        .map_err(|e| CliError::Msg(e.to_string()))?
        .ok_or_else(|| {
            CliError::Msg(
                "No parse result found. Run 'eak asset-parse --id <id>' first.".to_string(),
            )
        })?;

    let facts = get_datasheet_facts(&conn, id).map_err(|e| CliError::Msg(e.to_string()))?;

    let mut fact_coll = FactCollection::new(id, "1.0.0".to_string());
    for f in facts {
        fact_coll.add_fact(f);
    }

    let report = verify_identity_from_asset(&asset, &parse_result, &fact_coll);

    // Store identity verification
    store_identity_verification(&conn, id, &report).map_err(|e| CliError::Msg(e.to_string()))?;

    // Update asset
    let mut asset_mut = asset.clone();
    identity::apply_identity_result(&mut asset_mut, &report);
    store::update_asset(&conn, &asset_mut).map_err(|e| CliError::Msg(e.to_string()))?;

    println!(
        "Identity verification for {} ({}):",
        asset.component_mpn, asset.manufacturer
    );
    println!("Result: {}", report.result);
    println!("Confidence: {:.2}", report.confidence);
    println!("Summary: {}", report.summary);
    println!();
    println!(
        "Manufacturer: {} (component: {:?}, datasheet: {:?}, match: {}, conf: {:.2})",
        if report.manufacturer.matches {
            "✓"
        } else {
            "✗"
        },
        report.manufacturer.component_value,
        report.manufacturer.datasheet_value,
        report.manufacturer.matches,
        report.manufacturer.confidence
    );
    println!(
        "MPN: {} (component: {:?}, datasheet: {:?}, match: {}, conf: {:.2})",
        if report.mpn.matches { "✓" } else { "✗" },
        report.mpn.component_value,
        report.mpn.datasheet_value,
        report.mpn.matches,
        report.mpn.confidence
    );
    if let Some(pkg) = &report.package {
        println!(
            "Package: {} (component: {:?}, datasheet: {:?}, match: {}, conf: {:.2})",
            if pkg.matches { "✓" } else { "✗" },
            pkg.component_value,
            pkg.datasheet_value,
            pkg.matches,
            pkg.confidence
        );
    }

    if report.result == IdentityResult::IdentityMismatch {
        return Err(CliError::Msg("Identity mismatch detected".to_string()));
    }

    Ok(())
}

/// Parse a header string in "Key:Value" format.
fn parse_header(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("header must be in format 'Key:Value', got: {}", s));
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}

pub fn run_cli() -> ExitCode {
    let cli = Cli::parse();
    let result: Result<(), CliError> = match cli.command {
        Command::Run {
            intent,
            reasoning,
            cassette,
            log,
            model,
            seed,
            show_state,
            deterministic,
        } => {
            let choice = match reasoning.as_str() {
                "fixture" => ReasoningChoice::Fixture,
                "live" => ReasoningChoice::Live,
                other => {
                    eprintln!("error: unknown --reasoning '{other}' (use fixture|live)");
                    return ExitCode::FAILURE;
                }
            };
            let cfg = RunConfig {
                intent,
                reasoning: choice,
                cassette,
                log,
                model,
                seed,
                deterministic_clock: deterministic,
            };
            run(&cfg).and_then(|report| {
                print_run(&report, show_state);
                // A run that ends with open, blocking violations (e.g. a fault whose loop-back
                // exhausted its retries) is a design failure, not a CLI success: surface it in
                // the exit code so a CI gate can act on it. The library `run`/`run_with` still
                // return `Ok` so callers can inspect the report; only the binary's exit code
                // reflects design health.
                let open = report.state.open_blocking_violations().len();
                if open == 0 {
                    Ok(())
                } else {
                    Err(CliError::Msg(format!(
                        "design has {open} open blocking violation(s) — see the phase outcomes above"
                    )))
                }
            })
        }
        Command::Replay { log, show_state } => {
            replay_cmd(&log).map(|state| print_replay(&state, show_state))
        }
        Command::Trace { log, requirement } => trace_cmd(&log, &requirement).map(|s| print!("{s}")),
        Command::ListProviders { project_dir } => handle_list_providers(&project_dir),
        Command::ListModels {
            provider,
            project_dir,
        } => handle_list_models(&provider, &project_dir),
        Command::TestConnection {
            provider,
            project_dir,
        } => handle_test_connection(&provider, &project_dir),
        Command::ConfigureProvider {
            provider_id,
            name,
            endpoint,
            default_model,
            credential_store,
            credential_key,
            extra_header,
            project_dir,
        } => handle_configure_provider(
            &provider_id,
            &name,
            endpoint,
            default_model,
            credential_store,
            credential_key,
            extra_header,
            &project_dir,
        ),
        Command::AssetParse { id, db } => handle_asset_parse(id, db),
        Command::AssetFacts { id, kind, db } => handle_asset_facts(id, kind, db),
        Command::AssetCompare { id, class, db } => handle_asset_compare(id, class, db),
        Command::AssetVerifyIdentity { id, db } => handle_asset_verify_identity(id, db),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
