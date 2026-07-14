use super::cache::MarketplaceCache;
use super::http::{env_or_default, HttpClient};
use super::tokens::encode_install_token;
use crate::error::{ApiError, ApiResult};
use regex::Regex;
use serde_json::{json, Value};
use std::path::PathBuf;

const DEFAULT_BASE_URL: &str = "https://skills.sh";
const LEADERBOARD_TTL: u64 = 3600;
const SEARCH_TTL: u64 = 900;
const DETAIL_TTL: u64 = 86400;
const DETAIL_NS: &str = "details-v3";

#[derive(Clone)]
pub struct SkillsMarketplaceService {
    client: HttpClient,
    cache: MarketplaceCache,
}

impl SkillsMarketplaceService {
    pub fn new(cache_root: PathBuf) -> Self {
        let base = env_or_default("SKILL_MANAGER_MARKETPLACE_BASE_URL", DEFAULT_BASE_URL);
        Self {
            client: HttpClient::new(base),
            cache: MarketplaceCache::new(cache_root),
        }
    }

    pub async fn popular_page(&self, limit: Option<i64>, offset: i64) -> ApiResult<Value> {
        let records = self.leaderboard_records().await?;
        Ok(self.page(&records, limit, offset, false))
    }

    pub async fn search_page(&self, query: &str, limit: Option<i64>, offset: i64) -> ApiResult<Value> {
        let trimmed = query.trim();
        if trimmed.len() < 2 {
            return Err(ApiError::bad_request(
                "Enter at least 2 characters to search skills.sh.",
            ));
        }
        let page_limit = normalize_limit(limit);
        let fetch_limit = (offset + page_limit + 1).max(40);
        let records = self.search_records(trimmed, fetch_limit).await?;
        Ok(self.page(&records, Some(page_limit), offset, true))
    }

    pub async fn item_detail(&self, item_id: &str) -> ApiResult<Value> {
        let record = self
            .find_item(item_id)
            .await?
            .ok_or_else(|| ApiError::not_found(format!("unknown marketplace item: {item_id}")))?;
        Ok(json!({
            "id": item_id,
            "name": record.get("name").cloned().unwrap_or(Value::Null),
            "description": record.get("description").cloned().unwrap_or(json!("")),
            "installs": record.get("installs").cloned().unwrap_or(json!(0)),
            "stars": Value::Null,
            "repoLabel": record.get("repo").cloned().unwrap_or(json!("")),
            "repoImageUrl": Value::Null,
            "sourceLinks": {
                "repoLabel": record.get("repo").cloned().unwrap_or(json!("")),
                "repoUrl": github_repo_url(record.get("repo").and_then(|v| v.as_str()).unwrap_or("")),
                "folderUrl": Value::Null,
                "skillsDetailUrl": record.get("detailUrl").cloned().unwrap_or(json!("")),
            },
            "installation": { "status": "installable", "installedSkillRef": Value::Null },
            "installToken": record.get("installToken").cloned().unwrap_or(json!("")),
        }))
    }

    pub async fn item_document(&self, item_id: &str) -> ApiResult<Value> {
        let record = self
            .find_item(item_id)
            .await?
            .ok_or_else(|| ApiError::not_found(format!("unknown marketplace item: {item_id}")))?;
        let detail_url = record
            .get("detailUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if detail_url.is_empty() {
            return Ok(json!({
                "status": "unavailable",
                "documentMarkdown": Value::Null,
            }));
        }
        let cache_key = detail_url.to_string();
        if let Some(cached) = self.cache.read(DETAIL_NS, &cache_key, DETAIL_TTL) {
            return Ok(cached);
        }
        let html = self.client.fetch_text(detail_url).await.map_err(ApiError::internal)?;
        let markdown = extract_detail_description(&html);
        let payload = json!({
            "status": if markdown.is_empty() { "unavailable" } else { "available" },
            "documentMarkdown": if markdown.is_empty() { Value::Null } else { json!(markdown) },
        });
        self.cache.write(DETAIL_NS, &cache_key, &payload);
        Ok(payload)
    }

    async fn leaderboard_records(&self) -> Result<Vec<Value>, ApiError> {
        let cache_key = "leaderboard-v1".to_string();
        if let Some(cached) = self.cache.read("leaderboard-v1", &cache_key, LEADERBOARD_TTL) {
            if let Some(items) = cached.as_array() {
                return Ok(items.clone());
            }
        }
        let html = self
            .client
            .fetch_text("/")
            .await
            .map_err(ApiError::internal)?;
        let records = parse_homepage_leaderboard(&html, &self.client.base_url())?;
        self.cache.write("leaderboard-v1", &cache_key, &json!(records));
        Ok(records)
    }

