# EAK V1 Model Provider Pipeline — Forensic Audit Report

> **Audit Date**: 2025-08-20
> **Auditor**: Terminal 2 (Forensic Engineering Audit)
> **Scope**: Verify whether the repository actually implements the intended provider-neutral architecture

---

## 1. Executive Verdict

**VERDICT: YELLOW**

The foundation compiles and tests pass, but the implementation **does not match the intended architecture** in critical ways:

| Subsystem | Claimed | Actual |
|-----------|---------|--------|
| Multi-provider architecture | ✅ IMPLEMENTED | ❌ NOT IMPLEMENTED (types only, no selection) |
| ModelProvider as provider boundary | ✅ IMPLEMENTED | ⚠️ PARTIAL (blanket impl over ReasoningEngine) |
| Error normalization | ✅ IMPLEMENTED | ❌ DEFECTIVE (HTTP errors → Unknown) |
| Streaming | ✅ IMPLEMENTED | ❌ TYPES ONLY (UnsupportedCapability) |
| Tool calling | ✅ IMPLEMENTED | ❌ TYPES ONLY (no execution pipeline) |
| Credential abstraction | ✅ IMPLEMENTED | ❌ NOT IMPLEMENTED (CredentialRef unused) |
| Multi-provider selection | ✅ IMPLEMENTED | ❌ NOT IMPLEMENTED (Fixture/Live only) |
| Test count | 407 tests | 388 passed + 1 ignored |

**Critical Defects Found**: 7
**False/Overstated Claims**: 11
**Tests Actually Verified**: 388 passed, 1 ignored (live)
**Implementation Changes Made During Audit**: 0

---

## 2. Intended Architecture

From the task requirements and EAK story:

```
Agent
  ↓
ReasoningRequest (messages, tools, tool_choice, schema, params)
  ↓
ModelProvider (trait) ← provider-neutral boundary
  ↓
Provider Adapter (Anthropic, OpenAI, Ollama, etc.)
  ↓
Provider API
  ↓
Response
  ↓
StreamEvent / ToolCall / ToolResult / UsageMetadata
  ↓
Agent (deterministic validation)
  ↓
CapabilityRequest → Kernel → Validation → Commit
```

Key invariants:
- Agent code NEVER contains `if provider == "..."`
- Provider-specific code ONLY in adapters behind `ModelProvider` trait
- Multi-provider selection via registry/configuration
- Credentials via `CredentialRef` → `CredentialStore` (never raw keys in config)

---

## 3. Actual Architecture (Verified)

### 3.1 Call Chain Trace (Verified)

```
Agent (RequirementAgent/PartSelectionAgent/ReviewExplanationMachine)
  → builds ReasoningRequest with messages[], tools, tool_choice
  → AgentContext::reason(ReasoningRequest)
  → RuntimeCore::reason() [eak-runtime/src/runtime_core.rs:1246]
  → self.reasoning.request_judgement(req)  // Box<dyn ReasoningEngine>
  → FixtureEngine::request_judgement() OR AnthropicEngine::request_judgement()
  → ReasoningResponse (candidates, part_candidates, explanations, tool_calls=[], usage=0, raw)
  → Event::ReasoningCall { request, response } committed to event log
```

**File locations:**
- Agent request building: `eak/crates/eak-phases/src/agent.rs:50-60`, `part_agent.rs:88-99`, `review_explanation.rs:48-62`
- AgentContext::reason: `eak/crates/eak-runtime/src/protocol.rs:379`
- RuntimeCore::reason: `eak/crates/eak-runtime/src/runtime_core.rs:1246-1261`
- FixtureEngine: `eak/crates/eak-reasoning/src/fixture.rs:73-86`
- AnthropicEngine: `eak/crates/eak-reasoning/src/anthropic.rs:117-343`

### 3.2 Provider Boundary (Actual)

**RuntimeCore field** (`eak/crates/eak-runtime/src/runtime_core.rs:26`):
```rust
reasoning: Box<dyn ReasoningEngine>,  // NOT ModelProvider
```

**RuntimeCore::reason()** calls `self.reasoning.request_judgement()` directly — **never uses ModelProvider trait methods**.

