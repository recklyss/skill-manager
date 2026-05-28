import type {
  McpInventoryDto,
  McpInventoryEntryDto,
} from "../../features/mcp/api/management-types";

export function mcpInventoryPayload(
  entries: McpInventoryEntryDto[] = [],
  overrides: Partial<McpInventoryDto> = {},
): McpInventoryDto {
  return {
    columns: [],
    entries,
    issues: [],
    ...overrides,
  };
}

export function mcpInventoryEntry({
  name,
  kind,
  displayName = name,
  sightings = [],
  canEnable = kind === "managed",
  enabledStatus = "disabled",
  availabilityStatus = "unavailable",
  availabilityReason = null,
  mcpStatus = availabilityStatus === "available"
    ? { kind: "available" as const, reason: null }
    : {
        kind: "connection_issue" as const,
        reason: availabilityReason,
      },
  spec = null,
}: Pick<McpInventoryEntryDto, "name" | "kind"> & Partial<McpInventoryEntryDto>): McpInventoryEntryDto {
  return {
    name,
    displayName,
    kind,
    canEnable,
    enabledStatus,
    availabilityStatus,
    availabilityReason,
    mcpStatus,
    spec,
    sightings,
  };
}
