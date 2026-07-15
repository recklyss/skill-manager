use std::path::PathBuf;

use super::contracts::{
    BindingProfile, CommandFileBindingProfile, CommandFileRenderFormat, ConfigFileFormat,
    ConfigSubtreeBindingProfile, FamilyKey, FileTreeAvailability, FileTreeBindingProfile,
    FileTreeDiscoveryRoot, FileTreeLayout, HarnessDefinition,
};
use super::resolution::{
    agents_skills_root, claude_skills_root, codex_admin_skills_root, codex_legacy_skills_root,
    codex_skills_root, copilot_installed_plugins_root, copilot_skills_root, cursor_skills_root,
    hermes_home, hermes_skills_root,
    opencode_skills_root, openclaw_skills_root, ResolutionContext,
};

fn codex_config(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".codex").join("config.toml")
}

fn codex_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".codex")
}

fn codex_prompts(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".codex").join("prompts")
}

fn claude_config(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".claude.json")
}

fn claude_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".claude")
}

fn claude_commands(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".claude").join("commands")
}

fn cursor_config(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".cursor").join("mcp.json")
}

fn cursor_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".cursor")
}

fn cursor_commands(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".cursor").join("commands")
}

fn opencode_config(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".opencode").join("opencode.jsonc")
}

fn opencode_root(ctx: &ResolutionContext) -> PathBuf {
    ctx.xdg_config_home.join("opencode")
}

fn opencode_commands(ctx: &ResolutionContext) -> PathBuf {
    ctx.xdg_config_home.join("opencode").join("commands")
}

fn openclaw_config(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".openclaw").join("openclaw.json")
}

fn copilot_config(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join(".copilot").join("mcp-config.json")
}

static CODEX_DISCOVERY_ROOTS: &[FileTreeDiscoveryRoot] = &[
    FileTreeDiscoveryRoot {
        kind: "admin-root",
        scope: "admin",
        label: "Admin skills root",
        path_resolver: codex_admin_skills_root,
    },
    FileTreeDiscoveryRoot {
        kind: "legacy-root",
        scope: "legacy",
        label: "Legacy import root",
        path_resolver: codex_legacy_skills_root,
    },
];

static OPENCODE_DISCOVERY_ROOTS: &[FileTreeDiscoveryRoot] = &[
    FileTreeDiscoveryRoot {
        kind: "compat-root",
        scope: "claude-compat",
        label: "Claude compatibility root",
        path_resolver: claude_skills_root,
    },
    FileTreeDiscoveryRoot {
        kind: "compat-root",
        scope: "agents-compat",
        label: "Agents compatibility root",
        path_resolver: agents_skills_root,
    },
];

static OPENCLAW_DISCOVERY_ROOTS: &[FileTreeDiscoveryRoot] = &[
    FileTreeDiscoveryRoot {
        kind: "personal-root",
        scope: "personal-agent",
        label: "Personal agent skills root",
        path_resolver: agents_skills_root,
    },
];

static COPILOT_DISCOVERY_ROOTS: &[FileTreeDiscoveryRoot] = &[
    FileTreeDiscoveryRoot {
        kind: "compat-root",
        scope: "agents-compat",
        label: "Agents compatibility root",
        path_resolver: agents_skills_root,
    },
    FileTreeDiscoveryRoot {
        kind: "plugin-root",
        scope: "installed-plugins",
        label: "Installed plugin skills",
        path_resolver: copilot_installed_plugins_root,
    },
];

static CODEX_BINDINGS: &[(FamilyKey, BindingProfile)] = &[
    (
        FamilyKey::Skills,
        BindingProfile::FileTree(FileTreeBindingProfile {
            managed_env: Some("SKILL_MANAGER_CODEX_ROOT"),
            managed_default: codex_skills_root,
            discovery_roots: CODEX_DISCOVERY_ROOTS,
            availability: FileTreeAvailability::Cli,
            app_probe_paths: &[],
            layout: FileTreeLayout::Flat,
            default_category: None,
        }),
    ),
    (
        FamilyKey::Mcp,
        BindingProfile::ConfigSubtree(ConfigSubtreeBindingProfile {
            config_path_resolver: codex_config,
            file_format: ConfigFileFormat::Toml,
            subtree_path: &["mcp_servers"],
            codec: "codex",
            capability_probe: None,
            capability_unavailable_reason: None,
        }),
    ),
    (
        FamilyKey::SlashCommands,
        BindingProfile::CommandFile(CommandFileBindingProfile {
            root_path_resolver: codex_root,
            output_dir_resolver: codex_prompts,
            invocation_prefix: "/prompts:",
            render_format: CommandFileRenderFormat::FrontmatterMarkdown,
            docs_url: "https://developers.openai.com/codex/custom-prompts",
            file_glob: "*.md",
            supports_frontmatter: true,
            support_note: Some(
                "Codex custom prompts are deprecated in favor of skills, but this prompt directory remains verified for slash-command compatibility.",
            ),
        }),
    ),
];

