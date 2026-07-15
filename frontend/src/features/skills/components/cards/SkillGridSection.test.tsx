import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { SkillGridSection } from "./SkillGridSection";

describe("SkillGridSection", () => {
  it("toggles section visibility when the header is clicked", () => {
    render(
      <SkillGridSection category="coding" title="Coding" count={2}>
        <div>Skill card</div>
      </SkillGridSection>,
    );

    expect(screen.getByText("Skill card")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: /collapse coding/i }));
    expect(screen.getByRole("button", { name: /expand coding/i })).toHaveAttribute("aria-expanded", "false");

    fireEvent.click(screen.getByRole("button", { name: /expand coding/i }));
    expect(screen.getByRole("button", { name: /collapse coding/i })).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("Skill card")).toBeVisible();
  });
});
