use super::inventory::InventoryEntry;

pub fn has_local_changes(entry: &InventoryEntry) -> bool {
    entry.kind == "managed"
        && entry.recorded_revision.is_some()
        && entry.current_revision.is_some()
        && entry.recorded_revision != entry.current_revision
}

pub fn display_status(entry: &InventoryEntry) -> &'static str {
    if entry.kind == "unmanaged" {
        "Unmanaged"
    } else {
        "Managed"
    }
}

pub fn attention_message(entry: &InventoryEntry) -> Option<&'static str> {
    if has_local_changes(entry) {
        Some("Local changes detected. Source updates are disabled.")
    } else {
        None
    }
}

pub fn can_manage(entry: &InventoryEntry) -> bool {
    entry.kind == "unmanaged"
}

pub fn can_update(entry: &InventoryEntry) -> bool {
    entry.kind == "managed" && !has_local_changes(entry) && entry.source.kind == "github"
}

pub fn can_delete(entry: &InventoryEntry) -> bool {
    entry.kind == "managed"
        && entry.package_dir.is_some()
        && entry.package_path.is_some()
}

pub fn can_stop_managing(entry: &InventoryEntry) -> bool {
    can_delete(entry)
}

pub fn cell_state(entry: &InventoryEntry, harness: &str) -> &'static str {
    if entry.kind == "unmanaged" {
        if entry
            .sightings
            .iter()
            .any(|s| s.harness.as_deref() == Some(harness))
        {
            "found"
        } else {
            "empty"
        }
    } else if entry.sightings.iter().any(|s| {
        s.kind == "harness"
            && s.harness.as_deref() == Some(harness)
            && s.scope.as_deref() == Some("canonical")
            && entry.canonical_binding_is_merged(s)
    }) {
        "enabled"
    } else if entry
        .sightings
        .iter()
        .any(|s| s.kind == "harness" && s.harness.as_deref() == Some(harness))
    {
        "found"
    } else {
        "disabled"
    }
}

pub fn stop_managing_status(entry: &InventoryEntry) -> Option<&'static str> {
    if !can_stop_managing(entry) {
        return None;
    }
    if entry.linked_harnesses().is_empty() {
        Some("disabled_no_enabled")
    } else {
        Some("available")
    }
}

pub fn sort_entries(entries: &mut [InventoryEntry]) {
    entries.sort_by(|a, b| {
        let order = |entry: &InventoryEntry| match display_status(entry) {
            "Managed" => 0,
            _ => 1,
        };
        (
            order(a),
            a.name.to_lowercase(),
            a.skill_ref.clone(),
        )
            .cmp(&(order(b), b.name.to_lowercase(), b.skill_ref.clone()))
    });
}
