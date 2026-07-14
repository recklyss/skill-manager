mod adapters;
mod availability;
mod config_choice;
mod contracts;
mod env;
mod harness_application;
mod identity;
mod inventory;
mod managed_state;
mod mappers;
mod mutations;
mod planner;
mod queries;
mod redaction;
mod store;

pub use adapters::McpReadModelService;
pub use mutations::McpMutationService;
pub use queries::McpQueryService;
pub use store::McpServerStore;
pub use availability::{McpEnrichmentService, MarketplaceLink};

use crate::harness::HarnessKernelService;
use crate::marketplace::McpMarketplaceService;
use crate::paths::AppPaths;

use availability::McpEnrichmentService as EnrichmentService;
use planner::McpAdoptionPlanner;

#[derive(Clone)]
pub struct McpServices {
    pub store: McpServerStore,
    pub queries: McpQueryService,
    pub mutations: McpMutationService,
}

impl McpServices {
    pub fn new(
        paths: &AppPaths,
        kernel: &HarnessKernelService,
        marketplace: &McpMarketplaceService,
    ) -> Self {
        let store = McpServerStore::new(paths.mcp_store_manifest.clone());
        let read_models = McpReadModelService::new(store.clone(), kernel.clone());
        let planner = McpAdoptionPlanner::new(read_models.clone());
        let enrichment = EnrichmentService::new(marketplace.clone());
        let queries = McpQueryService::new(read_models.clone(), planner.clone(), enrichment.clone());
        let mutations = McpMutationService::new(
            store.clone(),
            read_models,
            planner,
            marketplace.clone(),
            enrichment,
        );
        Self {
            store,
            queries,
            mutations,
        }
    }
}
