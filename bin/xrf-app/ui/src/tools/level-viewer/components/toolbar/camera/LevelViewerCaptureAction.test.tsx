import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { mockLevelViewContainer, renderLevelView } from "@/core/level/fixtures/level-container";
import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { EToolId } from "@/core/routing/tool";
import { mockToolScope } from "@/fixtures/utils/container";
import { LevelViewerCaptureActions } from "@/tools/level-viewer/components/capture/LevelViewerCaptureActions";
import { LevelViewerCaptureAction } from "@/tools/level-viewer/components/toolbar/camera/LevelViewerCaptureAction";
import { ELevelCaptureMode } from "@/tools/level-viewer/lib/capture/level-capture";
import { LevelViewerCaptureService } from "@/tools/level-viewer/services/level-viewer-capture";
import { LevelViewerCaptureResultService } from "@/tools/level-viewer/services/level-viewer-capture-result";
import { LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";
import { LevelViewerScreenshotService } from "@/tools/level-viewer/services/level-viewer-screenshot";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";

function renderCaptureAction(): ReturnType<typeof renderLevelView> {
  jest.spyOn(LevelViewerSnapshotService.prototype, "level", "get").mockReturnValue("levels/zaton");
  jest.spyOn(LevelViewerCaptureService.prototype, "level", "get").mockReturnValue(mockSelectedLevelDescription());

  return renderLevelView(
    <>
      <LevelViewerCaptureAction />
      <LevelViewerCaptureActions />
    </>,
    mockLevelViewContainer([
      ...mockToolScope(EToolId.LEVEL_VIEWER),
      LevelViewerCaptureService,
      LevelViewerCaptureResultService,
      LevelViewerRecordingService,
      LevelViewerScreenshotService,
      LevelViewerSnapshotService,
    ])
  );
}

function isOff(item: HTMLElement): boolean {
  return item.getAttribute("aria-disabled") === "true";
}

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerCaptureAction", () => {
  it("lists the captures on a click, and opens the one chosen with the list closed", async () => {
    const { container, getByRole, findByRole, queryByRole } = renderCaptureAction();
    const service: LevelViewerCaptureService = container.get(LevelViewerCaptureService);

    await userEvent.click(getByRole("button", { name: "Capture" }));

    const menu: HTMLElement = await findByRole("menu", { name: "Capture" });
    const items: Array<HTMLElement> = within(menu).getAllByRole("menuitem");

    expect(items.map((item: HTMLElement) => item.textContent)).toEqual([
      "Take screenshot",
      "Record",
      "Capture map…",
      "Capture cubemap…",
    ]);

    await userEvent.click(items[3] as HTMLElement);

    await waitFor(() => expect(queryByRole("menu")).not.toBeInTheDocument());
    expect(service.isOpen).toBe(true);
    expect(service.mode).toBe(ELevelCaptureMode.CUBEMAP);
    expect(getByRole("dialog", { name: "Capture" })).toBeInTheDocument();
  });

  it("offers only to stop the recording while one runs", async () => {
    jest.spyOn(LevelViewerRecordingService.prototype, "isRecording", "get").mockReturnValue(true);

    const { getByRole, findByRole } = renderCaptureAction();

    await userEvent.click(getByRole("button", { name: "Capture" }));

    const menu: HTMLElement = await findByRole("menu", { name: "Capture" });
    const items: Array<HTMLElement> = within(menu).getAllByRole("menuitem");

    expect(items.map((item: HTMLElement) => [item.textContent, isOff(item)])).toEqual([
      ["Take screenshot", true],
      ["Stop recording", false],
      ["Capture map…", true],
      ["Capture cubemap…", true],
    ]);
  });
});
