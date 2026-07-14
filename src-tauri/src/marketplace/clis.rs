use super::cache::MarketplaceCache;
use super::http::{env_or_default, HttpClient};
use crate::error::{ApiError, ApiResult};
use serde_json::{json, Value};
use std::path::PathBuf;

const DEFAULT_BASE_URL: &str = "https://clis.dev";
const POPULAR_TTL: u64 = 3600;
const SEARCH_TTL: u64 = 900;
const POPULAR_NS: &str = "clisdev-popular-v1";
const SEARCH_NS: &str = "clisdev-search-v1";

#[derive(Clone)]
pub struct CliMarketplaceService {
    client: HttpClient,
    cache: MarketplaceCache,
}

impl CliMarketplaceService {
    pub fn new(cache_root: PathBuf) -> Self {
        let base = env_or_default("SKILL_MANAGER_CLIS_DEV_BASE_URL", DEFAULT_BASE_URL);
        Self {
            client: HttpClient::new(base),
            cache: MarketplaceCache::new(cache_root),
        }
    }

    pub async fn popular_page(&self, limit: Option<i64>, offset: i64) -> ApiResult<Value> {
        let records = self.known_records().await?;
        Ok(self.page(&records, limit, offset))
    }

    pub async fn search_page(&self, query: &str, limit: Option<i64>, offset: i64) -> ApiResult<Value> {
        let trimmed = query.trim();
        if trimmed.len() < 2 {
            return Err(ApiError::bad_request(
                "Enter at least 2 characters to search CLIs.dev.",
            ));
        }
        let cache_key = trimmed.to_lowercase();
        let records = if let Some(cached) = self.cache.read(SEARCH_NS, &cache_key, SEARCH_TTL) {
            cached
                .get("records")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            let payload = self
                .client
                .fetch_json(&format!("/api/search?q={}", url_encode(trimmed)))
                .await
                .map_err(ApiError::internal)?;
            let records = records_from_payload(&payload);
            self.cache.write(
                SEARCH_NS,
                &cache_key,
                &json!({ "records": records }),
            );
            records
        };
        Ok(self.page(&records, limit, offset))
    }

    pub async fn detail(&self, slug: &str) -> ApiResult<Value> {
        let records = self.known_records().await?;
        let record = records
            .into_iter()
            .find(|r| r.get("slug").and_then(|v| v.as_str()) == Some(slug))
            .ok_or_else(|| ApiError::not_found(format!("unknown CLI: {slug}")))?;
        Ok(detail_from_record(&record))
    }

    async fn known_records(&self) -> Result<Vec<Value>, ApiError> {
        let cache_key = "all".to_string();
        if let Some(cached) = self.cache.read(POPULAR_NS, &cache_key, POPULAR_TTL) {
            if let Some(items) = cached.get("records").and_then(|v| v.as_array()) {
                return Ok(items.clone());
            }
        }
        let payload = self
            .client
            .fetch_json("/api/clis")
            .await
            .map_err(ApiError::internal)?;
        let records = records_from_payload(&payload);
        self.cache
            .write(POPULAR_NS, &cache_key, &json!({ "records": records }));
        Ok(records)
    }

    fn page(&self, records: &[Value], limit: Option<i64>, offset: i64) -> Value {
        let page_size = normalize_limit(limit);
        let start = offset.max(0) as usize;
        let end = start + page_size as usize;
        let slice = records.get(start..end).unwrap_or(&[]);
        let items: Vec<Value> = slice.iter().map(item_from_record).collect();
        let has_more = records.len() > end;
        let next_offset = if has_more && !items.is_empty() {
            Some(offset + items.len() as i64)
        } else {
            None
        };
        json!({
            "items": items,
            "nextOffset": next_offset,
            "hasMore": next_offset.is_some(),
        })
    }
}

