use std::path::{Component, Path, PathBuf};

use super::targets::SlashTarget;

#[derive(Clone, Default)]
pub struct SlashCommandPathPolicy;

impl SlashCommandPathPolicy {
    pub fn output_path(&self, target: &SlashTarget, command_name: &str) -> PathBuf {
        self.normalize(&target.output_dir.join(format!("{command_name}.md")))
    }

    pub fn tracked_path(&self, target: &SlashTarget, path: &Path) -> Result<PathBuf, String> {
        let normalized = self.normalize(path);
        let output_dir = self.normalize(&target.output_dir);
        if !normalized.starts_with(&output_dir) {
            return Err(format!(
                "tracked slash command path is outside {} locations: {}",
                target.label,
                path.display()
            ));
        }
        Ok(normalized)
    }

    pub fn path_identity(&self, path: &Path) -> String {
        self.normalize(path).to_string_lossy().into_owned()
    }

    fn normalize(&self, path: &Path) -> PathBuf {
        let expanded = expand_user(path);
        absolute_path(&expanded)
    }
}

fn expand_user(path: &Path) -> PathBuf {
    if let Some(raw) = path.to_str() {
        if let Some(rest) = raw.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(rest);
            }
        }
    }
    path.to_path_buf()
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return normalize_components(path);
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    normalize_components(&cwd.join(path))
}

fn normalize_components(path: &Path) -> PathBuf {
    let mut parts = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir => {}
            other => parts.push(other.as_os_str()),
        }
    }
    parts
}
