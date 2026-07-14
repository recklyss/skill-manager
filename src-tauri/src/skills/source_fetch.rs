use std::path::{Path, PathBuf};

use crate::error::{ApiError, ApiResult};

use super::github::resolve_github_skill;

#[derive(Debug, Clone)]
pub struct FetchedSourcePackage {
    pub package_path: PathBuf,
    pub source_ref: Option<String>,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SourceFetchService;

impl SourceFetchService {
    pub fn new() -> Self {
        Self
    }

    pub fn fetch_package(
        &self,
        source_kind: &str,
        source_locator: &str,
        work_dir: &Path,
    ) -> ApiResult<FetchedSourcePackage> {
        if source_kind == "github" {
            let locator = source_locator.strip_prefix("github:").unwrap_or(source_locator);
            let resolved = resolve_github_skill(locator, work_dir)?;
            return Ok(FetchedSourcePackage {
                package_path: resolved.package_path,
                source_ref: resolved.git_ref,
                source_path: Some(resolved.relative_path),
            });
        }
        Err(ApiError::bad_request(format!("unsupported source kind: {source_kind}")))
    }

    pub fn fetch(&self, source_kind: &str, source_locator: &str, work_dir: &Path) -> ApiResult<PathBuf> {
        Ok(self
            .fetch_package(source_kind, source_locator, work_dir)?
            .package_path)
    }
}
