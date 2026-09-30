# Model/Provider Architecture: Electronics Agent Kit (EAK)

**Version:** 0.1.0-draft
**Status:** Canonical

---

## Overview

This document defines the reasoning provider abstraction, model capabilities, credential handling, structured responses, and agent proposal flow. The kernel never depends on a specific model; all providers implement a common port trait.

---

## Design Principles

1. **Provider Agnosticism** — Kernel sees only the `ReasoningEngine` port trait. Swap NVIDIA, OpenAI, local models without kernel changes.
2. **Structured Output** — All model responses are validated JSON Schema before crossing the capability seam.
3. **Streaming First** — Token streaming for UX; final structured output for validation.
4. **Capability Negotiation** — Session starts with capability discovery (tools, schemas, limits).
5. **Credential Isolation** — Credentials never enter kernel; handled by adapter process.
6. **Deterministic Fixture** — `FixtureReasoningEngine` for testing, replay, CI (zero network).

---

## Port Trait (`eak-ports`)

```rust
#[async_trait]
pub trait ReasoningEngine: Send + Sync {
    /// Negotiate capabilities at session start
    async fn negotiate(&self, caps: ClientCapabilities) -> Result<ServerCapabilities, NegotiationError>;

    /// Single request/response (non-streaming)
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CompletionError>;

    /// Streaming response (SSE)
    async fn stream(&self, req: CompletionRequest) -> Result<StreamingResponse, CompletionError>;

    /// List available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>, CompletionError>;

    /// Health check
    async fn health(&self) -> Result<HealthStatus, CompletionError>;
}
```

### Capability Negotiation

```rust
struct ClientCapabilities {
    max_tokens: u32,
    supports_structured_output: bool,
    supports_tool_calls: bool,
    supports_streaming: bool,
    supported_schemas: Vec<SchemaId>, // Kernel-defined function schemas
}

struct ServerCapabilities {
    model_id: String,
    max_context_tokens: u32,
    max_output_tokens: u32,
    supports_structured_output: bool,
    supports_tool_calls: bool,
    supports_streaming: bool,
    available_schemas: Vec<SchemaId>,
    rate_limits: RateLimits,
}
```

---

## Request/Response Schema

### CompletionRequest
```rust
struct CompletionRequest {
    messages: Vec<Message>,
    model: Option<String>,          // Override default
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    response_format: Option<ResponseFormat>, // JSON Schema
    tools: Option<Vec<ToolDefinition>>,      // Kernel-defined capabilities
    tool_choice: Option<ToolChoice>,
    metadata: RequestMetadata,      // correlation_id, agent_role, phase
}

enum ResponseFormat {
    JsonSchema { schema: serde_json::Value },
    Text,
}

struct ToolDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,  // JSON Schema
}
```

### CompletionResponse
```rust
struct CompletionResponse {
    id: String,
    model: String,
    choices: Vec<Choice>,
    usage: TokenUsage,
    metadata: ResponseMetadata,
}

struct Choice {
    index: u32,
    message: Message,
    finish_reason: FinishReason,    // Stop, ToolCalls, Length, ContentFilter
}

struct Message {
    role: Role,                     // System, User, Assistant, Tool
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

struct ToolCall {
    id: String,
    name: String,
    arguments: String,              // JSON string (validated against schema)
}
```

### StreamingResponse
```rust
// Server-Sent Events (SSE) with data: {delta}
struct StreamingDelta {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCallDelta>>,
    finish_reason: Option<FinishReason>,
}
```

---

## Kernel-Defined Tool Schemas (Capability Seams)

The kernel defines a fixed set of tool schemas that map to capability seams. Models **must** use these schemas for proposals.

| Tool Name | Capability | Parameters Schema | Description |
|-----------|------------|-------------------|-------------|
| `propose_requirements` | `RequirementProposal` | `RequirementProposalSchema` | Propose new/modified requirements |
| `select_components` | `ComponentSelection` | `ComponentSelectionSchema` | Select/suggest components with evidence |
| `edit_schematic` | `SchematicEdit` | `SchematicEditSchema` | Add/modify/delete schematic elements |
| `edit_pcb` | `PcbEdit` | `PcbEditSchema` | Placement, routing, stackup changes |
| `run_verification` | `VerificationRequest` | `VerificationRequestSchema` | Request verification analysis |
| `generate_manufacturing` | `ManufacturingOutput` | `ManufacturingOutputSchema` | Generate export artifacts |
| `sync_kicad` | `KicadSync` | `KicadSyncSchema` | Import/export KiCad project |

Each schema is a JSON Schema document stored in `eak-ports/schemas/`. Versioned with kernel.

---

## Provider Adapters (`eak-reasoning`)

