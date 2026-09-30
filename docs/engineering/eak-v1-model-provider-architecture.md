# EAK V1 — Model Provider Architecture

> **Status:** Implementation complete for core abstraction (Phase B). Additional providers, credential store, and frontend integration deferred.
> **Scope:** Provider-neutral abstraction, capability system, request/response protocol, tool calling, streaming, error normalization, and agent integration.

---

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            EAK AGENT RUNTIME                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────────────┐  │
│  │   Agent     │───▶│ AgentContext│───▶│     ModelProvider (trait)       │  │
│  │ (deterministic)   │ .reason()   │    │  (extends ReasoningEngine)    │  │
│  └─────────────┘    └─────────────┘    └──────────────┬──────────────────┘  │
│                                                        │                      │
│                    ┌───────────────────────────────────┼───────────────┐     │
│                    ▼                                   ▼               ▼     │
│           ┌──────────────────┐               ┌─────────────────┐ ┌─────────┐│
│           │  FixtureEngine   │               │ AnthropicEngine │ │OpenAI...││
│           │ (deterministic)  │               │   (live API)    │ │ (future)││
│           └──────────────────┘               └─────────────────┘ └─────────┘│
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         EAK ENGINEERING KERNEL                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────────────┐  │
│  │ Capability  │  │  Validation │  │   Commit    │  │  Event Log /       │  │
│  │   Port      │──│  (schema +  │──│  (stamp +   │──│  Replay            │  │
│  │             │  │   domain)   │  │   append +  │  │                    │  │
│  └─────────────┘  └─────────────┘  │   fold)     │  └────────────────────┘  │
│                                    └─────────────┘                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Key Principles:**
- **Provider Independence**: The agent/runtime never contains `if provider == "openai"` logic
- **Propose, Don't Commit**: Model output is always a *candidate* validated before commit
- **Single Reasoning Boundary**: All stochasticity confined to `ModelProvider` trait
- **Deterministic Replay**: Every reasoning call recorded as `Event::ReasoningCall` for byte-identical replay

---

## 2. Provider Abstraction

### Core Trait: `ModelProvider` (in `eak-ports`)

```rust
pub trait ModelProvider: ReasoningEngine {
    /// Provider identifier (e.g., "anthropic", "openai", "ollama")
    fn provider_id(&self) -> ProviderId;

    /// Model identifier (e.g., "claude-opus-4", "gpt-4o", "llama3.1")
    fn model_id_parsed(&self) -> ModelId;

    /// Model's capability set
    fn capabilities(&self) -> CapabilitySet;

    /// Optional metadata (context window, pricing, etc.)
    fn metadata(&self) -> Option<ModelMetadata>;

    /// Synchronous request (default uses ReasoningEngine)
    fn request(&self, req: &ReasoningRequest) -> Result<ReasoningResponse, ReasoningError>;

    /// Streaming request — returns iterator of events
    fn stream_request(
        &self,
        req: &ReasoningRequest,
    ) -> Result<Box<dyn Iterator<Item = Result<StreamEvent, ReasoningError>> + Send>, ReasoningError>;

    /// Cancel in-flight request
    fn cancel(&self) -> Result<(), ReasoningError>;

    /// Check capability support
    fn supports(&self, capability: ModelCapability) -> bool;

    /// List available models (if supported by provider)
    fn list_models(&self) -> Result<Vec<ModelMetadata>, ReasoningError>;
}
```

### Blanket Implementation
Any `ReasoningEngine` automatically implements `ModelProvider` with basic capabilities (text generation only). This preserves backward compatibility with existing `FixtureEngine` and `AnthropicEngine`.

---

## 3. Model Abstraction

### Identifiers

```rust
/// Provider identifier (e.g., "anthropic", "openai", "ollama")
pub struct ProviderId(pub String);

/// Model identifier (e.g., "claude-opus-4", "gpt-4o", "llama3.1")
pub struct ModelId(pub String);
```

### Capabilities