    async fn search_records(&self, query: &str, limit: i64) -> Result<Vec<Value>, ApiError> {
        let cache_key = format!("search:{query}:{limit}");
        if let Some(cached) = self.cache.read("search-v1", &cache_key, SEARCH_TTL) {
            if let Some(items) = cached.as_array() {
                return Ok(items.clone());
            }
        }
        let path = format!("/api/search?q={}&limit={}", urlencoding_encode(query), limit);
        let payload = self.client.fetch_json(&path).await.map_err(ApiError::internal)?;
        let records = normalize_search_payload(&payload, &self.client.base_url());
        self.cache.write("search-v1", &cache_key, &json!(records));
        Ok(records)
    }

    async fn find_item(&self, item_id: &str) -> Result<Option<Value>, ApiError> {
        if let Some((repo, skill_id)) = parse_item_id(item_id) {
            let detail_url = format!(
                "{}/{}/{}",
                self.client.base_url(),
                urlencoding_encode(&repo),
                urlencoding_encode(&skill_id)
            );
            return Ok(Some(json!({
                "repo": repo,
                "skillId": skill_id,
                "name": skill_id,
                "detailUrl": detail_url,
                "installs": 0,
                "description": "",
                "installToken": encode_install_token("github", &format!("github:{repo}/{skill_id}")),
            })));
        }
        let records = self.leaderboard_records().await?;
        Ok(records
            .into_iter()
            .find(|r| r.get("id").and_then(|v| v.as_str()) == Some(item_id)))
    }

    fn page(&self, records: &[Value], limit: Option<i64>, offset: i64, prefer_hints: bool) -> Value {
        let page_size = normalize_limit(limit);
        let page_offset = offset.max(0);
        let end = (page_offset + page_size) as usize;
        let start = page_offset as usize;
        let slice = records.get(start..end).unwrap_or(&[]);
        let has_more = records.len() > end;
        let items: Vec<Value> = slice
            .iter()
            .map(|record| self.card_from_record(record, prefer_hints))
            .collect();
        let next_offset = if has_more && !items.is_empty() {
            Some(page_offset + items.len() as i64)
        } else {
            None
        };
        json!({
            "items": items,
            "nextOffset": next_offset,
            "hasMore": next_offset.is_some(),
        })
    }

