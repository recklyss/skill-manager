use std::fs;
use std::path::{Path, PathBuf};

use crate::paths::AppPaths;

use super::manifest::{SkillManifest, SkillSource, SkillStatus};

/// A single skill stored in the shared store.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StoredSkill {
    pub name: String,
    pub path: PathBuf,
    pub manifest: SkillManifest,
    pub source: Option<SkillSource>,
    pub origin_harness: Option<String>,
    pub status: SkillStatus,
    pub harnesses: Vec<String>,
}

/// Filesystem-based skill store. Skills live as directories under
/// `data_dir/shared/`. Each skill has at minimum a SKILL.md file.
#[derive(Clone)]
pub struct SkillStore {
    root: PathBuf,
}

impl SkillStore {
    pub fn new(paths: &AppPaths) -> Self {
        Self {
            root: paths.skills_store_root.clone(),
        }
    }

    /// Ensure the store root exists.
    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)
    }

    /// List all skill directories in the store.
    pub fn list_skills(&self) -> Vec<StoredSkill> {
        let mut skills = Vec::new();
        let dir = match fs::read_dir(&self.root) {
            Ok(d) => d,
            Err(_) => return skills,
        };

        for entry in dir.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let manifest = self.read_manifest(&path).unwrap_or_else(|| {
                SkillManifest::from_name(&name)
            });

            let status = self.check_status(&path);

            skills.push(StoredSkill {
                name,
                path,
                manifest,
                source: None,
                origin_harness: None,
                status,
                harnesses: vec![],
            });
        }

        skills
    }

    /// Get a skill by name.
    pub fn get_skill(&self, name: &str) -> Option<StoredSkill> {
        let path = self.root.join(name);
        if !path.is_dir() {
            return None;
        }

        let manifest = self
            .read_manifest(&path)
            .unwrap_or_else(|| SkillManifest::from_name(name));

        let status = self.check_status(&path);

        Some(StoredSkill {
            name: name.to_string(),
            path,
            manifest,
            source: None,
            origin_harness: None,
            status,
            harnesses: vec![],
        })
    }

    /// Check if a skill directory has a valid SKILL.md.
    fn check_status(&self, path: &Path) -> SkillStatus {
        let skill_md = path.join("SKILL.md");
        if skill_md.exists() {
            SkillStatus::Ok
        } else {
            SkillStatus::Missing
        }
    }

    fn read_manifest(&self, path: &Path) -> Option<SkillManifest> {
        let skill_md = path.join("SKILL.md");
        let content = fs::read_to_string(&skill_md).ok()?;
        SkillManifest::from_skill_md(&content)
    }
}
