# Agent Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines the agent roles, proposal/validation loop, capability seams, bounded retries, approval gates, and durable state management for AI agents operating within the EAK kernel.

---

## Agent Roles (V1)

| Role | Responsibility | Capability Seams Used | Model Preference |
|------|----------------|----------------------|------------------|
| **Lead** | Orchestrates workflow, decomposes intent, delegates to specialists, manages approval gates | All (via delegation) | Reasoning model (Nemotron, GPT-4o) |
| **Research** | Gathers datasheets, application notes, standards, competitor info | `ComponentResolution`, `ComponentIngestEvidence` | Fast model with tool use |
| **Component** | Selects components, validates specs, generates assets, manages approvals | `ComponentSelection`, `ComponentValidate`, `ComponentGenerateSymbol`, `ComponentGenerateFootprint`, `ComponentApprove` | Fast model with structured output |
| **Schematic** | Creates/edits schematic, places components, wires nets, runs ERC | `SchematicEdit`, `VerificationRequest(ERC)` | Model with spatial reasoning |
| **PCB** | Places components, routes nets, defines stackup, runs DRC | `PcbEdit`, `VerificationRequest(DRC)` | Model with spatial reasoning |
| **Verification** | Runs power, clock, SI, EMC, DFM analyses; produces confidence/fidelity | `VerificationRequest(Power, Clock, SI, EMC, DFM)` | Reasoning model |
| **Manufacturing** | Generates BOM, Gerber, Drill, PnP; validates against fabricator rules | `ManufacturingOutput` | Fast model |
| **Handoff** | Prepares release package, documentation, sign-off artifacts | `ManufacturingOutput`, `KicadSync` | Fast model |

---

## Agent Framework (`eak-phases`)

Each agent is implemented as a **Phase** — a finite-state machine (FSM) running in the kernel's execution engine.

### Phase Trait
```rust
pub trait Phase: Send + Sync {
    fn phase_id(&self) -> PhaseId;
    fn initial_state(&self) -> PhaseState;
    fn transition(&self, state: PhaseState, event: Event) -> PhaseTransition;
    fn capabilities(&self) -> Vec<CapabilityId>; // Which capability seams this phase uses
}
```

### Phase State Machine
```
PhaseState = {
    status: PhaseStatus,        // Pending, Running, WaitingForProposal, WaitingForValidation, WaitingForApproval, Completed, Failed, Compensating
    context: PhaseContext,      // Input data, intermediate results
    retry_count: u32,
    last_proposal: Option<Proposal>,
    approval_gate: Option<ApprovalGate>,
}
```

### Phase Transitions
1. **Pending → Running** — Kernel schedules phase.
2. **Running → WaitingForProposal** — Agent needs to propose action (calls model).
3. **WaitingForProposal → WaitingForValidation** — Proposal submitted to capability seam.
4. **WaitingForValidation → Running** — Validation passed, state updated, continue.
5. **WaitingForValidation → WaitingForProposal** — Validation failed, agent retries (bounded).
6. **Running → WaitingForApproval** — Human approval gate required.
7. **WaitingForApproval → Running** — Approved (human or auto-approved for low-risk).
8. **Running → Completed** — Phase objectives met.
9. **Any → Failed** — Unrecoverable error (exhausted retries, hard validation failure).
10. **Failed → Compensating** — Run compensation actions (rollback, cleanup).
11. **Compensating → Failed** — Compensation failed (escalate).

---

## Proposal/Validation Loop

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   AGENT     │────►│  KERNEL     │────►│  CAPABILITY │────►│  EVENT LOG  │
│  (Phase)    │     │  ORCHESTRATOR│     │  HANDLER    │     │  (COMMIT)   │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
      ▲                                                                 │
      │                                                                 │
      └──────────────────────── STATE UPDATE ◄─────────────────────────┘
