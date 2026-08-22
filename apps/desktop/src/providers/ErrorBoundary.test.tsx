import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactElement } from "react";

import { ErrorBoundary } from "./ErrorBoundary";

function BrokenView(): ReactElement {
  throw new Error("render failed");
}

describe("ErrorBoundary", () => {
  let consoleError: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
  });

  afterEach(() => {
    consoleError.mockRestore();
  });

  it("renders children when no presentation error occurs", () => {
    render(
      <ErrorBoundary>
        <span>Dashboard ready</span>
      </ErrorBoundary>,
    );

    expect(screen.getByText("Dashboard ready")).toBeInTheDocument();
  });

  it("renders a fallback when a child throws during render", () => {
    render(
      <ErrorBoundary>
        <BrokenView />
      </ErrorBoundary>,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("DevAtlas encountered a presentation error.");
    expect(screen.getByText("render failed")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Reload" })).toBeInTheDocument();
  });
});
