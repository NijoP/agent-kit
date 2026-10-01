//! OpenAI-compatible provider adapter.
//!
//! Supports any OpenAI-compatible API endpoint (OpenAI, NVIDIA NIM, OpenRouter, Groq, Together,
//! Fireworks, vLLM, LM Studio, etc.) through a single configurable adapter.
//!
//! The adapter is configured via [`OpenAICompatConfig`] which specifies the base URL, model,
//! authentication, and optional custom headers.

use eak_domain::{Priority, RequirementCategory};
use eak_ports::{
    CandidateExplanation, CandidatePart, CandidateRequirement, Message, ModelId, ModelMetadata,
    ModelParameters, ProviderError, ProviderId, ReasoningEngine, ReasoningError, ReasoningRequest,
    ReasoningResponse, StreamEvent, ToolCall, ToolChoice, ToolDefinition, UsageMetadata,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for an OpenAI-compatible provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICompatConfig {
    /// Provider identifier (e.g., "openai", "openrouter", "groq", "nvidia", "together", "fireworks", "vllm", "lmstudio").
    pub provider_id: ProviderId,
    /// Human-readable name for display.
    pub display_name: String,
    /// Base URL for the OpenAI-compatible API (e.g., "https://api.openai.com/v1", "https://api.openrouter.ai/api/v1").
    pub base_url: String,
    /// Model identifier to use (e.g., "gpt-4o", "meta-llama/llama-3.1-70b-instruct").
    pub model: ModelId,
    /// Credential reference for the API key (store + key).
    pub credential_ref: Option<eak_ports::CredentialRef>,
    /// Optional API key directly (for testing; prefer credential_ref for production).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Optional custom headers to include in requests.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra_headers: HashMap<String, String>,
    /// Whether the provider supports model listing via `/models` endpoint.
    #[serde(default)]
    pub supports_model_listing: bool,
    /// Whether the provider supports streaming.
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
    /// Whether the provider supports tool/function calling.
    #[serde(default = "default_true")]
    pub supports_tool_calling: bool,
    /// Whether the provider supports structured output (JSON schema).
    #[serde(default)]
    pub supports_structured_output: bool,
    /// Whether the provider supports parallel tool calls.
    #[serde(default)]
    pub supports_parallel_tool_calls: bool,
    /// Whether the provider supports vision/multimodal.
    #[serde(default)]
    pub supports_vision: bool,
    /// Whether the provider supports system instructions.
    #[serde(default = "default_true")]
    pub supports_system_instructions: bool,
    /// Whether the provider supports reasoning effort parameter.
    #[serde(default)]
    pub supports_reasoning_effort: bool,
    /// Default request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    120
}

impl Default for OpenAICompatConfig {
    fn default() -> Self {
        Self {
            provider_id: ProviderId("openai".to_string()),
            display_name: "OpenAI Compatible".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: ModelId("gpt-4o".to_string()),
            credential_ref: None,
            api_key: None,
            extra_headers: HashMap::new(),
            supports_model_listing: true,
            supports_streaming: true,
            supports_tool_calling: true,
            supports_structured_output: false,
            supports_parallel_tool_calls: false,
            supports_vision: false,
            supports_system_instructions: true,
            supports_reasoning_effort: false,
            timeout_secs: 120,
        }
    }
}

/// OpenAI-compatible provider engine.
pub struct OpenAICompatEngine {
    config: OpenAICompatConfig,
    api_key: String,
    http_client: ureq::Agent,
}

impl OpenAICompatEngine {
    /// Create a new engine from configuration.
    pub fn new(config: OpenAICompatConfig) -> Result<Self, ReasoningError> {
        let api_key = resolve_api_key(&config)?;
        let http_client = ureq::Agent::new_with_config(
            ureq::Agent::config_builder()
                .timeout_global(std::time::Duration::from_secs(config.timeout_secs))
                .build(),
        );
        Ok(Self {
            config,
            api_key,
            http_client,
        })
    }

    /// Construct from environment variable (for backward compatibility with Anthropic-style usage).
    pub fn from_env(model: impl Into<String>) -> Result<Self, ReasoningError> {
        let config = OpenAICompatConfig {
            provider_id: ProviderId("openai".to_string()),
            display_name: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: ModelId(model.into()),
            credential_ref: None,
            api_key: None,
            extra_headers: HashMap::new(),
            supports_model_listing: true,
            supports_streaming: true,
            supports_tool_calling: true,
            supports_structured_output: false,
            supports_parallel_tool_calls: false,
            supports_vision: false,
            supports_system_instructions: true,
            supports_reasoning_effort: false,
            timeout_secs: 120,
        };
        Self::new(config)
    }

    /// Get the resolved API key from config or environment.
    fn resolve_api_key(config: &OpenAICompatConfig) -> Result<String, ReasoningError> {
        // Direct API key takes precedence (for testing)
        if let Some(key) = &config.api_key {
            if !key.is_empty() {
                return Ok(key.clone());
            }
        }
        // Try credential reference
        if let Some(cred_ref) = &config.credential_ref {
            // In a real implementation, this would resolve from a credential store.
            // For now, fall back to environment variable based on provider.
            let env_var = format!("{}_API_KEY", cred_ref.store.to_uppercase());
            if let Ok(key) = std::env::var(&env_var) {
                return Ok(key);
            }
        }
        // Fallback to provider-specific env var
        let env_var = format!("{}_API_KEY", config.provider_id.0.to_uppercase());
        std::env::var(&env_var)
            .map_err(|_| ReasoningError::Provider(ProviderError::AuthenticationFailed))
    }

    fn build_request_body(&self, req: &ReasoningRequest) -> serde_json::Value {
        let mut messages = Vec::new();
        for msg in &req.messages {
            match msg {
                Message::System { content } => {
                    messages.push(serde_json::json!({"role": "system", "content": content}));
                }
                Message::User { content } => {
                    messages.push(serde_json::json!({"role": "user", "content": content}));
                }
                Message::Assistant {
                    content,
                    tool_calls,
                } => {
                    let mut msg = serde_json::json!({"role": "assistant", "content": content});
                    if let Some(calls) = tool_calls {
                        if !calls.is_empty() {
                            msg["tool_calls"] = serde_json::to_value(calls).unwrap_or_default();
                        }
                    }
                    messages.push(msg);
                }
                Message::Tool {
                    content,
                    tool_call_id,
                } => {
                    messages.push(serde_json::json!({
                        "role": "tool",
                        "content": content,
                        "tool_call_id": tool_call_id
                    }));
                }
            }
        }

        let mut body = serde_json::json!({
            "model": self.config.model.0,
            "messages": messages,
            "temperature": req.temperature,
        });

        if let Some(top_p) = req.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }
        if let Some(max_tokens) = req.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(seed) = req.seed {
            if seed != 0 {
                body["seed"] = serde_json::json!(seed);
            }
        }
        if let Some(stop) = &req.stop_sequences {
            if !stop.is_empty() {
                body["stop"] = serde_json::json!(stop);
            }
        }
        if let Some(effort) = &req.reasoning_effort {
            if !effort.is_empty() && self.config.supports_reasoning_effort {
                body["reasoning_effort"] = serde_json::json!(effort);
            }
        }

        if let Some(tools) = &req.tools {
            if !tools.is_empty() && self.config.supports_tool_calling {
                body["tools"] = serde_json::to_value(tools).unwrap_or_default();
                body["tool_choice"] = match &req.tool_choice {
                    Some(ToolChoice::None) => serde_json::json!("none"),
                    Some(ToolChoice::Auto) => serde_json::json!("auto"),
                    Some(ToolChoice::Required) => serde_json::json!("required"),
                    Some(ToolChoice::Specific { name }) => {
                        serde_json::json!({"type": "function", "function": {"name": name}})
                    }
                    None => serde_json::json!("auto"),
                };
            }
        }

        body
    }

    fn send_request(&self, body: serde_json::Value) -> Result<serde_json::Value, ReasoningError> {
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let mut req = self
            .http_client
            .post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json");

        for (key, value) in &self.config.extra_headers {
            req = req.set(key, value);
        }

        let response = req.send_json(body);

        let value: serde_json::Value = match response {
            Ok(r) => r.into_json().map_err(|e| {
                ReasoningError::Provider(ProviderError::InvalidRequest {
                    reason: e.to_string(),
                })
            })?,
            Err(ureq::Error::Status(code, r)) => {
                let txt = r.into_string().unwrap_or_default();
                return Err(map_http_error(code, txt));
            }
            Err(e) => return Err(ReasoningError::Provider(ProviderError::NetworkError)),
        };

        Ok(value)
    }

    fn send_streaming_request(
        &self,
        body: serde_json::Value,
    ) -> Result<ureq::Response, ReasoningError> {
        let mut body = body;
        body["stream"] = serde_json::json!(true);

        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let mut req = self
            .http_client
            .post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .set("Accept", "text/event-stream");

        for (key, value) in &self.config.extra_headers {
            req = req.set(key, value);
        }

        let response = req.send_json(body);

        match response {
            Ok(r) => Ok(r),
            Err(ureq::Error::Status(code, r)) => {
                let txt = r.into_string().unwrap_or_default();
                Err(map_http_error(code, txt))
            }
            Err(e) => Err(ReasoningError::Provider(ProviderError::NetworkError)),
        }
    }

    fn parse_chat_response(
        &self,
        value: &serde_json::Value,
    ) -> Result<ReasoningResponse, ReasoningError> {
        let choices = value
            .get("choices")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ReasoningError::Schema("response missing choices array".into()))?;

        let first_choice = choices
            .first()
            .ok_or_else(|| ReasoningError::Schema("response has no choices".into()))?;

        let message = first_choice
            .get("message")
            .ok_or_else(|| ReasoningError::Schema("choice missing message".into()))?;

        let content = message
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let tool_calls = message
            .get("tool_calls")
            .and_then(|tc| tc.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|tc| {
                        let id = tc.get("id")?.as_str()?.to_string();
                        let function = tc.get("function")?;
                        let name = function.get("name")?.as_str()?.to_string();
                        let arguments = function.get("arguments")?.as_str()?.to_string();
                        let args_json = serde_json::from_str(&arguments).ok()?;
                        Some(ToolCall {
                            id,
                            name,
                            arguments: args_json,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let usage = value
            .get("usage")
            .map(|u| UsageMetadata {
                prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                completion_tokens: u
                    .get("completion_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                cached_tokens: u
                    .get("prompt_tokens_details")
                    .and_then(|d| d.get("cached_tokens"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32),
                reasoning_tokens: u
                    .get("completion_tokens_details")
                    .and_then(|d| d.get("reasoning_tokens"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32),
            })
            .unwrap_or_default();

        Ok(ReasoningResponse {
            candidates: vec![],
            explanations: vec![],
            part_candidates: vec![],
            tool_calls,
            usage,
            clarifying_questions: vec![],
            raw: value.to_string(),
        })
    }

    fn schema_name_to_tool_schema(&self, schema_name: &str) -> Option<serde_json::Value> {
        match schema_name {
            "requirement_candidates_v1" => Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "candidates": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "statement": {"type": "string"},
                                "category": {"type": "string", "enum": ["functional","electrical","mechanical","thermal","regulatory","cost","schedule"]},
                                "priority": {"type": "string", "enum": ["high","medium","low"]},
                                "acceptance_criterion": {"type": "string"},
                                "source_hint": {"type": "string"},
                                "confidence": {"type": "number"},
                                "rationale": {"type": "string"}
                            },
                            "required": ["statement","category","priority","acceptance_criterion"]
                        }
                    },
                    "clarifying_questions": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["candidates"]
            })),
            "violation_explanation_v1" => Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "explanation": {"type": "string"},
                    "suggested_fix": {"type": "string"}
                },
                "required": ["explanation", "suggested_fix"]
            })),
            "part_candidates_v1" => Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "part_candidates": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "component_class": {"type": "string", "enum": ["Connector","Regulator","Ic","Resistor","Capacitor"]},
                                "mpn": {"type": "string"},
                                "rationale": {"type": "string"},
                                "confidence": {"type": "number"}
                            },
                            "required": ["component_class","mpn"]
                        }
                    }
                },
                "required": ["part_candidates"]
            })),
            _ => None,
        }
    }
}

