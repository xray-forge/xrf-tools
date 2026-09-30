import { describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { WebviewOptions, WebviewOptionsStatus } from "@/core/ipc/types/xrf-app";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SettingsWebviewOptionsSection } from "./SettingsWebviewOptionsSection";
import { isRestartPending } from "./SettingsWebviewOptionsSection.utils";

const STARTED: WebviewOptions = { isShaderCacheDoubled: true, isWebgpuDeveloper: false };

function mockBackend(): { chosen: WebviewOptions } {
  const backend: { chosen: WebviewOptions } = { chosen: STARTED };

  setMockInvokeResponses({
    ["plugin:system|get_webview_options"]: (): WebviewOptionsStatus => ({ running: STARTED, chosen: backend.chosen }),
    ["plugin:system|set_webview_options"]: (args?: Record<string, unknown>): WebviewOptionsStatus => {
      backend.chosen = args?.options as WebviewOptions;

      return { running: STARTED, chosen: backend.chosen };
    },
  });

  return backend;
}

describe("SettingsWebviewOptionsSection", () => {
  it("shows the options chosen for the next start", async () => {
    mockBackend();

    const { findByRole, getByRole } = renderWithProviders(<SettingsWebviewOptionsSection />);

    expect(await findByRole("checkbox", { name: "Doubled shader cache" })).toBeChecked();
    expect(getByRole("checkbox", { name: "WebGPU developer features" })).not.toBeChecked();
  });

  it("keeps a choice through the backend, the rest of the choice with it, and says it waits for a restart", async () => {
    const backend: { chosen: WebviewOptions } = mockBackend();
    const { findByRole, findByText, queryByText } = renderWithProviders(<SettingsWebviewOptionsSection />);

    const developer: HTMLElement = await findByRole("checkbox", { name: "WebGPU developer features" });

    expect(queryByText("Restart to apply")).not.toBeInTheDocument();

    await userEvent.click(developer);

    await waitFor(() => expect(backend.chosen).toEqual({ isShaderCacheDoubled: true, isWebgpuDeveloper: true }));
    expect(mockInvoke).toHaveBeenCalledWith("plugin:system|set_webview_options", {
      options: { isShaderCacheDoubled: true, isWebgpuDeveloper: true },
    });
    expect(developer).toBeChecked();
    expect(await findByText("Restart to apply")).toBeInTheDocument();
  });

  it("waits for a restart only where a choice differs from what the webview started with", () => {
    expect(isRestartPending({ running: STARTED, chosen: STARTED })).toBe(false);
    expect(isRestartPending({ running: STARTED, chosen: { ...STARTED, isShaderCacheDoubled: false } })).toBe(true);
  });
});
