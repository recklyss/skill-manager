import type { ReactNode } from "react";
import * as Popover from "@radix-ui/react-popover";
import {
  Check,
  ChevronDown,
  Languages,
  RefreshCw,
  Settings,
} from "lucide-react";
import { Link, NavLink } from "react-router-dom";

import { useSidebarModel } from "../app/capability-registry";
import { LoadingSpinner } from "./LoadingSpinner";
import { ThemeSelector } from "./ThemeSelector";
import { useCommonCopy, useLocale } from "../i18n";

interface SidebarProps {
  onRefresh: () => void | Promise<void>;
  refreshPending: boolean;
}

export function Sidebar({ onRefresh, refreshPending }: SidebarProps) {
  const model = useSidebarModel();
  const common = useCommonCopy();

  return (
    <aside className="sidebar ui-scrollbar--thin" aria-label={common.nav.primary}>
      <div className="sidebar__brand">
        <Link to="/overview" className="sidebar__brand-name">
          skill-manager
        </Link>
      </div>

      <nav className="sidebar__nav">
        {model.topLinks.map((link) => (
          <SidebarLink key={link.key} to={link.to} label={link.label} />
        ))}

        {model.groups.map((group) => (
          <SidebarSection
            key={group.key}
            label={group.label}
            count={group.count}
          >
            {group.links.map((link) => (
              <SidebarLink
                key={link.key}
                to={link.to}
                label={link.label}
                count={link.count}
                nested
              />
            ))}
          </SidebarSection>
        ))}
      </nav>

      <div className="sidebar__footer">
        <button
          type="button"
          className="sidebar-footer-btn"
          onClick={() => void onRefresh()}
          disabled={refreshPending}
          aria-busy={refreshPending}
        >
          {refreshPending ? <LoadingSpinner size="sm" label={common.actions.refreshing} /> : <RefreshCw size={15} />}
          <span>{common.actions.refresh}</span>
        </button>
        <ThemeSelector triggerClassName="sidebar-footer-btn" />
        <SidebarLanguageMenu />
        <NavLink
          to="/settings"
          className={({ isActive }) => `sidebar-footer-btn${isActive ? " is-active" : ""}`}
        >
          <Settings size={15} />
          <span>{common.nav.settings}</span>
        </NavLink>
      </div>
    </aside>
  );
}

function SidebarLanguageMenu() {
  const common = useCommonCopy();
  const { locale, setLocale, supportedLocales } = useLocale();
  const activeLabel = supportedLocales.find((option) => option.value === locale)?.nativeLabel ?? locale;

  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <button
          type="button"
          className="sidebar-footer-btn"
          aria-label={common.language.ariaLabel(activeLabel)}
          aria-haspopup="menu"
        >
          <Languages size={15} />
          <span>{activeLabel}</span>
          <ChevronDown className="sidebar-footer-btn__chevron" size={13} aria-hidden="true" />
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          className="ui-popup ui-popup--menu ui-menu"
          side="right"
          align="end"
          sideOffset={8}
        >
          <ul className="ui-menu__list" role="menu" aria-label={common.language.label}>
            {supportedLocales.map((option) => {
              const selected = option.value === locale;
              return (
                <li key={option.value}>
                  <Popover.Close asChild>
                    <button
                      type="button"
                      className="ui-menu__item"
                      data-selected={selected || undefined}
                      role="menuitemradio"
                      aria-checked={selected}
                      onClick={() => setLocale(option.value)}
                    >
                      <span className="ui-menu__icon" aria-hidden="true">
                        {selected ? <Check size={14} /> : null}
                      </span>
                      <span className="ui-menu__label">{option.nativeLabel}</span>
                      <span className="ui-menu__meta">
                        {selected ? common.language.selected : option.label}
                      </span>
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

function SidebarSection({
  label,
  count,
  children,
}: {
  label: string;
  count?: number | null;
  children: ReactNode;
}) {
  const ariaLabel = count != null ? `${label} ${count}` : label;

  return (
    <div className="sidebar-section" role="group" aria-label={ariaLabel}>
      <div className="sidebar-section__label">
        <span>{label}</span>
        {count != null ? <span className="sidebar-section__count">{count}</span> : null}
      </div>
      <div className="sidebar-section__links">{children}</div>
    </div>
  );
}

function SidebarLink({
  to,
  label,
  count,
  nested = false,
}: {
  to: string;
  label: string;
  count?: number | null;
  nested?: boolean;
}) {
  const linkLabel = count != null ? `${label} ${count}` : label;

  return (
    <NavLink
      to={to}
      className={({ isActive }) =>
        `sidebar-link${nested ? " sidebar-link--nested" : ""}${isActive ? " is-active" : ""}`
      }
      aria-label={linkLabel}
    >
      <span className="sidebar-link__label">{label}</span>
      {count != null ? <span className="sidebar-link__count">{count}</span> : null}
    </NavLink>
  );
}