fn map_http_error(code: u16, message: String) -> ReasoningError {
    let provider_error = match code {
        401 => ProviderError::AuthenticationFailed,
        403 => ProviderError::InvalidApiKey,
        404 => ProviderError::ModelUnavailable {
            model: ModelId("unknown".to_string()),
        },
        429 => ProviderError::RateLimited {
            retry_after_secs: None,
        },
        408 => ProviderError::Timeout,
        400 => ProviderError::InvalidRequest { reason: message },
        500..=599 => ProviderError::ProviderUnavailable,
        _ => ProviderError::Unknown {
            code: code.to_string(),
            message,
        },
    };
    ReasoningError::Provider(provider_error)
}

impl OpenAICompatEngine {
    fn parse_schema_response(
        &self,
        value: &serde_json::Value,
        schema_name: &str,
    ) -> Result<ReasoningResponse, ReasoningError> {
        let choices = value
            .get("choices")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ReasoningError::Schema("response missing choices array".into()))?;

        let first_choice = choices
            .first()
            .ok_or_else(|| ReasoningError::Schema("response has no choices".into()))?;

        let message = first_choice
            .get("message")
            .ok_or_else(|| ReasoningError::Schema("choice missing message".into()))?;

        let content = message
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let tool_calls = message
            .get("tool_calls")
            .and_then(|tc| tc.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|tc| {
                        let id = tc.get("id")?.as_str()?.to_string();
                        let function = tc.get("function")?;
                        let name = function.get("name")?.as_str()?.to_string();
                        let arguments = function.get("arguments")?.as_str()?.to_string();
                        let args_json = serde_json::from_str(&arguments).ok()?;
                        Some(ToolCall {
                            id,
                            name,
                            arguments: args_json,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let usage = value
            .get("usage")
            .map(|u| UsageMetadata {
                prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                completion_tokens: u
                    .get("completion_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                cached_tokens: u
                    .get("prompt_tokens_details")
                    .and_then(|d| d.get("cached_tokens"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32),
                reasoning_tokens: u
                    .get("completion_tokens_details")
                    .and_then(|d| d.get("reasoning_tokens"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32),
            })
            .unwrap_or_default();

        match schema_name {
            "requirement_candidates_v1" => {
                let raw: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| ReasoningError::Schema(e.to_string()))?;
                let candidates = raw
                    .get("candidates")
                    .and_then(|c| c.as_array())
                    .ok_or_else(|| ReasoningError::Schema("missing candidates array".into()))?
                    .iter()
                    .filter_map(|c| {
                        let statement = c.get("statement")?.as_str()?.to_string();
                        let category = c.get("category")?.as_str()?;
                        let priority = c.get("priority")?.as_str()?;
                        let acceptance_criterion =
                            c.get("acceptance_criterion")?.as_str()?.to_string();
                        let source_hint = c
                            .get("source_hint")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let confidence =
                            c.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);
                        let rationale = c
                            .get("rationale")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        Some(CandidateRequirement {
                            statement,
                            category: match category.to_lowercase().as_str() {
                                "electrical" => RequirementCategory::Electrical,
                                "mechanical" => RequirementCategory::Mechanical,
                                "thermal" => RequirementCategory::Thermal,
                                "regulatory" => RequirementCategory::Regulatory,
                                "fabrication" => RequirementCategory::Fabrication,
                                "cost" => RequirementCategory::Cost,
                                "schedule" => RequirementCategory::Schedule,
                                _ => RequirementCategory::Functional,
                            },
                            priority: match priority.to_lowercase().as_str() {
                                "high" => Priority::High,
                                "low" => Priority::Low,
                                _ => Priority::Medium,
                            },
                            acceptance_criterion,
                            source_hint,
                            confidence,
                            rationale,
                            targets: vec![],
                        })
                    })
                    .collect();

                Ok(ReasoningResponse {
                    candidates,
                    part_candidates: vec![],
                    explanations: vec![],
                    tool_calls,
                    usage,
                    clarifying_questions: raw
                        .get("clarifying_questions")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default(),
                    raw: value.to_string(),
                })
            }
            "violation_explanation_v1" => {
                let raw: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| ReasoningError::Schema(e.to_string()))?;
                let explanation = raw.get("explanation")?.as_str()?.to_string();
                let suggested_fix = raw.get("suggested_fix")?.as_str()?.to_string();

                Ok(ReasoningResponse {
                    candidates: vec![],
                    part_candidates: vec![],
                    explanations: vec![CandidateExplanation {
                        explanation,
                        suggested_fix,
                    }],
                    tool_calls,
                    usage,
                    clarifying_questions: vec![],
                    raw: value.to_string(),
                })
            }
            "part_candidates_v1" => {
                let raw: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| ReasoningError::Schema(e.to_string()))?;
                let part_candidates = raw
                    .get("part_candidates")
                    .and_then(|c| c.as_array())
                    .ok_or_else(|| ReasoningError::Schema("missing part_candidates array".into()))?
                    .iter()
                    .filter_map(|c| {
                        Some(CandidatePart {
                            component_class: c.get("component_class")?.as_str()?.to_string(),
                            mpn: c.get("mpn")?.as_str()?.to_string(),
                            rationale: c
                                .get("rationale")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            confidence: c.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5),
                        })
                    })
                    .collect();

                Ok(ReasoningResponse {
                    candidates: vec![],
                    part_candidates,
                    explanations: vec![],
                    tool_calls,
                    usage,
                    clarifying_questions: vec![],
                    raw: value.to_string(),
                })
            }
            _ => Err(ReasoningError::Schema(format!(
                "unknown schema: {}",
                schema_name
            ))),
        }
    }
}

impl ReasoningEngine for OpenAICompatEngine {
    fn model_id(&self) -> String {
        format!("{}:{}", self.config.provider_id, self.config.model)
    }

