import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { App } from "../app/page";
import { QueryProvider } from "../providers/QueryProvider";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("../services/commands", () => ({
  askAi: vi.fn(),
  buildGraph: vi.fn(),
  detectTechnologies: vi.fn(),
  exportPackage: vi.fn(),
  generateDiagrams: vi.fn(),
  generateDocs: vi.fn(),
  getBackendStatus: vi.fn(),
  getCloudStatus: vi.fn(),
  getCommandErrorMessage: (error: unknown) =>
    error instanceof Error ? error.message : String(error),
  getGitStatus: vi.fn(),
  getMcpStatus: vi.fn(),
  getPerformanceStatus: vi.fn(),
  getPluginStatus: vi.fn(),
  getSecurityStatus: vi.fn(),
  getStorageStatus: vi.fn(),
  isTauriRuntime: () => true,
  listRepositories: vi.fn(async () => []),
  listRepositoryFiles: vi.fn(),
  openRepository: vi.fn(),
  scanRepository: vi.fn(),
}));

describe("App interactions", () => {
  it("switches pages from the sidebar", async () => {
    const user = userEvent.setup();

    render(
      <QueryProvider>
        <App />
      </QueryProvider>,
    );

    await user.click(screen.getByRole("button", { name: /scanner/i }));

    expect(screen.getByRole("heading", { name: "Scanner" })).toBeInTheDocument();
    expect(screen.getByLabelText("Repository path")).toBeInTheDocument();
  });
});