```

### Step-by-Step

1. **Agent prepares proposal** — Based on current state (read via `get_state`), agent constructs a `CapabilityInvocation` (tool call with arguments).
2. **Agent submits to kernel** — `orchestrator.invoke_capability(capability_id, payload, correlation_id)`.
3. **Kernel validates** — Capability handler runs deterministic validation function.
   - **Pass**: Event appended, state folded, `StateDelta` returned.
   - **Fail**: `ValidationError` returned (structured, with fix hints).
4. **Agent receives result** — On pass: continues FSM. On fail: increments retry, adjusts proposal, retries (bounded).
5. **State update** — Agent reads new state via `get_state` or live `EventSink` stream.

---

## Capability Seams (Recap from System Architecture)

Each capability seam is a **validated entry point** with:
- **Input schema** (JSON Schema, kernel-defined)
- **Validation function** (pure, deterministic, in `eak-engines`)
- **State mutation** (event appended on success)

| Capability | Validation Function | Typical Agent |
|------------|---------------------|---------------|
| `RequirementProposal` | `validate_requirement_proposal` | Lead, Research |
| `ComponentSelection` | `validate_component_selection` | Component |
| `SchematicEdit` | `validate_schematic_edit` | Schematic |
| `PcbEdit` | `validate_pcb_edit` | PCB |
| `VerificationRequest` | `validate_verification_request` | Verification |
| `ManufacturingOutput` | `validate_manufacturing_output` | Manufacturing |
| `KicadSync` | `validate_kicad_sync` | Handoff, Schematic, PCB |

---

## Approval Gates

Certain transitions require **human approval** (or Lead agent auto-approval for low-risk).

### Gate Types
| Gate | Trigger | Approver | Criteria |
|------|---------|----------|----------|
| **Component Approval** | Component lifecycle → `Approved` | Lead Agent / Human | All validation passed, supplier stock confirmed |
| **Schematic Sign-off** | Schematic phase → `Completed` | Lead Agent / Human | ERC clean, all requirements traced |
| **PCB Sign-off** | PCB phase → `Completed` | Lead Agent / Human | DRC clean, all constraints met |
| **Verification Release** | Verification phase → `Completed` | Lead Agent / Human | All critical checks pass, confidence > threshold |
| **Manufacturing Release** | Manufacturing phase → `Completed` | Human | Fabricator DFM pass, BOM locked |

### Approval Representation
```rust
struct ApprovalGate {
    id: EntityId,
    phase_id: PhaseId,
    kind: ApprovalKind,
    required_approvers: Vec<Approver>, // Human, LeadAgent
    status: ApprovalStatus,            // Pending, Approved, Rejected
    requested_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
    resolution: Option<ApprovalResolution>,
}
```

---

## Bounded Retries & Compensation

### Retry Policy (per capability)
| Capability | Max Retries | Backoff | Escalation |
|------------|-------------|---------|------------|
| `RequirementProposal` | 3 | 1s, 2s, 4s | Lead Agent |
| `ComponentSelection` | 5 | 1s, 2s, 4s, 8s, 16s | Lead Agent |
| `SchematicEdit` | 3 | 1s, 2s, 4s | Lead Agent |
| `PcbEdit` | 5 | 1s, 2s, 4s, 8s, 16s | Lead Agent |
| `VerificationRequest` | 2 | 5s, 10s | Lead Agent |
| `ManufacturingOutput` | 1 | — | Human |

### Compensation Actions
On phase failure after exhausted retries:
1. **Rollback** — Kernel replays to state before phase started (event log truncation not allowed; instead, compensating events).
2. **Compensating Events** — New events that undo effects (e.g., `ComponentDeselected`, `SchematicElementRemoved`).
3. **Escalation** — Lead Agent notified; human intervention required.

---

## Durable Agent State

All agent state is persisted in the **kernel event log**. No separate agent state store.

### What's Persisted
- Phase FSM state (serialized at each transition)
- Proposals submitted (including rejected ones)
- Validation errors (for learning/debugging)
- Approval gate status
- Model request/response (redacted, opt-in)

### Recovery
On kernel restart:
1. Replay event log → reconstruct all phase states.
2. Resume phases from last `PhaseState`.
3. In-flight proposals re-validated (idempotent).

---

## Agent Communication (Inter-Agent)

Agents **do not communicate directly**. All communication flows through the kernel:

1. **Shared State** — Agents read canonical model via `get_state`.
2. **Events** — Agents subscribe to `EventSink` for relevant events (e.g., Component Agent watches `ComponentSelected`).
3. **Delegation** — Lead Agent spawns child phases via `orchestrator.spawn_phase()`.
4. **Handoff** — Phase completion event triggers next phase (defined in workflow).

---

## Agent Configuration

```toml
# eak-phases config
[agent.lead]
model = "nemotron-3-ultra"
max_concurrent_phases = 3

[agent.component]
model = "gpt-4o-mini"
max_retries = 5
supplier_priority = ["local", "digikey", "mouser", "lcsc"]

[agent.schematic]
model = "nemotron-3-ultra"
grid_mm = 1.27
default_trace_width_mm = 0.2

[agent.pcb]
model = "nemotron-3-ultra"
routing_layers = [1, 2, 15, 16]
via_drill_mm = 0.3

[agent.verification]
model = "nemotron-3-ultra"
confidence_threshold = 0.85
fidelity_requirement = "Simulation"
```

---

## Testing Agents

- **Unit**: Test FSM transitions with mocked kernel.
- **Integration**: Run full phase with `FixtureReasoningEngine` (deterministic).
- **Property**: Fuzz proposals against validation functions.
- **Golden**: Record expert agent runs; replay and assert identical events.

---

## Next Document

[Terminal/Herdr Architecture →](../8-terminal-herdr-architecture/README.md)