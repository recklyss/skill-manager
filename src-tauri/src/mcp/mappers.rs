use std::collections::HashMap;

use serde_json::{json, Value};

use super::store::{McpServerSpec, McpSource};

pub trait TransportMapper: Send + Sync {
    fn observed_harness(&self) -> &str;
    fn spec_to_dict(&self, spec: &McpServerSpec) -> HashMap<String, Value>;
    fn dict_to_spec(
        &self,
        name: &str,
        raw: &HashMap<String, Value>,
        source: Option<&McpSource>,
    ) -> Result<McpServerSpec, String>;
}

struct TypedMcpServersMapper {
    harness: &'static str,
}

impl TransportMapper for TypedMcpServersMapper {
    fn observed_harness(&self) -> &str {
        self.harness
    }

    fn spec_to_dict(&self, spec: &McpServerSpec) -> HashMap<String, Value> {
        if spec.transport == "stdio" {
            let mut payload = HashMap::from([("type".into(), json!("stdio"))]);
            if let Some(cmd) = &spec.command {
                payload.insert("command".into(), json!(cmd));
            }
            if let Some(args) = &spec.args {
                if !args.is_empty() {
                    payload.insert("args".into(), json!(args));
                }
            }
            if let Some(env) = &spec.env {
                if !env.is_empty() {
                    payload.insert("env".into(), json!(env));
                }
            }
            return payload;
        }
        let mut payload = HashMap::from([("type".into(), json!(spec.transport))]);
        if let Some(url) = &spec.url {
            payload.insert("url".into(), json!(url));
        }
        if let Some(headers) = &spec.headers {
            if !headers.is_empty() {
                payload.insert("headers".into(), json!(headers));
            }
        }
        payload
    }

    fn dict_to_spec(
        &self,
        name: &str,
        raw: &HashMap<String, Value>,
        source: Option<&McpSource>,
    ) -> Result<McpServerSpec, String> {
        let type_value = str_or_none(raw.get("type")).or_else(|| str_or_none(raw.get("transport")));
        if type_value.as_deref() == Some("stdio") || raw.contains_key("command") || raw.contains_key("args") {
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted(self.harness, name)),
                transport: "stdio".into(),
                command: str_or_none(raw.get("command")),
                args: str_vec(raw.get("args")),
                env: str_map(raw.get("env")),
                url: None,
                headers: None,
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        if raw.contains_key("url") {
            let transport = if type_value.as_deref() == Some("sse") {
                "sse"
            } else {
                "http"
            };
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted(self.harness, name)),
                transport: transport.into(),
                command: None,
                args: None,
                env: None,
                url: str_or_none(raw.get("url")),
                headers: str_map(raw.get("headers")),
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        Err(format!(
            "unsupported {} mcp entry '{name}': missing 'command' and 'url'",
            self.harness
        ))
    }
}

struct OpenCodeMapper;

impl TransportMapper for OpenCodeMapper {
    fn observed_harness(&self) -> &str {
        "opencode"
    }

    fn spec_to_dict(&self, spec: &McpServerSpec) -> HashMap<String, Value> {
        if spec.transport == "stdio" {
            let mut command_list = Vec::new();
            if let Some(cmd) = &spec.command {
                command_list.push(cmd.clone());
            }
            if let Some(args) = &spec.args {
                command_list.extend(args.iter().cloned());
            }
            let mut payload = HashMap::from([
                ("type".into(), json!("local")),
                ("command".into(), json!(command_list)),
                ("enabled".into(), json!(true)),
            ]);
            if let Some(env) = &spec.env {
                if !env.is_empty() {
                    payload.insert("environment".into(), json!(env));
                }
            }
            return payload;
        }
        let mut payload = HashMap::from([
            ("type".into(), json!("remote")),
            ("url".into(), json!(spec.url)),
            ("enabled".into(), json!(true)),
        ]);
        if let Some(headers) = &spec.headers {
            if !headers.is_empty() {
                payload.insert("headers".into(), json!(headers));
            }
        }
        payload
    }

