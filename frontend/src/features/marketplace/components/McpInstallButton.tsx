import { type MouseEvent } from "react";
import { Link } from "react-router-dom";
import { ArrowUpRight, Loader2, Plus, RotateCcw } from "lucide-react";

import { UiTooltip } from "../../../components/ui/UiTooltip";
import { useMarketplaceCopy } from "../i18n";
import type { InstalledState } from "../model/installed-lookup";

interface McpInstallButtonProps {
  displayName: string;
  installedState: InstalledState;
  installing: boolean;
  onInstall: () => void;
}

/**
 * Install affordance for a marketplace MCP server.
 *
 *   installing  → disabled pill with spinner + "Installing"
 *   installed   → Re-install pill + optional Open in MCPs link
 *   default     → Install pill that triggers onInstall
 */
export function McpInstallButton({
  displayName,
  installedState,
  installing,
  onInstall,
}: McpInstallButtonProps) {
  const copy = useMarketplaceCopy();

  if (installing) {
    return (
      <button
        type="button"
        className="action-pill"
        disabled
        aria-label={copy.detail.installButton.installingAria(displayName)}
        onClick={stopPropagation}
      >
        <Loader2 size={12} className="mcp-dialog__spinner" aria-hidden="true" />
        {copy.detail.installButton.installing}
      </button>
    );
  }

  if (installedState.kind === "installed") {
    return (
      <div className="marketplace-install-actions">
        <button
          type="button"
          className="action-pill"
          onClick={(event) => {
            stopPropagation(event);
            onInstall();
          }}
          aria-label={copy.detail.installButton.reinstallToMcpAria(displayName)}
        >
          <RotateCcw size={12} aria-hidden="true" />
          {copy.detail.installButton.reinstallToMcp}
        </button>
        <UiTooltip content={copy.detail.installButton.openInMcpTooltip}>
          <Link
            to={`/mcp/use?server=${encodeURIComponent(installedState.managedName)}`}
            className="action-pill action-pill--ghost"
            style={{ textDecoration: "none" }}
            aria-label={copy.detail.installButton.openInMcpAria(displayName)}
            onClick={stopPropagation}
          >
            <ArrowUpRight size={12} aria-hidden="true" />
            {copy.detail.installButton.openInMcp}
          </Link>
        </UiTooltip>
      </div>
    );
  }

  return (
    <button
      type="button"
      className="action-pill"
      onClick={(event) => {
        stopPropagation(event);
        onInstall();
      }}
      aria-label={copy.detail.installButton.addToMcpAria(displayName)}
    >
      <Plus size={12} aria-hidden="true" />
      {copy.detail.installButton.addToMcp}
    </button>
  );
}

function stopPropagation(event: MouseEvent): void {
  event.stopPropagation();
}