**ModelProvider trait** (`eak/crates/eak-ports/src/lib.rs:894-945`):
```rust
pub trait ModelProvider: ReasoningEngine {
    fn provider_id(&self) -> ProviderId { ... }
    fn model_id_parsed(&self) -> ModelId { ... }
    fn capabilities(&self) -> CapabilitySet { CapabilitySet::TEXT_GENERATION }
    fn metadata(&self) -> Option<ModelMetadata> { None }
    fn request(&self, req: &ReasoningRequest) -> ... { self.request_judgement(req) }
    fn stream_request(...) -> ... { Err(UnsupportedCapability) }
    fn cancel(&self) -> ... { Err(UnsupportedCapability) }
    fn supports(&self, capability) -> bool { self.capabilities().contains(capability) }
    fn list_models(&self) -> ... { Err(UnsupportedCapability) }
}

/// Blanket implementation: any ReasoningEngine is a ModelProvider
impl<T: ReasoningEngine> ModelProvider for T {}
```

**The blanket implementation `impl<T: ReasoningEngine> ModelProvider for T` makes ModelProvider a mere alias over ReasoningEngine.** It adds zero enforcement — any ReasoningEngine automatically "is a" ModelProvider with default (non-functional) capabilities.

### 3.3 Multi-Provider Selection (Actual)

**CLI Provider Selection** (`eak/crates/eak-cli/src/lib.rs:37-122`):
```rust
pub enum ReasoningChoice { Fixture, Live }  // Only two options

fn build_reasoning(cfg: &RunConfig) -> Result<Box<dyn ReasoningEngine>, CliError> {
    match cfg.reasoning {
        ReasoningChoice::Fixture => Ok(Box::new(FixtureEngine::...)),
        ReasoningChoice::Live => Ok(Box::new(AnthropicEngine::from_env(...))),
    }
}
```

**No provider registry exists.** Search results:
```
$ grep -r "provider.*registry\|provider.*factory\|provider.*select" eak/crates/
eak/crates/eak-ports/src/lib.rs:pub trait ModelProvider: ReasoningEngine {
eak/crates/eak-ports/src/lib.rs:impl<T: ReasoningEngine> ModelProvider for T {}
```

**No `ModelRegistry`, no provider factory, no configuration-driven selection.**

### 3.4 Provider Adapters (Actual)

| Adapter | File | Implements | Status |
|---------|------|------------|--------|
| FixtureEngine | `eak-reasoning/src/fixture.rs` | `ReasoningEngine` | ✅ Working |
| AnthropicEngine | `eak-reasoning/src/anthropic.rs` | `ReasoningEngine` | ✅ Working |
| OpenAIEngine | — | — | ❌ Not created |
| OllamaEngine | — | — | ❌ Not created |
| GeminiEngine | — | — | ❌ Not created |
| KimiEngine | — | — | ❌ Not created |
| NvidiaEngine | — | — | ❌ Not created |

**Only two adapters exist** — both implement `ReasoningEngine` directly. The `ModelProvider` trait is unused by both.

---

## 4. Architecture Gap Analysis

| Intended | Implemented | Gap |
|----------|-------------|-----|
| `Agent → ModelProvider → Adapter` | `Agent → ReasoningEngine → Adapter` | ModelProvider unused in call chain |
| Multi-provider selection | Fixture/Live enum only | No registry, factory, config |
| `CredentialRef` → `CredentialStore` | `CredentialRef` type only | No resolution, no store trait |
| `stream_request()` functional | Returns `UnsupportedCapability` | Types only |
| `tool_calls` execution pipeline | Types defined, no pipeline | No Tool dispatcher |
| Error normalization | `ProviderError` enum | HTTP errors → `Unknown` |
| Provider registry | Types: `ProviderConfig`, `ModelConfig` | No registry, no CLI integration |

---

## 5. ModelProvider Audit

### 5.1 Method Analysis

| Method | Implementation | Functional? | Notes |
|--------|----------------|-------------|-------|
| `provider_id()` | Parses `model_id()` by splitting on `:` | ✅ Parses but fragile | Assumes format `"provider:model"` |
| `model_id_parsed()` | Parses `model_id()` by splitting on `:` | ✅ Parses but fragile | Same fragility |
| `capabilities()` | Returns `CapabilitySet::TEXT_GENERATION` | ⚠️ Hardcoded | Not overridden by adapters |
| `metadata()` | Returns `None` | ❌ Not implemented | Not overridden |
| `request()` | Delegates to `request_judgement()` | ✅ Works | But via ReasoningEngine |
| `stream_request()` | Returns `Err(UnsupportedCapability)` | ❌ Not implemented | Default only |
| `cancel()` | Returns `Err(UnsupportedCapability)` | ❌ Not implemented | Default only |
| `supports()` | Checks `capabilities().contains()` | ⚠️ Works but hardcoded | `TEXT_GENERATION` only |
| `list_models()` | Returns `Err(UnsupportedCapability)` | ❌ Not implemented | Default only |