    fn request_judgement(
        &self,
        req: &ReasoningRequest,
    ) -> Result<ReasoningResponse, ReasoningError> {
        // For schema-specific requests, we need to use tool calling to get structured output
        let tool_schema = self.schema_name_to_tool_schema(&req.schema_name);

        let body = if let Some(schema) = tool_schema {
            let body = self.build_request_body(req);
            // Add tool for structured output
            serde_json::json!({
                "model": self.config.model.0,
                "messages": req.messages.iter().map(|m| match m {
                    Message::System { content } => serde_json::json!({"role": "system", "content": content}),
                    Message::User { content } => serde_json::json!({"role": "user", "content": content}),
                    Message::Assistant { content, tool_calls } => {
                        let mut msg = serde_json::json!({"role": "assistant", "content": content});
                        if let Some(calls) = tool_calls {
                            if !calls.is_empty() {
                                msg["tool_calls"] = serde_json::to_value(calls).unwrap_or_default();
                            }
                        }
                        msg
                    },
                    Message::Tool { content, tool_call_id } => {
                        serde_json::json!({"role": "tool", "content": content, "tool_call_id": tool_call_id})
                    },
                }).collect::<Vec<_>>(),
                "tools": [{"type": "function", "function": {"name": "emit_schema", "description": "Emit structured output", "parameters": schema}}],
                "tool_choice": {"type": "function", "function": {"name": "emit_schema"}},
                "temperature": req.temperature,
            })
        } else {
            self.build_request_body(req)
        };

        let response = self.send_request(body)?;
        let tool_schema = self.schema_name_to_tool_schema(&req.schema_name);

        if tool_schema.is_some() {
            self.parse_schema_response(&response, &req.schema_name)
        } else {
            self.parse_chat_response(&response)
        }
    }
}

impl eak_ports::ModelProvider for OpenAICompatEngine {
    fn provider_id(&self) -> ProviderId {
        self.config.provider_id.clone()
    }

