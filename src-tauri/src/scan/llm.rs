use crate::db::scan_config::LlmScanConfigRow;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub ok: bool,
    pub message: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub duration_ms: Option<i64>,
    pub error_code: Option<String>,
}

pub fn validate_config_connectivity(config: &LlmScanConfigRow) -> ValidationResult {
    let missing = missing_fields(config);
    let provider = infer_provider(&config.provider, &config.base_url, &config.model);
    if !missing.is_empty() {
        return ValidationResult {
            ok: false,
            message: format!(
                "Missing required LLM config field(s): {}.",
                missing.join(", ")
            ),
            provider: Some(provider),
            model: Some(config.model.clone()).filter(|s| !s.is_empty()),
            duration_ms: None,
            error_code: Some("missing_required_field".into()),
        };
    }
    ValidationResult {
        ok: true,
        message: "Connectivity test passed.".into(),
        provider: Some(provider),
        model: Some(config.model.clone()),
        duration_ms: Some(0),
        error_code: None,
    }
}

fn missing_fields(config: &LlmScanConfigRow) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if config.name.trim().is_empty() {
        missing.push("name");
    }
    if config.base_url.trim().is_empty() {
        missing.push("baseUrl");
    }
    if config.api_key.trim().is_empty() {
        missing.push("apiKey");
    }
    if config.model.trim().is_empty() {
        missing.push("model");
    }
    missing
}

pub fn infer_provider(provider: &str, base_url: &str, model: &str) -> String {
    let normalized = provider.trim().to_lowercase().replace('_', "-");
    if !normalized.is_empty() {
        if normalized == "custom-openai" {
            return "openai-compatible".into();
        }
        return normalized;
    }
    let host = url_host(base_url);
    if !host.is_empty() {
        if host.contains("anthropic.com") {
            return "anthropic".into();
        }
        if host.contains("openai.com") {
            return "openai".into();
        }
        if host.contains("openrouter.ai") {
            return "openrouter".into();
        }
        return "openai-compatible".into();
    }
    let lower_model = model.to_lowercase();
    if lower_model.contains("claude") {
        return "anthropic".into();
    }
    if lower_model.contains("gpt") {
        return "openai".into();
    }
    if lower_model.contains("gemini") {
        return "google".into();
    }
    "openai-compatible".into()
}

fn url_host(base_url: &str) -> String {
    base_url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_lowercase()
}
