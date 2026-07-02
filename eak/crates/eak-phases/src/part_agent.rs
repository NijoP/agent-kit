//! The Part-Selection Agent (E6 C2) — the SECOND phase to reason, mirroring [`RequirementAgent`].
//!
//! The reasoning adapter (the `ctx.reason` call) asks the model to PROPOSE a manufacturer part
//! number (MPN) for each component class in the realized schematic — *judgement only*. The
//! deterministic use-case (everything around it) then VALIDATES each proposal against the
//! authoritative [`PartCatalog`] and commits only what survives, through the ordinary
//! [`CapabilityRequest`] seam (P3). This is the moat made concrete for part selection:
//!
//!   * The model NEVER writes state. Its `part_candidates_v1` response is untrusted text.
//!   * For each class, the agent looks up the catalog's part for that class. A proposal is
//!     ACCEPTED iff its MPN equals the catalogued MPN — i.e. the model correctly identified a part
//!     the catalog actually carries. A proposal whose MPN is NOT in the catalog for that class is
//!     REJECTED: nothing is committed for it, and the phase FAILS with a diagnostic naming the
//!     refused MPN (it neither forces the bad part in NOR silently substitutes the catalogued one,
//!     either of which would hide the moat), so the design cannot release with an unsourced class.
//!   * Even an ACCEPTED proposal only *chooses* among catalogued parts: the committed [`Part`] is
//!     built from the trusted [`CatalogPart`] record, never from the model's free text, so model
//!     output can never reach engineering state through this path.
//!   * A class the model is SILENT about falls back to the deterministic catalog selection (exactly
//!     the pre-C2 behaviour), so an offline/empty reasoning response degrades to the old
//!     bit-identical path rather than failing.
//!
//! See `docs/state-machines/bom-planning.md` and the reference agent `agent.rs`.

use eak_domain::{BomLineItem, ComponentClass, EntityId, Part, ProvenanceLink, RelationType};
use eak_engines::{CatalogPart, PartCatalog};
use eak_ports::ReasoningRequest;
use eak_runtime::{Agent, AgentActivation, AgentContext, AgentOutcome, CapabilityRequest};

/// The outcome of validating one class's proposed part against the catalog.
enum Selection {
    /// The catalogued part to commit — either a model proposal that matched the catalog, or the
    /// deterministic fallback for a class the model was silent about.
    Use(CatalogPart),
    /// The model proposed a part the catalog does not carry for this class: REJECTED, commit
    /// nothing. Carries the offending MPN so the phase can surface a diagnostic.
    Reject(String),
}

pub struct PartSelectionAgent;

impl PartSelectionAgent {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PartSelectionAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for PartSelectionAgent {
    fn name(&self) -> &str {
        "PartSelectionAgent"
    }

