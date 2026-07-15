import { fireEvent, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { renderWithAppProviders } from "../../../test/render";
import { createMarketplaceItem } from "../test-fixtures";
import { MarketplaceCard } from "./MarketplaceCard";

function renderCard(overrides: Parameters<typeof createMarketplaceItem>[0] = {}) {
  const item = createMarketplaceItem(overrides);
  const onOpenDetail = vi.fn();
  const onInstall = vi.fn();
  const onOpenInstalledSkill = vi.fn();

  const utils = renderWithAppProviders(
    <MarketplaceCard
      item={item}
      selected={false}
      installing={false}
      onOpenDetail={onOpenDetail}
      onInstall={onInstall}
      onOpenInstalledSkill={onOpenInstalledSkill}
    />,
  );

  return { ...utils, item, onOpenDetail, onInstall, onOpenInstalledSkill };
}

describe("MarketplaceCard", () => {
  it("keeps full long text available while using marketplace card text slots", () => {
    const longName =
      "Very Long Skill Name That Should Truncate Without Pushing The Installed Badge Outside The Card";
    const longRepo = "very-long-org/very-long-repository-name-that-should-ellipsize";
    const longDescription =
      "This skill description is intentionally long so the card should clamp it instead of stretching the marketplace grid layout.";
    const { container } = renderCard({
      name: longName,
      repoLabel: longRepo,
      description: longDescription,
      installation: { status: "installed", installedSkillRef: "shared/long-skill" },
    });

    expect(container.querySelector(".market-card__title")).toHaveAttribute("title", longName);
    expect(container.querySelector(".market-card__repo")).toHaveAttribute("title", longRepo);
    expect(container.querySelector(".market-card__body")).toHaveAttribute("title", longDescription);
    expect(screen.getByText("Installed")).toBeInTheDocument();
    expect(container.querySelector(".market-card__title-row")).toContainElement(
      screen.getByText("Installed"),
    );
  });

  it("renders install actions without opening detail when clicked", () => {
    const { onOpenDetail, onInstall } = renderCard();

    fireEvent.click(screen.getByRole("button", { name: /install mode switch/i }));
    expect(onInstall).toHaveBeenCalledTimes(1);
    expect(onOpenDetail).not.toHaveBeenCalled();
  });

  it("renders reinstall and open actions for installed skills", () => {
    const { onOpenInstalledSkill } = renderCard({
      installation: { status: "installed", installedSkillRef: "shared/mode-switch" },
    });

    expect(screen.getByText("Installed")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /re-install mode switch/i })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /open mode switch in skills/i }));
    expect(onOpenInstalledSkill).toHaveBeenCalledWith("shared/mode-switch");
  });
});
