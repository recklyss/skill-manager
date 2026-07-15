import { Link } from "react-router-dom";

import { HarnessAvatar } from "./harness/HarnessAvatar";
import { useSettingsQuery } from "../features/settings/public";

export function HarnessStatusStrip() {
  const { data } = useSettingsQuery();
  const harnesses = data?.harnesses.filter((harness) => harness.supportEnabled) ?? [];

  if (!data || harnesses.length === 0) {
    return null;
  }

  return (
    <div className="harness-status-strip" aria-label="Harness status">
      <div className="harness-status-strip__items">
        {harnesses.map((harness) => (
          <Link
            key={harness.harness}
            to="/settings"
            className="harness-status-strip__item"
            data-installed={harness.installed}
            title={`${harness.label}${harness.installed ? "" : " (not detected)"}`}
          >
            <HarnessAvatar
              harness={harness.harness}
              label={harness.label}
              logoKey={harness.logoKey}
            />
            <span className="harness-status-strip__label">{harness.label}</span>
            <span className="harness-status-strip__dot" aria-hidden="true" />
          </Link>
        ))}
      </div>
    </div>
  );
}
