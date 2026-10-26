import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelViewerCaptureActions } from "@/applications/level-viewer/components/capture/LevelViewerCaptureActions";
import { LevelViewerCaptureAction } from "@/applications/level-viewer/components/toolbar/camera/LevelViewerCaptureAction";
import { ELevelCaptureMode } from "@/applications/level-viewer/lib/capture/level-capture";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/capture";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/screenshot";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/snapshot";
import { mockLevelViewContainer, renderLevelView } from "@/core/level/fixtures/level-container";
import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerCaptureAction", () => {
  it("lists the captures on a click, and opens the one chosen with the list closed", async () => {
    jest.spyOn(LevelViewerSnapshotService.prototype, "level", "get").mockReturnValue("levels/zaton");
    jest.spyOn(LevelViewerCaptureService.prototype, "level", "get").mockReturnValue(mockSelectedLevelDescription());

    const { container, getByRole, findByRole, queryByRole } = renderLevelView(
      <>
        <LevelViewerCaptureAction />
        <LevelViewerCaptureActions />
      </>,
      mockLevelViewContainer([LevelViewerCaptureService, LevelViewerScreenshotService, LevelViewerSnapshotService])
    );
    const service: LevelViewerCaptureService = container.get(LevelViewerCaptureService);

    await userEvent.click(getByRole("button", { name: "Capture" }));

    const items: Array<HTMLElement> = within(await findByRole("menu", { name: "Capture" })).getAllByRole("menuitem");

    expect(items.map((item: HTMLElement) => item.textContent)).toEqual([
      "Take screenshot",
      "Capture map…",
      "Capture cubemap…",
    ]);

    await userEvent.click(items[2] as HTMLElement);

    await waitFor(() => expect(queryByRole("menu")).not.toBeInTheDocument());
    expect(service.isOpen).toBe(true);
    expect(service.mode).toBe(ELevelCaptureMode.CUBEMAP);
    expect(getByRole("dialog", { name: "Capture" })).toBeInTheDocument();
  });
});
