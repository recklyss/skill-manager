import { readFileSync, readdirSync } from "fs";
import { join } from "path";
import { describe, expect, it } from "vitest";

import { classifySkillCategory, groupRowsByCategory } from "./skillCategory";
import type { SkillListRow } from "./types";

function row(name: string, description = ""): SkillListRow {
  return {
    skillRef: `shared:${name}`,
    name,
    description,
    displayStatus: "Managed",
    actions: { canManage: false, canStopManaging: true, canDelete: false },
    cells: [],
  } as unknown as SkillListRow;
}

describe("classifySkillCategory", () => {
  it("classifies by skill name, not incidental description words", () => {
    expect(
      classifySkillCategory(
        row(
          "frontend-slides",
          "Build from topic/notes, PPTX conversion, or enhance an existing HTML deck.",
        ),
      ),
    ).toBe("media");
  });

  it("classifies common skill families from slug", () => {
    expect(classifySkillCategory(row("parallel-code-review"))).toBe("coding");
    expect(classifySkillCategory(row("memory-manager"))).toBe("memory");
    expect(classifySkillCategory(row("gemini-image-generator"))).toBe("media");
    expect(classifySkillCategory(row("firecrawl-search"))).toBe("research");
    expect(classifySkillCategory(row("blog-post-writer"))).toBe("docs");
    expect(classifySkillCategory(row("subagent-driven-development"))).toBe("agents");
    expect(classifySkillCategory(row("feature-plan"))).toBe("planning");
    expect(classifySkillCategory(row("perses-query-builder"))).toBe("data");
    expect(classifySkillCategory(row("perses-deploy"))).toBe("devops");
    expect(classifySkillCategory(row("security-threat-model"))).toBe("security");
    expect(classifySkillCategory(row("image-auditor"))).toBe("security");
    expect(classifySkillCategory(row("teach"))).toBe("communication");
  });

  it("falls back to description only when slug is ambiguous", () => {
    expect(classifySkillCategory(row("custom-helper", "Run a security audit for dependencies"))).toBe(
      "security",
    );
  });

  it("falls back to other for generic skills", () => {
    expect(classifySkillCategory(row("using-superpowers"))).toBe("other");
    expect(classifySkillCategory(row("verification-before-completion"))).toBe("other");
  });
});

describe("groupRowsByCategory", () => {
  it("groups rows and omits empty categories", () => {
    const groups = groupRowsByCategory([
      row("memory-manager"),
      row("parallel-code-review"),
      row("blog-post-writer"),
      row("frontend-slides"),
    ]);
    expect(groups.map((group) => group.category)).toEqual(["coding", "docs", "media", "memory"]);
  });
});

describe("classifySkillCategory fixture audit", () => {
  it("assigns every local fixture skill to a stable category", () => {
    const skillsRoot = join(process.env.HOME ?? "", ".claude", "skills");
    let names: string[] = [];
    try {
      names = readdirSync(skillsRoot).filter((entry) => !entry.startsWith("."));
    } catch {
      return;
    }

    const unexpected: string[] = [];
    for (const name of names) {
      let description = "";
      try {
        const raw = readFileSync(join(skillsRoot, name, "SKILL.md"), "utf8");
        const match =
          raw.match(/description:\s*\|\n([\s\S]*?)(?:\n[a-zA-Z][\w-]*:|\n---)/) ??
          raw.match(/description:\s*(.+)/);
        description = match?.[1]?.replace(/^\s+/gm, " ").trim() ?? "";
      } catch {
        // ignore unreadable fixtures
      }

      const category = classifySkillCategory(row(name, description));
      if (category === "other" && !["using-superpowers", "verification-before-completion", "shared-patterns", "INDEX.json"].includes(name)) {
        unexpected.push(name);
      }
    }

    expect(unexpected, `unexpected 'other' categories: ${unexpected.join(", ")}`).toEqual([]);
  });
});