fn normalize_limit(limit: Option<i64>) -> i64 {
    match limit {
        None => 30,
        Some(v) => v.clamp(1, 100),
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

fn records_from_payload(payload: &Value) -> Vec<Value> {
    let items = payload
        .get("clis")
        .or_else(|| payload.get("results"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    items
        .into_iter()
        .filter_map(|item| normalize_record(&item))
        .collect()
}

fn normalize_record(payload: &Value) -> Option<Value> {
    let slug = payload
        .get("slug")
        .or_else(|| payload.get("id"))
        .and_then(|v| v.as_str())?;
    Some(json!({
        "slug": slug,
        "name": payload.get("name").cloned().unwrap_or(json!(slug)),
        "description": payload.get("description").cloned().unwrap_or(json!("")),
        "longDescription": payload.get("longDescription").cloned().unwrap_or(Value::Null),
        "marketplaceUrl": payload.get("url").cloned().unwrap_or(json!(format!("https://clis.dev/cli/{slug}"))),
        "iconUrl": payload.get("iconUrl").cloned().unwrap_or(Value::Null),
        "githubUrl": payload.get("githubUrl").cloned().unwrap_or(Value::Null),
        "websiteUrl": payload.get("websiteUrl").cloned().unwrap_or(Value::Null),
        "stars": payload.get("stars").cloned().unwrap_or(json!(0)),
        "language": payload.get("language").cloned().unwrap_or(Value::Null),
        "category": payload.get("category").cloned().unwrap_or(Value::Null),
        "installCommand": payload.get("installCommand").cloned().unwrap_or(Value::Null),
        "hasMcp": payload.get("hasMcp").and_then(|v| v.as_bool()).unwrap_or(false),
        "hasSkill": payload.get("hasSkill").and_then(|v| v.as_bool()).unwrap_or(false),
        "isOfficial": payload.get("isOfficial").and_then(|v| v.as_bool()).unwrap_or(false),
        "isTui": payload.get("isTui").and_then(|v| v.as_bool()).unwrap_or(false),
        "sourceType": payload.get("sourceType").cloned().unwrap_or(Value::Null),
        "vendorName": payload.get("vendorName").cloned().unwrap_or(Value::Null),
        "platforms": payload.get("platforms").cloned().unwrap_or(json!([])),
        "categories": payload.get("categories").cloned().unwrap_or(json!([])),
        "readmeMarkdown": payload.get("readmeMarkdown").cloned().unwrap_or(json!("")),
    }))
}

fn item_from_record(record: &Value) -> Value {
    let slug = record.get("slug").and_then(|v| v.as_str()).unwrap_or("");
    json!({
        "id": format!("clisdev:{slug}"),
        "slug": slug,
        "name": record.get("name").cloned().unwrap_or(json!(slug)),
        "description": record.get("description").cloned().unwrap_or(json!("")),
        "marketplaceUrl": record.get("marketplaceUrl").cloned().unwrap_or(json!(format!("https://clis.dev/cli/{slug}"))),
        "iconUrl": record.get("iconUrl").cloned().unwrap_or(Value::Null),
        "githubUrl": record.get("githubUrl").cloned().unwrap_or(Value::Null),
        "websiteUrl": record.get("websiteUrl").cloned().unwrap_or(Value::Null),
        "stars": record.get("stars").cloned().unwrap_or(json!(0)),
        "language": record.get("language").cloned().unwrap_or(Value::Null),
        "category": record.get("category").cloned().unwrap_or(Value::Null),
        "hasMcp": record.get("hasMcp").cloned().unwrap_or(json!(false)),
        "hasSkill": record.get("hasSkill").cloned().unwrap_or(json!(false)),
        "isOfficial": record.get("isOfficial").cloned().unwrap_or(json!(false)),
        "isTui": record.get("isTui").cloned().unwrap_or(json!(false)),
        "sourceType": record.get("sourceType").cloned().unwrap_or(Value::Null),
        "vendorName": record.get("vendorName").cloned().unwrap_or(Value::Null),
    })
}

fn detail_from_record(record: &Value) -> Value {
    let slug = record.get("slug").and_then(|v| v.as_str()).unwrap_or("");
    let mut detail = item_from_record(record);
    if let Some(obj) = detail.as_object_mut() {
        obj.insert(
            "homepage".into(),
            record
                .get("websiteUrl")
                .cloned()
                .unwrap_or(json!("")),
        );
        obj.insert(
            "installs".into(),
            json!(0),
        );
        obj.insert(
            "command".into(),
            record
                .get("installCommand")
                .cloned()
                .unwrap_or(json!("")),
        );
        obj.insert(
            "platforms".into(),
            record.get("platforms").cloned().unwrap_or(json!([])),
        );
        obj.insert(
            "categories".into(),
            record.get("categories").cloned().unwrap_or(json!([])),
        );
        obj.insert(
            "sourceLinks".into(),
            json!({
                "repoLabel": record.get("githubUrl").and_then(|v| v.as_str()).unwrap_or(""),
                "repoUrl": record.get("githubUrl").cloned().unwrap_or(json!("")),
                "folderUrl": Value::Null,
            }),
        );
        obj.insert(
            "readmeMarkdown".into(),
            record.get("readmeMarkdown").cloned().unwrap_or(json!("")),
        );
        obj.insert("longDescription".into(), record.get("longDescription").cloned().unwrap_or(Value::Null));
        obj.insert("installCommand".into(), record.get("installCommand").cloned().unwrap_or(Value::Null));
        let _ = slug;
    }
    detail
}