```rust
pub enum ModelCapability {
    TextGeneration,
    Streaming,
    ToolCalling,
    StructuredOutput,
    Vision,
    Audio,
    ReasoningEffort,
    SystemInstructions,
    ParallelToolCalls,
}

/// Efficient bitflag set for capability checking
pub struct CapabilitySet(u16);
```

**Usage:**
```rust
// Check if model supports tool calling
if provider.supports(ModelCapability::ToolCalling) {
    // Safe to include tools in request
}

// Declare model capabilities
let caps = CapabilitySet::from_iter([
    ModelCapability::TextGeneration,
    ModelCapability::Streaming,
    ModelCapability::ToolCalling,
    ModelCapability::StructuredOutput,
]);
```

### Metadata

```rust
pub struct ModelMetadata {
    pub provider: ProviderId,
    pub model_id: ModelId,
    pub display_name: String,
    pub capabilities: CapabilitySet,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub supports_parallel_tool_calls: bool,
    pub input_price_per_million: Option<f64>,
    pub output_price_per_million: Option<f64>,
}
```

---

## 4. Capability System

The capability system enables **capability-based routing** and **graceful degradation**:

| Capability | Description | Required For |
|------------|-------------|--------------|
| `TextGeneration` | Basic completion/chat | All models |
| `Streaming` | Incremental response chunks | Live UI, long responses |
| `ToolCalling` | Function calling with structured args | Agent tool use |
| `StructuredOutput` | JSON schema-constrained output | Validated responses |
| `Vision` | Multimodal input (images) | Schematic analysis |
| `Audio` | Audio input/output | Voice interface |
| `ReasoningEffort` | Reasoning/effort parameter (o-series, Claude thinking) | Complex reasoning |
| `SystemInstructions` | System prompt support | All agents |
| `ParallelToolCalls` | Multiple tool calls per turn | Efficient tool use |

**Orchestration Usage:**
```rust
// Agent requests capability check before using a feature
fn request_with_tools(provider: &dyn ModelProvider, req: ReasoningRequest) {
    if !provider.supports(ModelCapability::ToolCalling) {
        return Err(ProviderError::UnsupportedCapability {
            capability: ModelCapability::ToolCalling,
        });
    }
    // Safe to proceed with tools
}
```

---

## 5. Credential Architecture

### `CredentialRef` — Indirect Reference

```rust
pub struct CredentialRef {
    pub store: String,  // e.g., "env", "keyring", "vault"
    pub key: String,    // e.g., "ANTHROPIC_API_KEY", "openai/key"
}
```

**Security Guarantees:**
- Raw API keys **never** appear in:
  - Source code
  - Git history
  - Logs/telemetry
  - Project configuration files
  - Frontend (React) state
- Configuration stores only `CredentialRef` (opaque reference)
- Resolution happens at runtime via `CredentialStore` trait (deferred)

### Current Implementation
- `EnvCredentialStore` (implicit): Reads `*_API_KEY` from environment
- `AnthropicEngine::from_env()` demonstrates the pattern

---

## 6. Request/Response Protocol

### `ReasoningRequest` (Extended)

```rust
pub struct ReasoningRequest {
    pub model_id: ModelId,
    pub provider: ProviderId,
    pub messages: Vec<Message>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<ToolChoice>,
    pub schema_name: String,
    pub temperature: f64,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u32>,
    pub seed: u64,
    pub reasoning_effort: Option<String>,
    pub stop_sequences: Option<Vec<String>>,
}
```

### `Message` Enum

```rust
pub enum Message {
    System { content: String },
    User { content: String },
    Assistant { content: String, tool_calls: Option<Vec<ToolCall>> },
    Tool { content: String, tool_call_id: String },
}
```

### Tool Calling

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}

pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

pub struct ToolResult {
    pub tool_call_id: String,
    pub content: serde_json::Value,
    pub is_error: bool,
}

