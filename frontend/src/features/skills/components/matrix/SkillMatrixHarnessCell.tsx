import {
  MatrixHarnessCellTarget,
  MatrixHarnessIcon,
} from "../../../../components/matrix";
import { UiTooltip } from "../../../../components/ui/UiTooltip";
import type { HarnessCell as HarnessCellType } from "../../model/types";

interface SkillMatrixHarnessCellProps {
  cell: HarnessCellType;
  skillName: string;
  pending?: boolean;
  onToggle: (cell: HarnessCellType) => void;
  foundTooltip: (harnessLabel: string, managed: boolean) => string;
}

export function SkillMatrixHarnessCell({
  cell,
  skillName,
  pending = false,
  onToggle,
  foundTooltip,
}: SkillMatrixHarnessCellProps) {
  if (cell.state === "empty") {
    return (
      <span className="matrix-harness-target" data-state="empty" aria-hidden="true">
        —
      </span>
    );
  }

  if (cell.state === "found") {
    const tooltip = foundTooltip(cell.label, cell.interactive);
    return (
      <UiTooltip content={tooltip}>
        <MatrixHarnessCellTarget
          ariaLabel={`${skillName} found in ${cell.label}`}
          state="observed"
          title={tooltip}
        >
          <MatrixHarnessIcon
            label={cell.label}
            logoKey={cell.logoKey}
            harness={cell.harness}
          />
        </MatrixHarnessCellTarget>
      </UiTooltip>
    );
  }

  const isEnabled = cell.state === "enabled";
  const action = isEnabled ? "Disable" : "Enable";

  const button = (
    <MatrixHarnessCellTarget
      ariaLabel={`${action} ${skillName} on ${cell.label}`}
      ariaPressed={isEnabled}
      state={cell.state}
      pending={pending}
      disabled={pending}
      onClick={(event) => {
        event.stopPropagation();
        onToggle(cell);
      }}
    >
      <MatrixHarnessIcon
        label={cell.label}
        logoKey={cell.logoKey}
        harness={cell.harness}
      />
    </MatrixHarnessCellTarget>
  );

  return (
    <UiTooltip content={`${cell.label} — ${isEnabled ? "enabled" : "disabled"}`}>
      {button}
    </UiTooltip>
  );
}
