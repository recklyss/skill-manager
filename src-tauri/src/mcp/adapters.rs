use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use regex::Regex;
use serde_json::{json, Value};

use crate::harness::{BindingProfile, ConfigFileFormat, ConfigSubtreeBindingProfile, FamilyKey, HarnessDefinition, ResolutionContext};

use super::contracts::{McpHarnessScan, McpObservedEntry};
use super::mappers::{get_mapper, value_to_payload_map, TransportMapper};
use super::store::{McpServerSpec, McpSource};

#[derive(Clone)]
pub struct FileBackedMcpAdapter {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub config_path: PathBuf,
    definition: &'static HarnessDefinition,
    profile: ConfigSubtreeBindingProfile,
    context: ResolutionContext,
    mapper: &'static dyn TransportMapper,
}

impl FileBackedMcpAdapter {
    pub fn new(
        definition: &'static HarnessDefinition,
        profile: ConfigSubtreeBindingProfile,
        context: ResolutionContext,
    ) -> Self {
        let mapper = get_mapper(profile.codec);
        Self {
            harness: definition.harness.to_string(),
            label: definition.label.to_string(),
            logo_key: definition.logo_key.map(str::to_string),
            config_path: profile.resolve_config_path(&context),
            definition,
            profile,
            context,
            mapper,
        }
    }

    pub fn status(&self) -> (bool, bool, bool, Option<String>) {
        let installed = which::which(self.definition.install_probe).is_ok();
        let config_present = self.config_path.is_file();
        let (mcp_writable, unavailable_reason) = self.mcp_write_capability(installed);
        (installed, config_present, mcp_writable, unavailable_reason)
    }

    pub fn scan(&self, specs: &[McpServerSpec]) -> McpHarnessScan {
        let (installed, config_present, mcp_writable, mcp_unavailable_reason) = self.status();
        let specs_by_name: HashMap<_, _> = specs.iter().map(|s| (s.name.clone(), s)).collect();
        let mut entries = Vec::new();
        let mut seen_names = std::collections::HashSet::new();
        let mut scan_issue = None;

        let raw_entries = if config_present {
            match self.read_entries() {
                Ok(items) => items,
                Err(reason) => {
                    scan_issue = Some(reason);
                    vec![]
                }
            }
        } else {
            vec![]
        };

        for (name, payload) in raw_entries {
            seen_names.insert(name.clone());
            let mut parsed_spec = None;
            let mut parse_issue = None;
            match self.mapper.dict_to_spec(
                &name,
                &payload,
                Some(&McpSource::adopted(&self.harness, &name)),
            ) {
                Ok(spec) => parsed_spec = Some(spec),
                Err(reason) => parse_issue = Some(reason),
            }

            let managed_spec = specs_by_name.get(&name);
            if managed_spec.is_none() {
                entries.push(McpObservedEntry {
                    name,
                    state: "unmanaged".into(),
                    raw_payload: Some(json!(payload)),
                    parsed_spec,
                    drift_detail: None,
                    parse_issue,
                });
                continue;
            }

            if let Some(reason) = parse_issue {
                entries.push(McpObservedEntry {
                    name,
                    state: "drifted".into(),
                    raw_payload: Some(json!(payload)),
                    parsed_spec,
                    drift_detail: Some(reason.clone()),
                    parse_issue: Some(reason),
                });
                continue;
            }

            let managed = managed_spec.unwrap();
            let expected = normalize_payload(&self.mapper.spec_to_dict(managed));
            let actual = normalize_payload(&payload);
            if expected == actual {
                entries.push(McpObservedEntry {
                    name,
                    state: "managed".into(),
                    raw_payload: Some(json!(payload)),
                    parsed_spec,
                    drift_detail: None,
                    parse_issue: None,
                });
            } else {
                entries.push(McpObservedEntry {
                    name,
                    state: "drifted".into(),
                    raw_payload: Some(json!(payload)),
                    parsed_spec,
                    drift_detail: Some(drift_detail(&expected, &actual)),
                    parse_issue: None,
                });
            }
        }

        for spec in specs {
            if !seen_names.contains(&spec.name) {
                entries.push(McpObservedEntry {
                    name: spec.name.clone(),
                    state: "missing".into(),
                    raw_payload: None,
                    parsed_spec: Some(spec.clone()),
                    drift_detail: None,
                    parse_issue: None,
                });
            }
        }

        McpHarnessScan {
            harness: self.harness.clone(),
            label: self.label.clone(),
            logo_key: self.logo_key.clone(),
            installed,
            config_present,
            config_path: self.config_path.clone(),
            mcp_writable,
            mcp_unavailable_reason,
            scan_issue,
            entries,
        }
    }

