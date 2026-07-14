use super::executor::SlashCommandSyncExecutor;
use super::path_policy::SlashCommandPathPolicy;
use super::planner::SlashCommandPlanner;
use super::queries::SlashCommandQueryService;
use super::review_resolver::SlashCommandReviewResolver;
use super::store::{SlashCommand, SlashCommandStore, validate_command_name};
use super::sync_state::SlashCommandSyncStateStore;
use super::targets::{default_target_ids, target_by_id, SlashTarget};
use crate::error::{ApiError, ApiResult};
use serde_json::{json, Value};

#[derive(Clone)]
pub struct SlashCommandMutationService {
    store: SlashCommandStore,
    sync_state: SlashCommandSyncStateStore,
    queries: SlashCommandQueryService,
    targets: Vec<SlashTarget>,
    sync_executor: SlashCommandSyncExecutor,
    review_resolver: SlashCommandReviewResolver,
}

impl SlashCommandMutationService {
    pub fn new(
        store: SlashCommandStore,
        sync_state: SlashCommandSyncStateStore,
        queries: SlashCommandQueryService,
        targets: Vec<SlashTarget>,
        path_policy: SlashCommandPathPolicy,
    ) -> Self {
        let planner = SlashCommandPlanner::new(path_policy.clone());
        let sync_executor = SlashCommandSyncExecutor::new(
            sync_state.clone(),
            planner.clone(),
            path_policy.clone(),
        );
        let review_resolver = SlashCommandReviewResolver::new(
            store.clone(),
            sync_state.clone(),
            queries.clone(),
            path_policy,
        );
        Self {
            store,
            sync_state,
            queries,
            targets,
            sync_executor,
            review_resolver,
        }
    }

    pub fn create_command(
        &self,
        name: &str,
        description: &str,
        prompt: &str,
        targets: Option<Vec<String>>,
    ) -> ApiResult<Value> {
        let command = SlashCommand {
            name: name.to_string(),
            description: description.to_string(),
            prompt: prompt.to_string(),
        };
        self.store.create_command(&command).map_err(ApiError::bad_request)?;
        let sync = self.sync_command(name, targets)?;
        Ok(json!({
            "ok": sync.get("ok").cloned().unwrap_or(json!(true)),
            "command": self.queries.get_command(name),
            "sync": sync.get("sync").cloned().unwrap_or(json!([])),
        }))
    }

    pub fn update_command(
        &self,
        name: &str,
        description: &str,
        prompt: &str,
        targets: Option<Vec<String>>,
    ) -> ApiResult<Value> {
        self.store
            .update_command(name, description, prompt)
            .map_err(ApiError::bad_request)?;
        let sync = self.sync_command(name, targets)?;
        Ok(json!({
            "ok": sync.get("ok").cloned().unwrap_or(json!(true)),
            "command": self.queries.get_command(name),
            "sync": sync.get("sync").cloned().unwrap_or(json!([])),
        }))
    }

    pub fn sync_command(&self, name: &str, targets: Option<Vec<String>>) -> ApiResult<Value> {
        let command = self
            .store
            .get_command(name)
            .ok_or_else(|| ApiError::not_found(format!("unknown slash command: {name}")))?;
        let selected = self.selected_targets(targets)?;
        self.sync_executor
            .sync_command(&command, &selected, &self.targets)
            .map_err(ApiError::internal)
    }

    pub fn delete_command(&self, name: &str) -> ApiResult<Value> {
        validate_command_name(name).map_err(ApiError::bad_request)?;
        self.store
            .get_command(name)
            .ok_or_else(|| ApiError::not_found(format!("unknown slash command: {name}")))?;
        let records = self.sync_state.load().get(name).cloned().unwrap_or_default();
        let removed = self
            .sync_executor
            .remove_tracked_outputs(&records, &self.targets)
            .map_err(ApiError::internal)?;
        if removed
            .get("ok")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            self.store.delete_command(name).map_err(ApiError::bad_request)?;
            self.sync_state.remove_command(name);
            Ok(json!({ "ok": true, "sync": removed.get("sync").cloned().unwrap_or(json!([])) }))
        } else {
            Ok(removed)
        }
    }

    pub fn import_unmanaged_command(&self, target_id: &str, name: &str) -> ApiResult<Value> {
        let target = self.require_target(target_id)?.clone();
        self.review_resolver
            .import_unmanaged_command(&target, name)
            .map_err(map_mutation_error)
    }

    pub fn resolve_review_command(
        &self,
        target_id: &str,
        name: &str,
        action: &str,
    ) -> ApiResult<Value> {
        let target = self.require_target(target_id)?.clone();
        self.review_resolver
            .resolve_review_command(&target, name, action)
            .map_err(map_mutation_error)
    }

    fn selected_targets(&self, targets: Option<Vec<String>>) -> ApiResult<Vec<SlashTarget>> {
        let ids = targets.unwrap_or_else(|| default_target_ids(&self.targets));
        let mut selected = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for id in ids {
            if !seen.insert(id.clone()) {
                continue;
            }
            selected.push(self.require_target(&id)?.clone());
        }
        Ok(selected)
    }

    fn require_target(&self, target_id: &str) -> ApiResult<&SlashTarget> {
        let target = target_by_id(&self.targets, target_id)
            .ok_or_else(|| ApiError::bad_request(format!("unknown slash command target: {target_id}")))?;
        if !target.enabled {
            return Err(ApiError::bad_request(format!(
                "harness support is disabled: {target_id}"
            )));
        }
        Ok(target)
    }
}

fn map_mutation_error(error: String) -> ApiError {
    if error.contains("already exists") {
        ApiError::conflict(error)
    } else if error.contains("not found") || error.contains("unknown slash command") {
        ApiError::not_found(error)
    } else if error.contains("not managed") {
        ApiError::conflict(error)
    } else {
        ApiError::bad_request(error)
    }
}
