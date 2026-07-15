import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { ToastProvider, useToast } from "./Toast";

function ToastTrigger({ message, variant }: { message: string; variant?: "success" | "error" | "info" }) {
  const { toast } = useToast();
  return (
    <button type="button" onClick={() => toast(message, variant ? { variant } : undefined)}>
      Show toast
    </button>
  );
}

describe("Toast", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders a flat minimal toast with the requested variant", () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Installed example-skill" variant="success" />
      </ToastProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: /show toast/i }));

    const toast = screen.getByText("Installed example-skill");
    expect(toast.closest(".toast")).toHaveClass("toast--success");
  });

  it("defaults to the info variant", () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Command copied" />
      </ToastProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: /show toast/i }));

    const toast = screen.getByText("Command copied");
    expect(toast.closest(".toast")).toHaveClass("toast--info");
  });

  it("removes the toast after the display duration", () => {
    render(
      <ToastProvider>
        <ToastTrigger message="Temporary message" variant="error" />
      </ToastProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: /show toast/i }));
    expect(screen.getByText("Temporary message")).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(3200);
    });

    expect(screen.queryByText("Temporary message")).not.toBeInTheDocument();
  });
});
