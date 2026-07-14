use super::read_models::{SkillReadModel, SkillsReadModelService};

#[derive(Clone)]
pub struct SkillsQueryService {
    read_models: SkillsReadModelService,
}

impl SkillsQueryService {
    pub fn new(read_models: SkillsReadModelService) -> Self {
        Self { read_models }
    }

    /// Get all skills for the frontend (in-use / needs-review split).
    pub fn all_skills(&self) -> SkillsListResponse {
        let managed = self.read_models.managed_skills();
        let unmanaged = self.read_models.unmanaged_skills();

        SkillsListResponse {
            managed,
            unmanaged,
        }
    }

    /// Health check — verifies the store is accessible.
    pub fn health(&self) -> bool {
        true
    }
}

#[derive(serde::Serialize)]
pub struct SkillsListResponse {
    pub managed: Vec<SkillReadModel>,
    pub unmanaged: Vec<SkillReadModel>,
}
