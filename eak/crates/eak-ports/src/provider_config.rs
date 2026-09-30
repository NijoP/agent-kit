//! Provider configuration persistence.
//!
//! Handles serialization/deserialization of [`ProviderRegistry`] and [`ModelRegistry`]
//! to/from JSON files. This enables project-level provider configuration that persists
//! across sessions without storing raw credentials.

use crate::{
    CredentialRef, ModelConfig, ModelId, ModelParameters, ModelRegistry, ProviderConfig,
    ProviderId, ProviderRegistry,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// File name for provider configuration persistence.
const PROVIDER_CONFIG_FILE: &str = "providers.json";

/// File name for model configuration persistence.
const MODEL_CONFIG_FILE: &str = "models.json";

/// Aggregated project configuration for model providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderProjectConfig {
    /// Provider registry.
    pub providers: ProviderRegistry,
    /// Model registry.
    pub models: ModelRegistry,
    /// Active provider ID (for quick selection).
    pub active_provider: Option<ProviderId>,
    /// Active model ID (for quick selection).
    pub active_model: Option<ModelId>,
}

impl ProviderProjectConfig {
    /// Create a new empty project configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a project directory.
    pub fn load(project_dir: impl AsRef<Path>) -> Result<Self, ProviderConfigError> {
        let project_dir = project_dir.as_ref();
        let config_path = project_dir.join(PROVIDER_CONFIG_FILE);

        if config_path.exists() {
            let data = std::fs::read_to_string(&config_path)
                .map_err(|e| ProviderConfigError::Io(e.to_string()))?;
            let config: ProviderProjectConfig =
                serde_json::from_str(&data).map_err(ProviderConfigError::Serialization)?;
            Ok(config)
        } else {
            // Fallback to legacy separate files
            let providers_path = project_dir.join("providers.json");
            let models_path = project_dir.join(MODEL_CONFIG_FILE);

            let providers = if providers_path.exists() {
                let data = std::fs::read_to_string(&providers_path)
                    .map_err(|e| ProviderConfigError::Io(e.to_string()))?;
                serde_json::from_str(&data).map_err(ProviderConfigError::Serialization)?
            } else {
                ProviderRegistry::new()
            };

            let models = if models_path.exists() {
                let data = std::fs::read_to_string(&models_path)
                    .map_err(|e| ProviderConfigError::Io(e.to_string()))?;
                serde_json::from_str(&data).map_err(ProviderConfigError::Serialization)?
            } else {
                ModelRegistry::new()
            };

            Ok(Self {
                providers,
                models,
                active_provider: None,
                active_model: None,
            })
        }
    }

    /// Save configuration to a project directory.
    pub fn save(&self, project_dir: impl AsRef<Path>) -> Result<(), ProviderConfigError> {
        let project_dir = project_dir.as_ref();
        std::fs::create_dir_all(project_dir).map_err(|e| ProviderConfigError::Io(e.to_string()))?;

        let config_path = project_dir.join(PROVIDER_CONFIG_FILE);
        let config_json =
            serde_json::to_string_pretty(self).map_err(ProviderConfigError::Serialization)?;
        std::fs::write(&config_path, config_json)
            .map_err(|e| ProviderConfigError::Io(e.to_string()))?;

        Ok(())
    }

    /// Register a provider and its default model.
    pub fn register_provider(&mut self, config: ProviderConfig) {
        if let Some(model_id) = &config.default_model {
            let model_config = ModelConfig {
                provider: config.id.clone(),
                model_id: model_id.clone(),
                enabled: true,
                capabilities: crate::CapabilitySet::TEXT_GENERATION,
                parameters: ModelParameters::default(),
                metadata: None,
            };
            self.models.register(model_config);
        }
        self.providers.register(config);
    }

    /// Set the active provider.
    pub fn set_active_provider(&mut self, id: ProviderId) -> Result<(), ProviderConfigError> {
        if self.providers.get(&id).is_none() {
            return Err(ProviderConfigError::ProviderNotFound(id));
        }
        self.active_provider = Some(id);
        Ok(())
    }

    /// Set the active model.
    pub fn set_active_model(
        &mut self,
        provider: ProviderId,
        model_id: ModelId,
    ) -> Result<(), ProviderConfigError> {
        if self.models.get(&provider, &model_id).is_none() {
            return Err(ProviderConfigError::ModelNotFound(provider, model_id));
        }
        self.active_provider = Some(provider);
        self.active_model = Some(model_id);
        Ok(())
    }

