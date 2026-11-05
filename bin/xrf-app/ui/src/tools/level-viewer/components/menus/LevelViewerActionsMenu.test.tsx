import { afterEach, describe, expect, it, jest } from "@jest/globals";

import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { EToolId } from "@/core/routing/tool";
import { EMenuEntryKind, IMenuContribution, isSameMenuSection, TMenuEntry } from "@/core/shell/menu/lib";
import { ShellContributionsService } from "@/core/shell/services/shell-contributions";
import { mockToolScope } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { LevelViewerActionsMenu } from "@/tools/level-viewer/components/menus/LevelViewerActionsMenu";
import { LEVEL_VIEWER_ACTIONS_MENU } from "@/tools/level-viewer/menus";
import { LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";

function renderCaptureRows(): Array<Array<string>> {
  const { container } = mockLevelViewContainer([
    ...mockToolScope(EToolId.LEVEL_VIEWER),
    LevelViewerRecordingService,
    LevelViewerSnapshotService,
  ]);

  // The tool open is read off the route, which is what keys what it publishes.
  renderWithProviders(<LevelViewerActionsMenu />, { container, route: "/level-viewer" });

  const [contribution] = container
    .get(ShellContributionsService)
    .menus.get(EToolId.LEVEL_VIEWER)
    .filter((it: IMenuContribution) => isSameMenuSection(it.section, LEVEL_VIEWER_ACTIONS_MENU.sections.capture));
  const [capture] = contribution?.entries ?? [];

  if (capture?.kind !== EMenuEntryKind.SUBMENU) {
    throw new Error("Actions has no Capture submenu");
  }

  return capture.sections.map((run: ReadonlyArray<TMenuEntry>) =>
    run.map((entry: TMenuEntry) =>
      entry.kind === EMenuEntryKind.COMMAND ? (entry.label ?? entry.command.label) : entry.kind
    )
  );
}

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerActionsMenu", () => {
  it("offers the captures under Actions › Capture, what the frame shows apart from the dialogs", () => {
    expect(renderCaptureRows()).toEqual([
      ["Take screenshot", "Record"],
      ["Capture map…", "Capture cubemap…"],
    ]);
  });

  it("offers to stop the recording while one runs", () => {
    jest.spyOn(LevelViewerRecordingService.prototype, "isRecording", "get").mockReturnValue(true);

    expect(renderCaptureRows()).toEqual([
      ["Take screenshot", "Stop recording"],
      ["Capture map…", "Capture cubemap…"],
    ]);
  });
});
