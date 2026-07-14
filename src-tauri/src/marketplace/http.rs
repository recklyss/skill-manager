use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

const USER_AGENT: &str = "skill-manager/0.4";

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    pub fn absolute_url(&self, path_or_url: &str) -> String {
        if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
            return path_or_url.to_string();
        }
        format!(
            "{}/{}",
            self.base_url,
            path_or_url.trim_start_matches('/')
        )
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn fetch_json(&self, path_or_url: &str) -> Result<Value, String> {
        let url = self.absolute_url(path_or_url);
        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("network error: {e}"))?;
        if !response.status().is_success() {
            return Err(format!("upstream returned HTTP {}", response.status()));
        }
        response
            .json::<Value>()
            .await
            .map_err(|e| format!("invalid JSON payload: {e}"))
    }

    pub async fn fetch_text(&self, path_or_url: &str) -> Result<String, String> {
        let url = self.absolute_url(path_or_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("network error: {e}"))?;
        if !response.status().is_success() {
            return Err(format!("upstream returned HTTP {}", response.status()));
        }
        response
            .text()
            .await
            .map_err(|e| format!("failed to read response: {e}"))
    }
}

pub fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}
