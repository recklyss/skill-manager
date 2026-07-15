use crate::error::{ApiError, ApiResult};
use crate::harness::HarnessKernelService;
use crate::scan::harness_scanner::{
    harness_supports_scan, list_scannable_harnesses, run_harness_scan, scannable_harnesses_json,
};
use crate::skills::queries::SkillsQueryService;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Instant;

#[derive(Clone)]
pub struct ScanService {
    harness_kernel: HarnessKernelService,
    skills_queries: SkillsQueryService,
}

impl ScanService {
    pub fn new(harness_kernel: HarnessKernelService, skills_queries: SkillsQueryService) -> Self {
        Self {
            harness_kernel,
            skills_queries,
        }
    }

    pub fn available(&self) -> bool {
        list_scannable_harnesses(&self.harness_kernel)
            .into_iter()
            .any(|entry| entry.cli_available)
    }

    pub fn scannable_harnesses(&self) -> Value {
        scannable_harnesses_json(&self.harness_kernel)
    }

    pub fn scan_skill(&self, skill_ref: &str, options: Option<Value>) -> ApiResult<Value> {
        let skill_path = self
            .skills_queries
            .get_skill_path(skill_ref)
            .ok_or_else(|| ApiError::not_found(format!("unknown skill ref: {skill_ref}")))?;

        let harness = options
            .as_ref()
            .and_then(|body| body.get("harness"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let static_only = options
            .as_ref()
            .and_then(|body| body.get("useLlm"))
            .and_then(|v| v.as_bool())
            .map(|use_llm| !use_llm)
            .unwrap_or(false);

        let skill_name = skill_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(skill_ref)
            .to_string();

        let started = Instant::now();

        if static_only {
            let findings = static_skill_scan(&skill_path);
            let max_severity = max_severity(&findings);
            return Ok(present_scan_result(
                &skill_name,
                findings,
                max_severity,
                vec!["static_analyzer".into()],
                started.elapsed().as_secs_f64(),
            ));
        }

        let Some(harness) = harness else {
            return Err(ApiError::bad_request(
                "harness is required for security scan (select an enabled agent CLI)",
            ));
        };

        if !self
            .harness_kernel
            .enabled_harness_ids()
            .iter()
            .any(|enabled| enabled == harness)
        {
            return Err(ApiError::bad_request(format!(
                "harness '{harness}' is not enabled"
            )));
        }

        if !harness_supports_scan(harness) {
            return Err(ApiError::bad_request(format!(
                "harness '{harness}' does not support security scanning"
            )));
        }

        let mut findings = static_skill_scan(&skill_path);
        let mut analyzers = vec!["static_analyzer".into()];

        match run_harness_scan(harness, &skill_path, &skill_name) {
            Ok(harness_findings) => {
                analyzers.push(format!("{harness}_scanner"));
                findings.extend(harness_findings);
            }
            Err(error) => {
                findings.push(harness_scan_error_finding(harness, &error.message));
                analyzers.push(format!("{harness}_scanner"));
            }
        }

        let max_severity = max_severity(&findings);
        Ok(present_scan_result(
            &skill_name,
            findings,
            max_severity,
            analyzers,
            started.elapsed().as_secs_f64(),
        ))
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
            "LOW",
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
                "LOW",
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
                "LOW",
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

fn harness_scan_error_finding(harness: &str, detail: &str) -> Value {
    finding(
        &format!("{harness}_scan_error"),
        "HARNESS_SCAN_ERROR",
        "policy_violation",
        "LOW",
        &format!("{harness} scan failed"),
        detail,
        None,
        None,
        &format!("{harness}_scanner"),
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
            .unwrap_or("LOW");
        rank = rank.max(severity_rank(severity));
    }
    match rank {
        4 => "CRITICAL",
        3 => "HIGH",
        2 => "MEDIUM",
        1 => "LOW",
        _ => "SAFE",
    }
}

fn severity_rank(severity: &str) -> i32 {
    match severity.to_uppercase().as_str() {
        "CRITICAL" => 4,
        "HIGH" => 3,
        "MEDIUM" => 2,
        "LOW" | "WARNING" => 1,
        "SAFE" => 0,
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
    let is_safe = !matches!(max_severity, "CRITICAL" | "HIGH");
    json!({
        "skillName": skill_name,
        "isSafe": is_safe,
        "maxSeverity": if findings.is_empty() { "SAFE" } else { max_severity },
        "findingsCount": findings_count,
        "findings": findings,
        "analyzersUsed": analyzers_used,
        "durationSeconds": duration_seconds,
    })
}
