import { ExternalLink } from "lucide-react";

import { ExternalAnchor } from "../../../../components/ExternalAnchor";
import { UiTooltip } from "../../../../components/ui/UiTooltip";
import type { McpMarketplaceLinkDto } from "../../api/management-types";
import { useMcpCopy } from "../../i18n";

interface McpMarketplaceLinkChipProps {
  link: McpMarketplaceLinkDto;
}

export function McpMarketplaceLinkChip({ link }: McpMarketplaceLinkChipProps) {
  const copy = useMcpCopy();
  return (
    <UiTooltip content={link.description || link.displayName}>
      <ExternalAnchor
        className="chip chip--verified mcp-marketplace-link-chip"
        href={link.externalUrl}
        rel="noreferrer"
      >
        {link.iconUrl ? (
          <img
            src={link.iconUrl}
            alt=""
            aria-hidden="true"
            className="mcp-marketplace-link-chip__icon"
          />
        ) : null}
        <span>{copy.detail.review.marketplaceMatch}</span>
        <ExternalLink size={12} aria-hidden="true" />
      </ExternalAnchor>
    </UiTooltip>
  );
}
