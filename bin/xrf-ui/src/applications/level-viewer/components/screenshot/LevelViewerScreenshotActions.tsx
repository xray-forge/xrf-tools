import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND } from "@/applications/level-viewer/commands";
import { LevelViewerScreenshotDialog } from "@/applications/level-viewer/components/screenshot/LevelViewerScreenshotDialog";
import { LEVEL_VIEWER_FILE_MENU_EXTENSION } from "@/applications/level-viewer/menus";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/screenshot";
import { useEditorMenus } from "@/core/shell/editor-shell";
import { IMenuContribution } from "@/core/shell/menu/lib";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";

/**
 * The level viewer's File › Take screenshot, the last screenshot's dialog, and why something done with one failed.
 */
export function LevelViewerScreenshotActions(): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);

  useEditorMenus(
    (): Array<IMenuContribution> => [
      {
        section: LEVEL_VIEWER_FILE_MENU_EXTENSION.sections.screenshot,
        entries: [{ kind: "command", command: TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND }],
      },
    ],
    []
  );

  return (
    <>
      {screenshotService.isShown && screenshotService.shot ? (
        <LevelViewerScreenshotDialog shot={screenshotService.shot} />
      ) : null}

      <ConfirmDialog
        isOpen={screenshotService.failure !== null}
        title={"Screenshot"}
        description={screenshotService.failure}
        confirmLabel={"OK"}
        onConfirm={screenshotService.dismissFailure}
        onClose={screenshotService.dismissFailure}
      />
    </>
  );
}
