import { ReactElement } from "react";

import { useShellMenus } from "@/core/shell/contributions";
import { EMenuEntryKind, IMenuContribution } from "@/core/shell/menu/lib";
import { RECORD_LEVEL_KEYBIND_COMMAND } from "@/tools/level-viewer/commands";
import { LEVEL_VIEWER_FILE_MENU_EXTENSION } from "@/tools/level-viewer/menus";

/** The level viewer's File › Record, mounted after the screenshot's actions and before the captures' to follow it. */
export function LevelViewerRecordingActions(): ReactElement {
  useShellMenus(
    (): Array<IMenuContribution> => [
      {
        section: LEVEL_VIEWER_FILE_MENU_EXTENSION.sections.screenshot,
        entries: [{ kind: EMenuEntryKind.COMMAND, command: RECORD_LEVEL_KEYBIND_COMMAND }],
      },
    ],
    []
  );

  return <></>;
}
