from __future__ import annotations

from dataclasses import replace
from typing import Iterable

from skill_manager.errors import MutationError

from .enrichment import McpEnrichmentService
from .install_resolver import resolve_registry_server_spec
from .marketplace.catalog import McpMarketplaceCatalog
from .planner import McpAdoptionPlanner
from .read_models import McpReadModelService
from .redaction import redacted_spec_dict
from .store import McpServerSpec, McpServerStore, McpSource


class McpMutationService:
    """Mutations for observed MCP configs.

    The managed manifest stores the canonical observed config. Harness files are
    projections of that canonical spec.
    """

    def __init__(
        self,
        *,
        store: McpServerStore,
        read_models: McpReadModelService,
        planner: McpAdoptionPlanner,
        marketplace_catalog: McpMarketplaceCatalog,
        enrichment: McpEnrichmentService | None = None,
    ) -> None:
        self.store = store
        self.read_models = read_models
        self.planner = planner
        self.marketplace = marketplace_catalog
        self.enrichment = enrichment

    # Install / uninstall ---------------------------------------------------

    def install_from_marketplace(
        self,
        qualified_name: str,
        *,
        source_harness: str,
        config: dict[str, object] | None = None,
    ) -> dict[str, object]:
        if not qualified_name:
            raise MutationError("qualifiedName is required", status=400)
        if not source_harness:
            raise MutationError("sourceHarness is required", status=400)
        self._require_install_target(source_harness)

        existing = self._managed_for_marketplace(qualified_name)
        if existing is not None:
            raise MutationError(
                f"a server named '{existing.name}' is already installed",
                status=409,
            )
        detail = self._marketplace_install_detail(qualified_name)
        if detail is None:
            raise MutationError(f"server not found in marketplace: {qualified_name}", status=404)
        source_spec = resolve_registry_server_spec(detail, config=config)
        if self.store.get_managed(source_spec.name) is not None:
            raise MutationError(
                f"a server named '{source_spec.name}' is already installed",
                status=409,
            )

        stored = self.store.upsert_from_spec(source_spec)
        adapter = self._source_adapter(source_harness)
        try:
            adapter.enable_server(stored)
        except Exception:  # noqa: BLE001
            self.store.remove(stored.name)
            self.read_models.invalidate()
            raise
        self.read_models.invalidate()
        return {"ok": True, "server": redacted_spec_dict(stored)}

    def install_targets(self) -> dict[str, object]:
        return {"targets": self._resolved_install_targets()}

    def uninstall_server(self, name: str) -> dict[str, object]:
        if self.store.get_managed(name) is None:
            raise MutationError(f"unknown server: {name}", status=404)
        bound_harnesses = self._harnesses_in_states(name, {"managed", "drifted"})
        succeeded: list[str] = []
        failures: list[dict[str, str]] = []
        for adapter in self.read_models.enabled_adapters():
            if adapter.harness not in bound_harnesses:
                continue
            try:
                adapter.disable_server(name)
                succeeded.append(adapter.harness)
            except Exception as error:  # noqa: BLE001
                failures.append({"harness": adapter.harness, "error": str(error)})
        if not failures:
            self.store.remove(name)
        if succeeded or not failures:
            self.read_models.invalidate()
        return {
            "ok": not failures,
            "succeeded": succeeded,
            "failed": failures,
        }

    # Per-harness toggle ----------------------------------------------------

    def enable_server(self, name: str, harness: str) -> dict[str, bool]:
        spec = self._require_server(name)
        adapter = self.read_models.require_enabled_adapter(harness)
        if adapter.has_binding(name):
            return {"ok": True}
        adapter.enable_server(spec)
        self.read_models.invalidate()
        return {"ok": True}

    def disable_server(self, name: str, harness: str) -> dict[str, bool]:
        if self.store.get_managed(name) is None:
            raise MutationError(f"unknown server: {name}", status=404)
        adapter = self.read_models.require_enabled_adapter(harness)
        adapter.disable_server(name)
        self.read_models.invalidate()
        return {"ok": True}

    def set_server_all_harnesses(self, name: str, target: str) -> dict[str, object]:
        if target not in ("enabled", "disabled"):
            raise MutationError("target must be 'enabled' or 'disabled'", status=400)
        spec = self._require_server(name)

        bound_now = self._harnesses_in_states(name, {"managed", "drifted"})

        succeeded: list[str] = []
        failures: list[dict[str, str]] = []
        flipped_any = False

        adapters = (
            self.read_models.enabled_writable_adapters()
            if target == "enabled"
            else self.read_models.enabled_addressable_adapters()
        )
        for adapter in adapters:
            if target == "enabled" and adapter.harness in bound_now:
                continue
            if target == "disabled" and adapter.harness not in bound_now:
                continue
            try:
                if target == "enabled":
                    adapter.enable_server(spec)
                else:
                    adapter.disable_server(name)
            except Exception as error:  # noqa: BLE001
                failures.append({"harness": adapter.harness, "error": str(error)})
                continue
            succeeded.append(adapter.harness)
            flipped_any = True

        if flipped_any:
            self.read_models.invalidate()

        return {
            "ok": not failures,
            "succeeded": succeeded,
            "failed": failures,
        }

    # Reconciliation -------------------------------------------------------

    def reconcile_server(
        self,
        name: str,
        *,
        source_kind: str,
        source_harness: str | None = None,
        harnesses: list[str] | None = None,
    ) -> dict[str, object]:
        if self.store.get_managed(name) is None:
            raise MutationError(f"unknown server: {name}", status=404)
        target_harnesses = (
            set(harnesses)
            if harnesses is not None
            else self._harnesses_in_states(name, {"managed", "drifted"}, addressable_only=True)
        )
        current = self._require_server(name)
        if source_kind == "managed":
            source_spec = current
        elif source_kind == "harness":
            if not source_harness:
                raise MutationError("sourceHarness is required when sourceKind is 'harness'", status=400)
            observed_spec = self._observed_spec(name, source_harness)
            source_spec = replace(
                observed_spec,
                name=current.name,
                display_name=current.display_name,
                source=current.source,
            )
            self.store.upsert_from_spec(source_spec)
            self.read_models.invalidate()
            source_spec = self._require_server(name)
        else:
            raise MutationError("sourceKind must be 'managed' or 'harness'", status=400)

        stored = self.store.get_public_spec(name) or source_spec
        binding_spec = self.store.get_binding_spec(name) or source_spec
        succeeded, failures = self._write_spec_to_harnesses(binding_spec, target_harnesses)
        if succeeded:
            self.read_models.invalidate()
        return {
            "ok": not failures,
            "server": redacted_spec_dict(stored),
            "succeeded": succeeded,
            "failed": failures,
        }

    # Adoption -------------------------------------------------------------

    def _apply_enrichment(self, spec: McpServerSpec) -> McpServerSpec:
        if self.enrichment is None:
            return spec
        link = self.enrichment.lookup(spec.name)
        if link is None:
            return spec
        return replace(
            spec,
            display_name=link.display_name or spec.display_name,
            source=McpSource.marketplace(link.qualified_name),
        )

    def adopt(
        self,
        name: str,
        *,
        source_harness: str | None = None,
        harnesses: list[str] | None = None,
    ) -> dict[str, object]:
        if self.store.get_managed(name) is not None:
            raise MutationError(
                f"a managed server named '{name}' already exists", status=409
            )
        group = self.planner.require_group(name)
        if source_harness:
            target_spec = next(
                (sighting.spec for sighting in group.sightings if sighting.harness == source_harness),
                None,
            )
            if target_spec is None:
                raise MutationError(
                    f"server '{name}' was not observed in harness '{source_harness}'",
                    status=400,
                )
        else:
            target_spec = group.canonical_spec
        if target_spec is None:
            raise MutationError(
                f"server '{name}' has different configs across harnesses; choose a sourceHarness to adopt",
                status=409,
            )
        if target_spec.name != name:
            target_spec = replace(target_spec, name=name)
        target_spec = self._apply_enrichment(target_spec)

        target_harnesses = set(harnesses) if harnesses else {s.harness for s in group.sightings}
        stored = self.store.upsert_from_spec(target_spec)
        stored_binding_spec = self.store.get_binding_spec(stored.name)
        if stored_binding_spec is None:
            raise MutationError(f"unknown server: {name}", status=404)

        succeeded, failures = self._write_spec_to_harnesses(
            stored_binding_spec,
            target_harnesses,
        )

        self.read_models.invalidate()
        response_spec = self.store.get_public_spec(stored.name) or stored_binding_spec
        return {
            "ok": not failures,
            "server": redacted_spec_dict(response_spec),
            "succeeded": succeeded,
            "failed": failures,
        }

    # Internal helpers -----------------------------------------------------

    def _resolved_install_targets(self) -> list[dict[str, object]]:
        enabled = set(self.read_models.enabled_harnesses())
        targets: list[dict[str, object]] = []
        for status in self.read_models.harness_statuses():
            supported = True
            reason = None
            if supported and status.harness not in enabled:
                supported = False
                reason = "Harness support is disabled"
            elif supported and not status.mcp_writable:
                supported = False
                reason = status.mcp_unavailable_reason or "MCP config is not writable for this harness"
            elif supported and not (status.installed or status.config_present):
                supported = False
                reason = f"{status.label} is not installed and has no MCP config file"
            targets.append(
                {
                    "harness": status.harness,
                    "label": status.label,
                    "logoKey": status.logo_key,
                    # Legacy frontend wire field; install now writes directly to the selected harness.
                    "smitheryClient": status.harness if supported else None,
                    "supported": supported,
                    "reason": reason,
                }
            )
        return targets

    def _require_install_target(self, harness: str) -> None:
        for target in self._resolved_install_targets():
            if target["harness"] != harness:
                continue
            if target["supported"]:
                return
            reason = target.get("reason")
            raise MutationError(str(reason or f"source harness is not installable: {harness}"), status=400)
        raise MutationError(f"unknown MCP source harness: {harness}", status=400)

    def _marketplace_install_detail(self, qualified_name: str):
        install_detail = getattr(self.marketplace, "install_detail", None)
        if callable(install_detail):
            detail = install_detail(qualified_name)
            if detail is not None:
                to_resolver_detail = getattr(detail, "to_resolver_detail", None)
                return to_resolver_detail() if callable(to_resolver_detail) else detail
        return self.marketplace.detail(qualified_name)

    def _source_adapter(self, harness: str):
        if harness not in self.read_models.enabled_harnesses():
            raise MutationError(f"harness support is disabled: {harness}", status=400)
        adapter = self.read_models.find_adapter(harness)
        if adapter is None:
            raise MutationError(f"unknown MCP source harness: {harness}", status=400)
        return adapter

    def _harnesses_in_states(
        self,
        name: str,
        states: Iterable[str],
        *,
        addressable_only: bool = False,
    ) -> set[str]:
        allowed_states = set(states)
        addressable = (
            {adapter.harness for adapter in self.read_models.enabled_addressable_adapters()}
            if addressable_only
            else set(self.read_models.enabled_harnesses())
        )
        snapshot = self.read_models.snapshot()
        result: set[str] = set()
        for scan in snapshot.harness_scans:
            if scan.harness not in addressable:
                continue
            for entry in scan.entries:
                if entry.name == name and entry.state in allowed_states:
                    result.add(scan.harness)
        return result

    def _observed_spec(self, name: str, harness: str) -> McpServerSpec:
        snapshot = self.read_models.snapshot()
        for scan in snapshot.harness_scans:
            if scan.harness != harness:
                continue
            for entry in scan.entries:
                if entry.name != name:
                    continue
                if entry.parsed_spec is None:
                    raise MutationError(
                        entry.parse_issue or f"unable to parse '{name}' in {harness}",
                        status=409,
                    )
                return entry.parsed_spec
        raise MutationError(f"server '{name}' was not observed in harness '{harness}'", status=404)

    def _write_spec_to_harnesses(
        self,
        spec: McpServerSpec,
        harnesses: Iterable[str],
    ) -> tuple[list[str], list[dict[str, str]]]:
        targets = set(harnesses)
        succeeded: list[str] = []
        failures: list[dict[str, str]] = []
        for adapter in self.read_models.enabled_adapters():
            if adapter.harness not in targets:
                continue
            try:
                adapter.enable_server(spec)
            except Exception as error:  # noqa: BLE001
                failures.append({"harness": adapter.harness, "error": str(error)})
                continue
            succeeded.append(adapter.harness)
        return succeeded, failures

    def _require_server(self, name: str) -> McpServerSpec:
        spec = self.store.get_binding_spec(name)
        if spec is None:
            raise MutationError(f"unknown server: {name}", status=404)
        return spec

    def _managed_for_marketplace(self, qualified_name: str) -> McpServerSpec | None:
        for server in self.store.list_managed():
            if server.source.kind == "marketplace" and server.source.locator == qualified_name:
                return server
        return None


__all__ = ["McpMutationService"]