    fn activate(
        &mut self,
        ctx: &mut dyn AgentContext,
        _activation: &AgentActivation,
    ) -> AgentOutcome {
        let components = ctx.components();
        if components.is_empty() {
            // Nothing to source (e.g. an import-only design) — a clean no-op.
            return AgentOutcome::Success { committed: 0 };
        }

        // Group components by class in first-appearance order — a deterministic pass so the minted
        // line items (and their ids) are reproducible (P4), identical to the pre-C2 machine.
        let mut groups: Vec<(ComponentClass, Vec<EntityId>)> = Vec::new();
        for component in &components {
            if let Some(group) = groups
                .iter_mut()
                .find(|(class, _)| *class == component.class)
            {
                group.1.push(component.id);
            } else {
                groups.push((component.class, vec![component.id]));
            }
        }

        let catalog = PartCatalog::new();

        // ---- reasoning half: ask the model to propose an MPN per class (judgement only) ----
        let classes: Vec<ComponentClass> = groups.iter().map(|(c, _)| *c).collect();
        let request = ReasoningRequest {
            model_id: String::new(), // filled in by the runtime with the real engine id
            system: "You are a procurement engineer. For each component class, propose the \
                     manufacturer part number of an APPROVED-CATALOG part that realizes it. \
                     A part number that is not in the approved catalog will be rejected."
                .into(),
            prompt: build_prompt(&classes),
            schema_name: "part_candidates_v1".into(),
            temperature: 0.0,
            seed: stable_seed(&classes),
        };
        // The reasoning call is recorded (a `ReasoningCall` event) so the run replays without the
        // model (P4). A reasoning engine that is unavailable/empty leaves `part_candidates` empty,
        // and every class then falls back to the deterministic catalog path below.
        let response = match ctx.reason(request) {
            Ok((_call_seq, response)) => response,
            Err(_) => Default::default(), // degrade to pure catalog selection (offline path).
        };

        // ---- deterministic half: validate each proposal against the catalog, then commit ----
        // Cache of MPN -> committed part id, seeded with any already-committed parts, so a part
        // reused across classes (same MPN) is ordered once (P13) — as in the pre-C2 machine.
        let mut part_ids: Vec<(String, EntityId)> =
            ctx.parts().iter().map(|p| (p.mpn.clone(), p.id)).collect();
        let mut committed = 0usize;
        let mut rejected: Vec<(ComponentClass, String)> = Vec::new();

        for (class, comp_ids) in &groups {
            match validate_selection(&catalog, *class, &response) {
                Selection::Reject(mpn) => {
                    // The moat: a non-catalog proposal is refused. Commit NOTHING for this class;
                    // the phase fails (below) with a diagnostic. The bad MPN never reaches state.
                    rejected.push((*class, mpn));
                    continue;
                }
                Selection::Use(cp) => {
                    // Dedup the part by MPN: reuse an existing one, else mint and commit it. The
                    // committed part is built from the TRUSTED catalog record, not the model text.
                    let part_id = if let Some((_, id)) =
                        part_ids.iter().find(|(mpn, _)| mpn.as_str() == cp.mpn)
                    {
                        *id
                    } else {
                        let pid = ctx.fresh_id();
                        let part = Part {
                            id: pid,
                            mpn: cp.mpn.into(),
                            manufacturer: cp.manufacturer.into(),
                            lifecycle: cp.lifecycle,
                            datasheet: cp.datasheet.into(),
                        };
                        // Through the real capability seam: the runtime re-validates the part (P3).
                        if ctx
                            .invoke(CapabilityRequest::CreatePart {
                                part,
                                links: vec![],
                            })
                            .is_err()
                        {
                            // Seam rejection (e.g. a malformed part): treat as a rejected proposal
                            // rather than forcing it in — the moat holds at the seam too.
                            rejected.push((*class, cp.mpn.to_string()));
                            continue;
                        }
                        part_ids.push((cp.mpn.to_string(), pid));
                        pid
                    };

                    // The line binds the part to every component of this class, with provenance to
                    // the part it orders AND each component it covers (P3), through the seam.
                    let item_id = ctx.fresh_id();
                    let item = BomLineItem {
                        id: item_id,
                        part: part_id,
                        components: comp_ids.clone(),
                        quantity: comp_ids.len() as u32,
                    };
                    let mut links = vec![ProvenanceLink {
                        id: ctx.fresh_id(),
                        from: item_id,
                        to: part_id,
                        relation: RelationType::TracesTo,
                    }];
                    for cid in comp_ids {
                        links.push(ProvenanceLink {
                            id: ctx.fresh_id(),
                            from: item_id,
                            to: *cid,
                            relation: RelationType::TracesTo,
                        });
                    }
                    if ctx
                        .invoke(CapabilityRequest::CreateBomLineItem { item, links })
                        .is_ok()
                    {
                        committed += 1;
                    }
                }
            }
        }

        // A rejected proposal is a surfaced FAILURE, never a silent correction: the model named a
        // part the catalog does not carry, so the class cannot be sourced and the phase reports it
        // as a diagnostic rather than either forcing the bad part in OR quietly substituting the
        // catalogued one (which would hide the moat). The survivors we DID commit remain valid; the
        // phase still fails so the design cannot release with an unsourceable class. This is the
        // moat in one line: the LLM proposed; the kernel validated; the bad proposal was refused
        // and never reached state.
        if !rejected.is_empty() {
            let detail = rejected
                .iter()
                .map(|(c, mpn)| format!("{c:?} <- \"{mpn}\""))
                .collect::<Vec<_>>()
                .join(", ");
            return AgentOutcome::Failed(format!(
                "rejected {} non-catalog part proposal(s) (not in the approved catalog): {detail}",
                rejected.len()
            ));
        }

        AgentOutcome::Success { committed }
    }
}

/// Validate the model's proposal for one class against the authoritative catalog. The catalog is
/// the single source of truth: the class's catalogued part is looked up, then the proposal (if any)
/// is checked against it. No proposal -> deterministic fallback; a matching proposal -> accepted;
/// a non-matching proposal -> REJECTED (the moat).
fn validate_selection(
    catalog: &PartCatalog,
    class: ComponentClass,
    response: &eak_ports::ReasoningResponse,
) -> Selection {
    let catalogued = catalog.part_for(class);
    match response
        .part_candidates
        .iter()
        .find(|p| class_matches(&p.component_class, class))
    {
        // The model proposed a part for this class: accept iff it is the catalogued MPN, else reject.
        Some(proposal) => {
            if proposal.mpn.trim().eq_ignore_ascii_case(catalogued.mpn) {
                Selection::Use(catalogued)
            } else {
                Selection::Reject(proposal.mpn.clone())
            }
        }
        // The model was silent about this class: fall back to the deterministic catalog part.
        None => Selection::Use(catalogued),
    }
}

/// Case-insensitive match of a model-supplied class label against a [`ComponentClass`].
fn class_matches(label: &str, class: ComponentClass) -> bool {
    label.trim().eq_ignore_ascii_case(class_name(class))
}

/// The canonical string name of a [`ComponentClass`], used in the prompt and for matching.
fn class_name(class: ComponentClass) -> &'static str {
    match class {
        ComponentClass::Connector => "Connector",
        ComponentClass::Regulator => "Regulator",
        ComponentClass::Ic => "Ic",
        ComponentClass::Resistor => "Resistor",
        ComponentClass::Capacitor => "Capacitor",
    }
}

fn build_prompt(classes: &[ComponentClass]) -> String {
    let list = classes
        .iter()
        .map(|c| format!("- {}", class_name(*c)))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Propose a manufacturer part number (MPN) from the approved catalog for each of the \
         following component classes. Return one entry per class with its `component_class` and \
         `mpn`. Only approved-catalog parts are accepted; any other MPN will be rejected.\n\n\
         COMPONENT CLASSES:\n{list}\n"
    )
}

/// Deterministic seed from the class list (FNV-1a 64) so a run is reproducible (P4).
fn stable_seed(classes: &[ComponentClass]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for class in classes {
        for b in class_name(*class).as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}
