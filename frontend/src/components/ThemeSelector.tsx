import * as Popover from "@radix-ui/react-popover";
import { Check, ChevronDown, Palette } from "lucide-react";

import { useTheme } from "../lib/useTheme";
import { THEMES } from "../lib/themes";
import { useLocale } from "../i18n";

interface ThemeSelectorProps {
  /** Additional class for the trigger button. */
  triggerClassName?: string;
  /** If true, shows a chevron arrow in the trigger. */
  showChevron?: boolean;
}

export function ThemeSelector({ triggerClassName = "", showChevron = false }: ThemeSelectorProps) {
  const { theme, setTheme } = useTheme();
  const { locale } = useLocale();
  const isZh = locale === "zh-CN";

  const currentTheme = THEMES.find((t) => t.id === theme) ?? THEMES[0];
  const label = isZh ? currentTheme.labelZh : currentTheme.label;

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
          className="ui-popup ui-popup--menu ui-menu"
          side="right"
          align="end"
          sideOffset={8}
        >
          <ul className="ui-menu__list" role="menu" aria-label="Theme">
            {THEMES.map((t) => {
              const selected = t.id === theme;
              const tLabel = isZh ? t.labelZh : t.label;
              return (
                <li key={t.id}>
                  <Popover.Close asChild>
                    <button
                      type="button"
                      className="ui-menu__item"
                      data-selected={selected || undefined}
                      role="menuitemradio"
                      aria-checked={selected}
                      onClick={() => setTheme(t.id)}
                    >
                      <span className="ui-menu__icon" aria-hidden="true">
                        {selected ? (
                          <Check size={14} />
                        ) : (
                          <span
                            className="theme-swatch"
                            style={{
                              background: t.tokens["--color-bg"],
                              borderColor: t.tokens["--color-border-strong"],
                            }}
                          />
                        )}
                      </span>
                      <span className="ui-menu__label">{tLabel}</span>
                    </button>
                  </Popover.Close>
                </li>
              );
            })}
          </ul>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
