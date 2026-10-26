import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import {
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
} from "@/applications/level-viewer/commands";
import { LevelViewerCaptureDialog } from "@/applications/level-viewer/components/capture/LevelViewerCaptureDialog";
import { LEVEL_VIEWER_FILE_MENU_EXTENSION } from "@/applications/level-viewer/menus";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/capture";
import { useEditorMenus } from "@/core/shell/editor-shell";
import { IMenuContribution } from "@/core/shell/menu/lib";

/**
 * The level viewer's File › Capture map and Capture cubemap, after Take screenshot, and the capture's dialog while it is
 * open.
 */
export function LevelViewerCaptureActions(): Nullable<ReactElement> {
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

  return captureService.isOpen ? <LevelViewerCaptureDialog /> : null;
}
