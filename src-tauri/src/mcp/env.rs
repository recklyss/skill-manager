use regex::Regex;
use std::sync::LazyLock;

static ENV_REF_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\$\{env:[A-Z][A-Z0-9_]*\}$").unwrap());

pub fn is_env_var_reference(value: &str) -> bool {
    ENV_REF_PATTERN.is_match(value)
}

pub fn annotate_env(env: Option<&std::collections::HashMap<String, String>>) -> Vec<serde_json::Value> {
    let Some(env) = env else {
        return vec![];
    };
    env.iter()
        .map(|(key, value)| {
            serde_json::json!({
                "key": key,
                "value": value,
                "isEnvRef": is_env_var_reference(value),
            })
        })
        .collect()
}
