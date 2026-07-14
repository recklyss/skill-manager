use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use super::env::is_env_var_reference;
use super::store::McpServerSpec;

pub const REDACTED_MCP_SECRET_VALUE: &str = "[redacted]";

static SECRET_KEY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(authorization|api[-_]?key|token|secret|password)").unwrap());

pub fn is_secret_key(key: &str) -> bool {
    SECRET_KEY_RE.is_match(key)
}

pub fn redact_url(url: Option<&str>) -> Option<String> {
    let url = url?;
    let Some(query_start) = url.find('?') else {
        return Some(url.to_string());
    };
    let (base, query_and_rest) = url.split_at(query_start + 1);
    let fragment = query_and_rest.find('#');
    let (query, suffix) = match fragment {
        Some(idx) => (&query_and_rest[..idx], &query_and_rest[idx..]),
        None => (query_and_rest, ""),
    };
    let redacted = query
        .split('&')
        .map(|pair| {
            let Some((key, value)) = pair.split_once('=') else {
                return pair.to_string();
            };
            if is_secret_key(key) {
                format!("{key}=%5Bredacted%5D")
            } else {
                format!("{key}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join("&");
    Some(format!("{base}{redacted}{suffix}"))
}

fn redact_pairs(pairs: Option<&HashMap<String, String>>) -> Option<HashMap<String, String>> {
    let pairs = pairs?;
    if pairs.is_empty() {
        return None;
    }
    Some(
        pairs
            .iter()
            .map(|(k, v)| {
                let value = if is_env_var_reference(v) || !is_secret_key(k) {
                    v.clone()
                } else {
                    REDACTED_MCP_SECRET_VALUE.to_string()
                };
                (k.clone(), value)
            })
            .collect(),
    )
}

pub fn redact_spec(spec: &McpServerSpec) -> McpServerSpec {
    McpServerSpec {
        name: spec.name.clone(),
        display_name: spec.display_name.clone(),
        source: spec.source.clone(),
        transport: spec.transport.clone(),
        command: spec.command.clone(),
        args: spec.args.clone(),
        env: redact_pairs(spec.env.as_ref()),
        url: redact_url(spec.url.as_deref()),
        headers: redact_pairs(spec.headers.as_ref()),
        installed_at: spec.installed_at.clone(),
        revision: spec.revision.clone(),
    }
}

pub fn redacted_spec_dict(spec: &McpServerSpec) -> serde_json::Value {
    serde_json::to_value(redact_spec(spec)).unwrap_or_default()
}

pub fn annotate_redacted_env(env: Option<&HashMap<String, String>>) -> Vec<serde_json::Value> {
    super::env::annotate_env(redact_pairs(env).as_ref())
}

pub fn redact_payload(value: &serde_json::Value, parent_key: &str) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(
                    k.clone(),
                    if is_secret_key(k) {
                        if let Some(s) = v.as_str() {
                            if is_env_var_reference(s) {
                                v.clone()
                            } else {
                                serde_json::Value::String(REDACTED_MCP_SECRET_VALUE.into())
                            }
                        } else {
                            serde_json::Value::String(REDACTED_MCP_SECRET_VALUE.into())
                        }
                    } else {
                        redact_payload(v, k)
                    },
                );
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(|v| redact_payload(v, parent_key)).collect())
        }
        serde_json::Value::String(s) if parent_key.eq_ignore_ascii_case("url") => {
            serde_json::Value::String(redact_url(Some(s)).unwrap_or_else(|| s.clone()))
        }
        other => other.clone(),
    }
}