    fn card_from_record(&self, record: &Value, prefer_hints: bool) -> Value {
        let repo = record.get("repo").and_then(|v| v.as_str()).unwrap_or("");
        let skill_id = record
            .get("skillId")
            .or_else(|| record.get("skill_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let name = record.get("name").and_then(|v| v.as_str()).unwrap_or(skill_id);
        let description = if prefer_hints {
            record
                .get("descriptionHint")
                .or_else(|| record.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
        } else {
            record.get("description").and_then(|v| v.as_str()).unwrap_or("")
        };
        let item_id = format!("github:{repo}/{skill_id}");
        json!({
            "id": item_id,
            "name": name,
            "description": description,
            "installs": record.get("installs").cloned().unwrap_or(json!(0)),
            "repoLabel": repo,
            "detailUrl": record.get("detailUrl").cloned().unwrap_or(json!(format!("{}/{}/{}", self.client.base_url(), repo, skill_id))),
            "installToken": encode_install_token("github", &format!("github:{repo}/{skill_id}")),
            "installation": { "status": "installable", "installedSkillRef": Value::Null },
        })
    }
}

fn normalize_limit(limit: Option<i64>) -> i64 {
    match limit {
        None => 20,
        Some(v) => v.clamp(1, 60),
    }
}

fn parse_item_id(item_id: &str) -> Option<(String, String)> {
    let rest = item_id.strip_prefix("github:")?;
    let (repo, skill_id) = rest.rsplit_once('/')?;
    if repo.is_empty() || skill_id.is_empty() {
        return None;
    }
    Some((repo.to_string(), skill_id.to_string()))
}

fn github_repo_url(repo: &str) -> String {
    format!("https://github.com/{repo}")
}

fn urlencoding_encode(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            '/' => "%2F".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn parse_homepage_leaderboard(html: &str, base_url: &str) -> Result<Vec<Value>, ApiError> {
    let marker = "initialSkills";
    let marker_index = html.find(marker).ok_or_else(|| {
        ApiError::internal("skills.sh homepage is missing the initial leaderboard payload")
    })?;
    let array_start = html[marker_index..]
        .find('[')
        .ok_or_else(|| ApiError::internal("skills.sh homepage leaderboard payload is malformed"))?
        + marker_index;
    let slice = &html[array_start..];
    let mut depth = 0usize;
    let mut end = None;
    for (idx, ch) in slice.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = Some(idx + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.ok_or_else(|| ApiError::internal("skills.sh homepage leaderboard payload is malformed"))?;
    let raw: Value = parse_embedded_json_payload(&slice[..end])
        .map_err(|e| ApiError::internal(format!("invalid leaderboard JSON: {e}")))?;
    let items = raw.as_array().cloned().unwrap_or_default();
    Ok(items
        .into_iter()
        .filter_map(|item| normalize_leaderboard_item(&item, base_url))
        .collect())
}

fn parse_embedded_json_payload(raw: &str) -> Result<Value, serde_json::Error> {
    match serde_json::from_str(raw) {
        Ok(value) => Ok(value),
        Err(_) => serde_json::from_str(&decode_unicode_escape(raw)),
    }
}

/// Decode JavaScript-style escape sequences embedded in skills.sh flight payloads.
fn decode_unicode_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('b') => out.push('\x08'),
            Some('f') => out.push('\x0c'),
            Some('u') => {
                let hex: String = chars.by_ref().take(4).collect();
                if hex.len() == 4 {
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(decoded) = char::from_u32(code) {
                            out.push(decoded);
                            continue;
                        }
                    }
                }
                out.push('\\');
                out.push('u');
                out.extend(hex.chars());
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn skill_id_from_payload(item: &Value) -> Option<&str> {
    item.get("skillId")
        .or_else(|| item.get("skill_id"))
        .or_else(|| item.get("id"))
        .and_then(|v| v.as_str())
        .filter(|value| !value.is_empty())
}

fn normalize_leaderboard_item(item: &Value, base_url: &str) -> Option<Value> {
    let source = item.get("source")?.as_str()?;
    let skill_id = skill_id_from_payload(item)?;
    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or(skill_id);
    let installs = item.get("installs").and_then(|v| v.as_i64()).unwrap_or(0);
    Some(json!({
        "repo": source,
        "skillId": skill_id,
        "name": name,
        "installs": installs,
        "description": item.get("description").cloned().unwrap_or(json!("")),
        "detailUrl": format!("{base_url}/{source}/{skill_id}"),
        "id": format!("github:{source}/{skill_id}"),
    }))
}

fn normalize_search_payload(payload: &Value, base_url: &str) -> Vec<Value> {
    let Some(items) = payload.get("skills").and_then(|v| v.as_array()) else {
        return vec![];
    };
    items
        .iter()
        .filter_map(|item| {
            let source = item.get("source")?.as_str()?;
            let skill_id = skill_id_from_payload(item)?;
            Some(json!({
                "repo": source,
                "skillId": skill_id,
                "name": item.get("name").cloned().unwrap_or(json!(skill_id)),
                "installs": item.get("installs").cloned().unwrap_or(json!(0)),
                "descriptionHint": item.get("description").cloned().unwrap_or(json!("")),
                "detailUrl": format!("{base_url}/{source}/{skill_id}"),
                "id": format!("github:{source}/{skill_id}"),
            }))
        })
        .collect()
}

fn extract_detail_description(html: &str) -> String {
    let re = Regex::new(r"(?s)<main[^>]*>(.*?)</main>").ok();
    let main = re
        .and_then(|r| r.captures(html))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or(html);
    let stripped = Regex::new(r"<[^>]+>")
        .ok()
        .map(|r| r.replace_all(main, " ").to_string())
        .unwrap_or_else(|| main.to_string());
    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_homepage_leaderboard_reads_escaped_next_payload() {
        let html = r#"
        <html>
          <body>
            <script>
              self.__next_f.push([1,"{\"initialSkills\":[{\"source\":\"mode-io/skills\",\"skillId\":\"mode-switch\",\"name\":\"Mode Switch\",\"installs\":128},{\"source\":\"vercel-labs/skills\",\"skillId\":\"trace-scout\",\"name\":\"Trace Scout\",\"installs\":84}]}"])
            </script>
          </body>
        </html>
        "#;
        let records = parse_homepage_leaderboard(html, "https://skills.sh").expect("parse");
        assert_eq!(records.len(), 2);
        assert_eq!(
            records[0].get("id").and_then(|v| v.as_str()),
            Some("github:mode-io/skills/mode-switch")
        );
        assert_eq!(
            records[1].get("skillId").and_then(|v| v.as_str()),
            Some("trace-scout")
        );
    }

    #[test]
    fn parse_homepage_leaderboard_reads_unescaped_script_payload() {
        let html = concat!(
            "<html><body><script>const initialSkills = ",
            r#"[{"source":"mode-io/skills","skillId":"mode-switch","name":"Mode Switch","installs":128}]"#,
            ";</script></body></html>"
        );
        let records = parse_homepage_leaderboard(html, "https://skills.sh").expect("parse");
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].get("repo").and_then(|v| v.as_str()),
            Some("mode-io/skills")
        );
    }

    #[test]
    fn normalize_search_payload_prefers_skill_id_field() {
        let payload = json!({
            "skills": [{
                "source": "github/awesome-copilot",
                "id": "github/awesome-copilot/model-recommendation",
                "skillId": "model-recommendation",
                "name": "model-recommendation",
                "installs": 8507
            }]
        });
        let records = normalize_search_payload(&payload, "https://skills.sh");
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].get("detailUrl").and_then(|v| v.as_str()),
            Some("https://skills.sh/github/awesome-copilot/model-recommendation")
        );
    }
}