    pub fn has_binding(&self, name: &str) -> bool {
        self.read_entries()
            .map(|entries| entries.iter().any(|(n, _)| n == name))
            .unwrap_or(false)
    }

    pub fn enable_server(&self, spec: &McpServerSpec) -> Result<(), String> {
        self.require_mcp_writable()?;
        let mut document = self.load_document(&self.config_path)?;
        let payload = self.mapper.spec_to_dict(spec);
        set_subtree_entry(
            &mut document,
            self.profile.subtree_path,
            &spec.name,
            payload,
            self.profile.file_format,
        )?;
        self.write_document(&self.config_path, &document)
    }

    pub fn disable_server(&self, name: &str) -> Result<(), String> {
        if !self.config_path.is_file() {
            return Ok(());
        }
        let mut document = self.load_document(&self.config_path)?;
        if remove_subtree_entry(&mut document, self.profile.subtree_path, name) {
            self.write_document(&self.config_path, &document)?;
        }
        Ok(())
    }

    fn require_mcp_writable(&self) -> Result<(), String> {
        let (_, _, writable, reason) = self.status();
        if writable {
            return Ok(());
        }
        Err(reason.unwrap_or_else(|| format!("{} MCP config is not writable", self.label)))
    }

    fn mcp_write_capability(&self, installed: bool) -> (bool, Option<String>) {
        let Some(probe) = self.profile.capability_probe else {
            return (true, None);
        };
        let reason = self
            .profile
            .capability_unavailable_reason
            .map(str::to_string)
            .unwrap_or_else(|| format!("{} MCP support is unavailable", self.label));
        if probe == "openclaw-mcp-command" {
            let executable = which::which(self.definition.install_probe).ok();
            let Some(executable) = executable else {
                return (false, Some(reason));
            };
            let output = Command::new(executable)
                .args(["mcp", "--help"])
                .output();
            match output {
                Ok(result) if result.status.success() => (true, None),
                _ => (false, Some(reason)),
            }
        } else {
            (installed, if installed { None } else { Some(reason) })
        }
    }

    fn read_entries(&self) -> Result<Vec<(String, HashMap<String, Value>)>, String> {
        if !self.config_path.is_file() {
            return Ok(vec![]);
        }
        let document = self.load_document(&self.config_path)?;
        let subtree = read_subtree(&document, self.profile.subtree_path)?;
        Ok(subtree
            .into_iter()
            .filter_map(|(name, value)| {
                value_to_payload_map(&value).map(|payload| (name, payload))
            })
            .collect())
    }

    fn load_document(&self, path: &Path) -> Result<Value, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        match self.profile.file_format {
            ConfigFileFormat::Json => serde_json::from_str(&text)
                .map_err(|e| format!("{} config file is not valid JSON: {e}", self.harness)),
            ConfigFileFormat::Jsonc => serde_json::from_str(&strip_jsonc(&text))
                .map_err(|e| format!("{} config file is not valid JSONC: {e}", self.harness)),
            ConfigFileFormat::Yaml => serde_yaml::from_str(&text)
                .map_err(|e| format!("{} config file is not valid YAML: {e}", self.harness)),
            ConfigFileFormat::Toml => {
                let parsed: toml::Value = toml::from_str(&text)
                    .map_err(|e| format!("{} config file is not valid TOML: {e}", self.harness))?;
                Ok(toml_to_json(&parsed))
            }
        }
    }

    fn write_document(&self, path: &Path, document: &Value) -> Result<(), String> {
        let contents = match self.profile.file_format {
            ConfigFileFormat::Json | ConfigFileFormat::Jsonc => {
                serde_json::to_string_pretty(document).map_err(|e| e.to_string())? + "\n"
            }
            ConfigFileFormat::Yaml => serde_yaml::to_string(document).map_err(|e| e.to_string())?,
            ConfigFileFormat::Toml => {
                let toml_value = json_to_toml(document);
                toml::to_string_pretty(&toml_value).map_err(|e| e.to_string())?
            }
        };
        atomic_write(path, &contents)
    }
}

pub fn build_mcp_adapters(
    definitions: &'static [HarnessDefinition],
    context: &ResolutionContext,
) -> Vec<FileBackedMcpAdapter> {
    definitions
        .iter()
        .filter_map(|definition| {
            let BindingProfile::ConfigSubtree(profile) = definition.binding_for(FamilyKey::Mcp)? else {
                return None;
            };
            Some(FileBackedMcpAdapter::new(definition, profile.clone(), context.clone()))
        })
        .collect()
}

