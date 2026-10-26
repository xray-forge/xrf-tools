import { default as PhotoCameraIcon } from "@mui/icons-material/PhotoCameraOutlined";
import { MenuList } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import {
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
  TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND,
} from "@/applications/level-viewer/commands";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/capture";
import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/screenshot";
import { IKeybindCommand, KeybindCommandsService } from "@/core/commands";
import { KeymapService } from "@/core/keybinds";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { IMenubarItem, MenubarRow, toCommandMenubarItem } from "@/core/shell/menu/components/Menubar";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What the capture's menu offers, in its order. */
const CAPTURES: ReadonlyArray<IKeybindCommand> = [
  TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
];

/**
 * Lists what the level viewer captures, a screenshot, a level map or a cubemap, on a click; lit while one is taken.
 */
export function LevelViewerCaptureAction({
  "data-testid": dataTestId = "level-viewer-capture-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);
  const commandsService: KeybindCommandsService = useInjection(KeybindCommandsService);
  const keymapService: KeymapService = useInjection(KeymapService);
  const isBusy: boolean = screenshotService.isTaking || captureService.isCapturing;

  const renderMenu = useCallback(
    (close: () => void): ReactElement => {
      function onChoose(item: IMenubarItem): void {
        close();
        item.onSelect();
      }

      return (
        <MenuList className={"py-1"} aria-label={"Capture"} autoFocus={true} dense={true} variant={"menu"}>
          {CAPTURES.map((command: IKeybindCommand) => (
            <MenubarRow
              key={command.id}
              item={toCommandMenubarItem(command, commandsService, keymapService)}
              onChoose={onChoose}
            />
          ))}
        </MenuList>
      );
    },
    [commandsService, keymapService]
  );

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Capture"}
      description={isBusy ? "Capturing…" : "Take a screenshot, or capture a level map or a cubemap"}
      icon={<PhotoCameraIcon />}
      isActive={isBusy}
      isDisabled={!screenshotService.canTake && !isBusy}
    >
      {renderMenu}
    </EditorPopoverAction>
  );
}
