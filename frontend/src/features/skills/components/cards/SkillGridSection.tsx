import { useId, useState, type ReactNode } from "react";
import { ChevronDown } from "lucide-react";

import type { SkillCategoryId } from "../../model/skillCategory";
import { useSkillsCopy } from "../../i18n";

interface SkillGridSectionProps {
  category: SkillCategoryId;
  title: string;
  count: number;
  defaultExpanded?: boolean;
  children: ReactNode;
}

export function SkillGridSection({
  category,
  title,
  count,
  defaultExpanded = true,
  children,
}: SkillGridSectionProps) {
  const copy = useSkillsCopy();
  const [expanded, setExpanded] = useState(defaultExpanded);
  const panelId = useId();
  const titleId = `skill-grid-section-${category}`;

  return (
    <section
      className={`skill-grid-section${expanded ? " skill-grid-section--expanded" : " skill-grid-section--collapsed"}`}
      data-category={category}
      aria-labelledby={titleId}
    >
      <header className="skill-grid-section__head">
        <button
          type="button"
          className="skill-grid-section__trigger"
          aria-expanded={expanded}
          aria-controls={panelId}
          aria-label={expanded ? copy.inUse.gridSection.collapse(title) : copy.inUse.gridSection.expand(title)}
          onClick={() => setExpanded((current) => !current)}
        >
          <ChevronDown
            className="skill-grid-section__chevron"
            size={16}
            aria-hidden="true"
            data-expanded={expanded ? "true" : "false"}
          />
          <h3 className="skill-grid-section__title" id={titleId}>
            {title}
          </h3>
          <span className="skill-grid-section__count" aria-label={`${count} skills`}>
            {count}
          </span>
        </button>
      </header>
      <div className="skill-grid-section__panel" id={panelId} hidden={!expanded}>
        <div className="skill-grid">{children}</div>
      </div>
    </section>
  );
}
