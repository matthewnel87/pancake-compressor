import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import type { AppInfo } from "./lib/ipc";

// The Tauri bridge does not exist under jsdom, so the IPC module is mocked.
const getAppInfo = vi.fn<() => Promise<AppInfo>>();
vi.mock("./lib/ipc", () => ({ getAppInfo: () => getAppInfo() }));

describe("launch screen", () => {
  beforeEach(() => {
    getAppInfo.mockReset();
  });

  it("shows the product name as the only level-one heading", async () => {
    getAppInfo.mockResolvedValue({
      schemaVersion: 1,
      appVersion: "0.1.0",
      engineVersion: "0.1.0",
      networkAccess: false,
      telemetry: false,
    });
    render(<App />);
    const headings = screen.getAllByRole("heading", { level: 1 });
    expect(headings).toHaveLength(1);
    expect(headings[0]).toHaveTextContent("Pancake Compressor");
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Version 0.1.0 · Engine 0.1.0 · Works offline · No tracking",
    );
  });

  it("does not offer image selection before compression exists", () => {
    getAppInfo.mockReturnValue(new Promise(() => {}));
    render(<App />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
    expect(screen.getByText(/Image compression is not available/)).toBeInTheDocument();
  });

  it("explains when it is opened outside the desktop app", async () => {
    getAppInfo.mockRejectedValue(new Error("no IPC"));
    render(<App />);
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Version information is only available in the desktop app.",
    );
  });
});
