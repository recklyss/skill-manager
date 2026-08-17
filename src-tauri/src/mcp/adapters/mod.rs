//! Per-harness MCP adapters.
//!
//! Every harness stores MCP servers differently, so this module is split by
//! concern:
//!
//! - [`McpAdapter`] — the uniform trait the read model consumes.
//! - [`document`] — parse/serialize a config file (JSON/JSONC/TOML/YAML).
//! - [`scan`] — shared per-entry parsing and drift detection.
//! - [`subtree`] — map-subtree layout (`mcpServers`, `mcp_servers`, ...).
//! - [`cordis_patch`] — YAML list-of-rows layout (DeepSeek Harness).
//! - [`read_model`] — the cached snapshot service.
//!
//! Adding a new harness's MCP support means adding a catalog binding that
//! picks one of the two layouts plus a `TransportMapper` codec; no new adapter
//! code is needed unless the harness uses a third storage shape.

mod cordis_patch;
mod document;
mod read_model;
mod scan;
mod subtree;

use std::sync::Arc;

use crate::harness::{BindingProfile, FamilyKey, HarnessDefinition, ResolutionContext};
use crate::mcp::contracts::McpHarnessScan;
use crate::mcp::store::McpServerSpec;

pub use cordis_patch::CordisPatchMcpAdapter;
pub use read_model::McpReadModelService;
pub use subtree::FileBackedMcpAdapter;

pub(crate) use document::{load_document, strip_jsonc, write_document};
pub(crate) use scan::build_harness_scan;

/// Common read/write surface shared by every harness-specific MCP adapter so
/// the read model can treat them uniformly.
pub trait McpAdapter: Send + Sync {
    fn harness(&self) -> &str;
    fn label(&self) -> &str;
    fn status(&self) -> (bool, bool, bool, Option<String>);
    fn scan(&self, specs: &[McpServerSpec]) -> McpHarnessScan;
    fn has_binding(&self, name: &str) -> bool;
    fn enable_server(&self, spec: &McpServerSpec) -> Result<(), String>;
    fn disable_server(&self, name: &str) -> Result<(), String>;
}

/// Build one adapter per catalog harness that declares an MCP binding.
///
/// `ConfigSubtree` bindings become a [`FileBackedMcpAdapter`]; `CordisPatch`
/// bindings become a [`CordisPatchMcpAdapter`].
pub fn build_mcp_adapters(
    definitions: &'static [HarnessDefinition],
    context: &ResolutionContext,
) -> Vec<Arc<dyn McpAdapter>> {
    let mut adapters: Vec<Arc<dyn McpAdapter>> = Vec::new();
    for definition in definitions {
        if let Some(BindingProfile::ConfigSubtree(profile)) = definition.binding_for(FamilyKey::Mcp) {
            adapters.push(Arc::new(FileBackedMcpAdapter::new(
                definition,
                profile.clone(),
                context.clone(),
            )));
        }
        if let Some(BindingProfile::CordisPatch(profile)) = definition.binding_for(FamilyKey::Mcp) {
            adapters.push(Arc::new(CordisPatchMcpAdapter::new(
                definition,
                profile.clone(),
                context.clone(),
            )));
        }
    }
    adapters
}