static CLAUDE_BINDINGS: &[(FamilyKey, BindingProfile)] = &[
    (
        FamilyKey::Skills,
        BindingProfile::FileTree(FileTreeBindingProfile {
            managed_env: Some("SKILL_MANAGER_CLAUDE_ROOT"),
            managed_default: claude_skills_root,
            discovery_roots: &[],
            availability: FileTreeAvailability::Cli,
            app_probe_paths: &[],
            layout: FileTreeLayout::Flat,
            default_category: None,
        }),
    ),
    (
        FamilyKey::Mcp,
        BindingProfile::ConfigSubtree(ConfigSubtreeBindingProfile {
            config_path_resolver: claude_config,
            file_format: ConfigFileFormat::Json,
            subtree_path: &["mcpServers"],
            codec: "claude-code",
            capability_probe: None,
            capability_unavailable_reason: None,
        }),
    ),
    (
        FamilyKey::SlashCommands,
        BindingProfile::CommandFile(CommandFileBindingProfile {
            root_path_resolver: claude_root,
            output_dir_resolver: claude_commands,
            invocation_prefix: "/",
            render_format: CommandFileRenderFormat::FrontmatterMarkdown,
            docs_url: "https://code.claude.com/docs/en/slash-commands",
            file_glob: "*.md",
            supports_frontmatter: true,
            support_note: Some(
                "Claude Code has merged custom commands into skills, while existing .claude/commands files remain supported.",
            ),
        }),
    ),
];

fn cursor_app_probe_0(_ctx: &ResolutionContext) -> PathBuf {
    PathBuf::from("/Applications/Cursor.app")
}

fn cursor_app_probe_1(ctx: &ResolutionContext) -> PathBuf {
    ctx.home.join("Applications").join("Cursor.app")
}

static CURSOR_APP_PROBES: &[fn(&ResolutionContext) -> PathBuf] =
    &[cursor_app_probe_0, cursor_app_probe_1];

static CURSOR_BINDINGS: &[(FamilyKey, BindingProfile)] = &[
    (
        FamilyKey::Skills,
        BindingProfile::FileTree(FileTreeBindingProfile {
            managed_env: Some("SKILL_MANAGER_CURSOR_ROOT"),
            managed_default: cursor_skills_root,
            discovery_roots: &[],
            availability: FileTreeAvailability::CliOrApp,
            app_probe_paths: CURSOR_APP_PROBES,
            layout: FileTreeLayout::Flat,
            default_category: None,
        }),
    ),
    (
        FamilyKey::Mcp,
        BindingProfile::ConfigSubtree(ConfigSubtreeBindingProfile {
            config_path_resolver: cursor_config,
            file_format: ConfigFileFormat::Json,
            subtree_path: &["mcpServers"],
            codec: "cursor",
            capability_probe: None,
            capability_unavailable_reason: None,
        }),
    ),
    (
        FamilyKey::SlashCommands,
        BindingProfile::CommandFile(CommandFileBindingProfile {
            root_path_resolver: cursor_root,
            output_dir_resolver: cursor_commands,
            invocation_prefix: "/",
            render_format: CommandFileRenderFormat::CursorPlaintext,
            docs_url: "https://cursor.com/changelog/1-6",
            file_glob: "*.md",
            supports_frontmatter: false,
            support_note: Some(
                "Cursor slash command support is verified locally; current public docs emphasize skills while older command files remain supported in practice.",
            ),
        }),
    ),
];

static OPENCODE_BINDINGS: &[(FamilyKey, BindingProfile)] = &[
    (
        FamilyKey::Skills,
        BindingProfile::FileTree(FileTreeBindingProfile {
            managed_env: Some("SKILL_MANAGER_OPENCODE_ROOT"),
            managed_default: opencode_skills_root,
            discovery_roots: OPENCODE_DISCOVERY_ROOTS,
            availability: FileTreeAvailability::Cli,
            app_probe_paths: &[],
            layout: FileTreeLayout::Flat,
            default_category: None,
        }),
    ),
    (
        FamilyKey::Mcp,
        BindingProfile::ConfigSubtree(ConfigSubtreeBindingProfile {
            config_path_resolver: opencode_config,
            file_format: ConfigFileFormat::Jsonc,
            subtree_path: &["mcp"],
            codec: "opencode",
            capability_probe: None,
            capability_unavailable_reason: None,
        }),
    ),
    (
        FamilyKey::SlashCommands,
        BindingProfile::CommandFile(CommandFileBindingProfile {
            root_path_resolver: opencode_root,
            output_dir_resolver: opencode_commands,
            invocation_prefix: "/",
            render_format: CommandFileRenderFormat::FrontmatterMarkdown,
            docs_url: "https://opencode.ai/docs/commands/",
            file_glob: "*.md",
            supports_frontmatter: true,
            support_note: None,
        }),
    ),
];

