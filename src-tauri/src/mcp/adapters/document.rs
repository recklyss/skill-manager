//! Config-document parsing and serialization.
//!
//! Every file-backed MCP adapter reads/writes one config file, but the file
//! format differs per harness (JSON, JSONC, TOML, YAML). This module keeps that
//! concern in one place so the layout adapters only deal with *where* a server
//! lives inside the document.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::{json, Value};

use crate::harness::ConfigFileFormat;

/// Parse a config file into a JSON `Value`, returning `empty` when it is absent.
pub(crate) fn load_document(
    path: &Path,
    format: ConfigFileFormat,
    harness: &str,
    empty: Value,
) -> Result<Value, String> {
    if !path.is_file() {
        return Ok(empty);
    }
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    match format {
        ConfigFileFormat::Json => serde_json::from_str(&text)
            .map_err(|e| format!("{harness} config file is not valid JSON: {e}")),
        ConfigFileFormat::Jsonc => serde_json::from_str(&strip_jsonc(&text))
            .map_err(|e| format!("{harness} config file is not valid JSONC: {e}")),
        ConfigFileFormat::Yaml => serde_yaml::from_str(&text)
            .map_err(|e| format!("{harness} config file is not valid YAML: {e}")),
        ConfigFileFormat::Toml => {
            let parsed: toml::Value = toml::from_str(&text)
                .map_err(|e| format!("{harness} config file is not valid TOML: {e}"))?;
            Ok(toml_to_json(&parsed))
        }
    }
}

/// Serialize a JSON `Value` back into the harness's config format.
pub(crate) fn write_document(
    path: &Path,
    format: ConfigFileFormat,
    document: &Value,
) -> Result<(), String> {
    let contents = match format {
        ConfigFileFormat::Json | ConfigFileFormat::Jsonc => {
            serde_json::to_string_pretty(document).map_err(|e| e.to_string())? + "\n"
        }
        ConfigFileFormat::Yaml => serde_yaml::to_string(document).map_err(|e| e.to_string())?,
        ConfigFileFormat::Toml => {
            let toml_value = json_to_toml(document);
            toml::to_string_pretty(&toml_value).map_err(|e| e.to_string())?
        }
    };
    crate::fsutil::atomic_write(path, contents.as_bytes())
}

/// Strip `//` and `/* */` comments so JSONC can be parsed as JSON.
pub(crate) fn strip_jsonc(text: &str) -> String {
    static BLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/\*.*?\*/").unwrap());
    static LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(^|[^:])//.*$").unwrap());
    static TRAIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r",(\s*[}\]])").unwrap());
    let without_block = BLOCK.replace_all(text, "");
    let without_line = LINE.replace_all(&without_block, "$1");
    TRAIL.replace_all(&without_line, "$1").into_owned()
}

fn toml_to_json(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(s) => json!(s),
        toml::Value::Integer(i) => json!(i),
        toml::Value::Float(f) => json!(f),
        toml::Value::Boolean(b) => json!(b),
        toml::Value::Array(arr) => json!(arr.iter().map(toml_to_json).collect::<Vec<_>>()),
        toml::Value::Table(table) => {
            json!(table.iter().map(|(k, v)| (k.clone(), toml_to_json(v))).collect::<HashMap<_, _>>())
        }
        toml::Value::Datetime(dt) => json!(dt.to_string()),
    }
}

fn json_to_toml(value: &Value) -> toml::Value {
    match value {
        Value::String(s) => toml::Value::String(s.clone()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                toml::Value::Integer(i)
            } else {
                toml::Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::Bool(b) => toml::Value::Boolean(*b),
        Value::Array(arr) => toml::Value::Array(arr.iter().map(json_to_toml).collect()),
        Value::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                table.insert(k.clone(), json_to_toml(v));
            }
            toml::Value::Table(table)
        }
        Value::Null => toml::Value::String(String::new()),
    }
}
