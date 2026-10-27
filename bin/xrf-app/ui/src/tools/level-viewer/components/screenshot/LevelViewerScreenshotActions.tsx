import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { useShellMenus } from "@/core/shell/contributions";
import { IMenuContribution } from "@/core/shell/menu/lib";
import { TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND } from "@/tools/level-viewer/commands";
import { LevelViewerScreenshotDialog } from "@/tools/level-viewer/components/screenshot/LevelViewerScreenshotDialog";
import { ILevelViewerScreenshot } from "@/tools/level-viewer/lib/screenshot/level-viewer-screenshot";
import { LEVEL_VIEWER_FILE_MENU_EXTENSION } from "@/tools/level-viewer/menus";
import { LevelViewerScreenshotService } from "@/tools/level-viewer/services/level-viewer-screenshot";

/**
 * The level viewer's File › Take screenshot and the last screenshot's dialog; what fails with one is noticed.
 */
export function LevelViewerScreenshotActions(): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);
  const shot: Nullable<ILevelViewerScreenshot> = screenshotService.shot.value;

  useShellMenus(
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