    fn dict_to_spec(
        &self,
        name: &str,
        raw: &HashMap<String, Value>,
        source: Option<&McpSource>,
    ) -> Result<McpServerSpec, String> {
        let type_value = str_or_none(raw.get("type"));
        if type_value.as_deref() == Some("local") {
            let (command, args) = match raw.get("command") {
                Some(Value::Array(arr)) if !arr.is_empty() => {
                    let cmd = arr[0].as_str().map(str::to_string);
                    let rest: Vec<String> = arr.iter().skip(1).filter_map(|v| v.as_str().map(str::to_string)).collect();
                    (cmd, if rest.is_empty() { None } else { Some(rest) })
                }
                Some(Value::String(s)) => (Some(s.clone()), None),
                _ => (None, None),
            };
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted("opencode", name)),
                transport: "stdio".into(),
                command,
                args,
                env: str_map(raw.get("environment")),
                url: None,
                headers: None,
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        if type_value.as_deref() == Some("remote") {
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted("opencode", name)),
                transport: "http".into(),
                command: None,
                args: None,
                env: None,
                url: str_or_none(raw.get("url")),
                headers: str_map(raw.get("headers")),
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        Err(format!(
            "unsupported opencode mcp entry '{name}': type must be 'local' or 'remote'"
        ))
    }
}

struct CodexMapper;

impl TransportMapper for CodexMapper {
    fn observed_harness(&self) -> &str {
        "codex"
    }

    fn spec_to_dict(&self, spec: &McpServerSpec) -> HashMap<String, Value> {
        if spec.transport == "stdio" {
            let mut payload = HashMap::new();
            if let Some(cmd) = &spec.command {
                payload.insert("command".into(), json!(cmd));
            }
            if let Some(args) = &spec.args {
                if !args.is_empty() {
                    payload.insert("args".into(), json!(args));
                }
            }
            if let Some(env) = &spec.env {
                if !env.is_empty() {
                    payload.insert("env".into(), json!(env));
                }
            }
            return payload;
        }
        let mut payload = HashMap::new();
        if let Some(url) = &spec.url {
            payload.insert("url".into(), json!(url));
        }
        if let Some(headers) = &spec.headers {
            if !headers.is_empty() {
                payload.insert("http_headers".into(), json!(headers));
            }
        }
        payload
    }

    fn dict_to_spec(
        &self,
        name: &str,
        raw: &HashMap<String, Value>,
        source: Option<&McpSource>,
    ) -> Result<McpServerSpec, String> {
        if raw.contains_key("command") || raw.contains_key("args") {
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted("codex", name)),
                transport: "stdio".into(),
                command: str_or_none(raw.get("command")),
                args: str_vec(raw.get("args")),
                env: str_map(raw.get("env")),
                url: None,
                headers: None,
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        if raw.contains_key("url") {
            let headers = str_map(raw.get("http_headers")).or_else(|| str_map(raw.get("headers")));
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted("codex", name)),
                transport: "http".into(),
                command: None,
                args: None,
                env: None,
                url: str_or_none(raw.get("url")),
                headers,
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        Err(format!(
            "unsupported codex mcp entry '{name}': missing 'command' and 'url'"
        ))
    }
}

struct HermesMapper;

impl TransportMapper for HermesMapper {
    fn observed_harness(&self) -> &str {
        "hermes"
    }

    fn spec_to_dict(&self, spec: &McpServerSpec) -> HashMap<String, Value> {
        if spec.transport == "stdio" {
            let mut payload = HashMap::new();
            if let Some(cmd) = &spec.command {
                payload.insert("command".into(), json!(cmd));
            }
            if let Some(args) = &spec.args {
                if !args.is_empty() {
                    payload.insert("args".into(), json!(args));
                }
            }
            if let Some(env) = &spec.env {
                if !env.is_empty() {
                    payload.insert("env".into(), json!(env));
                }
            }
            return payload;
        }
        let mut payload = HashMap::new();
        if let Some(url) = &spec.url {
            payload.insert("url".into(), json!(url));
        }
        if spec.transport == "sse" {
            payload.insert("transport".into(), json!("sse"));
        }
        if let Some(headers) = &spec.headers {
            if !headers.is_empty() {
                payload.insert("headers".into(), json!(headers));
            }
        }
        payload
    }

    fn dict_to_spec(
        &self,
        name: &str,
        raw: &HashMap<String, Value>,
        source: Option<&McpSource>,
    ) -> Result<McpServerSpec, String> {
        if raw.contains_key("command") || raw.contains_key("args") {
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted("hermes", name)),
                transport: "stdio".into(),
                command: str_or_none(raw.get("command")),
                args: str_vec(raw.get("args")),
                env: str_map(raw.get("env")),
                url: None,
                headers: None,
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        if raw.contains_key("url") {
            let transport = if str_or_none(raw.get("transport")).as_deref() == Some("sse") {
                "sse"
            } else {
                "http"
            };
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted("hermes", name)),
                transport: transport.into(),
                command: None,
                args: None,
                env: None,
                url: str_or_none(raw.get("url")),
                headers: str_map(raw.get("headers")),
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        Err(format!(
            "unsupported hermes mcp entry '{name}': missing 'command' and 'url'"
        ))
    }
}

