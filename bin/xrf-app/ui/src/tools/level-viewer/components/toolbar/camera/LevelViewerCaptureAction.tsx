import { useInjection } from "@wirestate/react";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FiberManualRecordIcon as RecordIcon } from "@xrf/ui/icons/fiber-manual-record";
import { PhotoCameraOutlinedIcon as PhotoCameraIcon } from "@xrf/ui/icons/photo-camera-outlined";
import { MenuList } from "@xrf/ui/material/menu";
import { IMenubarItem, MenubarRow } from "@xrf/ui/shell/menubar";
import { ReactElement, useCallback } from "react";

import { IKeybindCommand, KeybindCommandsService } from "@/core/commands";
import { KeymapService } from "@/core/keybinds";
import { toCommandMenubarItem } from "@/core/shell/menu/components/Menubar";
import {
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
  RECORD_LEVEL_KEYBIND_COMMAND,
  TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND,
} from "@/tools/level-viewer/commands";
import { LevelViewerRecordingReadout } from "@/tools/level-viewer/lib/recording/level-viewer-recording-readout";
import { LevelViewerCaptureService } from "@/tools/level-viewer/services/level-viewer-capture";
import { LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";
import { LevelViewerScreenshotService } from "@/tools/level-viewer/services/level-viewer-screenshot";

/** What the capture's menu offers, in its order: a screenshot, the recording, a level map and a cubemap. */
const CAPTURES: ReadonlyArray<IKeybindCommand> = [
  TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND,
  RECORD_LEVEL_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
];

/**
 * Lists what the level viewer captures on a click: a screenshot, the recording, a level map or a cubemap. While a
 * recording runs the others are off and Record stops it, its dot red; lit while one is taken or a recording runs, its
 * tooltip then the recording's time and drops.
 */
export function LevelViewerCaptureAction({
  "data-testid": dataTestId = "level-viewer-capture-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);
  const recordingService: LevelViewerRecordingService = useInjection(LevelViewerRecordingService);
  const commandsService: KeybindCommandsService = useInjection(KeybindCommandsService);
  const keymapService: KeymapService = useInjection(KeymapService);
  const isRecording: boolean = recordingService.isRecording;
  const isBusy: boolean = screenshotService.isTaking || captureService.isCapturing || isRecording;

  const renderMenu = useCallback(
    (close: () => void): ReactElement => {
      function onChoose(item: IMenubarItem): void {
        close();
        item.onSelect();
      }

      function toItem(command: IKeybindCommand): IMenubarItem {
        const item: IMenubarItem = toCommandMenubarItem(command, commandsService, keymapService);

        return command === RECORD_LEVEL_KEYBIND_COMMAND && isRecording
          ? { ...item, label: "Stop recording", icon: <RecordIcon className={"text-error"} /> }
          : item;
      }

      return (
        <MenuList className={"py-1"} aria-label={"Capture"} autoFocus={true} dense={true} variant={"menu"}>
          {CAPTURES.map((command: IKeybindCommand) => (
            <MenubarRow key={command.id} item={toItem(command)} onChoose={onChoose} />
          ))}
        </MenuList>
      );
    },
    [commandsService, isRecording, keymapService]
  );

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Capture"}
      description={
        isRecording
          ? LevelViewerRecordingReadout.describeProgress(recordingService.progress)
          : isBusy
            ? "Capturing…"
            : "Take a screenshot, record the viewport with its sound, or capture a level map or a cubemap"
      }
      icon={<PhotoCameraIcon />}
      isActive={isBusy}
      isDisabled={!screenshotService.canTake && !isBusy}
    >
      {renderMenu}
    </EditorPopoverAction>
  );
}
