import type { CellActionKey, StructuralSkillAction } from "../../model/pending";
import { groupRowsByCategory, type SkillCategoryId } from "../../model/skillCategory";
import type { SkillListRow } from "../../model/types";
import { useSkillsCopy } from "../../i18n";
import { SkillGridSection } from "./SkillGridSection";
import { SkillInUseCard } from "./SkillInUseCard";

interface SkillsInUseListProps {
  ariaLabel?: string;
  rows: SkillListRow[];
  pendingToggleKeys: ReadonlySet<CellActionKey>;
  pendingStructuralActions: ReadonlyMap<string, StructuralSkillAction>;
  selectedSkillRef: string | null;
  checkedRefs: ReadonlySet<string>;
  onOpenSkill: (skillRef: string) => void;
  onToggleChecked: (skillRef: string) => void;
  onSetAllHarnesses: (
    skillRef: string,
    target: "enabled" | "disabled",
  ) => Promise<unknown> | void;
  onRequestRemove: (row: SkillListRow) => void;
  onRequestDelete: (row: SkillListRow) => void;
}

export function SkillsInUseList({
  ariaLabel,
  rows,
  pendingToggleKeys,
  pendingStructuralActions,
  selectedSkillRef,
  checkedRefs,
  onOpenSkill,
  onToggleChecked,
  onSetAllHarnesses,
  onRequestRemove,
  onRequestDelete,
}: SkillsInUseListProps) {
  const copy = useSkillsCopy();
  const groups = groupRowsByCategory(rows);

  return (
    <div className="skill-grid-grouped" aria-label={ariaLabel ?? copy.detail.inUseList}>
      {groups.map((group) => (
        <SkillGridSection
          key={group.category}
          category={group.category}
          title={categoryLabel(copy, group.category)}
          count={group.rows.length}
        >
          {group.rows.map((row) => (
            <SkillInUseCard
              key={row.skillRef}
              row={row}
              pendingToggleKeys={pendingToggleKeys}
              pendingStructuralAction={pendingStructuralActions.get(row.skillRef) ?? null}
              selected={selectedSkillRef === row.skillRef}
              checked={checkedRefs.has(row.skillRef)}
              onOpenSkill={onOpenSkill}
              onToggleChecked={onToggleChecked}
              onSetAllHarnesses={onSetAllHarnesses}
              onRequestRemove={onRequestRemove}
              onRequestDelete={onRequestDelete}
            />
          ))}
        </SkillGridSection>
      ))}
    </div>
  );
}

function categoryLabel(copy: ReturnType<typeof useSkillsCopy>, category: SkillCategoryId): string {
  return copy.inUse.gridCategories[category];
}
