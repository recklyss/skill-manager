import { useCallback, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";

import { fetchMcpMarketplaceDetail } from "../api/mcp-client";
import { mcpMarketplaceKeys, useAddMcpServerMutation, useMcpInstallTargetsQuery } from "../api/mcp-queries";
import type {
  AddMcpServerResponseDto,
  McpInstallConfigDto,
  McpInstallTargetDto,
  McpMarketplaceDetailDto,
  McpMarketplaceItemDto,
} from "../api/mcp-types";
import { type InstalledState, useInstalledServerLookup } from "./installed-lookup";
import { useInstallingState } from "./installing-context";

export type McpInstallAvailability =
  | { kind: "available" }
  | { kind: "unavailable"; reason: string };

export type McpSourceHarness = string;
export type McpInstallConfigValues = Record<string, string | boolean | number>;
export interface PendingMcpInstallConfig {
  qualifiedName: string;
  sourceHarness: McpSourceHarness;
  displayName: string;
  installConfig: McpInstallConfigDto;
}
export type McpInstallTargetState =
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "ready"; targets: McpInstallTargetDto[] };

const INSTALL_TARGET_LOAD_ERROR = "Unable to load source harness installers";

export function summaryInstallAvailability(
  _item: Pick<McpMarketplaceItemDto, "isRemote" | "isDeployed">,
): McpInstallAvailability {
  return { kind: "available" };
}

export function detailInstallAvailability(
  _detail: McpMarketplaceDetailDto,
): McpInstallAvailability {
  return { kind: "available" };
}

interface UseMcpInstallActionStateParams {
  qualifiedName: string;
  displayName: string;
  onInstalled?: (response: AddMcpServerResponseDto) => void;
}

interface McpInstallActionState {
  installedState: InstalledState;
  installTargetState: McpInstallTargetState;
  installing: boolean;
  pendingConfig: PendingMcpInstallConfig | null;
  onInstall: (sourceHarness: McpSourceHarness) => void;
  onCancelConfig: () => void;
  onSubmitConfig: (config: McpInstallConfigValues) => void;
}

export function useMcpInstallActionState({
  qualifiedName,
  displayName,
  onInstalled,
}: UseMcpInstallActionStateParams): McpInstallActionState {
  const { lookup } = useInstalledServerLookup();
  const { isInstalling } = useInstallingState();
  const queryClient = useQueryClient();
  const installMutation = useAddMcpServerMutation();
  const installTargetsQuery = useMcpInstallTargetsQuery();
  const [pendingConfig, setPendingConfig] = useState<PendingMcpInstallConfig | null>(null);

  const submitInstall = useCallback(
    (sourceHarness: McpSourceHarness, config?: McpInstallConfigValues) => {
      installMutation.mutate(
        { qualifiedName, sourceHarness, displayName, config },
        {
          onSuccess: (response) => {
            setPendingConfig(null);
            onInstalled?.(response);
          },
        },
      );
    },
    [displayName, installMutation, onInstalled, qualifiedName],
  );

  const onInstall = useCallback(
    (sourceHarness: McpSourceHarness) => {
      void queryClient
        .fetchQuery({
          queryKey: mcpMarketplaceKeys.detail(qualifiedName),
          queryFn: () => fetchMcpMarketplaceDetail(qualifiedName),
        })
        .then((detail) => {
          const installConfig = detail.installConfig;
          if (installConfig?.fields?.length) {
            setPendingConfig({ qualifiedName, sourceHarness, displayName, installConfig });
            return;
          }
          submitInstall(sourceHarness);
        })
        .catch(() => {
          submitInstall(sourceHarness);
        });
    },
    [displayName, qualifiedName, queryClient, submitInstall],
  );

  const onCancelConfig = useCallback(() => {
    setPendingConfig(null);
  }, []);

  const onSubmitConfig = useCallback(
    (config: McpInstallConfigValues) => {
      if (!pendingConfig) {
        return;
      }
      submitInstall(pendingConfig.sourceHarness, config);
    },
    [pendingConfig, submitInstall],
  );

  return {
    installedState: lookup(qualifiedName),
    installTargetState: resolveInstallTargetState(
      installTargetsQuery.isPending,
      installTargetsQuery.error,
      installTargetsQuery.data?.targets,
    ),
    installing: isInstalling(qualifiedName),
    pendingConfig,
    onInstall,
    onCancelConfig,
    onSubmitConfig,
  };
}

function resolveInstallTargetState(
  isPending: boolean,
  error: unknown,
  targets: McpInstallTargetDto[] | undefined,
): McpInstallTargetState {
  if (isPending) {
    return { kind: "loading" };
  }
  if (error) {
    const message = error instanceof Error ? error.message.trim() : "";
    return {
      kind: "error",
      message: message ? `${INSTALL_TARGET_LOAD_ERROR}: ${message}` : INSTALL_TARGET_LOAD_ERROR,
    };
  }
  return { kind: "ready", targets: targets ?? [] };
}
