import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Channel } from "@tauri-apps/api/core";
import { waitFor } from "@testing-library/react";
import { EventBus, WireEvent } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { emitMockChannelMessage, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { RecordingOutcome, RecordingProgress } from "@xrf/ipc/types/xrf-app-levels";

import { ILevelViewContainerDescriptor, mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { LevelRenderCaptures } from "@/core/level/services/level-render/level-render.service.captures";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { EToolId } from "@/core/routing/tool";
import { mockToolScope } from "@/fixtures/utils/container";
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

function progress(ended: RecordingProgress["ended"] = null): RecordingProgress {
  return { elapsed: 3, frames: 180, droppedFrames: 0, droppedSound: 0, ended };
}

interface IMockedRecording {
  service: LevelViewerRecordingService;
  raised: Array<INotificationPayload>;
  asked: Array<string>;
  startRecording: jest.SpiedFunction<LevelRenderCaptures["startRecording"]>;
}

async function mockRecording(
  outcome: RecordingOutcome = OUTCOME,
  isAvailable: boolean = true
): Promise<IMockedRecording> {
  const asked: Array<string> = [];

  mockIpcCommands({
    [ELevelsCommand.DESCRIBE_RECORDING]: isAvailable
      ? { isAvailable: true, reason: null }
      : { isAvailable: false, reason: "mfplat.dll does not load: install the Media Feature Pack" },
    [ELevelsCommand.STOP_RECORDING]: () => {
      asked.push("stop");

      return outcome;
    },
    [ELevelsCommand.DELETE_RECORDING]: (args?: Record<string, unknown>) => {
      asked.push(`delete ${String(args?.path)}`);

      return null;
    },
  });
  jest.spyOn(LevelViewerSnapshotService.prototype, "level", "get").mockReturnValue("levels/zaton");

  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer([
    ...mockToolScope(EToolId.LEVEL_VIEWER),
    LevelViewerRecordingService,
    LevelViewerSnapshotService,
  ]);
  const raised: Array<INotificationPayload> = [];

  const startRecording: jest.SpiedFunction<LevelRenderCaptures["startRecording"]> = jest
    .spyOn(descriptor.render.captures, "startRecording")
    .mockResolvedValue(FILE);

  descriptor.container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );

  const service: LevelViewerRecordingService = descriptor.container.get(LevelViewerRecordingService);

  await waitFor(() => expect(service.availability).not.toBeNull());

  return { service, raised, asked, startRecording };
}

beforeEach(() => {
  resetMockInvoke();
  resetMockChannels();
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerRecordingService", () => {
  it("records on a toggle, telling where it stands, and stops on the next, its outcome kept for its toast", async () => {
    const { service, raised, asked } = await mockRecording();

    await flowResult(service.toggle());

    expect(service.isRecording).toBe(true);
    expect(service.file).toBe(FILE);

    emitMockChannelMessage(progress());

    await waitFor(() => expect(service.progress?.frames).toBe(180));

    await flowResult(service.toggle());

    expect(service.isRecording).toBe(false);
    expect(asked).toEqual(["stop"]);
    expect(service.outcome).toEqual(OUTCOME);
    // Its toast tells a recording that dropped nothing; no notice.
    expect(raised).toEqual([]);
  });

  it("warns of a recording that dropped frames or sound, as its toast fades", async () => {
    const { service, raised } = await mockRecording({ ...OUTCOME, droppedFrames: 3, droppedSound: 1 });

    await flowResult(service.start());
    await flowResult(service.stop());

    expect(raised).toEqual([
      expect.objectContaining({
        severity: ENotificationSeverity.WARNING,
        title: "Recorded with drops",
      }),
    ]);
  });

  it("stops and collects a recording that ended by itself", async () => {
    const { service, asked } = await mockRecording();

    await flowResult(service.start());

    emitMockChannelMessage(progress("The recording ended: the audio went from 48000 Hz to 44100 Hz"));

    await waitFor(() => expect(service.isRecording).toBe(false));
    expect(asked).toEqual(["stop"]);
  });

  it("collects a recording that said it ended before its start answered", async () => {
    const { service, asked, startRecording } = await mockRecording();

    startRecording.mockImplementation(async (_name: string, channel: Channel<RecordingProgress>) => {
      channel.onmessage(progress("Another long job runs"));

      return FILE;
    });

    expect(await flowResult(service.start())).toBeNull();
    expect(service.isRecording).toBe(false);
    expect(asked).toEqual(["stop"]);
  });

  it("joins a stop asked while the toggle stops, rather than stopping twice", async () => {
    const { service, asked } = await mockRecording();

    await flowResult(service.start());
    await Promise.all([flowResult(service.toggle()), flowResult(service.stop())]);

    expect(service.isRecording).toBe(false);
    expect(asked).toEqual(["stop"]);
  });

  it("starts nothing where this machine cannot record", async () => {
    const { service } = await mockRecording(OUTCOME, false);

    expect(service.canStart).toBe(false);
    expect(await flowResult(service.start())).toBeNull();
    expect(service.isRecording).toBe(false);
  });

  it("deletes the last recording for good, its toast gone with it", async () => {
    const { service, asked } = await mockRecording();

    await flowResult(service.start());
    await flowResult(service.stop());
    await flowResult(service.remove());

    expect(asked).toEqual(["stop", `delete ${FILE}`]);
    expect(service.outcome).toBeNull();
  });
});
