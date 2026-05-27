import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { UiTooltipProvider } from "../../../components/ui/UiTooltipProvider";
import type { McpMarketplaceItemDto } from "../api/mcp-types";
import { InstallingProvider } from "../model/installing-context";
import { McpMarketplaceCard } from "./McpMarketplaceCard";

const fetchMock = vi.fn();

function okJson(payload: object) {
  return {
    ok: true,
    status: 200,
    statusText: "OK",
    json: async () => payload,
  };
}

function createItem(overrides: Partial<McpMarketplaceItemDto> = {}): McpMarketplaceItemDto {
  return {
    qualifiedName: overrides.qualifiedName ?? "@exa/exa-mcp",
    namespace: overrides.namespace ?? "exa",
    displayName: overrides.displayName ?? "Exa Search",
    description: overrides.description ?? "Fast, intelligent web search and crawling.",
    iconUrl: overrides.iconUrl ?? null,
    isVerified: overrides.isVerified ?? true,
    isRemote: overrides.isRemote ?? true,
    isDeployed: overrides.isDeployed ?? true,
    useCount: overrides.useCount ?? 1200,
    createdAt: overrides.createdAt ?? null,
    homepage: overrides.homepage ?? null,
    externalUrl: overrides.externalUrl ?? "https://smithery.ai/server/exa",
  };
}

function detailPayload(item: McpMarketplaceItemDto, overrides: Record<string, unknown> = {}) {
  return {
    qualifiedName: item.qualifiedName,
    managedName: "exa-mcp",
    displayName: item.displayName,
    description: item.description,
    iconUrl: item.iconUrl,
    isRemote: item.isRemote,
    deploymentUrl: "https://exa.run.tools",
    connections: [],
    tools: [],
    resources: [],
    prompts: [],
    capabilityCounts: { tools: 0, resources: 0, prompts: 0 },
    externalUrl: item.externalUrl,
    installConfig: { required: false, fields: [] },
    ...overrides,
  };
}

function renderCard(
  item: McpMarketplaceItemDto,
  inventoryPayload: object = { columns: [], entries: [] },
  detailOverrides: Record<string, unknown> = {},
) {
  fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = typeof input === "string" ? input : input.toString();
    const method = init?.method ?? "GET";
    if (url.includes("/api/marketplace/mcp/install-targets") && method === "GET") {
      return okJson({
        targets: [
          {
            harness: "cursor",
            label: "Cursor",
            logoKey: "cursor",
            smitheryClient: "cursor",
            supported: true,
            reason: null,
          },
          {
            harness: "claude",
            label: "Claude",
            logoKey: "claude",
            smitheryClient: "claude-code",
            supported: true,
            reason: null,
          },
          {
            harness: "openclaw",
            label: "OpenClaw",
            logoKey: "openclaw",
            smitheryClient: null,
            supported: false,
            reason: "Smithery does not provide an OpenClaw MCP installer target",
          },
        ],
      });
    }
    if (url.includes("/api/mcp/servers") && method === "GET") {
      return okJson(inventoryPayload);
    }
    if (url.includes("/api/marketplace/mcp/items") && method === "GET") {
      return okJson(detailPayload(item, detailOverrides));
    }
    if (url.includes("/api/mcp/servers/exa-mcp/availability/check") && method === "POST") {
      return okJson({
        ok: true,
        name: "exa-mcp",
        availabilityStatus: "available",
        availabilityReason: null,
      });
    }
    if (url.includes("/api/mcp/servers") && method === "POST") {
      return okJson({
        ok: true,
        server: {
          name: "exa-mcp",
          displayName: "Exa Search",
          source: { kind: "marketplace", locator: item.qualifiedName },
          transport: "http",
          url: "https://exa.run.tools",
          installedAt: "2026-04-23T00:00:00Z",
          revision: "abc",
        },
      });
    }
    throw new Error(`Unhandled URL ${url}`);
  });

  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const onOpenDetail = vi.fn();
  const utils = render(
    <QueryClientProvider client={client}>
      <UiTooltipProvider delayDuration={0} skipDelayDuration={0}>
        <MemoryRouter>
          <InstallingProvider>
            <McpMarketplaceCard
              item={item}
              selected={false}
              onOpenDetail={onOpenDetail}
            />
          </InstallingProvider>
        </MemoryRouter>
      </UiTooltipProvider>
    </QueryClientProvider>,
  );
  return { ...utils, onOpenDetail };
}

