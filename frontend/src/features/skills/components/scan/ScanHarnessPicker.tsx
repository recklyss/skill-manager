import type { ScanHarnessOption } from "../../api/scan-client";
import { useSkillsCopy } from "../../i18n";

export type ScanHarnessPickerVariant = "select" | "list";

interface ScanHarnessPickerProps {
  variant: ScanHarnessPickerVariant;
  harnesses: ScanHarnessOption[];
  selectedHarness: string | null;
  harnessesLoaded: boolean;
  onSelectHarness: (harness: string) => void;
  className?: string;
  idPrefix?: string;
}

function unavailableReason(
  entry: ScanHarnessOption,
  copy: ReturnType<typeof useSkillsCopy>["scan"],
): string {
  if (!entry.cliAvailable) {
    return copy.harnessUnavailableCli;
  }
  return copy.harnessUnavailableNotScannable;
}

export function ScanHarnessPicker({
  variant,
  harnesses,
  selectedHarness,
  harnessesLoaded,
  onSelectHarness,
  className,
  idPrefix = "scan-harness",
}: ScanHarnessPickerProps) {
  const copy = useSkillsCopy().scan;
  const scannableHarnesses = harnesses.filter((entry) => entry.scannable);
  const rootClassName = [
    "scan-harness-picker",
    variant === "list" ? "scan-harness-picker--list" : "scan-harness-picker--select",
    className,
  ]
    .filter(Boolean)
    .join(" ");

  if (variant === "select") {
    return (
      <div className={rootClassName}>
        <label className="scan-toolbar__field">
          <span>{copy.harnessLabel}</span>
          <select
            className="scan-toolbar__select"
            value={selectedHarness ?? ""}
            disabled={!harnessesLoaded || scannableHarnesses.length === 0}
            onChange={(event) => onSelectHarness(event.target.value)}
            aria-label={copy.harnessAria}
          >
            {!harnessesLoaded ? (
              <option value="">{copy.loadingHarnesses}</option>
            ) : scannableHarnesses.length === 0 ? (
              <option value="">{copy.noHarnesses}</option>
            ) : (
              scannableHarnesses.map((entry) => (
                <option key={entry.harness} value={entry.harness}>
                  {entry.label}
                </option>
              ))
            )}
          </select>
        </label>
        {harnessesLoaded && scannableHarnesses.length === 0 ? (
          <p className="scan-toolbar__hint">{copy.noHarnessesHint}</p>
        ) : null}
      </div>
    );
  }

  if (!harnessesLoaded) {
    return (
      <div className={rootClassName} aria-busy="true">
        <p className="scan-harness-picker__loading">{copy.loadingHarnesses}</p>
      </div>
    );
  }

  if (harnesses.length === 0) {
    return (
      <div className={rootClassName}>
        <p className="scan-harness-picker__empty">{copy.noHarnesses}</p>
        <p className="scan-harness-picker__hint">{copy.noHarnessesHint}</p>
      </div>
    );
  }

  return (
    <fieldset className={rootClassName} aria-label={copy.configHarnessHeading}>
      <legend className="scan-harness-picker__legend">{copy.configHarnessHeading}</legend>
      <ul className="scan-harness-picker__options">
        {harnesses.map((entry) => {
          const inputId = `${idPrefix}-${entry.harness}`;
          const unavailable = !entry.scannable;
          const checked = selectedHarness === entry.harness;

          return (
            <li
              key={entry.harness}
              className={
                unavailable
                  ? "scan-harness-picker__option scan-harness-picker__option--unavailable"
                  : "scan-harness-picker__option"
              }
            >
              <label className="scan-harness-picker__label" htmlFor={inputId}>
                <input
                  id={inputId}
                  type="radio"
                  name={`${idPrefix}-harness`}
                  className="scan-harness-picker__input"
                  value={entry.harness}
                  checked={checked}
                  disabled={unavailable}
                  onChange={() => onSelectHarness(entry.harness)}
                />
                <span className="scan-harness-picker__name">{entry.label}</span>
                {unavailable ? (
                  <span className="scan-harness-picker__reason">
                    {unavailableReason(entry, copy)}
                  </span>
                ) : null}
              </label>
            </li>
          );
        })}
      </ul>
      {scannableHarnesses.length === 0 ? (
        <p className="scan-harness-picker__hint">{copy.noHarnessesHint}</p>
      ) : (
        <p className="scan-harness-picker__hint">{copy.configHarnessHint}</p>
      )}
    </fieldset>
  );
}
