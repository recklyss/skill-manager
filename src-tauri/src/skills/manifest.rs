use serde::{Deserialize, Serialize};

/// Represents a skill's manifest metadata, parsed from SKILL.md frontmatter
/// or a manifest.json file in the skill directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl SkillManifest {
    /// Parse a skill manifest from a SKILL.md file content.
    /// Expects YAML frontmatter between --- delimiters.
    pub fn from_skill_md(content: &str) -> Option<Self> {
        let mut lines = content.lines();
        let first = lines.next()?;
        if first.trim() != "---" {
            return None;
        }
        let mut yaml_lines = Vec::new();
        for line in lines.by_ref() {
            if line.trim() == "---" {
                break;
            }
            yaml_lines.push(line);
        }
        if yaml_lines.is_empty() {
            return None;
        }
        let yaml_str = yaml_lines.join("\n");
        serde_yaml::from_str(&yaml_str).ok()
    }

    /// Create a minimal manifest from a directory name (fallback).
    pub fn from_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            version: "0.0.0".into(),
            author: String::new(),
            tags: vec![],
        }
    }
}

/// Status of a skill in the shared store.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SkillStatus {
    Ok,
    Missing,
    Broken,
}

/// Represents the source of a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SkillSource {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "github")]
    GitHub {
        repo: String,
        #[serde(default)]
        path: String,
        #[serde(default)]
        ref_: String,
    },
    #[serde(rename = "marketplace")]
    Marketplace { package: String },
}
