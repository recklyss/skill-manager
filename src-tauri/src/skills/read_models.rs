use serde::Serialize;

use crate::harness::HarnessKernel;

use super::manifest::{SkillSource, SkillStatus};
use super::store::{SkillStore, StoredSkill};

/// Read model sent to the frontend — enriched with harness bindings.
#[derive(Debug, Serialize)]
pub struct SkillReadModel {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub status: SkillStatus,
    pub source: Option<SkillSource>,
    pub origin_harness: Option<String>,
    /// Harnesses where this skill is enabled.
    pub enabled_harnesses: Vec<String>,
    /// Whether this skill is managed (in shared store) vs. unmanaged.
    pub managed: bool,
}

#[derive(Clone)]
pub struct SkillsReadModelService {
    store: SkillStore,
    kernel: HarnessKernel,
}

impl SkillsReadModelService {
    pub fn new(store: SkillStore, kernel: HarnessKernel) -> Self {
        Self { store, kernel }
    }

    /// Build the read model for all managed skills.
    pub fn managed_skills(&self) -> Vec<SkillReadModel> {
        self.store
            .list_skills()
            .into_iter()
            .map(|s| self.to_read_model(s, true))
            .collect()
    }

    /// Unmanaged skills are discovered from harness skill directories.
    /// For now, return an empty list (will be implemented with harness probing).
    pub fn unmanaged_skills(&self) -> Vec<SkillReadModel> {
        vec![]
    }

    fn to_read_model(&self, skill: StoredSkill, managed: bool) -> SkillReadModel {
        let harnesses = self.kernel.statuses();
        let enabled_harnesses: Vec<String> = harnesses
            .iter()
            .filter(|h| h.installed)
            .map(|h| h.harness.clone())
            .collect();

        SkillReadModel {
            name: skill.name,
            description: skill.manifest.description,
            version: skill.manifest.version,
            author: skill.manifest.author,
            tags: skill.manifest.tags,
            status: skill.status,
            source: skill.source,
            origin_harness: skill.origin_harness,
            enabled_harnesses,
            managed,
        }
    }
}
