import { afterEach, describe, expect, it } from "@jest/globals";
import { ReactElement } from "react";

import { TMemoryDetailSource } from "@/core/diagnostics/lib";
import { useEditorStatus, usePublishedMemoryDetails } from "@/core/shell/editor-shell";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const MB: number = 1024 * 1024;

const STATUS: ReadonlyArray<string> = ["3 files"];

const DETAILS: ReadonlyArray<TMemoryDetailSource> = [() => ({ label: "Renderer copies", value: "12 MB" })];

function Tool({ isDetailed = false }: { isDetailed?: boolean }): ReactElement {
  useEditorStatus(STATUS);
  usePublishedMemoryDetails(isDetailed ? DETAILS : []);

  return <></>;
}

describe("ApplicationStatusBar", () => {
  afterEach(() => {
    resetMockInvoke();
  });

  // Every tool shows what the process holds, whether or not it publishes anything of its own about memory.
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
        <Tool isDetailed />
        <ApplicationStatusBar />
      </>,
      { route: "/level-viewer" }
    );

    expect(await findByText("3 files")).toBeInTheDocument();
    expect(getByTestId("application-status-bar").textContent).toBe("3 files");
  });
});
