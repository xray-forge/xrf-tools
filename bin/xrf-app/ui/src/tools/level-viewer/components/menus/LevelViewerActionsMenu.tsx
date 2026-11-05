import { useInjection } from "@wirestate/react";
import { FiberManualRecordIcon as RecordIcon } from "@xrf/ui/icons/fiber-manual-record";
import { PhotoCameraOutlinedIcon as PhotoCameraIcon } from "@xrf/ui/icons/photo-camera-outlined";
import { ReactElement } from "react";

import { GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND } from "@/core/level/commands";
import { useShellMenus } from "@/core/shell/contributions";
import { EMenuEntryKind, IMenuContribution } from "@/core/shell/menu/lib";
import {
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
  RECORD_LEVEL_KEYBIND_COMMAND,
  TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND,
} from "@/tools/level-viewer/commands";
import { LEVEL_VIEWER_ACTIONS_MENU } from "@/tools/level-viewer/menus";
import { LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";

/**
 * The level viewer's Actions menu: Capture ▸ a screenshot, the recording, a level map and a cubemap, Record turned to
 * Stop recording, its dot red, while one runs; then Go to coordinates.
 */
export function LevelViewerActionsMenu(): ReactElement {
  const recordingService: LevelViewerRecordingService = useInjection(LevelViewerRecordingService);
  const isRecording: boolean = recordingService.isRecording;

  useShellMenus(
    (): Array<IMenuContribution> => [
      {
        section: LEVEL_VIEWER_ACTIONS_MENU.sections.capture,
        entries: [
          {
            kind: EMenuEntryKind.SUBMENU,
            id: "level-viewer/capture",
            label: "Capture",
            icon: <PhotoCameraIcon />,
            sections: [
              [
                { kind: EMenuEntryKind.COMMAND, command: TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND },
                isRecording
                  ? {
                      kind: EMenuEntryKind.COMMAND,
                      command: RECORD_LEVEL_KEYBIND_COMMAND,
                      label: "Stop recording",
                      icon: <RecordIcon className={"text-error"} />,
                    }
                  : { kind: EMenuEntryKind.COMMAND, command: RECORD_LEVEL_KEYBIND_COMMAND },
              ],
              [
                { kind: EMenuEntryKind.COMMAND, command: CAPTURE_LEVEL_MAP_KEYBIND_COMMAND },
                { kind: EMenuEntryKind.COMMAND, command: CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND },
              ],
            ],
          },
        ],
      },
      {
        section: LEVEL_VIEWER_ACTIONS_MENU.sections.camera,
        entries: [{ kind: EMenuEntryKind.COMMAND, command: GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND }],
      },
    ],
    [isRecording]
  );

  return <></>;
}