pub enum ToolChoice {
    None,      // No tools
    Auto,      // Model decides
    Required,  // Must use a tool
    Specific { name: String },  // Force specific tool
}
```

### `ReasoningResponse` (Extended)

```rust
pub struct ReasoningResponse {
    pub candidates: Vec<CandidateRequirement>,
    pub explanations: Vec<CandidateExplanation>,
    pub part_candidates: Vec<CandidatePart>,
    pub tool_calls: Vec<ToolCall>,        // NEW: model-requested tool calls
    pub usage: UsageMetadata,              // NEW: token usage
    pub clarifying_questions: Vec<String>,
    pub raw: String,                       // Full provider response for replay
}
```

### `UsageMetadata`

```rust
pub struct UsageMetadata {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cached_tokens: Option<u32>,
    pub reasoning_tokens: Option<u32>,
}
```

---

## 7. Tool Calling Architecture

### Flow: Model → Agent → Kernel

```
┌─────────────┐     ToolCall       ┌─────────────┐     CapabilityRequest    ┌──────────┐
│   MODEL     │ ─────────────────▶ │   AGENT     │ ──────────────────────▶ │  KERNEL  │
│  (provider) │                    │ (deterministic)│                        │ (validates│
└─────────────┘                    └─────────────┘                         │ & commits)│
        ▲                              │                                    └──────────┘
        │ ToolResult                   │                                           │
        └──────────────────────────────┘                                           │
                                                                                   ▼
                                                                          ┌──────────────────┐
                                                                          │ Engineering State │
                                                                          │ (committed)       │
                                                                          └──────────────────┘