static HERMES_BINDINGS: &[(FamilyKey, BindingProfile)] = &[
    (
        FamilyKey::Skills,
        BindingProfile::FileTree(FileTreeBindingProfile {
            managed_env: Some("SKILL_MANAGER_HERMES_ROOT"),
            managed_default: hermes_skills_root,
            discovery_roots: &[],
            availability: FileTreeAvailability::Cli,
            app_probe_paths: &[],
            layout: FileTreeLayout::Categorized,
            default_category: Some("skill-manager"),
        }),
    ),
    (
        FamilyKey::Mcp,
        BindingProfile::ConfigSubtree(ConfigSubtreeBindingProfile {
            config_path_resolver: |ctx| hermes_home(ctx).join("config.yaml"),
            file_format: ConfigFileFormat::Yaml,
            subtree_path: &["mcp_servers"],
            codec: "hermes",
            capability_probe: None,
            capability_unavailable_reason: None,
        }),
    ),
];

static COPILOT_BINDINGS: &[(FamilyKey, BindingProfile)] = &[
    (
        FamilyKey::Skills,
        BindingProfile::FileTree(FileTreeBindingProfile {
            managed_env: Some("SKILL_MANAGER_COPILOT_ROOT"),
            managed_default: copilot_skills_root,
            discovery_roots: COPILOT_DISCOVERY_ROOTS,
            availability: FileTreeAvailability::Cli,
            app_probe_paths: &[],
            layout: FileTreeLayout::Flat,
            default_category: None,
        }),
    ),
    (
        FamilyKey::Mcp,
        BindingProfile::ConfigSubtree(ConfigSubtreeBindingProfile {
            config_path_resolver: copilot_config,
            file_format: ConfigFileFormat::Json,
            subtree_path: &["mcpServers"],
            codec: "copilot",
            capability_probe: None,
            capability_unavailable_reason: None,
        }),
    ),
];

static OPENCLAW_BINDINGS: &[(FamilyKey, BindingProfile)] = &[
    (
        FamilyKey::Skills,
        BindingProfile::FileTree(FileTreeBindingProfile {
            managed_env: Some("SKILL_MANAGER_OPENCLAW_ROOT"),
            managed_default: openclaw_skills_root,
            discovery_roots: OPENCLAW_DISCOVERY_ROOTS,
            availability: FileTreeAvailability::Cli,
            app_probe_paths: &[],
            layout: FileTreeLayout::Flat,
            default_category: None,
        }),
    ),
    (
        FamilyKey::Mcp,
        BindingProfile::ConfigSubtree(ConfigSubtreeBindingProfile {
            config_path_resolver: openclaw_config,
            file_format: ConfigFileFormat::Json,
            subtree_path: &["mcp", "servers"],
            codec: "openclaw",
            capability_probe: Some("openclaw-mcp-command"),
            capability_unavailable_reason: Some(
                "Installed OpenClaw does not expose MCP config support",
            ),
        }),
    ),
];

pub static SUPPORTED_HARNESS_DEFINITIONS: &[HarnessDefinition] = &[
    HarnessDefinition {
        harness: "codex",
        label: "Codex",
        logo_key: Some("codex"),
        install_probe: "codex",
        bindings: CODEX_BINDINGS,
    },
    HarnessDefinition {
        harness: "claude",
        label: "Claude",
        logo_key: Some("claude"),
        install_probe: "claude",
        bindings: CLAUDE_BINDINGS,
    },
    HarnessDefinition {
        harness: "cursor",
        label: "Cursor",
        logo_key: Some("cursor"),
        install_probe: "cursor-agent",
        bindings: CURSOR_BINDINGS,
    },
    HarnessDefinition {
        harness: "opencode",
        label: "OpenCode",
        logo_key: Some("opencode"),
        install_probe: "opencode",
        bindings: OPENCODE_BINDINGS,
    },
    HarnessDefinition {
        harness: "hermes",
        label: "Hermes",
        logo_key: Some("hermes"),
        install_probe: "hermes",
        bindings: HERMES_BINDINGS,
    },
    HarnessDefinition {
        harness: "openclaw",
        label: "OpenClaw",
        logo_key: Some("openclaw"),
        install_probe: "openclaw",
        bindings: OPENCLAW_BINDINGS,
    },
    HarnessDefinition {
        harness: "copilot",
        label: "GitHub Copilot",
        logo_key: Some("copilot"),
        install_probe: "copilot",
        bindings: COPILOT_BINDINGS,
    },
];

pub fn supported_harness_ids() -> Vec<&'static str> {
    SUPPORTED_HARNESS_DEFINITIONS
        .iter()
        .map(|definition| definition.harness)
        .collect()
}

pub fn harness_definitions_for_family(family: FamilyKey) -> Vec<&'static HarnessDefinition> {
    SUPPORTED_HARNESS_DEFINITIONS
        .iter()
        .filter(|definition| definition.supports_family(family))
        .collect()
}

// ponytail: discovery roots and extra MCP path resolvers omitted; DEV2 can extend catalog as needed.