    fn model_id_parsed(&self) -> ModelId {
        self.config.model.clone()
    }

    fn capabilities(&self) -> eak_ports::CapabilitySet {
        let mut caps = eak_ports::CapabilitySet::TEXT_GENERATION;
        if self.config.supports_streaming {
            caps |= eak_ports::CapabilitySet::STREAMING;
        }
        if self.config.supports_tool_calling {
            caps |= eak_ports::CapabilitySet::TOOL_CALLING;
        }
        if self.config.supports_structured_output {
            caps |= eak_ports::CapabilitySet::STRUCTURED_OUTPUT;
        }
        if self.config.supports_vision {
            caps |= eak_ports::CapabilitySet::VISION;
        }
        if self.config.supports_system_instructions {
            caps |= eak_ports::CapabilitySet::SYSTEM_INSTRUCTIONS;
        }
        if self.config.supports_parallel_tool_calls {
            caps |= eak_ports::CapabilitySet::PARALLEL_TOOL_CALLS;
        }
        if self.config.supports_reasoning_effort {
            caps |= eak_ports::CapabilitySet::REASONING_EFFORT;
        }
        caps
    }

    fn metadata(&self) -> Option<ModelMetadata> {
        Some(ModelMetadata {
            provider: self.config.provider_id.clone(),
            model_id: self.config.model.clone(),
            display_name: format!("{} ({})", self.config.display_name, self.config.model),
            capabilities: self.capabilities(),
            context_window: None,
            max_output_tokens: None,
            supports_parallel_tool_calls: self.config.supports_parallel_tool_calls,
            input_price_per_million: None,
            output_price_per_million: None,
        })
    }