    /// Get the active provider configuration.
    pub fn active_provider_config(&self) -> Option<&ProviderConfig> {
        self.active_provider
            .as_ref()
            .and_then(|id| self.providers.get(id))
    }

    /// Get the active model configuration.
    pub fn active_model_config(&self) -> Option<&ModelConfig> {
        match (&self.active_provider, &self.active_model) {
            (Some(p), Some(m)) => self.models.get(p, m),
            _ => None,
        }
    }

    /// Validate all configurations.
    pub fn validate(&self) -> Result<(), ProviderConfigError> {
        // Check for duplicate provider IDs (already handled by HashMap)
        // Check that each provider has a default model
        for config in self.providers.list_configs() {
            if config.enabled && config.default_model.is_none() {
                return Err(ProviderConfigError::MissingDefaultModel(config.id.clone()));
            }
            // Validate credential reference if present
            if let Some(cred_ref) = &config.credential_ref {
                if cred_ref.store.is_empty() || cred_ref.key.is_empty() {
                    return Err(ProviderConfigError::InvalidCredentialRef(config.id.clone()));
                }
            }
        }

        // Check that each model belongs to a registered provider
        for model in self.models.list_all() {
            if !self.providers.list().contains(&&model.provider) {
                return Err(ProviderConfigError::OrphanModel(
                    model.provider.clone(),
                    model.model_id.clone(),
                ));
            }
        }

        Ok(())
    }
}

/// Errors that can occur during provider configuration operations.
#[derive(Debug, thiserror::Error)]
pub enum ProviderConfigError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Provider not found: {0}")]
    ProviderNotFound(ProviderId),
    #[error("Model not found: provider={0}, model={1}")]
    ModelNotFound(ProviderId, ModelId),
    #[error("Provider {0} is enabled but has no default_model")]
    MissingDefaultModel(ProviderId),
    #[error("Provider {0} has invalid credential reference (empty store or key)")]
    InvalidCredentialRef(ProviderId),
    #[error("Model {1} references unregistered provider {0}")]
    OrphanModel(ProviderId, ModelId),
    #[error("Validation error: {0}")]
    Validation(String),
}

/// A secure credential store that uses the OS keyring (stub implementation).
///
/// This is a placeholder that provides the same interface as `EnvCredentialStore`
/// but can be extended to use the `keyring` crate for actual OS keyring integration.
/// Currently falls back to environment variables.
#[derive(Debug, Default, Clone)]
pub struct KeyringCredentialStore;

impl crate::CredentialStore for KeyringCredentialStore {
    fn resolve(&self, cred_ref: &CredentialRef) -> Option<String> {
        // Try keyring first (not implemented yet, falls through to env)
        // In a full implementation, this would use the `keyring` crate:
        // keyring::Entry::new(&cred_ref.store, &cred_ref.key)
        //     .ok()
        //     .and_then(|e| e.get_password().ok())

        // Fallback to environment variable convention
        let env_key = format!(
            "{}_{}",
            cred_ref.store.to_uppercase(),
            cred_ref.key.to_uppercase()
        );
        std::env::var(&env_key).ok()
    }
}

/// Validation result for a provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderValidationResult {
    pub provider_id: ProviderId,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate a single provider configuration without creating a provider.
