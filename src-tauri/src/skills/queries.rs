use std::path::PathBuf;

use crate::error::{ApiError, ApiResult};

use super::github::{github_folder_url, github_repo_from_locator, github_repo_url};
use super::inventory::InventoryEntry;
use super::package::fingerprint_package;
use super::policy::{can_stop_managing, can_update, has_local_changes};
use super::read_models::{
    SkillDetailResponse, SkillSourceLinksResponse, SkillSourceStatusResponse, SkillsPageResponse,
    SkillsReadModelService,
};
use super::source_fetch::SourceFetchService;

#[derive(Clone)]
pub struct SkillsQueryService {
    read_models: SkillsReadModelService,
    source_fetcher: SourceFetchService,
}

impl SkillsQueryService {
    pub fn new(read_models: SkillsReadModelService, source_fetcher: SourceFetchService) -> Self {
        Self {
            read_models,
            source_fetcher,
        }
    }

    pub fn page_response(&self) -> SkillsPageResponse {
        self.read_models.page_response()
    }

    pub fn get_skill_detail(&self, skill_ref: &str) -> Option<SkillDetailResponse> {
        let inventory = self.read_models.inventory();
        let entry = inventory.find(skill_ref)?;
        let mut detail = self.read_models.detail_response(entry);
        detail.source_links = self.build_source_links(entry);
        Some(detail)
    }

    pub fn get_skill_source_status(&self, skill_ref: &str) -> Option<SkillSourceStatusResponse> {
        let inventory = self.read_models.inventory();
        let entry = inventory.find(skill_ref)?;
        Some(SkillSourceStatusResponse {
            update_status: self.resolve_update_status(entry),
        })
    }

    pub fn require_entry(&self, skill_ref: &str) -> ApiResult<InventoryEntry> {
        self.read_models
            .inventory()
            .find(skill_ref)
            .cloned()
            .ok_or_else(|| ApiError::not_found(format!("unknown skill ref: {skill_ref}")))
    }

    pub fn read_models(&self) -> &SkillsReadModelService {
        &self.read_models
    }

    pub fn source_fetcher(&self) -> &SourceFetchService {
        &self.source_fetcher
    }

    pub fn check_for_update(&self, entry: &InventoryEntry) -> Option<bool> {
        if !can_update(entry) || entry.current_revision.is_none() {
            return None;
        }
        let work_dir = tempfile::tempdir().ok()?;
        let skill_path = self
            .source_fetcher
            .fetch(
                &entry.source.kind,
                &entry.source.locator,
                work_dir.path(),
            )
            .ok()?;
        let (fetched_revision, _) = fingerprint_package(&skill_path).ok()?;
        Some(fetched_revision != entry.current_revision.as_deref().unwrap_or(""))
    }

    pub fn build_source_links(&self, entry: &InventoryEntry) -> Option<SkillSourceLinksResponse> {
        if entry.source.kind != "github" {
            return None;
        }
        let repo = github_repo_from_locator(&entry.source.locator)?;
        Some(SkillSourceLinksResponse {
            repo_label: repo.clone(),
            repo_url: github_repo_url(&repo),
            folder_url: self.github_folder_url(entry, &repo),
        })
    }

    fn github_folder_url(&self, entry: &InventoryEntry, repo: &str) -> Option<String> {
        if entry.source_ref.is_some() && entry.source_path.is_some() {
            return github_folder_url(
                repo,
                entry.source_ref.as_deref(),
                entry.source_path.as_deref(),
            );
        }
        let locator = entry
            .source
            .locator
            .strip_prefix("github:")
            .unwrap_or(&entry.source.locator);
        if locator.matches('/').count() < 2 {
            return None;
        }
        let work_dir = tempfile::tempdir().ok()?;
        let fetched = self
            .source_fetcher
            .fetch_package(
                &entry.source.kind,
                &entry.source.locator,
                work_dir.path(),
            )
            .ok()?;
        github_folder_url(
            repo,
            fetched.source_ref.as_deref(),
            fetched.source_path.as_deref(),
        )
    }

    fn resolve_update_status(&self, entry: &InventoryEntry) -> Option<String> {
        if entry.kind != "managed" {
            return None;
        }
        if has_local_changes(entry) {
            return Some("local_changes_detected".into());
        }
        if !can_update(entry) {
            return Some("no_source_available".into());
        }
        if self.check_for_update(entry).unwrap_or(false) {
            return Some("update_available".into());
        }
        Some("no_update_available".into())
    }

    pub fn can_stop_managing(&self, entry: &InventoryEntry) -> bool {
        can_stop_managing(entry)
    }

    pub fn get_skill_path(&self, skill_ref: &str) -> Option<PathBuf> {
        let inventory = self.read_models.inventory();
        let entry = inventory.find(skill_ref)?;
        if let Some(path) = &entry.package_path {
            if path.join("SKILL.md").is_file() {
                return Some(path.clone());
            }
        }
        for sighting in entry.detail_sightings() {
            if let Some(path) = &sighting.path {
                if path.join("SKILL.md").is_file() {
                    return Some(path.clone());
                }
            }
        }
        None
    }
}
