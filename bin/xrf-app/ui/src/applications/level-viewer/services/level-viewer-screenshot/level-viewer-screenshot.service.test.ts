import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { EventBus, WireEvent } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ELevelsBulkRoute } from "@xrf/ipc/commands/levels-bulk";
import { mockIpcBulkRoutes, mockIpcCommands } from "@xrf/ipc/testing";
import { Nullable } from "@xrf/types";

import { ILevelViewerScreenshot } from "@/applications/level-viewer/lib/screenshot/level-viewer-screenshot";
import { LEVEL_VIEWER_SNAPSHOT_FORMAT } from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/level-viewer-screenshot";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/level-viewer-snapshot";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  noteLevelFrame,
} from "@/core/level/fixtures/level-container";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { clearLocalStorage } from "@/core/storage";
import { mockApplicationScope } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

const PATH: string = "appdata/level-viewer/screenshots/zaton-20261020-070509.png";
const REPORT_PATH: string = "appdata/level-viewer/screenshots/zaton-20261020-070509.json";

function mockScreenshots(): ILevelViewContainerDescriptor & {
  screenshots: LevelViewerScreenshotService;
  raised: Array<INotificationPayload>;
} {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer([
    ...mockApplicationScope(EApplicationId.LEVEL_VIEWER),
    LevelViewerScreenshotService,
    LevelViewerSnapshotService,
  ]);
  const raised: Array<INotificationPayload> = [];

  descriptor.container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );
  jest.spyOn(LevelViewerSnapshotService.prototype, "level", "get").mockReturnValue("levels/zaton");
  jest.spyOn(descriptor.render, "saveScreenshot").mockResolvedValue(true);

  return { ...descriptor, raised, screenshots: descriptor.container.get(LevelViewerScreenshotService) };
}

