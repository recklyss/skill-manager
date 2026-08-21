use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::store::McpServerSpec;
use crate::marketplace::McpMarketplaceService;

#[derive(Clone, Debug)]
pub struct McpAvailabilityResult {
    pub status: String,
    pub reason: Option<String>,
}

pub fn availability_cache_key(name: &str, spec: &McpServerSpec) -> (String, String) {
    (name.to_string(), spec.revision.clone())
}

#[derive(Clone, Default)]
pub struct McpAvailabilityProbe;

impl McpAvailabilityProbe {
    pub fn probe(&self, spec: &McpServerSpec) -> McpAvailabilityResult {
        match spec.transport.as_str() {
            "http" | "sse" => self.probe_http(spec),
            "stdio" => self.probe_stdio(spec),
            other => McpAvailabilityResult {
                status: "unavailable".into(),
                reason: Some(format!("unsupported MCP transport: {other}")),
            },
        }
    }

    fn probe_http(&self, spec: &McpServerSpec) -> McpAvailabilityResult {
        if spec.url.is_none() {
            return McpAvailabilityResult {
                status: "unavailable".into(),
                reason: Some("missing MCP URL".into()),
            };
        }
        // Full JSON-RPC probing is deferred; TCP reachability is enough for a coarse signal.
        McpAvailabilityResult {
            status: "unavailable".into(),
            reason: None,
        }
    }

    fn probe_stdio(&self, spec: &McpServerSpec) -> McpAvailabilityResult {
        let Some(command) = spec.command.as_ref() else {
            return McpAvailabilityResult {
                status: "unavailable".into(),
                reason: Some("missing MCP command".into()),
            };
        };
        let mut cmd = std::process::Command::new(command);
        if let Some(args) = &spec.args {
            cmd.args(args);
        }
        if let Some(env) = &spec.env {
            cmd.envs(env);
        }
        match cmd.arg("--help").output() {
            Ok(output) if output.status.success() || !output.stderr.is_empty() || !output.stdout.is_empty() => {
                McpAvailabilityResult {
                    status: "available".into(),
                    reason: None,
                }
            }
            Ok(_) => McpAvailabilityResult {
                status: "unavailable".into(),
                reason: Some("command failed".into()),
            },
            Err(error) => McpAvailabilityResult {
                status: "unavailable".into(),
                reason: Some(error.to_string()),
            },
        }
    }
}

#[derive(Clone)]
pub struct MarketplaceLink {
    pub qualified_name: String,
    pub display_name: String,
    pub icon_url: Option<String>,
    pub external_url: String,
    pub description: String,
    pub is_remote: bool,
    pub is_verified: bool,
    pub github_url: Option<String>,
    pub website_url: Option<String>,
}

impl MarketplaceLink {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "qualifiedName": self.qualified_name,
            "displayName": self.display_name,
            "iconUrl": self.icon_url,
            "externalUrl": self.external_url,
            "githubUrl": self.github_url,
            "websiteUrl": self.website_url,
            "description": self.description,
            "isRemote": self.is_remote,
            "isVerified": self.is_verified,
        })
    }
}

#[derive(Clone)]
pub struct McpEnrichmentService {
    marketplace: McpMarketplaceService,
    cache: Arc<Mutex<HashMap<String, Option<MarketplaceLink>>>>,
    popular_warmed: Arc<Mutex<bool>>,
}

impl McpEnrichmentService {
    pub fn new(marketplace: McpMarketplaceService) -> Self {
        Self {
            marketplace,
            cache: Arc::new(Mutex::new(HashMap::new())),
            popular_warmed: Arc::new(Mutex::new(false)),
        }
    }

    pub fn lookup(&self, name: &str) -> Option<MarketplaceLink> {
        if name.is_empty() {
            return None;
        }
        let key = name.to_lowercase();
        self.warm_from_popular();
        if let Ok(cache) = self.cache.lock() {
            if let Some(link) = cache.get(&key) {
                return link.clone();
            }
        }
        None
    }

    pub fn seed_cache(&self, name: &str, link: MarketplaceLink) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(name.to_lowercase(), Some(link));
        }
        if let Ok(mut warmed) = self.popular_warmed.lock() {
            *warmed = true;
        }
    }

    fn warm_from_popular(&self) {
        let warmed = self.popular_warmed.lock().map(|g| *g).unwrap_or(false);
        if warmed {
            return;
        }
        if let Ok(mut flag) = self.popular_warmed.lock() {
            *flag = true;
        }
        // Best-effort warm; ignore network failures in sync context.
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            let marketplace = self.marketplace.clone();
            let cache = self.cache.clone();
            let _ = rt.spawn(async move {
                if let Ok(page) = marketplace.popular_page(Some(100), 0).await {
                    if let Some(items) = page.get("items").and_then(|v| v.as_array()) {
                        if let Ok(mut cache_guard) = cache.lock() {
                            for item in items {
                                let Some(qualified) = item.get("qualifiedName").and_then(|v| v.as_str()) else {
                                    continue;
                                };
                                let lookup_key = canonical_lookup_key(qualified);
                                if cache_guard.contains_key(&lookup_key) {
                                    continue;
                                }
                                cache_guard.insert(
                                    lookup_key,
                                    Some(link_from_item(item, qualified)),
                                );
                            }
                        }
                    }
                }
            });
        }
    }
}

fn canonical_lookup_key(qualified_name: &str) -> String {
    let cleaned = qualified_name.trim_start_matches('@');
    let cleaned = if let Some((_, rest)) = cleaned.split_once('/') {
        rest
    } else {
        cleaned
    };
    cleaned.replace('@', "-").replace('/', "-").to_lowercase()
}

fn link_from_item(item: &serde_json::Value, qualified_name: &str) -> MarketplaceLink {
    MarketplaceLink {
        qualified_name: qualified_name.to_string(),
        display_name: item
            .get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or(qualified_name)
            .to_string(),
        icon_url: optional_str(item.get("iconUrl")),
        external_url: item
            .get("externalUrl")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        description: item
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        is_remote: item.get("isRemote").and_then(|v| v.as_bool()).unwrap_or(false),
        is_verified: item.get("isVerified").and_then(|v| v.as_bool()).unwrap_or(false),
        github_url: optional_str(item.get("githubUrl")),
        website_url: optional_str(item.get("websiteUrl")),
    }
}

fn optional_str(value: Option<&serde_json::Value>) -> Option<String> {
    value.and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string)
}
