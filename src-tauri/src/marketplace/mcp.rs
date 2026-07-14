use super::cache::MarketplaceCache;
use super::http::{env_or_default, HttpClient};
use crate::error::{ApiError, ApiResult};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use std::path::PathBuf;

const DEFAULT_REGISTRY_URL: &str = "https://registry.modelcontextprotocol.io";
const API_VERSION: &str = "v0.1";
const PAGE_TTL: u64 = 3600;
const DETAIL_TTL: u64 = 86400;
const PAGE_NS: &str = "mcp-registry-page-v1";
const DETAIL_NS: &str = "mcp-registry-detail-v1";

#[derive(Clone)]
pub struct McpMarketplaceService {
    client: HttpClient,
    cache: MarketplaceCache,
}

impl McpMarketplaceService {
    pub fn new(cache_root: PathBuf) -> Self {
        let base = env_or_default(
            "SKILL_MANAGER_MCP_REGISTRY_BASE_URL",
            DEFAULT_REGISTRY_URL,
        );
        Self {
            client: HttpClient::new(base),
            cache: MarketplaceCache::new(cache_root),
        }
    }

    pub async fn popular_page(&self, limit: Option<i64>, offset: i64) -> ApiResult<Value> {
        let page_size = normalize_limit(limit);
        let fetch_limit = offset + page_size + 1;
        let (items, maybe_more) = self.collect_items("", fetch_limit, None, None).await?;
        let start = offset.max(0) as usize;
        let end = start + page_size as usize;
        let page_items: Vec<Value> = items.get(start..end).unwrap_or(&[]).to_vec();
        let has_more = items.len() > end || maybe_more;
        let next_offset = if has_more && !page_items.is_empty() {
            Some(offset + page_items.len() as i64)
        } else {
            None
        };
        Ok(json!({
            "items": page_items,
            "nextOffset": next_offset,
            "hasMore": next_offset.is_some(),
        }))
    }

    pub async fn search_page(
        &self,
        query: &str,
        limit: Option<i64>,
        offset: i64,
        remote: Option<bool>,
        verified: Option<bool>,
    ) -> ApiResult<Value> {
        let trimmed = query.trim();
        if trimmed.len() < 2 && remote.is_none() && verified.is_none() {
            return Err(ApiError::bad_request(
                "Enter at least 2 characters to search the MCP registry.",
            ));
        }
        let page_size = normalize_limit(limit);
        let fetch_limit = (offset + page_size + 1).max(40);
        let (items, maybe_more) = self
            .collect_items(trimmed, fetch_limit, remote, verified)
            .await?;
        let start = offset.max(0) as usize;
        let end = start + page_size as usize;
        let page_items: Vec<Value> = items.get(start..end).unwrap_or(&[]).to_vec();
        let has_more = items.len() > end || maybe_more;
        let next_offset = if has_more && !page_items.is_empty() {
            Some(offset + page_items.len() as i64)
        } else {
            None
        };
        Ok(json!({
            "items": page_items,
            "nextOffset": next_offset,
            "hasMore": next_offset.is_some(),
        }))
    }

    pub async fn detail(&self, qualified_name: &str) -> ApiResult<Value> {
        let name = qualified_name.trim();
        if name.is_empty() {
            return Err(ApiError::not_found(format!("unknown MCP server: {qualified_name}")));
        }
        if let Some(cached) = self.cache.read(DETAIL_NS, name, DETAIL_TTL) {
            let mut payload = cached;
            if let Some(obj) = payload.as_object_mut() {
                obj.remove("registryServer");
                obj.insert("externalUrl".into(), json!(external_url(name)));
            }
            return Ok(payload);
        }
        let versions = self
            .fetch_registry(&format!("/{API_VERSION}/servers/{}/versions", url_encode(name)))
            .await?;
        let latest = latest_active_entry(&versions).ok_or_else(|| {
            ApiError::not_found(format!("unknown MCP server: {qualified_name}"))
        })?;
        let server = entry_server(&latest).ok_or_else(|| {
            ApiError::not_found(format!("unknown MCP server: {qualified_name}"))
        })?;
        let version = server
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if version.is_empty() {
            return Err(ApiError::not_found(format!("unknown MCP server: {qualified_name}")));
        }
        let raw = self
            .fetch_registry(&format!(
                "/{API_VERSION}/servers/{}/versions/{}",
                url_encode(name),
                url_encode(version)
            ))
            .await?;
        let payload = map_detail(&raw, name, server);
        self.cache.write(DETAIL_NS, name, &payload);
        let mut response = payload;
        if let Some(obj) = response.as_object_mut() {
            obj.remove("registryServer");
            obj.insert("externalUrl".into(), json!(external_url(name)));
        }
        Ok(response)
    }

    pub async fn install_detail(&self, qualified_name: &str) -> ApiResult<Option<Value>> {
        match self.detail(qualified_name).await {
            Ok(detail) => Ok(Some(detail)),
            Err(ApiError { status, .. }) if status == axum::http::StatusCode::NOT_FOUND => Ok(None),
            Err(err) => Err(err),
        }
    }

