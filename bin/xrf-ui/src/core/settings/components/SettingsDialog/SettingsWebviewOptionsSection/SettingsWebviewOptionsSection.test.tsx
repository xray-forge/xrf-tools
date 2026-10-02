import { describe, expect, it } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { EWebviewCollectionPace, WebviewOptions, WebviewOptionsStatus } from "@/core/ipc/types/xrf-app";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SettingsWebviewOptionsSection } from "./SettingsWebviewOptionsSection";
import { isRestartPending } from "./SettingsWebviewOptionsSection.utils";

const STARTED: WebviewOptions = {
  collectionPace: EWebviewCollectionPace.DEFAULT,
  isFrameRateLimited: true,
  isShaderCacheDoubled: true,
  isVsync: true,
  isWebgpuDeveloper: false,
};

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

    await waitFor(() => expect(backend.chosen).toEqual({ ...STARTED, isWebgpuDeveloper: true }));
    expect(mockInvoke).toHaveBeenCalledWith("plugin:system|set_webview_options", {
      options: { ...STARTED, isWebgpuDeveloper: true },
    });
    expect(developer).toBeChecked();
    expect(await findByText("Restart to apply")).toBeInTheDocument();
  });

  // The renderer's pause after each major collection follows what piled up since: collecting earlier shortens it.
  it("keeps a garbage collection pace chosen for the next start", async () => {
    const backend: { chosen: WebviewOptions } = mockBackend();
    const { findByRole, getByRole } = renderWithProviders(<SettingsWebviewOptionsSection />);

    expect(await findByRole("button", { name: "V8's own" })).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(getByRole("button", { name: "Earlier" }));

    await waitFor(() => expect(backend.chosen).toEqual({ ...STARTED, collectionPace: EWebviewCollectionPace.EARLIER }));
    expect(getByRole("button", { name: "Earlier" })).toHaveAttribute("aria-pressed", "true");
  });

  it("keeps vsync and the frame rate limit lifted each on its own for the next start", async () => {
    const backend: { chosen: WebviewOptions } = mockBackend();
    const { findByRole, getByRole } = renderWithProviders(<SettingsWebviewOptionsSection />);

    await userEvent.click(await findByRole("checkbox", { name: "Vsync" }));
    await waitFor(() => expect(backend.chosen).toEqual({ ...STARTED, isVsync: false }));
    expect(getByRole("checkbox", { name: "Frame rate limit" })).toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Frame rate limit" }));
    await waitFor(() => expect(backend.chosen).toEqual({ ...STARTED, isVsync: false, isFrameRateLimited: false }));
  });

  it("says the options could not be read, and reads them again on retry", async () => {
    let isFailing: boolean = true;

    setMockInvokeResponses({
      ["plugin:system|get_webview_options"]: (): WebviewOptionsStatus => {
        if (isFailing) {
          throw new Error("No preferences");
        }

        return { running: STARTED, chosen: STARTED };
      },
    });

    const { findByRole, findByText, getByRole } = renderWithProviders(<SettingsWebviewOptionsSection />);

    expect(await findByText("The webview options could not be read: No preferences")).toBeInTheDocument();

    isFailing = false;
    await userEvent.click(getByRole("button", { name: "Retry" }));

    expect(await findByRole("checkbox", { name: "Doubled shader cache" })).toBeChecked();
  });

  // Two choices made quickly are kept in turn, but their answers may land in either order.
  it("shows the answer to the latest choice, whatever an earlier one answers after it", async () => {
    const answers: Array<(status: WebviewOptionsStatus) => void> = [];

    setMockInvokeResponses({
      ["plugin:system|get_webview_options"]: (): WebviewOptionsStatus => ({ running: STARTED, chosen: STARTED }),
      ["plugin:system|set_webview_options"]: () =>
        new Promise((resolve: (status: WebviewOptionsStatus) => void) => answers.push(resolve)),
    });

    const { findByRole, getByRole } = renderWithProviders(<SettingsWebviewOptionsSection />);

    await userEvent.click(await findByRole("checkbox", { name: "Vsync" }));
    await userEvent.click(getByRole("checkbox", { name: "Frame rate limit" }));
    await waitFor(() => expect(answers).toHaveLength(2));

    const latest: WebviewOptions = { ...STARTED, isFrameRateLimited: false, isVsync: false };

    await act(async () => answers[1]({ running: STARTED, chosen: latest }));
    await act(async () => answers[0]({ running: STARTED, chosen: { ...STARTED, isVsync: false } }));

    expect(getByRole("checkbox", { name: "Frame rate limit" })).not.toBeChecked();
    expect(getByRole("checkbox", { name: "Vsync" })).not.toBeChecked();
  });

  it("waits for a restart only where a choice differs from what the webview started with", () => {
    expect(isRestartPending({ running: STARTED, chosen: STARTED })).toBe(false);
    expect(isRestartPending({ running: STARTED, chosen: { ...STARTED, isShaderCacheDoubled: false } })).toBe(true);
  });
});