### 5.2 Blanket Implementation Defect

**File**: `eak/crates/eak-ports/src/lib.rs:945`
```rust
impl<T: ReasoningEngine> ModelProvider for T {}
```

**Impact**: 
- Every `ReasoningEngine` automatically implements `ModelProvider`
- No enforcement that adapters override capability methods
- `AnthropicEngine` and `FixtureEngine` never override `capabilities()`, `metadata()`, `stream_request()`, etc.
- ModelProvider is **not a true provider boundary** — it's a passive extension trait

**Architectural Violation**: The intended architecture requires `ModelProvider` to be the **single boundary** that adapters implement. Instead, adapters implement `ReasoningEngine` and get `ModelProvider` for free with non-functional defaults.

---

## 6. Multi-Provider Execution Audit

### 6.1 Provider Selection Mechanism

**Does not exist.** The only selection is CLI `ReasoningChoice::Fixture | Live`.

### 6.2 Provider Registry

**Does not exist.** No `ModelRegistry` struct, no registration API, no discovery.

### 6.3 Provider Configuration

Types defined but unused:
- `ProviderConfig` (with `credential_ref: Option<CredentialRef>`) — unused
- `ModelConfig` — unused  
- `ModelParameters` — unused

### 6.4 Provider Instantiation

Hardcoded in `eak-cli/src/lib.rs:93-122`:
```rust
match cfg.reasoning {
    ReasoningChoice::Fixture => FixtureEngine,
    ReasoningChoice::Live => AnthropicEngine::from_env(...),
}
```

**No dependency injection, no config-driven selection, no dynamic loading.**

### 6.5 Conclusion

> **Multi-provider abstraction exists, but multi-provider execution does not.**

The types (`ProviderId`, `ModelId`, `ProviderConfig`, `ModelConfig`) are defined but **no executable code** connects them to provider selection or instantiation.

---

## 7. Error Mapping Audit

### 7.1 Claimed Mapping (from previous report)

| HTTP Status | Claimed Mapping |
|-------------|-----------------|
| 401 | AuthenticationFailed |
| 429 | RateLimited |
| 400 | InvalidRequest |

### 7.2 Actual Implementation (`eak/crates/eak-reasoning/src/anthropic.rs:148-176`)

```rust
let value: serde_json::Value = match response {
    Ok(r) => r.into_json().map_err(|e| {
        ReasoningError::Provider(ProviderError::InvalidRequest { reason: e.to_string() })
    })?,
    Err(ureq::Error::Status(code, r)) => {
        let txt = r.into_string().unwrap_or_default();
        return Err(ReasoningError::Provider(
            ProviderError::Unknown { code: code.to_string(), message: txt }
        ));
    }
    Err(e) => {
        return Err(ReasoningError::Provider(ProviderError::NetworkError))
    }
};
```

**Actual mapping:**

| Condition | Actual Mapping |
|-----------|----------------|
| `ureq::Error::Status(401, _)` | `ProviderError::Unknown { code: "401", message: "..." }` |
| `ureq::Error::Status(429, _)` | `ProviderError::Unknown { code: "429", message: "..." }` |
| `ureq::Error::Status(400, _)` | `ProviderError::Unknown { code: "400", message: "..." }` |
| `ureq::Error::Status(403, _)` | `ProviderError::Unknown { code: "403", message: "..." }` |
| `ureq::Error::Status(404, _)` | `ProviderError::Unknown { code: "404", message: "..." }` |
| `ureq::Error::Status(408, _)` | `ProviderError::Unknown { code: "408", message: "..." }` |
| `ureq::Error::Status(409, _)` | `ProviderError::Unknown { code: "409", message: "..." }` |
| `ureq::Error::Status(5xx, _)` | `ProviderError::Unknown { code: "500", message: "..." }` |
| `ureq::Error::Transport` | `ProviderError::NetworkError` |
| JSON parse error | `ProviderError::InvalidRequest` |

