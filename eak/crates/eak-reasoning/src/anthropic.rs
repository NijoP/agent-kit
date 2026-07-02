//! Live Anthropic reasoning adapter (feature `live`). The ONLY crate that knows the
//! provider (P3) — the domain core stays provider-independent.
//!
//! Calls the Anthropic Messages API in tool/JSON mode constrained to the requirement
//! schema, then maps the structured tool output to [`CandidateRequirement`]s. The runtime
//! records the response as a `ReasoningCall` event so the run can be replayed without the
//! model (P4).

use eak_domain::{Priority, RequirementCategory};
use eak_ports::{
    CandidateExplanation, CandidateRequirement, ReasoningEngine, ReasoningError, ReasoningRequest,
    ReasoningResponse,
};
use serde::Deserialize;

/// Schema name of the advisory review explainer (E6 C1). Kept in sync with the fixture path.
const EXPLANATION_SCHEMA: &str = "violation_explanation_v1";

pub struct AnthropicEngine {
    api_key: String,
    model: String,
}

impl AnthropicEngine {
    pub fn new(api_key: String, model: impl Into<String>) -> Self {
        Self {
            api_key,
            model: model.into(),
        }
    }

    /// Construct from `ANTHROPIC_API_KEY`.
    pub fn from_env(model: impl Into<String>) -> Result<Self, ReasoningError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| ReasoningError::Provider("ANTHROPIC_API_KEY not set".into()))?;
        Ok(Self::new(api_key, model))
    }
}

#[derive(Deserialize)]
struct RawCandidate {
    statement: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    priority: String,
    #[serde(default)]
    acceptance_criterion: String,
    #[serde(default)]
    source_hint: String,
    #[serde(default = "default_confidence")]
    confidence: f64,
    #[serde(default)]
    rationale: String,
}
fn default_confidence() -> f64 {
    0.5
}

#[derive(Deserialize)]
struct RawOutput {
    #[serde(default)]
    candidates: Vec<RawCandidate>,
    #[serde(default)]
    clarifying_questions: Vec<String>,
}

#[derive(Deserialize)]
struct RawExplanation {
    #[serde(default)]
    explanation: String,
    #[serde(default)]
    suggested_fix: String,
}

fn map_category(s: &str) -> RequirementCategory {
    match s.to_lowercase().as_str() {
        "electrical" => RequirementCategory::Electrical,
        "mechanical" => RequirementCategory::Mechanical,
        "thermal" => RequirementCategory::Thermal,
        "regulatory" => RequirementCategory::Regulatory,
        "fabrication" => RequirementCategory::Fabrication,
        "cost" => RequirementCategory::Cost,
        "schedule" => RequirementCategory::Schedule,
        _ => RequirementCategory::Functional,
    }
}

fn map_priority(s: &str) -> Priority {
    match s.to_lowercase().as_str() {
        "high" => Priority::High,
        "low" => Priority::Low,
        _ => Priority::Medium,
    }
}

impl AnthropicEngine {
    /// Shared transport: POST a single-tool Messages request constrained to `input_schema`, then
    /// return the tool_use `input` object plus the full response body string (recorded as `raw`).
    /// Both the requirement and the explanation schemas ride this one path.
    fn invoke_tool(
        &self,
        tool_name: &str,
        description: &str,
        input_schema: serde_json::Value,
        req: &ReasoningRequest,
    ) -> Result<(serde_json::Value, String), ReasoningError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 2048,
            "temperature": req.temperature,
            "system": req.system,
            "messages": [{"role": "user", "content": req.prompt}],
            "tools": [{
                "name": tool_name,
                "description": description,
                "input_schema": input_schema
            }],
            "tool_choice": {"type": "tool", "name": tool_name}
        });

        let response = ureq::post("https://api.anthropic.com/v1/messages")
            .set("x-api-key", &self.api_key)
            .set("anthropic-version", "2023-06-01")
            .set("content-type", "application/json")
            .send_json(body);

        let value: serde_json::Value = match response {
            Ok(r) => r
                .into_json()
                .map_err(|e| ReasoningError::Provider(e.to_string()))?,
            Err(ureq::Error::Status(code, r)) => {
                let txt = r.into_string().unwrap_or_default();
                return Err(ReasoningError::Provider(format!("HTTP {code}: {txt}")));
            }
            Err(e) => return Err(ReasoningError::Provider(e.to_string())),
        };

        let content = value
            .get("content")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ReasoningError::Schema("response has no content array".into()))?;
        let input = content
            .iter()
            .find(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
            .and_then(|b| b.get("input"))
            .ok_or_else(|| ReasoningError::Schema("no tool_use block in response".into()))?
            .clone();
        Ok((input, value.to_string()))
    }

    /// Requirement decomposition path (`requirement_candidates_v1`) — unchanged behavior.
    fn request_requirements(
        &self,
        req: &ReasoningRequest,
    ) -> Result<ReasoningResponse, ReasoningError> {
        let tool_schema = serde_json::json!({
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
        });
        let (input, raw_body) = self.invoke_tool(
            "emit_requirements",
            "Return structured, testable requirement candidates.",
            tool_schema,
            req,
        )?;
        let raw: RawOutput =
            serde_json::from_value(input).map_err(|e| ReasoningError::Schema(e.to_string()))?;

        let candidates = raw
            .candidates
            .into_iter()
            .map(|c| CandidateRequirement {
                statement: c.statement,
                category: map_category(&c.category),
                priority: map_priority(&c.priority),
                acceptance_criterion: c.acceptance_criterion,
                source_hint: c.source_hint,
                confidence: c.confidence,
                rationale: c.rationale,
                targets: vec![],
            })
            .collect();

        Ok(ReasoningResponse {
            candidates,
            explanations: vec![],
            clarifying_questions: raw.clarifying_questions,
            raw: raw_body,
        })
    }

    /// Advisory review-explainer path (`violation_explanation_v1`, E6 C1). Best-effort/parallel:
    /// it maps the tool output to advisory [`CandidateExplanation`]s only — the caller stores the
    /// text as metadata and NEVER lets it drive a mutation, so a stray or empty response can never
    /// corrupt engineering state (advisory-only invariant).
    fn request_explanation(
        &self,
        req: &ReasoningRequest,
    ) -> Result<ReasoningResponse, ReasoningError> {
        let tool_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "explanation": {"type": "string"},
                "suggested_fix": {"type": "string"}
            },
            "required": ["explanation", "suggested_fix"]
        });
        let (input, raw_body) = self.invoke_tool(
            "explain_violation",
            "Explain, in plain English, one design-rule violation and suggest a fix. Advisory only.",
            tool_schema,
            req,
        )?;
        let raw: RawExplanation =
            serde_json::from_value(input).map_err(|e| ReasoningError::Schema(e.to_string()))?;

        Ok(ReasoningResponse {
            candidates: vec![],
            explanations: vec![CandidateExplanation {
                explanation: raw.explanation,
                suggested_fix: raw.suggested_fix,
            }],
            clarifying_questions: vec![],
            raw: raw_body,
        })
    }
}

impl ReasoningEngine for AnthropicEngine {
    fn model_id(&self) -> String {
        format!("anthropic:{}", self.model)
    }

    fn request_judgement(
        &self,
        req: &ReasoningRequest,
    ) -> Result<ReasoningResponse, ReasoningError> {
        if req.schema_name == EXPLANATION_SCHEMA {
            self.request_explanation(req)
        } else {
            self.request_requirements(req)
        }
    }
}
