//! BOM Planning state machine (instance) — REASONING-DRIVEN part selection (E6 C2).
//!
//! It turns the realized schematic into a procurable bill of materials: every
//! [`Component`](eak_domain::Component) is grouped by [`ComponentClass`], each class is resolved to
//! a concrete [`Part`](eak_domain::Part), and a [`BomLineItem`](eak_domain::BomLineItem) binds that
//! part to the components it covers (P13: nothing ships unsourced). The part CHOICE is now proposed
//! by the model (`part_candidates_v1`) and re-validated against the authoritative
//! [`PartCatalog`](eak_engines::PartCatalog): the [`PartSelectionAgent`] asks
//! `ctx.reason`, then commits — through the ordinary capability seam — only proposals the catalog
//! actually carries, REJECTING any hallucinated MPN (which the `bom-coverage` gate then surfaces).
//! This is the SECOND phase to reason (after Requirement Planning), and the moat holds: the model
//! only *chooses* among catalogued parts, the committed part is built from the trusted catalog
//! record, and a rejected proposal never enters state (see `part_agent.rs`).
//!
//! DETERMINISM/REPLAY (P4): the reasoning call is recorded, and a class the model is silent about
//! (the offline default cassette, or a `FixtureEngine::single` scenario) falls back to the exact
//! pre-C2 catalog selection — so an offline run remains bit-identical. It is idempotent: if line
//! items already exist (e.g. on a BOM-loop back) it only (re)projects the [`BomIr`] and re-emits the
//! milestone, committing nothing new (and making no fresh reasoning call). See
//! `docs/state-machines/bom-planning.md`.

use crate::part_agent::PartSelectionAgent;
use eak_compiler::{BomIr, EngineeringIr, RequirementIr, SchematicIr};
use eak_ports::Event;
use eak_runtime::{
    Agent, AgentActivation, AgentContext, AgentOutcome, Budget, Machine, MachineError, StepResult,
};

pub struct BomPlanningMachine {
    agent: PartSelectionAgent,
}

impl BomPlanningMachine {
    pub fn new() -> Self {
        Self {
            agent: PartSelectionAgent::new(),
        }
    }
}
impl Default for BomPlanningMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl Machine for BomPlanningMachine {
    fn name(&self) -> &str {
        "BomPlanning"
    }

    fn initial(&self) -> String {
        "Idle".into()
    }

    fn step(
        &mut self,
        state: &str,
        ctx: &mut dyn AgentContext,
    ) -> Result<StepResult, MachineError> {
        match state {
            "Idle" => Ok(StepResult::Continue("Planning".into())),

            "Planning" => {
                // Idempotent: synthesize the BOM only once. A re-entry (BOM loop-back) skips
                // synthesis (and its reasoning call) and just (re)projects the IR below. Delegate
                // part selection to the reasoning-driven agent: it proposes (via `ctx.reason`),
                // re-validates every proposal against the catalog, and commits the survivors
                // through the seam — rejected proposals never enter state (the moat, P3).
                if ctx.bom_line_items().is_empty() {
                    let activation = AgentActivation {
                        phase: "BomPlanning".into(),
                        goal: "select an approved catalog part for each component class".into(),
                        budget: Budget {
                            max_reasoning_calls: 1,
                        },
                    };
                    match self.agent.activate(ctx, &activation) {
                        AgentOutcome::Success { .. } => {}
                        AgentOutcome::NeedsHuman(m) => {
                            return Ok(StepResult::Failed(format!(
                                "needs human: {m} (HITL deferred)"
                            )))
                        }
                        AgentOutcome::Failed(m) => return Ok(StepResult::Failed(m)),
                    }
                }

                // Project the BOM IR (P6) and record the boundary milestone. The projection
                // re-asserts procurement integrity: every line orders a known part, covers only
                // real components, and every component is covered (IrError -> Internal).
                let bom = project_bom(ctx)?;
                ctx.emit(vec![Event::BomIrProduced {
                    schema_version: bom.schema_version,
                    line_item_count: bom.line_items.len(),
                }])
                .map_err(|e| MachineError::Internal(e.to_string()))?;
                Ok(StepResult::Done)
            }

            other => Err(MachineError::Internal(format!("unknown state {other}"))),
        }
    }
}

/// Project canonical state into the [`BomIr`] through the full lowering chain (Requirement IR
/// -> Engineering IR -> Schematic IR -> BOM IR), mapping any `IrError` to an internal machine
/// error. Shared by the synthesis path and the idempotent re-projection path.
fn project_bom(ctx: &mut dyn AgentContext) -> Result<BomIr, MachineError> {
    let intent = ctx
        .design_intent()
        .ok_or_else(|| MachineError::Internal("no intent for lowering".into()))?;
    let reqs = ctx.requirements();
    let links = ctx.provenance_links();
    let blocks = ctx.functional_blocks();
    let constraints = ctx.constraints();
    let components = ctx.components();
    let pins = ctx.pins();
    let nets = ctx.nets();
    let parts = ctx.parts();
    let line_items = ctx.bom_line_items();
    let req_ir = RequirementIr::project(&intent, &reqs, &links)
        .map_err(|e| MachineError::Internal(e.to_string()))?;
    let eng_ir = EngineeringIr::project(&req_ir, &blocks, &constraints)
        .map_err(|e| MachineError::Internal(e.to_string()))?;
    let sch_ir = SchematicIr::project(&eng_ir, &components, &pins, &nets)
        .map_err(|e| MachineError::Internal(e.to_string()))?;
    BomIr::project(&sch_ir, &parts, &line_items).map_err(|e| MachineError::Internal(e.to_string()))
}