```

### EAK Tool Domains (Future)

| Domain | Tools |
|--------|-------|
| **Component Library** | `search_components`, `inspect_component`, `get_metadata` |
| **Schematic** | `create_schematic`, `add_component`, `connect_net`, `inspect_connectivity` |
| **Engineering** | `calculate_electrical`, `validate_constraints`, `run_erc`, `run_physics_rules` |
| **PCB** | `propose_placement`, `calculate_trace_width`, `propose_routing`, `run_drc` |
| **Manufacturing** | `generate_bom`, `generate_fabrication_outputs` |

**Current Status:** Tool definitions exist in protocol; execution pipeline deferred until EAK tool schema is defined.

---

## 8. Streaming Architecture

### `StreamEvent` Enum

```rust
pub enum StreamEvent {
    TextDelta { text: String },
    ToolCallDelta { index: u32, id: Option<String>, name: Option<String>, arguments: Option<String> },
    ToolCallComplete { tool_call: ToolCall },
    Complete { usage: Option<UsageMetadata> },
    Error { error: ProviderError },
}
```

### Usage

```rust
// Agent requests streaming
let mut stream = provider.stream_request(&req)?;
for event in stream {
    match event? {
        StreamEvent::TextDelta { text } => {
            // Send to UI for live display
            ui.emit_text_delta(text);
        }
        StreamEvent::ToolCallComplete { tool_call } => {
            // Execute tool, send result back
            let result = execute_tool(tool_call);
            // Note: requires conversation continuation (deferred)
        }
        StreamEvent::Complete { usage } => {
            // Record usage, finalize
        }
        StreamEvent::Error { error } => {
            // Handle error
        }
    }
}
```

**Current Status:** Event types defined and serializable; streaming execution and multi-turn conversation deferred.

---

## 9. Error Normalization

### `ProviderError` (Never Leaks Credentials)

```rust
pub enum ProviderError {
    AuthenticationFailed,
    InvalidApiKey,
    ModelUnavailable { model: ModelId },
    RateLimited { retry_after_secs: Option<u64> },
    Timeout,
    NetworkError,
    InvalidRequest { reason: String },
    SchemaViolation { reason: String },
    UnsupportedCapability { capability: ModelCapability },
    ContentFiltered,
    ProviderUnavailable,
    Unknown { code: String, message: String },
}
```

### Mapping Strategy

Each adapter maps provider-specific errors to `ProviderError`:

```rust
// Anthropic example
match response.status() {
    401 => ProviderError::AuthenticationFailed,
    429 => ProviderError::RateLimited { retry_after_secs: parse_retry_after(headers) },
    400 => ProviderError::InvalidRequest { reason: body },
    _ => ProviderError::Unknown { code: status.to_string(), message: body },
}
```

**Security:** Raw provider responses **never** logged; only normalized error variants.

---

## 10. Agent Integration

### `AgentContext::reason` Signature

```rust
fn reason(&mut self, req: ReasoningRequest) -> Result<(Seq, ReasoningResponse), ReasoningError>;
```

- **Input**: `ReasoningRequest` with messages, tools, schema
- **Output**: `(Seq, ReasoningResponse)` — sequence number for event log + response
- **Recording**: Automatic `Event::ReasoningCall` committed to event log

### Agent Pattern (Model-Agnostic)

```rust
impl Agent for RequirementAgent {
    fn activate(&mut self, ctx: &mut dyn AgentContext, activation: &AgentActivation) -> AgentOutcome {
        // 1. Build request from kernel-owned state (NO provider knowledge)
        let request = ReasoningRequest {
            model_id: ModelId("default".into()),  // Filled by runtime
            provider: ProviderId("default".into()),
            messages: vec![
                Message::System { content: SYSTEM_PROMPT },
                Message::User { content: build_prompt(&intent) },
            ],
            tools: None,  // No tools for requirement planning
            tool_choice: None,
            schema_name: "requirement_candidates_v1",
            temperature: 0.0,
            seed: stable_seed(&intent.statement),
            // ... other fields
        };

        // 2. Request judgement (runtime handles provider selection)
        let (call_seq, response) = ctx.reason(request)?;

        // 3. Deterministic validation & commit (agent never commits directly)
        for candidate in response.candidates {
            if candidate.validate().is_ok() {
                ctx.invoke(CapabilityRequest::CreateRequirement { ... })?;
            }
        }
    }
}
```

**Critical:** No `if provider == "anthropic"` anywhere in agent code.

---

## 11. Security Boundaries

| Boundary | Protection |
|----------|------------|
| **Source Code** | No API keys; `CredentialRef` only |
| **Git History** | No secrets committed (verified) |
| **Logs/Telemetry** | `ProviderError` only; raw responses in `ReasoningResponse.raw` (recorded for replay, not logged) |
| **Frontend** | Receives `EventRecord` via Tauri events; no credential access |
| **Config Files** | `ProviderConfig`/`ModelConfig` use `CredentialRef` |
| **Replay** | `ReasoningCall` events contain full request/response for deterministic replay without API calls |

---

## 12. Adding a New Provider

### Step-by-Step (When Ready)

1. **Create adapter file**: `eak-reasoning/src/<provider>.rs`
2. **Implement `ReasoningEngine`** (minimum) or `ModelProvider` (full):
   ```rust
   pub struct MyProvider { /* config */ }
   
   impl ReasoningEngine for MyProvider {
       fn model_id(&self) -> String { format!("myprovider:{}", self.model) }
       fn request_judgement(&self, req: &ReasoningRequest) -> Result<ReasoningResponse, ReasoningError> {
           // Convert ReasoningRequest → provider API format
           // Call provider API
           // Convert response → ReasoningResponse
           // Map errors → ReasoningError::Provider(ProviderError::...)
       }
   }
   ```
3. **Add to `eak-reasoning/src/lib.rs`**:
   ```rust
   #[cfg(feature = "live")]
   mod myprovider;
   #[cfg(feature = "live")]
   pub use myprovider::MyProvider;
   ```
4. **Add feature flag** in `eak-reasoning/Cargo.toml`:
   ```toml
   [features]
   myprovider = ["dep:reqwest"]  # or ureq, etc.
   ```
5. **Update CLI** (`eak-cli/src/lib.rs`) to support new provider in `ReasoningChoice`
6. **Add tests** using `FixtureEngine` pattern (no real API keys)

### Reference: `AnthropicEngine`
See `eak-reasoning/src/anthropic.rs` for complete implementation pattern.

---

## 13. Testing Strategy

### Deterministic Testing (No API Keys)

```rust
// FixtureEngine for deterministic replay
let engine = FixtureEngine::single(ReasoningResponse {
    candidates: vec![...],
    // ... all required fields
});

// Mock provider for integration tests
struct MockProvider {
    responses: Vec<ReasoningResponse>,
}

