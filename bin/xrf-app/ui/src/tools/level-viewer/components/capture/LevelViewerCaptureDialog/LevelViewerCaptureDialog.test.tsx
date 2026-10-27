import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ELevelsBulkRoute } from "@xrf/ipc/commands/levels-bulk";
import { mockIpcBulkRoutes, mockIpcCommands } from "@xrf/ipc/testing";

import { mockLevelViewContainer, renderLevelView, TLevelViewRenderResult } from "@/core/level/fixtures/level-container";
import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { clearLocalStorage } from "@/core/storage";
import { mockFlow } from "@/fixtures/utils/flow";
import { LevelViewerCaptureActions } from "@/tools/level-viewer/components/capture/LevelViewerCaptureActions";
import { LevelViewerCaptureService } from "@/tools/level-viewer/services/level-viewer-capture";
import { LevelViewerCaptureResultService } from "@/tools/level-viewer/services/level-viewer-capture-result";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";

/**
 * @returns The capture's dialog over a level, a capture answering with a 2048 pixel map, and the services behind it.
 */
function renderDialog(): TLevelViewRenderResult & { service: LevelViewerCaptureService } {
  jest.spyOn(LevelViewerSnapshotService.prototype, "level", "get").mockReturnValue("levels/zaton");
  jest.spyOn(LevelViewerCaptureService.prototype, "level", "get").mockReturnValue(mockSelectedLevelDescription());

  const rendered: TLevelViewRenderResult = renderLevelView(
    <LevelViewerCaptureActions />,
    mockLevelViewContainer([LevelViewerCaptureService, LevelViewerCaptureResultService, LevelViewerSnapshotService])
  );

  mockFlow(rendered.weather, "cutTo");
  mockFlow(rendered.weather, "release");
  jest.spyOn(rendered.render, "captureMap").mockResolvedValue({ outcome: "completed", width: 2048, height: 2048 });

  return { ...rendered, service: rendered.container.get(LevelViewerCaptureService) };
}

beforeEach(() => {
  clearLocalStorage();
  // A PNG's signature and header, 2048 by 2048.
  mockIpcBulkRoutes({
    [ELevelsBulkRoute.READ_SCREENSHOT]: Uint8Array.from([
      137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 8, 0, 0, 0, 8, 0,
    ]).buffer,
  });
  mockIpcCommands({ [ELevelsCommand.FIND_SCREENSHOT_PATH]: "appdata/screenshots/map_zaton.png" });
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
