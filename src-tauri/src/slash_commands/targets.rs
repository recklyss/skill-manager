use std::path::PathBuf;

use crate::harness::{BindingProfile, FamilyKey};
use crate::harness::HarnessKernelService;

const TARGET_ORDER: &[&str] = &["opencode", "claude", "cursor", "codex"];

#[derive(Debug, Clone)]
pub struct SlashTarget {
    pub id: String,
    pub label: String,
    pub root_path: PathBuf,
    pub output_dir: PathBuf,
    pub invocation_prefix: String,
    pub render_format: String,
    pub file_glob: String,
    pub enabled: bool,
    pub available: bool,
    pub default_selected: bool,
}

pub fn resolve_slash_targets(kernel: &HarnessKernelService) -> Vec<SlashTarget> {
    let enabled: std::collections::HashSet<_> = kernel
        .enabled_harness_ids_for_family(FamilyKey::SlashCommands)
        .into_iter()
        .collect();
    let mut targets = std::collections::HashMap::new();

    for binding in kernel.bindings_for_family(FamilyKey::SlashCommands) {
        let profile = match binding.profile {
            BindingProfile::CommandFile(profile) => profile,
            _ => continue,
        };
        let target_id = binding.definition.harness.to_string();
        let root_path = profile.resolve_root_path(&kernel.context);
        let output_dir = profile.resolve_output_dir(&kernel.context);
        let is_enabled = enabled.contains(&target_id);
        let available = root_path.exists();
        targets.insert(
            target_id.clone(),
            SlashTarget {
                id: target_id,
                label: binding.definition.label.to_string(),
                root_path,
                output_dir,
                invocation_prefix: profile.invocation_prefix.to_string(),
                render_format: profile.render_format.as_str().to_string(),
                file_glob: profile.file_glob.to_string(),
                enabled: is_enabled,
                available,
                default_selected: is_enabled && available,
            },
        );
    }

    TARGET_ORDER
        .iter()
        .filter_map(|id| targets.get(*id).cloned())
        .collect()
}

pub fn default_target_ids(targets: &[SlashTarget]) -> Vec<String> {
    targets
        .iter()
        .filter(|t| t.default_selected)
        .map(|t| t.id.clone())
        .collect()
}

pub fn target_by_id<'a>(targets: &'a [SlashTarget], target_id: &str) -> Option<&'a SlashTarget> {
    targets.iter().find(|t| t.id == target_id)
}