**The special case `AuthenticationFailed` only exists in `from_env()` for missing env var:**
```rust
std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
    ReasoningError::Provider(ProviderError::AuthenticationFailed)
})?
```

**Defect**: All HTTP error status codes map to `ProviderError::Unknown` instead of specific variants (`AuthenticationFailed`, `RateLimited`, `ModelUnavailable`, etc.). The normalized error enum exists but is **not used correctly**.

---

## 8. Streaming Audit

### 8.1 Type Definitions (Only)

| Type | File | Status |
|------|------|--------|
| `StreamEvent` enum (5 variants) | `eak-ports/src/lib.rs:462-480` | ✅ Defined |
| `UsageMetadata` | `eak-ports/src/lib.rs:484-490` | ✅ Defined |
| `ModelProvider::stream_request()` | `eak-ports/src/lib.rs:907-915` | ❌ Returns `UnsupportedCapability` |
| `ModelProvider::cancel()` | `eak-ports/src/lib.rs:917-925` | ❌ Returns `UnsupportedCapability` |

### 8.2 Adapter Implementation

| Adapter | `stream_request()` | `cancel()` |
|---------|-------------------|------------|
| FixtureEngine | Not overridden (default) | Not overridden (default) |
| AnthropicEngine | Not overridden (default) | Not overridden (default) |

### 8.3 End-to-End Streaming

| Component | Status |
|-----------|--------|
| Provider API streaming | ❌ Not implemented |
| SSE/NDJSON parser | ❌ Not implemented |
| `StreamEvent` emission | ❌ Not implemented |
| Runtime integration | ❌ Not implemented |
| Agent streaming handling | ❌ Not implemented |
| UI/Tauri event bridge | ❌ Not implemented |

### 8.4 Conclusion

**Streaming = TYPES ONLY**. The trait signature exists but every implementation returns `UnsupportedCapability`. No streaming parser, no event emission, no runtime integration.

---

## 9. Tool Calling Audit

### 9.1 Type Definitions (Only)

| Type | File | Status |
|------|------|--------|
| `ToolDefinition` | `eak-ports/src/lib.rs:386` | ✅ Defined |
| `ToolCall` | `eak-ports/src/lib.rs:394` | ✅ Defined |
| `ToolResult` | `eak-ports/src/lib.rs:401` | ✅ Defined |
| `ToolChoice` enum | `eak-ports/src/lib.rs:422` | ✅ Defined |
| `Message::Assistant.tool_calls` | `eak-ports/src/lib.rs:408` | ✅ Defined |
| `ReasoningRequest.tools` | `eak-ports/src/lib.rs:490` | ✅ Defined |
| `ReasoningResponse.tool_calls` | `eak-ports/src/lib.rs:526` | ✅ Defined |
| `ModelCapability::ToolCalling` | `eak-ports/src/lib.rs:621` | ✅ Defined |

### 9.2 Execution Pipeline

| Link | Status | Evidence |
|------|--------|----------|
| Model → ToolCall | ❌ | AnthropicEngine forces single tool via `tool_choice: {"type": "tool", "name": ...}` |
| ToolCall → Agent | ❌ | Agents never inspect `response.tool_calls` |
| Agent → Tool Dispatcher | ❌ | No dispatcher exists |
| Tool Dispatcher → CapabilityRequest | ❌ | No dispatcher |
| CapabilityRequest → Kernel | ✅ | Kernel capability handlers exist |
| ToolResult → Model continuation | ❌ | No multi-turn conversation support |

### 9.3 AnthropicEngine Tool Usage

**File**: `eak/crates/eak-reasoning/src/anthropic.rs:117-170` (`invoke_tool`)
```rust
let body = serde_json::json!({
    ...
    "tools": [{
        "name": tool_name,
        "description": description,
        "input_schema": input_schema
    }],
    "tool_choice": {"type": "tool", "name": tool_name}  // FORCES single tool
});
```

**Forces single tool call per request** — no parallel tool calls, no auto tool choice.

### 9.4 Agent Usage

Agents (`RequirementAgent`, `PartSelectionAgent`, `ReviewExplanationMachine`) build requests with:
```rust
tools: None,
tool_choice: None,
```

**Agents never use tools.** The `tool_calls` field in `ReasoningResponse` is always empty `vec![]`.

