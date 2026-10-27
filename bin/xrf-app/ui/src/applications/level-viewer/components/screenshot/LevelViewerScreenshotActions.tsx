import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND } from "@/applications/level-viewer/commands";
import { LevelViewerScreenshotDialog } from "@/applications/level-viewer/components/screenshot/LevelViewerScreenshotDialog";
import { ILevelViewerScreenshot } from "@/applications/level-viewer/lib/screenshot/level-viewer-screenshot";
import { LEVEL_VIEWER_FILE_MENU_EXTENSION } from "@/applications/level-viewer/menus";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/level-viewer-screenshot";
import { useEditorMenus } from "@/core/shell/editor-shell";
import { IMenuContribution } from "@/core/shell/menu/lib";

/**
 * The level viewer's File › Take screenshot and the last screenshot's dialog; what fails with one is noticed.
 */
export function LevelViewerScreenshotActions(): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);
  const shot: Nullable<ILevelViewerScreenshot> = screenshotService.shot.value;

  useEditorMenus(
    (): Array<IMenuContribution> => [
      {
        section: LEVEL_VIEWER_FILE_MENU_EXTENSION.sections.screenshot,
        entries: [{ kind: "command", command: TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND }],
      },
    ],
    []
  );

  return screenshotService.isShown && shot ? <LevelViewerScreenshotDialog shot={shot} /> : <></>;
}
