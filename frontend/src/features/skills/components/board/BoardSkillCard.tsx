import type { CSSProperties } from "react";
import { useDraggable } from "@dnd-kit/core";

import { CardTitleRow } from "../cards/CardTitleRow";
import { HarnessChipStack } from "../cards/HarnessChipStack";
import type { SkillListRow } from "../../model/types";

interface BoardSkillCardProps {
  row: SkillListRow;
  checked: boolean;
  pending?: boolean;
  multiDragCount?: number;
  onOpenSkill: (skillRef: string) => void;
  onToggleChecked: (skillRef: string) => void;
}

/** Presentational card body shared by the in-column card and the drag overlay. */
export function BoardCardBody({
  row,
  checked,
  onToggleChecked,
}: {
  row: SkillListRow;
  checked: boolean;
  onToggleChecked?: (skillRef: string) => void;
}) {
  return (
    <>
      <CardTitleRow
        name={row.name}
        checked={checked}
        onToggleChecked={() => onToggleChecked?.(row.skillRef)}
      />

      {row.description ? (
        <p className="skill-card__description skill-card__description--compact">{row.description}</p>
      ) : null}

      <HarnessChipStack cells={row.cells} />
    </>
  );
}

/** The floating clone rendered inside dnd-kit's DragOverlay while dragging. */
export function BoardCardOverlay({
  row,
  checked,
  multiDragCount,
}: {
  row: SkillListRow;
  checked: boolean;
  multiDragCount?: number;
}) {
  const showMultiDragBadge = checked && (multiDragCount ?? 0) > 1;
  return (
    <article
      className="skill-card skill-card--board skill-card--overlay"
      data-multi-drag={showMultiDragBadge ? "true" : undefined}
    >
      <BoardCardBody row={row} checked={checked} />
      {showMultiDragBadge ? (
        <span className="skill-card__multi-badge" aria-hidden="true">
          +{multiDragCount! - 1}
        </span>
      ) : null}
    </article>
  );
}

export function BoardSkillCard({
  row,
  checked,
  pending = false,
  onOpenSkill,
  onToggleChecked,
}: BoardSkillCardProps) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: row.skillRef,
  });

  // Movement is handled by the DragOverlay clone; the source card just dims.
  const style: CSSProperties = isDragging ? { opacity: 0 } : {};

  return (
    <article
      ref={setNodeRef}
      {...attributes}
      {...listeners}
      className="skill-card skill-card--board"
      data-checked={checked}
      data-dragging={isDragging ? "true" : undefined}
      data-pending={pending ? "true" : undefined}
      style={style}
      onClick={() => {
        if (isDragging) return;
        onOpenSkill(row.skillRef);
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onOpenSkill(row.skillRef);
        }
      }}
      role="button"
      tabIndex={0}
    >
      <BoardCardBody row={row} checked={checked} onToggleChecked={onToggleChecked} />
    </article>
  );
}