### 9.5 Conclusion

**Tool Calling = PROTOCOL FOUNDATION ONLY**. Types are fully defined and serializable, but:
- No agent uses tools
- No tool dispatcher exists
- No multi-turn conversation for tool results
- Anthropic adapter forces single tool per request
- No parallel tool calls supported

---

## 10. Credential Architecture Audit

### 10.1 Defined Types

| Type | File | Used? |
|------|------|-------|
| `CredentialRef { store, key }` | `eak-ports/src/lib.rs:762` | ❌ Defined, never resolved |
| `ProviderConfig.credential_ref` | `eak-ports/src/lib.rs:778` | ❌ Never instantiated |
| `CredentialStore` trait | — | ❌ Not defined |
| `EnvCredentialStore` | — | ❌ Not defined |
| `KeyringCredentialStore` | — | ❌ Not defined |

### 10.2 Actual Credential Flow

**AnthropicEngine** (`eak-reasoning/src/anthropic.rs:35-39`):
```rust
pub fn from_env(model: impl Into<String>) -> Result<Self, ReasoningError> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| ReasoningError::Provider(ProviderError::AuthenticationFailed))?;
    Ok(Self::new(api_key, model))
}
```

**Direct `std::env::var("ANTHROPIC_API_KEY")`** — no `CredentialRef`, no `CredentialStore`.

### 10.3 Configuration Usage

| Config Type | Instantiated? | Used? |
|-------------|---------------|-------|
| `ProviderConfig` | ❌ | ❌ |
| `ModelConfig` | ❌ | ❌ |
| `ModelParameters` | ❌ | ❌ |

### 10.4 Conclusion

**Credential Architecture = TYPES ONLY**. `CredentialRef` is defined but:
- Never resolved to actual credentials
- No `CredentialStore` trait exists
- Anthropic still reads directly from environment
- No secure storage abstraction

---

## 11. Security Audit

### 11.1 Repository Secret Scan

```bash
$ git log --all --grep -i -E "key|secret|token|api"
(no output)

$ grep -r "sk-\|sk_\|api_key\|API_KEY\|secret" eak/ --include="*.rs" | grep -v test
(eak/crates/eak-reasoning/src/anthropic.rs: ANTHROPIC_API_KEY env var reference only)
```

### 11.2 Test Fixtures

All test fixtures use `FixtureEngine::single(ReasoningResponse { ... })` with **no API keys**.

### 11.3 Configuration Files

No `.env`, `.env.local`, or credential files in tracked files.

### 11.4 `.gitignore`

Contains standard Rust ignores. No credential-specific patterns needed since none exist.

### 11.5 Conclusion

**Security: CLEAN**. No secrets in repository. However, the **credential architecture is not implemented** — the existing Anthropic adapter reads directly from environment variables, which is acceptable for development but the abstraction layer is missing.

---

## 12. Test Verification

### 12.1 Exact Test Counts (Verified)

| Crate | Tests | Passed | Failed | Ignored |
|-------|-------|--------|--------|---------|
| eak-cli | 29 | 28 | 0 | 1 (live) |
| eak-compiler | 20 | 20 | 0 | 0 |
| eak-domain | 103 | 103 | 0 | 0 |
| eak-engines | 111 | 111 | 0 | 0 |
| eak-kicad | 24 | 24 | 0 | 0 |
| eak-phases | 29 | 29 | 0 | 0 |
| eak-ports | 17 | 17 | 0 | 0 |
| eak-reasoning | 2 | 2 | 0 | 0 |
| eak-runtime | 46 | 46 | 0 | 0 |
| eak-store | 1 | 1 | 0 | 0 |
| eak-units | 7 | 7 | 0 | 0 |
| **Total** | **389** | **388** | **0** | **1** |

**Previous report claimed 407 tests** — **actual count is 389 (388 passed + 1 ignored)**. The discrepancy of 18 tests appears to be from double-counting or estimation.

### 12.2 Validation Commands

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | ✅ PASS |
| `cargo check --workspace` | ✅ PASS |
| `cargo test --workspace` | ✅ PASS (388 passed, 1 ignored) |
| `cargo clippy --workspace` | ✅ PASS (0 warnings) |
| `cargo build --workspace --release` | ✅ PASS |

---

## 13. TODO vs Reality Matrix

