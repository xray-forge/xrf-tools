import { afterEach, describe, expect, it } from "@jest/globals";
import { ReactElement } from "react";

import { useEditorStatus } from "@/core/shell/editor-shell";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const MB: number = 1024 * 1024;

const STATUS: ReadonlyArray<string> = ["3 files"];

function Tool(): ReactElement {
  useEditorStatus(STATUS);

  return <></>;
}

describe("ApplicationStatusBar", () => {
  afterEach(() => {
    resetMockInvoke();
  });

  // Every tool shows what the process holds.
  it("ends every application's status with what the backend and the webview hold, a letter each", async () => {
    setMockInvokeResponses({
      ["plugin:system|get_memory_usage"]: {
        application: { committed: 30 * MB, workingSet: 24 * MB, privateWorkingSet: 20 * MB },
        webview: [
          {
            kind: "gpu",
            memory: { committed: 2048 * MB, workingSet: 1100 * MB, privateWorkingSet: 1024 * MB },
            pid: 8,
          },
        ],
      },
    });

    const { findByText, getByTestId } = renderWithProviders(
      <>
        <Tool />
        <ApplicationStatusBar />
      </>,
      { route: "/exports-explorer" }
    );

    expect(await findByText("B 20MB W 1GB")).toBeInTheDocument();
    expect(getByTestId("application-status-bar").textContent).toBe("3 filesB 20MB W 1GB");
  });

  it("publishes nothing about memory on a platform that cannot read it", async () => {
    setMockInvokeResponses({ ["plugin:system|get_memory_usage"]: null });

    const { findByText, getByTestId } = renderWithProviders(
      <>
        <Tool />
        <ApplicationStatusBar />
      </>,
      { route: "/level-viewer" }
    );

    expect(await findByText("3 files")).toBeInTheDocument();
    expect(getByTestId("application-status-bar").textContent).toBe("3 files");
  });
});