describe("McpMarketplaceCard", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock);
    vi.stubGlobal(
      "ResizeObserver",
      class ResizeObserver {
        observe() {}
        unobserve() {}
        disconnect() {}
      },
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    fetchMock.mockReset();
  });

  it("renders an install button for remote deployed items", () => {
    renderCard(createItem());
    expect(screen.getByRole("button", { name: /add exa search to mcps/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /add exa search to mcps/i })).toHaveTextContent("Add to MCPs");
    expect(screen.queryByText("Remote")).not.toBeInTheDocument();
    expect(screen.queryByText("Verified")).not.toBeInTheDocument();
    expect(screen.queryByText("1.2k")).not.toBeInTheDocument();
  });

  it("falls back to a single initial when an item has no icon", () => {
    const { container } = renderCard(createItem({ iconUrl: null, displayName: "Exa Search" }));

    expect(container.querySelector(".market-card__avatar")).toHaveTextContent("E");
    expect(container.querySelector(".market-card__avatar")).not.toHaveTextContent("EX");
  });

  it("does not open detail when the install button is clicked", async () => {
    const { onOpenDetail } = renderCard(createItem());
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /add exa search to mcps/i })).toBeEnabled(),
    );
    const button = screen.getByRole("button", { name: /add exa search to mcps/i });
    fireEvent.click(button);
    expect(await screen.findByRole("button", { name: /claude/i })).toHaveTextContent("claude-code");
    expect(screen.queryByRole("button", { name: /openclaw/i })).not.toBeInTheDocument();
    fireEvent.click(await screen.findByRole("button", { name: /cursor/i }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        expect.stringContaining("/api/mcp/servers"),
        expect.objectContaining({ method: "POST" }),
      );
    });
    expect(onOpenDetail).not.toHaveBeenCalled();
  });

  it("checks availability after installing from the marketplace", async () => {
    renderCard(createItem());
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /add exa search to mcps/i })).toBeEnabled(),
    );

    fireEvent.click(screen.getByRole("button", { name: /add exa search to mcps/i }));
    fireEvent.click(await screen.findByRole("button", { name: /cursor/i }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        expect.stringContaining("/api/mcp/servers/exa-mcp/availability/check"),
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  it("renders an install button for local items", async () => {
    renderCard(createItem({ isRemote: false, isDeployed: false }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /add exa search to mcps/i })).toBeEnabled(),
    );
    const button = screen.getByRole("button", { name: /add exa search to mcps/i });

    fireEvent.click(button);
    fireEvent.click(await screen.findByRole("button", { name: /cursor/i }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        expect.stringContaining("/api/mcp/servers"),
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  it("opens a config dialog for required registry install fields and submits config", async () => {
    renderCard(createItem(), { columns: [], entries: [] }, {
      installConfig: {
        required: true,
        fields: [
          {
            name: "CUEAPI_API_KEY",
            label: "CUEAPI_API_KEY",
            description: "CueAPI API key. Generate at https://cueapi.ai or app.i18nagent.ai.",
            format: "string",
            required: true,
            secret: true,
            default: null,
            placeholder: null,
            choices: [],
            target: "env",
          },
        ],
      },
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /add exa search to mcps/i })).toBeEnabled(),
    );

    fireEvent.click(screen.getByRole("button", { name: /add exa search to mcps/i }));
    fireEvent.click(await screen.findByRole("button", { name: /cursor/i }));

    const input = await screen.findByLabelText(/CUEAPI_API_KEY/i, { selector: "input" });
    expect(screen.getByRole("link", { name: "https://cueapi.ai" })).toHaveAttribute(
      "href",
      "https://cueapi.ai",
    );
    expect(screen.getByRole("link", { name: "app.i18nagent.ai" })).toHaveAttribute(
      "href",
      "https://app.i18nagent.ai",
    );
    expect(screen.getByRole("button", { name: /^install$/i })).toBeDisabled();
    fireEvent.change(input, { target: { value: "cue-key" } });
    fireEvent.click(screen.getByRole("button", { name: /^install$/i }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        expect.stringContaining("/api/mcp/servers"),
        expect.objectContaining({
          method: "POST",
          body: expect.stringContaining("CUEAPI_API_KEY"),
        }),
      );
    });
    const postCall = fetchMock.mock.calls.find(
      ([url, init]) => String(url).includes("/api/mcp/servers") && init?.method === "POST",
    );
    expect(JSON.parse(String(postCall?.[1]?.body))).toMatchObject({
      config: { CUEAPI_API_KEY: "cue-key" },
    });
  });

  it("keeps undeployed remote items installable because Smithery writes the source config", async () => {
    renderCard(createItem({ isRemote: true, isDeployed: false }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /add exa search to mcps/i })).toBeEnabled(),
    );
    const button = screen.getByRole("button", { name: /add exa search to mcps/i });
    fireEvent.click(button);
    fireEvent.click(await screen.findByRole("button", { name: /cursor/i }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledWith(
        expect.stringContaining("/api/mcp/servers"),
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  it("renders 'Open in MCPs' when the server is already managed", async () => {
    renderCard(createItem(), {
      columns: [],
      entries: [
        {
          name: "exa-mcp",
          displayName: "Exa Search",
          kind: "managed",
          canEnable: true,
          spec: {
            name: "exa-mcp",
            displayName: "Exa Search",
            source: { kind: "marketplace", locator: "@exa/exa-mcp" },
            transport: "http",
            installedAt: "2026-04-23T00:00:00Z",
            revision: "",
            url: "https://exa.run.tools",
          },
          sightings: [],
        },
      ],
    });

    const link = await screen.findByRole("link", { name: /open exa search in mcps/i });
    expect(link).toHaveAttribute("href", "/mcp/use?server=exa-mcp");
  });
});