| Task ID | Task | Claimed | Actual | Evidence |
|---------|------|---------|--------|----------|
| B1 | ModelCapability enum/bitflags | ✅ | ✅ IMPLEMENTED | `eak-ports/src/lib.rs:621-660` |
| B2 | ProviderId/ModelId | ✅ | ✅ IMPLEMENTED | `eak-ports/src/lib.rs:565,603` |
| B3 | ModelProvider trait | ✅ | ⚠️ PARTIAL | Trait exists but blanket impl weakens it |
| B4 | ReasoningRequest for tools | ✅ | ✅ IMPLEMENTED | Types defined, not used |
| B5 | ReasoningResponse for tools | ✅ | ✅ IMPLEMENTED | Types defined, not used |
| B6 | Streaming support | ✅ | ❌ FALSE | Types only, returns UnsupportedCapability |
| B7 | ProviderError normalized | ✅ | ❌ DEFECTIVE | Enum exists, mapping broken |
| B8 | FixtureEngine updated | ✅ | ✅ IMPLEMENTED | Tests pass |
| C1-C6 | Additional adapters | ⏭️ | ⏭️ DEFERRED | Not created |
| D1 | CredentialStore trait | ⏭️ | ⏭️ DEFERRED | Not created |
| D4 | CredentialRef | ✅ | ⚠️ PARTIAL | Type defined, never resolved |
| E1-E2 | ProviderConfig/ModelConfig | ✅ | ⚠️ PARTIAL | Defined, unused |
| F1-F2 | Agent/Runtime integration | ✅ | ⚠️ PARTIAL | Uses ReasoningEngine, not ModelProvider |
| G10 | Full workspace tests | ✅ | ✅ PASS | 388 passed |
| I1-I7 | Validation | ✅ | ✅ PASS | All commands pass |

---

## 14. Hallucination / Overclaim Matrix

| Claim | Actual Repository State | Evidence | Severity |
|-------|------------------------|----------|----------|
| "Multi-provider architecture implemented" | Only Fixture/Live enum; no registry, factory, selection | `ReasoningChoice` enum only | HIGH |
| "ModelProvider fully implemented" | Blanket impl over ReasoningEngine; all methods default to non-functional | `impl<T: ReasoningEngine> ModelProvider for T {}` | HIGH |
| "Error normalization implemented" | HTTP 401/429/400 → Unknown; only missing env var → AuthenticationFailed | `anthropic.rs:148-176` | HIGH |
| "407 tests pass" | 388 passed + 1 ignored = 389 total | `cargo test --workspace` output | MEDIUM |
| "Streaming implemented" | Only types; `stream_request()` returns `UnsupportedCapability` | `ModelProvider::stream_request()` default impl | HIGH |
| "Tool calling implemented" | Types only; agents don't use tools; no dispatcher | `tools: None`, `tool_choice: None` in all agents | HIGH |
| "Credential abstraction implemented" | `CredentialRef` type only; no resolution, no store | `CredentialRef` unused | HIGH |
| "Provider registry implemented" | Types defined (`ProviderConfig`), no registry code | No registry in codebase | HIGH |
| "Multi-provider selection works" | Only Fixture/Live enum | `ReasoningChoice` enum | HIGH |
| "Security verified" | No secrets in git (true), but credential abstraction missing | `git log --grep` clean | LOW |
| "Deterministic replay works" | True for FixtureEngine | Tests pass | N/A |

---

## 15. Critical Defects

| # | Defect | File | Impact |
|---|--------|------|--------|
| 1 | ModelProvider blanket impl defeats provider boundary | `eak-ports/src/lib.rs:945` | Architectural — no enforcement |
| 2 | Error normalization maps all HTTP errors to Unknown | `eak-reasoning/src/anthropic.rs:148-176` | Observability — cannot distinguish auth/rate-limit/model-unavailable |
| 3 | RuntimeCore depends on ReasoningEngine, not ModelProvider | `eak-runtime/src/runtime_core.rs:26,1246` | Architectural — new trait unused |
| 4 | Streaming methods return UnsupportedCapability | `eak-ports/src/lib.rs:907-925` | Feature incomplete |
| 5 | Tool calling pipeline missing end-to-end | Agents don't use tools; no dispatcher | Feature incomplete |
| 6 | CredentialRef never resolved; no CredentialStore | `CredentialRef` unused | Security abstraction missing |
| 7 | No provider selection/registry mechanism | Only Fixture/Live enum | Cannot add providers without code changes |