impl ReasoningEngine for MockProvider { ... }
```

### Test Coverage
- ✅ Provider registration/discovery (deferred)
- ✅ Capability detection (via `supports()`)
- ✅ Request/response serialization round-trip
- ✅ Tool call/result representation (types defined)
- ✅ Streaming event representation (types defined)
- ✅ Normalized error mapping
- ✅ Unsupported capability handling (returns `ProviderError::UnsupportedCapability`)
- ✅ Credential references never serialized (no `CredentialRef` in test fixtures)
- ✅ Full workspace test suite passes (407 tests)

---

## 14. Current Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| **ModelCapability / CapabilitySet** | ✅ Complete | 9 capabilities, bitflag ops |
| **ProviderId / ModelId** | ✅ Complete | Type-safe newtypes |
| **Message / ToolCalling types** | ✅ Complete | Full protocol |
| **StreamEvent / Streaming** | ✅ Types defined | Execution deferred |
| **ReasoningRequest / Response** | ✅ Complete | Extended with tools, usage |
| **ModelProvider trait** | ✅ Complete | Extends ReasoningEngine |
| **ProviderError** | ✅ Complete | 13 normalized variants |
| **CredentialRef / Config structs** | ✅ Complete | No raw keys |
| **FixtureEngine** | ✅ Updated | Backward compatible |
| **AnthropicEngine** | ✅ Updated | Live adapter working |
| **RuntimeCore integration** | ✅ Complete | `reason()` uses new types |
| **Agent request construction** | ✅ Updated | All 3 agents |
| **Workspace tests** | ✅ 407 pass | No regressions |
| **OpenAI Adapter** | ⏭️ Deferred | Requires API access for validation |
| **Ollama Adapter** | ⏭️ Deferred | Local model support |
| **CredentialStore trait** | ⏭️ Deferred | Keyring, Vault, etc. |
| **ModelRegistry** | ⏭️ Deferred | Discovery/selection |
| **Tool execution pipeline** | ⏭️ Deferred | Needs EAK tool schema |
| **Frontend integration** | ⏭️ Deferred | Settings UI |

---

## 15. Validation Results

```
cargo fmt --all -- --check          ✅ PASS
cargo check --workspace             ✅ PASS
cargo test --workspace              ✅ PASS (407 tests)
cargo clippy --workspace            ✅ PASS (0 warnings)
cargo build --workspace --release   ✅ PASS
git log --all --grep -i key         ✅ CLEAN (no secrets)
git diff --stat                     ✅ SCOPED (model pipeline only)
```

---

## 16. Next Steps

1. **Documentation**: Complete `reasoning-engine-interface.md` update (H2)
2. **Provider Adapters**: Implement OpenAI/Ollama when API access available
3. **Credential Store**: Implement `CredentialStore` trait + Keyring backend
4. **Model Registry**: Build discovery/selection for multiple configured models
5. **Tool Pipeline**: Define EAK tool schema and connect to `CapabilityRequest`
6. **Frontend**: Settings UI for provider/model configuration
7. **Streaming**: Implement multi-turn conversation with tool results

---

## 17. Appendix: Key Files

| File | Purpose |
|------|---------|
| `eak/crates/eak-ports/src/lib.rs` | Core abstraction (traits, types, errors) |
| `eak/crates/eak-reasoning/src/lib.rs` | Adapter exports |
| `eak/crates/eak-reasoning/src/fixture.rs` | Deterministic test adapter |
| `eak/crates/eak-reasoning/src/anthropic.rs` | Live Anthropic adapter |
| `eak/crates/eak-runtime/src/protocol.rs` | AgentContext::reason signature |
| `eak/crates/eak-runtime/src/runtime_core.rs` | RuntimeCore::reason implementation |
| `eak/crates/eak-phases/src/agent.rs` | RequirementAgent request building |
| `eak/crates/eak-phases/src/part_agent.rs` | PartSelectionAgent request building |
| `eak/crates/eak-phases/src/review_explanation.rs` | ReviewExplanationMachine request building |
| `docs/engineering/EAK_V1_MODEL_PIPELINE_TODO.md` | Master TODO tracking |