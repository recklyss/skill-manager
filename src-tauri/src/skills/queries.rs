use super::read_models::{SkillsPageResponse, SkillsReadModelService};

#[derive(Clone)]
pub struct SkillsQueryService {
    read_models: SkillsReadModelService,
}

impl SkillsQueryService {
    pub fn new(read_models: SkillsReadModelService) -> Self {
        Self { read_models }
    }

    pub fn page_response(&self) -> SkillsPageResponse {
        self.read_models.page_response()
    }

    pub fn health(&self) -> bool {
        true
    }
}
