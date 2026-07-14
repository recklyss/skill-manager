mod codecs;
mod executor;
mod mutations;
mod path_policy;
mod planner;
mod queries;
mod review_resolver;
mod store;
mod sync_state;
mod targets;

pub use mutations::SlashCommandMutationService;
pub use queries::SlashCommandQueryService;
pub use store::SlashCommandStore;
pub use sync_state::SlashCommandSyncStateStore;

use crate::harness::HarnessKernelService;
use crate::paths::AppPaths;
use path_policy::SlashCommandPathPolicy;

#[derive(Clone)]
pub struct SlashCommandServices {
    pub store: SlashCommandStore,
    pub sync_state: SlashCommandSyncStateStore,
    pub queries: SlashCommandQueryService,
    pub mutations: SlashCommandMutationService,
}

impl SlashCommandServices {
    pub fn new(paths: &AppPaths, kernel: &HarnessKernelService) -> Self {
        let store = SlashCommandStore::new(paths.slash_command_commands_dir.clone());
        let sync_state = SlashCommandSyncStateStore::new(paths.slash_command_sync_state_path.clone());
        let targets = targets::resolve_slash_targets(kernel);
        let path_policy = SlashCommandPathPolicy::default();
        let queries = SlashCommandQueryService::new(
            store.clone(),
            sync_state.clone(),
            paths.slash_command_commands_dir.clone(),
            paths.slash_command_sync_state_path.clone(),
            targets.clone(),
            path_policy.clone(),
        );
        let mutations = SlashCommandMutationService::new(
            store.clone(),
            sync_state.clone(),
            queries.clone(),
            targets,
            path_policy,
        );
        Self {
            store,
            sync_state,
            queries,
            mutations,
        }
    }
}
