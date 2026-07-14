use std::path::PathBuf;

use super::resolution::{PathFn, ResolutionContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FamilyKey {
    Skills,
    Mcp,
    SlashCommands,
}

impl FamilyKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Skills => "skills",
            Self::Mcp => "mcp",
            Self::SlashCommands => "slash_commands",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTreeAvailability {
    Cli,
    CliOrApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTreeLayout {
    Flat,
    Categorized,
}

#[derive(Debug, Clone, Copy)]
pub struct FileTreeDiscoveryRoot {
    pub kind: &'static str,
    pub scope: &'static str,
    pub label: &'static str,
    pub path_resolver: PathFn,
}

#[derive(Debug, Clone)]
pub struct FileTreeBindingProfile {
    pub managed_env: Option<&'static str>,
    pub managed_default: PathFn,
    pub discovery_roots: &'static [FileTreeDiscoveryRoot],
    pub availability: FileTreeAvailability,
    pub app_probe_paths: &'static [PathFn],
    pub layout: FileTreeLayout,
    pub default_category: Option<&'static str>,
}

impl FileTreeBindingProfile {
    pub fn resolve_managed_root(&self, context: &ResolutionContext) -> PathBuf {
        if let Some(env_key) = self.managed_env {
            if let Some(override_path) = context.env.get(env_key) {
                return PathBuf::from(override_path);
            }
        }
        (self.managed_default)(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileFormat {
    Json,
    Jsonc,
    Toml,
    Yaml,
}

#[derive(Debug, Clone)]
pub struct ConfigSubtreeBindingProfile {
    pub config_path_resolver: PathFn,
    pub file_format: ConfigFileFormat,
    pub subtree_path: &'static [&'static str],
    pub codec: &'static str,
    pub capability_probe: Option<&'static str>,
    pub capability_unavailable_reason: Option<&'static str>,
}

impl ConfigSubtreeBindingProfile {
    pub fn resolve_config_path(&self, context: &ResolutionContext) -> PathBuf {
        (self.config_path_resolver)(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandFileRenderFormat {
    FrontmatterMarkdown,
    CursorPlaintext,
}

impl CommandFileRenderFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FrontmatterMarkdown => "frontmatter_markdown",
            Self::CursorPlaintext => "cursor_plaintext",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "frontmatter_markdown" => Some(Self::FrontmatterMarkdown),
            "cursor_plaintext" => Some(Self::CursorPlaintext),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandFileBindingProfile {
    pub root_path_resolver: PathFn,
    pub output_dir_resolver: PathFn,
    pub invocation_prefix: &'static str,
    pub render_format: CommandFileRenderFormat,
    pub docs_url: &'static str,
    pub file_glob: &'static str,
    pub supports_frontmatter: bool,
    pub support_note: Option<&'static str>,
}

impl CommandFileBindingProfile {
    pub fn resolve_root_path(&self, context: &ResolutionContext) -> PathBuf {
        (self.root_path_resolver)(context)
    }

    pub fn resolve_output_dir(&self, context: &ResolutionContext) -> PathBuf {
        (self.output_dir_resolver)(context)
    }
}

#[derive(Debug, Clone)]
pub enum BindingProfile {
    FileTree(FileTreeBindingProfile),
    ConfigSubtree(ConfigSubtreeBindingProfile),
    CommandFile(CommandFileBindingProfile),
}

#[derive(Debug, Clone)]
pub struct HarnessDefinition {
    pub harness: &'static str,
    pub label: &'static str,
    pub logo_key: Option<&'static str>,
    pub install_probe: &'static str,
    pub bindings: &'static [(FamilyKey, BindingProfile)],
}

impl HarnessDefinition {
    pub fn supports_family(&self, family: FamilyKey) -> bool {
        self.bindings.iter().any(|(key, _)| *key == family)
    }

    pub fn binding_for(&self, family: FamilyKey) -> Option<&BindingProfile> {
        self.bindings
            .iter()
            .find(|(key, _)| *key == family)
            .map(|(_, profile)| profile)
    }
}

#[derive(Debug, Clone)]
pub struct HarnessStatus {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub installed: bool,
    pub managed_location: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct FamilyBinding<'a> {
    pub definition: &'a HarnessDefinition,
    pub profile: &'a BindingProfile,
}