pub fn validate_provider_config(config: &ProviderConfig) -> ProviderValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Required fields
    if config.id.0.is_empty() {
        errors.push("Provider ID cannot be empty".to_string());
    }
    if config.name.is_empty() {
        errors.push("Provider name cannot be empty".to_string());
    }

    // If enabled, must have default_model
    if config.enabled {
        if config.default_model.is_none() {
            errors.push("Enabled provider must have a default_model".to_string());
        }
        if config.endpoint.is_none() {
            warnings.push("No endpoint specified; will use provider default".to_string());
        }
    }

    // Validate credential reference
    if let Some(cred_ref) = &config.credential_ref {
        if cred_ref.store.is_empty() || cred_ref.key.is_empty() {
            errors.push("Credential reference has empty store or key".to_string());
        }
    } else if config.enabled {
        warnings.push(
            "No credential reference configured; will rely on environment variables".to_string(),
        );
    }

    // Validate base URL format if provided
    if let Some(endpoint) = &config.endpoint {
        if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            errors.push("Endpoint must be a valid HTTP/HTTPS URL".to_string());
        }
    }

    ProviderValidationResult {
        provider_id: config.id.clone(),
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Validate a model configuration.
pub fn validate_model_config(config: &ModelConfig) -> ProviderValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if config.provider.0.is_empty() {
        errors.push("Model provider ID cannot be empty".to_string());
    }
    if config.model_id.0.is_empty() {
        errors.push("Model ID cannot be empty".to_string());
    }

    if !config.enabled {
        warnings.push("Model is disabled".to_string());
    }

    if config.capabilities.is_empty() {
        warnings.push("Model has no declared capabilities".to_string());
    }

    ProviderValidationResult {
        provider_id: config.provider.clone(),
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CapabilitySet, CredentialRef, CredentialStore, ModelCapability, ModelId, ModelParameters,
        ProviderConfig, ProviderFactory, ProviderId,
    };
    use std::collections::HashMap;

    #[test]
    fn provider_project_config_roundtrip() {
        let mut config = ProviderProjectConfig::new();
        let provider = ProviderConfig {
            id: ProviderId("test".to_string()),
            name: "Test Provider".to_string(),
            enabled: true,
            endpoint: Some("https://api.example.com/v1".to_string()),
            credential_ref: Some(CredentialRef::new("env", "API_KEY")),
            default_model: Some(ModelId("test-model".to_string())),
            extra_headers: Some(HashMap::from([(
                "X-Custom".to_string(),
                "value".to_string(),
            )])),
        };
        config.register_provider(provider);

        // Test serialization
        let json = serde_json::to_string_pretty(&config);
        eprintln!("JSON: {:?}", json);
        let json = json.unwrap();
        let parsed: ProviderProjectConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.providers.list().len(), parsed.providers.list().len());
    }

    #[test]
    fn validate_provider_config_missing_default_model() {
        let config = ProviderConfig {
            id: ProviderId("test".to_string()),
            name: "Test".to_string(),
            enabled: true,
            endpoint: None,
            credential_ref: None,
            default_model: None,
            extra_headers: None,
        };
        let result = validate_provider_config(&config);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("default_model")));
    }

    #[test]
    fn validate_provider_config_valid() {
        let config = ProviderConfig {
            id: ProviderId("test".to_string()),
            name: "Test".to_string(),
            enabled: true,
            endpoint: Some("https://api.example.com".to_string()),
            credential_ref: Some(CredentialRef::new("env", "API_KEY")),
            default_model: Some(ModelId("model-1".to_string())),
            extra_headers: None,
        };
        let result = validate_provider_config(&config);
        assert!(result.valid);
    }

    #[test]
    fn validate_provider_config_invalid_url() {
        let config = ProviderConfig {
            id: ProviderId("test".to_string()),
            name: "Test".to_string(),
            enabled: true,
            endpoint: Some("ftp://invalid.com".to_string()),
            credential_ref: Some(CredentialRef::new("env", "API_KEY")),
            default_model: Some(ModelId("model-1".to_string())),
            extra_headers: None,
        };
        let result = validate_provider_config(&config);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("HTTP/HTTPS")));
    }

    #[test]
    fn validate_provider_config_empty_credential_ref() {
        let config = ProviderConfig {
            id: ProviderId("test".to_string()),
            name: "Test".to_string(),
            enabled: true,
            endpoint: Some("https://api.example.com".to_string()),
            credential_ref: Some(CredentialRef::new("", "KEY")),
            default_model: Some(ModelId("model-1".to_string())),
            extra_headers: None,
        };
        let result = validate_provider_config(&config);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("empty store or key")));
    }

    #[test]
    fn provider_registry_operations() {
        let mut registry = ProviderRegistry::new();
        let provider = ProviderConfig {
            id: ProviderId("test".to_string()),
            name: "Test Provider".to_string(),
            enabled: true,
            endpoint: None,
            credential_ref: None,
            default_model: Some(ModelId("model-1".to_string())),
            extra_headers: None,
        };
        registry.register(provider.clone());
        assert!(registry.get(&ProviderId("test".to_string())).is_some());
        assert_eq!(registry.list().len(), 1);
        assert!(registry.is_enabled(&ProviderId("test".to_string())));
        registry.disable(&ProviderId("test".to_string()));
        assert!(!registry.is_enabled(&ProviderId("test".to_string())));
        registry.enable(&ProviderId("test".to_string()));
        assert!(registry.is_enabled(&ProviderId("test".to_string())));
        assert_eq!(
            registry.default_model(&ProviderId("test".to_string())),
            Some(&ModelId("model-1".to_string()))
        );
        registry.remove(&ProviderId("test".to_string()));
        assert!(registry.get(&ProviderId("test".to_string())).is_none());
    }

    #[test]
    fn model_registry_operations() {
        let mut registry = ModelRegistry::new();
        let model = ModelConfig {
            provider: ProviderId("test".to_string()),
            model_id: ModelId("model-1".to_string()),
            enabled: true,
            capabilities: CapabilitySet::TEXT_GENERATION | CapabilitySet::TOOL_CALLING,
            parameters: ModelParameters::default(),
            metadata: None,
        };
        registry.register(model.clone());
        assert!(registry
            .get(
                &ProviderId("test".to_string()),
                &ModelId("model-1".to_string())
            )
            .is_some());
        assert_eq!(
            registry
                .list_for_provider(&ProviderId("test".to_string()))
                .len(),
            1
        );
        assert_eq!(registry.list_all().len(), 1);
        let found = registry.find_by_capability(ModelCapability::ToolCalling);
        assert_eq!(found.len(), 1);
        let selected = registry.select_model(
            &ProviderId("test".to_string()),
            ModelCapability::ToolCalling,
        );
        assert!(selected.is_some());
        registry.remove(
            &ProviderId("test".to_string()),
            &ModelId("model-1".to_string()),
        );
        assert!(registry
            .get(
                &ProviderId("test".to_string()),
                &ModelId("model-1".to_string())
            )
            .is_none());
    }

    #[test]
    fn provider_project_config_save_load() {
        let mut config = ProviderProjectConfig::new();
        let provider = ProviderConfig {
            id: ProviderId("openai".to_string()),
            name: "OpenAI".to_string(),
            enabled: true,
            endpoint: Some("https://api.openai.com/v1".to_string()),
            credential_ref: Some(CredentialRef::new("env", "OPENAI_API_KEY")),
            default_model: Some(ModelId("gpt-4o".to_string())),
            extra_headers: Some(HashMap::from([(
                "X-Custom".to_string(),
                "value".to_string(),
            )])),
        };
        config.register_provider(provider);
        config
            .set_active_provider(ProviderId("openai".to_string()))
            .unwrap();
        config
            .set_active_model(
                ProviderId("openai".to_string()),
                ModelId("gpt-4o".to_string()),
            )
            .unwrap();

        let temp_dir = std::env::temp_dir().join("eak-test-provider-config");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        config.save(&temp_dir).unwrap();

        let loaded = ProviderProjectConfig::load(&temp_dir).unwrap();
        assert_eq!(loaded.providers.list().len(), 1);
        assert_eq!(
            loaded.active_provider,
            Some(ProviderId("openai".to_string()))
        );
        assert_eq!(loaded.active_model, Some(ModelId("gpt-4o".to_string())));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn credential_reference_does_not_serialize_secret_value() {
        let provider = ProviderConfig {
            id: ProviderId("test".to_string()),
            name: "Test Provider".to_string(),
            enabled: true,
            endpoint: None,
            credential_ref: Some(CredentialRef::new("env", "TEST_API_KEY")),
            default_model: Some(ModelId("model-1".to_string())),
            extra_headers: None,
        };

        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("TEST_API_KEY"));
        assert!(!json.contains("secret-value"));
    }

    #[test]
    fn provider_factory_resolves_credentials_through_store() {
        let factory = ProviderFactory::new(Box::new(KeyringCredentialStore));
        let cred_ref = CredentialRef::new("TEST_STORE", "TEST_KEY");
        std::env::set_var("TEST_STORE_TEST_KEY", "test-value");

        assert_eq!(
            factory.resolve_credential(&cred_ref),
            Some("test-value".to_string())
        );

        std::env::remove_var("TEST_STORE_TEST_KEY");
    }

    #[test]
    fn keyring_credential_store_fallback() {
        let store = KeyringCredentialStore;
        // Should fall back to env vars
        std::env::set_var("TEST_STORE_TEST_KEY", "test-value");
        let cred_ref = CredentialRef::new("TEST_STORE", "TEST_KEY");
        let result = store.resolve(&cred_ref);
        assert_eq!(result, Some("test-value".to_string()));
        std::env::remove_var("TEST_STORE_TEST_KEY");
    }
}