fn read_subtree(document: &Value, subtree_path: &[&str]) -> Result<HashMap<String, Value>, String> {
    let mut cursor = document;
    for key in subtree_path {
        cursor = cursor.get(*key).ok_or_else(|| format!("missing subtree '{key}'"))?;
    }
    let obj = cursor
        .as_object()
        .ok_or_else(|| "subtree must be an object".to_string())?;
    Ok(obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

fn set_subtree_entry(
    document: &mut Value,
    subtree_path: &[&str],
    name: &str,
    payload: HashMap<String, Value>,
    format: ConfigFileFormat,
) -> Result<(), String> {
    ensure_subtree(document, subtree_path, format);
    let mut cursor = document;
    for key in subtree_path {
        cursor = cursor.get_mut(*key).unwrap();
    }
    let obj = cursor.as_object_mut().unwrap();
    obj.insert(name.to_string(), json!(payload));
    Ok(())
}

fn remove_subtree_entry(document: &mut Value, subtree_path: &[&str], name: &str) -> bool {
    let mut cursor = &mut *document;
    for key in subtree_path {
        let Some(next) = cursor.get_mut(*key) else {
            return false;
        };
        cursor = next;
    }
    let Some(obj) = cursor.as_object_mut() else {
        return false;
    };
    let removed = obj.remove(name).is_some();
    if obj.is_empty() && subtree_path.len() > 1 {
        let mut parent = &mut *document;
        for key in &subtree_path[..subtree_path.len() - 1] {
            parent = parent.get_mut(*key).unwrap();
        }
        parent
            .as_object_mut()
            .unwrap()
            .remove(*subtree_path.last().unwrap());
    }
    removed
}

fn ensure_subtree(document: &mut Value, subtree_path: &[&str], format: ConfigFileFormat) {
    let mut cursor = document;
    if !cursor.is_object() {
        *cursor = json!({});
    }
    for key in subtree_path {
        if !cursor.get(key).map(|v| v.is_object()).unwrap_or(false) {
            cursor.as_object_mut().unwrap().insert(key.to_string(), json!({}));
        }
        cursor = cursor.get_mut(*key).unwrap();
    }
    let _ = format;
}

fn normalize_payload(value: &HashMap<String, Value>) -> HashMap<String, Value> {
    let mut normalized = HashMap::new();
    for (key, item) in value {
        if is_semantic_default(key, item) {
            continue;
        }
        normalized.insert(key.clone(), normalize_value(item));
    }
    let mut keys: Vec<_> = normalized.keys().cloned().collect();
    keys.sort();
    keys.into_iter()
        .map(|k| (k.clone(), normalized[&k].clone()))
        .collect()
}

fn normalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if !is_semantic_default(k, v) {
                    out.insert(k.clone(), normalize_value(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(normalize_value).collect()),
        other => other.clone(),
    }
}

fn is_semantic_default(key: &str, value: &Value) -> bool {
    (key == "enabled" && value == &json!(true))
        || (key == "transport" && value == &json!("stdio"))
        || (matches!(key, "headers" | "env" | "environment" | "http_headers")
            && value.as_object().map(|m| m.is_empty()).unwrap_or(false))
}

fn drift_detail(expected: &HashMap<String, Value>, actual: &HashMap<String, Value>) -> String {
    let expected_keys: std::collections::HashSet<_> = expected.keys().collect();
    let actual_keys: std::collections::HashSet<_> = actual.keys().collect();
    let missing: Vec<_> = expected_keys
        .difference(&actual_keys)
        .map(|k| (*k).clone())
        .collect();
    let extra: Vec<_> = actual_keys
        .difference(&expected_keys)
        .map(|k| (*k).clone())
        .collect();
    let changed: Vec<_> = expected_keys
        .intersection(&actual_keys)
        .filter(|k| expected[k.as_str()] != actual[k.as_str()])
        .map(|k| (*k).clone())
        .collect();
    let mut parts = Vec::new();
    if !missing.is_empty() {
        parts.push(format!("missing={}", missing.join(",")));
    }
    if !extra.is_empty() {
        parts.push(format!("extra={}", extra.join(",")));
    }
    if !changed.is_empty() {
        parts.push(format!("changed={}", changed.join(",")));
    }
    if parts.is_empty() {
        "value mismatch".into()
    } else {
        parts.join("; ")
    }
}

fn strip_jsonc(text: &str) -> String {
    static BLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/\*.*?\*/").unwrap());
    static LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(^|[^:])//.*$").unwrap());
    static TRAIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r",(\s*[}\]])").unwrap());
    let without_block = BLOCK.replace_all(text, "");
    let without_line = LINE.replace_all(&without_block, "$1");
    TRAIL.replace_all(&without_line, "$1").into_owned()
}

