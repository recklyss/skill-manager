import * as Popover from "@radix-ui/react-popover";
import { Check, ChevronDown, Palette } from "lucide-react";
import type { CSSProperties } from "react";

import { useTheme } from "../lib/useTheme";
import {
  COLORHUNT_THEME_OPTIONS,
  DEFAULT_THEMES,
  type ThemeDefinition,
  THEMES,
} from "../lib/themes";
import { useLocale } from "../i18n";

interface ThemeSelectorProps {
  /** Additional class for the trigger button. */
  triggerClassName?: string;
  /** If true, shows a chevron arrow in the trigger. */
  showChevron?: boolean;
}

/** Miniature live preview of a theme, rendered entirely from its own tokens. */
function ThemePreview({ theme }: { theme: ThemeDefinition }) {
  const t = theme.tokens;
  return (
    <span
      className="theme-card__preview"
      style={{ background: t["--color-bg"] }}
      aria-hidden="true"
    >
      <span
        className="theme-card__bar"
        style={{
          background: t["--color-surface-raised"],
          borderColor: t["--color-border"],
        }}
      >
        <span className="theme-card__dot" style={{ background: t["--color-accent"] }} />
        <span
          className="theme-card__seg theme-card__seg--sm"
          style={{ background: t["--color-text-muted"] }}
        />
      </span>
      <span className="theme-card__body">
        <span
          className="theme-card__seg theme-card__seg--text"
          style={{ background: t["--color-text"] }}
        />
        <span
          className="theme-card__seg theme-card__seg--muted"
          style={{ background: t["--color-text-muted"] }}
        />
        <span
          className="theme-card__chip"
          style={{ background: t["--color-accent"] }}
        />
      </span>
    </span>
  );
}

function ThemeCard({
  theme,
  selected,
  label,
  index,
  onSelect,
}: {
  theme: ThemeDefinition;
  selected: boolean;
  label: string;
  index: number;
  onSelect: () => void;
}) {
  return (
    <Popover.Close asChild>
      <button
        type="button"
        className="theme-card"
        data-selected={selected || undefined}
        role="menuitemradio"
        aria-checked={selected}
        onClick={onSelect}
        style={{ "--i": index } as CSSProperties}
      >
        <ThemePreview theme={theme} />
        <span className="theme-card__footer">
          <span className="theme-card__label">{label}</span>
          {selected ? (
            <span className="theme-card__check" aria-hidden="true">
              <Check size={12} strokeWidth={3} />
            </span>
          ) : null}
        </span>
      </button>
    </Popover.Close>
  );
}

export function ThemeSelector({ triggerClassName = "", showChevron = false }: ThemeSelectorProps) {
  const { theme, setTheme } = useTheme();
  const { locale } = useLocale();
  const isZh = locale === "zh-CN";

  const currentTheme = THEMES.find((t) => t.id === theme) ?? THEMES[0];
  const label = isZh ? currentTheme.labelZh : currentTheme.label;

  const renderSection = (sectionLabel: string, items: ThemeDefinition[], offset: number) => (
    <div className="theme-gallery__section">
      <p className="theme-gallery__heading" aria-hidden="true">
        {sectionLabel}
      </p>
      <div className="theme-gallery__grid" role="group" aria-label={sectionLabel}>
        {items.map((item, i) => {
          const selected = item.id === theme;
          const itemLabel = isZh ? item.labelZh : item.label;
          return (
            <ThemeCard
              key={item.id}
              theme={item}
              selected={selected}
              label={itemLabel}
              index={offset + i}
              onSelect={() => setTheme(item.id)}
            />
          );
        })}
      </div>
    </div>
  );

  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <button
          type="button"
          className={triggerClassName}
          aria-label={`Theme: ${label}`}
          aria-haspopup="menu"
        >
          <Palette size={16} />
          <span>{label}</span>
          {showChevron ? <ChevronDown className="sidebar-footer-btn__chevron" size={14} aria-hidden="true" /> : null}
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          className="ui-popup theme-gallery"
          side="right"
          align="end"
          sideOffset={8}
          role="menu"
          aria-label="Theme"
        >
          {renderSection(isZh ? "默认" : "Default", DEFAULT_THEMES, 0)}
          {renderSection(
            isZh ? "调色板" : "ColorHunt",
            COLORHUNT_THEME_OPTIONS,
            DEFAULT_THEMES.length,
          )}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
