use crate::db::scan_config::{mask_api_key, LlmScanConfigRow, ScanConfigRepository};
use crate::db::Database;
use crate::error::{ApiError, ApiResult};
use crate::scan::llm::validate_config_connectivity;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone)]
pub struct ScanConfigService {
    db: Arc<Database>,
}

impl ScanConfigService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    fn repo(&self) -> ScanConfigRepository {
        ScanConfigRepository::new(self.db.clone())
    }

    pub fn list_configs(&self) -> Value {
        let configs = self.repo().list_all().unwrap_or_default();
        let active_id = configs.iter().find(|c| c.is_active).and_then(|c| c.id);
        json!({
            "configs": configs.iter().map(config_to_item).collect::<Vec<_>>(),
            "activeId": active_id,
        })
    }

    pub fn reveal_secret(&self, config_id: i64) -> ApiResult<Value> {
        let config = self
            .repo()
            .get_by_id(config_id)
            .map_err(|e| ApiError::internal(e.to_string()))?
            .ok_or_else(|| ApiError::not_found(format!("Config {config_id} not found")))?;
        Ok(json!({ "apiKey": config.api_key }))
    }

    pub fn create_config(&self, body: &Value) -> ApiResult<Value> {
        let config = body_to_config(body, None, false, None)?;
        let validated = self.validate_and_stamp(&config)?;
        let id = self
            .repo()
            .save(&validated)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        let saved = self
            .repo()
            .get_by_id(id)
            .map_err(|e| ApiError::internal(e.to_string()))?
            .unwrap_or(validated);
        Ok(config_to_item(&saved))
    }

    pub fn update_config(&self, config_id: i64, body: &Value) -> ApiResult<Value> {
        let existing = self
            .repo()
            .get_by_id(config_id)
            .map_err(|e| ApiError::internal(e.to_string()))?
            .ok_or_else(|| ApiError::not_found(format!("Config {config_id} not found")))?;
        let api_key = body
            .get("apiKey")
            .and_then(|v| v.as_str())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or(existing.api_key.as_str())
            .to_string();
        let config = body_to_config(body, Some(config_id), existing.is_active, Some(api_key))?;
        let validated = self.validate_and_stamp(&config)?;
        self.repo()
            .save(&validated)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        let saved = self
            .repo()
            .get_by_id(config_id)
            .map_err(|e| ApiError::internal(e.to_string()))?
            .unwrap_or(validated);
        Ok(config_to_item(&saved))
    }

    pub fn delete_config(&self, config_id: i64) -> ApiResult<Value> {
        self.repo()
            .delete(config_id)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        Ok(json!({ "ok": true }))
    }

    pub fn set_active(&self, config_id: i64) -> ApiResult<Value> {
        if self
            .repo()
            .get_by_id(config_id)
            .map_err(|e| ApiError::internal(e.to_string()))?
            .is_none()
        {
            return Err(ApiError::not_found(format!("Config {config_id} not found")));
        }
        self.repo()
            .set_active(config_id)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        Ok(json!({ "ok": true }))
    }

    pub fn validate_config(&self, body: &Value) -> Value {
        let existing_id = body.get("existingConfigId").and_then(|v| v.as_i64());
        let mut api_key = body
            .get("apiKey")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if let Some(id) = existing_id {
            if api_key.is_empty() {
                match self.repo().get_by_id(id) {
                    Ok(Some(existing)) => api_key = existing.api_key,
                    Ok(None) => {
                        return json!({
                            "ok": false,
                            "message": format!("Config {id} not found."),
                            "errorCode": "config_not_found",
                        });
                    }
                    Err(err) => {
                        return json!({
                            "ok": false,
                            "message": err.to_string(),
                            "errorCode": "database_error",
                        });
                    }
                }
            }
        }
        let config = match body_to_config(body, existing_id, false, Some(api_key)) {
            Ok(c) => c,
            Err(err) => {
                return json!({
                    "ok": false,
                    "message": err.message,
                    "errorCode": "invalid_request",
                });
            }
        };
        validation_result_to_json(&validate_config_connectivity(&config))
    }

    fn validate_and_stamp(&self, config: &LlmScanConfigRow) -> ApiResult<LlmScanConfigRow> {
        let result = validate_config_connectivity(config);
        if !result.ok {
            return Err(ApiError::bad_request(result.message));
        }
        let mut stamped = config.clone();
        stamped.provider = result.provider.unwrap_or_else(|| config.provider.clone());
        stamped.last_validated_at = Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
        stamped.last_validation_error = String::new();
        Ok(stamped)
    }
}

fn config_to_item(config: &LlmScanConfigRow) -> Value {
    json!({
        "id": config.id,
        "name": config.name,
        "baseUrl": config.base_url,
        "apiKeyMasked": mask_api_key(&config.api_key),
        "model": config.model,
        "provider": config.provider,
        "apiVersion": config.api_version,
        "awsRegion": config.aws_region,
        "awsProfile": config.aws_profile,
        "maxTokens": config.max_tokens,
        "consensusRuns": config.consensus_runs,
        "isActive": config.is_active,
        "lastValidatedAt": config.last_validated_at,
        "lastValidationError": config.last_validation_error,
    })
}

fn body_to_config(
    body: &Value,
    config_id: Option<i64>,
    is_active: bool,
    api_key: Option<String>,
) -> ApiResult<LlmScanConfigRow> {
    Ok(LlmScanConfigRow {
        id: config_id,
        name: string_field(body, "name"),
        base_url: string_field(body, "baseUrl"),
        api_key: api_key.unwrap_or_else(|| string_field(body, "apiKey")),
        model: string_field(body, "model"),
        provider: string_field(body, "provider"),
        api_version: string_field(body, "apiVersion"),
        aws_region: string_field(body, "awsRegion"),
        aws_profile: string_field(body, "awsProfile"),
        aws_session_token: string_field(body, "awsSessionToken"),
        max_tokens: body.get("maxTokens").and_then(|v| v.as_i64()).unwrap_or(8192),
        consensus_runs: body.get("consensusRuns").and_then(|v| v.as_i64()).unwrap_or(1),
        is_active,
        last_validated_at: None,
        last_validation_error: String::new(),
    })
}

fn string_field(body: &Value, key: &str) -> String {
    body.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn validation_result_to_json(result: &crate::scan::llm::ValidationResult) -> Value {
    json!({
        "ok": result.ok,
        "message": result.message,
        "provider": result.provider,
        "model": result.model,
        "durationMs": result.duration_ms,
        "errorCode": result.error_code,
    })
}
