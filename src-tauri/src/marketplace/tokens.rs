use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use serde_json::Value;

pub fn encode_install_token(source_kind: &str, source_locator: &str) -> String {
    let payload = serde_json::json!([source_kind, source_locator]);
    let compact = serde_json::to_string(&payload).expect("install token payload");
    URL_SAFE.encode(compact.as_bytes()).trim_end_matches('=').to_string()
}

pub fn resolve_install_token(token: &str) -> Option<(String, String)> {
    let padding = "=".repeat((4 - token.len() % 4) % 4);
    let decoded = URL_SAFE
        .decode(format!("{token}{padding}"))
        .ok()?;
    let payload: Value = serde_json::from_slice(&decoded).ok()?;
    let items = payload.as_array()?;
    if items.len() != 2 {
        return None;
    }
    let source_kind = items[0].as_str()?.to_string();
    let source_locator = items[1].as_str()?.to_string();
    Some((source_kind, source_locator))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_matches_python_compact_json_token() {
        let locator = "github:mode-io/skills/mode-switch";
        let token = encode_install_token("github", locator);
        let (kind, resolved) = resolve_install_token(&token).expect("resolve");
        assert_eq!(kind, "github");
        assert_eq!(resolved, locator);
    }
}
