import { type KeyboardEvent, type MouseEvent, useState } from "react";
import { ArrowUpRight, Plus, RotateCcw, Star } from "lucide-react";

import { LoadingSpinner } from "../../../components/LoadingSpinner";
import type { MarketplaceItemDto } from "../api/types";
import { useMarketplaceCopy } from "../i18n";
import { formatMarketplaceInstalls, formatMarketplaceStars } from "../model/formatters";

interface MarketplaceCardProps {
  item: MarketplaceItemDto;
  selected: boolean;
  installing: boolean;
  onOpenDetail: () => void;
  onInstall: () => void;
  onOpenInstalledSkill: (skillRef: string) => void;
}

function avatarFallbackLabel(item: MarketplaceItemDto): string {
  const owner = item.repoLabel.split("/", 1)[0]?.slice(0, 2);
  if (owner && owner.trim()) {
    return owner.toUpperCase();
  }
  return item.name.slice(0, 2).toUpperCase();
}

function isInstalled(item: MarketplaceItemDto): boolean {
  return item.installation.status === "installed" && Boolean(item.installation.installedSkillRef);
}

export function MarketplaceCard({
  item,
  installing,
  onOpenDetail,
  onInstall,
  onOpenInstalledSkill,
}: MarketplaceCardProps) {
  const copy = useMarketplaceCopy();
  const [avatarFailed, setAvatarFailed] = useState(false);
  const avatarSrc = item.repoImageUrl && !avatarFailed ? item.repoImageUrl : null;
  const stars = item.stars ?? 0;
  const installs = formatMarketplaceInstalls(item.installs);
  const installed = isInstalled(item);

  function handleKeyDown(event: KeyboardEvent<HTMLElement>): void {
    if (event.key !== "Enter" && event.key !== " ") {
      return;
    }
    event.preventDefault();
    onOpenDetail();
  }

  function handleOpenInstalled(event: MouseEvent<HTMLButtonElement>): void {
    event.stopPropagation();
    if (!item.installation.installedSkillRef) {
      return;
    }
    onOpenInstalledSkill(item.installation.installedSkillRef);
  }

  return (
    <article
      className="market-card"
      role="button"
      tabIndex={0}
      onClick={onOpenDetail}
      onKeyDown={handleKeyDown}
      aria-label={copy.detail.cards.openSkillMarketplaceDetail(item.name)}
    >
      <div className="market-card__head">
        <div className="market-card__avatar">
          {avatarSrc ? (
            <img
              src={avatarSrc}
              alt={copy.detail.cards.avatarFor(item.repoLabel)}
              onError={() => setAvatarFailed(true)}
            />
          ) : (
            avatarFallbackLabel(item)
          )}
        </div>
        <div className="market-card__identity">
          <div className="market-card__title-row">
            <h4 className="market-card__title" title={item.name}>
              {item.name}
            </h4>
            {installed ? (
              <span className="chip chip--installed" aria-label={copy.detail.skill.installedAria(item.name)}>
                {copy.detail.skill.installed}
              </span>
            ) : null}
          </div>
          <div className="market-card__meta">
            <p className="market-card__repo" title={item.repoLabel}>
              {item.repoLabel}
            </p>
            {stars > 0 ? (
              <span className="market-card__stars">
                <Star size={11} fill="currentColor" />
                {formatMarketplaceStars(stars)}
              </span>
            ) : null}
          </div>
        </div>
      </div>

      <p className="market-card__body" title={item.description || undefined}>
        {item.description || copy.detail.cards.noSkillSummary}
      </p>

      <div className="market-card__footer">
        <span className="market-card__installs">{copy.detail.skill.installs(installs)}</span>
        <div className="marketplace-install-actions">
          {installed ? (
            <>
              <button
                type="button"
                className="action-pill"
                onClick={(event) => {
                  event.stopPropagation();
                  onInstall();
                }}
                aria-label={copy.detail.skill.reinstallAria(item.name)}
                data-pending={installing || undefined}
              >
                {installing ? (
                  <LoadingSpinner size="sm" label={copy.detail.skill.installing(item.name)} />
                ) : (
                  <RotateCcw size={12} aria-hidden="true" />
                )}
                {copy.detail.skill.reinstall}
              </button>
              <button
                type="button"
                className="action-pill action-pill--ghost"
                onClick={handleOpenInstalled}
                aria-label={copy.detail.skill.openInSkillsAria(item.name)}
              >
                <ArrowUpRight size={12} aria-hidden="true" />
                {copy.detail.skill.openInSkills}
              </button>
            </>
          ) : (
            <button
              type="button"
              className="action-pill"
              onClick={(event) => {
                event.stopPropagation();
                onInstall();
              }}
              aria-label={copy.detail.skill.installAria(item.name)}
              data-pending={installing || undefined}
            >
              {installing ? (
                <LoadingSpinner size="sm" label={copy.detail.skill.installing(item.name)} />
              ) : (
                <Plus size={12} aria-hidden="true" />
              )}
              {copy.detail.skill.install}
            </button>
          )}
        </div>
      </div>
    </article>
  );
}
