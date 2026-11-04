import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { RecordingOutcome } from "@xrf/ipc/types/xrf-app-levels";

import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/core/level/fixtures/level-container";
import { EToolId } from "@/core/routing/tool";
import { mockToolScope } from "@/fixtures/utils/container";
import { LevelViewerRecordingToast } from "@/tools/level-viewer/components/recording/LevelViewerRecordingToast";
import { LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";

const OUTCOME: RecordingOutcome = {
  file: "C:\\appdata\\level-viewer\\recordings\\zaton-20261104-230005.mp4",
  duration: 22.8,
  frames: 1369,
  droppedFrames: 0,
  droppedSound: 0,
  width: 2830,
  height: 1930,
  encoder: "NVIDIA H.264 Encoder MFT (hardware)",
};

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerRecordingToast", () => {
  it("tells the recording just written, and deletes it on a second click", async () => {
    const deleted: Array<unknown> = [];

    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_RECORDING]: { isAvailable: true, reason: null },
      [ELevelsCommand.DELETE_RECORDING]: (args?: Record<string, unknown>) => {
        deleted.push(args?.path);

        return null;
      },
    });

    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer([
      ...mockToolScope(EToolId.LEVEL_VIEWER),
      LevelViewerRecordingService,
      LevelViewerSnapshotService,
    ]);
    const service: LevelViewerRecordingService = descriptor.container.get(LevelViewerRecordingService);
    const { getByRole, getByText, queryByRole } = renderLevelView(<LevelViewerRecordingToast />, descriptor);

    expect(queryByRole("status")).toBeNull();

    act(() => {
      service.outcome = OUTCOME;
    });

    expect(getByRole("status", { name: "Recording saved" })).toBeInTheDocument();
    expect(getByText("zaton-20261104-230005.mp4")).toBeInTheDocument();
    expect(getByText(/2830 × 1930 · nothing dropped$/)).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Delete" }));
    await userEvent.click(getByRole("button", { name: "Delete for good" }));

    await waitFor(() => expect(queryByRole("status")).toBeNull());
    expect(deleted).toEqual([OUTCOME.file]);
  });
});