    fn stream_request(
        &self,
        req: &ReasoningRequest,
    ) -> Result<Box<dyn Iterator<Item = Result<StreamEvent, ReasoningError>> + Send>, ReasoningError>
    {
        if !self.config.supports_streaming {
            return Err(ReasoningError::Provider(
                ProviderError::UnsupportedCapability {
                    capability: eak_ports::ModelCapability::Streaming,
                },
            ));
        }

        let body = self.build_request_body(req);
        let response = self.send_streaming_request(body)?;
        let reader = response.into_reader();
        let lines = std::io::BufReader::new(reader).lines();

        let iter = lines.map(move |line_result| {
            let line =
                line_result.map_err(|e| ReasoningError::Provider(ProviderError::NetworkError))?;
            let line = line.trim();
            if line.is_empty() || line == "data: [DONE]" {
                return Ok(StreamEvent::Complete { usage: None });
            }
            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(choices) = value.get("choices").and_then(|c| c.as_array()) {
                        if let Some(first) = choices.first() {
                            if let Some(delta) = first.get("delta") {
                                if let Some(content) = delta.get("content").and_then(|c| c.as_str())
                                {
                                    if !content.is_empty() {
                                        return Ok(StreamEvent::TextDelta {
                                            text: content.to_string(),
                                        });
                                    }
                                }
                                if let Some(tool_calls) =
                                    delta.get("tool_calls").and_then(|tc| tc.as_array())
                                {
                                    for tc in tool_calls {
                                        let index =
                                            tc.get("index").and_then(|v| v.as_u64()).unwrap_or(0)
                                                as u32;
                                        let id = tc
                                            .get("id")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());
                                        let function = tc.get("function");
                                        let name = function
                                            .and_then(|f| f.get("name"))
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());
                                        let arguments = function
                                            .and_then(|f| f.get("arguments"))
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());
                                        return Ok(StreamEvent::ToolCallDelta {
                                            index,
                                            id,
                                            name,
                                            arguments,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Ok(StreamEvent::Complete { usage: None })
        });

        Ok(Box::new(iter))
    }

    fn cancel(&self) -> Result<(), ReasoningError> {
        // ureq doesn't support cancellation easily; return unsupported for now
        Err(ReasoningError::Provider(
            ProviderError::UnsupportedCapability {
                capability: eak_ports::ModelCapability::Streaming,
            },
        ))
    }

    fn list_models(&self) -> Result<Vec<ModelMetadata>, ReasoningError> {
        if !self.config.supports_model_listing {
            return Err(ReasoningError::Provider(
                ProviderError::UnsupportedCapability {
                    capability: eak_ports::ModelCapability::TextGeneration,
                },
            ));
        }

        let url = format!("{}/models", self.config.base_url.trim_end_matches('/'));
        let req = self
            .http_client
            .get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key));

        for (key, value) in &self.config.extra_headers {
            req = req.set(key, value);
        }

        let response = req.call();
        let value: serde_json::Value = match response {
            Ok(r) => r.into_json().map_err(|e| {
                ReasoningError::Provider(ProviderError::InvalidRequest {
                    reason: e.to_string(),
                })
            })?,
            Err(ureq::Error::Status(code, r)) => {
                let txt = r.into_string().unwrap_or_default();
                return Err(map_http_error(code, txt));
            }
            Err(e) => return Err(ReasoningError::Provider(ProviderError::NetworkError)),
        };

        let models = value
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ReasoningError::Schema("models response missing data array".into()))?;

        let metadata: Vec<ModelMetadata> = models
            .iter()
            .filter_map(|m| {
                Some(ModelMetadata {
                    provider: self.config.provider_id.clone(),
                    model_id: ModelId(m.get("id")?.as_str()?.to_string()),
                    display_name: m.get("id")?.as_str()?.to_string(),
                    capabilities: self.capabilities(),
                    context_window: None,
                    max_output_tokens: None,
                    supports_parallel_tool_calls: self.config.supports_parallel_tool_calls,
                    input_price_per_million: None,
                    output_price_per_million: None,
                })
            })
            .collect();

        Ok(metadata)
    }
}
