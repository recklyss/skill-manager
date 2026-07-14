mod catalog;
mod contracts;
mod resolution;
pub mod support_store;

pub use catalog::{supported_harness_ids, SUPPORTED_HARNESS_DEFINITIONS};
pub use contracts::{
    BindingProfile, ConfigFileFormat, ConfigSubtreeBindingProfile, FamilyBinding, FamilyKey,
    FileTreeDiscoveryRoot, FileTreeLayout, HarnessDefinition, HarnessStatus,
};
pub use resolution::{resolve_context, Platform, ResolutionContext};
pub use support_store::HarnessSupportStore;

use contracts::{BindingProfile as BP, FileTreeAvailability};

/// Central harness resolver shared across API domains.
#[derive(Clone)]
pub struct HarnessKernelService {
    definitions: &'static [HarnessDefinition],
    pub context: ResolutionContext,
    pub support_store: HarnessSupportStore,
}

/// Back-compat alias used by the skills read-model layer.
pub type HarnessKernel = HarnessKernelService;

impl HarnessKernelService {
    pub fn from_environment(
        env: Option<std::collections::HashMap<String, String>>,
        support_store: HarnessSupportStore,
    ) -> Self {
        Self {
            definitions: SUPPORTED_HARNESS_DEFINITIONS,
            context: resolve_context(env),
            support_store,
        }
    }

    pub fn supported_harness_ids(&self) -> Vec<&'static str> {
        supported_harness_ids()
    }

    pub fn is_known_harness(&self, harness: &str) -> bool {
        self.definitions
            .iter()
            .any(|definition| definition.harness == harness)
    }

    pub fn definition(&self, harness: &str) -> Option<&HarnessDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.harness == harness)
    }

    pub fn enabled_harness_ids(&self) -> Vec<String> {
        self.support_store
            .enabled_harnesses(&self.supported_harness_ids())
            .unwrap_or_default()
    }

    pub fn enabled_harness_ids_for_family(&self, family: FamilyKey) -> Vec<String> {
        let supported: Vec<&str> = self
            .bindings_for_family(family)
            .into_iter()
            .map(|binding| binding.definition.harness)
            .collect();
        self.support_store
            .enabled_harnesses(&supported)
            .unwrap_or_default()
    }

    pub fn bindings_for_family(&self, family: FamilyKey) -> Vec<FamilyBinding<'_>> {
        self.definitions
            .iter()
            .filter_map(|definition| {
                definition
                    .binding_for(family)
                    .map(|profile| FamilyBinding {
                        definition,
                        profile,
                    })
            })
            .collect()
    }

    pub fn binding_for(&self, harness: &str, family: FamilyKey) -> Option<&BindingProfile> {
        self.definition(harness)
            .and_then(|definition| definition.binding_for(family))
    }

    pub fn harness_statuses(&self) -> Vec<HarnessStatus> {
        self.definitions
            .iter()
            .map(|definition| {
                let skills_binding = definition.binding_for(FamilyKey::Skills);
                let managed_location = skills_binding.and_then(|profile| match profile {
                    BP::FileTree(binding) => Some(binding.resolve_managed_root(&self.context)),
                    _ => None,
                });
                HarnessStatus {
                    harness: definition.harness.to_string(),
                    label: definition.label.to_string(),
                    logo_key: definition.logo_key.map(str::to_string),
                    installed: self.is_installed(definition, skills_binding),
                    managed_location,
                }
            })
            .collect()
    }

    /// Back-compat helper for the skills read-model layer.
    pub fn statuses(&self) -> Vec<HarnessStatus> {
        self.harness_statuses()
    }

    fn is_installed(
        &self,
        definition: &HarnessDefinition,
        skills_binding: Option<&BindingProfile>,
    ) -> bool {
        if which::which(definition.install_probe).is_ok() {
            return true;
        }

        let Some(BP::FileTree(binding)) = skills_binding else {
            return false;
        };

        if binding.availability != FileTreeAvailability::CliOrApp {
            return false;
        }

        binding
            .app_probe_paths
            .iter()
            .map(|resolver| resolver(&self.context))
            .any(|path| path.exists())
    }
}
