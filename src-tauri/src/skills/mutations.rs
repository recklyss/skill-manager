use std::fs;

use super::read_models::SkillsReadModelService;
use super::store::SkillStore;

pub struct SkillsMutationService {
    store: SkillStore,
    read_models: SkillsReadModelService,
}

impl SkillsMutationService {
    pub fn new(store: SkillStore, read_models: SkillsReadModelService) -> Self {
        Self {
            store,
            read_models,
        }
    }

    /// Remove a skill from the shared store.
    pub fn remove_skill(&self, name: &str) -> Result<(), String> {
        let skill = self
            .store
            .get_skill(name)
            .ok_or_else(|| format!("skill '{}' not found", name))?;

        fs::remove_dir_all(&skill.path).map_err(|e| format!("failed to remove skill: {}", e))?;
        Ok(())
    }
}
