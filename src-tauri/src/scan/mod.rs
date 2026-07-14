mod config_service;
mod llm;
mod service;

pub use config_service::ScanConfigService;
pub use service::ScanService;

use crate::db::Database;
use crate::skills::queries::SkillsQueryService;

#[derive(Clone)]
pub struct ScanServices {
    pub config: ScanConfigService,
    pub service: ScanService,
}

impl ScanServices {
    pub fn new(db: std::sync::Arc<Database>, skills_queries: SkillsQueryService) -> Self {
        Self {
            config: ScanConfigService::new(db.clone()),
            service: ScanService::new(db, skills_queries),
        }
    }
}