beforeEach(() => {
  clearLocalStorage();
  // A PNG's signature and header, 1920 by 1080.
  mockIpcBulkRoutes({
    [ELevelsBulkRoute.READ_SCREENSHOT]: Uint8Array.from([
      137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 7, 128, 0, 0, 4, 56,
    ]).buffer,
  });
  URL.createObjectURL = jest.fn(() => "blob:screenshot");
  URL.revokeObjectURL = jest.fn();
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerScreenshotService", () => {
  it("saves the frame at once and its report with the timed frame, timed for the shot alone", async () => {
    const saved: Array<Record<string, unknown>> = [];

    mockIpcCommands({
      [ELevelsCommand.FIND_SCREENSHOT_PATH]: PATH,
      [ELevelsCommand.WRITE_SCREENSHOT_REPORT]: (args?: Record<string, unknown>) => {
        saved.push(args ?? {});

        return REPORT_PATH;
      },
    });

    const { screenshots, render, renderSettings } = mockScreenshots();
    const taking: Promise<Nullable<ILevelViewerScreenshot>> = flowResult(screenshots.take());

    expect(renderSettings.renderSettings.isGpuTimed).toBe(true);
    expect(renderSettings.isGpuTimed).toBe(false);
    expect(screenshots.canTake).toBe(false);

    // Asked again while it is taken, it is the same screenshot.
    const again: Promise<Nullable<ILevelViewerScreenshot>> = flowResult(screenshots.take());

    await waitFor(() => expect(screenshots.shot.value).not.toBeNull());

    expect(screenshots.shot.value).toMatchObject({
      height: 1080,
      name: "zaton-20261020-070509.png",
      path: PATH,
      width: 1920,
    });

    noteLevelFrame(render, { framesPerSecond: 60, isGpuTimed: true, passes: [{ gpuTime: 1, name: "a" }] });
    noteLevelFrame(render, { framesPerSecond: 61, isGpuTimed: true, passes: [{ gpuTime: 1, name: "a" }] });

    const shot: Nullable<ILevelViewerScreenshot> = await taking;
    const report: Record<string, unknown> = JSON.parse(String(saved[0]?.text));

    expect(saved[0]?.path).toBe(PATH);
    expect(report).toMatchObject({
      format: LEVEL_VIEWER_SNAPSHOT_FORMAT,
      frame: { framesPerSecond: 61, isGpuTimed: true },
      screenshot: { file: "zaton-20261020-070509.png", height: 1080, isTimingSwitched: true, width: 1920 },
    });
    expect(screenshots.report.value?.frame.framesPerSecond).toBe(61);
    expect(screenshots.report.value?.path).toBe(REPORT_PATH);
    await expect(again).resolves.toBe(shot);
    expect(renderSettings.renderSettings.isGpuTimed).toBe(false);
    expect(screenshots.canTake).toBe(true);
  });

  it("reports the frame as it stands where the passes are timed already", async () => {
    const saved: Array<Record<string, unknown>> = [];

    mockIpcCommands({
      [ELevelsCommand.FIND_SCREENSHOT_PATH]: PATH,
      [ELevelsCommand.WRITE_SCREENSHOT_REPORT]: (args?: Record<string, unknown>) => {
        saved.push(args ?? {});

        return REPORT_PATH;
      },
    });

    const { screenshots, render, renderSettings } = mockScreenshots();

    renderSettings.setGpuTimed(true);
    noteLevelFrame(render, { framesPerSecond: 90, isGpuTimed: true });

    await flowResult(screenshots.take());

    expect(JSON.parse(String(saved[0]?.text))).toMatchObject({
      frame: { framesPerSecond: 90 },
      screenshot: { isTimingSwitched: false },
    });
    expect(renderSettings.isGpuTimed).toBe(true);
  });

  it("keeps the screenshot when its report cannot be written, and says so apart", async () => {
    mockIpcCommands({
      [ELevelsCommand.FIND_SCREENSHOT_PATH]: PATH,
      [ELevelsCommand.WRITE_SCREENSHOT_REPORT]: () => {
        throw new Error("disk full");
      },
    });

    const { raised, screenshots, render, renderSettings } = mockScreenshots();

    jest.spyOn(screenshots.log, "error").mockImplementation(() => undefined);
    renderSettings.setGpuTimed(true);
    noteLevelFrame(render, { isGpuTimed: true });

    const shot: Nullable<ILevelViewerScreenshot> = await flowResult(screenshots.take());

    expect(shot?.path).toBe(PATH);
    expect(screenshots.shot.value).toBe(shot);
    expect(screenshots.report.value).toBeNull();
    expect(screenshots.report.error?.message).toBe("disk full");
    expect(raised).toEqual([
      expect.objectContaining({
        details: `${PATH}\ndisk full`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.LEVEL_VIEWER,
        title: "Could not write the screenshot's report",
      }),
    ]);
  });

  // A failed screenshot leaves the one before it shown, its report with it.
  it("says why a screenshot failed from the viewer, keeping the one before it", async () => {
    mockIpcCommands({
      [ELevelsCommand.FIND_SCREENSHOT_PATH]: PATH,
      [ELevelsCommand.WRITE_SCREENSHOT_REPORT]: REPORT_PATH,
    });

    const { raised, screenshots, render, renderSettings } = mockScreenshots();

    jest.spyOn(screenshots.log, "error").mockImplementation(() => undefined);
    renderSettings.setGpuTimed(true);
    noteLevelFrame(render, { isGpuTimed: true });

    const first: Nullable<ILevelViewerScreenshot> = await flowResult(screenshots.take());

    jest.mocked(render.saveScreenshot).mockResolvedValue(false);

    await expect(flowResult(screenshots.take())).resolves.toBeNull();
    expect(screenshots.shot.value).toBe(first);
    expect(screenshots.report.value?.path).toBe(REPORT_PATH);
    expect(screenshots.canTake).toBe(true);
    expect(raised).toEqual([
      expect.objectContaining({
        details: "levels/zaton\nThe level viewer has no viewport attached",
        source: EApplicationId.LEVEL_VIEWER,
        title: "Could not take a screenshot",
      }),
    ]);
  });

  it("holds no screenshot taken while the tool was left", async () => {
    const path = deferred<string>();

    mockIpcCommands({ [ELevelsCommand.FIND_SCREENSHOT_PATH]: () => path.promise });

    const { container, screenshots, render, renderSettings } = mockScreenshots();

    renderSettings.setGpuTimed(true);
    noteLevelFrame(render, { isGpuTimed: true });

    const taking: Promise<Nullable<ILevelViewerScreenshot>> = flowResult(screenshots.take());

    container.unbind(LevelViewerScreenshotService);
    path.resolve(PATH);

    await expect(taking).resolves.toBeUndefined();
    expect(screenshots.shot.value).toBeNull();
    expect(screenshots.isTaking).toBe(false);
  });

  it("deletes the screenshot through the backend and forgets it", async () => {
    const deleted: Array<string> = [];

    mockIpcCommands({
      [ELevelsCommand.DELETE_SCREENSHOT]: ({ path }) => {
        deleted.push(path);

        return null;
      },
      [ELevelsCommand.FIND_SCREENSHOT_PATH]: PATH,
    });

    const { screenshots, render, renderSettings } = mockScreenshots();

    renderSettings.setGpuTimed(true);
    noteLevelFrame(render, { isGpuTimed: true });
    await flowResult(screenshots.take());
    await screenshots.remove();

    expect(deleted).toEqual([PATH]);
    expect(screenshots.shot.value).toBeNull();
    expect(screenshots.report.value).toBeNull();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:screenshot");
  });
});
