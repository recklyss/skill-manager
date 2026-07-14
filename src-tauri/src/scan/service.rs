use crate::db::scan_config::ScanConfigRepository;
use crate::db::Database;
use crate::error::{ApiError, ApiResult};
use crate::scan::llm::detect_llm;
use crate::skills::queries::SkillsQueryService;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct ScanService {
    db: Arc<Database>,
    skills_queries: SkillsQueryService,
}

impl ScanService {
    pub fn new(db: Arc<Database>, skills_queries: SkillsQueryService) -> Self {
        Self { db, skills_queries }
    }

    pub fn available(&self) -> bool {
        ScanConfigRepository::new(self.db.clone())
            .get_active()
            .ok()
            .flatten()
            .is_some()
            || detect_llm()
                .get("hasAnyAvailable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
    }

    pub fn detect_llm(&self) -> Value {
        detect_llm()
    }

    pub fn scan_skill(&self, skill_ref: &str, options: Option<Value>) -> ApiResult<Value> {
        if !self.available() {
            return Err(ApiError::service_unavailable(
                "Scan service not available. Check LLM configuration.",
            ));
        }

        let skill_path = self
            .skills_queries
            .get_skill_path(skill_ref)
            .ok_or_else(|| ApiError::not_found(format!("unknown skill ref: {skill_ref}")))?;

        let use_llm = options
            .as_ref()
            .and_then(|body| body.get("useLlm"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let skill_name = skill_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(skill_ref)
            .to_string();

        let started = Instant::now();

        if !use_llm {
            return Ok(present_scan_result(
                &skill_name,
                vec![],
                "safe",
                vec![],
                started.elapsed().as_secs_f64(),
            ));
        }

        if !self.has_llm_credentials() {
            return Ok(present_scan_result(
                &skill_name,
                vec![llm_no_api_key_finding()],
                "info",
                vec!["llm_analyzer".into()],
                started.elapsed().as_secs_f64(),
            ));
        }

        let findings = static_skill_scan(&skill_path);
        let max_severity = max_severity(&findings);
        Ok(present_scan_result(
            &skill_name,
            findings,
            max_severity,
            vec!["static_analyzer".into(), "llm_analyzer".into()],
            started.elapsed().as_secs_f64(),
        ))
    }

    fn has_llm_credentials(&self) -> bool {
        if let Ok(Some(active)) = ScanConfigRepository::new(self.db.clone()).get_active() {
            if !active.api_key.trim().is_empty() {
                return true;
            }
            let provider = active.provider.to_lowercase();
            if provider == "ollama" || provider == "bedrock" {
                return true;
            }
        }
        for key in [
            "SKILL_SCANNER_LLM_API_KEY",
            "ANTHROPIC_API_KEY",
            "OPENAI_API_KEY",
            "OPENROUTER_API_KEY",
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY",
            "AZURE_OPENAI_API_KEY",
        ] {
            if std::env::var(key)
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
            {
                return true;
            }
        }
        false
    }
}

fn static_skill_scan(skill_path: &Path) -> Vec<Value> {
    let mut findings = Vec::new();
    let skill_md = skill_path.join("SKILL.md");
    if !skill_md.is_file() {
        findings.push(finding(
            "missing_skill_md",
            "MISSING_SKILL_MD",
            "policy_violation",
            "warning",
            "Missing SKILL.md",
            "Skill package does not contain a SKILL.md file.",
            Some(skill_path.display().to_string()),
            None,
            "static_analyzer",
        ));
        return findings;
    }

    let content = match std::fs::read_to_string(&skill_md) {
        Ok(text) => text,
        Err(error) => {
            findings.push(finding(
                "skill_md_unreadable",
                "SKILL_MD_UNREADABLE",
                "policy_violation",
                "warning",
                "Unable to read SKILL.md",
                &error.to_string(),
                Some(skill_md.display().to_string()),
                None,
                "static_analyzer",
            ));
            return findings;
        }
    };

    let lowered = content.to_lowercase();
    for (id, rule_id, title, needle) in [
        (
            "shell_exec_hint",
            "SHELL_EXEC_HINT",
            "Shell execution mentioned",
            "rm -rf",
        ),
        (
            "credential_hint",
            "CREDENTIAL_HINT",
            "Credential handling mentioned",
            "api_key",
        ),
    ] {
        if lowered.contains(needle) {
            findings.push(finding(
                id,
                rule_id,
                "suspicious_pattern",
                "info",
                title,
                &format!("SKILL.md contains '{needle}'."),
                Some(skill_md.display().to_string()),
                None,
                "static_analyzer",
            ));
        }
    }

    findings
}

fn llm_no_api_key_finding() -> Value {
    finding(
        "llm_no_api_key",
        "LLM_NO_API_KEY",
        "policy_violation",
        "info",
        "LLM scan skipped - no API key",
        "Set ANTHROPIC_API_KEY or OPENAI_API_KEY environment variable",
        None,
        None,
        "llm_analyzer",
    )
}

fn finding(
    id: &str,
    rule_id: &str,
    category: &str,
    severity: &str,
    title: &str,
    description: &str,
    file_path: Option<String>,
    line_number: Option<i64>,
    analyzer: &str,
) -> Value {
    json!({
        "id": id,
        "ruleId": rule_id,
        "category": category,
        "severity": severity,
        "title": title,
        "description": description,
        "filePath": file_path,
        "lineNumber": line_number,
        "snippet": Value::Null,
        "remediation": Value::Null,
        "analyzer": analyzer,
        "metadata": Value::Object(Default::default()),
    })
}

fn max_severity(findings: &[Value]) -> &'static str {
    let mut rank = 0;
    for finding in findings {
        let severity = finding
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("info");
        rank = rank.max(severity_rank(severity));
    }
    match rank {
        4 => "critical",
        3 => "high",
        2 => "medium",
        1 => "warning",
        0 => "safe",
        _ => "info",
    }
}

fn severity_rank(severity: &str) -> i32 {
    match severity {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "warning" => 1,
        "safe" => 0,
        _ => 0,
    }
}

fn present_scan_result(
    skill_name: &str,
    findings: Vec<Value>,
    max_severity: &str,
    analyzers_used: Vec<String>,
    duration_seconds: f64,
) -> Value {
    let findings_count = findings.len() as i64;
    let is_safe = matches!(max_severity, "safe" | "info");
    json!({
        "skillName": skill_name,
        "isSafe": is_safe,
        "maxSeverity": if findings.is_empty() { "safe" } else { max_severity },
        "findingsCount": findings_count,
        "findings": findings,
        "analyzersUsed": analyzers_used,
        "durationSeconds": duration_seconds,
    })
}
