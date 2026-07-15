mod config_service;
mod harness_scanner;
mod llm;
mod service;

pub use config_service::ScanConfigService;
pub use service::ScanService;

use crate::db::Database;
use crate::harness::HarnessKernelService;
use crate::skills::queries::SkillsQueryService;

#[derive(Clone)]
pub struct ScanServices {
    pub config: ScanConfigService,
    pub service: ScanService,
}

impl ScanServices {
    pub fn new(
        db: std::sync::Arc<Database>,
        harness_kernel: HarnessKernelService,
        skills_queries: SkillsQueryService,
    ) -> Self {
        Self {
            config: ScanConfigService::new(db),
            service: ScanService::new(harness_kernel, skills_queries),
        }
    }
}
