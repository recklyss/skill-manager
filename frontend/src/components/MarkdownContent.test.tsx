import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import MarkdownContent from "./MarkdownContent";

describe("MarkdownContent", () => {
  it("renders headings and paragraphs from remote markdown", () => {
    render(<MarkdownContent markdown={"## Usage\n\nInspect traces."} />);

    expect(screen.getByRole("heading", { level: 2, name: "Usage" })).toBeInTheDocument();
    expect(screen.getByText("Inspect traces.")).toBeInTheDocument();
  });

  it("renders external links with safe rel attributes", () => {
    render(<MarkdownContent markdown={"[Docs](https://example.com/docs)"} />);

    const link = screen.getByRole("link", { name: "Docs" });
    expect(link).toHaveAttribute("href", "https://example.com/docs");
    expect(link).toHaveAttribute("target", "_blank");
    expect(link).toHaveAttribute("rel", "noopener noreferrer");
  });

  it("does not render raw html injected into markdown", () => {
    render(<MarkdownContent markdown={'<script>alert("xss")</script>\n\n**safe**'} />);

    expect(document.querySelector("script")).toBeNull();
    expect(screen.getByText("safe")).toBeInTheDocument();
  });
});
