import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Nullable } from "@xrf/types";

import { ILevelViewerScreenshot } from "@/applications/level-viewer/lib/screenshot/level-viewer-screenshot";
import { LEVEL_VIEWER_SNAPSHOT_FORMAT } from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/screenshot";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/snapshot";
import { clearLocalStorage } from "@/core/storage";
import { setMockBulkResponses } from "@/fixtures/mocks/bulk.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  noteLevelFrame,
} from "@/fixtures/utils/level-container";

const PATH: string = "appdata/level-viewer/screenshots/zaton-20261020-070509.png";
const REPORT_PATH: string = "appdata/level-viewer/screenshots/zaton-20261020-070509.json";

function mockScreenshots(): ILevelViewContainerDescriptor & { screenshots: LevelViewerScreenshotService } {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer([
    LevelViewerScreenshotService,
    LevelViewerSnapshotService,
  ]);

  jest.spyOn(LevelViewerSnapshotService.prototype, "level", "get").mockReturnValue("levels/zaton");
  jest.spyOn(descriptor.render, "saveScreenshot").mockResolvedValue(true);

  return { ...descriptor, screenshots: descriptor.container.get(LevelViewerScreenshotService) };
}

beforeEach(() => {
  clearLocalStorage();
  // A PNG's signature and header, 1920 by 1080.
  setMockBulkResponses({
    ["levels/read_screenshot"]: Uint8Array.from([
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

    setMockInvokeResponses({
      ["plugin:levels|find_screenshot_path"]: PATH,
      ["plugin:levels|write_screenshot_report"]: (args?: Record<string, unknown>) => {
        saved.push(args ?? {});

        return REPORT_PATH;
      },
    });

    const { screenshots, render, renderSettings } = mockScreenshots();
    const taking: Promise<Nullable<ILevelViewerScreenshot>> = screenshots.take();

    expect(renderSettings.renderSettings.isGpuTimed).toBe(true);
    expect(renderSettings.isGpuTimed).toBe(false);
    expect(screenshots.canTake).toBe(false);
    await expect(screenshots.take()).resolves.toBeNull();

    await waitFor(() => expect(screenshots.shot).not.toBeNull());

    expect(screenshots.shot).toMatchObject({
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
    expect(shot?.frame?.framesPerSecond).toBe(61);
    expect(shot?.reportPath).toBe(REPORT_PATH);
    expect(renderSettings.renderSettings.isGpuTimed).toBe(false);
    expect(screenshots.canTake).toBe(true);
  });

  it("reports the frame as it stands where the passes are timed already", async () => {
    const saved: Array<Record<string, unknown>> = [];

    setMockInvokeResponses({
      ["plugin:levels|find_screenshot_path"]: PATH,
      ["plugin:levels|write_screenshot_report"]: (args?: Record<string, unknown>) => {
        saved.push(args ?? {});

        return REPORT_PATH;
      },
    });

    const { screenshots, render, renderSettings } = mockScreenshots();

    renderSettings.setGpuTimed(true);
    noteLevelFrame(render, { framesPerSecond: 90, isGpuTimed: true });

    await screenshots.take();

    expect(JSON.parse(String(saved[0]?.text))).toMatchObject({
      frame: { framesPerSecond: 90 },
      screenshot: { isTimingSwitched: false },
    });
    expect(renderSettings.isGpuTimed).toBe(true);
  });

  it("keeps the screenshot when its report cannot be written, and says so apart", async () => {
    setMockInvokeResponses({
      ["plugin:levels|find_screenshot_path"]: PATH,
      ["plugin:levels|write_screenshot_report"]: () => {
        throw new Error("disk full");
      },
    });

    const { screenshots, render, renderSettings } = mockScreenshots();

    renderSettings.setGpuTimed(true);
    noteLevelFrame(render, { isGpuTimed: true });

    const shot: Nullable<ILevelViewerScreenshot> = await screenshots.take();

    expect(shot?.path).toBe(PATH);
    expect(shot?.reportPath).toBeNull();
    expect(screenshots.failure).toContain("write the screenshot's report");
  });

  it("deletes the screenshot through the backend and forgets it", async () => {
    const deleted: Array<unknown> = [];

    setMockInvokeResponses({
      ["plugin:levels|delete_screenshot"]: (args?: Record<string, unknown>) => deleted.push(args?.path),
      ["plugin:levels|find_screenshot_path"]: PATH,
    });

    const { screenshots, render, renderSettings } = mockScreenshots();

    renderSettings.setGpuTimed(true);
    noteLevelFrame(render, { isGpuTimed: true });
    await screenshots.take();
    await screenshots.remove();

    expect(deleted).toEqual([PATH]);
    expect(screenshots.shot).toBeNull();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:screenshot");
  });
});
