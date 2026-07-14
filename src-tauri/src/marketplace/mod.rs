mod cache;
mod clis;
mod http;
pub mod installation;
pub mod installs;
mod mcp;
mod skills;
mod tokens;

pub use clis::CliMarketplaceService;
pub use installation::{
    enrich_skill_item, enrich_skill_marketplace_payload,
};
pub use installs::install_skill;
pub use mcp::McpMarketplaceService;
pub use skills::SkillsMarketplaceService;

use crate::paths::AppPaths;

#[derive(Clone)]
pub struct MarketplaceServices {
    pub skills: SkillsMarketplaceService,
    pub mcp: McpMarketplaceService,
    pub clis: CliMarketplaceService,
}

impl MarketplaceServices {
    pub fn new(paths: &AppPaths) -> Self {
        let cache_root = paths.marketplace_cache_root.clone();
        Self {
            skills: SkillsMarketplaceService::new(cache_root.clone()),
            mcp: McpMarketplaceService::new(cache_root.clone()),
            clis: CliMarketplaceService::new(cache_root),
        }
    }
}
