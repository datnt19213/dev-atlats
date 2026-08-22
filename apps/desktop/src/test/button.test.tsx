import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { Button } from "../components/ui/button";

describe("Button", () => {
  it("renders an accessible button and handles interaction", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Scan repository</Button>);
    await user.click(screen.getByRole("button", { name: "Scan repository" }));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("applies the borderless secondary Tailwind variant classes", () => {
    render(<Button variant="secondary">Browse</Button>);
    expect(screen.getByRole("button", { name: "Browse" })).toHaveClass("bg-secondary");
    expect(screen.getByRole("button", { name: "Browse" })).toHaveClass("text-secondary-foreground");
    expect(screen.getByRole("button", { name: "Browse" })).not.toHaveClass("border");
  });

  it("supports rendering a child element", () => {
    render(<Button asChild><a href="/exports">Exports</a></Button>);
    expect(screen.getByRole("link", { name: "Exports" })).toHaveAttribute("href", "/exports");
  });
});
