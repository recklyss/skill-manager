//! MCP read-model service: owns the adapter registry and produces the cached
//! snapshot consumed by the query/mutation layers.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::harness::{FamilyKey, HarnessKernelService, SUPPORTED_HARNESS_DEFINITIONS};
use crate::mcp::adapters::{build_mcp_adapters, McpAdapter};
use crate::mcp::contracts::{McpHarnessScan, McpReadModelSnapshot};
use crate::mcp::store::McpServerStore;

#[derive(Clone)]
pub struct McpReadModelService {
    store: McpServerStore,
    adapters: Vec<Arc<dyn McpAdapter>>,
    kernel: HarnessKernelService,
    cache: Arc<Mutex<Option<CachedSnapshot>>>,
    snapshot_ttl: Duration,
}

#[derive(Clone)]
struct CachedSnapshot {
    snapshot: McpReadModelSnapshot,
    captured_at: Instant,
}

impl McpReadModelService {
    pub fn new(store: McpServerStore, kernel: HarnessKernelService) -> Self {
        let adapters = build_mcp_adapters(SUPPORTED_HARNESS_DEFINITIONS, &kernel.context);
        Self {
            store,
            adapters,
            kernel,
            cache: Arc::new(Mutex::new(None)),
            snapshot_ttl: Duration::from_secs_f64(1.0),
        }
    }

    pub fn store(&self) -> &McpServerStore {
        &self.store
    }

    pub fn find_adapter(&self, harness: &str) -> Option<&dyn McpAdapter> {
        self.adapters
            .iter()
            .find(|a| a.harness() == harness)
            .map(|a| a.as_ref())
    }

    pub fn enabled_harnesses(&self) -> Vec<String> {
        self.kernel
            .enabled_harness_ids_for_family(FamilyKey::Mcp)
    }

    pub fn enabled_adapters(&self) -> Vec<&dyn McpAdapter> {
        let enabled: std::collections::HashSet<_> = self.enabled_harnesses().into_iter().collect();
        self.adapters
            .iter()
            .filter(|a| enabled.contains(a.harness()))
            .map(|a| a.as_ref())
            .collect()
    }

    pub fn enabled_addressable_adapters(&self) -> Vec<&dyn McpAdapter> {
        self.enabled_adapters()
            .into_iter()
            .filter(|adapter| {
                let (installed, config_present, _, _) = adapter.status();
                installed || config_present
            })
            .collect()
    }

    pub fn enabled_writable_adapters(&self) -> Vec<&dyn McpAdapter> {
        self.enabled_adapters()
            .into_iter()
            .filter(|adapter| {
                let (installed, config_present, writable, _) = adapter.status();
                writable && (installed || config_present)
            })
            .collect()
    }

    pub fn require_enabled_adapter(&self, harness: &str) -> Result<&dyn McpAdapter, String> {
        let adapter = self
            .find_adapter(harness)
            .ok_or_else(|| format!("unknown harness: {harness}"))?;
        if !self.enabled_harnesses().iter().any(|h| h == harness) {
            return Err(format!("harness support is disabled: {harness}"));
        }
        let (installed, config_present, _, _) = adapter.status();
        if !installed && !config_present {
            return Err(format!(
                "{} is not installed and has no MCP config file",
                adapter.label()
            ));
        }
        Ok(adapter)
    }

    pub fn snapshot(&self) -> McpReadModelSnapshot {
        if let Ok(guard) = self.cache.lock() {
            if let Some(cached) = guard.as_ref() {
                if cached.captured_at.elapsed() < self.snapshot_ttl {
                    return cached.snapshot.clone();
                }
            }
        }
        let specs = self.store.list_records();
        let scans = self
            .adapters
            .iter()
            .map(|adapter| adapter.scan(&specs))
            .collect();
        let snapshot = McpReadModelSnapshot { harness_scans: scans };
        if let Ok(mut guard) = self.cache.lock() {
            *guard = Some(CachedSnapshot {
                snapshot: snapshot.clone(),
                captured_at: Instant::now(),
            });
        }
        snapshot
    }

    pub fn visible_scans(&self, snapshot: &McpReadModelSnapshot) -> Vec<McpHarnessScan> {
        let visible: std::collections::HashSet<_> = self.enabled_harnesses().into_iter().collect();
        snapshot
            .harness_scans
            .iter()
            .filter(|scan| visible.contains(&scan.harness))
            .cloned()
            .collect()
    }

    pub fn invalidate(&self) {
        if let Ok(mut guard) = self.cache.lock() {
            *guard = None;
        }
    }
}
