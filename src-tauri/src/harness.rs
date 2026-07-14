use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct HarnessStatus {
    pub harness: String,
    pub label: String,
    pub logo_key: Option<String>,
    pub installed: bool,
    pub managed_location: Option<PathBuf>,
}

#[derive(Clone)]
pub struct HarnessKernel {
    definitions: Vec<HarnessDefinition>,
}

#[derive(Clone)]
struct HarnessDefinition {
    harness: String,
    label: String,
    logo_key: Option<String>,
    install_probe: &'static str,
    skills_root: Option<PathBuf>,
}

impl HarnessKernel {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let xdg_config = dirs::config_dir().unwrap_or_else(|| home.join(".config"));

        Self {
            definitions: vec![
                HarnessDefinition {
                    harness: "codex".into(),
                    label: "Codex".into(),
                    logo_key: Some("codex".into()),
                    install_probe: "codex",
                    skills_root: Some(home.join(".agents").join("skills")),
                },
                HarnessDefinition {
                    harness: "claude".into(),
                    label: "Claude".into(),
                    logo_key: Some("claude".into()),
                    install_probe: "claude",
                    skills_root: Some(home.join(".claude").join("skills")),
                },
                HarnessDefinition {
                    harness: "cursor".into(),
                    label: "Cursor".into(),
                    logo_key: Some("cursor".into()),
                    install_probe: "cursor-agent",
                    skills_root: Some(home.join(".cursor").join("skills")),
                },
                HarnessDefinition {
                    harness: "opencode".into(),
                    label: "OpenCode".into(),
                    logo_key: Some("opencode".into()),
                    install_probe: "opencode",
                    skills_root: Some(xdg_config.join("opencode").join("skills")),
                },
                HarnessDefinition {
                    harness: "hermes".into(),
                    label: "Hermes".into(),
                    logo_key: Some("hermes".into()),
                    install_probe: "hermes",
                    skills_root: Some(
                        std::env::var("HERMES_HOME")
                            .map(PathBuf::from)
                            .unwrap_or_else(|_| home.join(".hermes"))
                            .join("skills"),
                    ),
                },
                HarnessDefinition {
                    harness: "openclaw".into(),
                    label: "OpenClaw".into(),
                    logo_key: Some("openclaw".into()),
                    install_probe: "openclaw",
                    skills_root: Some(home.join(".openclaw").join("skills")),
                },
            ],
        }
    }

    pub fn statuses(&self) -> Vec<HarnessStatus> {
        self.definitions
            .iter()
            .map(|def| {
                let installed = which::which(def.install_probe).is_ok();
                HarnessStatus {
                    harness: def.harness.clone(),
                    label: def.label.clone(),
                    logo_key: def.logo_key.clone(),
                    installed,
                    managed_location: def.skills_root.clone(),
                }
            })
            .collect()
    }
}
