use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::identity::SourceDescriptor;

#[derive(Debug, Clone)]
pub struct SkillParseError(pub String);

impl std::fmt::Display for SkillParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct SkillManifestFields {
    pub declared_name: String,
    pub description: String,
    pub source_kind: Option<String>,
    pub source_locator: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SkillPackage {
    pub declared_name: String,
    pub description: String,
    pub root_path: PathBuf,
    pub resolved_path: PathBuf,
    pub relative_files: Vec<String>,
    pub revision: String,
    pub source: SourceDescriptor,
}

pub fn find_skill_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if !root.is_dir() {
        return roots;
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return roots;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            roots.push(path);
        }
    }
    roots.sort();
    roots
}

pub fn find_plugin_skill_containers(root: &Path) -> Vec<PathBuf> {
    let mut containers = Vec::new();
    collect_plugin_skill_containers(root, &mut containers);
    containers.sort();
    containers.dedup();
    containers
}

fn collect_plugin_skill_containers(dir: &Path, containers: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.starts_with('.') {
            continue;
        }
        if name == "skills" {
            containers.push(path);
            continue;
        }
        collect_plugin_skill_containers(&path, containers);
    }
}

pub fn fingerprint_package(root: &Path) -> Result<(String, Vec<String>), SkillParseError> {
    if !root.is_dir() {
        return Err(SkillParseError(format!(
            "skill root does not exist: {}",
            root.display()
        )));
    }
    let mut digest = Sha256::new();
    let mut relative_files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() || path.file_name().is_some_and(|n| n == ".DS_Store") {
            continue;
        }
        let relative_path = path
            .strip_prefix(root)
            .map_err(|e| SkillParseError(e.to_string()))?
            .to_string_lossy()
            .replace('\\', "/");
        relative_files.push(relative_path.clone());
        digest.update(relative_path.as_bytes());
        digest.update([0u8]);
        let bytes = std::fs::read(path).map_err(|e| SkillParseError(e.to_string()))?;
        digest.update(&bytes);
        digest.update([0u8]);
    }
    relative_files.sort();
    if !relative_files.iter().any(|f| f == "SKILL.md") {
        return Err(SkillParseError(format!("missing SKILL.md in {}", root.display())));
    }
    Ok((format!("{:x}", digest.finalize()), relative_files))
}

pub fn parse_skill_package(
    root: &Path,
    default_source: SourceDescriptor,
) -> Result<SkillPackage, SkillParseError> {
    let skill_path = root.join("SKILL.md");
    if !skill_path.is_file() {
        return Err(SkillParseError(format!("missing SKILL.md in {}", root.display())));
    }
    let content = std::fs::read_to_string(&skill_path).map_err(|e| SkillParseError(e.to_string()))?;
    let manifest = parse_skill_manifest_text(&content)?;
    let (fingerprint, relative_files) = fingerprint_package(root)?;
    let source = resolve_source(&manifest, &default_source);
    Ok(SkillPackage {
        declared_name: manifest.declared_name,
        description: manifest.description,
        root_path: root.to_path_buf(),
        resolved_path: root.canonicalize().unwrap_or_else(|_| root.to_path_buf()),
        relative_files,
        revision: fingerprint,
        source,
    })
}

pub fn parse_skill_manifest_text(document: &str) -> Result<SkillManifestFields, SkillParseError> {
    let metadata = parse_frontmatter(document);
    let declared_name = extract_declared_name(document, &metadata)?;
    Ok(SkillManifestFields {
        declared_name,
        description: normalize_scalar(metadata.get("description").map(String::as_str).unwrap_or("")),
        source_kind: optional_value(&metadata, "source_kind"),
        source_locator: optional_value(&metadata, "source_locator"),
    })
}

fn resolve_source(manifest: &SkillManifestFields, default_source: &SourceDescriptor) -> SourceDescriptor {
    if let (Some(kind), Some(locator)) = (&manifest.source_kind, &manifest.source_locator) {
        if !kind.is_empty() && !locator.is_empty() {
            return SourceDescriptor::new(kind.clone(), locator.clone());
        }
    }
    default_source.clone()
}

fn extract_declared_name(
    document: &str,
    metadata: &std::collections::HashMap<String, String>,
) -> Result<String, SkillParseError> {
    if let Some(name) = metadata.get("name") {
        let trimmed = normalize_scalar(name);
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }
    for line in document.lines() {
        let stripped = line.trim();
        if let Some(title) = stripped.strip_prefix("# ") {
            let title = title.trim();
            if !title.is_empty() {
                return Ok(title.to_string());
            }
        }
    }
    Err(SkillParseError("unable to determine declared skill name".into()))
}

fn parse_frontmatter(document: &str) -> std::collections::HashMap<String, String> {
    let mut metadata = std::collections::HashMap::new();
    let lines: Vec<&str> = document.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return metadata;
    }
    let mut i = 1usize;
    while i < lines.len() {
        let raw_line = lines[i];
        if raw_line.trim() == "---" {
            break;
        }
        if let Some((key, value)) = raw_line.split_once(':') {
            let mut value = value.trim().to_string();
            if matches!(value.as_str(), ">-" | ">" | "|" | "|-") {
                let join_char = if value.starts_with('>') { " " } else { "\n" };
                let mut continuation = Vec::new();
                i += 1;
                while i < lines.len() {
                    let cont_line = lines[i];
                    if cont_line.trim() == "---" {
                        break;
                    }
                    if !cont_line.is_empty() && !cont_line.starts_with(' ') && !cont_line.starts_with('\t') {
                        break;
                    }
                    continuation.push(cont_line.trim());
                    i += 1;
                }
                value = continuation.into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(join_char);
            } else {
                value = normalize_scalar(&value);
                i += 1;
            }
            metadata.insert(key.trim().to_string(), value);
            continue;
        }
        i += 1;
    }
    metadata
}

fn optional_value(metadata: &std::collections::HashMap<String, String>, key: &str) -> Option<String> {
    let value = normalize_scalar(metadata.get(key).map(String::as_str).unwrap_or(""));
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn normalize_scalar(value: &str) -> String {
    let normalized = value.trim();
    if normalized.len() >= 2 {
        let bytes = normalized.as_bytes();
        let quote = bytes[0];
        if (quote == b'\'' || quote == b'"') && bytes[bytes.len() - 1] == quote {
            return normalized[1..normalized.len() - 1].trim().to_string();
        }
    }
    normalized.to_string()
}
