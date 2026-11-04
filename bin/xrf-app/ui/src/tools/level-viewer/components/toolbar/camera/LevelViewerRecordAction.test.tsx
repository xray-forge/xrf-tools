import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { emitMockChannelMessage, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { RecordingOutcome, RecordingProgress } from "@xrf/ipc/types/xrf-app-levels";

import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { EToolId } from "@/core/routing/tool";
import { mockToolScope } from "@/fixtures/utils/container";
import { LevelViewerRecordAction } from "@/tools/level-viewer/components/toolbar/camera/LevelViewerRecordAction";
import { LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";

const FILE: string = "appdata/level-viewer/recordings/zaton-20261104-230000.mp4";

const OUTCOME: RecordingOutcome = {
  file: FILE,
  duration: 3,
  frames: 180,
  droppedFrames: 0,
  droppedSound: 0,
  width: 1846,
  height: 1012,
  encoder: "NVIDIA H.264 Encoder MFT (hardware)",
};

/** A recording's word, three seconds in. */
function progress(ended: RecordingProgress["ended"] = null): RecordingProgress {
  return { elapsed: 3, frames: 180, droppedFrames: 0, droppedSound: 0, ended };
}

/**
 * @param isAvailable - Whether this machine records.
 * @returns The action drawn over a recorder mocked at the commands, its service and how it was asked to stop.
 */
async function renderAction(
  isAvailable: boolean = true
): Promise<TLevelViewRenderResult & { service: LevelViewerRecordingService; stopped: Array<unknown> }> {
  const stopped: Array<unknown> = [];

  mockIpcCommands({
    [ELevelsCommand.DESCRIBE_RECORDING]: isAvailable
      ? { isAvailable: true, reason: null }
      : { isAvailable: false, reason: "mfplat.dll does not load: install the Media Feature Pack" },
    [ELevelsCommand.STOP_RECORDING]: () => {
      stopped.push(true);

      return OUTCOME;
    },
  });
  jest.spyOn(LevelViewerSnapshotService.prototype, "level", "get").mockReturnValue("levels/zaton");

  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer([
    ...mockToolScope(EToolId.LEVEL_VIEWER),
    LevelViewerRecordingService,
    LevelViewerSnapshotService,
  ]);

  jest.spyOn(descriptor.render.captures, "startRecording").mockResolvedValue(FILE);

  const rendered: TLevelViewRenderResult = renderLevelView(<LevelViewerRecordAction />, descriptor);
  const service: LevelViewerRecordingService = descriptor.container.get(LevelViewerRecordingService);

  await waitFor(() => expect(service.availability).not.toBeNull());

  return { ...rendered, service, stopped };
}

beforeEach(() => {
  resetMockInvoke();
  resetMockChannels();
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerRecordAction", () => {
  it("records on a click, lit and telling the time and the drops, and stops on the next", async () => {
    const { getByRole, service, stopped } = await renderAction();
    const button: HTMLElement = getByRole("button", { name: "Record" });

    await userEvent.click(button);
    await waitFor(() => expect(button).toHaveAttribute("aria-pressed", "true"));

    emitMockChannelMessage(progress());

    await waitFor(() => expect(button.getAttribute("aria-description")).toMatch(/^Recording .+, nothing dropped\./));

    await userEvent.click(button);
    await waitFor(() => expect(button).toHaveAttribute("aria-pressed", "false"));
    expect(stopped).toHaveLength(1);
    expect(service.outcome).toEqual(OUTCOME);
  });

  it("stops and collects a recording that ended by itself", async () => {
    const { getByRole, service, stopped } = await renderAction();

    await userEvent.click(getByRole("button", { name: "Record" }));
    await waitFor(() => expect(service.isRecording).toBe(true));

    emitMockChannelMessage(progress("The recording ended: the audio went from 48000 Hz to 44100 Hz"));

    await waitFor(() => expect(service.isRecording).toBe(false));
    expect(stopped).toHaveLength(1);
  });

  it("is off where this machine cannot record, saying why", async () => {
    const { getByRole } = await renderAction(false);
    const button: HTMLElement = getByRole("button", { name: "Record" });

    expect(button).toBeDisabled();
    expect(button).toHaveAttribute("aria-description", "mfplat.dll does not load: install the Media Feature Pack");
  });
});
