use serde::Serialize;

use crate::harness::HarnessKernel;

use super::store::SkillStore;

#[derive(Debug, Serialize)]
pub struct HarnessColumnResponse {
    pub harness: String,
    pub installed: bool,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logoKey: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HarnessCellResponse {
    pub harness: String,
    pub interactive: bool,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logoKey: Option<String>,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct SkillRowActionsResponse {
    pub canDelete: bool,
    pub canManage: bool,
    pub canStopManaging: bool,
}

#[derive(Debug, Serialize)]
pub struct SkillTableRowResponse {
    pub actions: SkillRowActionsResponse,
    pub cells: Vec<HarnessCellResponse>,
    pub description: String,
    pub displayStatus: String,
    pub name: String,
    pub skillRef: String,
}

#[derive(Debug, Serialize)]
pub struct SkillsSummaryResponse {
    pub managed: usize,
    pub unmanaged: usize,
}

#[derive(Debug, Serialize)]
pub struct SkillsPageResponse {
    #[serde(rename = "harnessColumns")]
    pub harness_columns: Vec<HarnessColumnResponse>,
    pub rows: Vec<SkillTableRowResponse>,
    pub summary: SkillsSummaryResponse,
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

    pub fn page_response(&self) -> SkillsPageResponse {
        let statuses = self.kernel.statuses();
        let installed_harnesses: Vec<String> = statuses
            .iter()
            .filter(|h| h.installed)
            .map(|h| h.harness.clone())
            .collect();

        let harness_columns: Vec<HarnessColumnResponse> = statuses
            .iter()
            .map(|h| HarnessColumnResponse {
                harness: h.harness.clone(),
                installed: h.installed,
                label: h.label.clone(),
                logoKey: h.logo_key.clone(),
            })
            .collect();

        let skills = self.store.list_skills();
        let managed_count = skills.len();

        let rows: Vec<SkillTableRowResponse> = skills
            .into_iter()
            .map(|s| {
                let cells: Vec<HarnessCellResponse> = statuses
                    .iter()
                    .map(|h| {
                        let state = if h.installed && installed_harnesses.contains(&h.harness) {
                            "enabled"
                        } else if h.installed {
                            "disabled"
                        } else {
                            "empty"
                        };
                        HarnessCellResponse {
                            harness: h.harness.clone(),
                            interactive: h.installed,
                            label: h.label.clone(),
                            logoKey: h.logo_key.clone(),
                            state: state.to_string(),
                        }
                    })
                    .collect();

                SkillTableRowResponse {
                    actions: SkillRowActionsResponse {
                        canDelete: true,
                        canManage: true,
                        canStopManaging: true,
                    },
                    cells,
                    description: s.manifest.description,
                    displayStatus: "Managed".to_string(),
                    name: s.name.clone(),
                    skillRef: format!("managed:{}", s.name),
                }
            })
            .collect();

        SkillsPageResponse {
            harness_columns,
            rows,
            summary: SkillsSummaryResponse {
                managed: managed_count,
                unmanaged: 0,
            },
        }
    }
}
