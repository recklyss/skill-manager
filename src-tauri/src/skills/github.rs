use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{ApiError, ApiResult};

#[derive(Debug, Clone)]
pub struct ResolvedGitHubSkill {
    pub repo: String,
    pub git_ref: Option<String>,
    pub relative_path: String,
    pub package_path: PathBuf,
}

pub fn github_repo_from_locator(locator: &str) -> Option<String> {
    let (_, repo, _) = parse_repo_identity(locator).ok()?;
    Some(repo)
}

pub fn github_repo_url(repo: &str) -> String {
    format!("https://github.com/{repo}")
}

pub fn github_folder_url(repo: &str, git_ref: Option<&str>, relative_path: Option<&str>) -> Option<String> {
    let git_ref = git_ref?;
    let normalized = normalize_relative_path(relative_path);
    if normalized == "." {
        return None;
    }
    Some(format!(
        "{}/tree/{}/{}",
        github_repo_url(repo),
        percent_encode(git_ref),
        percent_encode_path(&normalized)
    ))
}

pub fn parse_locator(locator: &str) -> ApiResult<(String, String, String)> {
    let (owner, repo, skill_dir) = parse_repo_identity(locator)?;
    let skill_dir = skill_dir.ok_or_else(|| {
        ApiError::bad_request(format!(
            "invalid github locator (expected owner/repo/<skill-path>): {locator}"
        ))
    })?;
    if skill_dir.trim().is_empty() {
        return Err(ApiError::bad_request(format!(
            "invalid github locator (expected owner/repo/<skill-path>): {locator}"
        )));
    }
    Ok((owner, repo, skill_dir))
}

fn parse_repo_identity(locator: &str) -> ApiResult<(String, String, Option<String>)> {
    let stripped = locator.strip_prefix("github:").unwrap_or(locator);
    let parts: Vec<&str> = stripped.split('/').collect();
    if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(ApiError::bad_request(format!(
            "invalid github locator (expected owner/repo or owner/repo/<skill-path>): {locator}"
        )));
    }
    let owner = parts[0].to_string();
    let repo = parts[1].to_string();
    let skill_dir = if parts.len() >= 3 {
        Some(parts[2..].join("/"))
    } else {
        None
    };
    let full_repo = format!("{owner}/{repo}");
    Ok((owner, full_repo, skill_dir))
}

fn normalize_relative_path(relative_path: Option<&str>) -> String {
    match relative_path {
        None => ".".to_string(),
        Some(path) => {
            let normalized = path.trim().trim_matches('/');
            if normalized.is_empty() {
                ".".to_string()
            } else {
                normalized.to_string()
            }
        }
    }
}

fn percent_encode(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn percent_encode_path(value: &str) -> String {
    value
        .split('/')
        .map(percent_encode)
        .collect::<Vec<_>>()
        .join("/")
}

pub fn resolve_github_skill(locator: &str, work_dir: &Path) -> ApiResult<ResolvedGitHubSkill> {
    let (owner, repo, skill_dir) = parse_locator(locator)?;
    let repo_name = repo.split('/').nth(1).unwrap_or(&repo);
    let clone_dir = work_dir.join(format!("{owner}--{repo_name}"));
    let output = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            &format!("https://github.com/{repo}.git"),
        ])
        .arg(&clone_dir)
        .output()
        .map_err(|error| ApiError::bad_request(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::bad_request(stderr.trim().to_string()));
    }

    let skill_path = find_skill(&clone_dir, &skill_dir).ok_or_else(|| {
        ApiError::bad_request(format!("skill directory '{skill_dir}' not found in {repo}"))
    })?;
    let relative_segment = skill_path
        .strip_prefix(&clone_dir)
        .ok()
        .map(|p| p.to_string_lossy().into_owned());
    let relative_path = normalize_relative_path(relative_segment.as_deref());

    Ok(ResolvedGitHubSkill {
        repo,
        git_ref: checked_out_ref(&clone_dir),
        relative_path,
        package_path: skill_path,
    })
}

fn find_skill(clone_dir: &Path, skill_dir: &str) -> Option<PathBuf> {
    for entry in walkdir::WalkDir::new(clone_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some("SKILL.md") {
            if path.parent().is_some_and(|parent| parent.file_name().and_then(|n| n.to_str()) == Some(skill_dir)) {
                return path.parent().map(Path::to_path_buf);
            }
        }
    }
    for entry in walkdir::WalkDir::new(clone_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) != Some("SKILL.md") {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            let mut in_frontmatter = false;
            for line in content.lines().skip(1) {
                if line.trim() == "---" {
                    break;
                }
                if line.starts_with("name:") {
                    let name_value = line
                        .split_once(':')
                        .map(|(_, v)| v.trim().trim_matches(['\'', '"']))
                        .unwrap_or("");
                    if name_value == skill_dir {
                        return path.parent().map(Path::to_path_buf);
                    }
                }
                if line.trim() == "---" {
                    in_frontmatter = true;
                }
                let _ = in_frontmatter;
            }
        }
    }
    None
}

fn checked_out_ref(clone_dir: &Path) -> Option<String> {
    let branch = Command::new("git")
        .args(["-C"])
        .arg(clone_dir)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if branch.status.success() {
        let value = String::from_utf8_lossy(&branch.stdout).trim().to_string();
        if !value.is_empty() && value != "HEAD" {
            return Some(value);
        }
    }
    let commit = Command::new("git")
        .args(["-C"])
        .arg(clone_dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if commit.status.success() {
        let value = String::from_utf8_lossy(&commit.stdout).trim().to_string();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    } else {
        None
    }
}
