import * as Popover from "@radix-ui/react-popover";
import { Check, ChevronDown, Palette } from "lucide-react";

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

function ThemeSwatch({ theme }: { theme: ThemeDefinition }) {
  if (theme.palette && theme.palette.length > 1) {
    return (
      <span className="theme-swatch theme-swatch--palette" aria-hidden="true">
        {theme.palette.map((color) => (
          <span key={color} style={{ background: color }} />
        ))}
      </span>
    );
  }

  return (
    <span
      className="theme-swatch"
      style={{
        background: theme.tokens["--color-bg"],
        borderColor: theme.tokens["--color-border-strong"],
      }}
      aria-hidden="true"
    />
  );
}

function ThemeMenuItem({
  theme,
  selected,
  label,
  onSelect,
}: {
  theme: ThemeDefinition;
  selected: boolean;
  label: string;
  onSelect: () => void;
}) {
  return (
    <li>
      <Popover.Close asChild>
        <button
          type="button"
          className="ui-menu__item"
          data-selected={selected || undefined}
          role="menuitemradio"
          aria-checked={selected}
          onClick={onSelect}
        >
          <span className="ui-menu__icon" aria-hidden="true">
            {selected ? <Check size={14} /> : <ThemeSwatch theme={theme} />}
          </span>
          <span className="ui-menu__label">{label}</span>
        </button>
      </Popover.Close>
    </li>
  );
}

export function ThemeSelector({ triggerClassName = "", showChevron = false }: ThemeSelectorProps) {
  const { theme, setTheme } = useTheme();
  const { locale } = useLocale();
  const isZh = locale === "zh-CN";

  const currentTheme = THEMES.find((t) => t.id === theme) ?? THEMES[0];
  const label = isZh ? currentTheme.labelZh : currentTheme.label;

  const renderSection = (sectionLabel: string, items: ThemeDefinition[]) => (
    <>
      <li className="ui-menu__section-label" aria-hidden="true">
        {sectionLabel}
      </li>
      {items.map((item) => {
        const selected = item.id === theme;
        const itemLabel = isZh ? item.labelZh : item.label;
        return (
          <ThemeMenuItem
            key={item.id}
            theme={item}
            selected={selected}
            label={itemLabel}
            onSelect={() => setTheme(item.id)}
          />
        );
      })}
    </>
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
          className="ui-popup ui-popup--menu ui-menu theme-selector-menu"
          side="right"
          align="end"
          sideOffset={8}
        >
          <ul className="ui-menu__list" role="menu" aria-label="Theme">
            {renderSection(isZh ? "默认" : "Default", DEFAULT_THEMES)}
            {renderSection(isZh ? "ColorHunt" : "ColorHunt", COLORHUNT_THEME_OPTIONS)}
          </ul>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
