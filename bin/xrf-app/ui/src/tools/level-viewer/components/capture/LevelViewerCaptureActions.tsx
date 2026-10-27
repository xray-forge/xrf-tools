import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { useEditorMenus } from "@/core/shell/editor-shell";
import { IMenuContribution } from "@/core/shell/menu/lib";
import {
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
} from "@/tools/level-viewer/commands";
import { LevelViewerCaptureDialog } from "@/tools/level-viewer/components/capture/LevelViewerCaptureDialog";
import { LEVEL_VIEWER_FILE_MENU_EXTENSION } from "@/tools/level-viewer/menus";
import { LevelViewerCaptureService } from "@/tools/level-viewer/services/level-viewer-capture";

/**
 * The level viewer's File › Capture map and Capture cubemap, after Take screenshot, and the capture's dialog while it is
 * open.
 */
export function LevelViewerCaptureActions(): ReactElement {
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);

  useEditorMenus(
    (): Array<IMenuContribution> => [
      {
        section: LEVEL_VIEWER_FILE_MENU_EXTENSION.sections.screenshot,
        entries: [
          { kind: "command", command: CAPTURE_LEVEL_MAP_KEYBIND_COMMAND },
          { kind: "command", command: CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND },
        ],
      },
    ],
    []
  );

  return captureService.isOpen ? <LevelViewerCaptureDialog /> : <></>;
}
