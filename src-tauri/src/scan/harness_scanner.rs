use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::harness::HarnessKernelService;

const SCAN_TIMEOUT_SECS: u64 = 120;
const MAX_PROMPT_BYTES: usize = 64 * 1024;

/// Harnesses with a known non-interactive CLI scan invocation.
const SCAN_CAPABLE_HARNESSES: &[(&str, &str)] = &[
    ("claude", "claude"),
    ("codex", "codex"),
    ("copilot", "copilot"),
    ("cursor", "cursor-agent"),
];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HarnessScanPayload {
    verdict: String,
    risk_level: String,
    summary: String,
    findings: Vec<HarnessScanFinding>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HarnessScanFinding {
    id: String,
    severity: String,
    title: String,
    detail: String,
    #[serde(default)]
    snippet: Option<String>,
    #[serde(default)]
    remediation: Option<String>,
}

pub struct ScannableHarness {
    pub harness: String,
    pub label: String,
    pub cli_available: bool,
}

pub fn list_scannable_harnesses(kernel: &HarnessKernelService) -> Vec<ScannableHarness> {
    let enabled: std::collections::HashSet<String> = kernel
        .enabled_harness_ids()
        .into_iter()
        .collect();

    SCAN_CAPABLE_HARNESSES
        .iter()
        .filter_map(|(harness_id, binary)| {
            if !enabled.contains(*harness_id) {
                return None;
            }
            let definition = kernel.definition(harness_id)?;
            let cli_available = which::which(binary).is_ok();
            Some(ScannableHarness {
                harness: harness_id.to_string(),
                label: definition.label.to_string(),
                cli_available,
            })
        })
        .collect()
}

pub fn harness_supports_scan(harness: &str) -> bool {
    SCAN_CAPABLE_HARNESSES
        .iter()
        .any(|(id, _)| *id == harness)
}

pub fn run_harness_scan(
    harness: &str,
    skill_path: &Path,
    skill_name: &str,
) -> ApiResult<Vec<Value>> {
    if !harness_supports_scan(harness) {
        return Err(ApiError::bad_request(format!(
            "harness '{harness}' does not support security scanning"
        )));
    }

    let binary = SCAN_CAPABLE_HARNESSES
        .iter()
        .find(|(id, _)| *id == harness)
        .map(|(_, binary)| *binary)
        .expect("checked above");

    if which::which(binary).is_err() {
        return Err(ApiError::service_unavailable(format!(
            "{binary} CLI is not installed or not on PATH"
        )));
    }

    let skill_content = collect_skill_content(skill_path)?;
    let prompt = build_scan_prompt(skill_name, &skill_content);
    let output = invoke_harness_cli(harness, binary, &prompt)?;
    let payload = parse_harness_scan_output(&output)?;
    Ok(map_harness_findings(&payload, harness))
}

fn collect_skill_content(skill_path: &Path) -> ApiResult<String> {
    let mut parts = Vec::new();
    let mut total = 0usize;

    let skill_md = skill_path.join("SKILL.md");
    if skill_md.is_file() {
        let text = std::fs::read_to_string(&skill_md).map_err(|error| {
            ApiError::internal(format!("failed to read SKILL.md: {error}"))
        })?;
        total += text.len();
        parts.push(format!("=== SKILL.md ===\n{text}"));
    }

    if let Ok(entries) = std::fs::read_dir(skill_path) {
        let mut files: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| path.file_name().and_then(|n| n.to_str()) != Some("SKILL.md"))
            .collect();
        files.sort();

        for path in files {
            if total >= MAX_PROMPT_BYTES {
                parts.push("\n[additional files truncated]".to_string());
                break;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !is_scan_candidate_file(name) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let budget = MAX_PROMPT_BYTES.saturating_sub(total);
            let chunk = truncate_bytes(&text, budget);
            total += chunk.len();
            parts.push(format!("=== {name} ===\n{chunk}"));
        }
    }

    if parts.is_empty() {
        return Err(ApiError::bad_request("skill has no readable content to scan"));
    }

    Ok(parts.join("\n\n"))
}

fn is_scan_candidate_file(name: &str) -> bool {
    matches!(
        name.rsplit('.').next(),
        Some("md" | "txt" | "sh" | "py" | "js" | "ts" | "json" | "yaml" | "yml" | "toml")
    )
}

fn truncate_bytes(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n[truncated]", &text[..end])
}

fn build_scan_prompt(skill_name: &str, skill_content: &str) -> String {
    format!(
        r#"You are a security auditor for AI agent skills. Analyze the skill "{skill_name}" for security risks: data exfiltration, credential theft, shell injection, prompt injection, malicious tool use, and policy violations.

Respond with ONLY valid JSON (no markdown fences, no commentary) matching this schema:
{{
  "verdict": "pass" | "warn" | "fail",
  "riskLevel": "low" | "medium" | "high" | "critical",
  "summary": "one line summary",
  "findings": [
    {{
      "id": "unique-id",
      "severity": "low" | "medium" | "high" | "critical",
      "title": "short title",
      "detail": "explanation",
      "snippet": "optional quoted excerpt or null",
      "remediation": "optional fix guidance or null"
    }}
  ]
}}

Skill content:
{skill_content}
"#
    )
}

fn invoke_harness_cli(harness: &str, binary: &str, prompt: &str) -> ApiResult<String> {
    let mut command = Command::new(binary);
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

    match harness {
        "claude" => {
            command
                .arg("-p")
                .arg(prompt)
                .arg("--output-format")
                .arg("text");
        }
        "codex" => {
            command.arg("exec").arg("-").stdin(Stdio::piped());
        }
        "copilot" => {
            command.arg("-p").arg(prompt).arg("--allow-all");
        }
        "cursor" => {
            command
                .arg("-p")
                .arg(prompt)
                .arg("--output-format")
                .arg("text")
                .arg("-f");
        }
        _ => {
            return Err(ApiError::bad_request(format!(
                "no CLI invocation defined for harness '{harness}'"
            )));
        }
    }

    let mut child = command.spawn().map_err(|error| {
        ApiError::service_unavailable(format!("failed to start {binary}: {error}"))
    })?;

    if harness == "codex" {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(prompt.as_bytes());
        }
    }

    let output = wait_with_timeout(child, Duration::from_secs(SCAN_TIMEOUT_SECS))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::service_unavailable(format!(
            "{binary} scan failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn wait_with_timeout(
    child: std::process::Child,
    timeout: Duration,
) -> ApiResult<std::process::Output> {
    let handle = std::thread::spawn(move || child.wait_with_output());
    let started = std::time::Instant::now();
    loop {
        if handle.is_finished() {
            return handle
                .join()
                .map_err(|_| ApiError::internal("scan thread panicked"))?
                .map_err(|error| ApiError::internal(format!("scan process error: {error}")));
        }
        if started.elapsed() >= timeout {
            return Err(ApiError::service_unavailable(
                "harness scan timed out after 120 seconds",
            ));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

pub fn parse_harness_scan_output(stdout: &str) -> ApiResult<HarnessScanPayload> {
    let json_text = extract_json_text(stdout)?;
    let payload: HarnessScanPayload = serde_json::from_str(&json_text).map_err(|error| {
        ApiError::service_unavailable(format!("harness returned invalid scan JSON: {error}"))
    })?;
    validate_payload(&payload)?;
    Ok(payload)
}

fn extract_json_text(stdout: &str) -> ApiResult<String> {
    let trimmed = stdout.trim();
    if let Some(fenced) = strip_markdown_fence(trimmed) {
        return Ok(fenced);
    }
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if let Some(nested) = value.get("result").and_then(|v| v.as_str()) {
            if let Some(fenced) = strip_markdown_fence(nested.trim()) {
                return Ok(fenced);
            }
            if serde_json::from_str::<Value>(nested).is_ok() {
                return Ok(nested.to_string());
            }
        }
        return Ok(trimmed.to_string());
    }
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            let candidate = &trimmed[start..=end];
            if serde_json::from_str::<Value>(candidate).is_ok() {
                return Ok(candidate.to_string());
            }
        }
    }
    Err(ApiError::service_unavailable(
        "harness scan output did not contain parseable JSON",
    ))
}

fn strip_markdown_fence(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return None;
    }
    let body = trimmed
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if serde_json::from_str::<Value>(body).is_ok() {
        Some(body.to_string())
    } else {
        None
    }
}

fn validate_payload(payload: &HarnessScanPayload) -> ApiResult<()> {
    let verdict = payload.verdict.to_lowercase();
    if !matches!(verdict.as_str(), "pass" | "warn" | "fail") {
        return Err(ApiError::service_unavailable(format!(
            "invalid verdict: {}",
            payload.verdict
        )));
    }
    let risk = payload.risk_level.to_lowercase();
    if !matches!(
        risk.as_str(),
        "low" | "medium" | "high" | "critical"
    ) {
        return Err(ApiError::service_unavailable(format!(
            "invalid riskLevel: {}",
            payload.risk_level
        )));
    }
    Ok(())
}

fn map_harness_findings(payload: &HarnessScanPayload, harness: &str) -> Vec<Value> {
    let analyzer = format!("{harness}_scanner");
    payload
        .findings
        .iter()
        .map(|finding| {
            json!({
                "id": finding.id,
                "ruleId": finding.id.to_uppercase(),
                "category": "security_risk",
                "severity": normalize_severity(&finding.severity),
                "title": finding.title,
                "description": finding.detail,
                "filePath": Value::Null,
                "lineNumber": Value::Null,
                "snippet": finding.snippet.clone().unwrap_or_default(),
                "remediation": finding.remediation.clone().unwrap_or_default(),
                "analyzer": analyzer,
                "metadata": {
                    "verdict": payload.verdict,
                    "riskLevel": payload.risk_level,
                    "summary": payload.summary,
                },
            })
        })
        .collect()
}

fn normalize_severity(severity: &str) -> String {
    match severity.to_lowercase().as_str() {
        "critical" => "CRITICAL".to_string(),
        "high" => "HIGH".to_string(),
        "medium" => "MEDIUM".to_string(),
        "low" => "LOW".to_string(),
        "warning" | "warn" => "LOW".to_string(),
        "info" => "LOW".to_string(),
        other => other.to_uppercase(),
    }
}

pub fn scannable_harnesses_json(kernel: &HarnessKernelService) -> Value {
    let harnesses: Vec<Value> = list_scannable_harnesses(kernel)
        .into_iter()
        .map(|entry| {
            json!({
                "harness": entry.harness,
                "label": entry.label,
                "cliAvailable": entry.cli_available,
                "scannable": entry.cli_available,
            })
        })
        .collect();
    json!({ "harnesses": harnesses })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OUTPUT: &str = r#"```json
{
  "verdict": "warn",
  "riskLevel": "medium",
  "summary": "Skill mentions shell commands",
  "findings": [
    {
      "id": "shell-exec",
      "severity": "medium",
      "title": "Shell execution",
      "detail": "SKILL.md references rm -rf",
      "snippet": "rm -rf",
      "remediation": "Remove destructive examples"
    }
  ]
}
```"#;

    #[test]
    fn parse_sample_cli_output() {
        let payload = parse_harness_scan_output(SAMPLE_OUTPUT).expect("parse");
        assert_eq!(payload.verdict, "warn");
        assert_eq!(payload.risk_level, "medium");
        assert_eq!(payload.findings.len(), 1);
        assert_eq!(payload.findings[0].title, "Shell execution");
    }

    #[test]
    fn parse_embedded_json_object() {
        let stdout = r#"Here is the result: {"verdict":"pass","riskLevel":"low","summary":"ok","findings":[]} done"#;
        let payload = parse_harness_scan_output(stdout).expect("parse");
        assert_eq!(payload.verdict, "pass");
        assert!(payload.findings.is_empty());
    }

    #[test]
    fn map_findings_to_scan_envelope_fields() {
        let payload = parse_harness_scan_output(SAMPLE_OUTPUT).expect("parse");
        let findings = map_harness_findings(&payload, "claude");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0]["severity"], "MEDIUM");
        assert_eq!(findings[0]["analyzer"], "claude_scanner");
    }

    #[test]
    fn harness_supports_known_ids() {
        assert!(harness_supports_scan("claude"));
        assert!(harness_supports_scan("codex"));
        assert!(!harness_supports_scan("hermes"));
    }
}