### 1. Fixture Adapter (Deterministic)
- **Purpose**: Testing, replay, CI, air-gapped.
- **Behavior**: Returns pre-recorded responses keyed by request hash.
- **Configuration**: `fixtures/<model_id>/<request_hash>.json`

### 2. NVIDIA NIM Adapter
- **Endpoint**: `https://integrate.api.nvidia.com/v1` (or self-hosted)
- **Auth**: Bearer token (NVIDIA_API_KEY)
- **Features**: Structured output, tool calls, streaming, long context.
- **Models**: Nemotron, Llama, Mistral, etc. via NIM.

### 3. OpenAI-Compatible Adapter
- **Endpoint**: Any OpenAI-compatible `/v1/chat/completions`
- **Auth**: Bearer token (OPENAI_API_KEY or custom)
- **Features**: Depends on endpoint (Ollama, vLLM, TGI, OpenAI, Azure).
- **Auto-detection**: Probes `/v1/models` and test completion.

### 4. Local Adapter (llama.cpp / Ollama)
- **Endpoint**: `http://localhost:11434` (Ollama) or custom
- **Auth**: Optional
- **Features**: Structured output (via grammar/GBNF), tool calls (Ollama 0.2+), streaming.

---

## Credential Handling

- **Never in kernel** — Credentials are read by adapter process from environment/files.
- **Adapter Process Isolation** — Each provider runs in its own process (or thread) with minimal privileges.
- **Credential Rotation** — Adapters watch for env var changes; no kernel restart needed.
- **Audit Log** — Adapter logs credential access (source, time) without values.

### Configuration
```toml
# eak-reasoning config (loaded by adapter, not kernel)
[provider.nvidia]
api_key_env = "NVIDIA_API_KEY"
endpoint = "https://integrate.api.nvidia.com/v1"
default_model = "nvidia/nemotron-3-ultra"

[provider.openai_compatible]
api_key_env = "OPENAI_API_KEY"
endpoint = "https://api.openai.com/v1"
default_model = "gpt-4o"

[provider.ollama]
endpoint = "http://localhost:11434/v1"
default_model = "nemotron-3-ultra"
```

---

## Agent Proposal Flow

```
1. Agent (via phase FSM) prepares CompletionRequest
   - Includes kernel-defined tool schemas
   - Includes context: current state, requirements, verification results
   - metadata.correlation_id = new Uuid

2. Agent calls ReasoningEngine::complete() or ::stream()

3. Model returns CompletionResponse with tool_calls

4. Agent parses tool_calls → validates JSON against kernel schema
   - If invalid: retry with error feedback (bounded retries)

5. Agent submits each tool_call as a CapabilityInvocation to kernel
   - Kernel validates via capability seam (deterministic)
   - If valid: event appended, state updated
   - If invalid: rejection event, agent retries (bounded)

6. Agent receives updated state → next FSM step
```

**Critical**: The model **never** writes state. It only proposes via tool calls. Kernel validates and commits.

---

## Bounded Retries & Circuit Breaker

| Parameter | Default |
|-----------|---------|
| Max retries per proposal | 3 |
| Retry backoff | Exponential (1s, 2s, 4s) |
| Circuit breaker threshold | 5 consecutive failures |
| Circuit breaker reset | 60s |
| Request timeout | 120s (configurable) |

Retries are tracked per `correlation_id`. Exhausted retries → escalate to human (Lead agent).

---

## Structured Output Enforcement

- **JSON Schema** for every tool call (kernel-defined).
- **Response format** enforced via `response_format: {json_schema: ...}`.
- **Validation**: Adapter validates response against schema before returning to agent.
- **Repair**: If validation fails, adapter can request model to retry (once) with error details.

---

## Model Capabilities Registry

```rust
struct ModelInfo {
    id: String,
    provider: ProviderId,
    capabilities: ModelCapabilities,
    context_window: u32,
    max_output: u32,
    pricing: Option<Pricing>,
    recommended_for: Vec<AgentRole>,
}

struct ModelCapabilities {
    structured_output: bool,
    tool_calls: bool,
    streaming: bool,
    vision: bool,
    reasoning: bool,           // Chain-of-thought, hidden reasoning tokens
    json_mode: bool,
}
```

Kernel uses `recommended_for` to route agent roles to appropriate models (e.g., reasoning models for Verification Agent, fast models for Component Agent).

---

## Observability

- **Tracing**: Every request/response traced with `correlation_id`.
- **Metrics**: Latency, token usage, error rate, validation failure rate per model.
- **Cost Tracking**: Token usage × pricing per provider (if available).
- **Audit**: Full request/response logged (opt-in, redacted for secrets).

---

## Next Document

[Agent Architecture →](../7-agent-architecture/README.md)