struct OpenClawMapper;

impl TransportMapper for OpenClawMapper {
    fn observed_harness(&self) -> &str {
        "openclaw"
    }

    fn spec_to_dict(&self, spec: &McpServerSpec) -> HashMap<String, Value> {
        if spec.transport == "stdio" {
            let mut payload = HashMap::new();
            if let Some(cmd) = &spec.command {
                payload.insert("command".into(), json!(cmd));
            }
            if let Some(args) = &spec.args {
                if !args.is_empty() {
                    payload.insert("args".into(), json!(args));
                }
            }
            if let Some(env) = &spec.env {
                if !env.is_empty() {
                    payload.insert("env".into(), json!(env));
                }
            }
            return payload;
        }
        let transport = if spec.transport == "http" {
            "streamable-http"
        } else {
            "sse"
        };
        let mut payload = HashMap::from([
            ("url".into(), json!(spec.url)),
            ("transport".into(), json!(transport)),
        ]);
        if let Some(headers) = &spec.headers {
            if !headers.is_empty() {
                payload.insert("headers".into(), json!(headers));
            }
        }
        payload
    }

    fn dict_to_spec(
        &self,
        name: &str,
        raw: &HashMap<String, Value>,
        source: Option<&McpSource>,
    ) -> Result<McpServerSpec, String> {
        if raw.contains_key("command") || raw.contains_key("args") {
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted("openclaw", name)),
                transport: "stdio".into(),
                command: str_or_none(raw.get("command")),
                args: str_vec(raw.get("args")),
                env: str_map(raw.get("env")),
                url: None,
                headers: None,
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        if raw.contains_key("url") {
            let transport_raw = str_or_none(raw.get("transport")).or_else(|| str_or_none(raw.get("type")));
            let transport = match transport_raw.as_deref() {
                None | Some("http") | Some("streamable-http") => "http",
                _ => "sse",
            };
            return Ok(McpServerSpec {
                name: name.into(),
                display_name: name.into(),
                source: source.cloned().unwrap_or_else(|| McpSource::adopted("openclaw", name)),
                transport: transport.into(),
                command: None,
                args: None,
                env: None,
                url: str_or_none(raw.get("url")),
                headers: str_map(raw.get("headers")),
                installed_at: String::new(),
                revision: String::new(),
            });
        }
        Err(format!(
            "unsupported openclaw mcp entry '{name}': missing 'command' and 'url'"
        ))
    }
}

static CLAUDE_MAPPER: TypedMcpServersMapper = TypedMcpServersMapper { harness: "claude" };
static CURSOR_MAPPER: TypedMcpServersMapper = TypedMcpServersMapper { harness: "cursor" };
static OPENCODE_MAPPER: OpenCodeMapper = OpenCodeMapper;
static CODEX_MAPPER: CodexMapper = CodexMapper;
static HERMES_MAPPER: HermesMapper = HermesMapper;
static OPENCLAW_MAPPER: OpenClawMapper = OpenClawMapper;

pub fn get_mapper(kind: &str) -> &'static dyn TransportMapper {
    match kind {
        "claude-code" => &CLAUDE_MAPPER,
        "cursor" => &CURSOR_MAPPER,
        "opencode" => &OPENCODE_MAPPER,
        "codex" => &CODEX_MAPPER,
        "hermes" => &HERMES_MAPPER,
        "openclaw" => &OPENCLAW_MAPPER,
        other => panic!("unknown mapper kind: {other}"),
    }
}

fn str_or_none(value: Option<&Value>) -> Option<String> {
    value.and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string)
}

fn str_vec(value: Option<&Value>) -> Option<Vec<String>> {
    match value {
        Some(Value::Array(arr)) if !arr.is_empty() => {
            Some(arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        }
        _ => None,
    }
}

fn str_map(value: Option<&Value>) -> Option<HashMap<String, String>> {
    match value {
        Some(Value::Object(map)) if !map.is_empty() => Some(
            map.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect(),
        ),
        _ => None,
    }
}

pub fn value_to_payload_map(value: &Value) -> Option<HashMap<String, Value>> {
    value.as_object().map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}
