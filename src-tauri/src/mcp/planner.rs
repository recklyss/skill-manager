use super::adapters::McpReadModelService;
use super::identity::{build_identity_plan, ServerIdentityGroup};

#[derive(Clone)]
pub struct McpAdoptionPlanner {
    read_models: McpReadModelService,
}

impl McpAdoptionPlanner {
    pub fn new(read_models: McpReadModelService) -> Self {
        Self { read_models }
    }

    pub fn plan(&self) -> super::identity::AdoptionPlan {
        let snapshot = self.read_models.snapshot();
        let managed_names: Vec<_> = self
            .read_models
            .store()
            .list_records()
            .into_iter()
            .map(|s| s.name)
            .collect();
        build_identity_plan(&snapshot.harness_scans, &managed_names)
    }

    pub fn require_group(&self, name: &str) -> Result<ServerIdentityGroup, String> {
        self.plan()
            .groups
            .into_iter()
            .find(|group| group.name == name)
            .ok_or_else(|| format!("no unmanaged server named '{name}'"))
    }
}
