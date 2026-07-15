mod common;

use std::fs;
use std::path::PathBuf;

use common::{harness_ids, TestFixture};
use skill_manager_lib::harness::{HarnessKernelService, HarnessSupportStore};

fn test_kernel(settings_path: PathBuf) -> HarnessKernelService {
    let store = HarnessSupportStore::new(settings_path);
    HarnessKernelService::from_environment(None, store)
}

/// Harness kernel reports all catalog harnesses with install probes.
#[test]
fn harness_kernel_lists_catalog_harnesses() {
    let dir = tempfile::tempdir().expect("tempdir");
    let statuses = test_kernel(dir.path().join("settings.json")).statuses();
    assert_eq!(statuses.len(), harness_ids().len());

    let ids: Vec<&str> = statuses.iter().map(|s| s.harness.as_str()).collect();
    for expected in harness_ids() {
        assert!(ids.contains(&expected), "missing harness {expected}");
    }
}

/// Each harness has label and optional logo_key in the kernel model.
#[test]
fn harness_kernel_status_fields_present() {
    let dir = tempfile::tempdir().expect("tempdir");
    let statuses = test_kernel(dir.path().join("settings.json")).statuses();

    for status in &statuses {
        assert!(!status.label.is_empty());
        assert!(status.logo_key.is_some());
        assert!(status.managed_location.is_some());
    }
}

/// managed_location points at the expected skills root for codex.
#[test]
fn harness_codex_managed_location() {
    let home = dirs::home_dir().expect("HOME");
    let dir = tempfile::tempdir().expect("tempdir");
    let statuses = test_kernel(dir.path().join("settings.json")).statuses();
    let codex = statuses.iter().find(|s| s.harness == "codex").unwrap();
    assert_eq!(
        codex.managed_location.as_ref().unwrap(),
        &home.join(".agents").join("skills")
    );
}

/// managed_location points at the expected skills root for copilot.
#[test]
fn harness_copilot_managed_location() {
    let home = dirs::home_dir().expect("HOME");
    let dir = tempfile::tempdir().expect("tempdir");
    let statuses = test_kernel(dir.path().join("settings.json")).statuses();
    let copilot = statuses.iter().find(|s| s.harness == "copilot").unwrap();
    assert_eq!(
        copilot.managed_location.as_ref().unwrap(),
        &home.join(".copilot").join("skills")
    );
    assert_eq!(copilot.label, "GitHub Copilot");
    assert_eq!(copilot.logo_key.as_deref(), Some("copilot"));
}

/// Install detection runs without network (local which probe only).
#[test]
fn harness_install_probe_does_not_panic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let statuses = test_kernel(dir.path().join("settings.json")).statuses();
    for status in &statuses {
        let _ = status.installed;
    }
}

/// Support store persists disabled harnesses to settings.json (camelCase payload).
#[test]
fn harness_support_store_persists_disabled_harnesses() {
    let fixture = TestFixture::new();
    let store = HarnessSupportStore::new(fixture.paths.settings_path.clone());

    store.set_enabled("codex", false).expect("persist disable");

    let raw = fs::read_to_string(&fixture.paths.settings_path).expect("settings file");
    assert!(raw.contains("disabledHarnesses"));
    assert!(raw.contains("codex"));

    let prefs = store.load().expect("load prefs");
    assert!(!prefs.is_enabled("codex"));
}