use std::sync::LazyLock;

fn atomic_write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, contents).map_err(|e| e.to_string())?;
    fs::rename(&temp, path).map_err(|e| e.to_string())?;
    Ok(())
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

#[derive(Clone)]
pub struct McpReadModelService {
    store: super::store::McpServerStore,
    adapters: Vec<FileBackedMcpAdapter>,
    kernel: crate::harness::HarnessKernelService,
    cache: std::sync::Arc<Mutex<Option<CachedSnapshot>>>,
    snapshot_ttl: Duration,
}

#[derive(Clone)]
struct CachedSnapshot {
    snapshot: super::contracts::McpReadModelSnapshot,
    captured_at: Instant,
}

impl McpReadModelService {
    pub fn new(
        store: super::store::McpServerStore,
        kernel: crate::harness::HarnessKernelService,
    ) -> Self {
        let adapters = build_mcp_adapters(
            crate::harness::SUPPORTED_HARNESS_DEFINITIONS,
            &kernel.context,
        );
        Self {
            store,
            adapters,
            kernel,
            cache: std::sync::Arc::new(Mutex::new(None)),
            snapshot_ttl: Duration::from_secs_f64(1.0),
        }
    }

    pub fn store(&self) -> &super::store::McpServerStore {
        &self.store
    }

    pub fn find_adapter(&self, harness: &str) -> Option<&FileBackedMcpAdapter> {
        self.adapters.iter().find(|a| a.harness == harness)
    }

    pub fn enabled_harnesses(&self) -> Vec<String> {
        self.kernel
            .enabled_harness_ids_for_family(FamilyKey::Mcp)
    }

    pub fn enabled_adapters(&self) -> Vec<&FileBackedMcpAdapter> {
        let enabled: std::collections::HashSet<_> = self.enabled_harnesses().into_iter().collect();
        self.adapters
            .iter()
            .filter(|a| enabled.contains(&a.harness))
            .collect()
    }

    pub fn enabled_addressable_adapters(&self) -> Vec<&FileBackedMcpAdapter> {
        self.enabled_adapters()
            .into_iter()
            .filter(|adapter| {
                let (installed, config_present, _, _) = adapter.status();
                installed || config_present
            })
            .collect()
    }

    pub fn enabled_writable_adapters(&self) -> Vec<&FileBackedMcpAdapter> {
        self.enabled_adapters()
            .into_iter()
            .filter(|adapter| {
                let (installed, config_present, writable, _) = adapter.status();
                writable && (installed || config_present)
            })
            .collect()
    }

    pub fn require_enabled_adapter(&self, harness: &str) -> Result<&FileBackedMcpAdapter, String> {
        let adapter = self
            .find_adapter(harness)
            .ok_or_else(|| format!("unknown harness: {harness}"))?;
        if !self.enabled_harnesses().iter().any(|h| h == harness) {
            return Err(format!("harness support is disabled: {harness}"));
        }
        let (installed, config_present, _, _) = adapter.status();
        if !installed && !config_present {
            return Err(format!(
                "{} is not installed and has no MCP config file",
                adapter.label
            ));
        }
        Ok(adapter)
    }

    pub fn snapshot(&self) -> super::contracts::McpReadModelSnapshot {
        if let Ok(guard) = self.cache.lock() {
            if let Some(cached) = guard.as_ref() {
                if cached.captured_at.elapsed() < self.snapshot_ttl {
                    return cached.snapshot.clone();
                }
            }
        }
        let specs = self.store.list_records();
        let scans = self
            .adapters
            .iter()
            .map(|adapter| adapter.scan(&specs))
            .collect();
        let snapshot = super::contracts::McpReadModelSnapshot { harness_scans: scans };
        if let Ok(mut guard) = self.cache.lock() {
            *guard = Some(CachedSnapshot {
                snapshot: snapshot.clone(),
                captured_at: Instant::now(),
            });
        }
        snapshot
    }

    pub fn visible_scans(&self, snapshot: &super::contracts::McpReadModelSnapshot) -> Vec<McpHarnessScan> {
        let visible: std::collections::HashSet<_> = self.enabled_harnesses().into_iter().collect();
        snapshot
            .harness_scans
            .iter()
            .filter(|scan| visible.contains(&scan.harness))
            .cloned()
            .collect()
    }

    pub fn invalidate(&self) {
        if let Ok(mut guard) = self.cache.lock() {
            *guard = None;
        }
    }
}