---

## 16. Recommended Corrections

### Priority 1 (Architectural)
1. **Remove blanket impl** — make `ModelProvider` a proper trait that adapters must implement explicitly
2. **Change `RuntimeCore.reasoning` to `Box<dyn ModelProvider>`** and use `provider.request()` / `provider.stream_request()`
3. **Implement proper HTTP error mapping** in AnthropicEngine (401→AuthFailed, 429→RateLimited, 404→ModelUnavailable, etc.)

### Priority 2 (Multi-Provider)
1. Create `ModelRegistry` with provider registration and model discovery
2. Add CLI `--provider` / `--model` arguments using registry
3. Implement provider factory pattern

### Priority 3 (Features)
1. Implement streaming in AnthropicEngine (SSE parsing)
2. Build tool dispatcher: Agent receives `tool_calls` → executes via CapabilityRequest → returns `ToolResult` → continues conversation
3. Implement `CredentialStore` trait + `EnvCredentialStore` + `KeyringCredentialStore`

### Priority 4 (Testing)
1. Add integration tests for error mapping (mock HTTP responses)
2. Add streaming tests with mock provider
3. Add tool calling round-trip tests

---

## 17. Corrected Architecture

### Current (Actual)
```
Agent → ReasoningRequest → RuntimeCore → Box<dyn ReasoningEngine> → FixtureEngine/AnthropicEngine
ModelProvider trait exists but unused; blanket impl makes it passive
```

### Required (Intended)
```
Agent → ReasoningRequest → RuntimeCore → Box<dyn ModelProvider> → Provider Adapter
     ↑                                                                      ↓
     └─────────────────────── ModelProvider trait ────────────────────────┘
     (stream_request, capabilities, supports, list_models all functional)
```

### Required Changes

| File | Change |
|------|--------|
| `eak-runtime/src/runtime_core.rs:26` | `reasoning: Box<dyn ModelProvider>` |
| `eak-runtime/src/runtime_core.rs:1246` | Use `self.reasoning.request(&req)` or `self.reasoning.stream_request(&req)` |
| `eak-ports/src/lib.rs:945` | Remove `impl<T: ReasoningEngine> ModelProvider for T {}` |
| `eak-reasoning/src/anthropic.rs` | Implement `ModelProvider` explicitly, override all methods |
| `eak-reasoning/src/fixture.rs` | Implement `ModelProvider` explicitly |
| `eak-cli/src/lib.rs` | Add provider registry, factory, CLI args |

---

## 18. Exact Next Engineering Task

**Task**: Refactor `RuntimeCore` to depend on `ModelProvider` instead of `ReasoningEngine`, and make `AnthropicEngine`/`FixtureEngine` explicitly implement `ModelProvider` with functional overrides.

**Files to modify:**
1. `eak/crates/eak-runtime/src/runtime_core.rs` — change `reasoning` field type and `reason()` method
2. `eak/crates/eak-ports/src/lib.rs` — remove blanket impl, ensure `ModelProvider` is the primary boundary
3. `eak/crates/eak-reasoning/src/anthropic.rs` — add `impl ModelProvider for AnthropicEngine { ... }` with functional overrides
4. `eak/crates/eak-reasoning/src/fixture.rs` — add `impl ModelProvider for FixtureEngine { ... }`
5. `eak/crates/eak-reasoning/src/lib.rs` — export `ModelProvider` implementations

**Validation**: After changes, `cargo test --workspace` must still pass 388 tests, and `AnthropicEngine` must correctly map HTTP errors to specific `ProviderError` variants.

---

## 19. Final Classification

**FORENSIC AUDIT COMPLETE**

**Verdict: YELLOW**

- **Critical defects**: 7
- **False/overstated claims**: 11
- **Tests actually verified**: 388 passed, 1 ignored
- **Implementation changes made during audit**: 0
- **Report**: `reports/EAK_V1_MODEL_PIPELINE_FORENSIC_AUDIT.md`

The core abstraction **compiles and tests pass**, but the **intended architecture is not implemented** — `ModelProvider` is a passive wrapper over `ReasoningEngine` with non-functional defaults, multi-provider execution is absent, and key features (streaming, tool calling, credentials) exist only as type definitions.