    async fn collect_items(
        &self,
        query: &str,
        limit: i64,
        remote: Option<bool>,
        verified: Option<bool>,
    ) -> Result<(Vec<Value>, bool), ApiError> {
        let mut collected = Vec::new();
        let mut cursor: Option<String> = None;
        let mut maybe_more = false;
        while (collected.len() as i64) < limit {
            let raw = self.list_registry_page(query, cursor.as_deref()).await?;
            for entry in entries(&raw) {
                if let Some(item) = map_summary_item(&entry) {
                    if item_matches_filters(&item, query, remote, verified) {
                        collected.push(item);
                        if collected.len() as i64 >= limit {
                            break;
                        }
                    }
                }
            }
            cursor = next_cursor(&raw);
            maybe_more = cursor.is_some();
            if cursor.is_none() {
                break;
            }
        }
        Ok((collected, maybe_more))
    }

    async fn list_registry_page(&self, query: &str, cursor: Option<&str>) -> Result<Value, ApiError> {
        let mut path = format!("/{API_VERSION}/servers?limit=60");
        if let Some(c) = cursor {
            path.push_str(&format!("&cursor={}", url_encode(c)));
        }
        if !query.trim().is_empty() {
            path.push_str(&format!("&search={}", url_encode(query.trim())));
        }
        self.fetch_registry(&path).await
    }

    async fn fetch_registry(&self, path: &str) -> Result<Value, ApiError> {
        let cache_key = format!("{:x}", Sha1::digest(path.as_bytes()));
        if let Some(cached) = self.cache.read(PAGE_NS, &cache_key, PAGE_TTL) {
            return Ok(cached);
        }
        let raw = self.client.fetch_json(path).await.map_err(ApiError::internal)?;
        self.cache.write(PAGE_NS, &cache_key, &raw);
        Ok(raw)
    }
}

fn normalize_limit(limit: Option<i64>) -> i64 {
    match limit {
        None => 20,
        Some(v) => v.clamp(1, 60),
    }
}

fn url_encode(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn entries(raw: &Value) -> Vec<&Value> {
    raw.get("servers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().collect())
        .unwrap_or_default()
}

fn entry_server(entry: &Value) -> Option<&Value> {
    entry.get("server")
}

fn latest_active_entry(raw: &Value) -> Option<Value> {
    if let Some(entry) = entries(raw).into_iter().find(|entry| {
        entry
            .get("meta")
            .and_then(|m| m.get("isLatest"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }) {
        return Some(entry.clone());
    }
    entries(raw).first().map(|v| (*v).clone())
}

fn next_cursor(raw: &Value) -> Option<String> {
    raw.get("metadata")
        .and_then(|m| m.get("nextCursor"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn map_summary_item(entry: &Value) -> Option<Value> {
    let server = entry_server(entry)?;
    let name = server
        .get("name")
        .or_else(|| server.get("title"))
        .and_then(|v| v.as_str())?;
    let description = server
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Some(json!({
        "qualifiedName": name,
        "displayName": server.get("title").and_then(|v| v.as_str()).unwrap_or(name),
        "description": description,
        "repository": server.get("repository").cloned().unwrap_or(json!("")),
        "verified": entry.get("meta").and_then(|m| m.get("official")).and_then(|v| v.as_bool()).unwrap_or(false),
        "remote": server.get("remotes").map(|v| !v.as_array().map(|a| a.is_empty()).unwrap_or(true)).unwrap_or(false),
    }))
}

fn item_matches_filters(item: &Value, query: &str, remote: Option<bool>, verified: Option<bool>) -> bool {
    if let Some(expected) = remote {
        let actual = item.get("remote").and_then(|v| v.as_bool()).unwrap_or(false);
        if actual != expected {
            return false;
        }
    }
    if let Some(expected) = verified {
        let actual = item.get("verified").and_then(|v| v.as_bool()).unwrap_or(false);
        if actual != expected {
            return false;
        }
    }
    let q = query.trim().to_lowercase();
    if q.len() < 2 {
        return true;
    }
    let haystacks = [
        item.get("qualifiedName").and_then(|v| v.as_str()).unwrap_or(""),
        item.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
        item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
    ];
    haystacks.iter().any(|h| h.to_lowercase().contains(&q))
}

fn map_detail(raw: &Value, qualified_name: &str, server: &Value) -> Value {
    let tools = raw
        .get("tools")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let resources = raw
        .get("resources")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let prompts = raw
        .get("prompts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    json!({
        "qualifiedName": qualified_name,
        "displayName": server.get("title").and_then(|v| v.as_str()).unwrap_or(qualified_name),
        "description": server.get("description").and_then(|v| v.as_str()).unwrap_or(""),
        "repository": server.get("repository").and_then(|v| v.as_str()).unwrap_or(""),
        "homepageUrl": server.get("websiteUrl").cloned().unwrap_or(Value::Null),
        "connection": server.get("packages").cloned().unwrap_or(Value::Null),
        "toolCount": tools.len(),
        "resourceCount": resources.len(),
        "promptCount": prompts.len(),
        "tools": tools,
        "resources": resources,
        "prompts": prompts,
        "capabilityCounts": {
            "tools": tools.len(),
            "resources": resources.len(),
            "prompts": prompts.len(),
        },
        "installation": { "status": "installable", "managedName": Value::Null },
        "registryServer": server,
        "externalUrl": external_url(qualified_name),
    })
}

fn external_url(name: &str) -> String {
    format!("https://registry.modelcontextprotocol.io/servers/{name}")
}
