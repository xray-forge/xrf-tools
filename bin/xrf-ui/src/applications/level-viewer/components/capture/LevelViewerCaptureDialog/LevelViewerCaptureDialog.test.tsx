import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { setMockBulkResponses, setMockInvokeResponses } from "@xrf/ipc/testing";

import { LevelViewerCaptureActions } from "@/applications/level-viewer/components/capture/LevelViewerCaptureActions";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/capture";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/snapshot";
import { clearLocalStorage } from "@/core/storage";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockLevelViewContainer, renderLevelView, TLevelViewRenderResult } from "@/fixtures/utils/level-container";

/**
 * @returns The capture's dialog over a level, a capture answering with a 2048 pixel map, and the services behind it.
 */
function renderDialog(): TLevelViewRenderResult & { service: LevelViewerCaptureService } {
  jest.spyOn(LevelViewerSnapshotService.prototype, "level", "get").mockReturnValue("levels/zaton");
  jest.spyOn(LevelViewerCaptureService.prototype, "level", "get").mockReturnValue(mockSelectedLevelDescription());

  const rendered: TLevelViewRenderResult = renderLevelView(
    <LevelViewerCaptureActions />,
    mockLevelViewContainer([LevelViewerCaptureService, LevelViewerSnapshotService])
  );

  jest.spyOn(rendered.weather, "cutTo").mockResolvedValue();
  jest.spyOn(rendered.weather, "release").mockResolvedValue();
  jest.spyOn(rendered.render, "captureMap").mockResolvedValue({ outcome: "completed", width: 2048, height: 2048 });

  return { ...rendered, service: rendered.container.get(LevelViewerCaptureService) };
}

beforeEach(() => {
  clearLocalStorage();
  // A PNG's signature and header, 2048 by 2048.
  setMockBulkResponses({
    ["levels/read_screenshot"]: Uint8Array.from([
      137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 8, 0, 0, 0, 8, 0,
    ]).buffer,
  });
  setMockInvokeResponses({ ["plugin:levels|find_screenshot_path"]: "appdata/screenshots/map_zaton.png" });
  URL.createObjectURL = jest.fn(() => "blob:map");
  URL.revokeObjectURL = jest.fn();
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerCaptureDialog", () => {
  it("asks before deleting a capture again once closed while asking", async () => {
    const { service, getByRole, findByRole, queryByRole } = renderDialog();

    act(() => service.open());
    await act(() => service.capture());
    await userEvent.click(await findByRole("button", { name: "Delete" }));

    expect(getByRole("button", { name: "Delete for good" })).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Close the capture" }));
    await waitFor(() => expect(queryByRole("dialog")).not.toBeInTheDocument());

    act(() => service.open());
    await act(() => service.capture());

    expect(await findByRole("button", { name: "Delete" })).toBeInTheDocument();
    expect(queryByRole("button", { name: "Delete for good" })).not.toBeInTheDocument();
  });
});
